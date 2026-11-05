//! # Le shell : ce qui entoure les écrans
//!
//! Barre latérale (desktop), barre d'onglets (mobile), barre supérieure avec fil d'Ariane,
//! bascule de thème persistée, bannière de statut en `aria-live`, et l'hôte du panneau
//! d'édition. C'est le même composant sur les trois surfaces : seule la largeur décide de
//! la forme.

use std::collections::BTreeMap;

use leptos::prelude::*;
use leptos::task::spawn_local;

use this_app::{Boot, BootState, Device, Notice, Notices, Settings};
use ui_shell::{now_ms, BrowserDevice, LocalStorage, SettingsPanel, Splash, Store, ToastHost};

use crate::api::{self, Capabilities, Module, ModuleDescriptor, Session, Snapshot, Surface};
use crate::format::entity_label;
use crate::icons;
use crate::views;

/// Au-delà de ce délai sans synchronisation, un retour recharge les données.
const RESUME_STALE_MS: u64 = 30_000;

#[derive(Clone, PartialEq)]
pub enum Page {
    Dashboard,
    Entity(String),
    NewOrder,
}

impl Page {
    pub fn title(&self) -> String {
        match self {
            Page::Dashboard => "Tableau de bord".into(),
            Page::Entity(name) => entity_label(name),
            Page::NewOrder => "Nouvelle commande".into(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Editing {
    pub entity: String,
    pub id: Option<String>,
}

/// Signaux partagés entre le shell et les écrans.
#[derive(Clone, Copy)]
pub struct Ctx {
    pub desc: RwSignal<Option<Module>>,
    pub page: RwSignal<Page>,
    pub editing: RwSignal<Option<Editing>>,
    pub values: RwSignal<BTreeMap<String, String>>,
    pub search: RwSignal<String>,
    pub msg: RwSignal<Option<(bool, String)>>,
    pub locales: RwSignal<Vec<String>>,
    /// Qui parle à cette surface (transmis par elle, jamais deviné).
    pub session: RwSignal<Session>,
    /// Ce que cette session peut faire, d'après les politiques du descripteur.
    pub caps: RwSignal<Capabilities>,
    /// Le contrat complet du module : il porte les politiques, donc les capacités.
    pub module: RwSignal<Option<ModuleDescriptor>>,
    /// Les enregistrements, une piste réactive par entité.
    pub store: StoredValue<Store>,
    /// Les avis à l'écran (toasts), bornés et regroupés par clé.
    pub notices: RwSignal<Notices>,
    pub refresh: Callback<()>,
    /// Date (ms) de la dernière synchronisation réussie, écrite par le chargement initial et
    /// par chaque rafraîchissement : la décision de reprise s'en sert.
    pub synced: StoredValue<Option<u64>>,
}

fn ctx_names(desc: RwSignal<Option<Module>>) -> Vec<String> {
    desc.get_untracked()
        .map(|d| d.entities.iter().map(|e| e.name.clone()).collect())
        .unwrap_or_default()
}

/// Change de page et referme ce qui est ouvert.
pub fn go(ctx: Ctx, page: Page) {
    ctx.editing.set(None);
    ctx.msg.set(None);
    ctx.page.set(page);
}

/// Adopte une interface : descripteur, langues livrées, session et capacités.
///
/// Les capacités ne sont pas devinées : elles se calculent depuis les politiques du
/// descripteur et la session que la surface vient de transmettre.
async fn adopt_interface(ctx: Ctx, iface: api::Interface) -> Result<Module, String> {
    // Une seule lecture du descripteur : le contrat typé sert aux politiques **et** au
    // modèle d'affichage.
    let locales = iface.locales();
    ctx.locales.set(locales);
    let desc = iface.model();
    ctx.desc.set(Some(desc.clone()));
    ctx.store.set_value(Store::of(&desc));
    ctx.module.set(Some(iface.descriptor));

    let session = api::session().await.unwrap_or(Session::anonymous());
    apply_session(ctx, session);
    Ok(desc)
}

impl Ctx {
    /// Les enregistrements d'une entité (lecture réactive : seule cette entité abonne).
    pub fn records(&self, entity: &str) -> Vec<serde_json::Value> {
        self.store.with_value(|store| store.records(entity))
    }

    /// Un enregistrement précis, sans réveiller les autres entités.
    pub fn record(&self, entity: &str, id: &str) -> Option<serde_json::Value> {
        self.store.with_value(|store| store.get(entity, id))
    }

    /// Tous les enregistrements : à réserver aux écrans qui ont besoin de résoudre des
    /// références entre entités.
    pub fn snapshot(&self) -> Snapshot {
        let mut snapshot = Snapshot::new();
        if let Some(module) = self.desc.get() {
            for name in module.entity_names() {
                snapshot.insert(name.clone(), self.records(&name));
            }
        }
        snapshot
    }

    pub fn record_count(&self, entity: &str) -> usize {
        self.store.with_value(|store| store.count(entity))
    }

    /// Pousse un avis : toast ici, notification système si la coquille y est autorisée.
    ///
    /// L'expiration est programmée à l'échéance calculée par la file, pas scrutée.
    pub fn notice(&self, notice: Notice) {
        let ttl = this_app::DEFAULT_TTL_MS;
        self.notices.update(|notices| {
            notices.push_for(notice.clone(), now_ms(), ttl);
        });
        // la coquille native relaie vers l'OS **si** l'utilisateur a accordé la permission ;
        // sinon `BrowserDevice` ne fait rien et le toast reste la seule trace
        BrowserDevice.notify(&notice);
        let notices = self.notices;
        set_timeout(
            move || {
                notices.update(|notices| {
                    notices.expire(now_ms());
                });
            },
            std::time::Duration::from_millis(ttl + 60),
        );
    }

    /// Applique un changement localement : pas de rechargement, l'enregistrement suffit.
    pub fn apply_local(&self, change: this_app::Change) {
        self.store.with_value(|store| store.apply(&change));
    }
}

/// Adopte une session et recalcule ce qu'elle peut faire, sans recharger le module.
pub fn apply_session(ctx: Ctx, session: Session) {
    ctx.session.set(session.clone());
    if let Some(module) = ctx.module.get_untracked() {
        ctx.caps.set(Capabilities::for_session(&module, &session));
    }
}

/// Recharge l'interface du module puis les données.
pub async fn open_module(ctx: Ctx) {
    match api::interface().await {
        Ok(iface) => match adopt_interface(ctx, iface).await {
            Ok(_) => ctx.refresh.run(()),
            Err(e) => ctx.msg.set(Some((false, e))),
        },
        Err(e) => ctx.msg.set(Some((false, e))),
    }
}

/// Un `Boot` neuf, première étape déjà nommée.
fn fresh_boot() -> Boot {
    let mut boot = Boot::new(600, now_ms());
    boot.step("démarrage");
    boot
}

/// Charge l'interface puis les données, en notant chaque étape pour le splash.
async fn boot_sequence(ctx: Ctx, boot: RwSignal<Boot>) -> Result<(), String> {
    let status = api::status().await?;
    if !status.loaded {
        boot.update(|b| {
            b.ready();
        });
        return Ok(());
    }
    boot.update(|b| {
        b.step("lecture de l'interface");
    });
    let iface = api::interface().await?;
    let desc = adopt_interface(ctx, iface).await?;
    boot.update(|b| {
        b.step("lecture des données");
    });
    let snapshot = fetch_all(Some(desc)).await?;
    ctx.store.with_value(|store| store.load(&snapshot));
    ctx.synced.set_value(Some(now_ms()));
    boot.update(|b| {
        b.ready();
    });
    Ok(())
}

async fn fetch_all(desc: Option<Module>) -> Result<Snapshot, String> {
    let names: Vec<String> = desc
        .map(|d| d.entities.iter().map(|e| e.name.clone()).collect())
        .unwrap_or_default();
    let mut out = Snapshot::new();
    for name in names {
        out.insert(name.clone(), api::records(&name).await?);
    }
    Ok(out)
}

/// Retire le splash dès que `this_app::Boot` l'autorise (durée minimale écoulée).
fn hide_when_ready(boot: RwSignal<Boot>, splash: RwSignal<bool>) {
    let remaining = boot.get_untracked().remaining_ms(now_ms());
    let hide = move || {
        if !boot.get_untracked().visible(now_ms()) {
            splash.set(false);
        }
    };
    if remaining == 0 {
        hide();
    } else {
        set_timeout(hide, std::time::Duration::from_millis(remaining));
    }
}

/// Les faits affichés dans les réglages : surface, session, capacités, module.
///
/// Tout vient du contrat : la session est transmise par la surface, les capacités sont
/// calculées depuis les politiques du descripteur. Rien n'est codé en dur ici.
fn settings_facts(
    ctx: Ctx,
    surface: Surface,
    chrome: RwSignal<this_app::ChromeLayout>,
) -> Vec<(String, String)> {
    let session = ctx.session.get();
    let caps = ctx.caps.get();
    let list = |names: Vec<&str>| {
        if names.is_empty() {
            "aucune".to_string()
        } else {
            names.join(", ")
        }
    };

    let mut facts = vec![
        ("Surface".to_string(), surface.id().to_string()),
        ("Habillage".to_string(), chrome.get_untracked().summary()),
        ("Transport".to_string(), surface.transport().to_string()),
        ("Session".to_string(), session.label().to_string()),
        (
            "Données".to_string(),
            if surface.is_native() {
                "répertoire de données de l'app"
            } else {
                "répertoire du serveur"
            }
            .to_string(),
        ),
        (
            "Permissions".to_string(),
            match &session.grants {
                Some(grants) if !grants.is_empty() => grants.join(", "),
                Some(_) => "aucune".to_string(),
                // la surface ne transmet pas les permissions : on ne prétend pas savoir
                None => "non transmises par la surface".to_string(),
            },
        ),
        ("Création".to_string(), list(caps.writable_entities())),
        ("Suppression".to_string(), list(caps.deletable_entities())),
    ];

    let uncertain = caps.uncertain_entities();
    if !uncertain.is_empty() {
        facts.push((
            "Indetermine".to_string(),
            format!("{} (dépend de l'hôte)", uncertain.join(", ")),
        ));
    }

    // Le flux d'événements n'est pas déclaré à part : il se dérive du contrat.
    if let Some(module) = ctx.module.get() {
        let topics: Vec<String> = module
            .event_topics()
            .into_iter()
            .map(|topic| topic.topic)
            .collect();
        facts.push(("Flux".to_string(), topics.join(" · ")));
    }

    if let Some(d) = ctx.desc.get() {
        facts.push(("Module".to_string(), format!("{} {}", d.name, d.version)));
        facts.push((
            "Contrat".to_string(),
            format!(
                "{} entités · {} liens · {} erreurs",
                d.entities.len(),
                d.links.len(),
                d.errors
            ),
        ));
    }
    let locales = ctx.locales.get();
    if !locales.is_empty() {
        facts.push(("Langues du module".to_string(), locales.join(" · ")));
    }
    facts
}

#[component]
pub fn App() -> impl IntoView {
    let desc = RwSignal::new(None::<Module>);
    let store = StoredValue::new(Store::of(&Module::default()));
    let msg = RwSignal::new(None::<(bool, String)>);
    let locales = RwSignal::new(Vec::<String>::new());
    let synced = StoredValue::new(None::<u64>);
    let refresh = Callback::new(move |_: ()| {
        let names = ctx_names(desc);
        spawn_local(async move {
            let mut out = Snapshot::new();
            for n in names {
                match api::records(&n).await {
                    Ok(rows) => {
                        out.insert(n, rows);
                    }
                    Err(e) => {
                        msg.set(Some((false, e)));
                        return;
                    }
                }
            }
            store.with_value(|store| store.load(&out));
            synced.set_value(Some(now_ms()));
        });
    });

    let ctx = Ctx {
        desc,
        page: RwSignal::new(Page::Dashboard),
        editing: RwSignal::new(None),
        values: RwSignal::new(BTreeMap::new()),
        search: RwSignal::new(String::new()),
        msg,
        locales,
        session: RwSignal::new(Session::anonymous()),
        caps: RwSignal::new(Capabilities::default()),
        module: RwSignal::new(None),
        store,
        notices: RwSignal::new(Notices::default()),
        refresh,
        synced,
    };

    let surface = api::platform();
    // Coquille native : pas de zoom, pas de sélection accidentelle. Le site n'est pas touché.
    ui_shell::behave_like_app(surface);
    // Habillage système : feux macOS, plein écran, encoche, île, barre d'accueil. Le kit
    // mesure et décide ; le style lit `--chrome-*`, l'en-tête reçoit la poignée de fenêtre.
    let chrome = ui_shell::install_chrome();
    let drag = ui_shell::drag_region(chrome);

    // Réglages : lus depuis `localStorage`, appliqués au document, réécrits à chaque changement.
    let settings = RwSignal::new(Settings::load(&LocalStorage));
    ui_shell::apply_settings(&settings.get_untracked());
    Effect::new(move |_| {
        let current = settings.get();
        ui_shell::apply_settings(&current);
        let mut store = LocalStorage;
        let _ = current.save(&mut store);
    });

    // Démarrage : `this-app` tient la machine à états, l'app ne fait que la piloter.
    let boot = RwSignal::new(fresh_boot());
    let splash = RwSignal::new(true);
    let start = move || {
        boot.set(fresh_boot());
        splash.set(true);
        ctx.msg.set(None);
        spawn_local(async move {
            boot.update(|b| {
                b.step("ouverture du store");
            });
            if let Err(e) = boot_sequence(ctx, boot).await {
                boot.update(|b| {
                    b.failed(e);
                });
            }
            hide_when_ready(boot, splash);
        });
    };
    start();
    let retry = Callback::new(move |_| start());

    // Le temps réel : chaque écriture du domaine corrige le magasin, sans redemander.
    // Rappelable : au retour de l'arrière-plan, le flux web a pu mourir avec la connexion.
    let live = move || {
        api::subscribe(
            move |change| ctx.apply_local(change),
            move |error| {
                // un flux interrompu n'est pas silencieux : le magasin cesse de se corriger
                ctx.notice(
                    this_app::Notice::new("realtime", "Temps réel interrompu", error)
                        .with_tone(this_app::Tone::Warn),
                );
            },
        );
    };
    live();

    // Retour au premier plan, ou réseau revenu : `resume_plan` décide, l'app exécute.
    ui_shell::on_resume(move || {
        let plan = ui_shell::platform::resume_plan(
            synced.get_value(),
            now_ms(),
            RESUME_STALE_MS,
            ui_shell::is_online(),
        );
        if plan.resync {
            refresh.run(());
        }
        // Sous Tauri, les changements viennent du Rust et survivent à la suspension : y
        // réinscrire doublerait les écouteurs. Sur le web, le flux est rouvert.
        if plan.reconnect && api::platform() == Surface::Web {
            live();
        }
    });

    let settings_open = RwSignal::new(false);

    let load = move |_| {
        spawn_local(async move {
            match api::load().await {
                Ok(_) => {
                    ctx.msg.set(Some((
                        true,
                        "Module shop chargé : entités, liens, erreurs et messages fr/en.".into(),
                    )));
                    open_module(ctx).await;
                }
                Err(e) => ctx.msg.set(Some((false, e))),
            }
        });
    };

    view! {
        <div class="shell">
            <aside class="sidebar" data-tauri-drag-region=drag>
                <div class="brand" data-tauri-drag-region=drag>
                    <span class="brand-mark" inner_html=icons::MODULE></span>
                    <span class="brand-text">
                        <strong>"Boutique"</strong>
                        <span>"module shop"</span>
                    </span>
                </div>

                <Show
                    when=move || ctx.desc.get().is_some()
                    fallback=|| view! { <p class="muted small">"Module non chargé."</p> }
                >
                    <nav class="nav" aria-label="Navigation principale">
                        <span class="nav-sep">"Pilotage"</span>
                        <button
                            class="nav-link"
                            aria-current=move || (ctx.page.get() == Page::Dashboard).then_some("page")
                            on:click=move |_| go(ctx, Page::Dashboard)
                        >
                            <span class="nav-ico" inner_html=icons::DASHBOARD></span>
                            <span>"Tableau de bord"</span>
                        </button>
                        <button
                            class="nav-link"
                            aria-current=move || (ctx.page.get() == Page::NewOrder).then_some("page")
                            on:click=move |_| go(ctx, Page::NewOrder)
                        >
                            <span class="nav-ico" inner_html=icons::CART></span>
                            <span>"Nouvelle commande"</span>
                        </button>

                        <span class="nav-sep">"Données"</span>
                        {move || ctx.desc.get().map(|d| d.entities.iter().map(|e| {
                            let name = e.name.clone();
                            let name_go = e.name.clone();
                            let icon = match e.name.as_str() {
                                "order" => icons::ORDERS,
                                "customer" => icons::CUSTOMERS,
                                "order_item" => icons::LINES,
                                _ => icons::PRODUCTS,
                            };
                            view! {
                                <button
                                    class="nav-link"
                                    aria-current=move || (ctx.page.get() == Page::Entity(name.clone())).then_some("page")
                                    on:click=move |_| go(ctx, Page::Entity(name_go.clone()))
                                >
                                    <span class="nav-ico" inner_html=icon></span>
                                    <span>{entity_label(&e.name)}</span>
                                </button>
                            }
                        }).collect_view())}
                    </nav>

                    <div class="side-foot">
                        <div class="module-card">
                            <div class="module-head">
                                <span class="module-name">{move || ctx.desc.get().map(|d| format!("{} {}", d.name, d.version)).unwrap_or_default()}</span>
                                <span class="pill info">{move || ctx.locales.get().join(" · ")}</span>
                            </div>
                            {move || ctx.desc.get().map(|d| view! {
                                <p class="muted small">
                                    {format!("{} entités · {} liens · {} erreurs métier",
                                        d.entities.len(), d.links.len(), d.errors)}
                                </p>
                                <ul class="contract">
                                    {d.links.iter().map(|l| view! {
                                        <li>
                                            <code>{l.source.clone()}</code>
                                            <span class="arrow">"→"</span>
                                            <em>{l.link_type.clone()}</em>
                                            <span class="arrow">"→"</span>
                                            <code>{l.target.clone()}</code>
                                        </li>
                                    }).collect_view()}
                                </ul>
                            })}
                        </div>
                        <div class="surface">
                            <span class="dot ok"></span>
                            <span class="small">{surface.id()}</span>
                            <span class="muted small">"· " {surface.transport()}</span>
                        </div>
                    </div>
                </Show>
            </aside>

            <div class="content">
                <header class="topbar" data-tauri-drag-region=drag>
                    // Fenêtre sans cadre système : les boutons sont à l'app. Ils ne
                    // s'affichent que si la coquille rapporte `data-titlebar="hidden"`,
                    // sinon l'OS les dessine déjà et on en aurait deux jeux.
                    <div class="win-controls">
                        <button class="win-btn close" aria-label="Fermer"
                            on:click=move |_| crate::window::close()></button>
                        <button class="win-btn minimize" aria-label="Réduire"
                            on:click=move |_| crate::window::minimize()></button>
                        <button class="win-btn zoom" aria-label="Plein écran"
                            on:click=move |_| crate::window::toggle_fullscreen()></button>
                    </div>
                    <nav class="crumbs" aria-label="Fil d'Ariane" data-tauri-drag-region=drag>
                        <span class="crumb-root">"shop"</span>
                        <span class="crumb-sep" inner_html=icons::ARROW></span>
                        <span class="crumb">{move || ctx.page.get().title()}</span>
                    </nav>
                    <div class="topbar-actions">
                        <span class="surface-pill" title=format!("{} — {}", surface.id(), surface.transport())>
                            <span class="dot ok"></span>
                            {surface.id()}
                        </span>
                        <button class="btn icon ghost" aria-label="Ouvrir les réglages"
                            on:click=move |_| settings_open.set(true)>
                            <span inner_html=icons::GEAR></span>
                        </button>
                    </div>
                </header>

                <main class="main">
                    {move || ctx.msg.get().map(|(ok, t)| view! {
                        <div class=if ok { "banner ok" } else { "banner danger" } role="status" aria-live="polite">
                            <span class="banner-ico" inner_html=if ok { icons::CHECK } else { icons::CLOSE }></span>
                            <span class="banner-text">{t}</span>
                            <button class="btn icon ghost small" aria-label="Fermer le message"
                                on:click=move |_| ctx.msg.set(None)>
                                <span inner_html=icons::CLOSE></span>
                            </button>
                        </div>
                    })}

                    <Show
                        when=move || ctx.desc.get().is_some()
                        fallback=move || view! {
                            <section class="hero">
                                <span class="hero-badge">
                                    <span class="dot ok"></span>
                                    "ModuleStore prêt"
                                </span>
                                <h1>"Une boutique, chargée comme module"</h1>
                                <p class="hero-lede">
                                    "Le module "<code>"shop"</code>" déclare ses entités, ses liens, ses erreurs métier et ses messages fr/en. Il est écrit dans le "
                                    <code>"ModuleStore"</code>" en une seule opération — descripteur et bundles ensemble."
                                </p>
                                <ul class="hero-list">
                                    <li>"L'interface de l'app est dérivée du descripteur : listes, formulaires, libellés."</li>
                                    <li>"Le domaine vérifie stock, total et références ; ses refus portent les messages du module."</li>
                                    <li>"La même UI tourne en desktop Tauri et en site web, seul le transport change."</li>
                                </ul>
                                <button class="btn primary" on:click=load>
                                    <span inner_html=icons::MODULE></span>
                                    "Charger la boutique"
                                </button>
                            </section>
                        }
                    >
                        {move || match ctx.page.get() {
                            Page::Dashboard => views::dashboard(ctx).into_any(),
                            Page::Entity(name) => views::entity_page(ctx, name).into_any(),
                            Page::NewOrder => views::new_order(ctx).into_any(),
                        }}
                    </Show>
                </main>

                <nav class="tabbar" aria-label="Navigation">
                    <button class="tab" aria-current=move || (ctx.page.get() == Page::Dashboard).then_some("page")
                        on:click=move |_| go(ctx, Page::Dashboard)>
                        <span class="tab-ico" inner_html=icons::DASHBOARD></span>
                        <span>"Accueil"</span>
                    </button>
                    {move || ctx.desc.get().map(|d| d.entities.iter().take(3).map(|e| {
                        let name = e.name.clone();
                        let name_go = e.name.clone();
                        let short = match e.name.as_str() {
                            "order" => "Commandes".to_string(),
                            "customer" => "Clients".to_string(),
                            "product" => "Produits".to_string(),
                            other => entity_label(other),
                        };
                        let icon = match e.name.as_str() {
                            "order" => icons::ORDERS,
                            "customer" => icons::CUSTOMERS,
                            "order_item" => icons::LINES,
                            _ => icons::PRODUCTS,
                        };
                        view! {
                            <button class="tab"
                                aria-current=move || (ctx.page.get() == Page::Entity(name.clone())).then_some("page")
                                on:click=move |_| go(ctx, Page::Entity(name_go.clone()))>
                                <span class="tab-ico" inner_html=icon></span>
                                <span>{short}</span>
                            </button>
                        }
                    }).collect_view())}
                    <button class="tab accent" aria-current=move || (ctx.page.get() == Page::NewOrder).then_some("page")
                        on:click=move |_| go(ctx, Page::NewOrder)>
                        <span class="tab-ico" inner_html=icons::PLUS></span>
                        <span>"Commande"</span>
                    </button>
                </nav>
            </div>

            <Show when=move || ctx.editing.get().is_some()>
                {move || views::panel(ctx)}
            </Show>

            <ToastHost
                notices=ctx.notices
                on_dismiss=Callback::new(move |key: String| {
                    ctx.notices.update(|notices| {
                        notices.dismiss(&key);
                    });
                })
            />

            {move || {
                if !splash.get() {
                    return ().into_any();
                }
                let b = boot.get();
                let steps = b.steps().to_vec();
                let error = match b.state() {
                    BootState::Failed(message) => Some(message.clone()),
                    _ => None,
                };
                view! {
                    <Splash
                        title="Boutique"
                        subtitle="Un module this-rs, chargé au démarrage"
                        steps=steps
                        error=error
                        on_retry=retry
                    />
                }
                .into_any()
            }}

            {move || {
                if !settings_open.get() {
                    return ().into_any();
                }
                view! {
                    <SettingsPanel
                        settings=settings
                        facts=settings_facts(ctx, surface, chrome)
                        on_close=Callback::new(move |_| settings_open.set(false))
                    />
                }
                .into_any()
            }}
        </div>
    }
}
