use std::collections::HashMap;

use crate::prelude::Messages;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tokens {
    pub(crate) token_word: HashMap<u32, String>,
    pub(crate) word_token: HashMap<String, u32>
}

impl Tokens {
    pub fn parse_from_messages(messages: &Messages) -> Self {
        let mut token_word = HashMap::new();
        let mut word_token = HashMap::new();

        for message in messages.messages() {
            for word in message {
                if !word_token.contains_key(word) {
                    let token = word_token.len() as u32;

                    word_token.insert(word.to_owned(), token);
                    token_word.insert(token, word.to_owned());
                }
            }
        }

        Self {
            token_word,
            word_token
        }
    }

    pub fn merge(mut self, tokens: Tokens) -> Self {
        for (word, _) in tokens.word_token {
            if !self.word_token.contains_key(&word) {
                let token = self.word_token.len() as u32;

                self.word_token.insert(word.clone(), token);
                self.token_word.insert(token, word);
            }
        }

        self
    }

    #[inline]
    pub fn find_token(&self, word: impl AsRef<str>) -> Option<u32> {
        self.word_token.get(word.as_ref()).copied()
    }

    #[inline]
    pub fn find_word(&self, token: u32) -> Option<&String> {
        self.token_word.get(&token)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.token_word.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.token_word.is_empty()
    }

    pub fn detokenize_message(&self, tokens: &[u32]) -> anyhow::Result<String> {
        let mut words = Vec::with_capacity(tokens.len());

        for token in tokens {
            let Some(word) = self.find_word(*token) else {
                anyhow::bail!("Could not find word for token: {token}");
            };

            words.push(word.to_owned());
        }

        Ok(words.join(" "))
    }
}
