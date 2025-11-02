# This-RS Examples Makefile
# 
# This Makefile provides convenient commands to run all examples and manage DynamoDB Local

.PHONY: help rest graphql dynamodb dynamodb-setup dynamodb-teardown test lint clean

# Default target
help:
	@echo "This-RS Examples - Available Commands:"
	@echo ""
	@echo "🚀 Run Examples:"
	@echo "  make rest        - Run REST-only example server"
	@echo "  make graphql     - Run GraphQL + REST example server"
	@echo "  make dynamodb    - Run DynamoDB example (with local DynamoDB setup)"
	@echo ""
	@echo "🗃️  DynamoDB Management:"
	@echo "  make dynamodb-setup     - Start DynamoDB Local + setup tables"
	@echo "  make dynamodb-teardown  - Stop DynamoDB Local and cleanup"
	@echo "  make dynamodb-logs      - Show DynamoDB Local logs"
	@echo "  make dynamodb-admin     - Open DynamoDB Admin UI"
	@echo ""
	@echo "🧪 Development:"
	@echo "  make test        - Run all tests"
	@echo "  make lint        - Run clippy with strict warnings"
	@echo "  make build       - Build all examples"
	@echo "  make clean       - Clean build artifacts"
	@echo ""
	@echo "📊 Status:"
	@echo "  make status      - Show running services status"

# Build all examples
build:
	@echo "🔨 Building all examples..."
	cargo build --all-targets
	cargo build --all-targets --features graphql
	cargo build --all-targets --features dynamodb
	cargo build --all-targets --all-features

# Run REST-only example
rest:
	@echo "🌐 Starting REST-only example..."
	@echo "Server will be available at: http://localhost:4242"
	@echo "Endpoints: GET/POST /orders, /invoices, /payments"
	@echo "Press Ctrl+C to stop"
	@echo ""
	cargo run -p rest_example

# Run GraphQL + REST example  
graphql:
	@echo "🌐 Starting GraphQL + REST example..."
	@echo "REST API: http://localhost:4242"
	@echo "GraphQL: http://localhost:4242/graphql"
	@echo "GraphQL Playground: http://localhost:4242/graphql/playground"
	@echo "Press Ctrl+C to stop"
	@echo ""
	cargo run -p graphql_example --features graphql

# DynamoDB example with full setup
dynamodb: dynamodb-setup
	@echo "⏳ Waiting for DynamoDB Local to be ready..."
	@sleep 3
	@echo ""
	@echo "🌐 Starting DynamoDB example..."
	@echo "REST API: http://localhost:4242"
	@echo "GraphQL: http://localhost:4242/graphql"
	@echo "GraphQL Playground: http://localhost:4242/graphql/playground"
	@echo "DynamoDB Admin: http://localhost:8001"
	@echo ""
	@echo "Press Ctrl+C to stop (DynamoDB Local will keep running)"
	@echo "Use 'make dynamodb-teardown' to stop DynamoDB Local"
	@echo ""
	cd examples/dynamodb && \
	export AWS_ENDPOINT_URL=http://localhost:8000 && \
	export AWS_ACCESS_KEY_ID=dummy && \
	export AWS_SECRET_ACCESS_KEY=dummy && \
	export AWS_DEFAULT_REGION=us-east-1 && \
	cargo run --features dynamodb,graphql

# Setup DynamoDB Local and create tables
dynamodb-setup:
	@echo "🗃️  Setting up DynamoDB Local..."
	@cd examples/dynamodb && \
	if ! docker-compose ps | grep -q dynamodb-local; then \
		echo "Starting DynamoDB Local..."; \
		docker-compose up -d; \
		echo "⏳ Waiting for DynamoDB to start..."; \
		sleep 5; \
	else \
		echo "✅ DynamoDB Local already running"; \
	fi
	@echo "📊 Creating tables if they don't exist..."
	@cd examples/dynamodb && \
	export AWS_ENDPOINT_URL=http://localhost:8000 && \
	export AWS_ACCESS_KEY_ID=dummy && \
	export AWS_SECRET_ACCESS_KEY=dummy && \
	export AWS_DEFAULT_REGION=us-east-1 && \
	./setup.sh || echo "⚠️  Setup script encountered issues (tables may already exist)"
	@echo "✅ DynamoDB setup complete!"
	@echo "   - DynamoDB Local: http://localhost:8000"
	@echo "   - Admin UI: http://localhost:8001"

# Stop DynamoDB Local
dynamodb-teardown:
	@echo "🗃️  Stopping DynamoDB Local..."
	@cd examples/dynamodb && docker-compose down
	@echo "✅ DynamoDB Local stopped"

# Show DynamoDB Local logs
dynamodb-logs:
	@echo "📋 DynamoDB Local logs:"
	@cd examples/dynamodb && docker-compose logs -f dynamodb-local

# Open DynamoDB Admin UI
dynamodb-admin:
	@echo "🌐 Opening DynamoDB Admin UI..."
	@open http://localhost:8001 || xdg-open http://localhost:8001 || echo "Please open http://localhost:8001 in your browser"

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test --all

# Run clippy with strict warnings
lint:
	@echo "🔍 Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Show status of services
status:
	@echo "📊 Services Status:"
	@echo ""
	@echo "DynamoDB Local:"
	@cd examples/dynamodb && \
	if docker-compose ps | grep -q dynamodb-local; then \
		echo "  ✅ Running - http://localhost:8000"; \
		echo "  ✅ Admin UI - http://localhost:8001"; \
	else \
		echo "  ❌ Stopped"; \
	fi
	@echo ""
	@echo "Ports in use:"
	@lsof -i :4242 || echo "  Port 4242: Available"
	@lsof -i :8000 || echo "  Port 8000: Available" 
	@lsof -i :8001 || echo "  Port 8001: Available"

# Development shortcuts
dev-rest: clean build rest
dev-graphql: clean build graphql  
dev-dynamodb: clean build dynamodb

# Quick test commands
test-rest:
	@echo "🧪 Testing REST API..."
	@echo "Starting REST server in background..."
	@cargo run -p rest_example > /dev/null 2>&1 & \
	echo $$! > .rest_pid && \
	sleep 3 && \
	echo "Testing endpoints..." && \
	curl -s http://localhost:4242/orders && \
	echo "" && \
	kill `cat .rest_pid` && \
	rm .rest_pid && \
	echo "✅ REST API test completed"

test-graphql:
	@echo "🧪 Testing GraphQL API..."
	@echo "Starting GraphQL server in background..."
	@cargo run -p graphql_example --features graphql > /dev/null 2>&1 & \
	echo $$! > .graphql_pid && \
	sleep 3 && \
	echo "Testing GraphQL endpoint..." && \
	curl -s -X POST http://localhost:4242/graphql \
		-H "Content-Type: application/json" \
		-d '{"query": "{ orders { id name } }"}' && \
	echo "" && \
	kill `cat .graphql_pid` && \
	rm .graphql_pid && \
	echo "✅ GraphQL API test completed"

# CI/CD targets
ci: lint test build
	@echo "✅ All CI checks passed!"

# Documentation 
docs:
	@echo "📚 Available documentation:"
	@echo "  - Main README: README.md"
	@echo "  - DynamoDB Setup: examples/dynamodb/README.md" 
	@echo "  - Full Documentation: docs/"
	@echo ""
	@echo "🌐 Online resources:"
	@echo "  - GraphQL Playground: http://localhost:4242/graphql/playground (when running)"
	@echo "  - DynamoDB Admin: http://localhost:8001 (when DynamoDB Local is running)"