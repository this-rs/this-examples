#!/bin/bash

# Script pour configurer DynamoDB Local pour les benchmarks

set -e

echo "🗄️ Configuration de DynamoDB Local pour les benchmarks..."

# Variables
DYNAMODB_LOCAL_DIR="dynamodb_local"
DYNAMODB_JAR="DynamoDBLocal.jar"
DYNAMODB_PORT=8000
AWS_ENDPOINT_URL="http://localhost:${DYNAMODB_PORT}"

# Function pour vérifier si DynamoDB est en cours d'exécution
check_dynamodb_running() {
    if curl -s "$AWS_ENDPOINT_URL" > /dev/null 2>&1; then
        echo "✅ DynamoDB Local est déjà en cours d'exécution sur le port $DYNAMODB_PORT"
        return 0
    else
        return 1
    fi
}

# Function pour démarrer DynamoDB Local avec Docker
start_dynamodb() {
    if ! check_dynamodb_running; then
        echo "🐳 Démarrage de DynamoDB Local avec Docker..."
        
        # Vérifier que Docker est disponible
        if ! command -v docker &> /dev/null; then
            echo "❌ Docker n'est pas installé ou disponible"
            exit 1
        fi
        
        # Arrêter le container existant s'il existe
        docker stop dynamodb-local-bench 2>/dev/null || true
        docker rm dynamodb-local-bench 2>/dev/null || true
        
        # Démarrer DynamoDB Local avec Docker
        docker run -d \
            --name dynamodb-local-bench \
            -p ${DYNAMODB_PORT}:8000 \
            amazon/dynamodb-local:latest \
            -jar DynamoDBLocal.jar -sharedDb -inMemory
        
        # Attendre que DynamoDB soit prêt
        echo "⏳ Attente du démarrage de DynamoDB Local..."
        for i in {1..30}; do
            if check_dynamodb_running; then
                echo "✅ DynamoDB Local démarré avec Docker"
                echo "dynamodb-local-bench" > .dynamodb_container
                break
            fi
            sleep 1
        done
        
        if ! check_dynamodb_running; then
            echo "❌ Erreur: DynamoDB Local n'a pas pu démarrer"
            docker logs dynamodb-local-bench
            exit 1
        fi
    fi
}

# Function pour créer les tables de test
create_tables() {
    echo "🏗️ Création des tables de benchmark..."
    
    export AWS_ACCESS_KEY_ID=dummy
    export AWS_SECRET_ACCESS_KEY=dummy
    export AWS_DEFAULT_REGION=us-east-1
    export AWS_ENDPOINT_URL="$AWS_ENDPOINT_URL"
    
    # Table orders
    aws dynamodb create-table \
        --table-name bench_orders \
        --attribute-definitions AttributeName=id,AttributeType=S \
        --key-schema AttributeName=id,KeyType=HASH \
        --billing-mode PAY_PER_REQUEST \
        --endpoint-url "$AWS_ENDPOINT_URL" > /dev/null 2>&1 || echo "Table bench_orders existe déjà"
    
    # Table invoices
    aws dynamodb create-table \
        --table-name bench_invoices \
        --attribute-definitions AttributeName=id,AttributeType=S \
        --key-schema AttributeName=id,KeyType=HASH \
        --billing-mode PAY_PER_REQUEST \
        --endpoint-url "$AWS_ENDPOINT_URL" > /dev/null 2>&1 || echo "Table bench_invoices existe déjà"
    
    # Table payments
    aws dynamodb create-table \
        --table-name bench_payments \
        --attribute-definitions AttributeName=id,AttributeType=S \
        --key-schema AttributeName=id,KeyType=HASH \
        --billing-mode PAY_PER_REQUEST \
        --endpoint-url "$AWS_ENDPOINT_URL" > /dev/null 2>&1 || echo "Table bench_payments existe déjà"
    
    # Table links
    aws dynamodb create-table \
        --table-name bench_links \
        --attribute-definitions AttributeName=id,AttributeType=S \
        --key-schema AttributeName=id,KeyType=HASH \
        --billing-mode PAY_PER_REQUEST \
        --endpoint-url "$AWS_ENDPOINT_URL" > /dev/null 2>&1 || echo "Table bench_links existe déjà"
    
    echo "✅ Tables créées"
}

# Function pour arrêter DynamoDB Local
stop_dynamodb() {
    if [ -f .dynamodb_container ]; then
        CONTAINER_NAME=$(cat .dynamodb_container)
        echo "🛑 Arrêt du container DynamoDB Local ($CONTAINER_NAME)..."
        docker stop $CONTAINER_NAME 2>/dev/null || echo "Container déjà arrêté"
        docker rm $CONTAINER_NAME 2>/dev/null || echo "Container déjà supprimé"
        rm -f .dynamodb_container
        echo "✅ DynamoDB Local arrêté"
    else
        echo "ℹ️ DynamoDB Local n'était pas en cours d'exécution"
        # Tenter de nettoyer un éventuel container orphelin
        docker stop dynamodb-local-bench 2>/dev/null || true
        docker rm dynamodb-local-bench 2>/dev/null || true
    fi
}

# Main logic
case "$1" in
    "start")
        start_dynamodb
        create_tables
        echo ""
        echo "🎉 DynamoDB Local est prêt pour les benchmarks!"
        echo "   Endpoint: $AWS_ENDPOINT_URL"
        echo "   Pour arrêter: ./setup_dynamodb.sh stop"
        echo ""
        echo "🚀 Vous pouvez maintenant lancer les benchmarks:"
        echo "   cargo bench -p endpoint-benchmarks --bench dynamodb_benchmarks"
        ;;
    "stop")
        stop_dynamodb
        ;;
    "restart")
        stop_dynamodb
        sleep 2
        start_dynamodb
        create_tables
        echo "🎉 DynamoDB Local redémarré!"
        ;;
    "status")
        if check_dynamodb_running; then
            echo "✅ DynamoDB Local est en cours d'exécution"
            if [ -f .dynamodb_container ]; then
                CONTAINER_NAME=$(cat .dynamodb_container)
                echo "   Container: $CONTAINER_NAME"
                docker ps --filter name=$CONTAINER_NAME --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
            fi
        else
            echo "❌ DynamoDB Local n'est pas en cours d'exécution"
        fi
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status}"
        echo ""
        echo "Commandes:"
        echo "  start    - Démarre DynamoDB Local et crée les tables"
        echo "  stop     - Arrête DynamoDB Local" 
        echo "  restart  - Redémarre DynamoDB Local"
        echo "  status   - Vérifie l'état de DynamoDB Local"
        exit 1
        ;;
esac