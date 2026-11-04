//! # `api` — le seul fichier qui sait sur quoi l'app tourne
//!
//! Le reste du front n'appelle jamais Tauri ni `fetch` : il appelle une **opération**
//! (`status`, `interface`, `records`, `save`, `order`…). Ce module choisit le transport
//! à l'exécution :
//!
//! - dans un webview Tauri (desktop **ou** mobile) → `invoke` sur une commande ;
//! - dans un navigateur → `fetch` sur `/api`, même origine que le bundle.
//!
//! Le même wasm porte les deux chemins : changer de surface ne recompile pas l'UI,
//! il change seulement la présence de `window.__TAURI_INTERNALS__`.

use std::collections::BTreeMap;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

// ---- Plateforme -----------------------------------------------------------------------

pub use app_kit::Change;
pub use app_kit::Surface;
pub use this_descriptor::{
    Capabilities, Gap, ModuleDescriptor, Operation, PermissionDecision, Requirement, Session,
};
pub use ui_shell::Module;

/// La surface courante, lue à l'exécution (pas à la compilation).
///
/// La définition (`Surface`, ses libellés, son transport) vit dans `app-kit` : l'UI et
/// les coquilles partagent la même, il n'y en a pas une deuxième ici.
pub fn platform() -> Surface {
    let Some(win) = web_sys::window() else {
        return Surface::Web;
    };
    let tauri = js_sys::Reflect::get(&win, &JsValue::from_str("__TAURI_INTERNALS__"))
        .map(|v| !v.is_undefined() && !v.is_null())
        .unwrap_or(false);
    if !tauri {
        return Surface::Web;
    }
    let ua = win
        .navigator()
        .user_agent()
        .unwrap_or_default()
        .to_lowercase();
    if ua.contains("android") || ua.contains("iphone") || ua.contains("ipad") {
        Surface::Mobile
    } else {
        Surface::Desktop
    }
}

// ---- Types miroirs (le sous-ensemble que l'écran utilise) ------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct Status {
    pub loaded: bool,
}

/// Instantané de tous les enregistrements, par entité.
pub type Snapshot = BTreeMap<String, Vec<Value>>;

/// Une ligne de commande envoyée à la surface.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LineIn {
    pub product_id: String,
    pub qty: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Interface {
    /// Le contrat **typé** : il porte les politiques (capacités) et de quoi construire le
    /// modèle d'affichage. Une seule lecture, plus d'interprétation de JSON.
    pub descriptor: ModuleDescriptor,
    #[serde(default)]
    pub bundles: Vec<Value>,
}

impl Interface {
    /// Les langues réellement livrées par le module (un bundle par locale).
    pub fn locales(&self) -> Vec<String> {
        self.bundles
            .iter()
            .filter_map(|b| b["locale"].as_str().map(str::to_string))
            .collect()
    }

    /// Le modèle d'affichage : entités, champs typés, liens. Transposable à tout module.
    pub fn model(&self) -> Module {
        Module::of(&self.descriptor, self.locales())
    }
}

// ---- Transport : un appel décrit une fois, deux surfaces ----------

/// Une opération décrite **une seule fois** pour les deux surfaces.
///
/// C'est la table de correspondance du connecteur : à gauche le nom de la commande Tauri et
/// ses arguments, à droite la méthode HTTP et sa route. Ajouter une opération, c'est écrire
/// une ligne ici — plus deux branches à garder synchronisées.
struct Call {
    command: &'static str,
    args: Value,
    method: &'static str,
    path: String,
    body: Option<Value>,
}

impl Call {
    fn get(command: &'static str, path: &str, args: Value) -> Self {
        Self {
            command,
            args,
            method: "GET",
            path: path.to_string(),
            body: None,
        }
    }

    fn post(command: &'static str, path: &str, args: Value, body: Value) -> Self {
        Self {
            command,
            args,
            method: "POST",
            path: path.to_string(),
            body: Some(body),
        }
    }

    fn delete(command: &'static str, path: &str, args: Value) -> Self {
        Self {
            command,
            args,
            method: "DELETE",
            path: path.to_string(),
            body: None,
        }
    }
}

fn is_tauri() -> bool {
    platform().is_native()
}

/// Le seul point de dispatch du connecteur.
async fn send<T: DeserializeOwned>(call: Call) -> Result<T, String> {
    if is_tauri() {
        via_tauri(call.command, call.args).await
    } else {
        let url = api(&call.path);
        match call.body {
            Some(body) => {
                let builder = match call.method {
                    "POST" => gloo_net::http::Request::post(&url),
                    "PUT" => gloo_net::http::Request::put(&url),
                    _ => gloo_net::http::Request::post(&url),
                };
                let request = builder.json(&body).map_err(|e| e.to_string())?;
                decode(request.send().await).await
            }
            None => {
                let response = match call.method {
                    "DELETE" => gloo_net::http::Request::delete(&url).send().await,
                    _ => gloo_net::http::Request::get(&url).send().await,
                };
                decode(response).await
            }
        }
    }
}

