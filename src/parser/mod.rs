mod tokens;

use crate::paper::{Dictionary, ObjectRef, PdfObject};
use crate::parser::ParseError::UnexpectedEndOfTokens;
use crate::parser::tokens::{Token, TokenIter};
use std::fs::File;
use std::io;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken,
    UnexpectedEndOfTokens,
    IOError(io::Error),
}

pub type ParseResult<T> = Result<T, ParseError>;

pub fn parse_dictionary(file: &mut File) -> Result<Dictionary, ParseError> {
    let mut token_iter = TokenIter::new(file);

    let start_token = token_iter
        .next_token()
        .map_err(|e| ParseError::IOError(e))?;
    match start_token {
        Some(Token::DictionaryStart) => (),
        _ => return Err(ParseError::UnexpectedToken),
    }

    let mut dictionary = Dictionary::new();
    loop {
        // Read Key
        let next_token = token_iter
            .next_token()
            .map_err(|e| ParseError::IOError(e))?;
        let key;
        match next_token {
            None => return Err(ParseError::UnexpectedEndOfTokens),
            Some(Token::DictionaryEnd) => break,
            Some(Token::Name(name_value)) => key = name_value,
            Some(_) => return Err(ParseError::UnexpectedToken),
        }

        // Read Value
        let value = parse_pdf_object(&mut token_iter)?;

        // Insert Into Dictionary
        dictionary.insert(String::from_utf8(key).unwrap(), value);
    }

    Err(ParseError::UnexpectedToken)
}

pub fn parse_pdf_object(token_iter: &mut TokenIter) -> ParseResult<PdfObject> {
    let first_token = to_parse_result(token_iter.peek_next_token())?.cloned();
    if let Some(first_token) = first_token {
        match first_token {
            Token::Integer(x) => {
                return {
                    let next_token = to_parse_result(token_iter.peek_nth_token(2))?.cloned();
                    match next_token {
                        None => Ok(PdfObject::Integer(x)),
                        Some(Token::Integer(_)) => {
                            Ok(PdfObject::ObjectRef(parse_object_reference(token_iter)?))
                        }
                        Some(_) => todo!("implement"),
                    }
                };
            }
            // Boolean(p) => todo!("add boolean to PdfObject"),
            _ => (),
        }
    } else {
        return Err(ParseError::UnexpectedEndOfTokens);
    }

    Err(ParseError::UnexpectedToken)
}

fn to_parse_result<T>(token_result: io::Result<T>) -> ParseResult<T> {
    match token_result {
        Err(io_err) => Err(ParseError::IOError(io_err))?,
        Ok(optional_token) => Ok(optional_token),
    }
}

fn parse_object_reference(token_iter: &mut TokenIter) -> ParseResult<ObjectRef> {
    let object_number =
        to_parse_result(token_iter.next_token())?.ok_or(ParseError::UnexpectedEndOfTokens)?;
    let generation_number =
        to_parse_result(token_iter.next_token())?.ok_or(ParseError::UnexpectedEndOfTokens)?;
    let r = to_parse_result(token_iter.next_token())?.ok_or(ParseError::UnexpectedEndOfTokens)?;

    match (object_number, generation_number, r) {
        (Token::Integer(object_number), Token::Integer(generation_number), Token::R) => {
            Ok(ObjectRef {
                object_number,
                generation_number,
            })
        }
        _ => Err(ParseError::UnexpectedToken),
    }
}
