# 🔥 Guide Complet des Stress Tests DynamoDB

Ce guide vous permet de découvrir les **vraies limites de performance** de votre application avec un stockage DynamoDB réaliste.

## 🎯 Objectif

Contrairement aux benchmarks avec stockage en mémoire qui donnent des résultats irréalistes (~140k req/s), les stress tests DynamoDB révèlent les performances réelles que vous pouvez attendre en production (~200-500 req/s).

## 🚀 Démarrage Rapide

### 1. Tests essentiels rapides (5-10 minutes)
```bash
cd benches/endpoint-benchmarks
./run_stress_test.sh --quick
```

### 2. Tests complets (20-30 minutes)
```bash
./run_stress_test.sh --all
```

## 📋 Types de stress tests

### 🔢 Tests de montée en charge
```bash
./run_stress_test.sh --scaling
```
Teste progressivement : **1, 5, 10, 25, 50, 100** requêtes concurrentes

**Objectif :** Trouver le point de rupture où les performances se dégradent

### ⚙️ Tests par opération
```bash
./run_stress_test.sh --operations
```
Teste toutes les opérations REST avec **20 requêtes concurrentes** :
- GET /order, /invoice, /payment
- POST /order, /invoice, /payment

**Objectif :** Comparer les performances READ vs WRITE

### 🌐 Tests GraphQL
```bash
./run_stress_test.sh --graphql
```
Teste les requêtes et mutations GraphQL avec **15 requêtes concurrentes**

**Objectif :** Mesurer l'overhead GraphQL vs REST

### 🏃‍♂️ Test d'endurance
```bash
./run_stress_test.sh --endurance
```
Teste une charge soutenue pendant **1 minute**

**Objectif :** Détecter les fuites mémoire et dégradations dans le temps

### ⚖️ Comparaison stockage
```bash
./run_stress_test.sh --comparison
```
Compare directement In-Memory vs DynamoDB

**Objectif :** Quantifier l'impact du stockage sur les performances

## 📊 Métriques clés à analyser

### 🎯 Latence (temps de réponse)
- **Min/Avg/Max** : Plage de latences
- **P50** : 50% des requêtes sont plus rapides
- **P95** : 95% des requêtes sont plus rapides (SLA typique)
- **P99** : 99% des requêtes sont plus rapides (utilisateurs exigeants)

### 🚀 Throughput (débit)
- **Requêtes/seconde** : Capacité de traitement
- **Temps total** : Temps pour traiter N requêtes concurrentes

### ✅ Fiabilité
- **Taux de succès** : % de requêtes réussies
- **Erreurs** : Requêtes échouées (500, 404, etc.)
- **Timeouts** : Requêtes trop lentes

## 📈 Résultats attendus

### In-Memory (référence irréaliste)
```
🏆 Performance fantasmagorique
├── GET:  ~33µs, ~30,000 req/s
├── POST: ~35µs, ~28,000 req/s
└── P95:  ~50µs, P99: ~70µs
```

### DynamoDB Local (réaliste)
```
🗄️ Performance réaliste
├── GET:  ~2-5ms,   ~200-500 req/s
├── POST: ~10-20ms, ~50-100 req/s  
└── P95:  ~15-30ms, P99: ~25-50ms
```

### DynamoDB Cloud (production estimée)
```
☁️ Performance production
├── GET:  ~5-15ms,   ~100-300 req/s
├── POST: ~20-50ms,  ~20-50 req/s
└── P95:  ~30-80ms,  P99: ~50-150ms
```

## 🔍 Analyse des résultats

### 🎉 Performance excellente
- **P95 < 100ms** : Excellente expérience utilisateur
- **Taux de succès > 99%** : Très fiable
- **Throughput stable** : Pas de dégradation sous charge

### 👍 Performance acceptable  
- **P95 < 500ms** : Expérience utilisateur correcte
- **Taux de succès > 95%** : Fiabilité acceptable
- **Légère dégradation** : Performance diminue avec la charge

### ⚠️ Performance dégradée
- **P95 > 500ms** : Expérience utilisateur frustrante
- **Taux de succès < 95%** : Trop d'erreurs
- **Forte dégradation** : Performance s'effondre sous charge

## 💡 Interprétation des patterns

### 📊 Courbe de montée en charge normale
```
1 req:   P95 = 5ms    ✅ Baseline
5 req:   P95 = 8ms    ✅ Augmentation linéaire
10 req:  P95 = 15ms   ✅ Encore acceptable
25 req:  P95 = 40ms   👍 Dégradation modérée
50 req:  P95 = 100ms  ⚠️ Limite atteinte
100 req: P95 = 300ms  ❌ Trop lent
```

### 🔥 Point de rupture identifié
Le **"sweet spot"** est généralement autour de **10-20 requêtes concurrentes** pour DynamoDB Local.

## 🏆 Optimisations recommandées

### Si P95 > 100ms avec peu de charge (< 10 req):
1. **Optimiser les requêtes DynamoDB** : Indexes, projections
2. **Connection pooling** : Réutiliser les connexions
3. **Réduire les aller-retours** : Batch operations

### Si dégradation rapide avec la charge:
1. **Implement caching** : Redis devant DynamoDB
2. **Read replicas** : DAX pour les lectures
3. **Partitioning** : Éviter les hot partitions

### Si trop d'erreurs/timeouts:
1. **Retry logic** : Exponential backoff
2. **Circuit breaker** : Fail fast sur surcharge
3. **Rate limiting** : Protéger le backend

## 🎯 Seuils de performance recommandés

### Pour une application web typique:
- **GET endpoints** : P95 < 50ms, throughput > 100 req/s
- **POST endpoints** : P95 < 200ms, throughput > 20 req/s
- **Taux de succès** : > 99.9%

### Pour une API haute performance:
- **GET endpoints** : P95 < 20ms, throughput > 500 req/s
- **POST endpoints** : P95 < 100ms, throughput > 50 req/s
- **Avec cache** : Peut atteindre des milliers de req/s

## 🚨 Signaux d'alarme

### ❌ Indicateurs critiques:
- P99 > 1 seconde
- Taux d'erreur > 5%
- Throughput qui chute de 50%+ sous charge
- Timeouts fréquents

### ⚠️ Indicateurs d'attention:
- P95 qui double avec 2x la charge
- Latence qui augmente dans le temps (fuite mémoire?)
- Variance élevée (latences imprévisibles)

## 📋 Checklist de validation

Avant de déployer en production, validez que :

- [ ] **P95 < 100ms** pour les endpoints critiques
- [ ] **Taux de succès > 99%** sous charge normale
- [ ] **Dégradation gracieuse** : performance diminue progressivement
- [ ] **Pas de timeouts** sous charge modérée
- [ ] **Performance stable** sur la durée (test d'endurance)

## 🎬 Exemple d'utilisation complète

```bash
# 1. Démarrer DynamoDB Local
./setup_dynamodb.sh start

# 2. Tests rapides pour validation
./run_stress_test.sh --quick

# 3. Analyse des résultats critiques
#    Si P95 > 100ms → Optimiser avant les tests complets

# 4. Tests complets si résultats satisfaisants
./run_stress_test.sh --all

# 5. Analyser les rapports HTML
open ../../target/criterion/report/index.html

# 6. Nettoyer
./setup_dynamodb.sh stop
```

Ce guide vous donne une **vision réaliste** des performances que vous pouvez attendre de votre application en production avec DynamoDB ! 🎯