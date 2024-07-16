mod utils;

use json_threat_protection as jtp;
use serde_json::json;

#[test]
fn smoke() {
    let v = mkjson!(false);

    jtp::from_slice(&v).validate().unwrap();

    // let v = mkjson!(
    //     {
    //         "key": "value",
    //         "key2": 123,
    //         "key3": true,
    //         "key4": false,
    //         "key5": null,
    //         "key6": [1, 2, 3],
    //         "key7": {
    //             "key8": "value",
    //             "key9": 123,
    //             "key10": true,
    //             "key11": false,
    //             "key12": null,
    //             "key13": [1, 2, 3]
    //         }
    //     }
    // );

    // let validator = jtp::from_slice(&v);
    // validator.validate().unwrap();
}
