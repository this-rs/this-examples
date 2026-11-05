//! # `shop-core` — la boutique comme module this-rs, sans aucune dépendance de plateforme
//!
//! Ce crate ne connaît ni Tauri, ni HTTP : il porte le **domaine** et il est le seul à
//! savoir comment la boutique fonctionne.
//!
//! - Le **module** (`shop`) déclare la boutique : entités, liens, erreurs métier, messages
//!   fr/en. Il est écrit dans le `ModuleStore` (`VfsModuleStore`) en **un seul `put`**
//!   (descripteur + bundles).
//! - Les **enregistrements** (produits, clients, commandes) vivent dans le répertoire de
//!   données passé à [`Data::open`], un fichier JSON par entité. Chaque écriture passe par
//!   le descripteur : champs requis, types, valeurs d'énumération, références existantes.
//! - La **logique métier** (stock, total de commande, intégrité référentielle) est ici, et
//!   ses refus sont des codes d'erreur du module, rendus dans la langue du bundle `fr`.
//!
//! Les surfaces (desktop Tauri, navigateur, serveur) sont des **adaptateurs minces** :
//! chacune traduit son protocole en appels aux mêmes fonctions publiques.

use std::collections::{BTreeMap, BTreeSet};

pub use this_app::{Change, ChangeOp};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use this::descriptor::field::{FieldDesc, FieldType};
use this::descriptor::{
    ContractError, EntityDesc, ErrorCode, ErrorDesc, ErrorKind, I18nDecl, LinkDesc, Locale,
    MessageBundle, MessageDecl, MessageKey, ModuleDescriptor,
};
use this::storage::vfs::{LocalFsVfs, ModuleStore, VfsModuleStore};

const SHOP: &str = "shop";
// Le descripteur est une **donnée** : changer sa déclaration dans le code n'a aucun effet
// sur un store qui contient déjà une version. Toute évolution du contrat passe par ce
// numéro, et par un `status` qui compare la version, pas la présence.
const SHOP_VERSION: &str = "0.3.0";

/// Données de l'app : le store de modules et les enregistrements, sur disque.
pub struct Data {
    store: VfsModuleStore,
    records_dir: PathBuf,
    /// Sérialise les écritures d'enregistrements (jamais tenu à travers un `await`).
    write_lock: Mutex<()>,
    /// Ce qui vient de changer : les surfaces s'y abonnent (SSE, événement Tauri) pour que
    /// les interfaces se corrigent sans redemander les données.
    changes: broadcast::Sender<Change>,
}

/// Erreur renvoyée au front : un code du module et son message (rendu en fr).
#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
}

fn app_err(e: ContractError) -> AppError {
    AppError {
        code: e.code.0.to_string(),
        message: e.message.clone(),
    }
}

fn io_err(e: std::io::Error) -> AppError {
    AppError {
        code: "this.io".into(),
        message: e.to_string(),
    }
}

/// Rend un message du module : gabarit du bundle, paramètres substitués.
fn render(bundle: Option<&MessageBundle>, code: &str, params: &[(&str, String)]) -> String {
    let template = bundle
        .and_then(|b| b.get(&MessageKey::new(code)))
        .unwrap_or(code)
        .to_string();
    params
        .iter()
        .fold(template, |t, (k, v)| t.replace(&format!("{{{k}}}"), v))
}

fn fail(bundle: Option<&MessageBundle>, code: &str, params: &[(&str, String)]) -> AppError {
    AppError {
        code: code.to_string(),
        message: render(bundle, code, params),
    }
}

// ---- Le module : la boutique ---------------------------------------------------------

fn enum_of(name: &str, variants: &[&str]) -> FieldType {
    FieldType::Enum {
        name: name.into(),
        variants: variants.iter().map(|v| v.to_string()).collect(),
    }
}

fn crud(e: &mut EntityDesc) {
    for op in Operation::CRUD {
        e.permissions.ops.insert(op, PolicyExpr::Authenticated);
    }
}

/// Une entrée du catalogue d'erreurs : (code, genre, paramètres, texte fr, texte en).
type ErrorSpec = (
    &'static str,
    ErrorKind,
    &'static [&'static str],
    &'static str,
    &'static str,
);

const ERRORS: [ErrorSpec; 9] = [
    (
        "shop.product.out_of_stock",
        ErrorKind::Validation,
        &["product", "available"],
        "Stock insuffisant pour {product} : {available} disponible(s).",
        "Not enough stock for {product}: {available} available.",
    ),
    (
        "shop.product.not_found",
        ErrorKind::NotFound,
        &["product"],
        "Produit introuvable : {product}.",
        "Product not found: {product}.",
    ),
    (
        "shop.customer.not_found",
        ErrorKind::NotFound,
        &["customer"],
        "Client introuvable : {customer}.",
        "Customer not found: {customer}.",
    ),
    (
        "shop.order.empty",
        ErrorKind::Validation,
        &[],
        "La commande doit contenir au moins une ligne.",
        "An order must contain at least one line.",
    ),
    (
        "shop.order.quantity_invalid",
        ErrorKind::Validation,
        &["product"],
        "Quantité invalide pour {product} : elle doit être supérieure à 0.",
        "Invalid quantity for {product}: it must be greater than 0.",
    ),
    (
        "shop.record.invalid",
        ErrorKind::Validation,
        &["field", "reason"],
        "Champ « {field} » : {reason}.",
        "Field \"{field}\": {reason}.",
    ),
    (
        "shop.record.referenced",
        ErrorKind::Conflict,
        &["entity", "id"],
        "Impossible de supprimer : {id} est encore utilisé par {entity}.",
        "Cannot delete: {id} is still used by {entity}.",
    ),
    (
        "shop.record.not_found",
        ErrorKind::NotFound,
        &["entity", "id"],
        "Aucun enregistrement {entity} avec l'identifiant {id}.",
        "No {entity} record with id {id}.",
    ),
    (
        "shop.not_loaded",
        ErrorKind::NotFound,
        &[],
        "Le module shop n'est pas chargé.",
        "The shop module is not loaded.",
    ),
];

