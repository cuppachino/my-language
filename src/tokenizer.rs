use std::{
    fs::File,
    io::{BufReader, Lines},
};

use my_language::into_chars;

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

const KEYWORDS: [&str; 2] = ["fn", "let"];

#[derive(Debug)]
pub enum Keyword {
    Fn,
    Let,
}

impl TryFrom<&str> for Keyword {
    type Error = CrateError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "fn" => Ok(Keyword::Fn),
            "let" => Ok(Keyword::Let),
            _ => Err(CrateError::TokenizerError(TokenizerError::InvalidKeyword(
                s.to_string(),
            ))),
        }
    }
}

#[derive(Debug)]
pub enum Symbol {
    Comma,
    Operator(Operator),
    Semicolon,
    NewLine,
    EndOfFile,
}

const OPERATORS: [&str; 15] = [
    "!", "&", "|", "+", "-", "*", "/", "<", ">", "=", "==", ">=", "<=", "::", "->",
];

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

impl TryFrom<char> for Operator {
    type Error = &'static str;
    fn try_from(c: char) -> Result<Operator, Self::Error> {
        let c = match c {
            '!' => Operator::Not,
            '&' => Operator::And,
            '|' => Operator::Or,
            '+' => Operator::Add,
            '-' => Operator::Subtract,
            '*' => Operator::Multiply,
            '/' => Operator::Divide,
            '<' => Operator::LessThan,
            '>' => Operator::GreaterThan,
            '=' => Operator::Assignment,
            _ => panic!("Invalid operator, you probably want to use From<&str> and not From<char>"),
        };
        Ok(c)
    }
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
            _ => {
                let c = s.chars().next().unwrap();
                Operator::try_from(c).unwrap()
            }
        };
        Ok(s)
    }
}

const PARENS: [&str; 6] = ["(", ")", "{", "}", "[", "]"];

#[derive(Debug)]
pub struct Paren(pub ParenType, pub Kind);

#[derive(Debug)]
pub enum ParenType {
    Round,
    Curly,
    Square,
}

#[derive(Debug)]
pub enum Kind {
    Open,
    Close,
}

impl TryFrom<char> for Paren {
    type Error = &'static str;
    fn try_from(c: char) -> Result<Paren, Self::Error> {
        let c = match c {
            '(' => Paren(ParenType::Round, Kind::Open),
            ')' => Paren(ParenType::Round, Kind::Close),
            '{' => Paren(ParenType::Curly, Kind::Open),
            '}' => Paren(ParenType::Curly, Kind::Close),
            '[' => Paren(ParenType::Square, Kind::Open),
            ']' => Paren(ParenType::Square, Kind::Close),
            _ => panic!("Invalid paren"),
        };
        Ok(c)
    }
}

impl TryFrom<&str> for Paren {
    type Error = &'static str;
    fn try_from(s: &str) -> Result<Paren, Self::Error> {
        Paren::try_from(s.chars().next().unwrap())
    }
}

impl TryFrom<&str> for Token {
    type Error = CrateError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let s = match s {
            ";" => Token::Symbol(Symbol::Semicolon),
            k if KEYWORDS.contains(&s) => Token::Keyword(Keyword::try_from(k)?),
            p if PARENS.contains(&s) => Token::Paren(Paren::try_from(p).unwrap()),
            o if OPERATORS.contains(&s) => Token::Symbol(Symbol::Operator(o.try_into().unwrap())),
            i if i.parse::<i32>().is_ok() => Token::IntegerConstant(i.parse::<i32>().unwrap()),
            _ => Token::Identifier(s.to_string()),
        };
        Ok(s)
    }
}

#[derive(Debug, Default)]
pub struct Tokenizer {
    chars: Vec<char>,
    pub tokens: Vec<Token>,
}

impl Tokenizer {
    pub fn new(lines: Lines<BufReader<File>>) -> Self {
        Self {
            chars: into_chars(lines),
            ..Self::default()
        }
    }

