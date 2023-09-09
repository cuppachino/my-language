use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::path::Path;
use std::process;

mod cli;
use cli::*;

fn parse_file(path: &Path) -> io::Result<()> {
    println!("Tokenizing");
    let mut tokenizer = Tokenizer::default();
    let lines = read_lines(path)?;
    tokenizer.tokenize(lines);
    println!("{:#?}", tokenizer.tokens);
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build(command) => {
            println!(
                "Compiling {:?} into {:?}",
                command.input_file, command.output_dir
            );

            let input_file = Path::new(&command.input_file);

            parse_file(&input_file)
                .map_err(|e| {
                    println!("Error parsing file: {:?}", e);
                    process::exit(1);
                })
                .unwrap();

            // let output_dir = Path::new(&command.output_dir);
            // prepare_out_dir(&output_dir).unwrap();
        }
    }
}

#[allow(dead_code)]
fn prepare_out_dir(path: &Path) -> io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(std::io::BufReader::new(file).lines())
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Quote {
    Single,
    Double,
    Backtick,
}

impl TryFrom<char> for Quote {
    type Error = &'static str;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '\'' => Ok(Quote::Single),
            '"' => Ok(Quote::Double),
            '`' => Ok(Quote::Backtick),
            _ => Err("Invalid quote character. Must be one of: ' \" `"),
        }
    }
}

#[derive(Debug)]
enum ParenType {
    Round,
    Square,
    Curly,
}

#[derive(Debug)]
enum Paren {
    Open(ParenType),
    Close(ParenType),
}

impl From<char> for Paren {
    fn from(c: char) -> Self {
        match c {
            '(' => Paren::Open(ParenType::Round),
            ')' => Paren::Close(ParenType::Round),
            '[' => Paren::Open(ParenType::Square),
            ']' => Paren::Close(ParenType::Square),
            '{' => Paren::Open(ParenType::Curly),
            '}' => Paren::Close(ParenType::Curly),
            _ => panic!("Invalid paren character: {}", c),
        }
    }
}

#[derive(Debug)]
enum Operator {
    Declaration,
    Assertion,
    Assignment,
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

#[derive(Debug)]
enum Symbol {
    Semi,
    Text(String),
    Number(u32),
    Keyword(String),
    Unknown(String),
}

#[derive(Debug)]
enum Token {
    Symbol(Symbol),
    Operator(Operator),
    Quote(Quote),
    Paren(Paren),
}

impl From<&str> for Token {
    fn from(s: &str) -> Self {
        match s {
            ";" => Token::Symbol(Symbol::Semi),
            "let" | "fn" => Token::Symbol(Symbol::Keyword(s.to_owned())),
            "::" => Token::Operator(Operator::Declaration),
            "->" => Token::Operator(Operator::Assertion),
            "=" => Token::Operator(Operator::Assignment),
            "+" => Token::Operator(Operator::Addition),
            "-" => Token::Operator(Operator::Subtraction),
            "*" => Token::Operator(Operator::Multiplication),
            "/" => Token::Operator(Operator::Division),
            _ => {
                if let Ok(n) = s.parse::<u32>() {
                    Token::Symbol(Symbol::Number(n))
                } else if s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    Token::Symbol(Symbol::Text(s.to_owned()))
                } else {
                    panic!("Invalid token: {}", s);
                }
            }
        }
    }
}

#[derive(Debug, Default)]
struct Tokenizer {
    tokens: Vec<Token>,
}

impl Tokenizer {
    fn tokenize(&mut self, lines: Lines<BufReader<File>>) {
        let mut token = String::new();
        let mut in_number = false;
        let mut prev_quote: Option<Quote> = None;
        let mut prev_escaped = false;
        let mut operator_buffer = String::new();

        for line in lines {
            if let Ok(ip) = line {
                let mut iter = ip.chars().peekable();

                while let Some(c) = iter.next() {
                    // WHITESPACE
                    if c.is_ascii_whitespace() {
                        if prev_quote.is_some() {
                            token.push(c);
                        } else if !token.is_empty() {
                            in_number = false;
                            self.tokens.push(Token::from(token.as_str()));
                            token.clear();
                        }
                        if !operator_buffer.is_empty() {
                            self.tokens.push(Token::from(operator_buffer.as_str()));
                            operator_buffer.clear();
                        }
                        continue;
                    }

                    if prev_escaped {
                        token.push(c);
                        prev_escaped = false;
                        continue;
                    }

                    // NUMBERS
                    if c.is_ascii_digit() {
                        token.push(c);
                        continue;
                    }
                    // ALPHABETIC
                    if c.is_ascii_alphabetic() {
                        if in_number {
                            panic!("Invalid ascii digit: {}", token);
                        }
                        token.push(c);
                        continue;
                    }
                    // PUNCTUATION
                    if c.is_ascii_punctuation() {
                        match c {
                            ';' => {
                                if prev_quote.is_some() {
                                    token.push(c);
                                } else if !token.is_empty() {
                                    self.tokens.push(Token::from(token.as_str()));
                                    token.clear();
                                }
                                self.tokens.push(Token::Symbol(Symbol::Semi));
                                continue;
                            }
                            '_' => {
                                if in_number {
                                    panic!("Invalid number: {}", token);
                                } else {
                                    token.push(c);
                                    continue;
                                }
                            }
                            '\\' => {
                                if prev_quote.is_some() {
                                    token.push(c);
                                } else if !token.is_empty() {
                                    panic!("Invalid escape character outside of string: {}", token);
                                }
                                prev_escaped = true;
                                continue;
                            }
                            '\'' | '\"' | '`' => {
                                let quote = Quote::try_from(c).ok().unwrap();
                                if let Some(previous) = prev_quote {
                                    if previous == quote {
                                        // close string
                                        self.tokens.push(Token::Symbol(Symbol::Text(
                                            token.clone().to_owned(),
                                        )));
                                        self.tokens.push(Token::Quote(quote));
                                        token.clear();
                                        prev_quote = None;
                                        continue;
                                    } else {
                                        token.push(c);
                                        continue;
                                    }
                                } else {
                                    prev_quote = Some(quote);
                                    self.tokens.push(Token::Quote(quote));
                                    continue;
                                }
                            }
                            '+' | '-' | '*' | '/' | '=' | ':' | '>' => {
                                if prev_quote.is_some() {
                                    token.push(c);
                                } else if !token.is_empty() {
                                    self.tokens.push(Token::from(token.as_str()));
                                    token.clear();
                                }
                                operator_buffer.push(c);
                                continue;
                            }
                            '(' | ')' | '[' | ']' | '{' | '}' => {
                                if prev_quote.is_some() {
                                    token.push(c);
                                } else if !token.is_empty() {
                                    self.tokens.push(Token::from(token.as_str()));
                                    token.clear();
                                }
                                self.tokens.push(Token::Paren(Paren::from(c)));
                                continue;
                            }
                            _ => {
                                if prev_quote.is_some() {
                                    token.push(c);
                                } else {
                                    panic!("Invalid punctuation: {}", c)
                                }
                            }
                        }
                    }
                }

                if prev_quote.is_none() && !token.is_empty() {
                    self.tokens.push(Token::from(token.as_str()));
                } else if prev_quote.is_some() {
                    panic!("Unclosed string, expected quote but got: {}", token);
                }
            }
        }
    }
}
