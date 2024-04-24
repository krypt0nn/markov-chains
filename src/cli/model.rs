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

        #[arg(long)]
        /// Build bigrams transitions table
        bigrams: bool,

        #[arg(long)]
        /// Build trigrams transitions table
        trigrams: bool,

        #[arg(long)]
        /// Header to add to the model
        /// 
        /// `--header key=value`
        header: Vec<String>,

        #[arg(short, long)]
        /// Path to the model output
        output: PathBuf
    },

    /// Build language model from plain messages files
    FromScratch {
        #[arg(short, long)]
        /// Path to the plain messages file
        messages: Vec<PathBuf>,

        #[arg(long)]
        /// Build bigrams transitions table
        bigrams: bool,

        #[arg(long)]
        /// Build trigrams transitions table
        trigrams: bool,

        #[arg(long)]
        /// Header to add to the model
        /// 
        /// `--header key=value`
        header: Vec<String>,

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
            Self::Build { dataset, bigrams, trigrams, header, output } => {
                println!("Reading dataset bundle...");

                let messages = postcard::from_bytes::<Dataset>(&std::fs::read(dataset)?)?;

                println!("Building model...");

                let mut model = Model::build(messages, *bigrams, *trigrams);

                for header in header {
                    if let Some((key, value)) = header.split_once('=') {
                        model = model.with_header(key, value);
                    }
                }

                println!("Storing model...");

                std::fs::write(output, postcard::to_allocvec(&model)?)?;

                println!("Done");
            }

            Self::FromScratch { messages: paths, bigrams, trigrams, header, output } => {
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

                let mut model = Model::build(dataset, *bigrams, *trigrams);

                for header in header {
                    if let Some((key, value)) = header.split_once('=') {
                        model = model.with_header(key, value);
                    }
                }

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

                let chains = (
                    model.transitions.trigrams_len()
                        .map(|len| len.to_string())
                        .unwrap_or(String::from("N/A")),

                    model.transitions.bigrams_len()
                        .map(|len| len.to_string())
                        .unwrap_or(String::from("N/A")),

                    model.transitions.unigrams_len()
                );

                let avg_paths = (
                    model.transitions.calc_avg_trigram_paths()
                        .map(|avg| format!("{:.4}", avg))
                        .unwrap_or(String::from("N/A")),

                    model.transitions.calc_avg_bigram_paths()
                        .map(|avg| format!("{:.4}", avg))
                        .unwrap_or(String::from("N/A")),

                    format!("{:.4}", model.transitions.calc_avg_unigram_paths())
                );

                let variety = (
                    model.transitions.calc_trigram_variety()
                        .map(|variety| format!("{:.4}%", variety * 100.0))
                        .unwrap_or(String::from("N/A")),

                    model.transitions.calc_bigram_variety()
                        .map(|variety| format!("{:.4}%", variety * 100.0))
                        .unwrap_or(String::from("N/A")),

                    format!("{:.4}%", model.transitions.calc_unigram_variety() * 100.0)
                );

                println!();
                println!("  Model loaded:");
                println!();
                println!("    Total tokens  :  {}", model.tokens.len());
                println!("    Chains        :  {} / {} / {}", chains.0, chains.1, chains.2);
                println!("    Avg paths     :  {} / {} / {}", avg_paths.0, avg_paths.1, avg_paths.2);
                println!("    Variety       :  {} / {} / {}", variety.0, variety.1, variety.2);

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

                    for token in model.generate(request, params) {
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
