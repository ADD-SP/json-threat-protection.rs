mod utils;

use json_threat_protection::{self as jtp, read::Position};
use serde_json::json;

#[test]
fn no_limit() {
    let v = mkjson!(
        {
            "key": "value",
            "key2": 123,
            "key3": true,
            "key4": false,
            "key5": null,
            "key6": [1, 2, 3],
            "key7": {
                "key8": "value",
                "key9": 123,
                "key10": true,
                "key11": false,
                "key12": null,
                "key13": [1, 2, 3]
            }
        }
    );

    let validator = jtp::from_slice(&v);
    validator.validate().unwrap();
}

#[test]
fn limit_string() {
    let v = mkjson!("123456");

    let result = jtp::from_slice(&v).with_max_string_length(5).validate();
    assert!(result.is_err());
    match result.unwrap_err() {
        jtp::Error::MaxStringLengthExceeded {
            position,
            limit,
            str,
        } => {
            assert_eq!(
                position,
                Position {
                    line: 1,
                    column: 8,
                    offset: 8,
                }
            );
            assert_eq!(limit, 5);
            assert_eq!(str, "123456");
        }
        _ => panic!("unexpected error"),
    }

    jtp::from_slice(&v)
        .with_max_string_length(6)
        .validate()
        .unwrap();

    let result = jtp::from_reader(std::io::BufReader::new(v.as_slice()))
        .with_max_string_length(5)
        .validate();
    assert!(result.is_err());
    match result.unwrap_err() {
        jtp::Error::MaxStringLengthExceeded {
            position,
            limit,
            str,
        } => {
            assert_eq!(
                position,
                Position {
                    line: 1,
                    column: 8,
                    offset: 8,
                }
            );
            assert_eq!(limit, 5);
            assert_eq!(str, "123456");
        }
        _ => panic!("unexpected error"),
    }

    jtp::from_reader(std::io::BufReader::new(v.as_slice()))
        .with_max_string_length(6)
        .validate()
        .unwrap();
}

#[test]
fn limit_array() {
    let v = mkjson!([1, 2, 3, 4, 5]);

    let result = jtp::from_slice(&v).with_max_array_entries(4).validate();
    assert!(result.is_err());
    match result.unwrap_err() {
        jtp::Error::MaxArrayEntriesExceeded { position, limit } => {
            assert_eq!(
                position,
                Position {
                    line: 1,
                    column: 10,
                    offset: 10,
                }
            );
            assert_eq!(limit, 4);
        }
        _ => panic!("unexpected error"),
    }

    jtp::from_slice(&v)
        .with_max_array_entries(5)
        .validate()
        .unwrap();

    let result = jtp::from_reader(std::io::BufReader::new(v.as_slice()))
        .with_max_array_entries(4)
        .validate();
    assert!(result.is_err());
    match result.unwrap_err() {
        jtp::Error::MaxArrayEntriesExceeded { position, limit } => {
            assert_eq!(
                position,
                Position {
                    line: 1,
                    column: 10,
                    offset: 10,
                }
            );
            assert_eq!(limit, 4);
        }
        _ => panic!("unexpected error"),
    }

    jtp::from_reader(std::io::BufReader::new(v.as_slice()))
        .with_max_array_entries(5)
        .validate()
        .unwrap();
}

#[test]
fn limit_object_entries() {
    let v = mkjson!({
        "key1": 1,
        "key2": 2,
        "key3": 3,
        "key4": 4,
        "key5": 5,
    });

    let result = jtp::from_slice(&v).with_max_object_entries(4).validate();
    assert!(result.is_err());
    match result.unwrap_err() {
        jtp::Error::MaxObjectEntriesExceeded { position, limit } => {
            assert_eq!(
                position,
                Position {
                    line: 1,
                    column: 45,
                    offset: 45,
                }
            );
            assert_eq!(limit, 4);
        }
        _ => panic!("unexpected error"),
    }

    jtp::from_slice(&v)
        .with_max_object_entries(5)
        .validate()
        .unwrap();

    let result = jtp::from_reader(std::io::BufReader::new(v.as_slice()))
        .with_max_object_entries(4)
        .validate();
    assert!(result.is_err());
    match result.unwrap_err() {
        jtp::Error::MaxObjectEntriesExceeded { position, limit } => {
            assert_eq!(
                position,
                Position {
                    line: 1,
                    column: 45,
                    offset: 45,
                }
            );
            assert_eq!(limit, 4);
        }
        _ => panic!("unexpected error"),
    }

    jtp::from_reader(std::io::BufReader::new(v.as_slice()))
        .with_max_object_entries(5)
        .validate()
        .unwrap();
}

