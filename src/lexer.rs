use std::borrow::Cow;

use crate::read::Read;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("unexpected end of input")]
    UnexpectedEndOfInput,
    #[error("unclosed string at byte {0}")]
    UnclosedString(usize),
    #[error("unclosed object at byte {0}")]
    UnclosedObject(usize),
    #[error("unclosed array at byte {0}")]
    UnclosedArray(usize),
    #[error("missing comma at byte {0}")]
    MissingComma(usize),
    #[error("missing colon at byte {0}")]
    MissingColon(usize),
    #[error("missing key at byte {0}")]
    MissingKey(usize),
    #[error("missing value at byte {0}")]
    MissingValue(usize),
    #[error("missing quote at byte {0}")]
    MissingQuote(usize),
    #[error("invalid escape sequence at byte {0}")]
    InvalidEscapeSequence(usize),
    #[error("invalid utf-8 sequence at byte {0}")]
    InvalidUtf8Sequence(usize),
    #[error("Found control character inside string at byte {0}")]
    ControlCharacterInString(usize),
    #[error("invalid number at byte {0}")]
    InvalidNumber(usize),
    #[error("number out of range at byte {0}")]
    NumberOutOfRange(usize),
    #[error("unpexpected byte at byte {0}")]
    UnexpectedByte(usize),
    #[error("I/O Error")]
    ReadError(#[from] crate::read::ReadError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]
    Comma,    // ,
    Colon,    // :
    Number,
    String(Cow<'a, String>),
    True,
    False,
    Null,
}

pub struct Lexer<'a, R: Read + 'a> {
    reader: R,
    consumed: usize,

    str_buf: String,
    bytes_buf: Vec<u8>,

    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, R: Read + 'a> Lexer<'a, R> {
    pub fn new(reader: R) -> Self {
        Lexer {
            reader,
            consumed: 0,

            str_buf: String::with_capacity(64),
            bytes_buf: Vec::with_capacity(1024),

            _phantom: std::marker::PhantomData,
        }
    }

    pub fn next(&mut self) -> Result<Option<Token>, LexerError> {
        loop {
            self.reader.skip_whitespace()?;
            let peek = self.peek_byte();
            if peek.is_none() {
                return Ok(None);
            }

            match peek.unwrap() {
                b'{' => {
                    self.next_byte()?.unwrap();
                    return Ok(Some(Token::LBrace));
                }
                b'}' => {
                    self.next_byte()?.unwrap();
                    return Ok(Some(Token::RBrace));
                }
                b'[' => {
                    self.next_byte()?.unwrap();
                    return Ok(Some(Token::LBracket));
                }
                b']' => {
                    self.next_byte()?.unwrap();
                    return Ok(Some(Token::RBracket));
                }
                b',' => {
                    self.next_byte()?.unwrap();
                    return Ok(Some(Token::Comma));
                }
                b':' => {
                    self.next_byte()?.unwrap();
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
                _ => return Err(LexerError::UnexpectedByte(self.consumed)),
            }
        }
    }

    fn parse_string<'c: 'a>(&mut self) -> Result<Token, LexerError> {
        self.bytes_buf.clear();
        self.reader.next_likely_string(&mut self.bytes_buf)?;

        // let mut escape = false;

        // self.bytes_buf.clear();

        // match self.next_byte()? {
        //     Some(b'"') => self.bytes_buf.push(b'"'),
        //     Some(_) => return Err(LexerError::MissingQuote(self.consumed)),
        //     None => return Err(LexerError::UnexpectedEndOfInput),
        // }

        // loop {
        //     let byte = self
        //         .next_byte()?
        //         .ok_or(LexerError::UnclosedString(self.consumed))?;

        //     if escape {
        //         self.bytes_buf.push(byte);

        //         match byte {
        //             b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => (),
        //             b'u' => {
        //                 for _ in 0..4 {
        //                     let byte = self.next_byte()?.ok_or(LexerError::UnexpectedEndOfInput)?;

        //                     if !matches!(byte, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F') {
        //                         return Err(LexerError::InvalidEscapeSequence(self.consumed));
        //                     }

        //                     self.bytes_buf.push(byte);
        //                 }
        //             }
        //             _ => return Err(LexerError::InvalidEscapeSequence(self.consumed)),
        //         }

        //         escape = false;
        //     } else {
        //         match byte {
        //             b'"' => {
        //                 self.bytes_buf.push(b'"');
        //                 break;
        //             }
        //             _ => {
        //                 self.bytes_buf.push(byte);

        //                 if byte == b'\\' {
        //                     escape = true;
        //                 }
        //             }
        //         }
        //     }
        // }

        if let Ok(str) = serde_json::from_slice::<Cow<'c, String>>(&self.bytes_buf) {
            return Ok(Token::String(str));
        }

        Err(LexerError::InvalidUtf8Sequence(self.consumed))
    }

    fn parse_number(&mut self) -> Result<Token, LexerError> {
        self.str_buf.clear();
        self.reader.next_numerical(&mut self.str_buf)?;

        if serde_json::from_str::<serde_json::Number>(&self.str_buf).is_err() {
            return Err(LexerError::InvalidNumber(self.consumed));
        }

        Ok(Token::Number)
    }

    fn parse_true(&mut self) -> Result<Token, LexerError> {
        match self.reader.next4()? {
            [b't', b'r', b'u', b'e'] => Ok(Token::True),
            _ => Err(LexerError::UnexpectedByte(self.consumed)),
        }
    }

    fn parse_false(&mut self) -> Result<Token, LexerError> {
        match self.reader.next5()? {
            [b'f', b'a', b'l', b's', b'e'] => Ok(Token::False),
            _ => Err(LexerError::UnexpectedByte(self.consumed)),
        }
    }

    fn parse_null(&mut self) -> Result<Token, LexerError> {
        match self.reader.next4()? {
            [b'n', b'u', b'l', b'l'] => Ok(Token::Null),
            _ => Err(LexerError::UnexpectedByte(self.consumed)),
        }
    }

    fn next_byte(&mut self) -> Result<Option<u8>, LexerError> {
        match self.reader.next() {
            Ok(Some(byte)) => {
                self.consumed += 1;
                Ok(Some(byte))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(LexerError::ReadError(e)),
        }
    }

    fn peek_byte(&mut self) -> Option<u8> {
        match self.reader.peek() {
            Ok(Some(byte)) => Some(byte),
            Ok(None) => None,
            Err(_) => None,
        }
    }

    pub fn get_consumed(&self) -> usize {
        self.consumed
    }
}
