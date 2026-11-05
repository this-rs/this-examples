//! Le branchement du modèle, côté **exemple**.
//!
//! Le framework apporte le contrat (`Completion`) et le prompt qui décrit sa
//! langue. Ici, on ne fait que choisir un endpoint et lire une clé — c'est le
//! travail d'une app, pas d'un framework.
//!
//! La clé ne quitte jamais le serveur : la page reçoit le programme, jamais le
//! secret. Un appel direct depuis le navigateur aurait exposé la clé à quiconque
//! ouvre l'inspecteur.

use this_module_harness::drafter::Completion;

/// Un endpoint OpenAI-compatible, configuré par l'environnement.
pub struct ChatCompletion {
    base_url: String,
    model: String,
    api_key: String,
    client: reqwest::Client,
}

impl ChatCompletion {
    /// Lit la configuration. Rend une erreur explicite plutôt qu'un appel qui
    /// échouerait plus tard sans dire pourquoi.
    pub fn from_env() -> Result<Self, String> {
        let api_key = std::env::var("DEEPSEEK_API_KEY")
            .map_err(|_| "DEEPSEEK_API_KEY absent de l'environnement du serveur".to_string())?;
        Ok(Self {
            base_url: std::env::var("MODEL_BASE_URL")
                .unwrap_or_else(|_| "https://api.deepseek.com".to_string()),
            model: std::env::var("MODEL_NAME").unwrap_or_else(|_| "deepseek-chat".to_string()),
            api_key,
            client: reqwest::Client::new(),
        })
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl Completion for ChatCompletion {
    async fn complete(&self, system: &str, request: &str) -> Result<String, String> {
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": request },
            ],
            "temperature": 0,
        });

        let response = self
            .client
            .post(format!(
                "{}/chat/completions",
                self.base_url.trim_end_matches('/')
            ))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|error| format!("appel au modèle impossible : {error}"))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|error| format!("réponse illisible : {error}"))?;

        if !status.is_success() {
            // On rend le statut et un extrait : un 401 (clé) et un 429 (quota) ne
            // se diagnostiquent pas de la même façon.
            return Err(format!(
                "le modèle a répondu {status} : {}",
                truncate(&text, 300)
            ));
        }

        let parsed: serde_json::Value =
            serde_json::from_str(&text).map_err(|error| format!("JSON invalide : {error}"))?;
        parsed["choices"][0]["message"]["content"]
            .as_str()
            .map(|content| content.to_string())
            .ok_or_else(|| format!("réponse sans contenu : {}", truncate(&text, 300)))
    }
}

fn truncate(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}
