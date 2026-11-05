//! # Le laboratoire de modules
//!
//! Un programme du langage de module, compilé et **exécuté dans la page** :
//! aucun aller-retour serveur. Le même programme passe deux fois — par
//! l'interpréteur, puis par le module wasm assemblé et instancié — et les deux
//! réponses sont affichées côte à côte. Si elles divergent, c'est visible.
//!
//! ## Pourquoi c'est ici et pas au serveur
//!
//! Un module chargé à chaud doit pouvoir être produit là où on l'écrit. Le
//! langage est total (pas de boucle, pas d'état, aucun I/O implicite), donc
//! l'exécuter dans la page ne demande aucune confiance : il n'y a rien à
//! échapper, parce que rien n'est exprimable.
//!
//! L'assemblage se fait en Rust (`this_module_lang::assemble`), pas en
//! JavaScript : la page reçoit des octets et appelle `WebAssembly.instantiate`,
//! exactement comme le serveur les donne à `wasmi`.
//!
//! ## Ce que le laboratoire fige
//!
//! `now()` rend la même valeur aux deux exécuteurs, capturée **une fois** avant
//! l'exécution. Sans cela un programme qui lit l'heure divergerait de quelques
//! millisecondes et la comparaison ne dirait plus rien.

use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use this_module_lang::assemble::compile_to_wasm;
use this_module_lang::compile;
use this_module_lang::eval::{Host, Value, run};
use this_module_lang::{Function, Type};

/// Le programme proposé : il montre un paramètre, un opérateur et un effet.
const START: &str = "fn run(a: Int) -> Int {\n    emit(\"bonjour depuis la page\");\n    a + 1\n}";

/// Ce qu'un exécuteur a produit. Comparable : c'est ce qui rend la divergence
/// visible plutôt que racontée.
#[derive(Clone, Debug, PartialEq)]
struct Outcome {
    value: String,
    effects: Vec<String>,
}

struct Recorder {
    now: i64,
    effects: Vec<String>,
}

impl Host for Recorder {
    fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        match (name, args) {
            ("emit", [Value::Str(text)]) => {
                self.effects.push(format!("emit({text:?})"));
                Ok(Value::Unit)
            }
            ("now", []) => {
                self.effects.push("now()".into());
                Ok(Value::Int(self.now))
            }
            ("emit", other) => Err(format!("`emit` attend une chaîne, reçu {other:?}")),
            _ => Err(format!("`{name}` n'a pas d'hôte dans le laboratoire")),
        }
    }
}

/// Le rapport affiché. Trois formes, parce qu'il y a trois choses à dire.
#[derive(Clone, Debug)]
enum Report {
    Refused(String),
    Compared {
        reference: Outcome,
        emitted: Outcome,
        bytes: usize,
        agree: bool,
    },
}

/// Les arguments que le programme attend, à zéro. Un laboratoire n'a pas de
/// saisie par paramètre : ce qui compte est que les deux côtés reçoivent la
/// même chose.
fn default_args(function: &Function) -> Vec<Value> {
    function
        .params
        .iter()
        .map(|param| match param.ty {
            Type::Int => Value::Int(0),
            Type::Bool => Value::Bool(false),
            Type::Str => Value::Str(String::new()),
            Type::Unit => Value::Unit,
        })
        .collect()
}

fn show(value: &Result<Value, String>) -> String {
    match value {
        Ok(Value::Int(n)) => n.to_string(),
        Ok(Value::Bool(b)) => b.to_string(),
        Ok(Value::Str(s)) => format!("{s:?}"),
        Ok(Value::Unit) => "()".to_string(),
        Err(error) => format!("erreur : {error}"),
    }
}

async fn evaluate(source: &str) -> Report {
    let compiled = match compile(source) {
        Ok(compiled) => compiled,
        Err(error) => return Report::Refused(error),
    };

    // La même horloge pour les deux exécuteurs.
    let now = js_sys::Date::now() as i64;

    let mut recorder = Recorder {
        now,
        effects: Vec::new(),
    };
    let args = default_args(&compiled.function);
    let reference = run(&compiled.function, &args, &mut recorder);
    let reference = Outcome {
        value: show(&reference),
        effects: recorder.effects,
    };

    let bytes = match compile_to_wasm(source) {
        Ok(bytes) => bytes,
        Err(error) => return Report::Refused(error),
    };

    match run_in_page(&bytes, &compiled.function, &args, now).await {
        Ok(emitted) => {
            let agree = emitted == reference;
            Report::Compared {
                reference,
                emitted,
                bytes: bytes.len(),
                agree,
            }
        }
        Err(error) => Report::Refused(error),
    }
}

