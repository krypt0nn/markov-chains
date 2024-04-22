use std::path::PathBuf;
use std::io::Write;

use clap::Subcommand;

use crate::prelude::{
    Tokens,
    TokenizedMessages,
    Model,
    GenerationParams
};

#[derive(Subcommand)]
pub enum CliModelCommand {
    /// Build language model
    Build {
        #[arg(short, long)]
        /// Path to the tokenized messages bundle
        messages: PathBuf,

        #[arg(short, long)]
        /// Path to the model output
        output: PathBuf
    },

    /// Load language model
    Load {
        #[arg(short, long)]
        /// Path to the model
        model: PathBuf,

        #[arg(short, long)]
        /// Path to the tokens bundle
        tokens: PathBuf,

        #[command(flatten)]
        params: GenerationParams
    }
}

impl CliModelCommand {
    #[inline]
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Self::Build { messages, output } => {
                println!("Reading tokenized messages bundle...");

                let messages = postcard::from_bytes::<TokenizedMessages>(&std::fs::read(messages)?)?;

                println!("Building model...");

                let model = Model::build(messages);

                println!("Storing model...");

                std::fs::write(output, postcard::to_allocvec(&model)?)?;

                println!("Done");
            }

            Self::Load { model, tokens, params } => {
                println!("Reading model...");

                let model = postcard::from_bytes::<Model>(&std::fs::read(model)?)?;

                println!("Reading tokens bundle...");

                let tokens = postcard::from_bytes::<Tokens>(&std::fs::read(tokens)?)?;

                println!("Starting model...");

                let input_prefix = format!("complexity: {} > ", model.complexity());

                let stdin = std::io::stdin();
                let mut stdout = std::io::stdout();

                stdout.write_all(b"\n")?;
                stdout.flush()?;

                loop {
                    let mut request = String::new();

                    stdout.write_all(input_prefix.as_bytes())?;
                    stdout.flush()?;

                    stdin.read_line(&mut request)?;

                    let request = request.split_whitespace()
                        .filter(|word| !word.is_empty())
                        .map(|word| word.to_lowercase())
                        .map(|word| tokens.find_token(word))
                        .collect::<Option<Vec<_>>>();

                    let Some(request) = request else {
                        continue;
                    };

                    if request.is_empty() {
                        continue;
                    }

                    stdout.write_all(b"\n  model: ")?;
                    stdout.flush()?;

                    for token in &request {
                        stdout.write_all(tokens.find_word(*token).unwrap().as_bytes())?;
                        stdout.write_all(b" ")?;
                        stdout.flush()?;
                    }

                    for token in model.generate(request.clone(), params) {
                        match token {
                            Ok(token) => {
                                let Some(word) = tokens.find_word(token) else {
                                    println!("\n  Failed to find word for token: {token}");

                                    break;
                                };

                                stdout.write_all(word.as_bytes())?;
                                stdout.write_all(b" ")?;
                                stdout.flush()?;
                            }

                            Err(err) => {
                                println!("\n  Failed to generate: {err}");

                                break;
                            }
                        }
                    }

                    stdout.write_all(b"\n\n")?;
                    stdout.flush()?;
                }
            }
        }

        Ok(())
    }
}
