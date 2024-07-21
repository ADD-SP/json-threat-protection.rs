use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn traverse_json(data: &serde_json::Value, depth: usize) {
    if depth > 1024 {
        panic!("Too deep");
    }

    match data {
        serde_json::Value::Object(map) => {
            let mut keys = std::collections::HashSet::with_capacity(8);
            let mut entries = 0;
            for (key, value) in map {
                if !keys.insert(key) {
                    panic!("Duplicate key");
                }
                traverse_json(value, depth + 1);

                entries += 1;
                if entries > isize::MAX as usize {
                    panic!("Too many entries");
                }
            }
        }
        serde_json::Value::Array(array) => {
            let mut entries = 0;
            for value in array {
                traverse_json(value, depth + 1);
                entries += 1;
                if entries > isize::MAX as usize {
                    panic!("Too many entries");
                }
            }
        }
        serde_json::Value::String(s) => {
            if s.len() > isize::MAX as usize {
                panic!("String too long");
            }
        }
        serde_json::Value::Number(_) | serde_json::Value::Bool(_) | serde_json::Value::Null => (),
    }
}

fn invoke_serde_json(content: &str) {
    let v: serde_json::Value = serde_json::from_str(content).unwrap();
    #[allow(clippy::unit_arg)]
    black_box(traverse_json(&v, 1));
}

fn invoke_validator(content: &str) {
    let validator = json_threat_protection::from_str(content);
    #[allow(clippy::unit_arg)]
    black_box(validator.validate().unwrap());
}

fn criterion_benchmark(c: &mut Criterion) {
    let path = match std::env::var("JSON_FILE") {
        Ok(path) => path,
        Err(_) => {
            panic!("Please set JSON_FILE environment variable to the path of the JSON file to be used for benchmarking.");
        }
    };

    let content = std::fs::read_to_string(path).unwrap();

    let mut group = c.benchmark_group("JSON Validation");

    group.bench_function("serde_json", |b| {
        b.iter(|| invoke_serde_json(black_box(&content)))
    });
    group.bench_function("json-threat-protection", |b| {
        b.iter(|| invoke_validator(black_box(&content)))
    });
    group.finish();

    // c.bench_function("serde_json", |b| {
    //     b.iter(|| invoke_serde_json(black_box(&content)))
    // });
    // c.bench_function("json-threat-protection", |b| {
    //     b.iter(|| invoke_validator(black_box(&content)))
    // });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