/// Assemble puis instancie le module dans la page, et l'exécute.
async fn run_in_page(
    bytes: &[u8],
    function: &Function,
    args: &[Value],
    now: i64,
) -> Result<Outcome, String> {
    // La mémoire n'existe qu'après l'instanciation, mais `emit` en a besoin
    // pendant l'exécution : on la dépose ici dès qu'elle est connue.
    let memory: Rc<RefCell<Option<JsValue>>> = Rc::new(RefCell::new(None));
    let effects: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

    let emit = {
        let memory = memory.clone();
        let effects = effects.clone();
        Closure::<dyn FnMut(i32, i32)>::new(move |ptr: i32, len: i32| {
            let Some(memory) = memory.borrow().clone() else {
                return;
            };
            let Ok(buffer) = js_sys::Reflect::get(&memory, &"buffer".into()) else {
                return;
            };
            let view = js_sys::Uint8Array::new_with_byte_offset_and_length(
                &buffer,
                ptr as u32,
                len as u32,
            );
            let text = String::from_utf8_lossy(&view.to_vec()).into_owned();
            effects.borrow_mut().push(format!("emit({text:?})"));
        })
    };

    let clock = {
        let effects = effects.clone();
        Closure::<dyn FnMut() -> i64>::new(move || {
            effects.borrow_mut().push("now()".into());
            now
        })
    };

    let host = js_sys::Object::new();
    set(&host, "emit", emit.as_ref().unchecked_ref())?;
    set(&host, "now", clock.as_ref().unchecked_ref())?;

    let imports = js_sys::Object::new();
    set(&imports, "host", &host)?;

    // `WebAssembly.instantiate(bytes, imports)` rend `{ module, instance }` — pas
    // l'instance seule. On lit le champ plutôt que de croire la signature.
    let source = JsFuture::from(js_sys::WebAssembly::instantiate_buffer(bytes, &imports))
        .await
        .map_err(|error| format!("instanciation refusée : {}", describe(&error)))?;
    let held = js_sys::Reflect::get(&source, &"instance".into()).unwrap_or(source);
    let instance: js_sys::WebAssembly::Instance = held
        .dyn_into()
        .map_err(|value| format!("instanciation : type inattendu ({})", describe(&value)))?;

    let exports = js_sys::WebAssembly::Instance::exports(&instance);
    *memory.borrow_mut() = js_sys::Reflect::get(&exports, &"memory".into()).ok();

    let entry = js_sys::Reflect::get(&exports, &"run".into())
        .map_err(|error| format!("le module n'exporte pas `run` : {}", describe(&error)))?;
    let entry: js_sys::Function = entry
        .dyn_into()
        .map_err(|_| "`run` n'est pas une fonction".to_string())?;

    let js_args: Vec<JsValue> = args
        .iter()
        .map(|value| match value {
            Value::Int(n) => js_sys::BigInt::from(*n).into(),
            Value::Bool(b) => JsValue::from_bool(*b),
            _ => JsValue::UNDEFINED,
        })
        .collect();

    let count = js_args.len() as u32;
    let outcome = match count {
        0 => entry.call0(&JsValue::NULL),
        1 => entry.call1(&JsValue::NULL, &js_args[0]),
        2 => entry.call2(&JsValue::NULL, &js_args[0], &js_args[1]),
        _ => return Err("le laboratoire n'accepte que deux paramètres".into()),
    }
    .map_err(|error| format!("exécution refusée : {}", describe(&error)))?;

    // Les effets sont lus avant que les closures ne soient relâchées.
    emit.forget();
    clock.forget();

    let value = match function.returns {
        Type::Unit => "()".to_string(),
        Type::Bool => outcome.as_f64().map(|n| (n != 0.0).to_string()).unwrap_or_default(),
        Type::Int => bigint_to_string(&outcome),
        Type::Str => "chaîne".to_string(),
    };

    let effects = effects.borrow().clone();
    Ok(Outcome { value, effects })
}

