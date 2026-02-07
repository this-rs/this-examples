use billing::entities::{invoice::Invoice, order::Order, payment::Payment};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use this::prelude::*;

fn benchmark_invoice_creation(c: &mut Criterion) {
    c.bench_function("invoice_create", |b| {
        b.iter(|| {
            let invoice = Invoice::new(
                black_box("Invoice Benchmark".into()),
                black_box("draft".into()),
                black_box("INV-2024-001".into()),
                black_box(1500.50),
                black_box(Some("2024-12-31".into())),
                black_box(None),
            );
            black_box(invoice)
        })
    });
}

fn benchmark_invoice_validation(c: &mut Criterion) {
    c.bench_function("invoice_validate_valid", |b| {
        b.iter(|| {
            let invoice = Invoice::new(
                black_box("Valid Invoice".into()),
                black_box("sent".into()),
                black_box("INV-2024-002".into()),
                black_box(2500.75),
                black_box(Some("2024-11-30".into())),
                black_box(None),
            );
            black_box(invoice)
        })
    });

    c.bench_function("invoice_with_paid_date", |b| {
        b.iter(|| {
            let invoice = Invoice::new(
                black_box("Paid Invoice".into()),
                black_box("paid".into()),
                black_box("INV-2024-003".into()),
                black_box(1200.00),
                black_box(Some("2024-11-30".into())),
                black_box(Some("2024-11-15".into())),
            );
            black_box(invoice)
        })
    });
}

fn benchmark_order_creation(c: &mut Criterion) {
    c.bench_function("order_create", |b| {
        b.iter(|| {
            let order = Order::new(
                black_box("Order Benchmark".into()),
                black_box("pending".into()),
                black_box("ORD-2024-001".into()),
                black_box(3500.25),
                black_box(Some("John Doe".into())),
                black_box(Some("Special delivery instructions".into())),
            );
            black_box(order)
        })
    });
}

fn benchmark_order_update(c: &mut Criterion) {
    c.bench_function("order_creation_and_clone", |b| {
        b.iter(|| {
            // Create order
            let order = Order::new(
                black_box("Order Update Test".into()),
                black_box("pending".into()),
                black_box("ORD-2024-002".into()),
                black_box(1200.00),
                black_box(Some("Customer".into())),
                black_box(None),
            );
            // Clone it to simulate update operations
            let cloned_order = order.clone();
            black_box((order, cloned_order))
        })
    });
}

fn benchmark_payment_creation(c: &mut Criterion) {
    c.bench_function("payment_create", |b| {
        b.iter(|| {
            let payment = Payment::new(
                black_box("Payment Benchmark".into()),
                black_box("pending".into()),
                black_box("PAY-2024-001".into()),
                black_box(750.00),
                black_box("credit_card".into()),
                black_box(Some("txn_bench_001".into())),
            );
            black_box(payment)
        })
    });
}

fn benchmark_bulk_operations(c: &mut Criterion) {
    c.bench_function("bulk_invoice_creation", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for i in 0..100 {
                let invoice = Invoice::new(
                    black_box(format!("Bulk Invoice {}", i)),
                    black_box("draft".into()),
                    black_box(format!("INV-BULK-{:03}", i)),
                    black_box((i as f64) * 10.5),
                    black_box(Some("2024-12-31".into())),
                    black_box(None),
                );
                results.push(invoice);
            }
            black_box(results)
        })
    });
}

fn benchmark_serialization(c: &mut Criterion) {
    let invoice = Invoice::new(
        "Serialization Test".into(),
        "paid".into(),
        "INV-SER-001".into(),
        1000.00,
        Some("2024-11-15".into()),
        Some("2024-11-10".into()),
    );

    c.bench_function("invoice_to_json", |b| {
        b.iter(|| {
            let json_str = serde_json::to_string(black_box(&invoice));
            black_box(json_str)
        })
    });

    let json_str = serde_json::to_string(&invoice).unwrap();
    c.bench_function("invoice_from_json", |b| {
        b.iter(|| {
            let invoice: Result<Invoice, _> = serde_json::from_str(black_box(&json_str));
            black_box(invoice)
        })
    });
}

fn benchmark_field_access(c: &mut Criterion) {
    let invoice = Invoice::new(
        "Field Access Test".into(),
        "sent".into(),
        "INV-ACCESS-001".into(),
        1500.00,
        Some("2024-12-01".into()),
        None,
    );

    c.bench_function("invoice_field_access", |b| {
        b.iter(|| {
            let id = invoice.id;
            let name = &invoice.name;
            let number = &invoice.number;
            let amount = invoice.amount;
            black_box((id, name, number, amount))
        })
    });
}

criterion_group!(
    benches,
    benchmark_invoice_creation,
    benchmark_invoice_validation,
    benchmark_order_creation,
    benchmark_order_update,
    benchmark_payment_creation,
    benchmark_bulk_operations,
    benchmark_serialization,
    benchmark_field_access
);

criterion_main!(benches);
