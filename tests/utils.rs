#[macro_export]
macro_rules! mkjson {
    ($($tt:tt)*) => {{
        let v: String = serde_json::to_string(&json!($($tt)*)).unwrap();
        v.into_bytes()
    }}
}
