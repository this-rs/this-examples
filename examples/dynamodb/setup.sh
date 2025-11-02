#!/bin/bash

# Setup script for DynamoDB example
set -e

echo "🚀 Setting up DynamoDB Example..."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker and try again."
    exit 1
fi

# Check if .env file exists, if not copy from .env.example
if [ ! -f .env ]; then
    echo "📋 Creating .env file from .env.example..."
    cp .env.example .env
    echo "✅ .env file created. You can modify it if needed."
else
    echo "📋 Using existing .env file..."
fi

# Start DynamoDB Local
echo "🗃️ Starting DynamoDB Local..."
docker-compose up -d

# Wait for DynamoDB to be ready
echo "⏳ Waiting for DynamoDB Local to be ready..."
sleep 5

# Check if DynamoDB is responding
if curl -s http://localhost:8000 > /dev/null; then
    echo "✅ DynamoDB Local is running on http://localhost:8000"
else
    echo "❌ DynamoDB Local is not responding. Check the logs with: docker-compose logs"
    exit 1
fi

# Check if DynamoDB Admin is responding
if curl -s http://localhost:8001 > /dev/null; then
    echo "✅ DynamoDB Admin UI is running on http://localhost:8001"
else
    echo "⚠️ DynamoDB Admin UI might not be ready yet, but DynamoDB Local is running"
fi

# Source environment variables
echo "🔧 Loading environment variables..."
set -a
source .env
set +a

# Create tables (optional - the app will create them if they don't exist)
echo "📊 Creating DynamoDB tables..."

create_table() {
    local table_name=$1
    echo "  Creating table: $table_name"
    
    if aws dynamodb describe-table --endpoint-url $AWS_ENDPOINT_URL --table-name $table_name > /dev/null 2>&1; then
        echo "    ✅ Table $table_name already exists"
    else
        aws dynamodb create-table \
            --endpoint-url $AWS_ENDPOINT_URL \
            --table-name $table_name \
            --attribute-definitions AttributeName=id,AttributeType=S \
            --key-schema AttributeName=id,KeyType=HASH \
            --provisioned-throughput ReadCapacityUnits=5,WriteCapacityUnits=5 \
            > /dev/null 2>&1
        echo "    ✅ Table $table_name created"
    fi
}

# Check if AWS CLI is available
if command -v aws > /dev/null; then
    create_table ${ORDERS_TABLE_NAME:-orders}
    create_table ${INVOICES_TABLE_NAME:-invoices}
    create_table ${PAYMENTS_TABLE_NAME:-payments}
    create_table ${LINKS_TABLE_NAME:-links}
else
    echo "⚠️ AWS CLI not found. Tables will be created automatically when the app starts."
fi

echo ""
echo "🎉 Setup complete!"
echo ""
echo "Next steps:"
echo "  1. Run the application: cargo run --features dynamodb"
echo "  2. Access DynamoDB Admin UI: http://localhost:8001"
echo "  3. Test the API endpoints (see README.md for details)"
echo ""
echo "To stop DynamoDB Local: docker-compose down"