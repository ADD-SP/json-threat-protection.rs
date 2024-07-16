use thiserror::Error;

use crate::lexer::Lexer;
use crate::lexer::LexerError;
use crate::lexer::Token;
use crate::read::Read;
use std::collections::HashSet;

#[allow(unused_macros)]
macro_rules! allow_duplicate_key {
    ($self:ident) => {
        $self.allow_duplicate_key
    };
}

macro_rules! disallow_duplicate_key {
    ($self:ident) => {{
        !$self.allow_duplicate_key
    }};
}

macro_rules! try_inc_depth {
    ($self:ident) => {
        $self.cur_depth += 1;

        if $self.cur_depth > $self.max_depth {
            return Err(ValidatorError::MaxDepthExceeded);
        }
    };
}

macro_rules! try_dec_depth {
    ($self:ident, $consumed:ident) => {
        if $self.cur_depth == 0 {
            return Err(ValidatorError::InvalidJSON($consumed));
        }

        $self.cur_depth -= 1;

        if $self.cur_depth > $self.max_depth {
            return Err(ValidatorError::MaxDepthExceeded);
        }
    };
}

macro_rules! try_active_array {
    ($self:ident) => {
        $self.entires.push(0);
        try_inc_depth!($self);
    };
}

macro_rules! try_add_array_entry {
    ($self:ident, $consumed:ident) => {
        let entries = $self
            .entires
            .last_mut()
            .ok_or(ValidatorError::InvalidJSON($consumed))?;
        *entries += 1;

        if *entries > $self.max_array_entries {
            return Err(ValidatorError::MaxArrayEntriesExceeded);
        }
    };
}

macro_rules! try_finalize_array {
    ($self:ident, $consumed:ident) => {{
        let entries = $self
            .entires
            .pop()
            .ok_or(ValidatorError::InvalidJSON($consumed))?;
        try_dec_depth!($self, $consumed);
        entries
    }};
}

macro_rules! try_active_object {
    ($self:ident) => {
        if disallow_duplicate_key!($self) {
            $self.keys.push(HashSet::with_capacity(8));
        }
        $self.entires.push(0);
        try_inc_depth!($self);
    };
}

macro_rules! try_add_object_key {
    ($self:ident, $key:ident, $consumed:ident) => {
        if $key.len() > $self.max_object_key_length {
            return Err(ValidatorError::MaxObjectKeyLengthExceeded);
        }

        if disallow_duplicate_key!($self) {
            let keys = $self
                .keys
                .last_mut()
                .ok_or(ValidatorError::InvalidJSON($consumed))?;
            if !keys.insert($key.into_owned()) {
                return Err(ValidatorError::DuplicateObjectKey);
            }
        }
    };
}

macro_rules! try_add_object_value {
    ($self:ident, $consumed:ident) => {
        let entries = $self
            .entires
            .last_mut()
            .ok_or(ValidatorError::InvalidJSON($consumed))?;
        *entries += 1;

        if *entries > $self.max_object_entries {
            return Err(ValidatorError::MaxObjectEntriesExceeded);
        }
    };
}

macro_rules! try_finalize_object {
    ($self:ident, $consumed:ident) => {{
        if disallow_duplicate_key!($self) {
            $self
                .keys
                .pop()
                .ok_or(ValidatorError::InvalidJSON($consumed))?;
        }
        let entries = $self
            .entires
            .pop()
            .ok_or(ValidatorError::InvalidJSON($consumed))?;
        try_dec_depth!($self, $consumed);
        entries
    }};
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    ProcessingObject,
    ProcessingArray,
    RequireElement,
    RequireColon,
    RequireObjectKey,
    OptionalComma,
    OptionalObjectKey,
    OptionalElement,
}

#[derive(Error, Debug)]
pub enum ValidatorError {
    #[error("Lexer error: {0}")]
    LexerError(#[from] LexerError),
    #[error("Invalid JSON at {0}th character")]
    InvalidJSON(usize),
    #[error("Trailing data")]
    TrailingData,
    #[error("Maximum depth exceeded")]
    MaxDepthExceeded,
    #[error("Maximum string length exceeded")]
    MaxStringLengthExceeded,
    #[error("Maximum array entries exceeded")]
    MaxArrayEntriesExceeded,
    #[error("Maximum object entries exceeded")]
    MaxObjectEntriesExceeded,
    #[error("Maximum object key length exceeded")]
    MaxObjectKeyLengthExceeded,
    #[error("Duplicate object key")]
    DuplicateObjectKey,
}

pub struct Validator<'a, R: Read + 'a> {
    lexer: Lexer<'a, R>,
    states: Vec<State>,
    entires: Vec<usize>,
    keys: Vec<HashSet<String>>,
    cur_depth: usize,

