use clap::{Parser, Subcommand};

mod messages;
mod tokens;
mod model;

use messages::CliMessagesCommand;
use tokens::CliTokensCommand;
use model::CliModelCommand;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands
}

impl Cli {
    #[inline]
    pub fn execute(&self) -> anyhow::Result<()> {
        self.command.execute()
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Work with messages
    Messages {
        #[command(subcommand)]
        action: CliMessagesCommand
    },

    /// Work with tokens
    Tokens {
        #[command(subcommand)]
        action: CliTokensCommand
    },

    Model {
        #[command(subcommand)]
        action: CliModelCommand
    }
}

impl Commands {
    #[inline]
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Self::Messages { action } => action.execute(),
            Self::Tokens { action } => action.execute(),
            Self::Model { action } => action.execute()
        }
    }
}
