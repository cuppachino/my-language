mod keyword;
pub use keyword::*;

mod operator;
pub use operator::*;

mod paren;
pub use paren::*;

mod symbol;
pub use symbol::*;

use crate::{error::CrateError, rules::is_valid_identifier, tokenizer::TokenizerError};

#[derive(Debug)]
pub enum Token {
    Keyword(Keyword),
    Symbol(Symbol),
    Identifier(String),
    IntegerConstant(i32),
    StringConstant(String),
    Comment(String),
    Paren(Paren),
}

impl TryFrom<&str> for Token {
    type Error = CrateError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let s = match s {
            ";" => Token::Symbol(Symbol::Semicolon),
            "(" | ")" | "[" | "]" | "{" | "}" => Token::Paren(Paren::try_from(s).unwrap()),
            o if is_operator(&s) => Token::Symbol(Symbol::Operator(o.try_into().unwrap())),
            i if i.parse::<i32>().is_ok() => Token::IntegerConstant(i.parse::<i32>().unwrap()),
            "let" | "fn" => Token::Keyword(Keyword::try_from(s).unwrap()),
            _ => {
                if is_valid_identifier(s) {
                    Token::Identifier(s.to_owned())
                } else {
                    return Err(CrateError::TokenizerError(
                        TokenizerError::InvalidIdentifier(s.to_string()),
                    ));
                }
            }
        };
        Ok(s)
    }
}
