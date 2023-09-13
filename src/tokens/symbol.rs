use super::Operator;

#[derive(Debug)]
pub enum Symbol {
    #[allow(dead_code)]
    Comma,
    Operator(Operator),
    Semicolon,
    NewLine,
    EndOfFile,
}
