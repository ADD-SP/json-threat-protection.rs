use json_threat_protection as jtp;

// const TWITTER: &str = include_str!("../../twitter.json");
// const CANADA: &str = include_str!("../../canada.json");
// const CITM_CATALOG: &str = include_str!("../../citm_catalog.json");

const TWITTER: &str = "twitter.json";
const CANADA: &str = "canada.json";
const CITM_CATALOG: &str = "citm_catalog.json";

const ITERATIONS: usize = 2000;

fn traverse_json(data: &serde_json::Value) {
    match data {
        serde_json::Value::Object(map) => {
            for (_, value) in map {
                traverse_json(value);
            }
        }
        serde_json::Value::Array(array) => {
            for value in array {
                traverse_json(value);
            }
        }
        _ => {}
    }
}

// fn serde_json_elapsed(data: &str) -> std::time::Duration {
//     let start_time = std::time::Instant::now();
//     for _ in 0..ITERATIONS {
//         let v = serde_json::from_str::<serde_json::Value>(data).unwrap();
//         traverse_json(&v);
//     }
//     start_time.elapsed()
// }

// fn validator_elapsed(data: &str) -> std::time::Duration {
//     let start_time = std::time::Instant::now();
//     for _ in 0..ITERATIONS {
//         let validator = jtp::from_str(data);
//         validator.validate().unwrap();
//     }
//     start_time.elapsed()
// }

fn serde_json_elapsed(path: &str) -> std::time::Duration {
    assert!(std::path::Path::new(path).exists(), "file not found: {}", path);
    let mut culumative = std::time::Duration::from_secs(0);
    for _ in 0..ITERATIONS {
        let file = std::fs::File::open(path).unwrap();
        let reader = std::io::BufReader::new(file);
        let start_time = std::time::Instant::now();
        let v = serde_json::from_reader(reader).unwrap();
        traverse_json(&v);
        culumative += start_time.elapsed();
    }
    culumative
}

fn validator_elapsed(path: &str) -> std::time::Duration {
    assert!(std::path::Path::new(path).exists(), "file not found: {}", path);
    let mut culumative = std::time::Duration::from_secs(0);
    for _ in 0..ITERATIONS {
        let file = std::fs::File::open(path).unwrap();
        let reader = std::io::BufReader::new(file);
        let start_time = std::time::Instant::now();
        let validator = jtp::from_reader(reader);
        validator.validate().unwrap();
        culumative += start_time.elapsed();
    }
    culumative
}

fn main() {
    // let serde_json_twitter = serde_json_elapsed(TWITTER);
    // let validator_twitter = validator_elapsed(TWITTER);

    // let serde_json_canada = serde_json_elapsed(CANADA);
    // let validator_canada = validator_elapsed(CANADA);

    // let serde_json_citm_catalog = serde_json_elapsed(CITM_CATALOG);
    let validator_citm_catalog = validator_elapsed(CITM_CATALOG);

    // let twitter_size = ((TWITTER.len() * ITERATIONS) as f64) / 1024f64 / 1024f64; // MB
    // let canada_size = ((CANADA.len() * ITERATIONS) as f64) / 1024f64 / 1024f64; // MB
    // let citm_catalog_size = ((CITM_CATALOG.len() * ITERATIONS) as f64) / 1024f64 / 1024f64; // MB

    // let serde_json_twitter_rate = ((twitter_size as f64) / serde_json_twitter.as_secs_f64()) as u64;
    // let validator_twitter_rate = ((twitter_size as f64) / validator_twitter.as_secs_f64()) as u64;

    // let serde_json_canada_rate = ((canada_size as f64) / serde_json_canada.as_secs_f64()) as u64;
    // let validator_canada_rate = ((canada_size as f64) / validator_canada.as_secs_f64()) as u64;

    // let serde_json_citm_catalog_rate =
    //     ((citm_catalog_size as f64) / serde_json_citm_catalog.as_secs_f64()) as u64;
    // let validator_citm_catalog_rate =
    //     ((citm_catalog_size as f64) / validator_citm_catalog.as_secs_f64()) as u64;

    // println!(
    //     "twitter: serde_json: {:?}, validator: {:?}, speedup: {:.2}%",
    //     serde_json_twitter_rate,
    //     validator_twitter_rate,
    //     ((serde_json_twitter.as_millis() as f64 / validator_twitter.as_millis() as f64) * 100f64) - 100f64
    // );
    // println!(
    //     "canada: serde_json: {:?}, validator: {:?}, speedup: {:.2}%",
    //     serde_json_canada_rate,
    //     validator_canada_rate,
    //     ((serde_json_canada.as_millis() as f64 / validator_canada.as_millis() as f64) * 100f64) - 100f64
    // );
    // println!(
    //     "citm_catalog: serde_json: {:?}, validator: {:?}, speedup: {:.2}%",
    //     serde_json_citm_catalog_rate,
    //     validator_citm_catalog_rate,
    //     ((serde_json_citm_catalog.as_millis() as f64 / validator_citm_catalog.as_millis() as f64) * 100f64) - 100f64
    // );
}
