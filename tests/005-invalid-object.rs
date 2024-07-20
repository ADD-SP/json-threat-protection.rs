mod utils;
use json_threat_protection::{self as jtp, read::Position, Error};

#[test]
fn unclosed() {
    let v = r#"{"key": "value""#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::InvalidJSON(Position {
                line: 1,
                column: 15,
                offset: 15
            })
        ),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn missing_comma() {
    let v = r#"{"key": "value" "key2": "value2"}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::InvalidJSON(Position {
                line: 1,
                column: 22,
                offset: 22
            })
        ),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn missing_colon() {
    let v = r#"{"key" "value"}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::InvalidJSON(Position {
                line: 1,
                column: 14,
                offset: 14
            })
        ),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn missing_key() {
    let v = r#"{: "value"}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::InvalidJSON(Position {
                line: 1,
                column: 2,
                offset: 2
            })
        ),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn missing_value() {
    let v = r#"{"key":}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::InvalidJSON(Position {
                line: 1,
                column: 8,
                offset: 8
            })
        ),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn missing_value2() {
    let v = r#"{"key":,}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::InvalidJSON(Position {
                line: 1,
                column: 8,
                offset: 8
            })
        ),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn missing_value3() {
    let v = r#"{"key":, "key2": "value2"}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::InvalidJSON(Position {
                line: 1,
                column: 8,
                offset: 8
            })
        ),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn leading_comma() {
    let v = r#"{,"key": "value"}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::InvalidJSON(Position {
                line: 1,
                column: 2,
                offset: 2
            })
        ),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn trailing_comma() {
    let v = r#"{"key": "value",}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::InvalidJSON(Position {
                line: 1,
                column: 17,
                offset: 17
            })
        ),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn missing_lb() {
    let v = r#""key": "value"}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::TrailingData(Position {
                line: 1,
                column: 6,
                offset: 6
            })
        ),
        "unexpected error: {:?}",
        err
    );
}
