use std::path::PathBuf;

use clap::Subcommand;

use crate::prelude::{
    Messages,
    Tokens
};

use super::search_files;

#[derive(Subcommand)]
pub enum CliTokensCommand {
    /// Parse tokens from a messages bundle
    Parse {
        #[arg(short, long)]
        /// Path to the messages bundle
        path: Vec<PathBuf>,

        #[arg(short, long)]
        /// Path to the tokens output
        output: PathBuf
    },

    /// Merge tokens bundles
    Merge {
        #[arg(short, long)]
        /// Path to the tokens bundle
        path: Vec<PathBuf>,

        #[arg(short, long)]
        /// Path to the merged tokens output
        output: PathBuf
    }
}

impl CliTokensCommand {
    #[inline]
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Self::Parse { path, output } => {
                println!("Reading messages bundles...");

                let mut messages = Messages::default();

                for path in search_files(path) {
                    println!("Reading {:?}...", path);

                    messages = messages.merge(postcard::from_bytes::<Messages>(&std::fs::read(path)?)?);
                }

                println!("Generating tokens...");

                let tokens = Tokens::parse_from_messages(&messages);

                println!("Storing tokens bundle...");

                std::fs::write(output, postcard::to_allocvec(&tokens)?)?;

                println!("Done");
            }

            Self::Merge { path, output } => {
                println!("Reading tokens bundles...");

                let mut tokens = Tokens::default();

                for path in search_files(path) {
                    println!("Reading {:?}...", path);

                    tokens = tokens.merge(postcard::from_bytes::<Tokens>(&std::fs::read(path)?)?);
                }

                println!("Storing merged tokens bundle...");

                std::fs::write(output, postcard::to_allocvec(&tokens)?)?;

                println!("Done");
            }
        }

        Ok(())
    }
}
