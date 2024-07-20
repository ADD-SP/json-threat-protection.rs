use thiserror::Error;

use crate::lexer::Lexer;
use crate::lexer::LexerError;
use crate::lexer::Token;
use crate::read::{Position, Read};
use std::collections::HashSet;

#[allow(unused_macros)]
macro_rules! allow_duplicate_object_entry_name {
    ($self:ident) => {
        $self.allow_duplicate_object_entry_name
    };
}

macro_rules! disallow_duplicate_object_entry_name {
    ($self:ident) => {{
        !$self.allow_duplicate_object_entry_name
    }};
}

macro_rules! try_inc_depth {
    ($self:ident, $position:ident) => {
        $self.cur_depth += 1;

        if $self.cur_depth > $self.max_depth {
            return Err(ValidatorError::MaxDepthExceeded {
                position: $position,
                limit: $self.max_depth,
            });
        }
    };
}

macro_rules! try_dec_depth {
    ($self:ident, $position:ident) => {
        if $self.cur_depth == 0 {
            return Err(ValidatorError::InvalidJSON($position));
        }

        $self.cur_depth -= 1;

        if $self.cur_depth > $self.max_depth {
            return Err(ValidatorError::MaxDepthExceeded {
                position: $position,
                limit: $self.max_depth,
            });
        }
    };
}

macro_rules! try_active_array {
    ($self:ident, $position:ident) => {
        $self.entires.push(0);
        try_inc_depth!($self, $position);
    };
}

macro_rules! try_add_array_entry {
    ($self:ident, $position:ident) => {
        let entries = $self
            .entires
            .last_mut()
            .ok_or(ValidatorError::InvalidJSON($position))?;
        *entries += 1;

        if *entries > $self.max_array_entries {
            return Err(ValidatorError::MaxArrayEntriesExceeded {
                position: $position,
                limit: $self.max_array_entries,
            });
        }
    };
}

macro_rules! try_finalize_array {
    ($self:ident, $position:ident) => {{
        let entries = $self
            .entires
            .pop()
            .ok_or(ValidatorError::InvalidJSON($position))?;
        try_dec_depth!($self, $position);
        entries
    }};
}

macro_rules! try_active_object {
    ($self:ident, $position:ident) => {
        if disallow_duplicate_object_entry_name!($self) {
            $self.keys.push(HashSet::with_capacity(8));
        }
        $self.entires.push(0);
        try_inc_depth!($self, $position);
    };
}

macro_rules! try_add_object_key {
    ($self:ident, $key:ident, $position:ident) => {
        if $key.len() > $self.max_object_entry_name_length {
            return Err(ValidatorError::MaxObjectEntryNameLengthExceeded {
                position: $position,
                limit: $self.max_object_entry_name_length,
                name: $key,
            });
        }

        if disallow_duplicate_object_entry_name!($self) {
            let keys = $self
                .keys
                .last_mut()
                .ok_or(ValidatorError::InvalidJSON($position))?;

            if !keys.insert($key.to_string()) {
                return Err(ValidatorError::DuplicateObjectEntryName {
                    position: $position,
                    key: $key.to_string(),
                });
            }
        }
    };
}

macro_rules! try_add_object_value {
    ($self:ident, $position:ident) => {
        let entries = $self
            .entires
            .last_mut()
            .ok_or(ValidatorError::InvalidJSON($position))?;
        *entries += 1;

        if *entries > $self.max_object_entries {
            return Err(ValidatorError::MaxObjectEntriesExceeded {
                position: $position,
                limit: $self.max_object_entries,
            });
        }
    };
}

macro_rules! try_finalize_object {
    ($self:ident, $position:ident) => {{
        if disallow_duplicate_object_entry_name!($self) {
            $self
                .keys
                .pop()
                .ok_or(ValidatorError::InvalidJSON($position))?;
        }
        let entries = $self
            .entires
            .pop()
            .ok_or(ValidatorError::InvalidJSON($position))?;
        try_dec_depth!($self, $position);
        entries
    }};
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    /// Validator is expecting an optional object entry
    ProcessingObject,

    /// Validator is expecting an optional array entry
    ProcessingArray,

    /// Validator is expecting an element
    RequireElement,

    /// Validator is expecting an required colon (`:`)
    RequireColon,

    /// Validator is expecting an required object key
    RequireObjectKey,

    /// Validator is expecting an optional comma (`,`) at the end of an entry
    OptionalComma,

    /// Validator is expecting an optional object key
    OptionalObjectKey,

    /// Validator is expecting an optional element
    OptionalElement,
}

