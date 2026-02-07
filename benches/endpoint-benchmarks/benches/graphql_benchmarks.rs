use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use endpoint_benchmarks::{create_graphql_server_in_memory, data, start_test_server, TestServer};
use serde_json::json;
use tokio::runtime::Runtime;

/// Benchmark GraphQL queries
async fn bench_graphql_queries(server: &TestServer) -> anyhow::Result<()> {
    let queries = vec![
        json!({"query": "query { orders { id name number amount status } }"}),
        json!({"query": "query { invoices { id name number amount status } }"}),
        json!({"query": "query { payments { id name number amount status } }"}),
    ];

    for query in queries {
        let response = server.post_json("/graphql", black_box(query)).await?;
        black_box(response);
    }

    Ok(())
}

/// Benchmark GraphQL mutations
async fn bench_graphql_mutations(server: &TestServer) -> anyhow::Result<()> {
    let mutations = vec![
        data::sample_graphql_mutation(),
        // Add more mutation examples here
    ];

    for mutation in mutations {
        let response = server.post_json("/graphql", black_box(mutation)).await?;
        black_box(response);
    }

    Ok(())
}

/// Benchmark individual GraphQL query
async fn bench_single_graphql_query(
    server: &TestServer,
    query: serde_json::Value,
) -> anyhow::Result<()> {
    let response = server.post_json("/graphql", query).await?;
    let body = TestServer::response_body_string(response).await?;
    black_box(body);
    Ok(())
}

fn benchmark_graphql_in_memory(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Create server once for all benchmarks
    let server = rt.block_on(async {
        let router = create_graphql_server_in_memory().await.unwrap();
        start_test_server(router).await.unwrap()
    });

    let mut group = c.benchmark_group("graphql_in_memory");

    // Benchmark individual queries
    let query_test_cases = vec![
        (
            "orders_query",
            json!({"query": "query { orders { id name number amount status customer_name notes } }"}),
        ),
        (
            "invoices_query",
            json!({"query": "query { invoices { id name number amount status due_date paid_at } }"}),
        ),
        (
            "payments_query",
            json!({"query": "query { payments { id name number amount status method transaction_id } }"}),
        ),
        ("health_query", json!({"query": "query { health }"})),
    ];

    for (name, query) in query_test_cases {
        group.bench_with_input(
            BenchmarkId::new("query", name),
            &(&server, query),
            |b, (server, query)| {
                b.to_async(&rt).iter(|| async {
                    bench_single_graphql_query(server, query.clone())
                        .await
                        .unwrap()
                });
            },
        );
    }

    // Benchmark mutations
    let mutation_test_cases = vec![
        (
            "create_order",
            json!({
                "query": "mutation CreateOrder($input: OrderInput!) { createOrder(input: $input) { id name number amount status } }",
                "variables": {
                    "input": {
                        "name": "Benchmark Order",
                        "status": "pending",
                        "number": "ORD-BENCH-001",
                        "amount": 999.99,
                        "customer_name": "Test Customer",
                        "notes": "Benchmark test order"
                    }
                }
            }),
        ),
        (
            "create_invoice",
            json!({
                "query": "mutation CreateInvoice($input: InvoiceInput!) { createInvoice(input: $input) { id name number amount status } }",
                "variables": {
                    "input": {
                        "name": "Benchmark Invoice",
                        "status": "draft",
                        "number": "INV-BENCH-001",
                        "amount": 1999.50,
                        "due_date": "2024-12-31"
                    }
                }
            }),
        ),
        (
            "create_payment",
            json!({
                "query": "mutation CreatePayment($input: PaymentInput!) { createPayment(input: $input) { id name number amount status } }",
                "variables": {
                    "input": {
                        "name": "Benchmark Payment",
                        "status": "pending",
                        "number": "PAY-BENCH-001",
                        "amount": 999.99,
                        "method": "credit_card",
                        "transaction_id": "txn_bench_001"
                    }
                }
            }),
        ),
    ];

    for (name, mutation) in mutation_test_cases {
        group.bench_with_input(
            BenchmarkId::new("mutation", name),
            &(&server, mutation),
            |b, (server, mutation)| {
                b.to_async(&rt).iter(|| async {
                    bench_single_graphql_query(server, mutation.clone())
                        .await
                        .unwrap()
                });
            },
        );
    }

    // Benchmark combined operations
    group.bench_function("all_queries", |b| {
        b.to_async(&rt)
            .iter(|| async { bench_graphql_queries(black_box(&server)).await.unwrap() });
    });

    group.bench_function("all_mutations", |b| {
        b.to_async(&rt)
            .iter(|| async { bench_graphql_mutations(black_box(&server)).await.unwrap() });
    });

    group.finish();
}

