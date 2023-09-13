use crate::{error::CrateError, tokenizer::TokenizerError};

const OPERATORS: [&str; 15] = [
    "!", "&", "|", "+", "-", "*", "/", "<", ">", "=", "==", ">=", "<=", "::", "->",
];

pub fn is_operator(s: &str) -> bool {
    OPERATORS.contains(&s)
}

#[derive(Debug)]
pub enum Operator {
    Not,
    And,
    Or,
    Assignment,
    DoubleColon,
    Arrow,
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl TryFrom<&str> for Operator {
    type Error = &'static str;
    fn try_from(s: &str) -> Result<Operator, Self::Error> {
        let s = match s {
            "<=" => Operator::LessThanOrEqual,
            ">=" => Operator::GreaterThanOrEqual,
            "==" => Operator::Equal,
            "::" => Operator::DoubleColon,
            "->" => Operator::Arrow,
            _ => Operator::try_from(s.chars().next().unwrap()).unwrap(),
        };
        Ok(s)
    }
}

impl TryFrom<char> for Operator {
    type Error = CrateError;
    fn try_from(c: char) -> Result<Operator, Self::Error> {
        match c {
            '!' => Ok(Operator::Not),
            '&' => Ok(Operator::And),
            '|' => Ok(Operator::Or),
            '+' => Ok(Operator::Add),
            '-' => Ok(Operator::Subtract),
            '*' => Ok(Operator::Multiply),
            '/' => Ok(Operator::Divide),
            '<' => Ok(Operator::LessThan),
            '>' => Ok(Operator::GreaterThan),
            '=' => Ok(Operator::Assignment),
            _ => Err(CrateError::TokenizerError(TokenizerError::UnknownOperator(
                c.to_string(),
            ))),
        }
    }
}
