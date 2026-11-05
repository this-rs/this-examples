//! Ce que la coquille fait à la fenêtre : fermer, réduire, agrandir.
//!
//! Sans cadre système (`decorations: false`), plus rien ne dessine ces boutons :
//! c'est l'app qui les porte, et donc à elle de demander l'action. On passe par
//! l'API fenêtre de Tauri exposée dans la page (`withGlobalTauri`), sans ajouter
//! de commande côté Rust : la coquille n'a rien de plus à décider que « ferme »,
//! « réduis », « agrandis ».

use wasm_bindgen::{JsCast, JsValue};

/// Appelle une méthode de la fenêtre courante, si la coquille en expose une.
fn call(method: &str) -> bool {
    let Ok(tauri) = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("__TAURI__"))
    else {
        return false;
    };
    let Ok(api) = js_sys::Reflect::get(&tauri, &JsValue::from_str("window")) else {
        return false;
    };
    let getter = js_sys::Reflect::get(&api, &JsValue::from_str("getCurrentWindow"))
        .ok()
        .and_then(|value| value.dyn_into::<js_sys::Function>().ok());
    let Some(getter) = getter else {
        return false;
    };
    let Ok(current) = getter.call0(&api) else {
        return false;
    };
    let action = js_sys::Reflect::get(&current, &JsValue::from_str(method))
        .ok()
        .and_then(|value| value.dyn_into::<js_sys::Function>().ok());
    let Some(action) = action else {
        return false;
    };
    action.call0(&current).is_ok()
}

/// Fermer la fenêtre (l'app peut rester dans le Dock : c'est la coquille qui décide).
pub fn close() {
    call("close");
}

/// Réduire dans le Dock.
pub fn minimize() {
    call("minimize");
}

/// Agrandir ou rendre sa taille à la fenêtre.
pub fn toggle_maximize() {
    call("toggleMaximize");
}
