use crate::prelude::{
    Dataset,
    Chains,
    Tokens,
    GenerationParams,
    TokenGenerator
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Model {
    pub(crate) chains: Chains,
    pub(crate) tokens: Tokens
}

impl Model {
    #[inline]
    pub fn build(dataset: Dataset) -> Self {
        Self {
            chains: dataset.build_chains(),
            tokens: dataset.tokens
        }
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
