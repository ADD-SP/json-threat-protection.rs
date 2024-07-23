use super::slice::SliceRead;
use super::{Position, Read, ReadError};

/// A reader for strings which implements the [`Read`] trait.
pub struct StrRead<'a> {
    /// The delegate reader.
    slice_read: SliceRead<'a>,
}

impl<'a> StrRead<'a> {
    pub fn new(string: &'a str) -> Self {
        StrRead {
            slice_read: SliceRead::new(string.as_bytes()),
        }
    }
}

impl Read for StrRead<'_> {
    fn position(&self) -> Position {
        self.slice_read.position()
    }

    fn peek(&mut self) -> Result<Option<u8>, ReadError> {
        self.slice_read.peek()
    }

    fn next(&mut self) -> Result<Option<u8>, ReadError> {
        self.slice_read.next()
    }

    fn next4(&mut self) -> Result<[u8; 4], ReadError> {
        self.slice_read.next4()
    }

    fn skip_whitespace(&mut self) -> Result<Option<u8>, ReadError> {
        self.slice_read.skip_whitespace()
    }

    fn next_number(&mut self, first: u8) -> Result<(), ReadError> {
        self.slice_read.next_number(first)
    }

    fn next_likely_string(&mut self, buf: &mut Vec<u8>) -> Result<(), ReadError> {
        self.slice_read.next_likely_string(buf)
    }
}
