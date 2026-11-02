//! # `module-signing` — un module stocké n'est chargé que s'il est signé
//!
//! La signature d'un module répond à une attaque précise : un opérateur qui peut
//! **écrire** dans le `ModuleStore` (fichiers locaux, objet distant, base…) ne doit
//! pas pouvoir **inventer** un module. Sans ancre de confiance, le store ne vérifie
//! que l'**intégrité** — les fichiers correspondent aux etags d'un manifeste que
//! quiconque a l'écriture peut réécrire. Avec une ancre, le manifeste doit porter
//! une signature d'une clé connue.
//!
//! Quatre issues, mesurées sur le même module `billing` :
//!
//! | Scénario | Ancre | Résultat | Code |
//! |---|---|---|---|
//! | signé, ancre qui connaît la clé | `release-2026` → clé du signataire | **accepté** | — |
//! | signature forgée sous une clé connue | `release-2026` → autre clé | refusé | `this.module.bad_signature` |
//! | clé inconnue de l'ancre | `someone-else` | refusé | `this.module.untrusted_key` |
//! | module non signé | ancre déclarée | refusé | `this.module.unsigned` |

use std::sync::Arc;

use ed25519_dalek::SigningKey;
use this::descriptor::field::{FieldDesc, FieldType};
use this::descriptor::{EntityDesc, ModuleDescriptor, Operation, PolicyExpr};
use this::storage::vfs::module_store::TrustAnchor;
use this::storage::vfs::{InMemoryVfs, ModuleStore, VfsModuleStore};

pub use this::descriptor::framework::codes;

/// L'identifiant de clé sous lequel le module est signé et publié.
pub const KEY_ID: &str = "release-2026";

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[7u8; 32])
}

fn other_key() -> SigningKey {
    SigningKey::from_bytes(&[9u8; 32])
}

/// Un module minimal mais valide : l'entité `invoice`, un champ, des politiques CRUD.
fn descriptor() -> ModuleDescriptor {
    let mut m = ModuleDescriptor::new("billing", "0.0.1");
    let mut e = EntityDesc::new("invoice");
    e.fields = vec![FieldDesc::new("amount", FieldType::F64).tag(16)];
    for op in Operation::CRUD {
        e.permissions.ops.insert(op, PolicyExpr::Authenticated);
    }
    m.entities.push(e);
    m
}

/// L'ancre qui fait confiance à `key` sous [`KEY_ID`].
fn trusting(key: &SigningKey) -> TrustAnchor {
    let mut anchor = TrustAnchor::new();
    anchor.insert(KEY_ID, key.verifying_key());
    anchor
}

/// Publie le module `billing` signé par `signing_key()` sous [`KEY_ID`].
async fn store_with_signed_module() -> Arc<InMemoryVfs> {
    let vfs = Arc::new(InMemoryVfs::new());
    VfsModuleStore::new(vfs.clone())
        .with_signing_key(KEY_ID, signing_key())
        .put(&descriptor(), &[])
        .await
        .expect("put signed module");
    vfs
}

/// Le résultat lisible d'un scénario.
#[derive(Debug, Clone)]
pub struct Outcome {
    pub label: &'static str,
    pub expected_accept: bool,
    pub accepted: bool,
    pub code: Option<String>,
}

impl Outcome {
    fn from_get(label: &'static str, expected_accept: bool, result: Result<(), String>) -> Self {
        match result {
            Ok(()) => Self {
                label,
                expected_accept,
                accepted: true,
                code: None,
            },
            Err(code) => Self {
                label,
                expected_accept,
                accepted: false,
                code: Some(code),
            },
        }
    }

    pub fn as_expected(&self) -> bool {
        self.accepted == self.expected_accept
    }
}

/// Tente de charger `name` sous `anchor`, en ne gardant que le code d'erreur.
async fn load_code(vfs: Arc<InMemoryVfs>, anchor: TrustAnchor, name: &str) -> Result<(), String> {
    let reader = VfsModuleStore::new(vfs).with_trust_anchor(anchor);
    reader
        .get(name, None)
        .await
        .map(|_| ())
        .map_err(|e| e.code.0.to_string())
}

pub async fn signed_module_is_accepted() -> Outcome {
    let vfs = store_with_signed_module().await;
    Outcome::from_get(
        "signé, ancre qui connaît la clé",
        true,
        load_code(vfs, trusting(&signing_key()), "billing").await,
    )
}

pub async fn forged_signature_is_refused() -> Outcome {
    let vfs = store_with_signed_module().await;
    // L'ancre connaît bien `KEY_ID`, mais détient une autre clé : la signature
    // ne se vérifie pas → falsification sous un nom de confiance.
    Outcome::from_get(
        "signature forgée sous une clé connue",
        false,
        load_code(vfs, trusting(&other_key()), "billing").await,
    )
}

pub async fn unknown_key_id_is_refused() -> Outcome {
    let vfs = store_with_signed_module().await;
    // L'ancre ne connaît pas du tout le `key_id` du manifeste.
    let mut anchor = TrustAnchor::new();
    anchor.insert("someone-else", other_key().verifying_key());
    Outcome::from_get(
        "clé inconnue de l'ancre",
        false,
        load_code(vfs, anchor, "billing").await,
    )
}

pub async fn unsigned_module_is_refused() -> Outcome {
    let vfs = Arc::new(InMemoryVfs::new());
    VfsModuleStore::new(vfs.clone())
        .put(&descriptor(), &[])
        .await
        .expect("put unsigned module");
    Outcome::from_get(
        "non signé, ancre déclarée",
        false,
        load_code(vfs, trusting(&signing_key()), "billing").await,
    )
}

/// Exécute les quatre scénarios, dans l'ordre du tableau de tête.
pub async fn run_all() -> Vec<Outcome> {
    vec![
        signed_module_is_accepted().await,
        forged_signature_is_refused().await,
        unknown_key_id_is_refused().await,
        unsigned_module_is_refused().await,
    ]
}
