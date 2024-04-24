use std::path::PathBuf;

use clap::Subcommand;

use crate::prelude::{
    Messages,
    Tokens,
    TokenizedMessages
};

use super::search_files;

#[derive(Subcommand)]
pub enum CliMessagesCommand {
    /// Parse messages from a file to a bundle
    Parse {
        #[arg(short, long)]
        /// Paths to the messages list
        path: Vec<PathBuf>,

        #[arg(short, long)]
        /// Path to the bundle output
        output: PathBuf
    },

    /// Merge different messages bundles into a single file
    Merge {
        #[arg(short, long)]
        /// Paths to the messages bundles
        path: Vec<PathBuf>,

        #[arg(short, long)]
        /// Path to the merged messages bundle
        output: PathBuf
    },

    /// Tokenize messages bundle
    Tokenize {
        #[arg(short, long)]
        /// Path to the messages bundle
        messages: PathBuf,

        #[arg(short, long)]
        /// Path to the tokens bundle
        tokens: PathBuf,

        #[arg(short, long)]
        /// Path to the tokenized messages bundle
        output: PathBuf
    }
}

impl CliMessagesCommand {
    #[inline]
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Self::Parse { path, output } => {
                let mut messages = Messages::default();

                println!("Parsing messages...");

                for path in search_files(path) {
                    println!("Parsing {:?}...", path);

                    messages = messages.merge(Messages::parse_from_messages(path)?);
                }

                println!("Storing messages bundle...");

                std::fs::write(output, postcard::to_allocvec(&messages)?)?;

                println!("Done");
            }

            Self::Merge { path, output } => {
                let mut messages = Messages::default();

                println!("Reading messages bundles...");

                for path in search_files(path) {
                    println!("Reading {:?}...", path);

                    let bundle = postcard::from_bytes::<Messages>(&std::fs::read(path)?)?;

                    messages = messages.merge(bundle);
                }

                println!("Storing merged messages bundle...");

                std::fs::write(output, postcard::to_allocvec(&messages)?)?;

                println!("Done");
            }

            Self::Tokenize { messages, tokens, output } => {
                println!("Reading messages bundle...");

                let messages = postcard::from_bytes::<Messages>(&std::fs::read(messages)?)?;

                println!("Reading tokens bundle...");
                
                let tokens = postcard::from_bytes::<Tokens>(&std::fs::read(tokens)?)?;

                println!("Tokenizing messages...");

                let tokenized = TokenizedMessages::tokenize_message(&messages, &tokens)?;

                println!("Storing tokenized messages bundle...");

                std::fs::write(output, postcard::to_allocvec(&tokenized)?)?;

                println!("Done");
            }
        }

        Ok(())
    }
}
