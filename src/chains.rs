use std::collections::{HashMap, HashSet};

use crate::prelude::Dataset;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Chains {
    /// (token, [(next token, percent of occurences)])
    pub(crate) chains: HashMap<u64, Vec<(u64, f32)>>,
    pub(crate) beginnings: HashSet<u64>,
    pub(crate) endings: HashSet<u64>
}

impl Chains {
    pub fn build_from_dataset(dataset: &Dataset) -> Self {
        let mut raw_chains: HashMap<u64, Vec<(u64, u64)>> = HashMap::new();
        let mut beginnings = HashSet::new();
        let mut endings = HashSet::new();

        for (messages, weight) in &dataset.messages {
            for message in &messages.messages {
                let beginning = message[0];
                let ending = message[message.len() - 1];

                beginnings.insert(beginning);
                endings.insert(ending);

                for i in 0..message.len() - 1 {
                    let mut continuations = raw_chains.get(&message[i])
                        .cloned()
                        .unwrap_or_default();

                    continuations.push((message[i + 1], *weight));

                    raw_chains.insert(message[i], continuations);
                }
            }
        }

        let mut chains = HashMap::new();

        for (token, continuations_raw) in raw_chains {
            let mut continuations_sized = HashMap::new();

            for (token, weight) in continuations_raw {
                let value = continuations_sized.entry(token)
                    .or_insert(0);

                *value += weight;
            }

            let mut continuations = Vec::with_capacity(continuations_sized.len());

            let total_tokens = continuations_sized.values()
                .copied()
                .sum::<u64>();

            for (token, amount) in continuations_sized {
                continuations.push((token, amount as f32 / total_tokens as f32));
            }

            continuations.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            chains.insert(token, continuations);
        }

        Self {
            chains,
            beginnings,
            endings
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.chains.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.chains.is_empty()
    }

    #[inline]
    pub fn is_beginning(&self, token: u64) -> bool {
        self.beginnings.contains(&token)
    }

    #[inline]
    pub fn is_ending(&self, token: u64) -> bool {
        self.endings.contains(&token)
    }

    #[inline]
    pub fn get_continuations(&self, token: u64) -> Option<&Vec<(u64, f32)>> {
        self.chains.get(&token)
    }

    pub fn get_probability(&self, first_token: u64, next_token: u64) -> Option<f32> {
        let continuations = self.get_continuations(first_token)?;

        let prob = continuations.iter()
            .find(|(token, _)| *token == next_token)
            .map(|(_, prob)| *prob)?;

        Some(prob)
    }

    pub fn calculate_complexity(&self) -> u64 {
        let mut complexity = 0;

        for continuations in self.chains.values() {
            complexity += continuations.len() as u64;
        }

        complexity
    }
}

mod tests {
    #[test]
    fn build_chains() -> anyhow::Result<()> {
        use crate::prelude::*;

        let messages = Messages::parse_from_lines(&[
            String::from("Hello, World!"),
            String::from("Example text")
        ]);

        let tokens = Tokens::parse_from_messages(&messages);

        let messages = TokenizedMessages::tokenize_message(&messages, &tokens)?;

        let dataset = Dataset::default()
            .with_messages(messages, 1)
            .with_tokens(tokens);

        // hello -> world
        // example -> text
        let chains = dataset.build_chains();

        let hello = dataset.tokens.find_token("hello,").unwrap();
        let world = dataset.tokens.find_token("world!").unwrap();
        let example = dataset.tokens.find_token("example").unwrap();
        let text = dataset.tokens.find_token("text").unwrap();

        assert_eq!(chains.len(), 2);

        assert!(chains.is_beginning(hello));
        assert!(!chains.is_beginning(world));
        assert!(chains.is_beginning(example));
        assert!(!chains.is_beginning(text));

        assert!(!chains.is_ending(hello));
        assert!(chains.is_ending(world));
        assert!(!chains.is_ending(example));
        assert!(chains.is_ending(text));

        assert_eq!(chains.get_continuations(hello), Some(&vec![(world, 1.0)]));
        assert_eq!(chains.get_continuations(example), Some(&vec![(text, 1.0)]));

        assert_eq!(chains.get_continuations(world), None);
        assert_eq!(chains.get_continuations(text), None);

        assert_eq!(chains.get_probability(hello, world), Some(1.0));
        assert_eq!(chains.get_probability(example, text), Some(1.0));

        assert_eq!(chains.get_probability(world, hello), None);
        assert_eq!(chains.get_probability(text, example), None);

        Ok(())
    }
}
