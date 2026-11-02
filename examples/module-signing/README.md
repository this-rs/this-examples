# module-signing

Un module stocké n'est chargé que s'il est **signé par une clé de l'ancre de confiance**.
Sans ancre, le `ModuleStore` ne vérifie que l'intégrité (les fichiers correspondent aux
etags d'un manifeste que n'importe qui ayant l'écriture peut réécrire) ; avec une ancre,
un opérateur qui peut écrire dans le store ne peut plus inventer un module.

| Scénario | Ancre | Résultat | Code |
|---|---|---|---|
| signé, ancre qui connaît la clé | `release-2026` → clé du signataire | **accepté** | — |
| signature forgée sous une clé connue | `release-2026` → autre clé | refusé | `this.module.bad_signature` |
| clé inconnue de l'ancre | `someone-else` | refusé | `this.module.untrusted_key` |
| module non signé | ancre déclarée | refusé | `this.module.unsigned` |

API démontrée (feature `module-signing` de `this-rs`) :

- `VfsModuleStore::with_signing_key(key_id, signing_key)` — côté éditeur, signe chaque
  manifeste committé ;
- `VfsModuleStore::with_trust_anchor(TrustAnchor)` — côté hôte, exige une signature d'une
  clé connue ;
- `TrustAnchor::insert(key_id, verifying_key)` / `insert_pem` — les clés de confiance.

## Lancer

```text
cargo run -p module_signing_example      # tableau des quatre issues
cargo test -p module_signing_example     # les mêmes issues en assertions
```

Le binaire sort en code non nul si un scénario ne suit pas la règle : il sert de test
de fumée.
