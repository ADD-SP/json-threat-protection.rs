mod utils;
use json_threat_protection::{self as jtp, read::Position, Error};

#[test]
fn unclosed_array() {
    let v = r#"[1, 2, 3"#;
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
fn missing_comma() {
    let v = r#"[1 2, 3]"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::InvalidJSON(Position {
                line: 1,
                column: 4,
                offset: 4
            })
        ),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn trailing_comma() {
    let v = r#"[1, 2, 3,]"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::InvalidJSON(Position {
                line: 1,
                column: 10,
                offset: 10
            })
        ),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn leading_comma() {
    let v = r#"[,1, 2, 3]"#;
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
fn missing_lb() {
    let v = r#"1, 2, 3]"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::TrailingData(Position {
                line: 1,
                column: 2,
                offset: 2
            })
        ),
        "unexpected error: {:?}",
        err
    );
}