fn set(target: &js_sys::Object, key: &str, value: &JsValue) -> Result<(), String> {
    js_sys::Reflect::set(target, &JsValue::from_str(key), value)
        .map(|_| ())
        .map_err(|error| format!("imports : {}", describe(&error)))
}

fn describe(error: &JsValue) -> String {
    error
        .as_string()
        .or_else(|| {
            js_sys::Reflect::get(error, &"message".into())
                .ok()
                .and_then(|message| message.as_string())
        })
        .unwrap_or_else(|| format!("{error:?}"))
}

fn bigint_to_string(value: &JsValue) -> String {
    let bigint: &js_sys::BigInt = value.unchecked_ref();
    match bigint.to_string(10) {
        Ok(text) => text.as_string().unwrap_or_default(),
        Err(_) => "?".to_string(),
    }
}

/// Le panneau.
pub fn page() -> AnyView {
    let source = RwSignal::new(START.to_string());
    let report = RwSignal::new(None::<Report>);
    let busy = RwSignal::new(false);

    let execute = move |_| {
        let text = source.get_untracked();
        busy.set(true);
        spawn_local(async move {
            report.set(Some(evaluate(&text).await));
            busy.set(false);
        });
    };

    view! {
        <section class="page">
            <header class="page-head">
                <div class="page-title">
                    <h1>"Laboratoire de modules"</h1>
                    <p class="page-sub">
                        "Compilé, assemblé et exécuté "
                        <strong>"dans cette page"</strong>
                        " — aucune requête au serveur."
                    </p>
                </div>
            </header>

            <section class="card">
                <header class="card-head">
                    <h2>"Programme"</h2>
                    <button class="btn" on:click=move |_| source.set(START.to_string())>
                        "Rétablir l'exemple"
                    </button>
                </header>

                <textarea
                    class="lab-text"
                    rows="8"
                    spellcheck="false"
                    aria-label="Programme du module"
                    prop:value=move || source.get()
                    on:input=move |ev| source.set(event_target_value(&ev))
                ></textarea>

                <div class="lab-actions">
                    <button class="btn primary" on:click=execute disabled=move || busy.get()>
                        <span>"Compiler et exécuter"</span>
                    </button>
                    <Show when=move || busy.get() fallback=|| ()>
                        <span class="muted small">"Compilation…"</span>
                    </Show>
                </div>
            </section>

            <section class="card">
                <header class="card-head">
                    <h2>"Résultat"</h2>
                </header>
                {move || match report.get() {
                    None => view! {
                        <p class="empty">"Rien n'a encore été exécuté."</p>
                    }.into_any(),
                    Some(Report::Refused(error)) => view! {
                        <div class="lab-refused">
                            <strong>"Refusé"</strong>
                            <pre>{error}</pre>
                        </div>
                    }.into_any(),
                    Some(Report::Compared { reference, emitted, bytes, agree }) => {
                        let (reference_value, reference_effects) = flatten(&reference);
                        let (emitted_value, emitted_effects) = flatten(&emitted);
                        view! {
                            <div class="lab-verdict" class:ok=move || agree class:ko=move || !agree>
                                {if agree {
                                    format!("Les deux exécuteurs sont d'accord — {bytes} octets de wasm assemblé")
                                } else {
                                    "DIVERGENCE : l'interpréteur et le module wasm ne disent pas la même chose".to_string()
                                }}
                            </div>
                            <div class="lab-grid">
                                <div>
                                    <h3>"Interpréteur (référence)"</h3>
                                    <pre>{reference_value}</pre>
                                    <h4>"Effets"</h4>
                                    <pre>{reference_effects}</pre>
                                </div>
                                <div>
                                    <h3>"Module wasm (assemblé ici)"</h3>
                                    <pre>{emitted_value}</pre>
                                    <h4>"Effets"</h4>
                                    <pre>{emitted_effects}</pre>
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </section>
        </section>
    }
    .into_any()
}

fn flatten(outcome: &Outcome) -> (String, String) {
    let effects = if outcome.effects.is_empty() {
        "(aucun)".to_string()
    } else {
        outcome.effects.join("\n")
    };
    (outcome.value.clone(), effects)
}