fn benchmark_graphql_complex_queries(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let server = rt.block_on(async {
        let router = create_graphql_server_in_memory().await.unwrap();
        start_test_server(router).await.unwrap()
    });

    let mut group = c.benchmark_group("graphql_complex_queries");

    // Complex nested queries
    let complex_queries = vec![
        (
            "all_entities_detailed",
            json!({
                "query": r#"
                query AllEntitiesDetailed {
                    orders {
                        id
                        name
                        number
                        amount
                        status
                        customer_name
                        notes
                        created_at
                        updated_at
                    }
                    invoices {
                        id
                        name
                        number
                        amount
                        status
                        due_date
                        paid_at
                        created_at
                        updated_at
                    }
                    payments {
                        id
                        name
                        number
                        amount
                        status
                        method
                        transaction_id
                        created_at
                        updated_at
                    }
                }
            "#
            }),
        ),
        (
            "filtered_query",
            json!({
                "query": r#"
                query FilteredEntities {
                    orders {
                        id
                        name
                        number
                        amount
                        status
                    }
                    invoices {
                        id
                        name
                        number
                        amount
                        status
                    }
                }
            "#
            }),
        ),
    ];

    for (name, query) in complex_queries {
        group.bench_with_input(
            BenchmarkId::new("complex", name),
            &(&server, query),
            |b, (server, query)| {
                b.to_async(&rt).iter(|| async {
                    bench_single_graphql_query(server, query.clone())
                        .await
                        .unwrap()
                });
            },
        );
    }

    group.finish();
}

fn benchmark_graphql_load_test(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let server = rt.block_on(async {
        let router = create_graphql_server_in_memory().await.unwrap();
        start_test_server(router).await.unwrap()
    });

    let mut group = c.benchmark_group("graphql_load_test");

    // Simulate concurrent GraphQL requests
    group.bench_function("concurrent_queries", |b| {
        b.to_async(&rt).iter(|| async {
            let queries = (0..10).map(|i| {
                let server = server.clone();
                let query = json!({
                    "query": format!("query {{ orders {{ id name number amount status }} }}")
                });
                tokio::spawn(async move { server.post_json("/graphql", query).await.unwrap() })
            });

            let results = futures::future::join_all(queries).await;
            black_box(results);
        });
    });

    group.bench_function("mixed_operations", |b| {
        b.to_async(&rt).iter(|| async {
            // Mix of queries and mutations
            let query_tasks = (0..5).map(|_| {
                let server = server.clone();
                let query = json!({"query": "query { orders { id name number } }"});
                tokio::spawn(async move {
                    server.post_json("/graphql", query).await.unwrap()
                })
            });

            let mutation_tasks = (0..3).map(|i| {
                let server = server.clone();
                let mutation = json!({
                    "query": "mutation CreateOrder($input: OrderInput!) { createOrder(input: $input) { id name } }",
                    "variables": {
                        "input": {
                            "name": format!("Concurrent Order {}", i),
                            "status": "pending",
                            "number": format!("ORD-CONC-{:03}", i),
                            "amount": 100.0 + i as f64,
                            "customer_name": "Concurrent Customer",
                            "notes": "Load test order"
                        }
                    }
                });
                tokio::spawn(async move {
                    server.post_json("/graphql", mutation).await.unwrap()
                })
            });

            let (query_results, mutation_results) = tokio::join!(
                futures::future::join_all(query_tasks),
                futures::future::join_all(mutation_tasks)
            );

            black_box((query_results, mutation_results));
        });
    });

    group.finish();
}

fn benchmark_graphql_response_parsing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let server = rt.block_on(async {
        let router = create_graphql_server_in_memory().await.unwrap();
        start_test_server(router).await.unwrap()
    });

    let mut group = c.benchmark_group("graphql_response_parsing");

    group.bench_function("parse_orders_response", |b| {
        b.to_async(&rt).iter(|| async {
            let query = json!({"query": "query { orders { id name number amount status } }"});
            let response = server.post_json("/graphql", query).await.unwrap();
            let body = TestServer::response_body_string(response).await.unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
            black_box(parsed);
        });
    });

    group.bench_function("parse_mutation_response", |b| {
        b.to_async(&rt).iter(|| async {
            let mutation = json!({
                "query": "mutation CreateInvoice($input: InvoiceInput!) { createInvoice(input: $input) { id name number amount } }",
                "variables": {
                    "input": {
                        "name": "Parse Test Invoice",
                        "status": "draft",
                        "number": "INV-PARSE-001",
                        "amount": 1500.00,
                        "due_date": "2024-12-31"
                    }
                }
            });
            let response = server.post_json("/graphql", mutation).await.unwrap();
            let body = TestServer::response_body_string(response).await.unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
            black_box(parsed);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_graphql_in_memory,
    benchmark_graphql_complex_queries,
    benchmark_graphql_load_test,
    benchmark_graphql_response_parsing
);

criterion_main!(benches);
