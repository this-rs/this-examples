//! # Les écrans
//!
//! Rien n'est codé en dur : les colonnes, les formulaires, les listes déroulantes et les
//! badges sont dérivés du descripteur du module chargé. Ce fichier ne sait pas ce qu'est
//! un produit ou une commande — il sait ce qu'est une entité, un champ et une référence.

use std::collections::BTreeMap;

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json::{json, Value};

use this_ui::{badge, kpi, meter, Cell, Column, Entity, EntityTable, Field, Kind, Row, Tone};

use crate::api::{self, Gap, LineIn, Operation, Requirement, Snapshot};
use crate::format::*;
use this_ui::Panel;

use crate::icons;
use crate::shell::{go, Ctx, Editing, Page};

// ---- Briques -------------------------------------------------------------------------

/// Le modèle d'une entité, s'il existe dans le module.
fn entity_of(module: &Option<api::Module>, name: &str) -> Option<Entity> {
    module.as_ref().and_then(|m| m.entity(name)).cloned()
}

/// Ce que l'écran considère comme indispensable, et ce qui n'est qu'un plus.
///
/// C'est un jugement de l'application, pas du module : le module dit *qui a le droit*,
/// l'écran dit *à quel point il en a besoin*.
fn requirement_for(entity: &str, op: &Operation) -> Requirement {
    match (entity, op) {
        // gérer les fiches clients EST l'écran : y supprimer est indispensable
        ("customer", Operation::Delete) => Requirement::Required,
        // ailleurs, supprimer enrichit l'écran sans le définir
        _ => Requirement::Optional,
    }
}

