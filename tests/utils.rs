#[macro_export]
macro_rules! mkjson {
    ($($tt:tt)*) => {{
        let v: String = serde_json::to_string(&json!($($tt)*)).unwrap();
        v.into_bytes()
    }}
}

#[macro_export]
macro_rules! assert_invalid_number {
    ($err:ident, $line:literal, $column:literal, $offset:literal) => {{
        if matches!(
            $err,
            Error::LexerError(LexerError::ReadError(
                ReadError::NoNumberCharactersAfterFraction(Position {
                    line: $line,
                    column: $column,
                    offset: $offset
                })
            ))
        ) {
            // expected
        } else {
            panic!("unexpected error: {:?}", $err);
        }
    }};
}
