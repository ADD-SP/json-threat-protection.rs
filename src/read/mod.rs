//! Defines the [`Read`] trait, and provided implementations for [`std::io::Read`], [`&str`], and slice for [`u8`].

mod io;
mod slice;
mod str;
mod utils;
pub use io::IoRead;
pub use slice::SliceRead;
pub use str::StrRead;
use thiserror::Error;

use utils::{decode_hex_sequence, IS_HEX, NEED_ESCAPE};


// let mut first = first;
//         if first == b'-' {
//             if self.is_eof() {
//                 return Err(ReadError::UnexpectedEndOfInput(self.position()));
//             }
//             first = match self.next_unchecked() {
//                 b'0'..=b'9' => first,
//                 _ => return Err(ReadError::NoNumberCharactersAfterMinusSign(self.position())),
//             }
//         }

macro_rules! parse_number {
    ($self:ident, $first:ident) => {{
        let mut first = $first;
        if first == b'-' {
            first = match $self.next()? {
                n @ Some(b'0'..=b'9') => n.unwrap(),
                Some(_) => return Err(ReadError::NoNumberCharactersAfterMinusSign($self.position())),
                None => return Err(ReadError::UnexpectedEndOfInput($self.position())),
            }
        }

        let second = $self.peek()?;
        if second.is_none() {
            return Ok(());
        }

        if first == b'0' && matches!(second, Some(b'0'..=b'9')) {
            return Err(ReadError::LeadingZerosInNumber($self.position()));
        }

        loop {
            match $self.peek()? {
                Some(b'0'..=b'9') => $self.discard(),
                Some(b'.') => return parse_float!($self),
                Some(b'e') | Some(b'E') => return parse_exponent!($self),
                _ => break,
            }
        }

        Ok(())
    }};
}

macro_rules! parse_float {
    ($self:ident) => {{
        if $self.next()? != Some(b'.') {
            return Err(ReadError::Bug {
                msg: "macro_rules! parse_float: assume the first character is a period".to_string(),
                position: $self.position(),
            });
        }

        match $self.peek()? {
            Some(b'0'..=b'9') => $self.discard(),
            Some(_) => return Err(ReadError::NoNumberCharactersAfterFraction($self.position())),
            None => return Err(ReadError::UnexpectedEndOfInput($self.position())),
        }

        loop {
            match $self.peek()? {
                Some(b'0'..=b'9') => $self.discard(),
                Some(b'e') | Some(b'E') => return parse_exponent!($self),
                _ => break,
            }
        }

        Ok(())
    }};
}

macro_rules! parse_exponent {
    ($self:ident) => {{
        if !matches!($self.next()?, Some(b'e') | Some(b'E')) {
            return Err(ReadError::Bug {
                msg: "macro_rules! parse_exponent: assume the first character is an exponent"
                    .to_string(),
                position: $self.position(),
            });
        }

        match $self.peek()? {
            Some(b'-') | Some(b'+') => $self.discard(),
            Some(b'0'..=b'9') => (),
            Some(_) => return Err(ReadError::NoNumberCharactersAfterExponent($self.position())),
            None => return Err(ReadError::UnexpectedEndOfInput($self.position())),
        }

        match $self.peek()? {
            Some(b'0'..=b'9') => (),
            Some(_) => return Err(ReadError::NoNumberCharactersAfterExponent($self.position())),
            None => return Err(ReadError::UnexpectedEndOfInput($self.position())),
        }

        loop {
            match $self.peek()? {
                Some(b'0'..=b'9') => $self.discard(),
                _ => break,
            }
        }

        Ok(())
    }};
}

macro_rules! next4_hex {
    ($self:ident) => {{
        let mut buf = [0; 4];
        for i in 0..4 {
            let next = $self.next()?;
            if next.is_none() {
                return Err(ReadError::UnexpectedEndOfInput($self.position()));
            }

            // unwrap is safe because we checked if next is None
            let next = next.unwrap();
            if IS_HEX[next as usize] {
                buf[i] = next;
            } else {
                return Err(ReadError::NonHexCharacterInUnicodeEscape($self.position()));
            }
        }
        buf
    }};
}

pub use utils::Position;

#[derive(Debug, Error)]
/// An error that can occur when reading characters.
pub enum ReadError {
    /// Unexpected end of input.
    #[error("unexpected end of input ({0})")]
    UnexpectedEndOfInput(Position),

    /// I/O error.
    #[error("I/O Error ({0})")]
    IoError(std::io::Error, Position),

    /// Non numerical character.
    #[error("non numirical character ({0})")]
    NonNumericalCharacter(Position),

    /// Unclosed string.
    #[error("unclosed string ({0})")]
    UnclosedString(Position),

    /// Invalid escape sequence.
    #[error("invalid escape sequence ({0})")]
    InvalidEscapeSequence(Position),

    /// Control character in string.
    #[error("control character in string ({0})")]
    ControlCharacterInString(Position),

    /// Non hex character in unicode escape sequence.
    #[error("non hex character in unicode escape sequence ({0})")]
    NonHexCharacterInUnicodeEscape(Position),

    /// No number characters after the minus sign.
    #[error("no number characters after the minus sign ({0})")]
    NoNumberCharactersAfterMinusSign(Position),

    /// Leading zeros in number.
    #[error("leading zeros in number ({0})")]
    LeadingZerosInNumber(Position),

    /// No number characters after fraction.
    #[error("no number characters after fraction ({0})")]
    NoNumberCharactersAfterFraction(Position),

