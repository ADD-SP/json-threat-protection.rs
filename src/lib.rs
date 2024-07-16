mod lexer;
pub mod read;
mod validator;

use read::IoRead;
use read::Read;
use read::SliceRead;
use read::StrRead;

pub const NO_LIMIT: usize = std::usize::MAX;

pub struct Validator<'a, R: Read + 'a> {
    inner: validator::Validator<'a, R>,
}

impl<'a, R: Read + 'a> Validator<'a, R> {
    pub fn new(read: R) -> Self {
        Validator {
            inner: validator::Validator::new(
                read, NO_LIMIT, NO_LIMIT, NO_LIMIT, NO_LIMIT, NO_LIMIT, true,
            ),
        }
    }

    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        let inner = self.inner.with_max_depth(max_depth);
        self.inner = inner;
        self
    }

    pub fn with_max_string_length(mut self, max_string_length: usize) -> Self {
        let inner = self.inner.with_max_string_length(max_string_length);
        self.inner = inner;
        self
    }

    pub fn with_max_array_entries(mut self, max_array_length: usize) -> Self {
        let inner = self.inner.with_max_array_entries(max_array_length);
        self.inner = inner;
        self
    }

    pub fn with_max_object_entries(mut self, max_object_length: usize) -> Self {
        let inner = self.inner.with_max_object_entries(max_object_length);
        self.inner = inner;
        self
    }

    pub fn with_max_object_key_length(mut self, max_object_key_length: usize) -> Self {
        let inner = self.inner.with_max_object_key_length(max_object_key_length);
        self.inner = inner;
        self
    }

    pub fn allow_duplicate_key(mut self) -> Self {
        let inner = self.inner.allow_duplicate_key();
        self.inner = inner;
        self
    }

    pub fn disallow_duplicate_key(mut self) -> Self {
        let inner = self.inner.disallow_duplicate_key();
        self.inner = inner;
        self
    }

    pub fn validate(self) -> Result<(), validator::ValidatorError> {
        self.inner.validate()
    }
}

pub fn from_slice(slice: &[u8]) -> Validator<SliceRead> {
    Validator::new(SliceRead::new(slice))
}

pub fn from_str(string: &str) -> Validator<StrRead> {
    Validator::new(StrRead::new(string))
}

pub fn from_reader<'a, R: std::io::Read + 'a>(reader: R) -> Validator<'a, IoRead<R>> {
    Validator::new(IoRead::new(reader))
}
