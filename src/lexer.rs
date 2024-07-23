use crate::read::{Position, Read};
use thiserror::Error;

#[derive(Error, Debug)]
/// An error that occurred while lexing a JSON input.
pub enum LexerError {
    /// Invalid UTF-8 sequence at the given position.
    #[error("invalid utf-8 sequence ({0}")]
    InvalidUtf8Sequence(Position),

    /// Unexpected byte at the given position.
    #[error("unpexpected byte at byte ({0})")]
    UnexpectedByte(Position),

    /// Error occurred while reading from the underlying reader.
    #[error("I/O Error")]
    ReadError(#[from] crate::read::ReadError),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Token {
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]
    Comma,    // ,
    Colon,    // :
    Number,
    String,
    True,
    False,
    Null,
}

/// A JSON lexer, which reads a JSON input and produces a stream of tokens.
pub struct Lexer<R: Read> {
    reader: R,
    peeked_str_buf: Vec<u8>,
    peeked: Option<Token>,
}

impl<R: Read> Lexer<R> {
    pub fn new(reader: R) -> Self {
        Lexer {
            reader,
            peeked_str_buf: Vec::with_capacity(64),
            peeked: None,
        }
    }

    pub fn position(&self) -> Position {
        self.reader.position()
    }

    pub fn peek(&mut self, str_buf: &mut Vec<u8>) -> Result<Option<Token>, LexerError> {
        if self.peeked.is_none() {
            self.peeked = self.next(str_buf)?;
            self.peeked_str_buf.clear();
            self.peeked_str_buf.extend_from_slice(str_buf);
        }
        Ok(self.peeked)
    }

    pub fn next(&mut self, str_buf: &mut Vec<u8>) -> Result<Option<Token>, LexerError> {
        if self.peeked.is_some() {
            let peeked = self.peeked;
            self.peeked = None;

            if matches!(peeked, Some(Token::String)) {
                str_buf.clear();
                str_buf.extend_from_slice(&self.peeked_str_buf);
            }

            return Ok(peeked);
        }

        let next = self.reader.skip_whitespace()?;
        if next.is_none() {
            return Ok(None);
        }

        // unwrap is safe because peek is not None
        match next.unwrap() {
            b'{' => {
                // unwrap is safe because peek is not None
                // self.reader.next()?.unwrap();
                Ok(Some(Token::LBrace))
            }
            b'}' => {
                // unwrap is safe because peek is not None
                // self.reader.next()?.unwrap();
                Ok(Some(Token::RBrace))
            }
            b'[' => {
                // unwrap is safe because peek is not None
                // self.reader.next()?.unwrap();
                Ok(Some(Token::LBracket))
            }
            b']' => {
                // unwrap is safe because peek is not None
                // self.reader.next()?.unwrap();
                Ok(Some(Token::RBracket))
            }
            b',' => {
                // unwrap is safe because peek is not None
                // self.reader.next()?.unwrap();
                Ok(Some(Token::Comma))
            }
            b':' => {
                // unwrap is safe because peek is not None
                // self.reader.next()?.unwrap();
                Ok(Some(Token::Colon))
            }
            b'"' => Ok(Some(self.parse_string(str_buf)?)),
            b't' => Ok(Some(self.parse_true()?)),
            b'f' => Ok(Some(self.parse_false()?)),
            b'n' => Ok(Some(self.parse_null()?)),
            digit @( b'-' | b'0'..=b'9') => Ok(Some(self.parse_number(digit)?)),
            _ => Err(LexerError::UnexpectedByte(self.position())),
        }
    }

    fn parse_string(&mut self, str_buf: &mut Vec<u8>) -> Result<Token, LexerError> {
        self.reader.next_likely_string(str_buf)?;

        let str = std::str::from_utf8(str_buf);
        if str.is_err() {
            return Err(LexerError::InvalidUtf8Sequence(self.position()));
        }

        Ok(Token::String)
    }

    fn parse_number(&mut self, first: u8) -> Result<Token, LexerError> {
        match self.reader.next_number(first) {
            Ok(_) => Ok(Token::Number),
            Err(e) => Err(e.into()),
        }
    }

    fn parse_true(&mut self) -> Result<Token, LexerError> {
        match self.reader.next3()? {
            [b'r', b'u', b'e'] => Ok(Token::True),
            _ => Err(LexerError::UnexpectedByte(self.position())),
        }
    }

    fn parse_false(&mut self) -> Result<Token, LexerError> {
        match self.reader.next4()? {
            [b'a', b'l', b's', b'e'] => Ok(Token::False),
            _ => Err(LexerError::UnexpectedByte(self.position())),
        }
    }

    fn parse_null(&mut self) -> Result<Token, LexerError> {
        match self.reader.next3()? {
            [b'u', b'l', b'l'] => Ok(Token::Null),
            _ => Err(LexerError::UnexpectedByte(self.position())),
        }
    }
}
