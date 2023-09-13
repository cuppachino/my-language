use crate::{
    error::CrateError,
    tokens::{Operator, Paren, Symbol, Token},
};
use std::{
    fs::File,
    io::{BufReader, Lines},
};

#[derive(Debug, Default)]
pub struct Tokenizer {
    chars: Vec<char>,
    pub tokens: Vec<Token>,
}

impl Tokenizer {
    fn into_chars(lines: Lines<BufReader<File>>) -> Vec<char> {
        let mut chars = Vec::new();
        for line in lines {
            for c in line.unwrap().chars() {
                chars.push(c);
            }
            chars.push('\n');
        }
        chars
    }

    pub fn new(lines: Lines<BufReader<File>>) -> Self {
        Self {
            chars: Tokenizer::into_chars(lines),
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
            }
            prev_newline = false;
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
                '(' | ')' | '[' | ']' | '{' | '}' => {
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
    UnknownOperator(String),
    UnknownParen(char),
    InvalidKeyword(String),
    InvalidIdentifier(String),
    UnsupportedFloat(String),
    UnterminatedString(Token),
    UnterminatedBlockComment(String),
}

impl std::fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenizerError::UnknownCharacter(c) => write!(f, "Invalid character: {}", c),
            TokenizerError::UnknownOperator(s) => write!(f, "Unknown operator token: {}", s),
            TokenizerError::UnknownParen(c) => write!(f, "Unknown paren character: {}", c),
            TokenizerError::InvalidKeyword(s) => write!(f, "Invalid keyword: {}", s),
            TokenizerError::InvalidIdentifier(s) => write!(f, "Invalid identifier: {}", s),
            TokenizerError::UnsupportedFloat(s) => write!(f, "Invalid integer: {}", s),
            TokenizerError::UnterminatedString(Token::StringConstant(s)) => {
                write!(f, "Unterminated string: {:?}", s)
            }
            TokenizerError::UnterminatedBlockComment(s) => {
                write!(f, "Block comment missing closing tag: {}", s)
            }
            _ => write!(f, "Unknown lexing error"),
        }
    }
}
