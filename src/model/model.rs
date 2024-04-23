use std::collections::HashMap;

use crate::prelude::{
    Dataset,
    Tokens,
    Ngram,
    GenerationParams,
    Transitions,
    Generator
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Model<const NGRAM_SIZE: usize> {
    pub(crate) headers: HashMap<String, String>,
    pub(crate) transitions: Transitions<NGRAM_SIZE>,
    pub(crate) tokens: Tokens
}

impl<const NGRAM_SIZE: usize> Model<NGRAM_SIZE> {
    #[inline]
    pub fn build(dataset: Dataset) -> Self {
        let model = Self {
            headers: HashMap::new(),
            transitions: dataset.build_transitions(),
            tokens: dataset.tokens
        };

        model.with_header("version", env!("CARGO_PKG_VERSION"))
             .with_header("ngram_size", NGRAM_SIZE)
    }

    #[inline]
    pub fn with_header(mut self, tag: impl ToString, value: impl ToString) -> Self {
        self.headers.insert(tag.to_string(), value.to_string());

        self
    }

    #[inline]
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    #[inline]
    pub fn transitions(&self) -> &Transitions<NGRAM_SIZE> {
        &self.transitions
    }

    #[inline]
    pub fn tokens(&self) -> &Tokens {
        &self.tokens
    }

    pub fn get_ngram(&self, words: [impl AsRef<str>; NGRAM_SIZE]) -> anyhow::Result<Ngram<NGRAM_SIZE>> {
        let mut ngram = [0; NGRAM_SIZE];

        for i in 0..NGRAM_SIZE {
            let Some(token) = self.tokens.find_token(words[i].as_ref()) else {
                return Err(anyhow::anyhow!("Couldn't find token for word: {}", words[i].as_ref()));
            };

            ngram[i] = token;
        }

        Ok(Ngram::new(ngram))
    }

    #[inline]
    pub fn generate<'a>(&'a self, beginning: impl Into<Vec<Ngram<NGRAM_SIZE>>>, params: &'a GenerationParams) -> Generator<'a, NGRAM_SIZE> {
        Generator {
            chain: beginning.into(),
            params,
            model: self
        }
    }
}