    max_depth: usize,
    max_string_length: usize,
    max_array_entries: usize,
    max_object_entries: usize,
    max_object_key_length: usize,
    allow_duplicate_key: bool,
}

impl<'a, R: Read + 'a> Validator<'a, R> {
    pub fn new(
        read: R,
        max_depth: usize,
        max_string_length: usize,
        max_array_entries: usize,
        max_object_entries: usize,
        max_object_key_length: usize,
        allow_duplicate_key: bool,
    ) -> Self {
        Validator {
            lexer: Lexer::new(read),
            states: Vec::with_capacity(32),
            entires: Vec::with_capacity(32),
            keys: Vec::with_capacity(32),
            cur_depth: 0,

            max_depth,
            max_string_length,
            max_array_entries,
            max_object_entries,
            max_object_key_length,
            allow_duplicate_key,
        }
    }

    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub fn with_max_string_length(mut self, max_string_length: usize) -> Self {
        self.max_string_length = max_string_length;
        self
    }

    pub fn with_max_array_entries(mut self, max_array_entries: usize) -> Self {
        self.max_array_entries = max_array_entries;
        self
    }

    pub fn with_max_object_entries(mut self, max_object_entries: usize) -> Self {
        self.max_object_entries = max_object_entries;
        self
    }

    pub fn with_max_object_key_length(mut self, max_object_key_length: usize) -> Self {
        self.max_object_key_length = max_object_key_length;
        self
    }

    pub fn allow_duplicate_key(mut self) -> Self {
        self.allow_duplicate_key = true;
        self
    }

    pub fn disallow_duplicate_key(mut self) -> Self {
        self.allow_duplicate_key = false;
        self
    }

    pub fn validate(mut self) -> Result<(), ValidatorError> {
        self.states.push(State::RequireElement);

        while let Some(state) = self.states.pop() {
            let consumed = self.lexer.get_consumed();
            let token = self.lexer.next()?;

            if token.is_none() {
                return Err(ValidatorError::InvalidJSON(consumed));
            }

            match token.unwrap() {
                Token::LBrace => match state {
                    State::RequireElement | State::OptionalElement => {
                        self.states.push(State::ProcessingObject);
                        self.states.push(State::OptionalObjectKey);
                        try_active_object!(self);
                    }
                    _ => return Err(ValidatorError::InvalidJSON(consumed)),
                },
                Token::RBrace => {
                    match state {
                        State::OptionalComma | State::OptionalObjectKey => {
                            let state = self
                                .states
                                .pop()
                                .ok_or(ValidatorError::InvalidJSON(consumed))?;
                            if state != State::ProcessingObject {
                                return Err(ValidatorError::InvalidJSON(consumed));
                            }

                            let entires = try_finalize_object!(self, consumed);
                            if state == State::OptionalObjectKey && entires != 0 {
                                return Err(ValidatorError::InvalidJSON(consumed));
                            }
                        }
                        State::ProcessingObject => {
                            if try_finalize_object!(self, consumed) == 0 {
                                return Err(ValidatorError::InvalidJSON(consumed));
                            }
                        }
                        _ => return Err(ValidatorError::InvalidJSON(consumed)),
                    }

                    if matches!(
                        self.states.last(),
                        Some(State::ProcessingArray | State::ProcessingObject)
                    ) {
                        self.states.push(State::OptionalComma);
                    }
                }
                Token::LBracket => match state {
                    State::RequireElement | State::OptionalElement => {
                        self.states.push(State::ProcessingArray);
                        self.states.push(State::OptionalElement);
                        try_active_array!(self);
                    }
                    _ => return Err(ValidatorError::InvalidJSON(consumed)),
                },
                Token::RBracket => {
                    match state {
                        State::OptionalComma | State::OptionalElement => {
                            let state = self
                                .states
                                .pop()
                                .ok_or(ValidatorError::InvalidJSON(consumed))?;
                            if state != State::ProcessingArray {
                                return Err(ValidatorError::InvalidJSON(consumed));
                            }

                            let entries = try_finalize_array!(self, consumed);
                            if state == State::OptionalElement && entries != 0 {
                                return Err(ValidatorError::InvalidJSON(consumed));
                            }
                        }
                        State::ProcessingArray => {
                            if try_finalize_array!(self, consumed) == 0 {
                                return Err(ValidatorError::InvalidJSON(consumed));
                            }
                        }
                        _ => return Err(ValidatorError::InvalidJSON(consumed)),
                    }

                    if matches!(
                        self.states.last(),
                        Some(State::ProcessingArray | State::ProcessingObject)
                    ) {
                        self.states.push(State::OptionalComma);
                    }
                }
                Token::Colon => match state {
                    State::RequireColon => self.states.push(State::RequireElement),
                    _ => return Err(ValidatorError::InvalidJSON(consumed)),
                },
                Token::Comma => match state {
                    State::OptionalComma => match self.states.last() {
                        Some(State::ProcessingObject) => self.states.push(State::RequireObjectKey),
                        Some(State::ProcessingArray) => self.states.push(State::RequireElement),
                        _ => return Err(ValidatorError::InvalidJSON(consumed)),
                    },
                    _ => return Err(ValidatorError::InvalidJSON(consumed)),
                },
                Token::String(str) => match state {
                    State::OptionalObjectKey | State::RequireObjectKey => {
                        try_add_object_key!(self, str, consumed);
                        self.states.push(State::RequireColon);
                    }
                    State::OptionalElement | State::RequireElement => {
                        if str.len() > self.max_string_length {
                            return Err(ValidatorError::MaxStringLengthExceeded);
                        }

                        let last_state = self.states.last_mut();
                        match last_state {
                            Some(State::ProcessingObject) => {
                                try_add_object_value!(self, consumed);
                                self.states.push(State::OptionalComma);
                            }
                            Some(State::ProcessingArray) => {
                                try_add_array_entry!(self, consumed);
                                self.states.push(State::OptionalComma);
                            }
                            _ => (),
                        }
                    }
                    _ => return Err(ValidatorError::InvalidJSON(consumed)),
                },
                Token::Number | Token::True | Token::False | Token::Null => match state {
                    State::OptionalElement | State::RequireElement => {
                        let last_state = self.states.last_mut();
                        match last_state {
                            Some(State::ProcessingObject) => {
                                try_add_object_value!(self, consumed);
                                self.states.push(State::OptionalComma);
                            }
                            Some(State::ProcessingArray) => {
                                try_add_array_entry!(self, consumed);
                                self.states.push(State::OptionalComma);
                            }
                            _ => (),
                        }
                    }
                    _ => return Err(ValidatorError::InvalidJSON(consumed)),
                },
            }
        }

        if self.cur_depth != 0 {
            return Err(ValidatorError::InvalidJSON(self.lexer.get_consumed()));
        }

        if self.lexer.next()?.is_some() {
            return Err(ValidatorError::TrailingData);
        }

        Ok(())
    }

