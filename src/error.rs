use crate::tokenizer::TokenizerError;

#[derive(Debug)]
pub enum CrateError {
    IoError(std::io::Error),
    TokenizerError(TokenizerError),
}

impl From<std::io::Error> for CrateError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl std::fmt::Display for CrateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO Error: {}", e),
            Self::TokenizerError(e) => write!(f, "Tokenizer Error: {}", e),
        }
    }
}
