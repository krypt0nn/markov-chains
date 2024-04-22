use std::path::PathBuf;

use clap::Subcommand;

use crate::prelude::{
    TokenizedMessages,
    Tokens,
    Dataset
};

#[derive(Subcommand)]
pub enum CliDatasetCommand {
    /// Create dataset from the tokenized messages and tokens bundle
    Create {
        #[arg(short, long)]
        /// Path to the messages bundle
        messages: PathBuf,

        #[arg(short, long)]
        /// Path to the tokens bundle
        tokens: PathBuf,

        #[arg(short, long, default_value_t = 1)]
        /// Messages weight in the dataset
        weight: u64,

        #[arg(short, long)]
        /// Path to the dataset output
        output: PathBuf
    },

    /// Extend existing dataset with the tokenized messages
    AddMessages {
        #[arg(short, long)]
        /// Path to the dataset bundle
        path: PathBuf,

        #[arg(short, long)]
        /// Path to the messages bundle
        messages: Vec<PathBuf>,

        #[arg(short, long, default_value_t = 1)]
        /// Messages weight
        weight: u64,

        #[arg(short, long)]
        /// Path to the dataset output
        output: PathBuf
    },

    /// Extend existing dataset with the tokenized messages
    AddTokens {
        #[arg(short, long)]
        /// Path to the dataset bundle
        path: PathBuf,

        #[arg(short, long)]
        /// Path to the tokens bundle
        tokens: Vec<PathBuf>,

        #[arg(short, long)]
        /// Path to the dataset output
        output: PathBuf
    }
}

impl CliDatasetCommand {
    #[inline]
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Self::Create { messages, tokens, weight, output } => {
                println!("Reading tokenized messages bundle...");

                let tokenized_messages = postcard::from_bytes::<TokenizedMessages>(&std::fs::read(messages)?)?;

                println!("Reading tokens bundle...");

                let tokens = postcard::from_bytes::<Tokens>(&std::fs::read(tokens)?)?;

                println!("Creating dataset...");

                let dataset = Dataset::default()
                    .with_messages(tokenized_messages, *weight)
                    .with_tokens(tokens);

                println!("Storing dataset bundle...");

                std::fs::write(output, postcard::to_allocvec(&dataset)?)?;

                println!("Done");
            }

            Self::AddMessages { path, messages, weight, output } => {
                println!("Reading dataset bundle...");

                let mut dataset = postcard::from_bytes::<Dataset>(&std::fs::read(path)?)?;

                println!("Reading tokenized messages bundles...");

                for path in messages {
                    println!("Reading {:?}...", path);

                    let tokenized_messages = postcard::from_bytes::<TokenizedMessages>(&std::fs::read(path)?)?;

                    dataset = dataset.with_messages(tokenized_messages, *weight);
                }

                println!("Storing dataset bundle...");

                std::fs::write(output, postcard::to_allocvec(&dataset)?)?;

                println!("Done");
            }

            Self::AddTokens { path, tokens, output } => {
                println!("Reading dataset bundle...");

                let mut dataset = postcard::from_bytes::<Dataset>(&std::fs::read(path)?)?;

                println!("Reading tokens bundles...");

                for path in tokens {
                    println!("Reading {:?}...", path);

                    let tokens = postcard::from_bytes::<Tokens>(&std::fs::read(path)?)?;

                    dataset = dataset.with_tokens(tokens);
                }

                println!("Storing dataset bundle...");

                std::fs::write(output, postcard::to_allocvec(&dataset)?)?;

                println!("Done");
            }
        }

        Ok(())
    }
}
