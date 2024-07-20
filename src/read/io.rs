use super::{Position, Read, ReadError};

/// A reader that reads from an object that implements [`std::io::Read`].
///
/// # Performance
///
/// This reader uses the provided methods of [`Read`] to read from the underlying reader,
/// which results in a lot of checking on [`std::io::Error`],
/// so it should be slower than [`super::SliceRead`] and [`super::StrRead`].
pub struct IoRead<R: std::io::Read> {
    reader: R,
    ch: Option<u8>,
    position: Position,
}

impl<R: std::io::Read> IoRead<R> {
    pub fn new(reader: R) -> Self {
        IoRead {
            reader,
            ch: None,
            position: Position::default(),
        }
    }
}

impl<R: std::io::Read> Read for IoRead<R> {
    fn position(&self) -> Position {
        self.position
    }

    fn peek(&mut self) -> Result<Option<u8>, ReadError> {
        if self.ch.is_some() {
            return Ok(self.ch);
        }

        let mut buf = [0; 1];
        match self.reader.read(&mut buf) {
            Ok(0) => Ok(None),
            Ok(1) => {
                self.ch = Some(buf[0]);
                Ok(self.ch)
            }
            Err(err) => Err(ReadError::IoError(err, self.position())),
            _ => Err(ReadError::Bug {
                msg: "IoRead.peek: `self.reader.read()` returned unexpected value".to_string(),
                position: self.position(),
            }),
        }
    }

    fn next(&mut self) -> Result<Option<u8>, ReadError> {
        let ch = match self.ch {
            Some(ch) => {
                self.ch = None;
                ch
            }
            None => {
                let mut buf = [0; 1];
                match self.reader.read(&mut buf) {
                    Ok(0) => return Ok(None),
                    Ok(1) => buf[0],
                    Err(err) => return Err(ReadError::IoError(err, self.position())),
                    _ => {
                        return Err(ReadError::Bug {
                            msg: "IoRead.peek: `self.reader.read()` returned unexpected value"
                                .to_string(),
                            position: self.position(),
                        })
                    }
                }
            }
        };

        match ch {
            b'\n' => {
                self.position.line += 1;
                self.position.column = 1;
            }
            _ => {
                self.position.column += 1;
            }
        }

        self.position.offset += 1;
        Ok(Some(ch))
    }
}
