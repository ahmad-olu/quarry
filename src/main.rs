use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use miette::{Context, IntoDiagnostic};
use quarry::lexer::Lexer;

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
    //println!("Hello, world!");

    //let args = Args::parse();

    // match args.command {
    //     Commands::Run { filename } => {
    //         let _file_contents = fs::read_to_string(&filename)
    //             .into_diagnostic()
    //             .wrap_err_with(|| format!("unable to read `{}`", &filename.display()))?;
    //         todo!()
    //     }
    // };

    // for token in Lexer::new(&file_contents) {}

    for token in Lexer::new(
        "
        name updateUser
        put '${base}/users/42 /${ace}'
        json {
                \"name\": \"Jane Doe\",
                \"email\": \"<jane@example.com>\"
            }
        32.44 aa
        -12.4
        ",
    ) {
        let token = match token {
            Ok(t) => t,
            Err(e) => {
                // eprintln!("===>{:?}", e.to_string());
                return Err(e);
                //continue;
            }
        };
        println!("{:?}", token);
    }

    Ok(())
}

// "

//     //hello
//     !={}:{}!=   {}===
//     ",

//false nil true let import json multipart raw form save assert matches in contains test ws graphql