/// La boutique complète : cinq entités, trois liens, neuf erreurs, deux locales.
pub fn build_shop() -> (ModuleDescriptor, Vec<MessageBundle>) {
    let mut m = ModuleDescriptor::new(SHOP, SHOP_VERSION);

    let mut customer = EntityDesc::new("customer");
    customer.fields = vec![
        FieldDesc::new("name", FieldType::String),
        FieldDesc::new("email", FieldType::String),
        FieldDesc::new("city", FieldType::String).optional(),
    ];

    let mut product = EntityDesc::new("product");
    product.fields = vec![
        FieldDesc::new("name", FieldType::String),
        FieldDesc::new("sku", FieldType::String),
        FieldDesc::new(
            "category",
            enum_of(
                "product_category",
                &["audio", "peripherique", "ecran", "accessoire", "maison"],
            ),
        ),
        FieldDesc::new("price", FieldType::Decimal { scale: 2 }),
        FieldDesc::new("stock", FieldType::I32),
        FieldDesc::new("active", FieldType::Bool),
    ];

    let mut order = EntityDesc::new("order");
    order.fields = vec![
        FieldDesc::new("number", FieldType::String).read_only(),
        FieldDesc::new(
            "customer",
            FieldType::Ref {
                entity: "customer".into(),
            },
        )
        .read_only(),
        FieldDesc::new(
            "state",
            enum_of("order_state", &["draft", "paid", "shipped", "cancelled"]),
        ),
        FieldDesc::new("total", FieldType::Decimal { scale: 2 }).read_only(),
        FieldDesc::new("placed_on", FieldType::Date)
            .optional()
            .read_only(),
    ];

    let mut item = EntityDesc::new("order_item");
    item.fields = vec![
        // Supprimer une commande supprime ses lignes : sans ça, la commande devient
        // indestructible (sa ligne la référence) — et l'interface ne peut rien y faire.
        FieldDesc::new(
            "order",
            FieldType::Ref {
                entity: "order".into(),
            },
        )
        .read_only()
        .cascade_on_delete(),
        FieldDesc::new(
            "product",
            FieldType::Ref {
                entity: "product".into(),
            },
        )
        .read_only(),
        FieldDesc::new("qty", FieldType::I32).read_only(),
        FieldDesc::new("unit_price", FieldType::Decimal { scale: 2 }).read_only(),
    ];

    // Module de livraison : une expédition concerne une commande. Elle ne possède ni
    // le client ni les produits : elle référence la commande, comme une ligne.
    let mut shipment = EntityDesc::new("shipment");
    shipment.fields = vec![
        FieldDesc::new(
            "order",
            FieldType::Ref {
                entity: "order".into(),
            },
        ),
        FieldDesc::new("carrier", FieldType::String),
        FieldDesc::new("tracking", FieldType::String).optional(),
        FieldDesc::new(
            "state",
            enum_of("shipment_state", &["preparing", "shipped", "delivered"]),
        ),
    ];

    for e in [
        &mut customer,
        &mut product,
        &mut order,
        &mut item,
        &mut shipment,
    ] {
        crud(e);
    }
    // Deux suppressions passent par une permission IAM, pas seulement par l'authentification :
    // le module déclare ainsi ce qu'un front doit demander, et pour quelle opération.
    customer
        .permissions
        .ops
        .insert(Operation::Delete, PolicyExpr::Perm("crm.delete".into()));
    product
        .permissions
        .ops
        .insert(Operation::Delete, PolicyExpr::Perm("catalog.delete".into()));
    m.entities = vec![customer, product, order, item, shipment];

    let mut placed_by = LinkDesc::new("placed_by", "order", "customer");
    placed_by.description = Some("Une commande appartient à un client.".into());
    let mut contains = LinkDesc::new("contains", "order", "product");
    contains.description = Some("Une commande contient des produits, via ses lignes.".into());
    let mut ships = LinkDesc::new("ships", "shipment", "order");
    ships.description = Some("Une expédition concerne une commande.".into());
    m.links = vec![placed_by, contains, ships];

    m.errors = ERRORS
        .iter()
        .map(|(code, kind, params, _, _)| ErrorDesc {
            code: ErrorCode::new(*code),
            kind: *kind,
            message_key: MessageKey::new(*code),
            params: params.iter().map(|p| p.to_string()).collect(),
            description: None,
        })
        .collect();

    m.i18n = I18nDecl {
        default_locale: Locale::new("fr"),
        locales: vec![Locale::new("fr"), Locale::new("en")],
        keys: ERRORS
            .iter()
            .map(|(code, _, params, _, _)| MessageDecl {
                key: MessageKey::new(*code),
                params: params.iter().map(|p| p.to_string()).collect(),
            })
            .collect(),
    };

    let bundle = |locale: &str, pick: fn(&ErrorSpec) -> &'static str| MessageBundle {
        module: SHOP.to_string(),
        locale: Locale::new(locale),
        syntax: Default::default(),
        messages: ERRORS
            .iter()
            .map(|s| (MessageKey::new(s.0), pick(s).to_string()))
            .collect(),
    };
    let bundles = vec![bundle("fr", |s| s.3), bundle("en", |s| s.4)];
    (m, bundles)
}

// ---- Enregistrements : validation et persistance ------------------------------------

type Records = BTreeMap<String, Value>;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn new_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{nanos:x}{n:03x}")
}

/// Date du jour `YYYY-MM-DD` (UTC), sans dépendance de date.
fn today() -> String {
    let days = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        / 86_400) as i64;
    // Algorithme « civil from days » (H. Hinnant).
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

