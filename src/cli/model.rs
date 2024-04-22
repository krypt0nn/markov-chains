use std::path::PathBuf;
use std::io::Write;

use clap::Subcommand;

use crate::prelude::{
    Messages,
    Tokens,
    TokenizedMessages,
    Dataset,
    GenerationParams,
    Model
};

#[derive(Subcommand)]
pub enum CliModelCommand {
    /// Build language model
    Build {
        #[arg(short, long)]
        /// Path to the dataset bundle
        dataset: PathBuf,

        #[arg(short, long)]
        /// Path to the model output
        output: PathBuf
    },

    /// Build language model from plain messages files
    FromScratch {
        #[arg(short, long)]
        /// Path to the plain messages file
        messages: Vec<PathBuf>,

        #[arg(short, long)]
        /// Path to the model output
        output: PathBuf
    },

    /// Load language model
    Load {
        #[arg(short, long)]
        /// Path to the model
        model: PathBuf,

        #[command(flatten)]
        params: GenerationParams
    }
}

impl CliModelCommand {
    #[inline]
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Self::Build { dataset, output } => {
                println!("Reading dataset bundle...");

                let messages = postcard::from_bytes::<Dataset>(&std::fs::read(dataset)?)?;

                println!("Building model...");

                let model = Model::build(messages);

                println!("Storing model...");

                std::fs::write(output, postcard::to_allocvec(&model)?)?;

                println!("Done");
            }

            Self::FromScratch { messages: paths, output } => {
                println!("Parsing messages...");

                let mut messages = Messages::default();

                for path in paths {
                    println!("Parsing {:?}...", path);

                    let parsed = Messages::parse_from_messages(path)?;

                    messages = messages.merge(parsed);
                }

                println!("Generating tokens...");

                let tokens = Tokens::parse_from_messages(&messages);

                println!("Tokenizing messages...");

                let tokenized_messages = TokenizedMessages::tokenize_message(&messages, &tokens)?;

                println!("Creating dataset...");

                let dataset = Dataset::default()
                    .with_messages(tokenized_messages, 1)
                    .with_tokens(tokens);

                println!("Building model...");

                let model = Model::build(dataset);

                println!("Storing model...");

                std::fs::write(output, postcard::to_allocvec(&model)?)?;

                println!("Done");
            }

            Self::Load { model, params } => {
                println!("Reading model...");

                let model = postcard::from_bytes::<Model>(&std::fs::read(model)?)?;

                println!("Starting model...");

                let stdin = std::io::stdin();
                let mut stdout = std::io::stdout();

                println!();
                println!("  Model loaded:");
                println!("        Tokens: {}", model.tokens.len());
                println!("        Chains: {}", model.chains.len());
                println!("    Complexity: {}", model.chains.calculate_complexity());
                println!();

                loop {
                    let mut request = String::new();

                    stdout.write_all(b"> ")?;
                    stdout.flush()?;

                    stdin.read_line(&mut request)?;

                    let request = request.split_whitespace()
                        .filter(|word| !word.is_empty())
                        .map(|word| word.to_lowercase())
                        .map(|word| model.tokens.find_token(word))
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
                        stdout.write_all(model.tokens.find_word(*token).unwrap().as_bytes())?;
                        stdout.write_all(b" ")?;
                        stdout.flush()?;
                    }

                    for token in model.generate(request.clone(), params) {
                        match token {
                            Ok(token) => {
                                let Some(word) = model.tokens.find_word(token) else {
                                    print!("\n\n  Failed to find word for token: {token}");

                                    break;
                                };

                                stdout.write_all(word.as_bytes())?;
                                stdout.write_all(b" ")?;
                                stdout.flush()?;
                            }

                            Err(err) => {
                                print!("\n\n  Failed to generate: {err}");

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
