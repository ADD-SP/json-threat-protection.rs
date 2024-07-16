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

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]
    Comma,    // ,
    Colon,    // :
    Number,
    String(String),
    True,
    False,
    Null,
}

/// A JSON lexer, which reads a JSON input and produces a stream of tokens.
pub struct Lexer<R: Read> {
    reader: R,
    byte_buf: Vec<u8>,
    peeked: Option<Token>,
}

impl<R: Read> Lexer<R> {
    pub fn new(reader: R) -> Self {
        Lexer {
            reader,
            byte_buf: Vec::with_capacity(64),
            peeked: None,
        }
    }

    pub fn position(&self) -> Position {
        self.reader.position()
    }

    pub fn peek(&mut self) -> Result<Option<Token>, LexerError> {
        if self.peeked.is_none() {
            self.peeked = self.next()?;
        }
        Ok(self.peeked.clone())
    }

    pub fn next(&mut self) -> Result<Option<Token>, LexerError> {
        if self.peeked.is_some() {
            let peeked = self.peeked.clone();
            self.peeked = None;
            return Ok(peeked);
        }

        loop {
            self.reader.skip_whitespace()?;
            let peek = self.reader.peek()?;
            if peek.is_none() {
                return Ok(None);
            }

            // unwrap is safe because peek is not None
            match peek.unwrap() {
                b'{' => {
                    // unwrap is safe because peek is not None
                    self.reader.next()?.unwrap();
                    return Ok(Some(Token::LBrace));
                }
                b'}' => {
                    // unwrap is safe because peek is not None
                    self.reader.next()?.unwrap();
                    return Ok(Some(Token::RBrace));
                }
                b'[' => {
                    // unwrap is safe because peek is not None
                    self.reader.next()?.unwrap();
                    return Ok(Some(Token::LBracket));
                }
                b']' => {
                    // unwrap is safe because peek is not None
                    self.reader.next()?.unwrap();
                    return Ok(Some(Token::RBracket));
                }
                b',' => {
                    // unwrap is safe because peek is not None
                    self.reader.next()?.unwrap();
                    return Ok(Some(Token::Comma));
                }
                b':' => {
                    // unwrap is safe because peek is not None
                    self.reader.next()?.unwrap();
                    return Ok(Some(Token::Colon));
                }
                b'"' => {
                    return Ok(Some(self.parse_string()?));
                }
                b't' => {
                    return Ok(Some(self.parse_true()?));
                }
                b'f' => {
                    return Ok(Some(self.parse_false()?));
                }
                b'n' => {
                    return Ok(Some(self.parse_null()?));
                }
                b'-' | b'+' | b'0'..=b'9' => {
                    return Ok(Some(self.parse_number()?));
                }
                _ => return Err(LexerError::UnexpectedByte(self.position())),
            }
        }
    }

    fn parse_string(&mut self) -> Result<Token, LexerError> {
        self.byte_buf.clear();
        self.reader.next_likely_string(&mut self.byte_buf)?;

        let str = std::str::from_utf8(&self.byte_buf);
        if str.is_err() {
            return Err(LexerError::InvalidUtf8Sequence(self.position()));
        }

        // unwrap is safe because str is not Err
        Ok(Token::String(str.unwrap().to_string()))
    }

    fn parse_number(&mut self) -> Result<Token, LexerError> {
        match self.reader.next_number() {
            Ok(_) => Ok(Token::Number),
            Err(e) => Err(e.into()),
        }
    }

    fn parse_true(&mut self) -> Result<Token, LexerError> {
        match self.reader.next4()? {
            [b't', b'r', b'u', b'e'] => Ok(Token::True),
            _ => Err(LexerError::UnexpectedByte(self.position())),
        }
    }

    fn parse_false(&mut self) -> Result<Token, LexerError> {
        match self.reader.next5()? {
            [b'f', b'a', b'l', b's', b'e'] => Ok(Token::False),
            _ => Err(LexerError::UnexpectedByte(self.position())),
        }
    }

    fn parse_null(&mut self) -> Result<Token, LexerError> {
        match self.reader.next4()? {
            [b'n', b'u', b'l', b'l'] => Ok(Token::Null),
            _ => Err(LexerError::UnexpectedByte(self.position())),
        }
    }
}