fn type_name(ty: &FieldType) -> &'static str {
    match ty {
        FieldType::String | FieldType::Text => "texte",
        FieldType::I32 | FieldType::I64 | FieldType::U32 | FieldType::U64 => "entier",
        FieldType::F32 | FieldType::F64 | FieldType::Decimal { .. } => "nombre",
        FieldType::Bool => "booléen",
        FieldType::Ref { .. } => "identifiant",
        FieldType::Date => "date AAAA-MM-JJ",
        _ => "valeur",
    }
}

fn check_type(ty: &FieldType, v: &Value) -> Result<(), String> {
    if let FieldType::Enum { variants, .. } = ty {
        return match v.as_str() {
            Some(s) if variants.iter().any(|x| x == s) => Ok(()),
            _ => Err(format!("valeur attendue parmi {}", variants.join(", "))),
        };
    }
    let ok = match ty {
        FieldType::String
        | FieldType::Text
        | FieldType::Uuid
        | FieldType::DateTime
        | FieldType::Date => v.is_string(),
        FieldType::I32 | FieldType::I64 | FieldType::U32 | FieldType::U64 => v.as_i64().is_some(),
        FieldType::F32 | FieldType::F64 => v.is_number(),
        FieldType::Decimal { .. } => decimal(v).is_some(),
        FieldType::Bool => v.is_boolean(),
        FieldType::Ref { .. } => v.is_string(),
        _ => true,
    };
    if ok {
        Ok(())
    } else {
        Err(format!("type attendu : {}", type_name(ty)))
    }
}

/// Valeur décimale : nombre JSON ou chaîne `"89.90"`.
fn decimal(v: &Value) -> Option<f64> {
    v.as_f64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

impl Data {
    pub fn open(app_dir: PathBuf) -> Result<Self, String> {
        let modules = app_dir.join("modules");
        let records_dir = app_dir.join("records");
        std::fs::create_dir_all(&modules).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&records_dir).map_err(|e| e.to_string())?;
        let vfs = LocalFsVfs::open(&modules).map_err(|e| format!("{e:?}"))?;
        let (changes, _) = broadcast::channel(256);
        Ok(Self {
            store: VfsModuleStore::new(Arc::new(vfs)),
            records_dir,
            write_lock: Mutex::new(()),
            changes,
        })
    }

    /// S'abonner aux écritures. Chaque appelant obtient son propre flux.
    pub fn changes(&self) -> broadcast::Receiver<Change> {
        self.changes.subscribe()
    }

    /// Publie sans bloquer : sans abonné, le changement est simplement ignoré.
    fn publish(&self, change: Change) {
        let _ = self.changes.send(change);
    }

    pub fn read_records(&self, entity: &str) -> Records {
        let path = self.records_dir.join(format!("{entity}.json"));
        std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    fn write_records(&self, entity: &str, records: &Records) -> Result<(), AppError> {
        let path = self.records_dir.join(format!("{entity}.json"));
        let tmp = path.with_extension("json.tmp");
        let bytes = serde_json::to_vec_pretty(records).map_err(|e| AppError {
            code: "this.io".into(),
            message: e.to_string(),
        })?;
        std::fs::write(&tmp, bytes).map_err(io_err)?;
        std::fs::rename(&tmp, &path).map_err(io_err)
    }

    fn has_records(&self, entity: &str) -> bool {
        self.records_dir.join(format!("{entity}.json")).exists()
    }

    /// Le module doit être chargé : sinon refus `shop.not_loaded`.
    pub async fn descriptor(&self) -> Result<ModuleDescriptor, AppError> {
        self.store
            .get(SHOP, None)
            .await
            .map_err(|_| fail(None, "shop.not_loaded", &[]))
    }

    pub async fn fr(&self, version: &str) -> Option<MessageBundle> {
        self.store.bundle(SHOP, version, "fr").await.ok().flatten()
    }

    /// Valide un enregistrement contre son entité : requis, types, énumérations, références.
    fn check_record(
        &self,
        ent: &EntityDesc,
        data: &Value,
        fr: Option<&MessageBundle>,
        pending: &[(&str, &str)],
    ) -> Result<Map<String, Value>, AppError> {
        let invalid = |field: &str, reason: String| {
            fail(
                fr,
                "shop.record.invalid",
                &[("field", field.to_string()), ("reason", reason)],
            )
        };
        let obj = data
            .as_object()
            .ok_or_else(|| invalid(&ent.name, "un objet est attendu".into()))?;
        let mut out = Map::new();
        for f in &ent.fields {
            match obj.get(&f.name).filter(|v| !v.is_null()) {
                None => {
                    if !f.optional && f.default.is_none() {
                        return Err(invalid(&f.name, "champ requis".into()));
                    }
                }
                Some(v) => {
                    check_type(&f.ty, v).map_err(|r| invalid(&f.name, r))?;
                    if let FieldType::Ref { entity } = &f.ty {
                        let id = v.as_str().unwrap_or_default();
                        let known = self.read_records(entity).contains_key(id)
                            || pending
                                .iter()
                                .any(|(e, i)| *e == entity.as_str() && *i == id);
                        if !known {
                            return Err(invalid(&f.name, format!("référence inconnue « {id} »")));
                        }
                    }
                    out.insert(f.name.clone(), v.clone());
                }
            }
        }
        Ok(out)
    }
}

fn entity_of<'a>(
    desc: &'a ModuleDescriptor,
    name: &str,
    fr: Option<&MessageBundle>,
) -> Result<&'a EntityDesc, AppError> {
    desc.entities
        .iter()
        .find(|e| e.name == name)
        .ok_or_else(|| {
            fail(
                fr,
                "shop.record.invalid",
                &[("field", name.into()), ("reason", "entité inconnue".into())],
            )
        })
}

// ---- Opérations ----------------------------------------------------------------------

/// Charge la boutique (idempotent) et, à la première fois, y met des données de démo.
pub async fn load_shop(d: &Data) -> Result<(), AppError> {
    if d.store.get(SHOP, Some(SHOP_VERSION)).await.is_err() {
        let (desc, bundles) = build_shop();
        d.store.put(&desc, &bundles).await.map_err(app_err)?;
    }
    if !d.has_records("product") {
        seed(d).await?;
    }
    Ok(())
}

