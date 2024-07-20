use super::utils::{decode_hex_sequence, LineColumnIterator, IS_HEX, IS_WHITESPACE, NEED_ESCAPE};
use super::{Position, Read, ReadError};

/// A reader for slices which implements the [`Read`] trait.
pub struct SliceRead<'a> {
    iter: LineColumnIterator<'a, std::slice::Iter<'a, u8>>,
}

impl<'a> SliceRead<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        SliceRead {
            iter: LineColumnIterator::new(slice.iter()),
        }
    }

    fn discard(&mut self) {
        self.iter.discard();
    }

    /// Fast version of `peek` that does not return an error.
    fn peek_no_error(&mut self) -> Option<u8> {
        self.iter.peek().copied()
    }

    /// Fast version of `next` that does not return an error.
    fn next_no_error(&mut self) -> Option<u8> {
        self.iter.next().copied()
    }

    /// Fast version of `next4` that does not return an error.
    fn next4_no_error(&mut self) -> Option<[u8; 4]> {
        let mut buf = [0; 4];
        for i in 0..4 {
            match self.next_no_error() {
                Some(ch) => buf[i] = ch,
                None => return None,
            }
        }
        Some(buf)
    }

    /// Fast version of `next5` that does not return an error.
    fn next5_no_error(&mut self) -> Option<[u8; 5]> {
        let mut buf = [0; 5];
        for i in 0..5 {
            match self.next_no_error() {
                Some(ch) => buf[i] = ch,
                None => return None,
            }
        }
        Some(buf)
    }

    /// Fast version of `position` that does not return an error.
    fn next4_hex(&mut self) -> Result<[u8; 4], ReadError> {
        let mut buf = [0; 4];
        for i in 0..4 {
            match self.next_no_error() {
                Some(ch) => {
                    if !IS_HEX[ch as usize] {
                        return Err(ReadError::NonHexCharacterInUnicodeEscape(self.position()));
                    }
                    buf[i] = ch;
                }
                None => return Err(ReadError::UnexpectedEndOfInput(self.position())),
            }
        }
        Ok(buf)
    }

    fn parse_number(&mut self) -> Result<(), ReadError> {
        match self.peek_no_error() {
            Some(b'-') => self.discard(),
            Some(b'0'..=b'9') => (),
            Some(_) => return Err(ReadError::Bug {
                msg:
                    "SliceRead.parse_number: assume the first character is a number or a minus sign"
                        .to_string(),
                position: self.position(),
            }),
            None => return Err(ReadError::UnexpectedEndOfInput(self.position())),
        }

        let first = match self.next_no_error() {
            Some(n @ b'0'..=b'9') => n,
            _ => {
                return Err(ReadError::Bug {
                    msg: "SliceRead.parse_number: assume the first character is a number"
                        .to_string(),
                    position: self.position(),
                })
            }
        };

        let second = self.peek_no_error();
        if second.is_none() {
            return Ok(());
        }

        if first == b'0' && matches!(second, Some(b'0'..=b'9')) {
            return Err(ReadError::LeadingZerosInNumber(self.position()));
        }

        loop {
            match self.peek_no_error() {
                Some(b'0'..=b'9') => self.discard(),
                Some(b'.') => return self.parse_float(),
                Some(b'e') | Some(b'E') => return self.parse_exponent(),
                _ => break,
            }
        }

        Ok(())
    }

    fn parse_float(&mut self) -> Result<(), ReadError> {
        if self.next_no_error() != Some(b'.') {
            return Err(ReadError::Bug {
                msg: "SliceRead.parse_float: assume the first character is a period".to_string(),
                position: self.position(),
            });
        }

        match self.peek_no_error() {
            Some(b'0'..=b'9') => self.discard(),
            Some(_) => return Err(ReadError::NoNumberCharactersAfterFraction(self.position())),
            None => return Err(ReadError::UnexpectedEndOfInput(self.position())),
        }

        loop {
            match self.peek_no_error() {
                Some(b'0'..=b'9') => self.discard(),
                Some(b'e') | Some(b'E') => return self.parse_exponent(),
                _ => break,
            }
        }

        Ok(())
    }

    fn parse_exponent(&mut self) -> Result<(), ReadError> {
        if !matches!(self.next_no_error(), Some(b'e') | Some(b'E')) {
            return Err(ReadError::Bug {
                msg: "SliceRead.parse_exponent: assume the first character is an exponent"
                    .to_string(),
                position: self.position(),
            });
        }

        match self.peek_no_error() {
            Some(b'-') | Some(b'+') => self.discard(),
            Some(b'0'..=b'9') => (),
            Some(_) => return Err(ReadError::NoNumberCharactersAfterExponent(self.position())),
            None => return Err(ReadError::UnexpectedEndOfInput(self.position())),
        }

        match self.peek_no_error() {
            Some(b'0'..=b'9') => (),
            Some(_) => return Err(ReadError::NoNumberCharactersAfterExponent(self.position())),
            None => return Err(ReadError::UnexpectedEndOfInput(self.position())),
        }

        loop {
            match self.peek_no_error() {
                Some(b'0'..=b'9') => self.discard(),
                _ => break,
            }
        }

        Ok(())
    }

    fn parse_escape_sequence(&mut self, buf: &mut Vec<u8>) -> Result<(), ReadError> {
        // assume that the previous character is b'\'
        let mut simple_escape = true;

        match self.next_no_error() {
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
            return Ok(());
        }

        let hex = decode_hex_sequence(&self.next4_hex()?);
        let ch = match hex {
            _n @ 0xDC00..=0xDFFF => {
                return Err(ReadError::InvalidEscapeSequence(self.position()));
            }
            n @ 0xD800..=0xDBFF => {
                let high = n;
                if self.next_no_error() != Some(b'\\') {
                    return Err(ReadError::InvalidEscapeSequence(self.position()));
                }
                if self.next_no_error() != Some(b'u') {
                    return Err(ReadError::InvalidEscapeSequence(self.position()));
                }
                let low = decode_hex_sequence(&self.next4_hex()?);
                if !matches!(low, 0xDC00..=0xDFFF) {
                    return Err(ReadError::InvalidEscapeSequence(self.position()));
                }

                let high = ((high & 0x03FF) << 10) as u32;
                let low = (low & 0x03FF) as u32;
                let codepoint = 0x10000u32 + high + low;

                match std::char::from_u32(codepoint) {
                    Some(ch) => ch,
                    None => return Err(ReadError::Bug {
                        msg: "SliceRead.parse_escape_sequence: assume the codepoint is a valid Unicode codepoint".to_string(),
                        position: self.position(),
                    }),
                }
            }
            n => match std::char::from_u32(n as u32) {
                Some(ch) => ch,
                None => return Err(ReadError::Bug {
                    msg: "SliceRead.parse_escape_sequence: assume the codepoint is a valid Unicode codepoint".to_string(),
                    position: self.position(),
                }),
            }
        };

        buf.extend_from_slice(ch.encode_utf8(&mut [0u8; 4]).as_bytes());
        Ok(())
    }
}

