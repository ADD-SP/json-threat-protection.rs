use super::utils::{decode_hex_sequence, IS_HEX, IS_WHITESPACE, NEED_ESCAPE};
use super::{Position, Read, ReadError};

/// A reader for slices which implements the [`Read`] trait.
pub struct SliceRead<'a> {
    slice: &'a [u8],
    index: usize,
}

impl<'a> SliceRead<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        SliceRead {
            slice,
            index: 0,
        }
    }

    fn is_eof(&self) -> bool {
        debug_assert!(self.index <= self.slice.len());
        self.index == self.slice.len()
    }

    fn remaining(&self) -> usize {
        self.slice.len() - self.index
    }

    fn discard_unchecked(&mut self) {
        debug_assert!(!self.is_eof());
        self.index += 1;
    }

    fn peek_unchecked(&self) -> u8 {
        debug_assert!(!self.is_eof());
        self.slice[self.index]
    }

    fn next_unchecked(&mut self) -> u8 {
        debug_assert!(!self.is_eof());
        let byte = self.slice[self.index];
        self.index += 1;
        byte
    }

    fn next3_unchecked(&mut self) -> [u8; 3] {
        debug_assert!(self.remaining() >= 3);
        let buf = [
            self.slice[self.index],
            self.slice[self.index + 1],
            self.slice[self.index + 2],
        ];
        self.index += 3;
        buf
    }

    fn next4_unchecked(&mut self) -> [u8; 4] {
        debug_assert!(self.remaining() >= 4);
        let buf = [
            self.slice[self.index],
            self.slice[self.index + 1],
            self.slice[self.index + 2],
            self.slice[self.index + 3],
        ];
        self.index += 4;
        buf
    }

    fn next4_hex_unchecked(&mut self) -> Result<[u8; 4], ReadError> {
        let buf = self.next4_unchecked();
        if !buf.iter().all(|&n| IS_HEX[n as usize]) {
            return Err(ReadError::InvalidEscapeSequence(self.position()));
        }
        Ok(buf)
    }

    fn parse_number(&mut self, first: u8) -> Result<(), ReadError> {
        let mut first = first;
        if first == b'-' {
            if self.is_eof() {
                return Err(ReadError::UnexpectedEndOfInput(self.position()));
            }
            first = match self.next_unchecked() {
                n @ b'0'..=b'9' =>n,
                _ => return Err(ReadError::NoNumberCharactersAfterMinusSign(self.position())),
            }
        }

        if self.is_eof() {
            return Ok(());
        }

        if first == b'0' && self.peek_unchecked().is_ascii_digit() {
            return Err(ReadError::LeadingZerosInNumber(self.position()));
        }

        while !self.is_eof() {
            match self.peek_unchecked() {
                b'0'..=b'9' => self.discard_unchecked(),
                b'.' => return self.parse_float(),
                b'e' | b'E' => return self.parse_exponent(),
                _ => break,
            }
        }

        Ok(())
    }

    fn parse_float(&mut self) -> Result<(), ReadError> {
        match self.next_unchecked() {
            b'.' => (),
            _ => unreachable!(),
        }

        if self.is_eof() {
            return Err(ReadError::UnexpectedEndOfInput(self.position()));
        }

        match self.next_unchecked() {
            b'0'..=b'9' => (),
            _ => return Err(ReadError::NoNumberCharactersAfterFraction(self.position())),
        }

        while !self.is_eof() {
            match self.peek_unchecked() {
                b'0'..=b'9' => self.discard_unchecked(),
                b'e' | b'E' => return self.parse_exponent(),
                _ => break,
            }
        }

        Ok(())
    }

    fn parse_exponent(&mut self) -> Result<(), ReadError> {
        match self.next_unchecked() {
            b'e' | b'E' => (),
            _ => unreachable!(),
        }

        if self.is_eof() {
            return Err(ReadError::UnexpectedEndOfInput(self.position()));
        }

        match self.peek_unchecked() {
            b'-' | b'+' => self.discard_unchecked(),
            _ => (),
        }

        if self.is_eof() {
            return Err(ReadError::UnexpectedEndOfInput(self.position()));
        }

        match self.next_unchecked() {
            b'0'..=b'9' => (),
            _ => return Err(ReadError::NoNumberCharactersAfterExponent(self.position())),
        }

        while !self.is_eof() {
            match self.peek_unchecked() {
                b'0'..=b'9' => self.discard_unchecked(),
                _ => break,
            }
        }

        Ok(())
    }

    fn parse_escape_sequence(&mut self, buf: &mut Vec<u8>) -> Result<(), ReadError> {
        if self.is_eof() {
            return Err(ReadError::UnexpectedEndOfInput(self.position()));
        }

        let mut simple_escape = true;

        match self.next_unchecked() {
            b'"' => buf.push(b'"'),
            b'\\' => buf.push(b'\\'),
            b'/' => buf.push(b'/'),
            b'b' => buf.push(b'\x08'),
            b'f' => buf.push(b'\x0C'),
            b'n' => buf.push(b'\n'),
            b'r' => buf.push(b'\r'),
            b't' => buf.push(b'\t'),
            b'u' => simple_escape = false,
            _ => return Err(ReadError::InvalidEscapeSequence(self.position())),
        };

        if simple_escape {
            return Ok(());
        }

        // check if there are 4 hex characters
        if self.remaining() < 4 {
            return Err(ReadError::UnexpectedEndOfInput(self.position()));
        }

        let hex = decode_hex_sequence(&self.next4_hex_unchecked()?);
        let ch = match hex {
            _n @ 0xDC00..=0xDFFF => {
                return Err(ReadError::InvalidEscapeSequence(self.position()));
            }
            n @ 0xD800..=0xDBFF => {
                let high = n;

                // check if there is \uXXXX sequence
                if self.remaining() < 6 {
                    return Err(ReadError::UnexpectedEndOfInput(self.position()));
                }

                if self.next_unchecked() != b'\\' {
                    return Err(ReadError::InvalidEscapeSequence(self.position()));
                }
                if self.next_unchecked() != b'u' {
                    return Err(ReadError::InvalidEscapeSequence(self.position()));
                }

                let low = decode_hex_sequence(&self.next4_hex_unchecked()?);
                if !matches!(low, 0xDC00..=0xDFFF) {
                    return Err(ReadError::InvalidEscapeSequence(self.position()));
                }

                let high = ((high & 0x03FF) << 10) as u32;
                let low = (low & 0x03FF) as u32;
                let codepoint = 0x10000u32 + high + low;

                match std::char::from_u32(codepoint) {
                    Some(ch) => ch,
                    None => return Err(ReadError::InvalidEscapeSequence(self.position())),
                }
            }
            n => std::char::from_u32(n as u32).unwrap(),
        };

        buf.extend_from_slice(ch.encode_utf8(&mut [0u8; 4]).as_bytes());
        Ok(())
    }
}

