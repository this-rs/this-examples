//! # Le laboratoire de modules
//!
//! Un programme s'écrit, se compile **dans la page**, se range dans un VFS
//! navigateur, se relit depuis ce VFS, puis s'exécute — et on peut lui passer un
//! argument pour voir ce qu'il répond.
//!
//! ## Ce que la vue fait, et ce qu'elle ne fait plus
//!
//! Elle saisit, appelle, affiche. La compilation, le rangement et l'exécution
//! vivent dans `this-module-harness` et `this-module-harness-web` : c'est ce qui
//! permet à un harness de faire la même chose sans interface.
//!
//! ## Pourquoi deux exécuteurs
//!
//! L'interpréteur et le module wasm doivent dire la même chose. Cette comparaison
//! est une valeur du laboratoire, pas une fonctionnalité du harness : la vue
//! appelle deux fois, avec deux moteurs.
//!
//! ## Pourquoi l'artefact est relu du VFS
//!
//! `load` le relit après l'avoir rangé. Passer les octets encore en main ne
//! prouverait rien du stockage — et c'est justement ce que ce panneau existe pour
//! montrer : le module survit à un rechargement de la page.

use leptos::prelude::*;
use leptos::task::spawn_local;

use this_module_harness::Harness;
use this_module_harness::executor::{Executor, Outcome, show};
use this_module_harness_web::BrowserExecutor;
use this_module_lang::compile;
use this_module_lang::eval::{Host, Value, run};
use this_vfs_web::IndexedDbVfs;

/// Le programme proposé : un paramètre, un opérateur, un effet.
const START: &str = "fn run(a: Int) -> Int {\n    emit(\"bonjour depuis la page\");\n    a + 1\n}";
const NAME: &str = "lab";
const VERSION: &str = "0.1.0";

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
            _ => Err(format!("`{name}` n'a pas d'hôte dans le laboratoire")),
        }
    }
}

#[derive(Clone, Debug)]
enum Report {
    Refused(String),
    Ran {
        stored: usize,
        etag: String,
        reference: Outcome,
        executed: Outcome,
        agree: bool,
        /// Vrai quand l'artefact vient du VFS sans recompilation — la preuve que
        /// le stockage sert à quelque chose.
        reloaded: bool,
    },
}

/// Relit l'artefact **sans recompiler** et l'exécute. C'est ce geste qui montre
/// que le VFS compte : si la compilation seule suffisait, il n'y aurait rien à
/// ranger.
async fn reload_and_run(arg: i64) -> Report {
    let now = js_sys::Date::now() as i64;
    let vfs = match IndexedDbVfs::open("this-lab").await {
        Ok(vfs) => vfs,
        Err(error) => return Report::Refused(format!("VFS : {error}")),
    };
    let harness = Harness::new(&vfs);
    let program = match harness.load(NAME, VERSION).await {
        Ok(program) => program,
        Err(error) => return Report::Refused(format!("rien en store : {error}")),
    };
    let executed = match BrowserExecutor.run(&program, &[Value::Int(arg)], now).await {
        Ok(outcome) => outcome,
        Err(error) => return Report::Refused(format!("exécution : {error}")),
    };
    Report::Ran {
        stored: program.len(),
        etag: this_vfs::etag_of(&program),
        // Sans compilation, il n'y a pas d'interpréteur à comparer : la référence
        // est le module lui-même, relu.
        reference: executed.clone(),
        executed,
        agree: true,
        reloaded: true,
    }
}

async fn evaluate(source: String, arg: i64) -> Report {
    // L'horloge est figée une fois, pour que les deux exécutions soient comparables.
    let now = js_sys::Date::now() as i64;

    let compiled = match compile(&source) {
        Ok(compiled) => compiled,
        Err(error) => return Report::Refused(error),
    };
    let mut recorder = Recorder {
        now,
        effects: Vec::new(),
    };
    let reference = Outcome {
        value: show(&run(&compiled.function, &[Value::Int(arg)], &mut recorder)),
        effects: recorder.effects,
    };

    // Le vrai chemin : compiler → ranger → RELIRE → exécuter.
    let vfs = match IndexedDbVfs::open("this-lab").await {
        Ok(vfs) => vfs,
        Err(error) => return Report::Refused(format!("VFS : {error}")),
    };
    let harness = Harness::new(&vfs);

    let artifact = match harness.compile(NAME, VERSION, &source).await {
        Ok(artifact) => artifact,
        Err(error) => return Report::Refused(format!("compilation : {error}")),
    };
    let program = match harness.load(NAME, VERSION).await {
        Ok(program) => program,
        Err(error) => return Report::Refused(format!("relecture : {error}")),
    };
    let executed = match BrowserExecutor
        .run(&program, &[Value::Int(arg)], now)
        .await
    {
        Ok(outcome) => outcome,
        Err(error) => return Report::Refused(format!("exécution : {error}")),
    };

    let agree = executed == reference;
    Report::Ran {
        stored: artifact.bytes,
        etag: artifact.etag,
        reference,
        executed,
        agree,
        reloaded: false,
    }
}

/// Le panneau.
pub fn page() -> AnyView {
    let source = RwSignal::new(START.to_string());
    let argument = RwSignal::new("1".to_string());
    let report = RwSignal::new(None::<Report>);
    let busy = RwSignal::new(false);

    let execute = move |_| {
        let text = source.get_untracked();
        let arg = argument.get_untracked().trim().parse().unwrap_or(0);
        busy.set(true);
        spawn_local(async move {
            report.set(Some(evaluate(text, arg).await));
            busy.set(false);
        });
    };

    view! {
        <section class="page">
            <header class="page-head">
                <div class="page-title">
                    <h1>"Laboratoire de modules"</h1>
                    <p class="page-sub">
                        "Compilé, rangé dans un VFS, relu et exécuté "
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
                    <label class="lab-arg">
                        <span class="muted small">"argument (a)"</span>
                        <input
                            type="number"
                            prop:value=move || argument.get()
                            on:input=move |ev| argument.set(event_target_value(&ev))
                        />
                    </label>
                    <button class="btn primary" on:click=execute disabled=move || busy.get()>
                        <span>"Compiler, ranger et exécuter"</span>
                    </button>
                    <button
                        class="btn"
                        on:click=move |_| {
                            let arg = argument.get_untracked().trim().parse().unwrap_or(0);
                            busy.set(true);
                            spawn_local(async move {
                                report.set(Some(reload_and_run(arg).await));
                                busy.set(false);
                            });
                        }
                        disabled=move || busy.get()
                    >
                        "Relire depuis le VFS"
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
                    Some(Report::Ran { stored, etag, reference, executed, agree, reloaded }) => {
                        let (reference_value, reference_effects) = flatten(&reference);
                        let (emitted_value, emitted_effects) = flatten(&executed);
                        let short_etag = etag.chars().take(12).collect::<String>();
                        view! {
                            <div class="lab-verdict" class:ok=move || agree class:ko=move || !agree>
                                {if reloaded {
                                    format!("Relu du VFS sans recompiler ({stored} octets, etag {short_etag}) puis exécuté")
                                } else if agree {
                                    format!("Rangé ({stored} octets, etag {short_etag}), relu du VFS, exécuté — les deux moteurs sont d'accord")
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
                                    <h3>"Module wasm (relu du VFS)"</h3>
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
