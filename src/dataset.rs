use crate::prelude::{
    TokenizedMessages,
    Tokens,
    Chains
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dataset {
    /// (messages, weight)
    pub(crate) messages: Vec<(TokenizedMessages, u32)>,
    pub(crate) tokens: Tokens
}

impl Dataset {
    #[inline]
    pub fn with_messages(mut self, messages: TokenizedMessages, weight: u32) -> Self {
        self.messages.push((messages, weight));

        self
    }

    #[inline]
    pub fn with_tokens(mut self, tokens: Tokens) -> Self {
        self.tokens = self.tokens.merge(tokens);

        self
    }

    #[inline]
    pub fn messages(&self) -> &[(TokenizedMessages, u32)] {
        &self.messages
    }

    #[inline]
    pub fn tokens(&self) -> &Tokens {
        &self.tokens
    }

    pub fn build_chains(&self) -> Chains {
        Chains::build_from_dataset(self)
    }
}