/// Les permissions IAM que la politique de l'opération nomme.
///
/// Vide = il n'y a rien à demander (l'opération dépend de l'identité, pas d'une permission) :
/// l'interface ne propose donc pas un bouton qui ne pourrait rien obtenir.
fn needed_permissions(
    module: &Option<api::ModuleDescriptor>,
    entity: &str,
    op: &Operation,
) -> Vec<String> {
    module
        .as_ref()
        .and_then(|m| m.entities.iter().find(|e| e.name == entity))
        .map(|e| {
            e.permissions
                .effective(op)
                .permissions()
                .into_iter()
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn operation_label(op: &Operation) -> &'static str {
    match op {
        Operation::Create => "Création",
        Operation::Update => "Modification",
        Operation::Delete => "Suppression",
        Operation::List => "Lecture (liste)",
        Operation::Get => "Lecture",
        _ => "Opération",
    }
}

/// Demande l'autorisation manquante à la surface, puis réapplique la session reçue.
fn request_access(
    ctx: Ctx,
    entity: String,
    op: Operation,
    requirement: Requirement,
    reason: String,
) {
    spawn_local(async move {
        match api::request_permission(&entity, &op, requirement, &reason).await {
            Ok(outcome) => {
                let granted = outcome.decision.granted();
                crate::shell::apply_session(ctx, outcome.session);
                ctx.notice(
                    this_app::Notice::new("permission", "Autorisation", outcome.message).with_tone(
                        if granted {
                            this_app::Tone::Ok
                        } else {
                            this_app::Tone::Danger
                        },
                    ),
                );
            }
            Err(e) => ctx.msg.set(Some((false, e))),
        }
    });
}

// ---- Tableau de bord -----------------------------------------------------------------

pub fn dashboard(ctx: Ctx) -> impl IntoView {
    move || {
        // une lecture par entité : chacune n'abonne qu'à sa piste
        let products = ctx.records("product");
        let orders = ctx.records("order");
        let customers = ctx.records("customer");

        let active = products
            .iter()
            .filter(|p| p["active"] == json!(true))
            .count();
        let stock_value: f64 = products
            .iter()
            .map(|p| num(&p["price"]) * num(&p["stock"]))
            .sum();
        let revenue: f64 = orders
            .iter()
            .filter(|o| o["state"] != json!("cancelled"))
            .map(|o| num(&o["total"]))
            .sum();
        let units: f64 = products.iter().map(|p| num(&p["stock"])).sum();
        let ticket = if orders.is_empty() {
            0.0
        } else {
            revenue / orders.len() as f64
        };

        let out: Vec<Value> = products
            .iter()
            .filter(|p| num(&p["stock"]) <= 0.0)
            .cloned()
            .collect();
        let low: Vec<Value> = products
            .iter()
            .filter(|p| num(&p["stock"]) > 0.0 && num(&p["stock"]) <= 5.0)
            .cloned()
            .collect();
        let max_stock = products
            .iter()
            .map(|p| num(&p["stock"]))
            .fold(1.0, f64::max);

        let mut recent = orders.clone();
        recent.sort_by(|a, b| b["number"].as_str().cmp(&a["number"].as_str()));
        recent.truncate(5);

        let mut cats: BTreeMap<String, (usize, f64, f64)> = BTreeMap::new();
        for p in &products {
            let key = p["category"].as_str().unwrap_or("—").to_string();
            let row = cats.entry(key).or_insert((0, 0.0, 0.0));
            row.0 += 1;
            row.1 += num(&p["stock"]);
            row.2 += num(&p["price"]) * num(&p["stock"]);
        }
        let max_cat = cats.values().map(|v| v.2).fold(1.0, f64::max);

        let alerts_empty = out.is_empty() && low.is_empty();
        // les commandes récentes n'ont besoin que des clients pour résoudre la référence
        let mut snap_ref = Snapshot::new();
        snap_ref.insert("customer".to_string(), customers.clone());

        view! {
            <section class="page">
                <header class="page-head">
                    <div class="page-title">
                        <h1>"Tableau de bord"</h1>
                        <p class="page-sub">
                            {format!("{} produits · {} clients · {} commandes",
                                products.len(), customers.len(), orders.len())}
                        </p>
                    </div>
                    <button class="btn primary" on:click=move |_| go(ctx, Page::NewOrder)>
                        <span inner_html=icons::PLUS></span>"Nouvelle commande"
                    </button>
                </header>

                <div class="kpis">
                    {kpi("Chiffre d'affaires", money_f(revenue), format!("{} commande(s) hors annulées", orders.len()))}
                    {kpi("Valeur du stock", money_f(stock_value), format!("{} unités · {} références", count_f(units), products.len()))}
                    {kpi("Panier moyen", money_f(ticket), "sur les commandes enregistrées".into())}
                    {kpi("Produits actifs", format!("{} / {}", active, products.len()), format!("{} en rupture", out.len()))}
                    {kpi("Clients", count_f(customers.len() as f64), "fichier du store".into())}
                </div>

                <div class="grid-2">
                    <section class="card">
                        <header class="card-head">
                            <h2>"Stock à surveiller"</h2>
                            <span class="pill muted">{format!("{} alerte(s)", out.len() + low.len())}</span>
                        </header>
                        {if alerts_empty {
                            view! { <p class="empty">"Aucun produit sous le seuil."</p> }.into_any()
                        } else {
                            view! {
                                <div class="row-list">
                                    {out.iter().map(|p| {
                                        let name = display_name("product", p);
                                        let sku = p["sku"].as_str().unwrap_or_default().to_string();
                                        view! {
                                            <div class="row">
                                                <span class="dot danger"></span>
                                                <span class="row-main">
                                                    <strong>{name}</strong>
                                                    <span class="muted small">{sku}</span>
                                                </span>
                                                {meter(num(&p["stock"]), max_stock, 5.0)}
                                                {badge("Rupture", Tone::Danger)}
                                            </div>
                                        }
                                    }).collect_view()}
                                    {low.iter().map(|p| {
                                        let name = display_name("product", p);
                                        let sku = p["sku"].as_str().unwrap_or_default().to_string();
                                        let stock = num(&p["stock"]);
                                        view! {
                                            <div class="row">
                                                <span class="dot warn"></span>
                                                <span class="row-main">
                                                    <strong>{name}</strong>
                                                    <span class="muted small">{sku}</span>
                                                </span>
                                                {meter(stock, max_stock, 5.0)}
                                                {badge(format!("{} restant(s)", count_f(stock)), Tone::Warn)}
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }}
                    </section>

                    <section class="card">
                        <header class="card-head">
                            <h2>"Dernières commandes"</h2>
                            <button class="btn ghost small" on:click=move |_| go(ctx, Page::Entity("order".into()))>
                                "Tout voir"
                            </button>
                        </header>
                        <div class="row-list">
                            {recent.iter().map(|o| {
                                let oc = o.clone();
                                let number = display_name("order", o);
                                let customer = ref_label(&snap_ref, "customer", o["customer"].as_str().unwrap_or_default());
                                let date = o["placed_on"].as_str().unwrap_or_default().to_string();
                                let state = o["state"].as_str().unwrap_or_default();
                                view! {
                                    <button class="row row-btn" on:click=move |_| open_record(ctx, "order", &oc)>
                                        <span class="row-main">
                                            <strong>{number}</strong>
                                            <span class="muted small">{format!("{customer} · {date}")}</span>
                                        </span>
                                        {badge(enum_label(state), state_tone(state))}
                                        <span class="amount">{money(&o["total"])}</span>
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </section>
                </div>

                <section class="card">
                    <header class="card-head">
                        <h2>"Catalogue par catégorie"</h2>
                        <span class="muted small">"valeur = prix × stock"</span>
                    </header>
                    <table class="mini">
                        <thead>
                            <tr>
                                <th>"Catégorie"</th>
                                <th class="num">"Références"</th>
                                <th class="num">"Unités"</th>
                                <th class="num">"Valeur"</th>
                                <th class="col-bar"></th>
                            </tr>
                        </thead>
                        <tbody>
                            {cats.iter().map(|(cat, (n, units, value))| view! {
                                <tr>
                                    <td>{enum_label(cat)}</td>
                                    <td class="num">{n.to_string()}</td>
                                    <td class="num">{count_f(*units)}</td>
                                    <td class="num">{money_f(*value)}</td>
                                    <td class="col-bar">
                                        <span class="meter"><span class="meter-fill" style=format!("width:{:.1}%", value / max_cat * 100.0)></span></span>
                                    </td>
                                </tr>
                            }).collect_view()}
                        </tbody>
                    </table>
                </section>
            </section>
        }
    }
}

// ---- Liste d'entité ------------------------------------------------------------------

pub fn entity_page(ctx: Ctx, name: String) -> impl IntoView {
    let n_head = name.clone();
    let n_new = name.clone();
    let n_table = name.clone();

    view! {
        <section class="page">
            <header class="page-head">
                <div class="page-title">
                    <h1>{entity_label(&name)}</h1>
                    <p class="page-sub">{move || {
                        let count = ctx.record_count(&n_head);
                        format!("{} enregistrement(s) dans le store", count)
                    }}</p>
                </div>
                {move || {
                    let desc = ctx.desc.get();
                    // deux conditions : l'entité est saisissable à la main, et la politique
                    // du module autorise la création pour cette session.
                    let allowed = ctx
                        .caps
                        .get()
                        .allows(&n_new, &Operation::Create)
                        .is_allowed();
                    let can = allowed
                        && desc
                            .as_ref()
                            .and_then(|module| module.entity(&n_new))
                            .map(creatable)
                            .unwrap_or(false);
                    let n = n_new.clone();
                    can.then(|| view! {
                        <button class="btn primary" on:click=move |_| open_new(ctx, &n)>
                            <span inner_html=icons::PLUS></span>"Nouveau"
                        </button>
                    })
                }}
            </header>

            <div class="toolbar">
                <label class="search">
                    <span class="search-ico" inner_html=icons::SEARCH></span>
                    <input
                        type="search"
                        placeholder="Rechercher dans cette liste…"
                        aria-label="Rechercher"
                        prop:value=move || ctx.search.get()
                        on:input=move |ev| ctx.search.set(event_target_value(&ev))
                    />
                </label>
            </div>

            {move || render_table(ctx, &n_table)}
        </section>
    }
}

fn render_table(ctx: Ctx, name: &str) -> AnyView {
    let module = ctx.desc.get();
    let Some(entity) = module.as_ref().and_then(|m| m.entity(name)) else {
        return this_ui::empty("Entité inconnue.");
    };

    let snap = ctx.snapshot();
    let query = ctx.search.get().to_lowercase();
    let all: Vec<Value> = snap.get(name).cloned().unwrap_or_default();
    let max_stock = all.iter().map(|p| num(&p["stock"])).fold(1.0, f64::max);

    // Le filtre cherche dans ce que l'œil voit : le texte de chaque cellule.
    let records: Vec<Value> = all
        .into_iter()
        .filter(|record| {
            query.is_empty()
                || entity
                    .fields
                    .iter()
                    .map(|field| cell(&snap, field, &record[&field.name]))
                    .collect::<Vec<_>>()
                    .join(" ")
                    .to_lowercase()
                    .contains(&query)
        })
        .collect();

    let columns: Vec<Column> = entity
        .fields
        .iter()
        .map(|field| Column {
            label: field_label(&field.name),
            numeric: field.kind.is_numeric(),
        })
        .collect();

    let rows: Vec<Row> = records
        .iter()
        .map(|record| Row {
            id: record["id"].as_str().unwrap_or_default().to_string(),
            cells: entity
                .fields
                .iter()
                .map(|field| cell_of(&snap, name, field, &record[&field.name], max_stock))
                .collect(),
        })
        .collect();
    let count = rows.len();

    let entity_name = name.to_string();
    let snap_for_open = snap.clone();
    let on_open = Callback::new(move |id: String| {
        if let Some(record) = find(&snap_for_open, &entity_name, &id) {
            open_record(ctx, &entity_name, &record);
        }
    });

    view! {
        <EntityTable columns=columns rows=rows action_label="Ouvrir" on_open=on_open />
        <p class="muted small table-foot">{format!("{count} ligne(s) affichée(s)")}</p>
    }
    .into_any()
}

/// Ce qu'une cellule contient, décidé ici : le kit ne sait pas ce qu'est un « stock ».
fn cell_of(snap: &Snapshot, entity: &str, field: &Field, value: &Value, max_stock: f64) -> Cell {
    let on = value == &Value::Bool(true);
    match &field.kind {
        Kind::Enum(_) => Cell::Badge {
            text: enum_label(value.as_str().unwrap_or_default()),
            tone: state_tone(value.as_str().unwrap_or_default()),
        },
        Kind::Decimal => Cell::Amount(money(value)),
        Kind::Bool => Cell::Badge {
            text: if on { "Oui" } else { "Non" }.to_string(),
            tone: if on { Tone::Ok } else { Tone::Muted },
        },
        Kind::Ref(target) => {
            Cell::Reference(ref_label(snap, target, value.as_str().unwrap_or_default()))
        }
        Kind::Date | Kind::DateTime => Cell::Text(text(value)),
        // deux conventions de cette boutique : un stock se lit comme une jauge, un nom de
        // client comme une personne
        _ if field.name == "stock" && entity == "product" => Cell::Meter {
            value: num(value),
            max: max_stock,
            low: 5.0,
        },
        _ if field.name == "name" && entity == "customer" => {
            let name = text(value);
            Cell::Person {
                initials: initials(&name),
                name,
            }
        }
        _ => Cell::Text(text(value)),
    }
}

// ---- Formulaire (panneau) --------------------------------------------------------------

pub fn open_record(ctx: Ctx, entity: &str, rec: &Value) {
    let desc = ctx.desc.get_untracked();
    let fields = entity_of(&desc, entity)
        .map(|e| e.fields)
        .unwrap_or_default();
    let values = fields
        .iter()
        .map(|f| (f.name.clone(), to_form(&rec[&f.name])))
        .collect();
    ctx.values.set(values);
    ctx.editing.set(Some(Editing {
        entity: entity.into(),
        id: rec["id"].as_str().map(str::to_string),
    }));
    ctx.msg.set(None);
    ctx.page.set(Page::Entity(entity.into()));
}

pub fn open_new(ctx: Ctx, entity: &str) {
    let desc = ctx.desc.get_untracked();
    let values = entity_of(&desc, entity)
        .map(|e| {
            e.fields
                .iter()
                .map(|f| {
                    let v = match &f.kind {
                        Kind::Bool => "true".to_string(),
                        Kind::Enum(variants) => variants.first().cloned().unwrap_or_default(),
                        _ => f.default.as_ref().map(to_form).unwrap_or_default(),
                    };
                    (f.name.clone(), v)
                })
                .collect()
        })
        .unwrap_or_default();
    ctx.values.set(values);
    ctx.editing.set(Some(Editing {
        entity: entity.into(),
        id: None,
    }));
    ctx.msg.set(None);
}

pub fn panel(ctx: Ctx) -> impl IntoView {
    move || {
        let Some(ed) = ctx.editing.get() else {
            return ().into_any();
        };
        let desc = ctx.desc.get();
        let Some(ent) = desc
            .as_ref()
            .and_then(|module| module.entity(&ed.entity))
            .cloned()
        else {
            return ().into_any();
        };
        let title = match &ed.id {
            Some(id) => {
                let snap = ctx.snapshot();
                find(&snap, &ed.entity, id)
                    .map(|r| display_name(&ed.entity, &r))
                    .unwrap_or_else(|| "Enregistrement".into())
            }
            None => format!("Nouveau {}", entity_label_one(&ed.entity).to_lowercase()),
        };
        let inputs = ent
            .fields
            .iter()
            .map(|f| field_input(ctx, f))
            .collect_view();
        // Ce que cette session peut faire sur cette entité, d'après le descripteur.
        let caps = ctx.caps.get();
        let entity_caps = caps.entity(&ed.entity);
        let writable = entity_caps.map(|c| c.writable()).unwrap_or(false);
        let deletable = entity_caps.map(|c| c.can_delete()).unwrap_or(false);
        let session_label = ctx.session.get().label();

        // Ce qui manque pour les actions que cet écran sait faire, et ce qui est réellement
        // demandable (une politique qui nomme une permission).
        let needs: Vec<(String, Operation, Requirement)> = [Operation::Delete, Operation::Update]
            .into_iter()
            .map(|op| {
                (
                    ed.entity.clone(),
                    op.clone(),
                    requirement_for(&ed.entity, &op),
                )
            })
            .collect();
        let module = ctx.module.get_untracked();
        let requestable: Vec<(Gap, Vec<String>)> = ctx
            .caps
            .get()
            .gaps(&needs)
            .into_iter()
            .filter_map(|gap| {
                let needed = needed_permissions(&module, &gap.entity, &gap.op);
                (!needed.is_empty()).then_some((gap, needed))
            })
            .collect();

        let ent_save = ed.entity.clone();
        let id_save = ed.id.clone();
        let ent_del = ed.entity.clone();
        let id_del = ed.id.clone();
        let eyebrow = entity_label_one(&ed.entity);

        view! {
            <Panel
                eyebrow=eyebrow
                title=title.clone()
                on_close=Callback::new(move |_| ctx.editing.set(None))
            >
                <div class="form-grid">{inputs}</div>
                <p class="muted small">
                    {if writable {
                        "Les champs grisés sont déclarés en lecture seule par le module ; le domaine les refuse à l'écriture.".to_string()
                    } else {
                        format!("La session « {session_label} » ne peut pas modifier cette entité : le module le refuse.")
                    }}
                </p>
                {requestable.into_iter().map(|(gap, needed)| {
                    let entity = gap.entity.clone();
                    let op = gap.op.clone();
                    let requirement = gap.requirement;
                    let title = format!(
                        "{} · {}",
                        operation_label(&gap.op),
                        requirement.label()
                    );
                    let detail = format!(
                        "La session « {session_label} » doit obtenir {}.",
                        needed.join(", ")
                    );
                    let reason = format!(
                        "{} sur {} depuis l'écran, pour la session « {session_label} ».",
                        operation_label(&op),
                        entity
                    );
                    view! {
                        <div class="grant">
                            <div class="grant-text">
                                <strong>{title}</strong>
                                <span class="muted small">{detail}</span>
                            </div>
                            <button class="btn small"
                                on:click=move |_| request_access(
                                    ctx,
                                    entity.clone(),
                                    op.clone(),
                                    requirement,
                                    reason.clone(),
                                )>
                                "Demander l'accès"
                            </button>
                        </div>
                    }
                }).collect_view()}

                <footer class="panel-foot">
                    <button class="btn primary" disabled=move || !writable
                        on:click=move |_| {
                            save(ctx, ent_save.clone(), id_save.clone())
                        }>
                        <span inner_html=icons::CHECK></span>"Enregistrer"
                    </button>
                    {move || {
                        if !deletable {
                            return ().into_any();
                        }
                        let Some(rid) = id_del.clone() else { return ().into_any() };
                        let entity = ent_del.clone();
                        view! {
                            <button class="btn danger" on:click=move |_| {
                                delete(ctx, entity.clone(), rid.clone())
                            }>
                                <span inner_html=icons::TRASH></span>"Supprimer"
                            </button>
                        }
                        .into_any()
                    }}
                    <button class="btn ghost" on:click=move |_| ctx.editing.set(None)>"Annuler"</button>
                </footer>
            </Panel>
        }
        .into_any()
    }
}

/// Un widget par type de champ, lu dans le descripteur.
fn field_input(ctx: Ctx, f: &Field) -> impl IntoView {
    let name = f.name.clone();
    let label = field_label(&f.name);
    let n_val = name.clone();
    let value = move || ctx.values.get().get(&n_val).cloned().unwrap_or_default();
    let n_set = name.clone();
    let set = move |v: String| {
        ctx.values.update(|m| {
            m.insert(n_set.clone(), v);
        })
    };

    let control: AnyView = if f.read_only {
        view! { <input class="input" disabled prop:value=value /> }.into_any()
    } else if let Kind::Enum(variants) = &f.kind {
        let variants = variants.clone();
        view! {
            <select class="input" prop:value=value on:change=move |ev| set(event_target_value(&ev))>
                {f.optional.then(|| view! { <option value="">"—"</option> })}
                {variants.into_iter().map(|v| {
                    let l = enum_label(&v);
                    view! { <option value=v>{l}</option> }
                }).collect_view()}
            </select>
        }
        .into_any()
    } else if let Some(target) = f.kind.as_ref_entity() {
        let target = target.to_string();
        view! {
            <select class="input" prop:value=value on:change=move |ev| set(event_target_value(&ev))>
                <option value="">"— choisir —"</option>
                {move || ctx.records(&target).into_iter().map(|r| {
                    let id = r["id"].as_str().unwrap_or_default().to_string();
                    let l = display_name(&target, &r);
                    view! { <option value=id>{l}</option> }
                }).collect_view()}
            </select>
        }
        .into_any()
    } else if f.kind == Kind::Bool {
        let n_chk = name.clone();
        view! {
            <label class="switch">
                <input type="checkbox"
                    prop:checked=move || ctx.values.get().get(&n_chk).map(|v| v == "true").unwrap_or(false)
                    on:change=move |ev| set(event_target_checked(&ev).to_string()) />
                <span>"Oui"</span>
            </label>
        }
        .into_any()
    } else {
        let input_type = f.kind.input_type();
        let step = f.kind.step();
        view! {
            <input class="input" type=input_type step=step prop:value=value
                on:input=move |ev| set(event_target_value(&ev)) />
        }
        .into_any()
    };

    let hint = if f.optional { "facultatif" } else { "requis" };

    view! {
        <div class=if f.kind == Kind::Bool { "field field-check" } else { "field" }>
            <span class="field-label">{label}<span class="field-hint">{hint}</span></span>
            {control}
        </div>
    }
}

/// Enregistrement : valeurs éditées → JSON, champs en lecture seule exclus.
fn save(ctx: Ctx, entity: String, id: Option<String>) {
    let desc = ctx.desc.get_untracked();
    let Some(ent) = entity_of(&desc, &entity) else {
        return;
    };
    let values = ctx.values.get_untracked();
    let mut data = serde_json::Map::new();
    for f in ent.fields.iter().filter(|f| !f.read_only) {
        let raw = values
            .get(&f.name)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if raw.is_empty() {
            if f.optional {
                continue;
            }
            if f.kind == Kind::Bool {
                data.insert(f.name.clone(), json!(false));
                continue;
            }
            ctx.msg.set(Some((
                false,
                format!("Champ requis : {}.", field_label(&f.name)),
            )));
            return;
        }
        let v = match f.kind {
            Kind::Bool => json!(raw == "true"),
            Kind::Int => match raw.parse::<i64>() {
                Ok(n) => json!(n),
                Err(_) => {
                    ctx.msg.set(Some((
                        false,
                        format!("Nombre entier attendu : {}.", field_label(&f.name)),
                    )));
                    return;
                }
            },
            _ => json!(raw),
        };
        data.insert(f.name.clone(), v);
    }
    spawn_local(async move {
        match api::save(&entity, id, Value::Object(data)).await {
            Ok(rec) => {
                ctx.msg.set(Some((true, "Enregistré.".into())));
                let rid = rec["id"].as_str().map(str::to_string);
                ctx.editing.set(Some(Editing {
                    entity: entity.clone(),
                    id: rid,
                }));
                // le magasin se corrige tout seul : un enregistrement, zéro requête
                ctx.apply_local(this_app::Change::updated(entity, rec));
            }
            Err(e) => ctx.msg.set(Some((false, e))),
        }
    });
}

fn delete(ctx: Ctx, entity: String, id: String) {
    spawn_local(async move {
        match api::delete(&entity, &id).await {
            Ok(()) => {
                ctx.msg.set(Some((true, "Enregistrement supprimé.".into())));
                ctx.editing.set(None);
                ctx.apply_local(this_app::Change::deleted(entity, id));
            }
            Err(e) => ctx.msg.set(Some((false, e))),
        }
    });
}

// ---- Nouvelle commande ----------------------------------------------------------------

pub fn new_order(ctx: Ctx) -> impl IntoView {
    let customer = RwSignal::new(String::new());
    let state = RwSignal::new("draft".to_string());
    let lines = RwSignal::new(vec![(String::new(), "1".to_string())]);

    let price_of = move |product: &str| -> Option<(f64, f64)> {
        let snap = ctx.snapshot();
        let p = find(&snap, "product", product)?;
        Some((num(&p["price"]), num(&p["stock"])))
    };

    let total = move || {
        lines
            .get()
            .iter()
            .filter_map(|(p, q)| {
                let q: f64 = q.parse().ok()?;
                price_of(p).map(|(price, _)| price * q)
            })
            .sum::<f64>()
    };

    let submit = move |_| {
        let cust = customer.get_untracked();
        let st = state.get_untracked();
        let ls: Vec<LineIn> = lines
            .get_untracked()
            .into_iter()
            .filter(|(p, _)| !p.is_empty())
            .map(|(p, q)| LineIn {
                product_id: p,
                qty: q.parse().unwrap_or(0),
            })
            .collect();
        if cust.is_empty() {
            ctx.msg.set(Some((
                false,
                "Choisissez un client avant de valider.".into(),
            )));
            return;
        }
        if ls.is_empty() {
            ctx.msg.set(Some((
                false,
                "Ajoutez au moins une ligne de produit.".into(),
            )));
            return;
        }
        spawn_local(async move {
            match api::place_order(&cust, ls, &st).await {
                Ok(o) => {
                    // `go` efface le message courant : on navigue d'abord, on annonce ensuite.
                    lines.set(vec![(String::new(), "1".to_string())]);
                    customer.set(String::new());
                    go(ctx, Page::Entity("order".into()));
                    ctx.msg.set(Some((
                        true,
                        format!(
                            "Commande {} enregistrée : {}.",
                            o["number"].as_str().unwrap_or_default(),
                            money(&o["total"])
                        ),
                    )));
                    ctx.refresh.run(());
                }
                Err(e) => ctx.msg.set(Some((false, e))),
            }
        });
    };

    view! {
        <section class="page">
            <header class="page-head">
                <div class="page-title">
                    <h1>"Nouvelle commande"</h1>
                    <p class="page-sub">"Le stock, le total et les références sont vérifiés par le module avant toute écriture."</p>
                </div>
            </header>

            <div class="order-grid">
                <div class="card">
                    <div class="form-grid">
                        <div class="field">
                            <span class="field-label">"Client"<span class="field-hint">"requis"</span></span>
                            <select class="input" prop:value=move || customer.get()
                                on:change=move |ev| customer.set(event_target_value(&ev))>
                                <option value="">"— choisir —"</option>
                                {move || ctx.records("customer").into_iter().map(|c| {
                                    let id = c["id"].as_str().unwrap_or_default().to_string();
                                    let l = display_name("customer", &c);
                                    view! { <option value=id>{l}</option> }
                                }).collect_view()}
                            </select>
                        </div>
                        <div class="field">
                            <span class="field-label">"Statut initial"<span class="field-hint">"requis"</span></span>
                            <select class="input" prop:value=move || state.get()
                                on:change=move |ev| state.set(event_target_value(&ev))>
                                {["draft", "paid", "shipped"].into_iter().map(|s| view! {
                                    <option value=s>{enum_label(s)}</option>
                                }).collect_view()}
                            </select>
                        </div>
                    </div>

                    <div class="card-head lines-head">
                        <h2>"Lignes"</h2>
                        <button class="btn ghost small" on:click=move |_| lines.update(|ls| ls.push((String::new(), "1".into())))>
                            <span inner_html=icons::PLUS></span>"Ajouter une ligne"
                        </button>
                    </div>

                    <div class="lines">
                        {move || lines.get().into_iter().enumerate().map(|(i, (p, q))| {
                            let note = price_of(&p).map(|(price, stock)| {
                                let want: f64 = q.parse().unwrap_or(0.0);
                                (price * want, stock, want > stock)
                            });
                            let tone = match note { Some((_, _, true)) => "danger", _ => "muted" };
                            let note_text = match note {
                                Some((_, stock, true)) => format!("Stock insuffisant · {stock} disponible(s)"),
                                Some((_, stock, false)) => format!("Stock : {}", count_f(stock)),
                                None => "—".to_string(),
                            };
                            let subtotal = note.map(|(s, _, _)| s).unwrap_or(0.0);
                            view! {
                                <div class="line">
                                    <select class="input" prop:value=p.clone() on:change=move |ev| {
                                        let v = event_target_value(&ev);
                                        lines.update(|ls| ls[i].0 = v);
                                    }>
                                        <option value="">"— produit —"</option>
                                        {move || ctx.records("product").into_iter().map(|pr| {
                                            let id = pr["id"].as_str().unwrap_or_default().to_string();
                                            let l = format!("{} — {}", display_name("product", &pr), money(&pr["price"]));
                                            view! { <option value=id>{l}</option> }
                                        }).collect_view()}
                                    </select>
                                    <input class="input qty" type="number" min="1" step="1" aria-label="Quantité"
                                        prop:value=q.clone() on:input=move |ev| {
                                            let v = event_target_value(&ev);
                                            lines.update(|ls| ls[i].1 = v);
                                        } />
                                    <span class=format!("line-total")>{money_f(subtotal)}</span>
                                    <span class=format!("line-note {tone}")>{note_text}</span>
                                    <button class="btn icon ghost small" aria-label="Retirer la ligne"
                                        on:click=move |_| lines.update(|ls| { ls.remove(i); })>
                                        <span inner_html=icons::CLOSE></span>
                                    </button>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>

                <aside class="card summary">
                    <h2>"Récapitulatif"</h2>
                    <dl class="summary-list">
                        <div><dt>"Lignes"</dt><dd>{move || lines.get().iter().filter(|(p, _)| !p.is_empty()).count().to_string()}</dd></div>
                        <div><dt>"Client"</dt><dd>{move || {
                            let c = customer.get();
                            if c.is_empty() { "—".to_string() } else { ctx.record("customer", &c).map(|r| display_name("customer", &r)).unwrap_or_else(|| c.clone()) }
                        }}</dd></div>
                        <div><dt>"Statut"</dt><dd>{move || enum_label(&state.get())}</dd></div>
                    </dl>
                    <div class="summary-total">
                        <span>"Total"</span>
                        <strong>{move || money_f(total())}</strong>
                    </div>
                    <button class="btn primary block"
                        disabled=move || !ctx.caps.get().allows("order", &Operation::Create).is_allowed()
                        on:click=submit>
                        <span inner_html=icons::CART></span>"Valider la commande"
                    </button>
                    <p class="muted small">"Un refus (stock, client, quantité) revient avec le message du module et n'écrit rien."</p>
                </aside>
            </div>
        </section>
    }
}
