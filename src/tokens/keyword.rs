use crate::{error::CrateError, tokenizer::TokenizerError};

#[derive(Debug)]
pub struct Keyword(pub String);

impl TryFrom<&str> for Keyword {
    type Error = CrateError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "let" | "fn" => Ok(Keyword(s.to_owned())),
            _ => Err(CrateError::TokenizerError(TokenizerError::InvalidKeyword(
                s.to_string(),
            ))),
        }
    }
}
