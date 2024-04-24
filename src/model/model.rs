use std::collections::HashMap;

use crate::prelude::{
    Dataset,
    Tokens,
    GenerationParams,
    Transitions,
    Generator
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Model {
    pub(crate) headers: HashMap<String, String>,
    pub(crate) transitions: Transitions,
    pub(crate) tokens: Tokens
}

impl Model {
    #[inline]
    pub fn build(dataset: Dataset, build_bigrams: bool, build_trigrams: bool) -> Self {
        let model = Self {
            headers: HashMap::new(),
            transitions: dataset.build_transitions(build_bigrams, build_trigrams),
            tokens: dataset.tokens
        };

        model.with_header("version", env!("CARGO_PKG_VERSION"))
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
    pub fn transitions(&self) -> &Transitions {
        &self.transitions
    }

    #[inline]
    pub fn tokens(&self) -> &Tokens {
        &self.tokens
    }

    #[inline]
    pub fn generate<'a>(&'a self, beginning: impl Into<Vec<u64>>, params: &'a GenerationParams) -> Generator<'a> {
        Generator {
            chain: beginning.into(),
            params,
            model: self
        }
    }
}
