use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use miette::{Context, IntoDiagnostic};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run { filename: PathBuf }, //cargo run -- run "test.txt" //TODO: make optional so it just read current workdir
}

fn main() -> miette::Result<()> {
    println!("Hello, world!");

    let args = Args::parse();

    match args.command {
        Commands::Run { filename } => {
            let _file_contents = fs::read_to_string(&filename)
                .into_diagnostic()
                .wrap_err_with(|| format!("unable to read `{}`", &filename.display()))?;
            todo!()
        }
    }

    Ok(())
}
