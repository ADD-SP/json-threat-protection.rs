use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("unexpected end of input")]
    UnexpectedEndOfInput,
    #[error("I/O Error")]
    IoError(std::io::Error),
    #[error("non numirical character")]
    NonNumericalCharacter,
    #[error("unclosed string")]
    UnclosedString,
    #[error("invalid escape sequence")]
    InvalidEscapeSequence,
}

pub trait Read {
    fn peek(&mut self) -> Result<Option<u8>, ReadError>;
    fn next(&mut self) -> Result<Option<u8>, ReadError>;
    fn next4(&mut self) -> Result<[u8; 4], ReadError> {
        let mut buf = [0; 4];
        for i in 0..4 {
            match self.next()? {
                Some(ch) => buf[i] = ch,
                None => return Err(ReadError::UnexpectedEndOfInput),
            }
        }
        Ok(buf)
    }
    fn next5(&mut self) -> Result<[u8; 5], ReadError> {
        let mut buf = [0; 5];
        for i in 0..5 {
            match self.next()? {
                Some(ch) => buf[i] = ch,
                None => return Err(ReadError::UnexpectedEndOfInput),
            }
        }
        Ok(buf)
    }
    fn skip_whitespace(&mut self) -> Result<(), ReadError> {
        loop {
            match self.peek()? {
                Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') => {
                    self.next()?;
                }
                _ => break,
            }
        }
        Ok(())
    }
    fn next_numerical(&mut self, buf: &mut String) -> Result<(), ReadError> {
        let mut got = false;

        loop {
            match self.peek()? {
                Some(b'0'..=b'9' | b'.' | b'e' | b'E' | b'-' | b'+') => {
                    buf.push(self.next()?.unwrap() as char);
                    got = true;
                }
                Some(_) => break,
                None => return Ok(()),
            }
        }

        got.then(|| ()).ok_or(ReadError::NonNumericalCharacter)
    }
    fn next_likely_string(&mut self, buf: &mut Vec<u8>) -> Result<(), ReadError> {
        let mut escape = false;

        match self.next()? {
            Some(b'"') => buf.push(b'"'),
            Some(_) => return Ok(()),
            None => return Ok(()),
        }

        loop {
            let byte = self.next()?.ok_or(ReadError::UnclosedString)?;
            buf.push(byte);

            if escape {
                match byte {
                    b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => (),
                    b'u' => {
                        for _ in 0..4 {
                            let byte = self.next()?.ok_or(ReadError::InvalidEscapeSequence)?;

                            if !matches!(byte, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F') {
                                return Err(ReadError::InvalidEscapeSequence);
                            }

                            buf.push(byte);
                        }
                    }
                    _ => return Err(ReadError::InvalidEscapeSequence),
                }

                escape = false;
            } else {
                match byte {
                    b'"' => break,
                    _ => {
                        if byte == b'\\' {
                            escape = true;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct SliceRead<'a> {
    slice: &'a [u8],
    pos: usize,
}

impl<'a> SliceRead<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        SliceRead { slice, pos: 0 }
    }
}

impl Read for SliceRead<'_> {
    fn peek(&mut self) -> Result<Option<u8>, ReadError> {
        if self.pos < self.slice.len() {
            Ok(Some(self.slice[self.pos]))
        } else {
            Ok(None)
        }
    }

    fn next(&mut self) -> Result<Option<u8>, ReadError> {
        if self.pos < self.slice.len() {
            let ch = self.slice[self.pos];
            self.pos += 1;
            Ok(Some(ch))
        } else {
            Ok(None)
        }
    }

    fn next4(&mut self) -> Result<[u8; 4], ReadError> {
        if self.pos + 4 <= self.slice.len() {
            let mut buf = [0; 4];
            buf.copy_from_slice(&self.slice[self.pos..self.pos + 4]);
            self.pos += 4;
            Ok(buf)
        } else {
            Err(ReadError::UnexpectedEndOfInput)
        }
    }

    fn next5(&mut self) -> Result<[u8; 5], ReadError> {
        if self.pos + 5 <= self.slice.len() {
            let mut buf = [0; 5];
            buf.copy_from_slice(&self.slice[self.pos..self.pos + 5]);
            self.pos += 5;
            Ok(buf)
        } else {
            Err(ReadError::UnexpectedEndOfInput)
        }
    }

    fn skip_whitespace(&mut self) -> Result<(), ReadError> {
        while self.pos < self.slice.len() {
            match self.slice[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
        Ok(())
    }

    fn next_numerical(&mut self, buf: &mut String) -> Result<(), ReadError> {
        let mut got = false;

        while self.pos < self.slice.len() {
            match self.slice[self.pos] {
                b'0'..=b'9' | b'.' | b'e' | b'E' | b'-' | b'+' => {
                    buf.push(self.slice[self.pos] as char);
                    self.pos += 1;
                    got = true;
                }
                _ => break,
            }
        }

        got.then(|| ()).ok_or(ReadError::NonNumericalCharacter)
    }

    fn next_likely_string(&mut self, buf: &mut Vec<u8>) -> Result<(), ReadError> {
        let start_pos = self.pos;

        if self.pos < self.slice.len() && self.slice[self.pos] == b'"' {
            self.pos += 1;
        } else {
            return Err(ReadError::UnclosedString);
        }

        let mut escape = false;
        let mut closed = false;
        while self.pos < self.slice.len() {
            let byte = self.slice[self.pos];
            self.pos += 1;

            if escape {
                match byte {
                    b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => (),
                    b'u' => {
                        if self.pos + 4 > self.slice.len() {
                            return Err(ReadError::InvalidEscapeSequence);
                        }

                        for j in 0..4 {
                            if !matches!(self.slice[self.pos + j], b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')
                            {
                                return Err(ReadError::InvalidEscapeSequence);
                            }
                        }

                        self.pos += 4;
                    }
                    _ => return Err(ReadError::InvalidEscapeSequence),
                }

                escape = false;
            } else {
                match byte {
                    b'"' => {
                        closed = true;
                        break;
                    }
                    _ => {
                        if byte == b'\\' {
                            escape = true;
                        }
                    }
                }
            }
        }

        if closed {
            buf.extend_from_slice(&self.slice[start_pos..self.pos]);
            return Ok(());
        }

        Err(ReadError::UnclosedString)
    }
}

pub struct StrRead<'a> {
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
    fn peek(&mut self) -> Result<Option<u8>, ReadError> {
        self.slice_read.peek()
    }

    fn next(&mut self) -> Result<Option<u8>, ReadError> {
        self.slice_read.next()
    }

    fn next4(&mut self) -> Result<[u8; 4], ReadError> {
        self.slice_read.next4()
    }

    fn next5(&mut self) -> Result<[u8; 5], ReadError> {
        self.slice_read.next5()
    }

    fn skip_whitespace(&mut self) -> Result<(), ReadError> {
        self.slice_read.skip_whitespace()
    }

    fn next_numerical(&mut self, buf: &mut String) -> Result<(), ReadError> {
        self.slice_read.next_numerical(buf)
    }

    fn next_likely_string(&mut self, buf: &mut Vec<u8>) -> Result<(), ReadError> {
        self.slice_read.next_likely_string(buf)
    }
}

pub struct IoRead<R: std::io::Read> {
    reader: R,
    buf: [u8; 1024],
    buf_pos: usize,
    buf_len: usize,
}

impl<R: std::io::Read> IoRead<R> {
    pub fn new(reader: R) -> Self {
        IoRead {
            reader,
            buf: [0; 1024],
            buf_pos: 0,
            buf_len: 0,
        }
    }

    fn need_fill_buf(&self) -> bool {
        assert!(self.buf_pos <= self.buf_len, "buffer read overflow");
        self.buf_pos == self.buf_len
    }

    fn is_buf_empty(&self) -> bool {
        self.buf_pos == self.buf_len
    }

    fn buf_remaining(&self) -> usize {
        self.buf_len - self.buf_pos
    }

    fn fill_buf(&mut self) -> Result<(), ReadError> {
        if self.need_fill_buf() {
            self.buf_len = self
                .reader
                .read(&mut self.buf)
                .map_err(ReadError::IoError)?;
            self.buf_pos = 0;
        }
        Ok(())
    }
}

impl<R: std::io::Read> Read for IoRead<R> {
    fn peek(&mut self) -> Result<Option<u8>, ReadError> {
        if self.need_fill_buf() {
            self.fill_buf()?;
        }

        if self.buf_pos < self.buf_len {
            Ok(Some(self.buf[self.buf_pos]))
        } else {
            Ok(None)
        }
    }

    fn next(&mut self) -> Result<Option<u8>, ReadError> {
        if self.need_fill_buf() {
            self.fill_buf()?;
        }

        if self.buf_pos < self.buf_len {
            let ch = self.buf[self.buf_pos];
            self.buf_pos += 1;
            Ok(Some(ch))
        } else {
            Ok(None)
        }
    }

    fn next4(&mut self) -> Result<[u8; 4], ReadError> {
        let mut buf = [0; 4];
        let mut got = 0;

        loop {
            if self.need_fill_buf() {
                self.fill_buf()?;
            }

            if self.is_buf_empty() {
                return Err(ReadError::UnexpectedEndOfInput);
            }

            if self.buf_remaining() >= 4 - got {
                let n = 4 - got;
                buf[got..got + n].copy_from_slice(&self.buf[self.buf_pos..self.buf_pos + n]);
                self.buf_pos += n;
                return Ok(buf);
            }

            while self.buf_pos < self.buf_len {
                buf[got] = self.buf[self.buf_pos];
                self.buf_pos += 1;
                got += 1;
            }
        }
    }

    fn next5(&mut self) -> Result<[u8; 5], ReadError> {
        let mut buf = [0; 5];
        let mut got = 0;

        loop {
            if self.need_fill_buf() {
                self.fill_buf()?;
            }

            if self.is_buf_empty() {
                return Err(ReadError::UnexpectedEndOfInput);
            }

            if self.buf_remaining() >= 5 - got {
                let n = 5 - got;
                buf[got..got + n].copy_from_slice(&self.buf[self.buf_pos..self.buf_pos + n]);
                self.buf_pos += n;
                return Ok(buf);
            }

            while self.buf_pos < self.buf_len {
                buf[got] = self.buf[self.buf_pos];
                self.buf_pos += 1;
                got += 1;
            }
        }
    }

    fn skip_whitespace(&mut self) -> Result<(), ReadError> {
        let mut found_non_whitespace = false;

        while !found_non_whitespace {
            if self.need_fill_buf() {
                self.fill_buf()?;
            }

            if self.is_buf_empty() {
                break;
            }

            while self.buf_pos < self.buf_len {
                match self.buf[self.buf_pos] {
                    b' ' | b'\t' | b'\n' | b'\r' => {
                        self.buf_pos += 1;
                    }
                    _ => {
                        found_non_whitespace = true;
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn next_numerical(&mut self, buf: &mut String) -> Result<(), ReadError> {
        let mut got = false;

        loop {
            if self.need_fill_buf() {
                self.fill_buf()?;
            }

            if self.is_buf_empty() {
                return got.then(|| ()).ok_or(ReadError::NonNumericalCharacter);
            }

            while self.buf_pos < self.buf_len {
                match self.buf[self.buf_pos] {
                    b'0'..=b'9' | b'.' | b'e' | b'E' | b'-' | b'+' => {
                        buf.push(self.buf[self.buf_pos] as char);
                        self.buf_pos += 1;
                        got = true;
                    }
                    _ => {
                        if !got {
                            return Err(ReadError::NonNumericalCharacter);
                        }

                        return Ok(());
                    }
                }
            }
        }
    }

    fn next_likely_string(&mut self, buf: &mut Vec<u8>) -> Result<(), ReadError> {
        if self.need_fill_buf() {
            self.fill_buf()?;
        }

        if self.is_buf_empty() {
            return Err(ReadError::UnexpectedEndOfInput);
        }

        if self.buf[self.buf_pos] != b'"' {
            return Err(ReadError::UnclosedString);
        }

        buf.push(b'"');
        self.buf_pos += 1;

        
        let mut closed = false;

        loop {
            if self.need_fill_buf() {
                self.fill_buf()?;
            }

            if self.is_buf_empty() {
                break;
            }

            assert!(!closed, "string already closed");
            
            let mut escape = false;
            let start_pos = self.buf_pos;

            while self.buf_pos < self.buf_len {
                match self.buf[self.buf_pos] {
                    b'"' => {
                        closed = true;
                        break;
                    }
                    b'\\' => {
                        escape = true;
                        break;
                    }
                    _ => (),
                }
                self.buf_pos += 1;
            }

            // self.buf_pos -> b'""
            // self.buf_pos -> b'\\'
            // self.buf_pos -> self.buf_len

            buf.extend_from_slice(&self.buf[start_pos..self.buf_pos]);

            if self.is_buf_empty() {
                assert!(!closed && !escape && self.buf_pos == self.buf_len, "invalid escape sequence");
                continue;
            }

            if closed {
                assert!(!escape, "invalid escape sequence");
                assert!(self.buf[self.buf_pos] == b'"', "invalid string closing");
                self.buf_pos += 1;
                buf.push(b'"');
                return Ok(());
            }
           
            assert!(escape && !closed, "invalid escape sequence");
            assert!(self.buf[self.buf_pos] == b'\\', "invalid escape sequence");
            self.buf_pos += 1;
            buf.push(b'\\');

            let byte = self.next()?.ok_or(ReadError::UnclosedString)?;
            buf.push(byte);
            match byte {
                b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => (),
                b'u' => {
                    let hexes = self.next4()?;
                    for &hex in &hexes {
                        if !matches!(hex, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F') {
                            return Err(ReadError::InvalidEscapeSequence);
                        }
                    }
                    buf.extend_from_slice(&hexes);
                }
                _ => return Err(ReadError::InvalidEscapeSequence),
            }
        }
        closed.then(|| ()).ok_or(ReadError::UnclosedString)
    }
}
