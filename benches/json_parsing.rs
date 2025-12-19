//! Benchmarks for JSON parsing performance.
//!
//! Run with: cargo bench --bench json_parsing

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

fn bench_trade_parsing(c: &mut Criterion) {
    let trade_json = r#"{"ev":"T","sym":"AAPL","x":4,"i":"123","z":3,"p":150.25,"s":100,"c":[0],"t":1703001234567,"q":12345}"#;

    let mut group = c.benchmark_group("trade_parsing");
    group.throughput(Throughput::Bytes(trade_json.len() as u64));

    group.bench_function("serde_json_single", |b| {
        b.iter(|| {
            let _: massive_rs::ws::WsEvent = serde_json::from_str(black_box(trade_json)).unwrap();
        })
    });

    group.bench_function("parse_ws_events", |b| {
        b.iter(|| {
            let _ = massive_rs::parse::parse_ws_events(black_box(trade_json)).unwrap();
        })
    });

    group.finish();
}

fn bench_quote_parsing(c: &mut Criterion) {
    let quote_json = r#"{"ev":"Q","sym":"AAPL","bx":4,"bp":150.0,"bs":100,"ax":7,"ap":150.10,"as":200,"t":1703001234567}"#;

    let mut group = c.benchmark_group("quote_parsing");
    group.throughput(Throughput::Bytes(quote_json.len() as u64));

    group.bench_function("serde_json_single", |b| {
        b.iter(|| {
            let _: massive_rs::ws::WsEvent = serde_json::from_str(black_box(quote_json)).unwrap();
        })
    });

    group.bench_function("parse_ws_events", |b| {
        b.iter(|| {
            let _ = massive_rs::parse::parse_ws_events(black_box(quote_json)).unwrap();
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
            let _: massive_rs::models::AggregateBar =
                serde_json::from_str(black_box(agg_json)).unwrap();
        })
    });

    group.finish();
}

fn bench_batch_parsing(c: &mut Criterion) {
    // Generate a batch of 100 trades
    let trades: Vec<String> = (0..100)
        .map(|i| {
            format!(
                r#"{{"ev":"T","sym":"AAPL","x":4,"i":"{}","z":3,"p":{},"s":{},"c":[0],"t":{},"q":{}}}"#,
                i,
                150.0 + (i as f64 * 0.01),
                100 + i,
                1703001234567i64 + i as i64,
                12345 + i
            )
        })
        .collect();
    let batch_json = format!("[{}]", trades.join(","));

    let mut group = c.benchmark_group("batch_parsing");
    group.throughput(Throughput::Elements(100));

    group.bench_function("parse_100_trades", |b| {
        b.iter(|| {
            let _ = massive_rs::parse::parse_ws_events(black_box(&batch_json)).unwrap();
        })
    });

    group.finish();
}

fn bench_bytes_parsing(c: &mut Criterion) {
    let trade_json = r#"{"ev":"T","sym":"AAPL","x":4,"i":"123","z":3,"p":150.25,"s":100,"c":[0],"t":1703001234567,"q":12345}"#;

    let mut group = c.benchmark_group("bytes_parsing");
    group.throughput(Throughput::Bytes(trade_json.len() as u64));

    group.bench_function("parse_ws_events_bytes", |b| {
        b.iter(|| {
            let mut bytes = trade_json.as_bytes().to_vec();
            let _ = massive_rs::parse::parse_ws_events_bytes(black_box(&mut bytes)).unwrap();
        })
    });

    group.finish();
}

fn bench_event_type_extraction(c: &mut Criterion) {
    let trade_json = r#"{"ev":"T","sym":"AAPL","x":4,"i":"123","z":3,"p":150.25,"s":100}"#;
    let status_json = r#"{"ev":"status","status":"connected","message":"Connected successfully"}"#;

    let mut group = c.benchmark_group("event_helpers");

    group.bench_function("extract_event_type", |b| {
        b.iter(|| {
            let _ = massive_rs::parse::extract_event_type(black_box(trade_json));
        })
    });

    group.bench_function("is_status_message_trade", |b| {
        b.iter(|| {
            let _ = massive_rs::parse::is_status_message(black_box(trade_json));
        })
    });

    group.bench_function("is_status_message_status", |b| {
        b.iter(|| {
            let _ = massive_rs::parse::is_status_message(black_box(status_json));
        })
    });

    group.bench_function("estimate_event_count", |b| {
        let batch = r#"[{"ev":"T"},{"ev":"T"},{"ev":"T"}]"#;
        b.iter(|| {
            let _ = massive_rs::parse::estimate_event_count(black_box(batch));
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_trade_parsing,
    bench_quote_parsing,
    bench_aggregate_parsing,
    bench_batch_parsing,
    bench_bytes_parsing,
    bench_event_type_extraction
);
criterion_main!(benches);