/// Crée ou remplace un enregistrement, après validation par le descripteur.
pub async fn save(
    d: &Data,
    entity: &str,
    id: Option<String>,
    data: Value,
) -> Result<Value, AppError> {
    let desc = d.descriptor().await?;
    let fr = d.fr(&desc.version).await;
    let ent = entity_of(&desc, entity, fr.as_ref())?.clone();
    let _g = d.write_lock.lock().unwrap_or_else(|p| p.into_inner());
    let existing = id
        .as_ref()
        .and_then(|i| d.read_records(entity).get(i).cloned());
    // Un champ en lecture seule ne s'écrit jamais, sauf à sa valeur déjà stockée.
    for f in ent.fields.iter().filter(|f| f.read_only) {
        if let Some(v) = data.get(&f.name).filter(|v| !v.is_null()) {
            let unchanged = existing.as_ref().and_then(|e| e.get(&f.name)) == Some(v);
            if !unchanged {
                return Err(fail(
                    fr.as_ref(),
                    "shop.record.invalid",
                    &[
                        ("field", f.name.clone()),
                        ("reason", "champ en lecture seule".into()),
                    ],
                ));
            }
        }
    }
    let mut obj = d.check_record(&ent, &data, fr.as_ref(), &[])?;
    let id = id.unwrap_or_else(|| new_id(&entity.replace('_', "-")));
    obj.insert("id".into(), json!(id));
    let value = Value::Object(obj);
    let mut records = d.read_records(entity);
    records.insert(id, value.clone());
    d.write_records(entity, &records)?;
    d.publish(match existing.is_some() {
        true => Change::updated(entity, value.clone()),
        false => Change::created(entity, value.clone()),
    });
    Ok(value)
}

