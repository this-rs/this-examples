# Endpoint Benchmarks

Ce crate contient des benchmarks de performance pour les endpoints REST et GraphQL du système de facturation.

## 🚀 Exécution des benchmarks

### Benchmarks REST

```bash
# Tous les benchmarks REST
cargo bench -p endpoint-benchmarks --bench rest_benchmarks

# Benchmark spécifique
cargo bench -p endpoint-benchmarks --bench rest_benchmarks rest_in_memory/get/orders
cargo bench -p endpoint-benchmarks --bench rest_benchmarks rest_in_memory/post/invoices

# Tests de charge REST
cargo bench -p endpoint-benchmarks --bench rest_benchmarks rest_load_test
```

### Benchmarks GraphQL

```bash
# Tous les benchmarks GraphQL  
cargo bench -p endpoint-benchmarks --bench graphql_benchmarks

# Requêtes spécifiques
cargo bench -p endpoint-benchmarks --bench graphql_benchmarks graphql_in_memory/query/orders_query
cargo bench -p endpoint-benchmarks --bench graphql_benchmarks graphql_in_memory/mutation/create_order

# Requêtes complexes
cargo bench -p endpoint-benchmarks --bench graphql_benchmarks graphql_complex_queries
```

### Tous les benchmarks

```bash
# Exécuter tous les benchmarks d'endpoints
cargo bench -p endpoint-benchmarks
```

## 📊 Types de benchmarks

### REST API
- **Endpoints GET** : `/order`, `/invoice`, `/payment`
- **Endpoints POST** : Création d'entités
- **Tests de charge** : Requêtes concurrentes
- **Parsing des réponses** : Sérialisation/Désérialisation JSON

### GraphQL API
- **Queries simples** : Récupération d'entités individuelles
- **Mutations** : Création d'orders, invoices, payments
- **Queries complexes** : Récupération de plusieurs entités avec tous les champs
- **Tests de charge** : Queries et mutations concurrentes

## 🏗️ Architecture

Les benchmarks utilisent :

- **Serveurs HTTP réels** avec des ports dynamiques
- **Stockage en mémoire** pour des performances optimales
- **Clients HTTP asynchrones** (Hyper) pour les requêtes
- **Criterion** pour les mesures de performance précises

## 📈 Métriques mesurées

- **Latence** : Temps de réponse des endpoints
- **Throughput** : Performance sous charge concurrent
- **Parsing** : Temps de sérialisation/désérialisation
- **Memory usage** : Utilisation mémoire des serveurs

## 🔧 Configuration

Les benchmarks sont configurés pour :
- 100 iterations par test par défaut
- Rapports HTML générés dans `target/criterion/`
- Tests avec données réalistes via le module `test-data`

## 📁 Structure

```
endpoint-benchmarks/
├── src/lib.rs              # Utilitaires communs et serveurs de test
├── benches/
│   ├── rest_benchmarks.rs  # Benchmarks des endpoints REST
│   └── graphql_benchmarks.rs # Benchmarks des endpoints GraphQL
└── Cargo.toml             # Configuration des dépendances
```

## 🎯 Cas d'usage

Ces benchmarks sont utiles pour :
- Mesurer les performances des API avant/après optimisations
- Comparer REST vs GraphQL pour différents scénarios
- Identifier les goulots d'étranglement de performance
- Valider la performance sous charge
- Tests de régression de performance dans la CI/CD

## 📊 Rapports

Les résultats sont sauvegardés dans `target/criterion/` avec :
- Graphiques de performance
- Comparaisons historiques
- Détection de régressions automatique
- Exports HTML pour visualisation