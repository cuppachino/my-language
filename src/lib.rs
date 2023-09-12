use std::{
    fs::File,
    io::{BufReader, Lines},
};

pub fn into_chars(lines: Lines<BufReader<File>>) -> Vec<char> {
    let mut chars = Vec::new();
    for line in lines {
        for c in line.unwrap().chars() {
            chars.push(c);
        }
        chars.push('\n');
    }
    chars
}
