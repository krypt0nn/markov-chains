use std::collections::HashSet;

use crate::prelude::{
    Messages,
    Tokens
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenizedMessages {
    pub(crate) messages: HashSet<Vec<u64>>
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

    #[inline]
    pub fn messages(&self) -> &HashSet<Vec<u64>> {
        &self.messages
    }
}

mod tests {
    #[test]
    fn tokenize() -> anyhow::Result<()> {
        use super::{Messages, Tokens, TokenizedMessages};

        let messages = Messages::parse_from_lines(&[
            String::from("Hello, World!"),
            String::from("Example text")
        ]);

        let tokens = Tokens::parse_from_messages(&messages);

        let tokenized = TokenizedMessages::tokenize_message(&messages, &tokens)?;

        let hello = tokens.find_token("hello,").unwrap();
        let world = tokens.find_token("world!").unwrap();
        let example = tokens.find_token("example").unwrap();
        let text = tokens.find_token("text").unwrap();

        assert!(tokenized.messages.contains(&vec![hello, world]));
        assert!(tokenized.messages.contains(&vec![example, text]));

        Ok(())
    }
}