    pub fn tokenize(&mut self) -> Result<&mut Self, CrateError> {
        let mut tokens = Vec::new();
        let mut chars = self.chars.iter().peekable();
        let mut prev_newline = false;
        let mut in_string = false;
        while let Some(c) = chars.next() {
            if (c == &'\n' || c == &'\r') && prev_newline {
                continue;
            } else {
                prev_newline = false;
            }
            match c {
                ' ' | '\t' => continue,
                '\0' => tokens.push(Token::Symbol(Symbol::EndOfFile)),
                '\n' => {
                    prev_newline = true;
                    tokens.push(Token::Symbol(Symbol::NewLine));
                }
                '/' => {
                    let next = *chars.peek().unwrap_or(&&'\0');
                    let this_is_line_comment = next == &'/';
                    if this_is_line_comment {
                        let mut comment = String::from("/");
                        while let Some(c) = chars.next() {
                            if c == &'\n' {
                                break;
                            }
                            comment.push(*c);
                        }
                        tokens.push(Token::Comment(comment));
                        continue;
                    }
                    let this_is_block_comment = next == &'*';
                    if this_is_block_comment {
                        let mut comment = String::from("/");
                        let mut closed = false;
                        while let Some(c) = chars.next() {
                            if c == &'*' {
                                let next = *chars.peek().unwrap_or(&&'\0');
                                if next == &'/' {
                                    comment.push(*c);
                                    comment.push(*next);
                                    chars.next();
                                    closed = true;
                                    break;
                                }
                            }
                            comment.push(*c);
                        }
                        if closed {
                            tokens.push(Token::Comment(comment.to_string()));
                        } else {
                            return Err(CrateError::TokenizerError(
                                TokenizerError::UnterminatedBlockComment(comment),
                            ));
                        }
                        continue;
                    }
                    tokens.push(Token::Symbol(Symbol::Operator(
                        Operator::try_from(*c).unwrap(),
                    )));
                }
                c if c.is_alphabetic() => {
                    let mut identifier = String::new();
                    identifier.push(c.to_owned());
                    while let Some(c) = chars.peek() {
                        if c.is_alphabetic() || c.is_numeric() || *c == &'_' {
                            identifier.push(**c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::try_from(&identifier[..]).unwrap());
                }
                c if c.is_numeric() => {
                    let mut number = String::new();
                    number.push(*c);
                    while let Some(c) = chars.peek() {
                        if *c == &'.' {
                            number.push(**c);
                            chars.next();
                            while let Some(n) = chars.next() {
                                if n.is_numeric() {
                                    number.push(*n);
                                } else {
                                    break;
                                }
                            }
                            return Err(CrateError::TokenizerError(
                                TokenizerError::UnsupportedFloat(number),
                            ));
                        }
                        if !c.is_numeric() {
                            break;
                        }
                        number.push(**c);
                        chars.next();
                    }
                    tokens.push(Token::IntegerConstant(number.parse::<i32>().unwrap()));
                }
                c if PARENS.contains(&c.to_string().as_str()) => {
                    tokens.push(Token::Paren(Paren::try_from(*c).unwrap()));
                }
                '!' | '&' | '|' | '+' | '*' | '<' | '>' | '=' => {
                    tokens.push(Token::Symbol(Symbol::Operator(
                        Operator::try_from(*c).unwrap(),
                    )));
                }
                '-' => {
                    let next = *chars.peek().unwrap_or(&&'\0');
                    let this_is_arrow = next == &'>';
                    if this_is_arrow {
                        tokens.push(Token::Symbol(Symbol::Operator(
                            Operator::try_from("->").unwrap(),
                        )));
                        chars.next();
                        continue;
                    }
                    tokens.push(Token::Symbol(Symbol::Operator(
                        Operator::try_from(*c).unwrap(),
                    )));
                }
                ':' => {
                    let next = *chars.peek().unwrap_or(&&'\0');
                    let this_is_double_colon = next == &':';
                    if this_is_double_colon {
                        tokens.push(Token::Symbol(Symbol::Operator(
                            Operator::try_from("::").unwrap(),
                        )));
                        chars.next();
                        continue;
                    }
                    tokens.push(Token::Symbol(Symbol::Operator(
                        Operator::try_from(*c).unwrap(),
                    )));
                }
                '\\' => {
                    let mut string = String::new();
                    while let Some(c) = chars.next() {
                        if *c == '\"' {
                            break;
                        }
                        string.push(*c);
                    }
                    tokens.push(Token::StringConstant(string));
                }
                ';' => {
                    tokens.push(Token::Symbol(Symbol::Semicolon));
                }
                '"' | '\'' | '`' => {
                    in_string = true;
                    let mut string = String::new();
                    let original = *c;
                    let mut is_escaped = false;
                    while let Some(c) = chars.next() {
                        if is_escaped {
                            string.push(*c);
                            is_escaped = false;
                            continue;
                        }
                        if *c == '\\' {
                            string.push(*c);
                            is_escaped = true;
                            continue;
                        }
                        if *c == original {
                            in_string = false;
                            break;
                        }
                        string.push(*c);
                    }
                    tokens.push(Token::StringConstant(string));
                }
                _ => {
                    return Err(CrateError::TokenizerError(
                        TokenizerError::UnknownCharacter(*c),
                    ))
                }
            }
        }

        if in_string {
            let last_token = tokens.pop().unwrap();
            Err(CrateError::TokenizerError(
                TokenizerError::UnterminatedString(last_token),
            ))
        } else {
            tokens.push(Token::Symbol(Symbol::EndOfFile));
            self.tokens = tokens;
            Ok(self)
        }
    }
}

#[derive(Debug)]
pub enum TokenizerError {
    UnknownCharacter(char),
    InvalidOperator(String),
    InvalidParen(String),
    InvalidKeyword(String),
    InvalidIdentifier(String),
    UnsupportedFloat(String),
    UnterminatedString(Token),
    UnterminatedBlockComment(String),
}

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

use std::fmt::{Display, Formatter};
impl Display for CrateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO Error: {}", e),
            Self::TokenizerError(e) => match e {
                TokenizerError::UnknownCharacter(c) => write!(f, "Invalid character: {}", c),
                TokenizerError::InvalidOperator(s) => write!(f, "Invalid operator: {}", s),
                TokenizerError::InvalidParen(s) => write!(f, "Invalid paren: {}", s),
                TokenizerError::InvalidKeyword(s) => write!(f, "Invalid keyword: {}", s),
                TokenizerError::UnsupportedFloat(s) => write!(f, "Invalid integer: {}", s),
                TokenizerError::InvalidIdentifier(s) => write!(f, "Invalid identifier: {}", s),
                TokenizerError::UnterminatedBlockComment(s) => {
                    write!(f, "Block comment missing closing tag: {}", s)
                }
                TokenizerError::UnterminatedString(Token::StringConstant(s)) => {
                    write!(f, "Unterminated string: {:?}", s)
                }
                _ => write!(f, "Tokenizer Error (unknown): {:?}", e),
            },
        }
    }
}
