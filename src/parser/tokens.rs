use std::collections::VecDeque;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read};

#[derive(Clone)]
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

pub struct TokenIter<'a> {
    reader: BufReader<&'a mut File>,
    queue: VecDeque<Token>,
}

impl<'a> TokenIter<'a> {
    pub fn new(file: &'a mut File) -> Self {
        Self {
            reader: BufReader::new(file),
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
                buf.push(next_byte?);
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
                    Token::HexString(tok) => Ok(Some(self.read_hex_string(tok)?)),
                    Token::Name(tok) => Ok(Some(self.read_name(tok)?)),
                    _ => Ok(Some(token)),
                };
            }
        }
    }

    fn read_literal_string(&mut self, mut token: Vec<u8>) -> io::Result<Token> {
        self.reader.read_until(b')', &mut token)?;
        Ok(Token::LiteralString(token))
    }

    fn read_hex_string(&mut self, mut token: Vec<u8>) -> io::Result<Token> {
        self.reader.read_until(b'>', &mut token)?;
        Ok(Token::HexString(token))
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
        Ok(Token::LiteralString(token))
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
            x if (x[0] as char).is_whitespace() => None,
            _ => None,
        }
    }
}
