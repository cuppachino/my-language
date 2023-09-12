use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;

mod cli;
mod tokenizer;
use cli::*;
use tokenizer::CrateError;

use crate::tokenizer::Tokenizer;

fn parse_file(path: &Path) -> Result<(), CrateError> {
    println!("Tokenizing");
    let mut tokenizer = Tokenizer::new(read_lines(path)?);
    let tokenizer = tokenizer.tokenize()?;
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
                    println!("Error parsing file: {}", e);
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
