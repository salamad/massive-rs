//! Benchmarks for WebSocket message throughput.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

fn generate_trade_batch(count: usize) -> String {
    let mut batch = String::from("[");
    for i in 0..count {
        if i > 0 {
            batch.push(',');
        }
        batch.push_str(&format!(
            r#"{{"ev":"T","sym":"AAPL","x":4,"i":"{}","z":3,"p":150.25,"s":100,"c":[0],"t":1703001234567,"q":{}}}"#,
            i, i
        ));
    }
    batch.push(']');
    batch
}

fn bench_batch_parsing(c: &mut Criterion) {
    let batch_json = generate_trade_batch(100);

    let mut group = c.benchmark_group("batch_parsing");
    group.throughput(Throughput::Elements(100));

    group.bench_function("parse_100_trades", |b| {
        b.iter(|| {
            let events: Vec<massive_rs::ws::WsEvent> = serde_json::from_str(&batch_json).unwrap();
            assert_eq!(events.len(), 100);
        })
    });

    group.finish();
}

fn bench_batch_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_sizes");

    for size in [10, 50, 100, 500].iter() {
        let batch_json = generate_trade_batch(*size);
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_function(format!("parse_{}_trades", size), |b| {
            b.iter(|| {
                let events: Vec<massive_rs::ws::WsEvent> =
                    serde_json::from_str(&batch_json).unwrap();
                assert_eq!(events.len(), *size);
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_batch_parsing, bench_batch_sizes);
criterion_main!(benches);
