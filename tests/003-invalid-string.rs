mod utils;
use json_threat_protection::{self as jtp, read::Position, Error, LexerError, ReadError};

#[test]
fn invalid_utf8_sequence() {
    let v = r#"{"key": "\x80"}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::LexerError(LexerError::ReadError(ReadError::InvalidEscapeSequence(
                Position {
                    line: 1,
                    column: 11,
                    offset: 11
                }
            )))
        ),
        "unexpected error: {:?}",
        err
    );

    let v = r#"{"key": "\uD800"}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::LexerError(LexerError::ReadError(ReadError::UnexpectedEndOfInput(
                Position {
                    line: 1,
                    column: 15,
                    offset: 15
                }
            )))
        ),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn unclosed() {
    let v = r#"{"key": "value}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::LexerError(LexerError::ReadError(ReadError::UnclosedString(Position {
                line: 1,
                column: 15,
                offset: 15
            })))
        ),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn invalid_escape() {
    let v = r#"{"key": "\z"}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::LexerError(LexerError::ReadError(ReadError::InvalidEscapeSequence(
                Position {
                    line: 1,
                    column: 11,
                    offset: 11
                }
            )))
        ),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn invalid_incomplete_escape() {
    let v = r#"{"key": "\u"}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::LexerError(LexerError::ReadError(
                ReadError::UnexpectedEndOfInput(Position {
                    line: 1,
                    column: 11,
                    offset: 11
                })
            ))
        ),
        "unexpected error: {:?}",
        err
    );

    let v = r#"{"key": "\u1"}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::LexerError(LexerError::ReadError(
                ReadError::UnexpectedEndOfInput(Position {
                    line: 1,
                    column: 11,
                    offset: 11
                })
            ))
        ),
        "unexpected error: {:?}",
        err
    );

    let v = r#"{"key": "\u12"}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::LexerError(LexerError::ReadError(
                ReadError::InvalidEscapeSequence(Position {
                    line: 1,
                    column: 15,
                    offset: 15
                })
            ))
        ),
        "unexpected error: {:?}",
        err
    );

    let v = r#"{"key": "\u123"}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::LexerError(LexerError::ReadError(
                ReadError::InvalidEscapeSequence(Position {
                    line: 1,
                    column: 15,
                    offset: 15
                })
            ))
        ),
        "unexpected error: {:?}",
        err
    );
}