    // fn try_active_array(&mut self) -> Result<(), ValidatorError> {
    //     self.entires.push(0);
    //     self.try_inc_depth()
    // }

    // fn try_add_array_entry(&mut self) -> Result<(), ValidatorError> {
    //     let entries = self
    //         .entires
    //         .last_mut()
    //         .ok_or(ValidatorError::InvalidJSON(self.lexer.get_consumed()))?;
    //     *entries += 1;

    //     if *entries > self.max_array_entries {
    //         return Err(ValidatorError::MaxArrayEntriesExceeded);
    //     }

    //     Ok(())
    // }

    // fn try_finalize_array(&mut self) -> Result<usize, ValidatorError> {
    //     let entries = self
    //         .entires
    //         .pop()
    //         .ok_or(ValidatorError::InvalidJSON(self.lexer.get_consumed()))?;
    //     self.try_dec_depth()?;
    //     Ok(entries)
    // }

    // fn try_active_object(&mut self) -> Result<(), ValidatorError> {
    //     if !self.allow_duplicate_key {
    //         self.keys.push(HashSet::with_capacity(8));
    //     }
    //     self.entires.push(0);
    //     self.try_inc_depth()
    // }

    // fn try_add_object_key<'c: 'b>(&mut self, key: Cow<'c, String>) -> Result<(), ValidatorError> {
    //     if key.len() > self.max_object_key_length {
    //         return Err(ValidatorError::MaxObjectKeyLengthExceeded);
    //     }

    //     if self.allow_duplicate_key {
    //         return Ok(());
    //     }

    //     if self
    //         .keys
    //         .last_mut()
    //         .ok_or(ValidatorError::InvalidJSON(self.lexer.get_consumed()))?
    //         .insert(key.into_owned())
    //     {
    //         return Ok(());
    //     }

    //     Err(ValidatorError::DuplicateObjectKey)
    // }

    // fn try_add_object_value(&mut self) -> Result<(), ValidatorError> {
    //     let entries = self
    //         .entires
    //         .last_mut()
    //         .ok_or(ValidatorError::InvalidJSON(self.lexer.get_consumed()))?;
    //     *entries += 1;

    //     if *entries > self.max_object_entries {
    //         return Err(ValidatorError::MaxObjectEntriesExceeded);
    //     }

    //     Ok(())
    // }

    // fn try_finalize_object(&mut self) -> Result<usize, ValidatorError> {
    //     if !self.allow_duplicate_key {
    //         self.keys
    //             .pop()
    //             .ok_or(ValidatorError::InvalidJSON(self.lexer.get_consumed()))?;
    //     }
    //     let entries = self
    //         .entires
    //         .pop()
    //         .ok_or(ValidatorError::InvalidJSON(self.lexer.get_consumed()))?;
    //     self.try_dec_depth()?;
    //     Ok(entries)
    // }

    // fn try_inc_depth(&mut self) -> Result<(), ValidatorError> {
    //     self.cur_depth += 1;

    //     if self.cur_depth > self.max_depth {
    //         return Err(ValidatorError::MaxDepthExceeded);
    //     }

    //     Ok(())
    // }

    // fn try_dec_depth(&mut self) -> Result<(), ValidatorError> {
    //     self.cur_depth -= 1;

    //     if self.cur_depth > self.max_depth {
    //         return Err(ValidatorError::MaxDepthExceeded);
    //     }

    //     Ok(())
    // }
}
