#!/bin/bash

# Script pour lancer les stress tests DynamoDB complets

set -e

echo "🔥 === STRESS TEST DYNAMODB COMPLET ==="
echo ""

# Variables
SETUP_SCRIPT="./setup_dynamodb.sh"
CARGO_BENCH="cargo bench -p endpoint-benchmarks"

# Function pour vérifier les prérequis
check_requirements() {
    echo "🔍 Vérification des prérequis..."
    
    if ! command -v docker &> /dev/null; then
        echo "❌ Docker n'est pas installé. DynamoDB Local nécessite Docker"
        exit 1
    fi
    
    if ! command -v aws &> /dev/null; then
        echo "❌ AWS CLI n'est pas installé. Nécessaire pour créer les tables"
        exit 1
    fi
    
    echo "✅ Prérequis OK (Docker + AWS CLI)"
}

# Function pour démarrer DynamoDB
setup_dynamodb() {
    echo ""
    echo "🗄️ Configuration de DynamoDB Local..."
    
    if [ -f "$SETUP_SCRIPT" ]; then
        chmod +x "$SETUP_SCRIPT"
        $SETUP_SCRIPT start
    else
        echo "❌ Script setup_dynamodb.sh non trouvé"
        exit 1
    fi
}

# Function pour les stress tests progressifs
run_scaling_tests() {
    echo ""
    echo "📈 === TESTS DE MONTÉE EN CHARGE ==="
    echo "Test progressif: 1, 5, 10, 25, 50, 100 requêtes concurrentes"
    echo ""
    
    $CARGO_BENCH --bench dynamodb_stress_test dynamodb_stress_scaling
}

# Function pour les tests par opération
run_operations_tests() {
    echo ""
    echo "⚙️ === TESTS PAR OPÉRATION ==="
    echo "Test de toutes les opérations REST avec 20 requêtes concurrentes"
    echo ""
    
    $CARGO_BENCH --bench dynamodb_stress_test dynamodb_stress_operations
}

# Function pour les tests GraphQL
run_graphql_tests() {
    echo ""
    echo "🌐 === TESTS GRAPHQL ==="
    echo "Test des requêtes et mutations GraphQL avec 15 requêtes concurrentes"
    echo ""
    
    $CARGO_BENCH --bench dynamodb_stress_test dynamodb_stress_graphql
}

# Function pour le test d'endurance
run_endurance_test() {
    echo ""
    echo "🏃‍♂️ === TEST D'ENDURANCE ==="
    echo "Test de charge soutenue sur 1 minute"
    echo ""
    
    $CARGO_BENCH --bench dynamodb_stress_test dynamodb_endurance
}

# Function pour les tests de comparaison
run_comparison_tests() {
    echo ""
    echo "⚖️ === COMPARAISON IN-MEMORY vs DYNAMODB ==="
    echo "Test de comparaison directe des performances"
    echo ""
    
    $CARGO_BENCH --bench dynamodb_benchmarks storage_comparison
}

# Function pour nettoyer
cleanup() {
    echo ""
    echo "🧹 Nettoyage..."
    if [ -f "$SETUP_SCRIPT" ]; then
        $SETUP_SCRIPT stop
    fi
}

# Function pour générer un rapport
generate_report() {
    echo ""
    echo "📊 === RAPPORT DES TESTS ==="
    echo ""
    echo "Les résultats détaillés sont disponibles dans:"
    echo "  - target/criterion/ (rapports HTML)"
    echo "  - Console output ci-dessus"
    echo ""
    echo "🎯 Points clés à analyser:"
    echo "  1. Latence P95/P99 vs charge concurrente"
    echo "  2. Throughput maximal avant dégradation"
    echo "  3. Taux d'erreur/timeout sous forte charge"
    echo "  4. Différence de performance REST vs GraphQL"
    echo "  5. Comparaison In-Memory vs DynamoDB"
    echo ""
    echo "📈 Pour visualiser les résultats:"
    echo "  open target/criterion/report/index.html"
}

# Function pour afficher l'aide
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --scaling          Tests de montée en charge uniquement"
    echo "  --operations       Tests par opération uniquement"
    echo "  --graphql          Tests GraphQL uniquement"
    echo "  --endurance        Test d'endurance uniquement"
    echo "  --comparison       Comparaison In-Memory vs DynamoDB"
    echo "  --quick            Tests rapides (scaling + comparison)"
    echo "  --all              Tous les tests (défaut)"
    echo "  --help             Affiche cette aide"
    echo ""
    echo "Exemples:"
    echo "  $0                 # Lance tous les tests"
    echo "  $0 --quick         # Tests essentiels rapides"
    echo "  $0 --scaling       # Seulement les tests de charge"
}

# Function principale
main() {
    local test_type="all"
    
    # Parse arguments
    case "${1:-}" in
        "--scaling")
            test_type="scaling"
            ;;
        "--operations") 
            test_type="operations"
            ;;
        "--graphql")
            test_type="graphql"
            ;;
        "--endurance")
            test_type="endurance"
            ;;
        "--comparison")
            test_type="comparison"
            ;;
        "--quick")
            test_type="quick"
            ;;
        "--all"|"")
            test_type="all"
            ;;
        "--help"|"-h")
            show_help
            exit 0
            ;;
        *)
            echo "❌ Option inconnue: $1"
            show_help
            exit 1
            ;;
    esac
    
    echo "🚀 Lancement des stress tests DynamoDB (mode: $test_type)"
    echo ""
    
    # Setup trap pour le nettoyage
    trap cleanup EXIT
    
    # Vérifications et setup
    check_requirements
    setup_dynamodb
    
    # Exécution des tests selon le mode
    case "$test_type" in
        "scaling")
            run_scaling_tests
            ;;
        "operations")
            run_operations_tests
            ;;
        "graphql")
            run_graphql_tests
            ;;
        "endurance")
            run_endurance_test
            ;;
        "comparison")
            run_comparison_tests
            ;;
        "quick")
            run_scaling_tests
            run_comparison_tests
            ;;
        "all")
            run_scaling_tests
            run_operations_tests
            run_graphql_tests
            run_comparison_tests
            run_endurance_test
            ;;
    esac
    
    generate_report
    
    echo ""
    echo "✅ Stress tests terminés avec succès!"
    echo ""
}

# Point d'entrée
main "$@"