impl<'a> Read for SliceRead<'a> {
    fn position(&self) -> Position {
        let mut pos = Position::default();
        for &n in &self.slice[..self.index] {
            if n == b'\n' {
                pos.line += 1;
                pos.column = 0;
            } else {
                pos.column += 1;
            }
            pos.offset += 1;
        }
        pos
    }

    fn peek(&mut self) -> Result<Option<u8>, ReadError> {
        if self.is_eof() {
            return Ok(None);
        }
        Ok(Some(self.peek_unchecked()))
    }

    fn next(&mut self) -> Result<Option<u8>, ReadError> {
        if self.is_eof() {
            return Ok(None);
        }
        Ok(Some(self.next_unchecked()))
    }

    fn next3(&mut self) -> Result<[u8; 3], ReadError> {
        if self.remaining() < 3 {
            return Err(ReadError::UnexpectedEndOfInput(self.position()));
        }
        Ok(self.next3_unchecked())
    }

    fn next4(&mut self) -> Result<[u8; 4], ReadError> {
        if self.remaining() < 4 {
            return Err(ReadError::UnexpectedEndOfInput(self.position()));
        }
        Ok(self.next4_unchecked())
    }

    fn skip_whitespace(&mut self) -> Result<Option<u8>, ReadError> {
        while !self.is_eof() {
            let next = self.next_unchecked();
            if !IS_WHITESPACE[next as usize] {
                return Ok(Some(next));
            }
        }
        Ok(None)
    }

    fn next_number(&mut self, first: u8) -> Result<(), ReadError> {
        self.parse_number(first)
    }

    fn next_likely_string(&mut self, buf: &mut Vec<u8>) -> Result<(), ReadError> {
        buf.clear();

        while !self.is_eof() {
            let byte = self.next_unchecked();

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
