//! # module-runtime — construire un module à l'exécution
//!
//! Un module est stocké sous forme de données (`descriptor.json` + bundles de textes),
//! puis **construit** au moment où on en a besoin : lu, parsé, vérifié selon le profil
//! `Stored` (chaque opération CRUD a une politique explicite, `public` refusé sauf
//! autorisation de l'hôte). Le store est un **trait** : la démo tourne sur
//! [`InMemoryStore`] ; un store adossé à Obrain (ou IndexedDB) s'y branchera sans
//! toucher à la construction.
//!
//! Ce crate n'a ni tokio, ni I/O, ni serveur : il compile pour `wasm32-unknown-unknown`.

use std::collections::BTreeMap;

use async_trait::async_trait;
use this_descriptor::{
    CheckProfile, CustomRegistry, DescriptorIssue, MessageBundle, ModuleDescriptor,
};

/// Ce qu'un store garde d'un module : le descripteur tel que déposé, non encore vérifié,
/// et ses textes. Le store ne sait pas ce qu'est un descripteur valide : c'est la
/// construction qui décide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stored {
    pub descriptor_json: String,
    pub bundles: Vec<MessageBundle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// Aucun module de ce nom (à cette version, si elle est demandée).
    NotFound {
        name: String,
        version: Option<String>,
    },
    /// Le dépôt ne se lit même pas comme un descripteur (nom ou version manquants).
    Unreadable(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::NotFound {
                name,
                version: Some(v),
            } => write!(f, "module {name} {v} absent"),
            StoreError::NotFound {
                name,
                version: None,
            } => write!(f, "module {name} absent"),
            StoreError::Unreadable(r) => write!(f, "dépôt illisible : {r}"),
        }
    }
}

/// Le contrat de stockage. `?Send` : le navigateur est mono-thread, et un store
/// IndexedDB ou Obrain n'a pas à être `Send`.
#[async_trait(?Send)]
pub trait ModuleStore {
    /// Dépose un module. Renvoie `(nom, version)` tels que lus dans le descripteur.
    async fn put(
        &mut self,
        descriptor_json: &str,
        bundles: Vec<MessageBundle>,
    ) -> Result<(String, String), StoreError>;

    /// Lit un module. `None` = la version la plus haute.
    async fn get(&self, name: &str, version: Option<&str>) -> Result<Stored, StoreError>;

    /// Versions présentes pour un nom, de la plus basse à la plus haute.
    async fn versions(&self, name: &str) -> Vec<String>;

    /// Retire une version. Absente = `NotFound`.
    async fn remove(&mut self, name: &str, version: &str) -> Result<(), StoreError>;
}

/// Un module construit : vérifié, prêt à être rendu.
#[derive(Debug, Clone)]
pub struct Built {
    pub descriptor: ModuleDescriptor,
    pub bundles: Vec<MessageBundle>,
}

#[derive(Debug)]
pub enum BuildError {
    Store(StoreError),
    /// Le JSON ne se parse pas comme descripteur.
    Parse(String),
    /// Le descripteur parse mais échoue aux vérifications du profil `Stored`.
    Invalid(Vec<DescriptorIssue>),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::Store(e) => write!(f, "{e}"),
            BuildError::Parse(r) => write!(f, "descripteur invalide : {r}"),
            BuildError::Invalid(issues) => {
                let first = issues
                    .first()
                    .map(|i| format!("{} ({})", i.message, i.path));
                write!(
                    f,
                    "{} problème(s) : {}",
                    issues.len(),
                    first.unwrap_or_default()
                )
            }
        }
    }
}

/// Construit un module : lit, parse, vérifie. Rien n'est écrit.
pub async fn build<S: ModuleStore + ?Sized>(
    store: &S,
    name: &str,
    version: Option<&str>,
    allow_public: bool,
) -> Result<Built, BuildError> {
    let stored = store.get(name, version).await.map_err(BuildError::Store)?;
    let descriptor = ModuleDescriptor::from_json(&stored.descriptor_json)
        .map_err(|e| BuildError::Parse(format!("{e:?}")))?;
    descriptor
        .check(
            &CustomRegistry::default(),
            CheckProfile::Stored { allow_public },
        )
        .map_err(BuildError::Invalid)?;
    Ok(Built {
        descriptor,
        bundles: stored.bundles,
    })
}

/// Store en mémoire. C'est tout ce que la démo utilise tant qu'Obrain n'est pas branché.
#[derive(Debug, Default)]
pub struct InMemoryStore {
    /// (nom, version) → module. `BTreeMap` : `versions()` sort sans effort.
    items: BTreeMap<(String, String), Stored>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait(?Send)]
impl ModuleStore for InMemoryStore {
    async fn put(
        &mut self,
        descriptor_json: &str,
        bundles: Vec<MessageBundle>,
    ) -> Result<(String, String), StoreError> {
        // On lit seulement le nom et la version : la validité complète est vérifiée
        // à la construction, pas au dépôt.
        let head: serde_json::Value = serde_json::from_str(descriptor_json)
            .map_err(|e| StoreError::Unreadable(e.to_string()))?;
        let name = head
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let version = head
            .get("version")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let (Some(name), Some(version)) = (name, version) else {
            return Err(StoreError::Unreadable("name ou version absent".into()));
        };
        self.items.insert(
            (name.clone(), version.clone()),
            Stored {
                descriptor_json: descriptor_json.to_string(),
                bundles,
            },
        );
        Ok((name, version))
    }

