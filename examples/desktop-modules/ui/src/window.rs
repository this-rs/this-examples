//! Ce que la coquille fait à la fenêtre : fermer, réduire, plein écran.
//!
//! Sans cadre système (`decorations: false`), plus rien ne dessine ces boutons :
//! c'est l'app qui les porte, et donc à elle de demander l'action. On passe par
//! l'API fenêtre de Tauri exposée dans la page (`withGlobalTauri`) — mais chaque
//! verbe doit être **accordé** dans `capabilities/default.json`, sinon Tauri
//! rejette la promesse en silence et le bouton ne fait rien.
//!
//! Le plein écran est le vrai : `setFullscreen`, pas `toggleMaximize`. Sous
//! macOS, le premier ouvre un bureau à part entière ; le second agrandit la
//! fenêtre dans le même espace, ce qui n'est pas ce qu'on attend d'un bouton
//! vert.

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{JsFuture, spawn_local};

/// La fenêtre courante, si la coquille en expose une.
fn current() -> Option<JsValue> {
    let tauri = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("__TAURI__")).ok()?;
    let api = js_sys::Reflect::get(&tauri, &JsValue::from_str("window")).ok()?;
    let getter = js_sys::Reflect::get(&api, &JsValue::from_str("getCurrentWindow"))
        .ok()?
        .dyn_into::<js_sys::Function>()
        .ok()?;
    getter.call0(&api).ok()
}

fn method(window: &JsValue, name: &str) -> Option<js_sys::Function> {
    js_sys::Reflect::get(window, &JsValue::from_str(name))
        .ok()?
        .dyn_into::<js_sys::Function>()
        .ok()
}

fn call(name: &str) -> bool {
    let Some(window) = current() else {
        return false;
    };
    let Some(method) = method(&window, name) else {
        return false;
    };
    method.call0(&window).is_ok()
}

async fn is_fullscreen() -> Option<bool> {
    let window = current()?;
    let method = method(&window, "isFullscreen")?;
    let promise: js_sys::Promise = method.call0(&window).ok()?.dyn_into().ok()?;
    JsFuture::from(promise).await.ok()?.as_bool()
}

/// Fermer la fenêtre. Sur macOS la coquille peut décider de la masquer plutôt
/// que de quitter : c'est elle qui garde l'app dans le Dock.
pub fn close() {
    call("close");
}

/// Réduire dans le Dock.
pub fn minimize() {
    call("minimize");
}

/// Basculer le vrai plein écran (un bureau à part sous macOS).
pub fn toggle_fullscreen() {
    spawn_local(async move {
        let fullscreen = is_fullscreen().await.unwrap_or(false);
        if let Some(window) = current() {
            if let Some(set) = method(&window, "setFullscreen") {
                let _ = set.call1(&window, &JsValue::from_bool(!fullscreen));
            }
        }
    });
}
