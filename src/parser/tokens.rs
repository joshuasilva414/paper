use std::collections::VecDeque;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read};

#[derive(Debug, Clone)]
pub enum Token {
    Boolean(bool),
    Integer(isize),
    Real(f64),
    LiteralString(Vec<u8>),
    HexString(Vec<u8>),
    Name(Vec<u8>),

    ArrayStart,
    ArrayEnd,
    DictionaryStart,
    DictionaryEnd,

    Null,
    Keyword(Vec<u8>),
    R,

    Comment(Vec<u8>),
    Eof,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Boolean(value) => write!(f, "{value}"),
            Token::Integer(value) => write!(f, "{value}"),
            Token::Real(value) => write!(f, "{value}"),
            Token::LiteralString(value) => write!(f, "({})", String::from_utf8_lossy(value)),
            Token::HexString(value) => write!(f, "<{}", String::from_utf8_lossy(value)),
            Token::Name(value) => write!(f, "/{}", String::from_utf8_lossy(value)),
            Token::ArrayStart => f.write_str("["),
            Token::ArrayEnd => f.write_str("]"),
            Token::DictionaryStart => f.write_str("<<"),
            Token::DictionaryEnd => f.write_str(">>"),
            Token::Null => f.write_str("null"),
            Token::Keyword(value) => f.write_str(&String::from_utf8_lossy(value)),
            Token::R => f.write_str("R"),
            Token::Comment(value) => write!(f, "%{}", String::from_utf8_lossy(value)),
            Token::Eof => f.write_str("EOF"),
        }
    }
}

pub struct TokenIter<'a> {
    reader: &'a mut BufReader<File>,
    queue: VecDeque<Token>,
}

impl<'a> TokenIter<'a> {
    pub fn new(reader: &'a mut BufReader<File>) -> Self {
        Self {
            reader,
            queue: VecDeque::new(),
        }
    }

    pub fn next_token(&mut self) -> io::Result<Option<Token>> {
        if self.queue.is_empty() {
            self.read_token()
        } else {
            Ok(self.queue.pop_front())
        }
    }

    pub fn peek_next_token(&mut self) -> io::Result<Option<&Token>> {
        if self.queue.is_empty() {
            if let Some(token) = self.read_token()? {
                self.queue.push_back(token);
            }
        }

        Ok(self.queue.front())
    }

    pub fn peek_nth_token(&mut self, n: usize) -> io::Result<Option<&Token>> {
        while self.queue.len() < n {
            if let Some(token) = self.read_token()? {
                self.queue.push_back(token);
            } else {
                return Ok(None);
            }
        }

        Ok(self.queue.get(n - 1))
    }

    fn read_token(&mut self) -> io::Result<Option<Token>> {
        let mut buf: Vec<u8> = Vec::new();
        let mut bytes = self.reader.by_ref().bytes();

        loop {
            if let Some(next_byte) = bytes.next() {
                let byte = next_byte?;
                if buf.is_empty() && byte.is_ascii_whitespace() {
                    continue;
                }
                buf.push(byte);
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Ran out of bytes parsing token",
                ));
            }

            let token = Self::match_token(buf.as_slice());

            if let Some(token) = token {
                return match token {
                    Token::LiteralString(tok) => Ok(Some(self.read_literal_string(tok)?)),
                    Token::HexString(tok) => {
                        if let Some(next_byte) = bytes.next() {
                            let byte = next_byte?;
                            if buf.is_empty() && byte.is_ascii_whitespace() {
                                continue;
                            }
                            buf.push(byte);
                        } else {
                            return Err(io::Error::new(
                                io::ErrorKind::UnexpectedEof,
                                "Ran out of bytes parsing token",
                            ));
                        }
                        match Self::match_token(buf.as_slice()) {
                            Some(Token::DictionaryStart) => Ok(Some(Token::DictionaryStart)),
                            None => {
                                self.reader.seek_relative(-1)?;
                                Ok(Some(self.read_hex_string(tok)?))
                            }
                            Some(_) => Err(io::Error::new(
                                io::ErrorKind::InvalidInput,
                                format!("token buffer has {}", String::from_utf8(buf).unwrap()),
                            )),
                        }
                    }
                    Token::Integer(_) => Ok(Some(self.read_integer(buf)?)),
                    Token::Comment(tok) => Ok(Some(self.read_comment(tok)?)),
                    Token::Name(tok) => Ok(Some(self.read_name(tok)?)),
                    tok => Ok(Some(tok)),
                };
            }
        }
    }

    fn read_integer(&mut self, mut buf: Vec<u8>) -> io::Result<Token> {
        loop {
            let available = self.reader.fill_buf()?;

            if available.is_empty() {
                break;
            }

            let digit_count = available
                .iter()
                .take_while(|byte| byte.is_ascii_digit())
                .count();
            let has_non_digit = digit_count < available.len();

            buf.extend_from_slice(&available[..digit_count]);
            self.reader.consume(digit_count);

            if has_non_digit {
                break;
            }
        }

        let text = std::str::from_utf8(&buf)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

        let value = text.parse::<isize>().map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid integer {text:?}: {error}"),
            )
        })?;

        Ok(Token::Integer(value))
    }

    fn read_literal_string(&mut self, mut token: Vec<u8>) -> io::Result<Token> {
        self.reader.read_until(b')', &mut token)?;
        Ok(Token::LiteralString(token))
    }

    fn read_hex_string(&mut self, mut token: Vec<u8>) -> io::Result<Token> {
        self.reader.read_until(b'>', &mut token)?;
        token.pop();
        Ok(Token::HexString(token))
    }

    fn read_comment(&mut self, mut token: Vec<u8>) -> io::Result<Token> {
        self.reader.read_until(b'\n', &mut token)?;
        Ok(Token::Comment(token))
    }

    fn read_name(&mut self, mut token: Vec<u8>) -> io::Result<Token> {
        let mut bytes = self.reader.by_ref().bytes();

        while let Some(byte) = bytes.next() {
            if let Ok(ok_byte) = byte {
                if !(ok_byte as char).is_whitespace() {
                    token.push(ok_byte);
                } else {
                    break;
                }
            }
        }
        Ok(Token::Name(token))
    }

    fn match_token(buf: &[u8]) -> Option<Token> {
        match buf {
            b"<<" => Some(Token::DictionaryStart),
            b">>" => Some(Token::DictionaryEnd),
            b"[" => Some(Token::ArrayStart),
            b"]" => Some(Token::ArrayEnd),
            b"/" => Some(Token::Name(Vec::new())),
            b"(" => Some(Token::LiteralString(Vec::new())),
            b"<" => Some(Token::HexString(Vec::new())),
            b"R" => Some(Token::R),
            b"%" => Some(Token::Comment(Vec::new())),
            x if !x.is_empty() && (x[0].is_ascii_digit() || x[0] == b'.' || x[0] == b'-') => {
                Some(Token::Integer(0))
            }
            _ => None,
        }
    }
}
