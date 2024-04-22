use std::collections::HashMap;

use crate::prelude::Messages;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tokens {
    pub(crate) token_word: HashMap<u64, String>,
    pub(crate) word_token: HashMap<String, u64>
}

impl Tokens {
    pub fn parse_from_messages(messages: &Messages) -> Self {
        let mut token_word = HashMap::new();
        let mut word_token = HashMap::new();

        for message in messages.messages() {
            for word in message {
                if !word_token.contains_key(word) {
                    let mut token = rand::random::<u64>();

                    while token_word.contains_key(&token) {
                        token = rand::random::<u64>();
                    }

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
        for (word, mut token) in tokens.word_token {
            if !self.word_token.contains_key(&word) {
                while self.token_word.contains_key(&token) {
                    token = rand::random::<u64>();
                }

                self.word_token.insert(word.clone(), token);
                self.token_word.insert(token, word);
            }
        }

        self
    }

    #[inline]
    pub fn find_token(&self, word: impl AsRef<str>) -> Option<u64> {
        self.word_token.get(word.as_ref()).copied()
    }

    #[inline]
    pub fn find_word(&self, token: u64) -> Option<&String> {
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

    pub fn detokenize_message(&self, tokens: &[u64]) -> anyhow::Result<String> {
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

mod tests {
    #[test]
    fn tokenizing() {
        use super::{Tokens, Messages};

        let messages = Messages::parse_from_lines(&[
            String::from("Hello, World!"),
            String::from("Example text")
        ]);

        let tokens = Tokens::parse_from_messages(&messages);

        let hello = tokens.find_token("hello,").unwrap();
        let world = tokens.find_token("world!").unwrap();
        let example = tokens.find_token("example").unwrap();
        let text = tokens.find_token("text").unwrap();

        assert_eq!(tokens.find_word(hello), Some(&String::from("hello,")));
        assert_eq!(tokens.find_word(world), Some(&String::from("world!")));
        assert_eq!(tokens.find_word(example), Some(&String::from("example")));
        assert_eq!(tokens.find_word(text), Some(&String::from("text")));
    }

    #[test]
    fn merging() {
        use super::{Tokens, Messages};

        let messages = Messages::default()
            .merge(Messages::parse_from_lines(&[
                String::from("Hello, World!")
            ]))
            .merge(Messages::parse_from_lines(&[
                String::from("Example text")
            ]));

        let tokens = Tokens::parse_from_messages(&messages);

        let hello = tokens.find_token("hello,").unwrap();
        let world = tokens.find_token("world!").unwrap();
        let example = tokens.find_token("example").unwrap();
        let text = tokens.find_token("text").unwrap();

        assert_eq!(tokens.find_word(hello), Some(&String::from("hello,")));
        assert_eq!(tokens.find_word(world), Some(&String::from("world!")));
        assert_eq!(tokens.find_word(example), Some(&String::from("example")));
        assert_eq!(tokens.find_word(text), Some(&String::from("text")));
    }
}