impl<'a> Read for SliceRead<'a> {
    fn position(&self) -> Position {
        self.iter.position()
    }

    fn peek(&mut self) -> Result<Option<u8>, ReadError> {
        Ok(self.peek_no_error())
    }

    fn next(&mut self) -> Result<Option<u8>, ReadError> {
        Ok(self.next_no_error())
    }

    fn next4(&mut self) -> Result<[u8; 4], ReadError> {
        self.next4_no_error()
            .ok_or(ReadError::UnexpectedEndOfInput(self.position()))
    }

    fn next5(&mut self) -> Result<[u8; 5], ReadError> {
        self.next5_no_error()
            .ok_or(ReadError::UnexpectedEndOfInput(self.position()))
    }

    fn skip_whitespace(&mut self) -> Result<(), ReadError> {
        loop {
            match self.peek_no_error() {
                Some(byte) => {
                    if IS_WHITESPACE[byte as usize] {
                        self.next_no_error();
                        continue;
                    }
                    break;
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn next_number(&mut self) -> Result<(), ReadError> {
        self.parse_number()
    }

    fn next_likely_string(&mut self, buf: &mut Vec<u8>) -> Result<(), ReadError> {
        if self.next_no_error() != Some(b'"') {
            return Err(ReadError::Bug {
                msg: "SliceRead.next_likely_string: assume the first character is a double quote"
                    .to_string(),
                position: self.position(),
            });
        }

        while let Some(byte) = self.next_no_error() {
            if !NEED_ESCAPE[byte as usize] {
                buf.push(byte);
                continue;
            }

            match byte {
                b'"' => return Ok(()),
                b'\\' => self.parse_escape_sequence(buf)?,
                _ => return Err(ReadError::ControlCharacterInString(self.position())),
            }
        }

        Err(ReadError::UnclosedString(self.position()))
    }
}
