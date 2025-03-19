mod cli;
mod command;

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

        CLICommand::LsTree {
            name_only,
            tree_hash
        } => {
            command::ls_tree(tree_hash, name_only)?;
        }
    }

    Ok(())
}
