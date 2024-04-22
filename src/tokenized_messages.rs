use std::collections::HashSet;

use crate::prelude::{
    Messages,
    Tokens
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenizedMessages {
    pub(crate) messages: HashSet<Vec<u32>>
}

impl TokenizedMessages {
    pub fn tokenize_message(messages: &Messages, tokens: &Tokens) -> anyhow::Result<Self> {
        let mut tokenized = HashSet::new();

        for message in messages.messages() {
            let mut message_tokens = Vec::with_capacity(message.len());

            for word in message {
                let Some(token) = tokens.find_token(word) else {
                    anyhow::bail!("Could not find token for word: {word}");
                };

                message_tokens.push(token);
            }

            tokenized.insert(message_tokens);
        }

        Ok(Self {
            messages: tokenized
        })
    }
}