#[derive(Error, Debug)]
/// Error occurred during JSON validation
pub enum ValidatorError {
    /// Error occurred during lexing
    #[error("lexer error: {0}")]
    LexerError(#[from] LexerError),

    /// Error occurred once the JSON is invalid, such as unclosed object or array
    #[error("invalid JSON ({0})")]
    InvalidJSON(Position),

    /// Error occurred when there is trailing data after the valid JSON
    #[error("trailing data (position {0} bytes)")]
    TrailingData(Position),

    /// Error occurred when the maximum depth is exceeded
    #[error("maximum depth exceeded (limit: {limit}, {position})")]
    MaxDepthExceeded {
        /// Position where the error occurred
        position: Position,

        /// Maximum depth allowed
        limit: usize,
    },

    /// Error occurred when the maximum string length is exceeded
    #[error("maximum string length exceeded (limit: {limit}, {position})")]
    MaxStringLengthExceeded {
        /// Position where the error occurred
        position: Position,

        /// Maximum string length allowed
        limit: usize,

        /// String that exceeds the limit
        str: String,
    },

    /// Error occurred when the maximum array entries is exceeded
    #[error("maximum array entries exceeded (limit: {limit}, {position})")]
    MaxArrayEntriesExceeded {
        /// Position where the error occurred
        position: Position,

        /// Maximum array entries allowed
        limit: usize,
    },

    /// Error occurred when the maximum object entries is exceeded
    #[error("maximum object entries exceeded (limit: {limit}, {position})")]
    MaxObjectEntriesExceeded {
        /// Position where the error occurred
        position: Position,

        /// Maximum object entries allowed
        limit: usize,
    },

    /// Error occurred when the maximum object entry name length is exceeded
    #[error("maximum object entry name length exceeded (limit: {limit}, {position})")]
    MaxObjectEntryNameLengthExceeded {
        /// Position where the error occurred
        position: Position,

        /// Maximum object entry name length allowed
        limit: usize,

        /// Object entry name that exceeds the limit
        name: String,
    },

    /// Error occurred when there is a duplicate object entry name
    #[error("duplicate object entry name (key: {key}, {position})")]
    DuplicateObjectEntryName {
        /// Position where the error occurred
        position: Position,

        /// Duplicate object entry name
        key: String,
    },

    /// Error occurred when running into unexpected state, please report this issue to the maintainer
    #[error("running into unexpected state, please report this issue to the maintainer, ({msg}) ({position})")]
    Bug {
        /// Diagnostic message
        msg: String,

        /// Position where the error occurred
        position: Position,
    },
}

/// Interal JSON validator
pub struct Validator<R: Read> {
    lexer: Lexer<R>,

    /// Stack of states, keep track of the current state of the validator
    states: Vec<State>,

    /// Stack of entries, keep track of the number of entries in the current array or object
    entires: Vec<usize>,

    /// Stack of keys, keep track of the keys in the current object
    keys: Vec<HashSet<String>>,

    /// Current depth of the JSON
    cur_depth: usize,

