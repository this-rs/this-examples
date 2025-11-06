use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use endpoint_benchmarks::{
    create_rest_server_in_memory, start_test_server, data, TestServer
};
use tokio::runtime::Runtime;

/// Benchmark REST GET endpoints
async fn bench_rest_get_endpoints(server: &TestServer) -> anyhow::Result<()> {
    // Test different GET endpoints
    let endpoints = vec![
        "/orders",
        "/invoices", 
        "/payments",
    ];

    for endpoint in endpoints {
        let response = server.get(endpoint).await?;
        black_box(response);
    }

    Ok(())
}

/// Benchmark REST POST endpoints 
async fn bench_rest_post_endpoints(server: &TestServer) -> anyhow::Result<()> {
    // Test POST endpoints with sample data
    let test_cases = vec![
        ("/orders", data::sample_order()),
        ("/invoices", data::sample_invoice()),
        ("/payments", data::sample_payment()),
    ];

    for (endpoint, sample_data) in test_cases {
        let response = server.post_json(endpoint, black_box(sample_data)).await?;
        black_box(response);
    }

    Ok(())
}

/// Benchmark individual GET endpoint
async fn bench_single_get_endpoint(server: &TestServer, endpoint: &str) -> anyhow::Result<()> {
    let response = server.get(endpoint).await?;
    let body = TestServer::response_body_string(response).await?;
    black_box(body);
    Ok(())
}

/// Benchmark individual POST endpoint
async fn bench_single_post_endpoint(server: &TestServer, endpoint: &str, data: serde_json::Value) -> anyhow::Result<()> {
    let response = server.post_json(endpoint, data).await?;
    let body = TestServer::response_body_string(response).await?;
    black_box(body);
    Ok(())
}

fn benchmark_rest_in_memory(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Create server once for all benchmarks
    let server = rt.block_on(async {
        let router = create_rest_server_in_memory().await.unwrap();
        start_test_server(router).await.unwrap()
    });

    let mut group = c.benchmark_group("rest_in_memory");

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
                    bench_single_get_endpoint(server, endpoint).await.unwrap()
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
                    bench_single_post_endpoint(server, endpoint, data.clone()).await.unwrap()
                });
            },
        );
    }

    // Benchmark combined operations
    group.bench_function("get_all_endpoints", |b| {
        b.to_async(&rt).iter(|| async {
            bench_rest_get_endpoints(black_box(&server)).await.unwrap()
        });
    });

    group.bench_function("post_all_endpoints", |b| {
        b.to_async(&rt).iter(|| async {
            bench_rest_post_endpoints(black_box(&server)).await.unwrap()
        });
    });

    group.finish();
}

fn benchmark_rest_load_test(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let server = rt.block_on(async {
        let router = create_rest_server_in_memory().await.unwrap();
        start_test_server(router).await.unwrap()
    });

    let mut group = c.benchmark_group("rest_load_test");

    // Simulate concurrent requests
    group.bench_function("concurrent_order_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let tasks = (0..10).map(|_| {
                let server = server.clone();
                tokio::spawn(async move {
                    server.post_json("/orders", data::sample_order()).await.unwrap()
                })
            });

            let results = futures::future::join_all(tasks).await;
            black_box(results);
        });
    });

    group.bench_function("mixed_operations", |b| {
        b.to_async(&rt).iter(|| async {
            // Mix of GET and POST operations
            let get_tasks = (0..5).map(|_| {
                let server = server.clone();
                tokio::spawn(async move {
                    server.get("/orders").await.unwrap()
                })
            });

            let post_tasks = (0..5).map(|_| {
                let server = server.clone();
                tokio::spawn(async move {
                    server.post_json("/invoices", data::sample_invoice()).await.unwrap()
                })
            });

            let (get_results, post_results) = tokio::join!(
                futures::future::join_all(get_tasks),
                futures::future::join_all(post_tasks)
            );

            black_box((get_results, post_results));
        });
    });

    group.finish();
}

fn benchmark_rest_response_parsing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let server = rt.block_on(async {
        let router = create_rest_server_in_memory().await.unwrap();
        start_test_server(router).await.unwrap()
    });

    let mut group = c.benchmark_group("rest_response_parsing");

    group.bench_function("parse_orders_response", |b| {
        b.to_async(&rt).iter(|| async {
            let response = server.get("/orders").await.unwrap();
            let body = TestServer::response_body_string(response).await.unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
            black_box(parsed);
        });
    });

    group.bench_function("create_and_parse_invoice", |b| {
        b.to_async(&rt).iter(|| async {
            let response = server.post_json("/invoices", data::sample_invoice()).await.unwrap();
            let body = TestServer::response_body_string(response).await.unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
            black_box(parsed);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_rest_in_memory,
    benchmark_rest_load_test,
    benchmark_rest_response_parsing
);

criterion_main!(benches);