#[test]
fn limit_object_entry_name_length() {
    let v = mkjson!({
        "12345": 1,
        "123456": 2,
    });

    let result = jtp::from_slice(&v)
        .with_max_object_entry_name_length(5)
        .validate();
    assert!(result.is_err());
    match result.unwrap_err() {
        jtp::Error::MaxObjectEntryNameLengthExceeded {
            position,
            limit,
            name,
        } => {
            assert_eq!(
                position,
                Position {
                    line: 1,
                    column: 19,
                    offset: 19,
                }
            );
            assert_eq!(limit, 5);
            assert_eq!(name, "123456");
        }
        _ => panic!("unexpected error"),
    }

    jtp::from_slice(&v)
        .with_max_object_entry_name_length(6)
        .validate()
        .unwrap();

    let result = jtp::from_reader(std::io::BufReader::new(v.as_slice()))
        .with_max_object_entry_name_length(5)
        .validate();
    assert!(result.is_err());
    match result.unwrap_err() {
        jtp::Error::MaxObjectEntryNameLengthExceeded {
            position,
            limit,
            name,
        } => {
            assert_eq!(
                position,
                Position {
                    line: 1,
                    column: 19,
                    offset: 19,
                }
            );
            assert_eq!(limit, 5);
            assert_eq!(name, "123456");
        }
        _ => panic!("unexpected error"),
    }

    jtp::from_reader(std::io::BufReader::new(v.as_slice()))
        .with_max_object_entry_name_length(6)
        .validate()
        .unwrap();
}

#[test]
fn disallow_duplicate_object_entry_name() {
    let v = r#"{"key":1,"key":2}"#;

    let result = jtp::from_str(v)
        .disallow_duplicate_object_entry_name()
        .validate();
    assert!(result.is_err());
    match result.unwrap_err() {
        jtp::Error::DuplicateObjectEntryName { position, key } => {
            assert_eq!(
                position,
                Position {
                    line: 1,
                    column: 14,
                    offset: 14,
                }
            );
            assert_eq!(key, "key");
        }
        _ => panic!("unexpected error"),
    }

    jtp::from_str(v).validate().unwrap();

    let result = jtp::from_reader(std::io::BufReader::new(v.as_bytes()))
        .disallow_duplicate_object_entry_name()
        .validate();
    assert!(result.is_err());
    match result.unwrap_err() {
        jtp::Error::DuplicateObjectEntryName { position, key } => {
            assert_eq!(
                position,
                Position {
                    line: 1,
                    column: 14,
                    offset: 14,
                }
            );
            assert_eq!(key, "key");
        }
        _ => panic!("unexpected error"),
    }

    jtp::from_reader(std::io::BufReader::new(v.as_bytes()))
        .validate()
        .unwrap();
}

#[test]
fn limit_depth() {
    let v = r#"{"key":{"key":{"key":{"key":{"key":1}}}}}"#;

    let result = jtp::from_str(v).with_max_depth(4).validate();
    assert!(result.is_err());
    match result.unwrap_err() {
        jtp::Error::MaxDepthExceeded { position, limit } => {
            assert_eq!(
                position,
                Position {
                    line: 1,
                    column: 29,
                    offset: 29,
                }
            );
            assert_eq!(limit, 4);
        }
        _ => panic!("unexpected error"),
    }

    jtp::from_str(v).with_max_depth(5).validate().unwrap();

    let result = jtp::from_reader(std::io::BufReader::new(v.as_bytes()))
        .with_max_depth(4)
        .validate();
    assert!(result.is_err());
    match result.unwrap_err() {
        jtp::Error::MaxDepthExceeded { position, limit } => {
            assert_eq!(
                position,
                Position {
                    line: 1,
                    column: 29,
                    offset: 29,
                }
            );
            assert_eq!(limit, 4);
        }
        _ => panic!("unexpected error"),
    }

    jtp::from_reader(std::io::BufReader::new(v.as_bytes()))
        .with_max_depth(5)
        .validate()
        .unwrap();
}

#[test]
fn skip_whitespace() {
    let v = r#"    {"key"    :   1
    
    ,   
     
        "key2":    "32 "
     }     "#;

    jtp::from_str(v).validate().unwrap();
    jtp::from_reader(std::io::BufReader::new(v.as_bytes()))
        .validate()
        .unwrap();
}
