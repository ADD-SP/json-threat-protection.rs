#![no_main]

use libfuzzer_sys::fuzz_target;
use json_threat_protection as jtp;
use serde_json::Value;


fuzz_target!(|data: &[u8]| {
    let lossy_str = String::from_utf8_lossy(data);
    if lossy_str.contains(r#""$serde_json::private::Number""#) {
        // The `$serde_json::private::Number` as the first field name is a magic string in serde_json,
        // which expects a string value, but the fuzzer may generate a number value like:
        // `{"$serde_json::private::Number":1234567890}`, which will cause a error in serde_json like:
        // `Error("invalid type: integer `1234567890`, expected a string", line: 1, column: 23)`.
        // To make the fuzzer happy, we just skip this case.
        // Please check out the following links for more details:
        // https://github.com/serde-rs/json/blob/3f1c6de4af28b1f6c5100da323f2bffaf7c2083f/src/number.rs#L17-L18
        // https://github.com/serde-rs/json/blob/3f1c6de4af28b1f6c5100da323f2bffaf7c2083f/tests/test.rs#L1004-L1018
        let re = regex::Regex::new(r#"(?s).*\{[[:space:]]*"\$serde_json::private::Number"[[:space:]]*:.+"#).unwrap();
        if re.is_match(&lossy_str) {
            return;
        }
    }

    let is_valid = serde_json::from_slice::<Value>(data).is_ok();
    assert_eq!(jtp::from_slice(data).validate().is_ok(), is_valid);

    let mut validator = jtp::from_slice(data);
    loop {
        let r = validator.validate_with_steps(2000);
        if r.is_err() {
            if !is_valid {
                break;
            }
            
            panic!("Unexpected error: {:?}", r);
        }

        if r.unwrap() {
            assert_eq!(true, is_valid);
            break;
        }
    }

    let reader = std::io::BufReader::new(data);
    assert_eq!(jtp::from_reader(reader).validate().is_ok(), is_valid);
});