/// Une opération sans corps de réponse (204) : on ne décode rien, on vérifie le statut.
///
/// Le connecteur unifié décode du JSON partout ; une suppression ne renvoie rien. Sans ce
/// chemin, le serveur supprimait bien mais le front croyait à un échec de lecture.
async fn send_empty(call: Call) -> Result<(), String> {
    if is_tauri() {
        return via_tauri(call.command, call.args).await;
    }
    let response = gloo_net::http::Request::delete(&api(&call.path))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = response.status();
    if !(200..300).contains(&status) {
        let text = response.text().await.unwrap_or_default();
        return Err(refusal(&text, status));
    }
    Ok(())
}

async fn via_tauri<T: DeserializeOwned>(cmd: &str, args: Value) -> Result<T, String> {
    let args = serde_wasm_bindgen::to_value(&args).map_err(|e| e.to_string())?;
    let out = invoke(cmd, args).await.map_err(|e| js_error(&e))?;
    serde_wasm_bindgen::from_value(out).map_err(|e| e.to_string())
}

/// Le refus du domaine, côté Tauri : `{code, message}`.
fn js_error(e: &JsValue) -> String {
    js_sys::Reflect::get(e, &JsValue::from_str("message"))
        .ok()
        .and_then(|m| m.as_string())
        .or_else(|| e.as_string())
        .unwrap_or_else(|| "Erreur inconnue".into())
}

/// Décode une réponse : 2xx → JSON, sinon le refus `{code, message}` du domaine.
async fn decode<T: DeserializeOwned>(
    resp: Result<gloo_net::http::Response, gloo_net::Error>,
) -> Result<T, String> {
    let resp = resp.map_err(|e| e.to_string())?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| e.to_string())?;
    if !(200..300).contains(&status) {
        return Err(refusal(&text, status));
    }
    serde_json::from_str(&text).map_err(|e| format!("réponse illisible ({status}) : {e}"))
}

/// Le refus du domaine, côté HTTP : le même `{code, message}`.
fn refusal(body: &str, status: u16) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v["message"].as_str().map(str::to_string))
        .unwrap_or_else(|| {
            if body.trim().is_empty() {
                format!("Erreur HTTP {status}")
            } else {
                body.to_string()
            }
        })
}

fn api(path: &str) -> String {
    format!("/api{path}")
}

// ---- Temps réel : le même `Change`, deux transports --------------------------------

/// S'abonne aux écritures du module : `EventSource` sur `/events` côté web, événement
/// `module://change` côté coquille native. L'appelant applique, le kit stocke.
pub fn subscribe(on_change: impl Fn(Change) + 'static) {
    if is_tauri() {
        subscribe_tauri(on_change);
    } else {
        subscribe_sse(on_change);
    }
}

fn subscribe_sse(on_change: impl Fn(Change) + 'static) {
    let Ok(source) = web_sys::EventSource::new(&api("/events")) else {
        return;
    };
    let handler =
        Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |event: web_sys::MessageEvent| {
            let Some(text) = event.data().as_string() else {
                return;
            };
            if let Ok(change) = serde_json::from_str::<Change>(&text) {
                on_change(change);
            }
        });
    source.set_onmessage(Some(handler.as_ref().unchecked_ref()));
    // l'abonnement vit aussi longtemps que la page
    handler.forget();
}

fn subscribe_tauri(on_change: impl Fn(Change) + 'static) {
    let Some(win) = web_sys::window() else {
        return;
    };
    let Ok(tauri) = js_sys::Reflect::get(&win, &JsValue::from_str("__TAURI__")) else {
        return;
    };
    let Ok(event) = js_sys::Reflect::get(&tauri, &JsValue::from_str("event")) else {
        return;
    };
    let Ok(listen) = js_sys::Reflect::get(&event, &JsValue::from_str("listen")) else {
        return;
    };
    let Ok(listen) = listen.dyn_into::<js_sys::Function>() else {
        return;
    };
    let handler = Closure::<dyn FnMut(JsValue)>::new(move |payload: JsValue| {
        if let Ok(change) = serde_wasm_bindgen::from_value::<Change>(payload) {
            on_change(change);
        }
    });
    let _ = listen.call2(
        &event,
        &JsValue::from_str("module://change"),
        handler.as_ref(),
    );
    handler.forget();
}

// ---- Opérations : une ligne par opération, les deux surfaces côte à côte ----------

