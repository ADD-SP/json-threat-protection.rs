#![no_main]

use libfuzzer_sys::fuzz_target;
use json_threat_protection as jtp;
use serde_json::Value;


fuzz_target!(|data: &[u8]| {
    let is_valid = serde_json::from_slice::<Value>(data).is_ok();

    assert_eq!(jtp::from_slice(data).validate().is_ok(), is_valid);

    let reader = std::io::BufReader::new(data);
    assert_eq!(jtp::from_reader(reader).validate().is_ok(), is_valid);
});