    max_depth: usize,
    max_string_length: usize,
    max_array_entries: usize,
    max_object_entries: usize,
    max_object_entry_name_length: usize,
    allow_duplicate_object_entry_name: bool,
}

impl<R: Read> Validator<R> {
    pub fn new(
        read: R,
        max_depth: usize,
        max_string_length: usize,
        max_array_entries: usize,
        max_object_entries: usize,
        max_object_entry_name_length: usize,
        allow_duplicate_object_entry_name: bool,
    ) -> Self {
        let mut states = Vec::with_capacity(32);
        states.push(State::RequireElement);

        Validator {
            lexer: Lexer::new(read),
            states,
            entires: Vec::with_capacity(32),
            keys: Vec::with_capacity(32),
            cur_depth: 0,

            max_depth,
            max_string_length,
            max_array_entries,
            max_object_entries,
            max_object_entry_name_length,
            allow_duplicate_object_entry_name,
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

    pub fn with_max_object_entry_name_length(
        mut self,
        max_object_entry_name_length: usize,
    ) -> Self {
        self.max_object_entry_name_length = max_object_entry_name_length;
        self
    }

    pub fn allow_duplicate_object_entry_name(mut self) -> Self {
        self.allow_duplicate_object_entry_name = true;
        self
    }

    pub fn disallow_duplicate_object_entry_name(mut self) -> Self {
        self.allow_duplicate_object_entry_name = false;
        self
    }

    pub fn validate(mut self) -> Result<(), ValidatorError> {
        loop {
            match self.validate_with_steps(usize::MAX)? {
                true => return Ok(()),
                false => (),
            }
        }
    }

    pub fn validate_with_steps(&mut self, steps: usize) -> Result<bool, ValidatorError> {
        match self.inner_validate(steps) {
            Ok(finished) => Ok(finished),
            Err(e) => Err(self.correct_position(e)),
        }
    }

    /// Correct the position of the error
    fn correct_position(&self, err: ValidatorError) -> ValidatorError {
        match err {
            ValidatorError::InvalidJSON(_) => ValidatorError::InvalidJSON(self.lexer.position()),
            ValidatorError::TrailingData(_) => ValidatorError::TrailingData(self.lexer.position()),
            ValidatorError::MaxDepthExceeded { position: _, limit } => {
                ValidatorError::MaxDepthExceeded {
                    position: self.lexer.position(),
                    limit,
                }
            }
            ValidatorError::MaxStringLengthExceeded {
                position: _,
                limit,
                str,
            } => ValidatorError::MaxStringLengthExceeded {
                position: self.lexer.position(),
                limit,
                str,
            },
            ValidatorError::MaxArrayEntriesExceeded { position: _, limit } => {
                ValidatorError::MaxArrayEntriesExceeded {
                    position: self.lexer.position(),
                    limit,
                }
            }
            ValidatorError::MaxObjectEntriesExceeded { position: _, limit } => {
                ValidatorError::MaxObjectEntriesExceeded {
                    position: self.lexer.position(),
                    limit,
                }
            }
            ValidatorError::MaxObjectEntryNameLengthExceeded {
                position: _,
                limit,
                name,
            } => ValidatorError::MaxObjectEntryNameLengthExceeded {
                position: self.lexer.position(),
                limit,
                name,
            },
            ValidatorError::DuplicateObjectEntryName { position: _, key } => {
                ValidatorError::DuplicateObjectEntryName {
                    position: self.lexer.position(),
                    key,
                }
            }
            ValidatorError::LexerError(e) => ValidatorError::LexerError(e),
            ValidatorError::Bug { msg, position: _ } => ValidatorError::Bug {
                msg,
                position: self.lexer.position(),
            },
        }
    }

    fn inner_validate(&mut self, steps: usize) -> Result<bool, ValidatorError> {
        let mut remaining_steps = steps;

        // Dummy position for constructing `ValidatorError`
        // so that we can keep the error reason once the error occurs,
        // and the real position will be updated by the caller of this method.
        // Since the `self.lexer.next()` will borrow the `self.lexer` mutably,
        // and its return value has the same lifetime of this borrow,
        // so we can't borrow `self.lexer` imuutably to get the position
        // after the invoking of `self.lexer.next()`.
        // This is a workaround to make the borrow checker happy.
        let dummy_position = Position::default();

        while let Some(state) = self.states.pop() {
            let token = self.lexer.next()?;

            if token.is_none() {
                return Err(ValidatorError::InvalidJSON(dummy_position));
            }

            // unwrap is safe here since we have checked the token is not None.
            match token.unwrap() {
                Token::LBrace => match state {
                    State::RequireElement | State::OptionalElement => {
                        self.states.push(State::ProcessingObject);
                        self.states.push(State::OptionalObjectKey);
                        try_active_object!(self, dummy_position);
                    }
                    _ => return Err(ValidatorError::InvalidJSON(dummy_position)),
                },
                Token::RBrace => {
                    match state {
                        State::OptionalComma | State::OptionalObjectKey => {
                            let state = self
                                .states
                                .pop()
                                .ok_or(ValidatorError::InvalidJSON(dummy_position))?;
                            if state != State::ProcessingObject {
                                return Err(ValidatorError::InvalidJSON(dummy_position));
                            }

                            let entires = try_finalize_object!(self, dummy_position);
                            if state == State::OptionalObjectKey && entires != 0 {
                                return Err(ValidatorError::InvalidJSON(dummy_position));
                            }
                        }
                        State::ProcessingObject => {
                            if try_finalize_object!(self, dummy_position) == 0 {
                                return Err(ValidatorError::InvalidJSON(dummy_position));
                            }
                        }
                        _ => return Err(ValidatorError::InvalidJSON(dummy_position)),
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
                        try_active_array!(self, dummy_position);
                    }
                    _ => return Err(ValidatorError::InvalidJSON(dummy_position)),
                },
                Token::RBracket => {
                    match state {
                        State::OptionalComma | State::OptionalElement => {
                            let state = self
                                .states
                                .pop()
                                .ok_or(ValidatorError::InvalidJSON(dummy_position))?;
                            if state != State::ProcessingArray {
                                return Err(ValidatorError::InvalidJSON(dummy_position));
                            }

                            let entries = try_finalize_array!(self, dummy_position);
                            if state == State::OptionalElement && entries != 0 {
                                return Err(ValidatorError::InvalidJSON(dummy_position));
                            }
                        }
                        State::ProcessingArray => {
                            if try_finalize_array!(self, dummy_position) == 0 {
                                return Err(ValidatorError::InvalidJSON(dummy_position));
                            }
                        }
                        _ => return Err(ValidatorError::InvalidJSON(dummy_position)),
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
                    _ => return Err(ValidatorError::InvalidJSON(dummy_position)),
                },
                Token::Comma => match state {
                    State::OptionalComma => match self.states.last() {
                        Some(State::ProcessingObject) => self.states.push(State::RequireObjectKey),
                        Some(State::ProcessingArray) => self.states.push(State::RequireElement),
                        _ => return Err(ValidatorError::InvalidJSON(dummy_position)),
                    },
                    _ => return Err(ValidatorError::InvalidJSON(dummy_position)),
                },
                Token::String(str) => match state {
                    State::OptionalObjectKey | State::RequireObjectKey => {
                        try_add_object_key!(self, str, dummy_position);
                        self.states.push(State::RequireColon);
                    }
                    State::OptionalElement | State::RequireElement => {
                        if str.len() > self.max_string_length {
                            return Err(ValidatorError::MaxStringLengthExceeded {
                                position: dummy_position,
                                limit: self.max_string_length,
                                str: str,
                            });
                        }

                        let last_state = self.states.last_mut();
                        match last_state {
                            Some(State::ProcessingObject) => {
                                try_add_object_value!(self, dummy_position);
                                self.states.push(State::OptionalComma);
                            }
                            Some(State::ProcessingArray) => {
                                try_add_array_entry!(self, dummy_position);
                                self.states.push(State::OptionalComma);
                            }
                            _ => (),
                        }
                    }
                    _ => return Err(ValidatorError::InvalidJSON(dummy_position)),
                },
                Token::Number | Token::True | Token::False | Token::Null => match state {
                    State::OptionalElement | State::RequireElement => {
                        let last_state = self.states.last_mut();
                        match last_state {
                            Some(State::ProcessingObject) => {
                                try_add_object_value!(self, dummy_position);
                                self.states.push(State::OptionalComma);
                            }
                            Some(State::ProcessingArray) => {
                                try_add_array_entry!(self, dummy_position);
                                self.states.push(State::OptionalComma);
                            }
                            _ => (),
                        }
                    }
                    _ => return Err(ValidatorError::InvalidJSON(dummy_position)),
                },
            }

            remaining_steps -= 1;
            if remaining_steps == 0 {
                break;
            }
        }

        // At this point, there are four possible cases:
        // * The JSON is valid and there is no more token to process.
        //     * `self.states.is_empty() && self.cur_depth == 0 && self.lexer.peek()?.is_none()`.
        // * The JSON is valid but there is trailing data.
        //     * `self.states.is_empty() && self.cur_depth == 0 && self.lexer.peek()?.is_some()`.
        // * The JSON is invalid.
        //     * `(!self.states.is_empty() || self.cur_depth != 0) && self.lexer.peek()?.is_none()`.
        // * The JSON is still being processed, and the `remaining_steps`` are exhausted.
        //     * `(!self.states.is_empty() || self.cur_depth != 0) && self.lexer.peek()?.is_some()`.

        let has_states = !self.states.is_empty();
        let no_depth = self.cur_depth == 0;
        let has_more_token = self.lexer.peek()?.is_some();

        if has_states || !no_depth {
            if has_more_token {
                if remaining_steps != 0 {
                    return Err(ValidatorError::Bug {
                        msg: format!(
                            "Validator.inner_validate: remaining_steps should be 0, but got {}",
                            remaining_steps
                        ),
                        position: dummy_position,
                    });
                }
                return Ok(false);
            }

            return Err(ValidatorError::InvalidJSON(dummy_position));
        }

        if !no_depth {
            return Err(ValidatorError::Bug {
                msg: "Validator.inner_validate: current depth should be 0".to_string(),
                position: dummy_position,
            });
        }

        if !has_states && !has_more_token {
            return Ok(true);
        }

        if has_states {
            return Err(ValidatorError::InvalidJSON(dummy_position));
        }

        Err(ValidatorError::TrailingData(dummy_position))
    }
}
