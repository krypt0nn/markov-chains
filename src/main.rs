use clap::Parser;

pub mod messages;
pub mod tokens;
pub mod tokenized_messages;
pub mod dataset;
pub mod model;

pub mod cli;

pub mod prelude {
    pub use super::messages::Messages;
    pub use super::tokens::Tokens;
    pub use super::tokenized_messages::TokenizedMessages;
    pub use super::dataset::Dataset;
    pub use super::model::params::GenerationParams;
    pub use super::model::transitions::Transitions;
    pub use super::model::generator::TokenGenerator;
    pub use super::model::model::Model;
}

fn main() -> anyhow::Result<()> {
    cli::Cli::parse().execute()
}
