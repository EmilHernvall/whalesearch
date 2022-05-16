use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::Value;

use whalesearch;

const TEST_QUERY: &str = r#"size > 20 || range == "antarctic""#;

fn ast(c: &mut Criterion) {
    let parser = whalesearch::dsl::BoolExprParser::new();
    let query: whalesearch::BoolExpr = parser.parse(TEST_QUERY).unwrap();

    let data = std::fs::read_to_string("whales.json").unwrap();
    let whales: Vec<HashMap<String, Value>> = serde_json::from_str(&data).unwrap();

    c.bench_function("ast", |b| b.iter(|| {
        let mut matches = 0;
        for whale in &whales {
            if matches!(query.evaluate(&whale), Some(true)) {
                matches += 1;
            }
        }
        assert_eq!(matches, 12);
    }));
}

fn vm(c: &mut Criterion) {
    let parser = whalesearch::dsl::BoolExprParser::new();
    let query: whalesearch::BoolExpr = parser.parse(TEST_QUERY).unwrap();
    let code = query.to_opcode();

    let data = std::fs::read_to_string("whales.json").unwrap();
    let whales: Vec<HashMap<String, Value>> = serde_json::from_str(&data).unwrap();

    c.bench_function("vm", |b| b.iter(|| {
        let mut matches = 0;
        for whale in &whales {
            let result = whalesearch::evaluate(&code, &whale);
            if matches!(result, Some(Value::Bool(true))) {
                matches += 1;
            }
        }
        assert_eq!(matches, 12);
    }));
}

criterion_group!(benches, ast, vm);
criterion_main!(benches);
