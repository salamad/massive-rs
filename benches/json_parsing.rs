//! Benchmarks for JSON parsing performance.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

fn bench_trade_parsing(c: &mut Criterion) {
    let trade_json = r#"{"ev":"T","sym":"AAPL","x":4,"i":"123","z":3,"p":150.25,"s":100,"c":[0],"t":1703001234567,"q":12345}"#;

    let mut group = c.benchmark_group("trade_parsing");
    group.throughput(Throughput::Bytes(trade_json.len() as u64));

    group.bench_function("serde_json", |b| {
        b.iter(|| {
            let _: massive_rs::ws::WsEvent = serde_json::from_str(trade_json).unwrap();
        })
    });

    group.finish();
}

fn bench_aggregate_parsing(c: &mut Criterion) {
    let agg_json = r#"{"T":"AAPL","o":150.0,"h":155.0,"l":148.0,"c":153.0,"v":1000000.0,"vw":151.5,"t":1703001234567,"n":5000}"#;

    let mut group = c.benchmark_group("aggregate_parsing");
    group.throughput(Throughput::Bytes(agg_json.len() as u64));

    group.bench_function("serde_json", |b| {
        b.iter(|| {
            let _: massive_rs::models::AggregateBar = serde_json::from_str(agg_json).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, bench_trade_parsing, bench_aggregate_parsing);
criterion_main!(benches);
