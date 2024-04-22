use std::path::PathBuf;

use clap::Subcommand;

use crate::prelude::{
    Messages,
    Tokens
};

#[derive(Subcommand)]
pub enum CliTokensCommand {
    /// Parse tokens from a messages bundle
    Parse {
        #[arg(short, long)]
        /// Path to the messages bundle
        path: PathBuf,

        #[arg(short, long)]
        /// Path to the tokens output
        output: PathBuf
    }
}

impl CliTokensCommand {
    #[inline]
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Self::Parse { path, output } => {
                println!("Reading messages bundle...");

                let messages = postcard::from_bytes::<Messages>(&std::fs::read(path)?)?;

                println!("Generating tokens...");

                let tokens = Tokens::parse_from_messages(&messages);

                println!("Storing tokens bundle...");

                std::fs::write(output, postcard::to_allocvec(&tokens)?)?;

                println!("Done");
            }
        }

        Ok(())
    }
}
