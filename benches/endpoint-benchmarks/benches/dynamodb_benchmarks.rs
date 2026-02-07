use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use endpoint_benchmarks::{
    create_graphql_server_dynamodb, create_rest_server_dynamodb, data, start_test_server,
    TestServer,
};
use serde_json::json;
use tokio::runtime::Runtime;

/// Benchmark individual GET endpoint with DynamoDB
async fn bench_single_get_endpoint_dynamo(
    server: &TestServer,
    endpoint: &str,
) -> anyhow::Result<()> {
    let response = server.get(endpoint).await?;
    let body = TestServer::response_body_string(response).await?;
    black_box(body);
    Ok(())
}

/// Benchmark individual POST endpoint with DynamoDB
async fn bench_single_post_endpoint_dynamo(
    server: &TestServer,
    endpoint: &str,
    data: serde_json::Value,
) -> anyhow::Result<()> {
    let response = server.post_json(endpoint, data).await?;
    let body = TestServer::response_body_string(response).await?;
    black_box(body);
    Ok(())
}

/// Stress test with DynamoDB
async fn stress_test_dynamo_rest(
    server: &TestServer,
    concurrent_requests: usize,
) -> anyhow::Result<Vec<u128>> {
    let start_time = std::time::Instant::now();

    let tasks = (0..concurrent_requests).map(|_| {
        let server = server.clone();
        tokio::spawn(async move {
            let start = std::time::Instant::now();
            let response = server.get("/orders").await?;
            let _body = TestServer::response_body_string(response).await?;
            Ok::<u128, anyhow::Error>(start.elapsed().as_micros())
        })
    });

    let results = futures::future::join_all(tasks).await;
    let total_time = start_time.elapsed();

    let latencies: Result<Vec<_>, _> = results
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect();

    println!(
        "🗄️ DynamoDB - {} requêtes concurrentes en {:?}",
        concurrent_requests, total_time
    );
    Ok(latencies?)
}

fn benchmark_rest_dynamodb(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Try to create server with DynamoDB - skip if not available
    let server_result = rt.block_on(async {
        match create_rest_server_dynamodb().await {
            Ok(router) => start_test_server(router).await,
            Err(e) => {
                println!("⚠️ DynamoDB non disponible, benchmark ignoré: {}", e);
                return Err(e);
            }
        }
    });

    let server = match server_result {
        Ok(s) => s,
        Err(_) => {
            println!("🚫 Skipping DynamoDB benchmarks - DynamoDB non disponible");
            return;
        }
    };

    let mut group = c.benchmark_group("rest_dynamodb");

    // Benchmark GET endpoints individually
    let get_endpoints = vec![
        ("orders", "/orders"),
        ("invoices", "/invoices"),
        ("payments", "/payments"),
    ];

    for (name, endpoint) in get_endpoints {
        group.bench_with_input(
            BenchmarkId::new("get", name),
            &(&server, endpoint),
            |b, (server, endpoint)| {
                b.to_async(&rt).iter(|| async {
                    bench_single_get_endpoint_dynamo(server, endpoint)
                        .await
                        .unwrap()
                });
            },
        );
    }

    // Benchmark POST endpoints individually
    let post_test_cases = vec![
        ("orders", "/orders", data::sample_order()),
        ("invoices", "/invoices", data::sample_invoice()),
        ("payments", "/payments", data::sample_payment()),
    ];

    for (name, endpoint, sample_data) in post_test_cases {
        group.bench_with_input(
            BenchmarkId::new("post", name),
            &(&server, endpoint, sample_data),
            |b, (server, endpoint, data)| {
                b.to_async(&rt).iter(|| async {
                    bench_single_post_endpoint_dynamo(server, endpoint, data.clone())
                        .await
                        .unwrap()
                });
            },
        );
    }

    group.finish();
}