/// Le chemin gRPC-Web d'une méthode du service `Data`.
fn rpc(method: &str) -> String {
    format!("/shop.data.v1.Data/{method}")
}

/// La réponse à une demande d'autorisation : décision **et** session à jour.
#[derive(Clone, Debug, Deserialize)]
pub struct RequestOutcome {
    pub decision: PermissionDecision,
    pub session: Session,
    pub message: String,
}

pub async fn status() -> Result<Status, String> {
    if is_tauri() {
        return send(Call::get("module_status", "/status", json!({}))).await;
    }
    let raw = crate::grpc_web::call(&rpc("Status"), Vec::new()).await?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

pub async fn load() -> Result<Status, String> {
    if is_tauri() {
        return send(Call::post("module_load", "/load", json!({}), json!({}))).await;
    }
    let raw = crate::grpc_web::call(&rpc("Load"), Vec::new()).await?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

pub async fn interface() -> Result<Interface, String> {
    if is_tauri() {
        return send(Call::get("module_interface", "/interface", json!({}))).await;
    }
    let raw = crate::grpc_web::call(&rpc("Interface"), Vec::new()).await?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

/// La session de la surface. Le front ne décide pas qui il est : il le demande.
pub async fn session() -> Result<Session, String> {
    if is_tauri() {
        return send(Call::get("app_session", "/session", json!({}))).await;
    }
    let raw = crate::grpc_web::call(&rpc("Session"), Vec::new()).await?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

/// Les enregistrements d'une entité.
///
/// **gRPC-Web par défaut sur le web** : le navigateur appelle le même service `Data` que le
/// client natif, seul le transport diffère (gRPC-Web au lieu de HTTP/JSON). La coquille
/// native garde `invoke`, puisqu'elle n'a pas de serveur HTTP.
pub async fn records(entity: &str) -> Result<Vec<Value>, String> {
    if is_tauri() {
        send(Call::get(
            "list_records",
            &format!("/records/{entity}"),
            json!({ "entity": entity }),
        ))
        .await
    } else {
        crate::grpc_web::list(&rpc("List"), entity).await
    }
}

pub async fn save(entity: &str, id: Option<String>, data: Value) -> Result<Value, String> {
    if is_tauri() {
        return send(Call::post(
            "save_record",
            &format!("/records/{entity}"),
            json!({ "entity": entity, "id": id, "data": data }),
            json!({ "id": id, "data": data }),
        ))
        .await;
    }
    let request = crate::grpc_web::Message::new()
        .string(1, entity)
        .string(2, id.as_deref().unwrap_or_default())
        .string(3, &data.to_string())
        .finish();
    let raw = crate::grpc_web::call(&rpc("Save"), request).await?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

pub async fn delete(entity: &str, id: &str) -> Result<(), String> {
    if is_tauri() {
        return send_empty(Call::delete(
            "delete_record",
            &format!("/records/{entity}/{id}"),
            json!({ "entity": entity, "id": id }),
        ))
        .await;
    }
    let request = crate::grpc_web::Message::new()
        .string(1, entity)
        .string(2, id)
        .finish();
    crate::grpc_web::call(&rpc("Remove"), request)
        .await
        .map(|_| ())
}

pub async fn place_order(
    customer_id: &str,
    lines: Vec<LineIn>,
    state: &str,
) -> Result<Value, String> {
    if is_tauri() {
        return send(Call::post(
            "place_order_cmd",
            "/orders",
            json!({ "customerId": customer_id, "lines": lines, "state": state }),
            json!({ "customerId": customer_id, "lines": lines, "state": state }),
        ))
        .await;
    }
    let request = crate::grpc_web::Message::new()
        .string(1, customer_id)
        .string(
            2,
            &serde_json::to_string(&lines).map_err(|e| e.to_string())?,
        )
        .string(3, state)
        .finish();
    let raw = crate::grpc_web::call(&rpc("PlaceOrder"), request).await?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

/// Demande une autorisation à la surface. Le front ne s'accorde rien : il demande.
pub async fn request_permission(
    entity: &str,
    op: &Operation,
    requirement: Requirement,
    reason: &str,
) -> Result<RequestOutcome, String> {
    let payload = json!({
        "entity": entity,
        "op": op,
        "requirement": requirement,
        "reason": reason,
    });
    if is_tauri() {
        return send(Call::post(
            "permission_request",
            "/permissions/request",
            json!({ "req": payload }),
            payload,
        ))
        .await;
    }
    let request = crate::grpc_web::Message::new()
        .string(1, &payload.to_string())
        .finish();
    let raw = crate::grpc_web::call(&rpc("RequestPermission"), request).await?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}
