//! Le branchement du modèle, côté **exemple**.
//!
//! Le framework apporte le contrat (`this_llm::Provider`, `Completion`) et les
//! prompts qui décrivent sa langue et ses descripteurs. Ici, on choisit un
//! endpoint, on lit une clé si elle existe, et on transporte. C'est le travail
//! d'une app, pas d'un framework.
//!
//! ## Stateless, et pas autre chose
//!
//! Cet endpoint sert à **générer** : même demande, même réponse. Il ne doit donc
//! pas être un modèle branché sur une mémoire (obrain ou autre) — y envoyer une
//! demande de module mêlerait la génération à un contexte qui n'a rien à voir, et
//! rendrait le résultat irreproductible. Un vLLM ou un `uzu` qui sert le modèle
//! nu convient exactement ; un `llama-server` adossé à une base, non.
//!
//! ## La configuration est tout l'exemple
//!
//! Rien n'est codé en dur : `LLM_BASE_URL` désigne le moteur, `LLM_MODEL` le
//! modèle (à défaut, le premier que le moteur annonce), `LLM_API_KEY` une clé
//! distante si le moteur en exige une. Le défaut vise vLLM sur son port habituel.

use this_llm::{Completion, Provider};
use this_module_harness::drafter::Drafter;
use this_module_harness::module_draft::ModuleDrafter;
use this_module_harness::Error as HarnessError;

/// Un endpoint OpenAI-compatible, configuré par l'environnement.
pub struct ChatCompletion {
    provider: Provider,
    client: reqwest::Client,
    /// Qwen et ses pareils dépensent le budget en réflexion avant de répondre.
    /// Éteinte par défaut : pour une démo, la latence compte plus que la méditation.
    thinking: bool,
    max_tokens: u32,
}

impl ChatCompletion {
    /// Lit la configuration, ou explique précisément quoi faire.
    pub async fn from_env() -> Result<Self, String> {
        let base_url = std::env::var("LLM_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8000/v1".to_string());
        let api_key = std::env::var("LLM_API_KEY").ok();

        let client = reqwest::Client::new();

        // Le modèle se demande au moteur plutôt que d'être deviné : un vLLM, un
        // uzu et un llama-server n'annoncent pas les mêmes identifiants.
        let model = match std::env::var("LLM_MODEL") {
            Ok(model) => model,
            Err(_) => first_model(&client, &base_url, api_key.as_deref())
                .await
                .map_err(|error| {
                    format!(
                        "aucun modèle annoncé par {base_url} ({error}). \
                         Lance un moteur stateless (vLLM : `vllm serve <modèle>` sur :8000, \
                         ou uzu) puis renseigne LLM_BASE_URL. N'utilise pas un endpoint \
                         adossé à obrain : le draft doit être reproductible."
                    )
                })?,
        };

        Ok(Self {
            provider: Provider::local(this_llm::Preset::OpenAiCompatible)
                .with_base_url(base_url)
                .with_model(model),
            client,
            thinking: std::env::var("LLM_THINKING").is_ok_and(|v| v == "1"),
            max_tokens: std::env::var("LLM_MAX_TOKENS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4096),
        })
    }

    /// Ce que l'app afficherait dans un écran de réglages — base et modèle, jamais la clé.
    pub fn describe(&self) -> String {
        format!("{} · {}", self.provider.base_url, self.provider.model)
    }

    async fn complete(
        &self,
        system: &str,
        request: &str,
        key: Option<&str>,
    ) -> Result<String, String> {
        let mut body = serde_json::json!({
            "model": self.provider.model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": request },
            ],
            "temperature": 0,
            "max_tokens": self.max_tokens,
        });
        if !self.thinking {
            // Accepté par llama.cpp et vLLM pour Qwen3 ; ignoré ailleurs.
            body["chat_template_kwargs"] = serde_json::json!({ "enable_thinking": false });
        }

        let mut call = self
            .client
            .post(format!(
                "{}/chat/completions",
                self.provider.base_url.trim_end_matches('/')
            ))
            .json(&body);
        if let Some(key) = key {
            call = call.bearer_auth(key);
        }

        let response = call.send().await.map_err(|error| {
            format!("moteur injoignable ({} ) : {error}", self.provider.base_url)
        })?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|error| format!("réponse illisible : {error}"))?;
        if !status.is_success() {
            return Err(format!(
                "le moteur a répondu {status} : {}",
                truncate(&text, 300)
            ));
        }

        let parsed: serde_json::Value =
            serde_json::from_str(&text).map_err(|error| format!("JSON invalide : {error}"))?;
        parsed["choices"][0]["message"]["content"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| format!("réponse sans contenu : {}", truncate(&text, 300)))
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl Completion for ChatCompletion {
    async fn complete(&self, system: &str, request: &str) -> Result<String, String> {
        // La clé est lue ici et ne quitte pas ce processus : elle ne sera ni
        // journalisée, ni renvoyée au client.
        let key = std::env::var("LLM_API_KEY").ok();
        ChatCompletion::complete(self, system, request, key.as_deref()).await
    }
}

/// Le premier modèle que le moteur annonce.
async fn first_model(
    client: &reqwest::Client,
    base_url: &str,
    key: Option<&str>,
) -> Result<String, String> {
    let mut call = client.get(format!("{}/models", base_url.trim_end_matches('/')));
    if let Some(key) = key {
        call = call.bearer_auth(key);
    }
    let response = call.send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("statut {}", response.status()));
    }
    let parsed: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    parsed["data"][0]["id"]
        .as_str()
        .or_else(|| parsed["models"][0]["model"].as_str())
        .map(str::to_string)
        .ok_or_else(|| "aucun modèle dans la réponse".to_string())
}

fn truncate(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}

/// Demande un **programme** au modèle.
pub async fn draft_program(prompt: &str) -> Result<String, HarnessError> {
    let completion = ChatCompletion::from_env()
        .await
        .map_err(HarnessError::Model)?;
    Drafter::new(completion).draft(prompt).await
}

/// Demande une **feature complète** au modèle, validée par le framework.
pub async fn draft_feature(
    prompt: &str,
) -> Result<this_descriptor::ModuleDescriptor, HarnessError> {
    let completion = ChatCompletion::from_env()
        .await
        .map_err(HarnessError::Model)?;
    ModuleDrafter::new(completion)
        .draft(prompt)
        .await
        .map(|draft| draft.descriptor)
}
