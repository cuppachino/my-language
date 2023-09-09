use clap::Args;
pub use clap::{Parser, Subcommand, ValueEnum};

/// Simple program to greet a person
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Whether to actually run the command or just print what would happen.
    #[arg(value_enum, default_value_t = Mode::DryRun)]
    pub mode: Mode,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Compile a project.
    Build(BuildCommand),
}

#[derive(Args, Debug)]
pub struct BuildCommand {
    /// The name of the project.
    #[arg(short, long)]
    pub input_file: String,

    /// The output directory.
    #[arg(short, long)]
    pub output_dir: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Mode {
    /// Just print what would happen.
    DryRun,
    /// Potentially alter the filesystem.
    Run,
}
