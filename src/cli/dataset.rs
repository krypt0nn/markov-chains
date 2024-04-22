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
    },

    /// Check the word appearance in the dataset
    CheckWord {
        #[arg(short, long)]
        /// Path to the dataset bundle
        path: PathBuf,

        #[arg(short, long)]
        /// Word to check
        word: String
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

            Self::CheckWord { path, word } => {
                println!("Reading dataset bundle...");

                let dataset = postcard::from_bytes::<Dataset>(&std::fs::read(path)?)?;

                println!("Checking word appearance...");

                let Some(token) = dataset.tokens().find_token(word) else {
                    anyhow::bail!("Could not find token for word: {word}");
                };

                let mut distinct_num = 0;
                let mut total_num = 0;
                let mut importance = 0;

                let mut total_messages = 0;

                for (message, weight) in dataset.messages() {
                    for message in message.messages() {
                        let num = message.iter().filter(|t| *t == &token).count() as u64;

                        distinct_num += if num > 0 { 1 } else { 0 };
                        total_num += num;

                        importance += num * *weight;

                        total_messages += 1;
                    }
                }

                println!();
                println!("Distinct num: {distinct_num}");
                println!("   Total num: {total_num}");
                println!("  Importance: {importance}");
                println!("   Frequency: {:.5}%", distinct_num as f64 / total_messages as f64 * 100.0);
            }
        }

        Ok(())
    }
}
