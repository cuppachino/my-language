use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;

mod cli;
use cli::*;

fn parse_file(path: &Path) -> io::Result<()> {
    println!("Tokenizing");
    let mut tokenizer = Tokenizer::new();
    if let Ok(lines) = read_lines(path) {
        for line in lines {
            if let Ok(ip) = line {
                tokenizer.tokenize(&ip);
            }
        }
    }
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

#[derive(Debug)]
enum Operator {
    Assignment,
    Addition,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Quote {
    Single,
    Double,
    Backtick,
}

#[derive(Debug)]
enum Symbol {
    Semi,
    Text(String),
    Unknown(String),
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

#[derive(Debug)]
enum Token {
    Symbol(Symbol),
    Number(u32),
    Operator(Operator),
    Quote(Quote),
    Paren(Paren),
    // Boolean(bool),
    // Null,
}

#[derive(Debug)]
struct Tokenizer {
    tokens: Vec<Token>,
}

impl Tokenizer {
    fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    fn tokenize(&mut self, line: &str) {
        let mut token = String::new();
        let mut in_string = false;
        let mut in_number = false;
        let mut prev_escape = false;
        let mut prev_quote: Option<Quote> = None;

        for c in line.chars() {
            if token.is_empty() {
                if c.is_ascii_whitespace() {
                    in_number = false;
                    continue;
                }
                if c.is_ascii_digit() {
                    in_number = true;
                    token.push(c);
                    continue;
                }
                if c.is_ascii_alphabetic() {
                    if !in_number {
                        token.push(c);
                        continue;
                    }
                    panic!("Invalid number: {}", token);
                }
                if c == '\\' {
                    prev_escape = true;
                    continue;
                }
                if c.is_ascii_punctuation() {
                    match c {
                        ';' => {
                            if in_string {
                                token.push(c);
                                continue;
                            }
                            if !token.is_empty() {
                                self.tokens.push(Token::Symbol(Symbol::Semi));
                                token.clear();
                            }
                            continue;
                        }
                        '"' | '\'' | '`' => {
                            if in_string {
                                if prev_escape {
                                    // in a string and the previous character escaped this one.
                                    token.push(c);
                                    prev_escape = false;
                                    continue;
                                } else {
                                    // close string
                                    self.tokens.push(Token::Symbol(Symbol::Text(
                                        token.clone().to_owned(),
                                    )));
                                    token.clear();
                                    in_string = false;
                                    continue;
                                }
                            } else {
                                in_string = true;
                                match c {
                                    '"' => {
                                        prev_quote = Some(Quote::Double);
                                        self.tokens.push(Token::Quote(Quote::Double));
                                    }

                                    '\'' => {
                                        prev_quote = Some(Quote::Single);
                                        self.tokens.push(Token::Quote(Quote::Single));
                                    }
                                    '`' => {
                                        prev_quote = Some(Quote::Backtick);
                                        self.tokens.push(Token::Quote(Quote::Backtick));
                                    }
                                    _ => panic!("Invalid quote: {}", c),
                                }
                                continue;
                            }
                            // self.tokens.push(Token::Quote(match c {
                            //     '"' => Quote::Double,
                            //     '\'' => Quote::Single,
                            //     '`' => Quote::Backtick,
                            //     _ => panic!("Invalid quote: {}", c),
                            // }));
                            // token.clear();
                            // continue;
                        }
                        '=' | ':' => {
                            self.tokens.push(Token::Operator(Operator::Assignment));
                            continue;
                        }
                        '+' | '-' | '*' | '/' => {
                            self.tokens.push(Token::Operator(match c {
                                '+' => Operator::Addition,
                                // '-' => Operator::Subtraction,
                                // '*' => Operator::Multiplication,
                                // '/' => Operator::Division,
                                _ => panic!("Algebraic operator not implemented: {}", c),
                            }));
                            continue;
                        }
                        '(' | ')' | '[' | ']' | '{' | '}' => {}
                        _ => {
                            panic!("Syntax error! Invalid symbol: {}", c);
                        }
                    }
                }
            } else {
                if c.is_ascii_whitespace() {
                    if in_string {
                        token.push(c);
                        continue;
                    }

                    if in_number {
                        self.tokens
                            .push(Token::Number(token.parse::<u32>().unwrap()));
                        token.clear();
                        in_number = false;
                        continue;
                    }

                    self.tokens
                        .push(Token::Symbol(Symbol::Unknown(token.clone().to_owned())));
                    token.clear();
                    continue;
                }

                if c.is_ascii_digit() {
                    if in_string {
                        token.push(c);
                        continue;
                    }

                    if !in_number {
                        panic!("Invalid number: {}", token);
                    }

                    token.push(c);
                    continue;
                }

                if c.is_ascii_alphabetic() {
                    token.push(c);
                    continue;
                }

                if c.is_ascii_punctuation() {
                    if c == '\\' {
                        prev_escape = true;
                        continue;
                    }
                    match c {
                        ';' => {
                            if in_string {
                                token.push(c);
                                continue;
                            }
                            if in_number {
                                self.tokens
                                    .push(Token::Number(token.parse::<u32>().unwrap()));
                                token.clear();
                                in_number = false;
                                self.tokens.push(Token::Symbol(Symbol::Semi));
                                continue;
                            }
                            if !token.is_empty() {
                                self.tokens
                                    .push(Token::Symbol(Symbol::Unknown(token.clone().to_owned())));
                                token.clear();
                                self.tokens.push(Token::Symbol(Symbol::Semi));
                            }
                            continue;
                        }
                        '"' | '\'' | '`' => {
                            if in_string {
                                // in a string and the previous character escaped this one.
                                if prev_escape {
                                    token.push(c);
                                    prev_escape = false;
                                    continue;
                                }

                                if let Some(quote) = prev_quote {
                                    if quote
                                        == match c {
                                            '"' => Quote::Double,
                                            '\'' => Quote::Single,
                                            '`' => Quote::Backtick,
                                            _ => panic!("Invalid quote: {}", c),
                                        }
                                    {
                                        // close string
                                        self.tokens.push(Token::Symbol(Symbol::Text(
                                            token.clone().to_owned(),
                                        )));
                                        token.clear();
                                        in_string = false;
                                        self.tokens.push(Token::Quote(quote));
                                        continue;
                                    }
                                }

                                // close string
                                self.tokens
                                    .push(Token::Symbol(Symbol::Text(token.clone().to_owned())));
                                token.clear();
                                in_string = false;
                                continue;
                            }

                            // not in a string and the token is not empty
                            panic!("Syntax error! Invalid symbol: {}", c);
                        }
                        '=' | ':' => {
                            if in_string {
                                token.push(c);
                                continue;
                            } else {
                                self.tokens
                                    .push(Token::Symbol(Symbol::Unknown(token.clone().to_owned())));
                                token.clear();
                            }
                            self.tokens.push(Token::Operator(Operator::Assignment));
                            continue;
                        }
                        '+' | '-' | '*' | '/' => {
                            self.tokens.push(Token::Operator(match c {
                                '+' => Operator::Addition,
                                // '-' => Operator::Subtraction,
                                // '*' => Operator::Multiplication,
                                // '/' => Operator::Division,
                                _ => panic!("Algebraic operator not implemented: {}", c),
                            }));
                            continue;
                        }
                        '(' | ')' | '[' | ']' | '{' | '}' => {
                            if in_string {
                                token.push(c);
                                continue;
                            } else {
                                self.tokens
                                    .push(Token::Symbol(Symbol::Unknown(token.clone().to_owned())));
                                token.clear();
                            }
                            match c {
                                '(' => {
                                    self.tokens
                                        .push(Token::Paren(Paren::Open(ParenType::Round)));
                                }
                                ')' => {
                                    self.tokens
                                        .push(Token::Paren(Paren::Close(ParenType::Round)));
                                }
                                '[' => {
                                    self.tokens
                                        .push(Token::Paren(Paren::Open(ParenType::Square)));
                                }
                                ']' => {
                                    self.tokens
                                        .push(Token::Paren(Paren::Close(ParenType::Square)));
                                }
                                '{' => {
                                    self.tokens
                                        .push(Token::Paren(Paren::Open(ParenType::Curly)));
                                }
                                '}' => {
                                    self.tokens
                                        .push(Token::Paren(Paren::Close(ParenType::Curly)));
                                }
                                _ => panic!("Invalid paren: {}", c),
                            }
                            continue;
                        }
                        _ => {
                            panic!("Syntax error! Invalid symbol: {}", c);
                        }
                    }
                }
            }
        }

        if in_string {
            panic!("Syntax error! Unclosed string: {}", token);
        }

        if !token.is_empty() {
            if in_number {
                self.tokens
                    .push(Token::Number(token.parse::<u32>().unwrap()));
            } else {
                self.tokens
                    .push(Token::Symbol(Symbol::Unknown(token.clone().to_owned())));
            }
        }
    }
}
