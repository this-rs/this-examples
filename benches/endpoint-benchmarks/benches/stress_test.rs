use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use endpoint_benchmarks::{
    create_rest_server_in_memory, create_graphql_server_in_memory, start_test_server, data, TestServer
};
use tokio::runtime::Runtime;
use serde_json::json;

async fn stress_test_rest(server: &TestServer, concurrent_requests: usize) -> anyhow::Result<Vec<u128>> {
    let start_time = std::time::Instant::now();
    
    let tasks = (0..concurrent_requests).map(|i| {
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
    
    let latencies: Result<Vec<_>, _> = results.into_iter().collect::<Result<Vec<_>, _>>()?
        .into_iter().collect();
    
    println!("🚀 {} requêtes concurrentes en {:?}", concurrent_requests, total_time);
    Ok(latencies?)
}

async fn stress_test_graphql(server: &TestServer, concurrent_requests: usize) -> anyhow::Result<Vec<u128>> {
    let start_time = std::time::Instant::now();
    
    let query = json!({"query": "query { orders { id name number amount status } }"});
    
    let tasks = (0..concurrent_requests).map(|_| {
        let server = server.clone();
        let query = query.clone();
        tokio::spawn(async move {
            let start = std::time::Instant::now();
            let response = server.post_json("/graphql", query).await?;
            let _body = TestServer::response_body_string(response).await?;
            Ok::<u128, anyhow::Error>(start.elapsed().as_micros())
        })
    });

    let results = futures::future::join_all(tasks).await;
    let total_time = start_time.elapsed();
    
    let latencies: Result<Vec<_>, _> = results.into_iter().collect::<Result<Vec<_>, _>>()?
        .into_iter().collect();
    
    println!("🌐 {} requêtes GraphQL concurrentes en {:?}", concurrent_requests, total_time);
    Ok(latencies?)
}

fn stress_test_rest_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let server = rt.block_on(async {
        let router = create_rest_server_in_memory().await.unwrap();
        start_test_server(router).await.unwrap()
    });

    let mut group = c.benchmark_group("rest_stress_test");
    
    // Test différents niveaux de charge
    for concurrent_req in [1, 10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_requests", concurrent_req),
            concurrent_req,
            |b, &concurrent_req| {
                b.to_async(&rt).iter(|| async {
                    let latencies = stress_test_rest(&server, concurrent_req).await.unwrap();
                    
                    // Calcul des stats
                    let mut sorted_latencies = latencies;
                    sorted_latencies.sort();
                    let len = sorted_latencies.len();
                    let avg = sorted_latencies.iter().sum::<u128>() / len as u128;
                    let p50 = sorted_latencies[len / 2];
                    let p95 = sorted_latencies[len * 95 / 100];
                    let p99 = sorted_latencies[len * 99 / 100];
                    
                    println!("📊 Stats pour {} requêtes:", concurrent_req);
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

fn stress_test_graphql_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let server = rt.block_on(async {
        let router = create_graphql_server_in_memory().await.unwrap();
        start_test_server(router).await.unwrap()
    });

    let mut group = c.benchmark_group("graphql_stress_test");
    
    // Test différents niveaux de charge
    for concurrent_req in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_queries", concurrent_req),
            concurrent_req,
            |b, &concurrent_req| {
                b.to_async(&rt).iter(|| async {
                    let latencies = stress_test_graphql(&server, concurrent_req).await.unwrap();
                    
                    // Calcul des stats
                    let mut sorted_latencies = latencies;
                    sorted_latencies.sort();
                    let len = sorted_latencies.len();
                    let avg = sorted_latencies.iter().sum::<u128>() / len as u128;
                    let p50 = sorted_latencies[len / 2];
                    let p95 = sorted_latencies[len * 95 / 100];
                    let p99 = sorted_latencies[len * 99 / 100];
                    
                    println!("🌐 Stats GraphQL pour {} requêtes:", concurrent_req);
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

fn throughput_test(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let server = rt.block_on(async {
        let router = create_rest_server_in_memory().await.unwrap();
        start_test_server(router).await.unwrap()
    });

    let mut group = c.benchmark_group("throughput_test");
    
    group.bench_function("sustained_load_1000_requests", |b| {
        b.to_async(&rt).iter(|| async {
            let start = std::time::Instant::now();
            
            // Envoyer 1000 requêtes par vagues de 50
            for _wave in 0..20 {
                let wave_tasks = (0..50).map(|_| {
                    let server = server.clone();
                    tokio::spawn(async move {
                        server.get("/orders").await
                    })
                });
                
                let _wave_results = futures::future::join_all(wave_tasks).await;
            }
            
            let total_time = start.elapsed();
            let rps = 1000.0 / total_time.as_secs_f64();
            
            println!("🔥 Throughput: {:.0} requêtes/seconde", rps);
            black_box(rps);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    stress_test_rest_scaling,
    stress_test_graphql_scaling,
    throughput_test
);

criterion_main!(benches);