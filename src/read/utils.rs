pub const IS_HEX: [bool; 256] = {
    let mut is_hex = [false; 256];
    is_hex[b'0' as usize] = true;
    is_hex[b'1' as usize] = true;
    is_hex[b'2' as usize] = true;
    is_hex[b'3' as usize] = true;
    is_hex[b'4' as usize] = true;
    is_hex[b'5' as usize] = true;
    is_hex[b'6' as usize] = true;
    is_hex[b'7' as usize] = true;
    is_hex[b'8' as usize] = true;
    is_hex[b'9' as usize] = true;
    is_hex[b'a' as usize] = true;
    is_hex[b'b' as usize] = true;
    is_hex[b'c' as usize] = true;
    is_hex[b'd' as usize] = true;
    is_hex[b'e' as usize] = true;
    is_hex[b'f' as usize] = true;
    is_hex[b'A' as usize] = true;
    is_hex[b'B' as usize] = true;
    is_hex[b'C' as usize] = true;
    is_hex[b'D' as usize] = true;
    is_hex[b'E' as usize] = true;
    is_hex[b'F' as usize] = true;
    is_hex
};

pub const IS_WHITESPACE: [bool; 256] = {
    let mut is_whitespace = [false; 256];
    is_whitespace[b' ' as usize] = true;
    is_whitespace[b'\t' as usize] = true;
    is_whitespace[b'\n' as usize] = true;
    is_whitespace[b'\r' as usize] = true;
    is_whitespace
};

pub const NEED_ESCAPE: [bool; 256] = {
    let mut need_escape = [false; 256];
    // Control characters: 00..=1F
    need_escape[0] = true;
    need_escape[1] = true;
    need_escape[2] = true;
    need_escape[3] = true;
    need_escape[4] = true;
    need_escape[5] = true;
    need_escape[6] = true;
    need_escape[7] = true;
    need_escape[8] = true;
    need_escape[9] = true;
    need_escape[10] = true;
    need_escape[11] = true;
    need_escape[12] = true;
    need_escape[13] = true;
    need_escape[14] = true;
    need_escape[15] = true;
    need_escape[16] = true;
    need_escape[17] = true;
    need_escape[18] = true;
    need_escape[19] = true;
    need_escape[20] = true;
    need_escape[21] = true;
    need_escape[22] = true;
    need_escape[23] = true;
    need_escape[24] = true;
    need_escape[25] = true;
    need_escape[26] = true;
    need_escape[27] = true;
    need_escape[28] = true;
    need_escape[29] = true;
    need_escape[30] = true;
    need_escape[31] = true;
    // " and \
    need_escape[b'"' as usize] = true;
    need_escape[b'\\' as usize] = true;
    need_escape
};

/// Decode a sequence of 4 hex characters into a u16 value.
///
/// # Panics
///
/// This function panics if the input is not a valid hex sequence.
pub fn decode_hex_sequence(hexes: &[u8; 4]) -> u16 {
    let mut value = 0;
    for &hex in hexes {
        value <<= 4;
        value |= match hex {
            b'0'..=b'9' => hex - b'0',
            b'a'..=b'f' => hex - b'a' + 10,
            b'A'..=b'F' => hex - b'A' + 10,
            _ => unreachable!(),
        } as u16;
    }
    value
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Represents a position where is the validator currently at.
pub struct Position {
    /// Line number, starting from `1``
    pub line: usize,

    /// Column number, starting from `0`, `0` mean did read any character from this `line`
    pub column: usize,

    /// Offset from the beginning of the entire input, starting from `0`
    pub offset: usize,
}

impl Default for Position {
    fn default() -> Self {
        Position {
            line: 1,
            column: 0,
            offset: 0,
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "line: {}, column: {}, offset: {}",
            self.line, self.column, self.offset
        )
    }
}

/// An iterator that keeps track of the current line and column number.
pub struct LineColumnIterator<'a, I: Iterator<Item = &'a u8>> {
    /// The underlying iterator.
    iter: I,

    /// The current character.
    ch: Option<&'a u8>,

    /// The current position.
    position: Position,
}

impl<'a, I: Iterator<Item = &'a u8>> LineColumnIterator<'a, I> {
    pub fn new(iter: I) -> Self {
        LineColumnIterator {
            iter,
            ch: None,
            position: Position::default(),
        }
    }

    /// Discard the current character.
    ///
    /// # Panics
    ///
    /// This function panics if there is no character to discard.
    pub fn discard(&mut self) {
        self.next().unwrap();
    }

    pub fn peek(&mut self) -> Option<&u8> {
        if self.ch.is_some() {
            return self.ch;
        }

        self.ch = self.iter.next();
        self.ch
    }

    pub fn position(&self) -> Position {
        self.position
    }
}

impl<'a, I: Iterator<Item = &'a u8>> Iterator for LineColumnIterator<'a, I> {
    type Item = &'a u8;

    fn next(&mut self) -> Option<Self::Item> {
        let ch = match self.ch {
            Some(_) => {
                let ch = self.ch;
                self.ch = None;
                ch
            }
            None => self.iter.next(),
        };

        match ch {
            Some(b'\n') => {
                self.position.line += 1;
                self.position.column = 0;
            }
            Some(_) => {
                self.position.column += 1;
            }
            None => return None,
        }

        self.position.offset += 1;
        ch
    }
}
