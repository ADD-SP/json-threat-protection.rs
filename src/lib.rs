//! A crate to protect against malicious JSON payloads.
//!
//! This crate provides functionality to validate JSON payloads against a set of constraints.
//! * Maximum depth of the JSON structure.
//! * Maximum length of strings.
//! * Maximum number of entries in arrays.
//! * Maximum number of entries in objects.
//! * Maximum length of object entry names.
//! * Whether to allow duplicate object entry names.
//!
//! This crate is designed to process untrusted JSON payloads,
//! such as it does not use recursion to validate the JSON structure.
//!
//! # Examples
//!
//! ```rust
//! use json_threat_protection as jtp;
//!
//! fn reject_highly_nested_json(data: &[u8], depth: usize) -> Result<(), jtp::Error> {
//!     jtp::from_slice(data).with_max_depth(depth).validate()
//! }
//!
//! fn reject_too_long_strings(data: &[u8], max_string_length: usize) -> Result<(), jtp::Error> {
//!     jtp::from_slice(data).with_max_string_length(max_string_length).validate()
//! }
//!
//! fn reject_too_many_array_entries(data: &[u8], max_array_entries: usize) -> Result<(), jtp::Error> {
//!     jtp::from_slice(data).with_max_array_entries(max_array_entries).validate()
//! }
//!
//! fn reject_too_many_object_entries(data: &[u8], max_object_entries: usize) -> Result<(), jtp::Error> {
//!    jtp::from_slice(data).with_max_object_entries(max_object_entries).validate()
//! }
//!
//! fn reject_too_long_object_entry_names(data: &[u8], max_object_entry_name_length: usize) -> Result<(), jtp::Error> {
//!    jtp::from_slice(data).with_max_object_entry_name_length(max_object_entry_name_length).validate()
//! }
//!
//! fn reject_duplicate_object_entry_names(data: &[u8]) -> Result<(), jtp::Error> {
//!   jtp::from_slice(data).disallow_duplicate_object_entry_name().validate()
//! }
//! ```
//!
//! # Default constraints
//!
//! By default, the validator just checks the JSON syntax without any constraints,
//! and also allows duplicate object entry names.
//!
//! You could set the limit to [`NO_LIMIT`] to disable a specific constraint.
//!
//! # Incremental validation
//!
//! The `Validator` struct is designed to be used incrementally,
//! so you can validate huge JSON payloads in multiple function calls
//! without blocking the current thread for a long time.
//!
//! ```rust
//! use json_threat_protection as jtp;
//!
//! fn validate_incrementally(data: &[u8]) -> Result<(), jtp::Error> {
//!     let mut validator = jtp::from_slice(data);
//!     
//!     // validate the JSON payload in 2000 steps,
//!     // and return `Some(true)` if the validation is finished and no errors.
//!     // return `Some(false)` to continue the validation.
//!     // Otherwise, return `Err` if an error occurred.
//!     while validator.validate_with_steps(2000)? {
//!         // do something else such as processing other tasks
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! This feature is useful when you want to validate a JSON payload in a non-blocking way,
//! the typical use case is used to build FFI bindings to other software
//! that needs to validate JSON payloads in a non-blocking way to avoid blocking the thread.
//!
//! # Error handling
//!
//! This crate has limited place where might panic, most of errors are returned as `Err`.
//! And some unintended bugs might also return as `Err` with explicit error kind
//! to indicate we are running into buggy code.
//!
//! Whatever the error kind is,
//! it always contains the position where the error occurred, such as line, column, and offset,
//! the `offset` is the byte offset from the beginning of the JSON payload.
//!
//! # Special behavior compared to [serde_json](https://crates.io/crates/serde_json)
//!
//! This crate do it best to keep consistent with `serde_json`'s behavior
//! using the [cargo-fuzz](https://crates.io/crates/cargo-fuzz)
//! to process the same JSON payloading with both this crate and `serde_json`
//! and compare the validation results.
//!
//! However, there are some differences between this crate and `serde_json` so far:
//! * This crate allow any precision of numbers,
//!   even if it cannot be represented in Rust's native number types ([`i64`], [`u64`], [`i128`], [`u128`], [`f64`], [`f128`]).
//!   The [serde_json](https://crates.io/crates/serde_json)
//!   without [arbitrary_precision](https://github.com/serde-rs/json/blob/3f1c6de4af28b1f6c5100da323f2bffaf7c2083f/Cargo.toml#L69-L75)
//!   feature enabled will return an error for such numbers.
//!
//! # Performance
//!
//! This crate is designed to be fast and efficient,
//! and has its own benchmark suite under the `benches` directory.
//! You can run the benchmarks with the following command:
//! ```bash
//! JSON_FILE=/path/to/file.json cargo bench --bench memory -- --verbose
//! ```
//!
//! This suite validates the JSON syntax using both this crate and `serde_json`,
//! you could get your own performance number by specifying the `JSON_FILE` to your dataset.
//!
//! # Fuzzing
//!
//! This crate is fuzzed using the [cargo-fuzz](https://crates.io/crates/cargo-fuzz) tool,
//! program is under the `fuzz` directory.
//!
//! The initial seed corpus is from [nlohmann/json_test_data](https://github.com/nlohmann/json_test_data/),
//! and extra corpus follows the [nlohmann/json/blob/develop/tests/fuzzing](https://github.com/nlohmann/json/blob/develop/tests/fuzzing.md).
//!
mod lexer;
pub mod read;
mod validator;

