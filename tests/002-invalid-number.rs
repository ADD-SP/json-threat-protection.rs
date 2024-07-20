mod utils;
use json_threat_protection::{self as jtp, read::Position, Error, LexerError, ReadError};

#[test]
fn no_number_characters_after_fraction() {
    let v = r#"{"key": 123.}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::LexerError(LexerError::ReadError(
                ReadError::NoNumberCharactersAfterFraction(Position {
                    line: 1,
                    column: 12,
                    offset: 12
                })
            ))
        ),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn no_number_characters_after_exponent() {
    let v = r#"{"key": 123e}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::LexerError(LexerError::ReadError(
                ReadError::NoNumberCharactersAfterExponent(Position {
                    line: 1,
                    column: 12,
                    offset: 12
                })
            ))
        ),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn leading_zeros() {
    let v = r#"{"key": 0123}"#;
    let err = jtp::from_slice(v.as_bytes()).validate().unwrap_err();

    assert!(
        matches!(
            err,
            Error::LexerError(LexerError::ReadError(ReadError::LeadingZerosInNumber(
                Position {
                    line: 1,
                    column: 9,
                    offset: 9
                }
            )))
        ),
        "unexpected error: {:?}",
        err
    );
}
