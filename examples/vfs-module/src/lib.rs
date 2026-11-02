//! # `vfs-module` — charger, remplacer et décharger un module depuis un `ModuleStore`
//!
//! Un module déclaratif (`ModuleDescriptor`) vit dans un [`ModuleStore`] (ici un `Vfs`,
//! en mémoire ou sur disque). Ce que démontre cet exemple, c'est le **cycle de vie**
//! d'un module stocké, identique quelle que soit l'implémentation du `Vfs` :
//!
//! 1. **charger** — `put("0.1.0")`, puis `get(name, None)` sert la version stockée ;
//! 2. **remplacer** — `put("0.2.0")` ; `get(name, None)` sert désormais la **plus haute**
//!    version (sémantique du store), et `get(name, Some("0.1.0"))` sert encore l'ancienne ;
//! 3. **lister** — `list()` énumère `(nom, version)` ;
//! 4. **décharger** — `remove("0.2.0")` fait retomber `get(None)` sur `0.1.0` ;
//!    `remove("0.1.0")` fait répondre le store `this.vfs.not_found`.
//!
//! Le tombstone HTTP `this.module.not_found` relève de `ModuleRuntime`, pas du store.
//!
//! Le rendu **HTTP** de ce cycle (une route module qui répond 404 après déchargement)
//! est l'exemple `hot_module` du dépôt `this` ; ici, on mesure le contrat du store
//! lui-même, sans serveur.

use std::path::Path;
use std::sync::Arc;

use this::descriptor::field::{FieldDesc, FieldType};
use this::descriptor::{EntityDesc, ModuleDescriptor, Operation, PolicyExpr};
use this::storage::vfs::{InMemoryVfs, LocalFsVfs, ModuleStore, VfsModuleStore};

pub use this::descriptor::framework::codes;

/// Le nom du module stocké.
pub const MODULE: &str = "billing";

/// Un module minimal mais valide à la version demandée.
fn module(version: &str) -> ModuleDescriptor {
    let mut m = ModuleDescriptor::new(MODULE, version);
    let mut e = EntityDesc::new("invoice");
    e.fields = vec![FieldDesc::new("amount", FieldType::F64).tag(16)];
    for op in Operation::CRUD {
        e.permissions.ops.insert(op, PolicyExpr::Authenticated);
    }
    m.entities.push(e);
    m
}

/// Ce qu'un cycle de vie a observé à chaque étape.
#[derive(Debug, Clone, Default)]
pub struct Lifecycle {
    /// Version servie par `get(None)` après le `put("0.1.0")`.
    pub after_first_put: Option<String>,
    /// Version servie après `put("0.2.0")` — la plus haute.
    pub after_second_put: Option<String>,
    /// Version servie par `get(Some("0.1.0"))` — une version précise reste adressable.
    pub explicit_old_version: Option<String>,
    /// `list()` après les deux `put`.
    pub listed: Vec<(String, String)>,
    /// Version servie après `remove("0.2.0")` — retombée sur `0.1.0`.
    pub after_remove_new: Option<String>,
    /// Code d'erreur de `get(None)` après `remove("0.1.0")` — le module n'existe plus.
    pub final_error: Option<String>,
}

impl Lifecycle {
    /// Le cycle s'est-il comporté exactement comme le contrat du store le dit ?
    pub fn is_expected(&self) -> bool {
        self.after_first_put.as_deref() == Some("0.1.0")
            && self.after_second_put.as_deref() == Some("0.2.0")
            && self.explicit_old_version.as_deref() == Some("0.1.0")
            && self.listed.iter().any(|(n, v)| n == MODULE && v == "0.1.0")
            && self.listed.iter().any(|(n, v)| n == MODULE && v == "0.2.0")
            && self.after_remove_new.as_deref() == Some("0.1.0")
            && self.final_error.as_deref() == Some(codes::VFS_NOT_FOUND)
    }
}

/// Version servie par `get(None)`, ou le code d'erreur.
async fn served_version(store: &dyn ModuleStore) -> Result<String, String> {
    store
        .get(MODULE, None)
        .await
        .map(|d| d.version)
        .map_err(|e| e.code.0.to_string())
}

/// Joue le cycle de vie complet contre n'importe quel `ModuleStore`.
pub async fn run_on(store: &dyn ModuleStore) -> Lifecycle {
    let mut l = Lifecycle::default();

    store.put(&module("0.1.0"), &[]).await.expect("put 0.1.0");
    l.after_first_put = served_version(store).await.ok();

    // « Remplacer » = publier une version supérieure ; le store garde les deux.
    store.put(&module("0.2.0"), &[]).await.expect("put 0.2.0");
    l.after_second_put = served_version(store).await.ok();
    l.explicit_old_version = store
        .get(MODULE, Some("0.1.0"))
        .await
        .ok()
        .map(|d| d.version);
    l.listed = store.list().await.expect("list");

    // Décharger la version courante : la précédente redevient la plus haute servie.
    store.remove(MODULE, "0.2.0").await.expect("remove 0.2.0");
    l.after_remove_new = served_version(store).await.ok();

    // Décharger la dernière version : le module n'existe plus.
    store.remove(MODULE, "0.1.0").await.expect("remove 0.1.0");
    l.final_error = served_version(store).await.err();

    l
}

/// Le même cycle sur un `Vfs` en mémoire.
pub async fn lifecycle_in_memory() -> Lifecycle {
    let store = VfsModuleStore::new(Arc::new(InMemoryVfs::new()));
    run_on(&store).await
}

/// Le même cycle sur un `Vfs` local (répertoire `dir`).
pub async fn lifecycle_on_disk(dir: &Path) -> Lifecycle {
    let vfs = LocalFsVfs::open(dir).expect("open LocalFsVfs");
    let store = VfsModuleStore::new(Arc::new(vfs));
    run_on(&store).await
}