use read::{IoRead, Read, SliceRead, StrRead};

/// Represents no limit for a specific constraint.
pub const NO_LIMIT: usize = std::usize::MAX;
pub use lexer::LexerError;
pub use read::ReadError;
pub use validator::ValidatorError as Error;

/// The JSON validator.
pub struct Validator<R: Read> {
    inner: validator::Validator<R>,
}

impl<R: Read> Validator<R> {
    /// Creates a new `Validator` instance with the given reader without any constraints.
    /// You could prefer to use the [`from_slice`], [`from_str`], or [`from_reader`] functions
    pub fn new(read: R) -> Self {
        Validator {
            inner: validator::Validator::new(
                read, NO_LIMIT, NO_LIMIT, NO_LIMIT, NO_LIMIT, NO_LIMIT, true,
            ),
        }
    }

    /// Sets the maximum depth of the JSON structure.
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        let inner = self.inner.with_max_depth(max_depth);
        self.inner = inner;
        self
    }

    /// Sets the maximum length of strings.
    pub fn with_max_string_length(mut self, max_string_length: usize) -> Self {
        let inner = self.inner.with_max_string_length(max_string_length);
        self.inner = inner;
        self
    }

    /// Sets the maximum number of entries in arrays.
    pub fn with_max_array_entries(mut self, max_array_length: usize) -> Self {
        let inner = self.inner.with_max_array_entries(max_array_length);
        self.inner = inner;
        self
    }

    /// Sets the maximum number of entries in objects.
    pub fn with_max_object_entries(mut self, max_object_length: usize) -> Self {
        let inner = self.inner.with_max_object_entries(max_object_length);
        self.inner = inner;
        self
    }

    /// Sets the maximum length of object entry names.
    pub fn with_max_object_entry_name_length(
        mut self,
        max_object_entry_name_length: usize,
    ) -> Self {
        let inner = self
            .inner
            .with_max_object_entry_name_length(max_object_entry_name_length);
        self.inner = inner;
        self
    }

    /// Allows duplicate object entry names.
    pub fn allow_duplicate_object_entry_name(mut self) -> Self {
        let inner = self.inner.allow_duplicate_object_entry_name();
        self.inner = inner;
        self
    }

    /// Disallows duplicate object entry names.
    pub fn disallow_duplicate_object_entry_name(mut self) -> Self {
        let inner = self.inner.disallow_duplicate_object_entry_name();
        self.inner = inner;
        self
    }

    /// Validates the JSON payload in a single call, and consumes current [`Validator`] instance.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the JSON payload is valid and did not violate any constraints.
    /// * `Err` - If the JSON payload is invalid or violates any constraints.
    ///
    /// # Errors
    ///
    /// * [`Error`] - If the JSON payload is invalid or violates any constraints.
    ///
    /// In the extreme case, the error might return an `Err` and indicate
    /// this crate is running into buggy code.
    /// Please report it to the crate maintainer if you see this error.
    pub fn validate(self) -> Result<(), validator::ValidatorError> {
        self.inner.validate()
    }

    /// Validates the JSON payload in a single call.
    ///
    /// # Arguments
    ///
    /// * `steps` - The number of steps to validate the JSON payload,
    ///             roughly corresponds to the number of tokens processed.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the validation is finished and no errors.
    /// * `Ok(false)` - If the validation is not finished yet, and you should call this function again.
    /// * `Err` - If the JSON payload is invalid or violates any constraints.
    ///
    /// # Errors
    ///
    /// * [`Error`] - If the JSON payload is invalid or violates any constraints.
    ///
    /// In the extreme case, the error might return an `Err` and indicate
    /// this crate is running into buggy code.
    /// Please report it to the crate maintainer if you see this error.
    pub fn validate_with_steps(&mut self, steps: usize) -> Result<bool, validator::ValidatorError> {
        self.inner.validate_with_steps(steps)
    }
}

/// Creates a new `Validator` instance with the given slice of bytes without any constraints.
pub fn from_slice(slice: &[u8]) -> Validator<SliceRead> {
    Validator::new(SliceRead::new(slice))
}

/// Creates a new `Validator` instance with the given `&str` without any constraints.
pub fn from_str(string: &str) -> Validator<StrRead> {
    Validator::new(StrRead::new(string))
}

/// Creates a new `Validator` instance with the given reader without any constraints.
///
/// # Arguments
///
/// * `reader` - The value that implements the [`std::io::Read`] trait.
///
/// # Performance
///
/// Constructing a `Validator` instance with a reader is slower than
/// using [`from_slice`] or [`from_str`] functions.
///
/// And also it is recommended to use the [`std::io::BufReader`] to wrap the reader
/// to improve the performance instead of using the reader directly.
///
/// # Examples
///
/// ```rust
/// use std::io::BufReader;
///
/// fn validate_from_reader<R: std::io::Read>(reader: R) -> Result<(), json_threat_protection::Error> {
///     let buf_reader = BufReader::new(std::fs::File::open("huge.json").unwrap());
///     json_threat_protection::from_reader(buf_reader).validate()
/// }
/// ```
pub fn from_reader<R: std::io::Read>(reader: R) -> Validator<IoRead<R>> {
    Validator::new(IoRead::new(reader))
}