    async fn get(&self, name: &str, version: Option<&str>) -> Result<Stored, StoreError> {
        let not_found = || StoreError::NotFound {
            name: name.to_string(),
            version: version.map(str::to_string),
        };
        match version {
            Some(v) => self
                .items
                .get(&(name.to_string(), v.to_string()))
                .cloned()
                .ok_or_else(not_found),
            None => self
                .items
                .iter()
                .filter(|((n, _), _)| n == name)
                .max_by(|((_, a), _), ((_, b), _)| a.cmp(b))
                .map(|(_, s)| s.clone())
                .ok_or_else(not_found),
        }
    }

    async fn versions(&self, name: &str) -> Vec<String> {
        self.items
            .keys()
            .filter(|(n, _)| n == name)
            .map(|(_, v)| v.clone())
            .collect()
    }

    async fn remove(&mut self, name: &str, version: &str) -> Result<(), StoreError> {
        self.items
            .remove(&(name.to_string(), version.to_string()))
            .map(|_| ())
            .ok_or_else(|| StoreError::NotFound {
                name: name.to_string(),
                version: Some(version.to_string()),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use this_descriptor::field::{FieldDesc, FieldType};
    use this_descriptor::{EntityDesc, Operation, PolicyExpr};

    /// Un module minimal et complet : une entité, politiques CRUD explicites.
    fn shop(version: &str) -> String {
        let mut m = ModuleDescriptor::new("shop", version);
        let mut e = EntityDesc::new("product");
        e.fields = vec![FieldDesc::new("name", FieldType::String)];
        for op in Operation::CRUD {
            e.permissions.ops.insert(op, PolicyExpr::Authenticated);
        }
        m.entities.push(e);
        serde_json::to_string(&m).expect("serialize")
    }

    #[tokio::test]
    async fn put_then_build_round_trips_a_valid_module() {
        let mut store = InMemoryStore::new();
        let (name, version) = store.put(&shop("0.3.0"), vec![]).await.unwrap();
        assert_eq!((name.as_str(), version.as_str()), ("shop", "0.3.0"));

        let built = build(&store, "shop", None, false).await.expect("build");
        assert_eq!(built.descriptor.version, "0.3.0");
        assert_eq!(built.descriptor.entities.len(), 1);
    }

    #[tokio::test]
    async fn no_version_serves_the_highest_and_a_version_serves_itself() {
        let mut store = InMemoryStore::new();
        store.put(&shop("0.2.0"), vec![]).await.unwrap();
        store.put(&shop("0.3.0"), vec![]).await.unwrap();

        assert_eq!(
            build(&store, "shop", None, false)
                .await
                .unwrap()
                .descriptor
                .version,
            "0.3.0"
        );
        assert_eq!(
            build(&store, "shop", Some("0.2.0"), false)
                .await
                .unwrap()
                .descriptor
                .version,
            "0.2.0"
        );
        assert_eq!(store.versions("shop").await, vec!["0.2.0", "0.3.0"]);
    }

    #[tokio::test]
    async fn a_missing_module_is_not_found_not_a_panic() {
        let store = InMemoryStore::new();
        match build(&store, "shop", None, false).await {
            Err(BuildError::Store(StoreError::NotFound { name, .. })) => assert_eq!(name, "shop"),
            other => panic!("attendu NotFound, reçu {other:?}"),
        }
    }

    #[tokio::test]
    async fn a_stored_module_without_explicit_policies_is_refused_at_build() {
        // Le profil `Stored` exige une politique explicite sur chaque opération CRUD.
        let mut m = ModuleDescriptor::new("shop", "0.3.0");
        let mut e = EntityDesc::new("product");
        e.fields = vec![FieldDesc::new("name", FieldType::String)];
        m.entities.push(e);
        let json = serde_json::to_string(&m).unwrap();

        let mut store = InMemoryStore::new();
        store.put(&json, vec![]).await.unwrap();
        match build(&store, "shop", None, false).await {
            Err(BuildError::Invalid(issues)) => assert!(!issues.is_empty()),
            other => panic!("attendu Invalid, reçu {other:?}"),
        }
    }

    #[tokio::test]
    async fn an_unreadable_deposit_is_refused_at_put() {
        let mut store = InMemoryStore::new();
        assert!(matches!(
            store.put("{ pas du json", vec![]).await,
            Err(StoreError::Unreadable(_))
        ));
        assert!(matches!(
            store.put(r#"{"name":"shop"}"#, vec![]).await,
            Err(StoreError::Unreadable(_))
        ));
    }

    #[tokio::test]
    async fn remove_takes_one_version_and_the_next_highest_comes_back() {
        let mut store = InMemoryStore::new();
        store.put(&shop("0.2.0"), vec![]).await.unwrap();
        store.put(&shop("0.3.0"), vec![]).await.unwrap();
        store.remove("shop", "0.3.0").await.unwrap();
        assert_eq!(
            build(&store, "shop", None, false)
                .await
                .unwrap()
                .descriptor
                .version,
            "0.2.0"
        );
        assert!(store.remove("shop", "0.3.0").await.is_err());
    }
}
