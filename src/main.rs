mod command;
mod cli;

use clap::Parser;
use cli::{ Args, CLICommand };

use anyhow::Result;

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        CLICommand::Init => {
            command::init()?;
        }

        CLICommand::CatFile {
            object_hash,
            pretty_print
        } => {
            command::cat_file(object_hash, pretty_print)?;
        }

        CLICommand::HashObject {
            write,
            filename
        } => {
            command::hash_object(filename, write)?;
        }
    }

    Ok(())
}
