//! # L'habillage appliqué au document
//!
//! [`install_chrome`] réunit ce que la coquille sait (plateforme, barre de titre, plein écran,
//! modèle d'appareil — par `plugin:this-shell|env` sous Tauri) et ce que seule la webview sait (marges
//! de sécurité, taille du viewport), demande la décision à `app_kit::chrome::layout`, puis
//! écrit le résultat sur `<html>` :
//!
//! - variables `--chrome-top/right/bottom/left`, `--chrome-header-min`, `--chrome-lead`,
//!   `--chrome-trail`, et `--chrome-cutout-*` quand une découpe existe ;
//! - attributs `data-platform`, `data-titlebar`, `data-fullscreen`, `data-focused`,
//!   `data-drag`, `data-cutout`, `data-orientation`.
//!
//! Et la poignée de fenêtre : [`drag_region`] rend l'attribut `data-tauri-drag-region`
//! **seulement** quand l'en-tête sert de poignée (barre intégrée, hors plein écran). L'app
//! le pose sur ses zones vides d'en-tête ; elle n'écrit aucun cas de plateforme.
//!
//! Recalculé à chaque redimensionnement, rotation, entrée ou sortie du plein écran, et à
//! chaque `chrome://changed` de la coquille.

use app_kit::chrome::{layout, ChromeEnv, ChromeLayout, Insets, Platform, Size, WindowState};
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

const PROBE_ID: &str = "chrome-safe-area-probe";
const CHANGED: &str = "shell://event";
/// Commande de la coquille `this-shell` : l'habillage de la fenêtre appelante.
const ENV_COMMAND: &str = "plugin:this-shell|env";

fn get(target: &JsValue, key: &str) -> Option<JsValue> {
    js_sys::Reflect::get(target, &JsValue::from_str(key))
        .ok()
        .filter(|v| !v.is_undefined() && !v.is_null())
}

fn tauri() -> Option<JsValue> {
    get(web_sys::window()?.as_ref(), "__TAURI__")
}

/// Sans coquille native : la plateforme se lit dans l'agent utilisateur. Sur iPhone, c'est
/// ce qui permet à une page installée de déduire sa découpe depuis la marge du haut.
fn web_env() -> ChromeEnv {
    let ua = web_sys::window()
        .and_then(|w| w.navigator().user_agent().ok())
        .unwrap_or_default()
        .to_lowercase();
    let platform = if ua.contains("iphone") || ua.contains("ipad") {
        Platform::Ios
    } else if ua.contains("android") {
        Platform::Android
    } else {
        Platform::Web
    };
    ChromeEnv {
        platform,
        window: WindowState {
            fullscreen: false,
            maximized: false,
            focused: true,
        },
        ..Default::default()
    }
}

/// Une sonde invisible dont le padding vaut `env(safe-area-inset-*)` : c'est la seule
/// façon de lire ces valeurs depuis le code, et elle reste dans le kit, pas dans l'app.
fn probe(doc: &web_sys::Document) -> Option<web_sys::Element> {
    if let Some(el) = doc.get_element_by_id(PROBE_ID) {
        return Some(el);
    }
    let el = doc.create_element("div").ok()?;
    el.set_id(PROBE_ID);
    let _ = el.set_attribute(
        "style",
        "position:fixed;inset:0 auto auto 0;width:0;height:0;visibility:hidden;\
         pointer-events:none;padding:env(safe-area-inset-top,0px) \
         env(safe-area-inset-right,0px) env(safe-area-inset-bottom,0px) \
         env(safe-area-inset-left,0px)",
    );
    let _ = el.set_attribute("aria-hidden", "true");
    doc.body()?.append_child(&el).ok()?;
    Some(el)
}

fn measured_safe_area() -> Insets {
    let Some(win) = web_sys::window() else {
        return Insets::ZERO;
    };
    let Some(el) = win.document().and_then(|d| probe(&d)) else {
        return Insets::ZERO;
    };
    let Ok(Some(style)) = win.get_computed_style(&el) else {
        return Insets::ZERO;
    };
    let px = |prop: &str| {
        style
            .get_property_value(prop)
            .ok()
            .and_then(|v| v.trim_end_matches("px").parse::<f32>().ok())
            .unwrap_or(0.0)
    };
    Insets::new(
        px("padding-top"),
        px("padding-right"),
        px("padding-bottom"),
        px("padding-left"),
    )
}

fn viewport() -> Size {
    let Some(win) = web_sys::window() else {
        return Size::default();
    };
    let dim = |v: Result<JsValue, JsValue>| v.ok().and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    Size {
        width: dim(win.inner_width()),
        height: dim(win.inner_height()),
    }
}

/// Le plein écran **du navigateur** (Fullscreen API ou `display-mode: fullscreen`).
/// Sous Tauri c'est la coquille qui le dit.
fn web_fullscreen() -> bool {
    let Some(win) = web_sys::window() else {
        return false;
    };
    let element = win
        .document()
        .and_then(|d| get(d.as_ref(), "fullscreenElement"))
        .is_some();
    let mode = win
        .match_media("(display-mode: fullscreen)")
        .ok()
        .flatten()
        .map(|m| m.matches())
        .unwrap_or(false);
    element || mode
}

