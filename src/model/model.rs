use std::collections::HashMap;

use crate::prelude::{
    Dataset,
    Chains,
    Tokens,
    GenerationParams,
    TokenGenerator
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Model {
    pub(crate) headers: HashMap<String, String>,
    pub(crate) chains: Chains,
    pub(crate) tokens: Tokens
}

impl Model {
    #[inline]
    pub fn build(dataset: Dataset) -> Self {
        let model = Self {
            headers: HashMap::new(),
            chains: dataset.build_chains(),
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
    pub fn chains(&self) -> &Chains {
        &self.chains
    }

    #[inline]
    pub fn tokens(&self) -> &Tokens {
        &self.tokens
    }

    #[inline]
    pub fn generate<'a>(&'a self, beginning: impl Into<Vec<u64>>, params: &'a GenerationParams) -> TokenGenerator<'a> {
        TokenGenerator {
            chain: beginning.into(),
            params,
            model: self
        }
    }
}