/// Supprime un enregistrement, sauf s'il est encore référencé par un autre.
pub async fn remove(d: &Data, entity: &str, id: &str) -> Result<(), AppError> {
    let desc = d.descriptor().await?;
    let fr = d.fr(&desc.version).await;
    let fr = fr.as_ref();

    if !d.read_records(entity).contains_key(id) {
        return Err(fail(
            fr,
            "shop.record.not_found",
            &[("entity", entity.into()), ("id", id.into())],
        ));
    }

    // Ce qui doit partir : la cible, plus ce qui la référence en `cascade`. Une cascade peut
    // révéler d'autres références, donc on répète jusqu'à stabilisation.
    let mut doomed: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    doomed
        .entry(entity.to_string())
        .or_default()
        .insert(id.to_string());
    let mut pending: Vec<(String, String)> = vec![(entity.to_string(), id.to_string())];

    while let Some((target, target_id)) = pending.pop() {
        for other in &desc.entities {
            for field in &other.fields {
                let FieldType::Ref { entity: referenced } = &field.ty else {
                    continue;
                };
                if referenced != &target {
                    continue;
                }
                let referencing: Vec<String> = d
                    .read_records(&other.name)
                    .into_iter()
                    .filter(|(_, record)| {
                        record.get(&field.name).and_then(Value::as_str) == Some(target_id.as_str())
                    })
                    .map(|(key, _)| key)
                    .collect();
                if referencing.is_empty() {
                    continue;
                }
                match field.on_delete {
                    OnDelete::Cascade => {
                        for child in referencing {
                            if doomed
                                .entry(other.name.clone())
                                .or_default()
                                .insert(child.clone())
                            {
                                pending.push((other.name.clone(), child));
                            }
                        }
                    }
                    OnDelete::Restrict => {
                        return Err(fail(
                            fr,
                            "shop.record.referenced",
                            &[("entity", other.name.clone()), ("id", target_id.clone())],
                        ));
                    }
                }
            }
        }
    }

    // Une seule section critique, une écriture par entité touchée.
    let _g = d.write_lock.lock().unwrap_or_else(|p| p.into_inner());
    for (name, ids) in &doomed {
        let mut records = d.read_records(name);
        for doomed_id in ids {
            records.remove(doomed_id);
        }
        d.write_records(name, &records)?;
        // après l'écriture seulement : un abonné qui relit voit déjà l'état d'après
        for doomed_id in ids {
            d.publish(Change::deleted(name, doomed_id));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineIn {
    pub product_id: String,
    pub qty: i64,
}

/// Passe une commande : vérifie le stock de chaque ligne, calcule le total, décrémente le stock.
/// Tout est vérifié avant la moindre écriture.
pub async fn place_order(
    d: &Data,
    customer_id: &str,
    lines: &[LineIn],
    state: &str,
) -> Result<Value, AppError> {
    let desc = d.descriptor().await?;
    let fr = d.fr(&desc.version).await;
    let fr = fr.as_ref();
    let _g = d.write_lock.lock().unwrap_or_else(|p| p.into_inner());

    if lines.is_empty() {
        return Err(fail(fr, "shop.order.empty", &[]));
    }
    if !d.read_records("customer").contains_key(customer_id) {
        return Err(fail(
            fr,
            "shop.customer.not_found",
            &[("customer", customer_id.into())],
        ));
    }

    // Copie de travail des produits : le stock n'est écrit qu'à la fin.
    let mut products = d.read_records("product");
    let mut total = 0.0_f64;
    let mut items: Vec<(String, i64, f64)> = Vec::new();
    for line in lines {
        let name_of = |p: &Value| p["name"].as_str().unwrap_or_default().to_string();
        let Some(p) = products.get(&line.product_id).cloned() else {
            return Err(fail(
                fr,
                "shop.product.not_found",
                &[("product", line.product_id.clone())],
            ));
        };
        if line.qty <= 0 {
            return Err(fail(
                fr,
                "shop.order.quantity_invalid",
                &[("product", name_of(&p))],
            ));
        }
        let available = p["stock"].as_i64().unwrap_or(0);
        if available < line.qty {
            return Err(fail(
                fr,
                "shop.product.out_of_stock",
                &[
                    ("product", name_of(&p)),
                    ("available", available.to_string()),
                ],
            ));
        }
        let price = decimal(&p["price"]).unwrap_or(0.0);
        total += price * line.qty as f64;
        items.push((line.product_id.clone(), line.qty, price));
        // Une même référence répétée dans la commande reste cohérente.
        if let Some(entry) = products.get_mut(&line.product_id) {
            entry["stock"] = json!(available - line.qty);
        }
    }

    let order_ent = entity_of(&desc, "order", fr)?.clone();
    let item_ent = entity_of(&desc, "order_item", fr)?.clone();
    let order_id = new_id("ord");
    let mut orders = d.read_records("order");
    let number = format!("CMD-{:04}", orders.len() + 1);

    // Validation de tout le lot avant la moindre écriture.
    let mut order_obj = d.check_record(
        &order_ent,
        &json!({
            "number": number,
            "customer": customer_id,
            "state": state,
            "total": format!("{total:.2}"),
            "placed_on": today(),
        }),
        fr,
        &[],
    )?;
    order_obj.insert("id".into(), json!(order_id));

    let mut new_items: Vec<(String, Value)> = Vec::new();
    for (product_id, qty, price) in &items {
        let line_id = new_id("line");
        let mut obj = d.check_record(
            &item_ent,
            &json!({
                "order": order_id,
                "product": product_id,
                "qty": qty,
                "unit_price": format!("{price:.2}"),
            }),
            fr,
            &[("order", order_id.as_str())],
        )?;
        obj.insert("id".into(), json!(line_id));
        new_items.push((line_id, Value::Object(obj)));
    }

    // Écritures : stock, commande, lignes.
    d.write_records("product", &products)?;
    orders.insert(order_id.clone(), Value::Object(order_obj.clone()));
    d.write_records("order", &orders)?;
    let mut lines_store = d.read_records("order_item");
    lines_store.extend(new_items.iter().cloned());
    d.write_records("order_item", &lines_store)?;

    // Les abonnés voient la commande, ses lignes et le stock corrigé — sans redemander.
    d.publish(Change::created("order", Value::Object(order_obj.clone())));
    for (_, line) in &new_items {
        d.publish(Change::created("order_item", line.clone()));
    }
    for (product_id, _, _) in &items {
        if let Some(product) = products.get(product_id) {
            d.publish(Change::updated("product", product.clone()));
        }
    }
    Ok(Value::Object(order_obj))
}

/// Données de démonstration : un catalogue réaliste, des clients, trois commandes.
async fn seed(d: &Data) -> Result<(), AppError> {
    let products = [
        (
            "prod-casque",
            "Casque audio sans fil",
            "AUD-CAS-01",
            "audio",
            "89.90",
            14,
        ),
        (
            "prod-clavier",
            "Clavier mécanique 75 %",
            "PER-CLV-75",
            "peripherique",
            "129.00",
            6,
        ),
        (
            "prod-souris",
            "Souris ergonomique",
            "PER-SOU-ERG",
            "peripherique",
            "49.50",
            22,
        ),
        (
            "prod-ecran",
            "Écran 27\" 4K",
            "ECR-27-4K",
            "ecran",
            "349.00",
            4,
        ),
        (
            "prod-webcam",
            "Webcam Full HD",
            "PER-WEB-FHD",
            "peripherique",
            "59.90",
            0,
        ),
        (
            "prod-sac",
            "Sac à dos 20 L",
            "ACC-SAC-20",
            "accessoire",
            "39.00",
            31,
        ),
        (
            "prod-chargeur",
            "Chargeur USB-C 65 W",
            "ACC-CHG-65",
            "accessoire",
            "35.00",
            48,
        ),
        (
            "prod-lampe",
            "Lampe de bureau LED",
            "MAI-LMP-LED",
            "maison",
            "42.00",
            9,
        ),
    ];
    for (id, name, sku, category, price, stock) in products {
        save(
            d,
            "product",
            Some(id.into()),
            json!({"name": name, "sku": sku, "category": category, "price": price, "stock": stock, "active": true}),
        )
        .await?;
    }
    let customers = [
        (
            "cust-camille",
            "Camille Martin",
            "camille.martin@example.fr",
            "Lyon",
        ),
        (
            "cust-hugo",
            "Hugo Bernard",
            "hugo.bernard@example.fr",
            "Paris",
        ),
        ("cust-lea", "Léa Dubois", "lea.dubois@example.fr", "Nantes"),
        (
            "cust-yanis",
            "Yanis Petit",
            "yanis.petit@example.fr",
            "Bordeaux",
        ),
        (
            "cust-ines",
            "Inès Moreau",
            "ines.moreau@example.fr",
            "Lille",
        ),
    ];
    for (id, name, email, city) in customers {
        save(
            d,
            "customer",
            Some(id.into()),
            json!({"name": name, "email": email, "city": city}),
        )
        .await?;
    }
    let line = |p: &str, q: i64| LineIn {
        product_id: p.into(),
        qty: q,
    };
    place_order(
        d,
        "cust-camille",
        &[line("prod-casque", 1), line("prod-souris", 1)],
        "paid",
    )
    .await?;
    place_order(
        d,
        "cust-hugo",
        &[
            line("prod-ecran", 1),
            line("prod-clavier", 1),
            line("prod-chargeur", 2),
        ],
        "shipped",
    )
    .await?;
    place_order(
        d,
        "cust-lea",
        &[line("prod-sac", 1), line("prod-lampe", 1)],
        "draft",
    )
    .await?;
    Ok(())
}

// ---- Surface commune aux adaptateurs ------------------------------------------------

/// État du module dans le store. Même forme JSON pour Tauri et pour HTTP.
#[derive(Debug, Clone, Serialize)]
pub struct Status {
    pub loaded: bool,
    pub version: &'static str,
}

/// L'interface d'une version : le descripteur et ses bundles de messages.
#[derive(Debug, Clone, Serialize)]
pub struct Interface {
    pub descriptor: ModuleDescriptor,
    pub bundles: Vec<MessageBundle>,
}

/// Le module est-il chargé ?
pub async fn status(d: &Data) -> Status {
    Status {
        // la version chargée doit être CELLE DU CODE : sinon une app qui a déjà un store
        // ne verrait jamais une évolution du descripteur
        loaded: d.store.get(SHOP, Some(SHOP_VERSION)).await.is_ok(),
        version: SHOP_VERSION,
    }
}

/// Charge (ou recharge) la boutique, puis renvoie son statut.
pub async fn load_and_report(d: &Data) -> Result<Status, AppError> {
    load_shop(d).await?;
    Ok(Status {
        loaded: true,
        version: SHOP_VERSION,
    })
}

/// Descripteur + bundles, tels que le front les consomme.
pub async fn interface(d: &Data) -> Result<Interface, AppError> {
    let descriptor = d.descriptor().await?;
    let mut bundles = Vec::new();
    for locale in &descriptor.i18n.locales {
        if let Some(b) = d
            .store
            .bundle(SHOP, &descriptor.version, locale.as_str())
            .await
            .map_err(app_err)?
        {
            bundles.push(b);
        }
    }
    Ok(Interface {
        descriptor,
        bundles,
    })
}

// ---- Ce que la session peut faire, d'après le contrat du module ----------------------

pub use this::descriptor::{
    Capabilities, Capability, Gap, OnDelete, Operation, PermissionDecision, PermissionRequest,
    PolicyExpr, Requirement, Session, SessionKind,
};

/// Capacités calculées depuis le descripteur du module et la session de l'appelant.
pub async fn capabilities(d: &Data, session: &Session) -> Result<Capabilities, AppError> {
    let descriptor = d.descriptor().await?;
    Ok(Capabilities::for_session(&descriptor, session))
}

/// Refuse une opération que le module n'autorise pas pour cette session.
///
/// Le front grise ce qu'il ne peut pas faire ; la surface le refuse. Les deux lisent la
/// **même** politique du descripteur, mais seule la surface fait foi.
pub async fn guard(
    d: &Data,
    session: &Session,
    entity: &str,
    op: Operation,
) -> Result<(), AppError> {
    if capabilities(d, session)
        .await?
        .allows(entity, &op)
        .is_allowed()
    {
        return Ok(());
    }
    Err(AppError {
        code: "this.policy.denied".into(),
        message: format!(
            "Action « {} » refusée sur « {} » pour la session « {} ».",
            op.as_str(),
            entity,
            session.label()
        ),
    })
}

/// La politique déclarée pour une opération d'une entité (`None` si l'entité n'existe pas).
pub async fn policy(
    d: &Data,
    entity: &str,
    op: &Operation,
) -> Result<Option<PolicyExpr>, AppError> {
    let descriptor = d.descriptor().await?;
    Ok(descriptor
        .entities
        .iter()
        .find(|e| e.name == entity)
        .map(|e| e.permissions.effective(op)))
}

/// La session d'une surface et les autorisations accordées pendant la vie du processus.
///
/// Les deux adaptateurs (Tauri, axum) partagent ce type : la décision d'accorder ou non une
/// autorisation demandée depuis l'interface ne doit exister qu'une fois.
pub struct Sessions {
    kind: SessionKind,
    roles: Vec<String>,
    grants: Mutex<BTreeSet<String>>,
}

impl Sessions {
    pub fn new(
        kind: SessionKind,
        roles: Vec<String>,
        grants: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            kind,
            roles,
            grants: Mutex::new(grants.into_iter().collect()),
        }
    }

    /// Ce que la surface transmet au front : l'identité, et la liste **complète** des
    /// permissions (accordées comprises), pour que les politiques `perm:…` soient décidables.
    pub fn session(&self) -> Session {
        let grants = self
            .grants
            .lock()
            .map(|g| g.iter().cloned().collect())
            .unwrap_or_default();
        Session {
            kind: self.kind,
            roles: self.roles.clone(),
            grants: Some(grants),
        }
    }

    fn grant(&self, permission: &str) -> bool {
        self.grants
            .lock()
            .map(|mut grants| grants.insert(permission.to_string()))
            .unwrap_or(false)
    }

    /// Traite une demande d'autorisation venue de l'interface.
    ///
    /// N'accorde **que** ce que la politique de l'opération réclame réellement : une demande
    /// ne peut pas élargir les droits au-delà de ce que le module a déclaré.
    pub async fn request(
        &self,
        d: &Data,
        req: &PermissionRequest,
    ) -> Result<(PermissionDecision, String), AppError> {
        let session = self.session();
        if !session.is_authenticated() {
            return Ok((
                PermissionDecision::Denied,
                "Connectez-vous pour demander cette autorisation.".into(),
            ));
        }

        let Some(policy) = policy(d, &req.entity, &req.op).await? else {
            return Ok((
                PermissionDecision::Denied,
                format!("Entité inconnue : {}.", req.entity),
            ));
        };
        let needed = policy.permissions();
        if needed.is_empty() {
            return Ok((
                PermissionDecision::Denied,
                "Cette action ne dépend d'aucune autorisation demandable.".into(),
            ));
        }

        let mut granted = Vec::new();
        for permission in &needed {
            if self.grant(permission) {
                granted.push((*permission).to_string());
            }
        }

        let session = self.session();
        let allowed = capabilities(d, &session)
            .await?
            .allows(&req.entity, &req.op)
            .is_allowed();
        if allowed {
            Ok((
                PermissionDecision::Granted,
                format!("Autorisation accordée : {}.", granted.join(", ")),
            ))
        } else {
            Ok((
                PermissionDecision::Pending,
                format!(
                    "Demande enregistrée : il manque encore {}.",
                    needed.join(", ")
                ),
            ))
        }
    }
}

/// Enregistrements d'une entité, chacun portant son `id`.
pub fn list_records(d: &Data, entity: &str) -> Vec<Value> {
    d.read_records(entity)
        .into_iter()
        .map(|(id, mut v)| {
            if let Value::Object(o) = &mut v {
                o.insert("id".into(), json!(id));
            }
            v
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use this_app::ChangeOp;

    fn temp_data(tag: &str) -> (Data, PathBuf) {
        let dir = std::env::temp_dir().join(format!("shop-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let data = Data::open(dir.clone()).unwrap();
        (data, dir)
    }

    fn line(p: &str, q: i64) -> LineIn {
        LineIn {
            product_id: p.into(),
            qty: q,
        }
    }

    #[tokio::test]
    async fn module_and_bundles_land_in_one_put() {
        let (d, dir) = temp_data("interface");
        load_shop(&d).await.unwrap();
        let desc = d.descriptor().await.unwrap();
        assert_eq!(desc.entities.len(), 5);
        assert_eq!(desc.links.len(), 3);
        assert_eq!(desc.errors.len(), 9);
        let fr = d.fr(SHOP_VERSION).await.expect("bundle fr");
        assert!(fr
            .get(&MessageKey::new("shop.product.out_of_stock"))
            .unwrap()
            .contains("Stock insuffisant"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn an_anonymous_session_is_refused_by_the_module_policies() {
        let (d, dir) = temp_data("policy");
        load_shop(&d).await.unwrap();
        // le module déclare `authenticated` sur ses opérations CRUD
        let caps = capabilities(&d, &Session::anonymous()).await.unwrap();
        assert_eq!(caps.allows("order", &Operation::Create), Capability::Denied);
        assert!(!caps.any_write());

        let err = guard(&d, &Session::anonymous(), "order", Operation::Create)
            .await
            .unwrap_err();
        assert_eq!(err.code, "this.policy.denied");
        // et la même garde laisse passer une session utilisateur
        guard(&d, &Session::user(), "order", Operation::Create)
            .await
            .unwrap();
        assert!(capabilities(&d, &Session::user())
            .await
            .unwrap()
            .any_write());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn a_permission_requested_in_app_unlocks_the_action() {
        let (d, dir) = temp_data("request");
        load_shop(&d).await.unwrap();
        let sessions = Sessions::new(SessionKind::User, vec![], Vec::new());

        // au départ : la surface transmet ses permissions, et n'en a aucune
        let caps = capabilities(&d, &sessions.session()).await.unwrap();
        assert_eq!(
            caps.allows("customer", &Operation::Delete),
            Capability::Denied
        );
        assert!(
            guard(&d, &sessions.session(), "customer", Operation::Delete)
                .await
                .is_err()
        );

        // demande in-app : le module réclame `crm.delete`, la surface l'accorde
        let req = PermissionRequest {
            entity: "customer".into(),
            op: Operation::Delete,
            requirement: Requirement::Required,
            reason: "Supprimer une fiche client depuis l'écran Clients".into(),
        };
        let (decision, message) = sessions.request(&d, &req).await.unwrap();
        assert_eq!(decision, PermissionDecision::Granted);
        assert!(message.contains("crm.delete"), "{message}");

        // la session suivante porte la permission : le front peut recalculer, la garde passe
        let session = sessions.session();
        assert_eq!(session.has_grant("crm.delete"), Some(true));
        assert!(capabilities(&d, &session)
            .await
            .unwrap()
            .allows("customer", &Operation::Delete)
            .is_allowed());
        guard(&d, &session, "customer", Operation::Delete)
            .await
            .unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn a_request_grants_only_what_the_module_declares() {
        let (d, dir) = temp_data("request-scope");
        load_shop(&d).await.unwrap();
        let sessions = Sessions::new(SessionKind::User, vec![], Vec::new());

        let ask = |entity: &str, op: Operation| PermissionRequest {
            entity: entity.into(),
            op,
            requirement: Requirement::Optional,
            reason: "test".into(),
        };

        // `order.delete` ne dépend d'aucune permission : il n'y a rien à accorder
        let (decision, message) = sessions
            .request(&d, &ask("order", Operation::Delete))
            .await
            .unwrap();
        assert_eq!(decision, PermissionDecision::Denied);
        assert!(message.contains("aucune autorisation"), "{message}");

        // une entité inconnue ne donne rien
        assert_eq!(
            sessions
                .request(&d, &ask("fantome", Operation::Delete))
                .await
                .unwrap()
                .0,
            PermissionDecision::Denied
        );
        assert!(sessions.session().grants.unwrap().is_empty());

        // un visiteur non connecté ne peut pas demander
        let anon = Sessions::new(SessionKind::Anonymous, vec![], Vec::new());
        assert_eq!(
            anon.request(&d, &ask("customer", Operation::Delete))
                .await
                .unwrap()
                .0,
            PermissionDecision::Denied
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn seed_gives_a_working_shop_and_is_idempotent() {
        let (d, dir) = temp_data("seed");
        load_shop(&d).await.unwrap();
        assert_eq!(d.read_records("product").len(), 8);
        assert_eq!(d.read_records("customer").len(), 5);
        assert_eq!(d.read_records("order").len(), 3);
        load_shop(&d).await.unwrap();
        assert_eq!(d.read_records("order").len(), 3);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn order_decrements_stock_and_totals_correctly() {
        let (d, dir) = temp_data("order");
        load_shop(&d).await.unwrap();
        let before = d.read_records("product")["prod-souris"]["stock"]
            .as_i64()
            .unwrap();
        let order = place_order(&d, "cust-lea", &[line("prod-souris", 2)], "draft")
            .await
            .unwrap();
        assert_eq!(order["total"], "99.00");
        let after = d.read_records("product")["prod-souris"]["stock"]
            .as_i64()
            .unwrap();
        assert_eq!(after, before - 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn out_of_stock_is_refused_with_the_french_message_and_writes_nothing() {
        let (d, dir) = temp_data("oos");
        load_shop(&d).await.unwrap();
        let orders_before = d.read_records("order").len();
        let souris_before = d.read_records("product")["prod-souris"]["stock"].clone();
        let err = place_order(
            &d,
            "cust-hugo",
            &[line("prod-souris", 1), line("prod-webcam", 1)],
            "draft",
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "shop.product.out_of_stock");
        assert_eq!(
            err.message,
            "Stock insuffisant pour Webcam Full HD : 0 disponible(s)."
        );
        assert_eq!(d.read_records("order").len(), orders_before);
        assert_eq!(
            d.read_records("product")["prod-souris"]["stock"],
            souris_before
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn record_validation_uses_the_descriptor() {
        let (d, dir) = temp_data("validate");
        load_shop(&d).await.unwrap();
        let e = save(
            &d,
            "product",
            None,
            json!({"sku": "X", "category": "audio", "price": "1.00", "stock": 1, "active": true}),
        )
        .await
        .unwrap_err();
        assert!(e.message.contains("champ requis"), "{}", e.message);
        let e = save(&d, "product", None, json!({"name": "X", "sku": "X", "category": "jouets", "price": "1.00", "stock": 1, "active": true}))
            .await
            .unwrap_err();
        assert!(e.message.contains("valeur attendue"), "{}", e.message);
        let e = place_order(&d, "cust-fantome", &[line("prod-souris", 1)], "draft")
            .await
            .unwrap_err();
        assert_eq!(e.code, "shop.customer.not_found");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn writes_are_published_to_subscribers() {
        let (d, dir) = temp_data("changes");
        load_shop(&d).await.unwrap();
        seed(&d).await.unwrap();
        let mut rx = d.changes();

        let order = place_order(
            &d,
            "cust-camille",
            &[LineIn {
                product_id: "prod-casque".into(),
                qty: 2,
            }],
            "draft",
        )
        .await
        .unwrap();
        let order_id = order["id"].as_str().unwrap().to_string();

        let mut seen = Vec::new();
        while let Ok(change) = rx.try_recv() {
            seen.push((change.entity.clone(), change.op));
        }
        assert!(
            seen.iter()
                .any(|(entity, op)| entity == "order" && *op == ChangeOp::Created),
            "commande : {seen:?}"
        );
        assert!(seen
            .iter()
            .any(|(entity, op)| entity == "order_item" && *op == ChangeOp::Created));
        assert!(
            seen.iter()
                .any(|(entity, op)| entity == "product" && *op == ChangeOp::Updated),
            "le stock a changé : {seen:?}"
        );

        remove(&d, "order", &order_id).await.unwrap();
        let mut deleted = Vec::new();
        while let Ok(change) = rx.try_recv() {
            if change.op == ChangeOp::Deleted {
                deleted.push(change.entity);
            }
        }
        assert!(deleted.contains(&"order".to_string()), "{deleted:?}");
        assert!(deleted.contains(&"order_item".to_string()), "{deleted:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn deleting_an_order_deletes_its_lines() {
        let (d, dir) = temp_data("cascade");
        load_shop(&d).await.unwrap();
        seed(&d).await.unwrap();

        let order = place_order(
            &d,
            "cust-camille",
            &[LineIn {
                product_id: "prod-casque".into(),
                qty: 1,
            }],
            "draft",
        )
        .await
        .unwrap();
        let order_id = order["id"].as_str().unwrap().to_string();

        let lines_before = list_records(&d, "order_item").len();
        let mine_before = list_records(&d, "order_item")
            .iter()
            .filter(|i| i["order"] == order["id"])
            .count();
        assert_eq!(mine_before, 1);

        remove(&d, "order", &order_id).await.unwrap();

        assert!(!d.read_records("order").contains_key(&order_id));
        let lines_after = list_records(&d, "order_item").len();
        assert_eq!(
            lines_after,
            lines_before - 1,
            "la ligne de commande est partie avec elle"
        );
        assert!(list_records(&d, "order_item")
            .iter()
            .all(|i| i["order"] != order["id"]));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn cannot_delete_a_referenced_customer() {
        let (d, dir) = temp_data("refs");
        load_shop(&d).await.unwrap();
        let e = remove(&d, "customer", "cust-camille").await.unwrap_err();
        assert_eq!(e.code, "shop.record.referenced");
        save(
            &d,
            "customer",
            Some("cust-test".into()),
            json!({"name": "T", "email": "t@x.fr"}),
        )
        .await
        .unwrap();
        remove(&d, "customer", "cust-test").await.unwrap();
        assert!(!d.read_records("customer").contains_key("cust-test"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Seed manuel du store réel de l'app : charge la boutique et ses données de démo.
    /// `cargo test -p desktop_modules seed_app_shop -- --ignored`
    #[tokio::test]
    #[ignore]
    async fn seed_app_shop() {
        let home = std::env::var("HOME").unwrap();
        let dir =
            PathBuf::from(home).join("Library/Application Support/dev.thisrs.desktop-modules");
        let d = Data::open(dir).unwrap();
        load_shop(&d).await.unwrap();
    }

    #[test]
    fn today_is_a_plausible_date() {
        let t = today();
        assert_eq!(t.len(), 10);
        assert!(t.starts_with("20"), "{t}");
    }
}

#[cfg(test)]
mod browser_seed {
    //! `ui/assets/shop-interface.json` est le point de départ du navigateur : la démo
    //! construit le module à partir de ce fichier. Il doit rester identique à ce que la
    //! boutique déclare, sinon la démo et le serveur divergent sans qu'on le voie.
    use super::*;

    #[test]
    fn the_browser_seed_is_exactly_the_shop_declaration() {
        let seed: serde_json::Value =
            serde_json::from_str(include_str!("../../ui/assets/shop-interface.json")).unwrap();
        let (desc, bundles) = build_shop();
        assert_eq!(seed["descriptor"], serde_json::to_value(&desc).unwrap());
        assert_eq!(seed["bundles"], serde_json::to_value(&bundles).unwrap());
    }
}