fn apply(l: &ChromeLayout) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(root) = doc
        .document_element()
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    else {
        return;
    };
    let style = root.style();
    for name in app_kit::chrome::CUTOUT_VARS {
        let _ = style.remove_property(name);
    }
    for (name, value) in l.css_vars() {
        let _ = style.set_property(name, &value);
    }
    for (name, value) in l.attributes() {
        let _ = root.set_attribute(name, value);
    }
}

/// La valeur de `data-tauri-drag-region` pour une zone d'en-tête : présente quand l'en-tête
/// est la poignée de la fenêtre, absente sinon (barre native, plein écran, mobile, web).
///
/// ```ignore
/// <header class="topbar" data-tauri-drag-region=ui_shell::chrome::drag_region(chrome)>
/// ```
/// Tauri ne déplace la fenêtre que si la cible du clic porte l'attribut : le poser sur les
/// zones vides (l'en-tête, le fil d'Ariane), jamais sur un bouton.
pub fn drag_region(chrome: RwSignal<ChromeLayout>) -> impl Fn() -> Option<&'static str> + Copy {
    move || chrome.get().drag_region.then_some("")
}

/// Recalcule depuis la dernière base connue et l'état mesuré à l'instant.
fn refresh(base: RwSignal<ChromeEnv>, out: RwSignal<ChromeLayout>, native: bool) {
    let mut env = base.get_untracked();
    env.safe = measured_safe_area();
    env.viewport = viewport();
    if !native {
        env.window.fullscreen = web_fullscreen();
    }
    let l = layout(&env);
    apply(&l);
    out.set(l);
}

fn on(target: &web_sys::EventTarget, event: &str, f: impl Fn() + 'static) {
    let cb = Closure::<dyn Fn()>::new(f);
    let _ = target.add_event_listener_with_callback(event, cb.as_ref().unchecked_ref());
    cb.forget();
}

fn decode(value: &JsValue) -> Option<ChromeEnv> {
    let text = js_sys::JSON::stringify(value).ok()?.as_string()?;
    serde_json::from_str(&text).ok()
}

/// Installe l'habillage et rend un signal qu'un composant peut lire (placer un élément
/// à côté de l'île, masquer un bouton en plein écran…).
///
/// À appeler une fois, au démarrage. Le signal est aussi fourni en contexte
/// ([`use_chrome`]) pour les composants qui en ont besoin sans le recevoir en paramètre.
pub fn install_chrome() -> RwSignal<ChromeLayout> {
    let native = tauri().is_some();
    let base = RwSignal::new(web_env());
    let out = RwSignal::new(layout(&base.get_untracked()));
    provide_context(out);
    refresh(base, out, native);

    if let Some(win) = web_sys::window() {
        let w: &web_sys::EventTarget = win.as_ref();
        on(w, "resize", move || refresh(base, out, native));
        on(w, "orientationchange", move || refresh(base, out, native));

        // L'activation de la fenêtre : macOS grise les feux quand elle n'est plus
        // active. C'est une information de coquille, pas de page — `focus`/`blur`
        // sur `window` la porte, et le CSS la lit via `data-focused`.
        if let Some(root) = win.document().and_then(|doc| doc.document_element()) {
            let mark = {
                let root = root.clone();
                move |active: bool| {
                    let _ = root.set_attribute("data-focused", if active { "true" } else { "false" });
                }
            };
            mark(true);
            let on_focus = mark.clone();
            on(w, "focus", move || on_focus(true));
            on(w, "blur", move || mark(false));
        }
        if let Some(doc) = win.document() {
            on(doc.as_ref(), "fullscreenchange", move || {
                refresh(base, out, native)
            });
        }
    }

    if let Some(t) = tauri() {
        // L'état initial, puis chaque changement annoncé par la coquille.
        if let Some(invoke) = get(&t, "core")
            .and_then(|c| get(&c, "invoke").map(|f| (c, f)))
            .and_then(|(c, f)| f.dyn_into::<js_sys::Function>().ok().map(|f| (c, f)))
        {
            let (core, invoke) = invoke;
            if let Ok(promise) = invoke.call1(&core, &JsValue::from_str(ENV_COMMAND)) {
                if let Ok(promise) = promise.dyn_into::<js_sys::Promise>() {
                    leptos::task::spawn_local(async move {
                        if let Ok(value) = wasm_bindgen_futures::JsFuture::from(promise).await {
                            if let Some(env) = decode(&value) {
                                base.set(env);
                                refresh(base, out, native);
                            }
                        }
                    });
                }
            }
        }
        if let Some((event, listen)) = get(&t, "event").and_then(|e| {
            get(&e, "listen")
                .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
                .map(|f| (e, f))
        }) {
            let handler = Closure::<dyn Fn(JsValue)>::new(move |evt: JsValue| {
                // `shell://event` porte des `ShellEvent` : on ne garde que l'habillage.
                let env = get(&evt, "payload")
                    .and_then(|p| get(&p, "env"))
                    .and_then(|e| decode(&e));
                if let Some(env) = env {
                    base.set(env);
                    refresh(base, out, native);
                }
            });
            let _ = listen.call2(&event, &JsValue::from_str(CHANGED), handler.as_ref());
            handler.forget();
        }
    }

    out
}

/// L'habillage installé par [`install_chrome`], depuis n'importe quel composant.
pub fn use_chrome() -> Option<RwSignal<ChromeLayout>> {
    use_context::<RwSignal<ChromeLayout>>()
}
