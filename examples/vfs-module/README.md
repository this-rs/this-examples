# vfs-module

Charger, remplacer et décharger un module déclaratif (`ModuleDescriptor`) dans un
`ModuleStore` — la preuve lisible de la « capacité à load une interface par module »,
mesurée sur le contrat du store, sans serveur.

Le cycle est identique pour un `Vfs` en mémoire et un `Vfs` local :

1. **charger** — `put("0.1.0")` puis `get(name, None)` sert `0.1.0` ;
2. **remplacer** — `put("0.2.0")` : `get(None)` sert la **plus haute** version, et
   `get(name, Some("0.1.0"))` sert encore l'ancienne ;
3. **lister** — `list()` énumère `(nom, version)` ;
4. **décharger** — `remove("0.2.0")` fait retomber `get(None)` sur `0.1.0` ;
   `remove("0.1.0")` fait répondre `this.vfs.not_found` (le tombstone HTTP
   `this.module.not_found` est du ressort de `ModuleRuntime`, exemple `hot_module`).

Le rendu **HTTP** de ce cycle (une route module qui répond 404 après déchargement) est
l'exemple `hot_module` du dépôt `this`.

## Lancer

```text
cargo run -p vfs_module_example      # tableau du cycle, en mémoire puis sur disque
cargo test -p vfs_module_example     # les mêmes étapes en assertions
```

Le binaire sort en code non nul si un cycle ne suit pas le contrat : il sert de test de
fumée.
