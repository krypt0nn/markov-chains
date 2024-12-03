use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod parser;
// mod tokens;
// mod dataset;
// mod model;

use parser::CliMessagesCommand;
// use tokens::CliTokensCommand;
// use dataset::CliDatasetCommand;
// use model::CliModelCommand;

pub fn search_files(paths: impl IntoIterator<Item = impl Into<PathBuf>>) -> Vec<PathBuf> {
    let mut files = Vec::new();

    let mut paths = paths.into_iter()
        .map(|path| path.into())
        .collect::<Vec<PathBuf>>();

    while let Some(path) = paths.pop() {
        if path.is_file() {
            files.push(path);
        }

        else if path.is_dir() {
            if let Ok(dir_paths) = path.read_dir() {
                let dir_paths = dir_paths.flatten()
                    .map(|path| path.path())
                    .collect::<Vec<_>>();

                paths.extend(dir_paths);
            }
        }
    }

    files
}

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
    /// Parse input files into separate messages
    Parser {
        #[command(subcommand)]
        action: CliParserCommand
    }
}

impl Commands {
    #[inline]
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Self::Parser { action } => action.execute(),
            // Self::Tokens { action } => action.execute(),
            // Self::Dataset { action } => action.execute(),
            // Self::Model { action } => action.execute()
        }
    }
}
