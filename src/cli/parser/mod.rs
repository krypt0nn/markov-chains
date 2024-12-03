use std::path::PathBuf;

use clap::Subcommand;

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
