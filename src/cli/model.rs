use std::path::PathBuf;
use std::io::Write;

use clap::Subcommand;

use crate::prelude::{
    Messages,
    Tokens,
    TokenizedMessages,
    Ngram,
    Dataset,
    GenerationParams,
    Model
};

const DEFAULT_NGRAM_SIZE: usize = 2;

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
    },

    /// Check the word appearance in the model
    CheckWord {
        #[arg(short, long)]
        /// Path to the model
        path: PathBuf,

        #[arg(short, long)]
        /// Word to check
        word: String
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

                let model = Model::<DEFAULT_NGRAM_SIZE>::build(messages);

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

                let model = Model::<DEFAULT_NGRAM_SIZE>::build(dataset);

                println!("Storing model...");

                std::fs::write(output, postcard::to_allocvec(&model)?)?;

                println!("Done");
            }

            Self::Load { model, params } => {
                println!("Reading model...");

                let model = postcard::from_bytes::<Model<DEFAULT_NGRAM_SIZE>>(&std::fs::read(model)?)?;

                println!("Starting model...");

                let stdin = std::io::stdin();
                let mut stdout = std::io::stdout();

                println!();
                println!("  Model loaded:");
                println!();
                println!("    Total tokens  :  {}", model.tokens.len());
                println!("    Forward len   :  {}", model.transitions.forward_len());
                println!("    Backward len  :  {}", model.transitions.backward_len());
                println!("    Complexity    :  {}", model.transitions.calc_complexity());
                println!("    Average paths :  {:.5}", model.transitions.calc_avg_paths());
                println!("    Variety       :  {:.5}%", model.transitions.calc_variety() * 100.0);

                if !model.headers().is_empty() {
                    println!();
                    println!("  Headers:");
                    println!();

                    let max_len = model.headers()
                        .keys()
                        .map(|key| key.len())
                        .max()
                        .unwrap_or(0);

                    for (key, value) in model.headers() {
                        let offset = " ".repeat(max_len - key.len());

                        println!("    [{key}]{offset} : {value}");
                    }
                }

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

                    let request = Ngram::construct_tailless(&request);

                    for ngram in model.generate(request, params) {
                        match ngram {
                            Ok(ngram) => {
                                let token = ngram.token();

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

            Self::CheckWord { path, word } => {
                todo!()

                // println!("Reading model...");

                // let model = postcard::from_bytes::<Model<DEFAULT_NGRAM_SIZE>>(&std::fs::read(path)?)?;

                // let Some(token) = model.tokens().find_token(word) else {
                //     anyhow::bail!("Could not find token for word: {word}");
                // };

                // println!("Getting forward transitions...");

                // let Some(forward_transitions) = model.transitions().get_forward_transitions(token) else {
                //     anyhow::bail!("Could not find forward transitions for token: {token}");
                // };

                // println!("Getting backward transitions...");

                // let Some(backward_transitions) = model.transitions().get_backward_transitions(token) else {
                //     anyhow::bail!("Could not find backward transitions for token: {token}");
                // };

                // let mut forward_transitions = forward_transitions.collect::<Vec<_>>();
                // let mut backward_transitions = backward_transitions.collect::<Vec<_>>();

                // forward_transitions.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
                // backward_transitions.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

                // println!();
                // println!("Top 10 forward transitions (current -> next):");

                // for (next_token, prob) in forward_transitions.into_iter().take(10) {
                //     let word = model.tokens()
                //         .find_word(*next_token)
                //         .unwrap();

                //     println!("  [{word}]: {:.5}%", prob * 100.0);
                // }

                // println!();
                // println!("Top 10 backward transitions (previous -> current):");

                // for (previous_token, prob) in backward_transitions.into_iter().take(10) {
                //     let word = model.tokens()
                //         .find_word(*previous_token)
                //         .unwrap();

                //     println!("  [{word}]: {:.5}%", prob * 100.0);
                // }
            }
        }

        Ok(())
    }
}
