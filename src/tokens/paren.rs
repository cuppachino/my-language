use crate::{error::CrateError, tokenizer::TokenizerError};

#[derive(Debug)]
pub enum ParenType {
    Round,
    Curly,
    Square,
}

#[derive(Debug)]
pub enum ParenKind {
    Open,
    Close,
}

#[derive(Debug)]
pub struct Paren(pub ParenType, pub ParenKind);

impl TryFrom<&str> for Paren {
    type Error = CrateError;
    fn try_from(s: &str) -> Result<Paren, Self::Error> {
        Paren::try_from(s.chars().next().unwrap())
    }
}

impl TryFrom<char> for Paren {
    type Error = CrateError;
    fn try_from(c: char) -> Result<Paren, Self::Error> {
        let c = match c {
            '(' => Paren(ParenType::Round, ParenKind::Open),
            ')' => Paren(ParenType::Round, ParenKind::Close),
            '{' => Paren(ParenType::Curly, ParenKind::Open),
            '}' => Paren(ParenType::Curly, ParenKind::Close),
            '[' => Paren(ParenType::Square, ParenKind::Open),
            ']' => Paren(ParenType::Square, ParenKind::Close),
            _ => return Err(CrateError::TokenizerError(TokenizerError::UnknownParen(c))),
        };
        Ok(c)
    }
}