    /// No number characters after exponent.
    #[error("no number characters after exponent ({0})")]
    NoNumberCharactersAfterExponent(Position),

    /// Running into unexpected state.
    #[error("running into unexpected state, please report this issue to the maintainer, ({msg}) ({position})")]
    Bug {
        /// Diagnostic message.
        msg: String,

        /// The position where the bug happened.
        position: Position,
    },
}

/// A trait for reading characters from a source.
///
/// # Performance
///
/// All provided methods might not be the most efficient way to read characters
/// as it might return the [`ReadError`], which basically an [`std::io::Error`].
/// However, reading characters from in memory source should raise [`std::io::Error`],
/// so you could implement your own [`Read`] trait and its provided methods
/// for better performance, such as [`SliceRead`].
pub trait Read {
    /// Get the current position of the reader.
    fn position(&self) -> Position;

    /// Peek the next character without consuming it.
    fn peek(&mut self) -> Result<Option<u8>, ReadError>;

    /// Get the next character and consume it.
    fn next(&mut self) -> Result<Option<u8>, ReadError>;

    /// Discard the next character.
    ///
    /// # Panic
    ///
    /// This method panics if the next character is None.
    fn discard(&mut self) {
        self.next().unwrap();
    }

    fn next3(&mut self) -> Result<[u8; 3], ReadError> {
        let mut buf = [0; 3];
        for byte in &mut buf {
            *byte = match self.next()? {
                Some(byte) => byte,
                None => return Err(ReadError::UnexpectedEndOfInput(self.position())),
            };
        }
        Ok(buf)
    }

    fn next4(&mut self) -> Result<[u8; 4], ReadError> {
        let mut buf = [0; 4];
        for byte in &mut buf {
            *byte = match self.next()? {
                Some(byte) => byte,
                None => return Err(ReadError::UnexpectedEndOfInput(self.position())),
            };
        }
        Ok(buf)
    }

    /// Skip whitespace characters (`' '`, `'\t'`, `'\n'`, `'\r'`).
    fn skip_whitespace(&mut self) -> Result<Option<u8>, ReadError> {
        loop {
            match self.next()? {
                Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') => continue,
                ch => return Ok(ch),
            }
        }
    }

    /// Parse a number and allow arbitrary precision.
    fn next_number(&mut self, first: u8) -> Result<(), ReadError> {
        parse_number!(self, first)
    }

    /// Parse a string, but not guaranteed to be correct UTF-8.
    fn next_likely_string(&mut self, buf: &mut Vec<u8>) -> Result<(), ReadError> {
        buf.clear();

        while let Some(byte) = self.next()? {
            if !NEED_ESCAPE[byte as usize] {
                buf.push(byte);
                continue;
            }

            match byte {
                b'"' => return Ok(()),
                b'\\' => {
                    let mut simple_escape = true;

                    match self.next()? {
                        Some(b'"') => buf.push(b'"'),
                        Some(b'\\') => buf.push(b'\\'),
                        Some(b'/') => buf.push(b'/'),
                        Some(b'b') => buf.push(b'\x08'),
                        Some(b'f') => buf.push(b'\x0C'),
                        Some(b'n') => buf.push(b'\n'),
                        Some(b'r') => buf.push(b'\r'),
                        Some(b't') => buf.push(b'\t'),
                        Some(b'u') => simple_escape = false,
                        Some(_) => return Err(ReadError::InvalidEscapeSequence(self.position())),
                        None => return Err(ReadError::UnexpectedEndOfInput(self.position())),
                    };

                    if simple_escape {
                        continue;
                    }

                    let hex = decode_hex_sequence(&next4_hex!(self));
                    let ch = match hex {
                        _n @ 0xDC00..=0xDFFF => {
                            return Err(ReadError::InvalidEscapeSequence(self.position()));
                        }
                        n @ 0xD800..=0xDBFF => {
                            let high = n;
                            if self.next()? != Some(b'\\') {
                                return Err(ReadError::InvalidEscapeSequence(self.position()));
                            }
                            if self.next()? != Some(b'u') {
                                return Err(ReadError::InvalidEscapeSequence(self.position()));
                            }
                            let low = decode_hex_sequence(&next4_hex!(self));
                            if !matches!(low, 0xDC00..=0xDFFF) {
                                return Err(ReadError::InvalidEscapeSequence(self.position()));
                            }

                            let high = ((high & 0x03FF) << 10) as u32;
                            let low = (low & 0x03FF) as u32;
                            let codepoint = 0x10000u32 + high + low;

                            match std::char::from_u32(codepoint) {
                                Some(ch) => ch,
                                None => {
                                    return Err(ReadError::Bug {
                                        msg:
                                            "Read.next_likely_string: assume the codepoint is valid"
                                                .to_string(),
                                        position: self.position(),
                                    })
                                }
                            }
                        }
                        n => match std::char::from_u32(n as u32) {
                            Some(ch) => ch,
                            None => {
                                return Err(ReadError::Bug {
                                    msg: "Read.next_likely_string: assume the codepoint is valid"
                                        .to_string(),
                                    position: self.position(),
                                });
                            }
                        },
                    };

                    buf.extend_from_slice(ch.encode_utf8(&mut [0u8; 4]).as_bytes());
                }
                _ => return Err(ReadError::ControlCharacterInString(self.position())),
            }
        }

        Err(ReadError::UnclosedString(self.position()))
    }
}