fn benchmark_graphql_dynamodb(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Try to create server with DynamoDB - skip if not available
    let server_result = rt.block_on(async {
        match create_graphql_server_dynamodb().await {
            Ok(router) => start_test_server(router).await,
            Err(e) => {
                println!(
                    "⚠️ DynamoDB non disponible pour GraphQL, benchmark ignoré: {}",
                    e
                );
                return Err(e);
            }
        }
    });

    let server = match server_result {
        Ok(s) => s,
        Err(_) => {
            println!("🚫 Skipping GraphQL DynamoDB benchmarks - DynamoDB non disponible");
            return;
        }
    };

    let mut group = c.benchmark_group("graphql_dynamodb");

    // Benchmark individual queries
    let query_test_cases = vec![
        (
            "orders_query",
            json!({"query": "query { orders { id name number amount status } }"}),
        ),
        (
            "invoices_query",
            json!({"query": "query { invoices { id name number amount status } }"}),
        ),
        (
            "payments_query",
            json!({"query": "query { payments { id name number amount status } }"}),
        ),
    ];

    for (name, query) in query_test_cases {
        group.bench_with_input(
            BenchmarkId::new("query", name),
            &(&server, query),
            |b, (server, query)| {
                b.to_async(&rt).iter(|| async {
                    let response = server.post_json("/graphql", query.clone()).await.unwrap();
                    let body = TestServer::response_body_string(response).await.unwrap();
                    black_box(body);
                });
            },
        );
    }

    // Benchmark mutations
    let mutation_test_cases = vec![(
        "create_order",
        json!({
            "query": "mutation CreateOrder($input: OrderInput!) { createOrder(input: $input) { id name number amount status } }",
            "variables": {
                "input": {
                    "name": "DynamoDB Benchmark Order",
                    "status": "pending",
                    "number": "ORD-DDB-001",
                    "amount": 999.99,
                    "customer_name": "DynamoDB Customer",
                    "notes": "Benchmark test order with DynamoDB"
                }
            }
        }),
    )];

    for (name, mutation) in mutation_test_cases {
        group.bench_with_input(
            BenchmarkId::new("mutation", name),
            &(&server, mutation),
            |b, (server, mutation)| {
                b.to_async(&rt).iter(|| async {
                    let response = server
                        .post_json("/graphql", mutation.clone())
                        .await
                        .unwrap();
                    let body = TestServer::response_body_string(response).await.unwrap();
                    black_box(body);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_dynamodb_load_test(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let server_result = rt.block_on(async {
        match create_rest_server_dynamodb().await {
            Ok(router) => start_test_server(router).await,
            Err(e) => {
                println!(
                    "⚠️ DynamoDB non disponible pour load test, benchmark ignoré: {}",
                    e
                );
                return Err(e);
            }
        }
    });

    let server = match server_result {
        Ok(s) => s,
        Err(_) => {
            println!("🚫 Skipping DynamoDB load tests - DynamoDB non disponible");
            return;
        }
    };

    let mut group = c.benchmark_group("dynamodb_load_test");

    // Test différents niveaux de charge avec DynamoDB
    for concurrent_req in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_requests", concurrent_req),
            concurrent_req,
            |b, &concurrent_req| {
                b.to_async(&rt).iter(|| async {
                    let latencies = stress_test_dynamo_rest(&server, concurrent_req)
                        .await
                        .unwrap();

                    // Calcul des stats
                    let mut sorted_latencies = latencies;
                    sorted_latencies.sort();
                    let len = sorted_latencies.len();
                    let avg = sorted_latencies.iter().sum::<u128>() / len as u128;
                    let p50 = sorted_latencies[len / 2];
                    let p95 = sorted_latencies[len * 95 / 100];
                    let p99 = sorted_latencies[len * 99 / 100];

                    println!("🗄️ Stats DynamoDB pour {} requêtes:", concurrent_req);
                    println!("   Moyenne: {}µs", avg);
                    println!("   P50: {}µs", p50);
                    println!("   P95: {}µs", p95);
                    println!("   P99: {}µs", p99);

                    black_box(sorted_latencies);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_comparison(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Create both servers for comparison
    let (in_memory_server, dynamo_server_opt) = rt.block_on(async {
        let in_memory_router = endpoint_benchmarks::create_rest_server_in_memory()
            .await
            .unwrap();
        let in_memory_server = start_test_server(in_memory_router).await.unwrap();

        let dynamo_server_opt = match create_rest_server_dynamodb().await {
            Ok(router) => Some(start_test_server(router).await.unwrap()),
            Err(e) => {
                println!("⚠️ DynamoDB comparison ignoré: {}", e);
                None
            }
        };

        (in_memory_server, dynamo_server_opt)
    });

    if dynamo_server_opt.is_none() {
        println!("🚫 Skipping storage comparison - DynamoDB non disponible");
        return;
    }

    let dynamo_server = dynamo_server_opt.unwrap();

    let mut group = c.benchmark_group("storage_comparison");

    // Compare GET performance
    group.bench_function("get_orders_in_memory", |b| {
        b.to_async(&rt).iter(|| async {
            bench_single_get_endpoint_dynamo(&in_memory_server, "/orders")
                .await
                .unwrap()
        });
    });

    group.bench_function("get_orders_dynamodb", |b| {
        b.to_async(&rt).iter(|| async {
            bench_single_get_endpoint_dynamo(&dynamo_server, "/orders")
                .await
                .unwrap()
        });
    });

    // Compare POST performance
    group.bench_function("post_orders_in_memory", |b| {
        b.to_async(&rt).iter(|| async {
            bench_single_post_endpoint_dynamo(&in_memory_server, "/orders", data::sample_order())
                .await
                .unwrap()
        });
    });

    group.bench_function("post_orders_dynamodb", |b| {
        b.to_async(&rt).iter(|| async {
            bench_single_post_endpoint_dynamo(&dynamo_server, "/orders", data::sample_order())
                .await
                .unwrap()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_rest_dynamodb,
    benchmark_graphql_dynamodb,
    benchmark_dynamodb_load_test,
    benchmark_comparison
);

criterion_main!(benches);
