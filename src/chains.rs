use std::collections::HashMap;

use crate::prelude::Dataset;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Chains {
    /// probability = forward_transitions\[current_token\]\[next_token\]
    pub(crate) forward_transitions: HashMap<u64, HashMap<u64, f64>>,

    /// probability = backward_transitions\[current_token\]\[previous_token\]
    pub(crate) backward_transitions: HashMap<u64, HashMap<u64, f64>>,
}

impl Chains {
    pub fn build_from_dataset(dataset: &Dataset) -> Self {
        let mut forward_transitions_counter: HashMap<u64, HashMap<u64, u64>> = HashMap::new();
        let mut backward_transitions_counter: HashMap<u64, HashMap<u64, u64>> = HashMap::new();

        for (messages, weight) in &dataset.messages {
            for message in &messages.messages {
                let n = message.len();

                for i in 0..n {
                    if i > 0 {
                        *backward_transitions_counter.entry(message[i])
                            .or_default()
                            .entry(message[i - 1])
                            .or_default() += *weight;
                    }

                    if i < n - 1 {
                        *forward_transitions_counter.entry(message[i])
                            .or_default()
                            .entry(message[i + 1])
                            .or_default() += *weight;
                    }
                }
            }
        }

        let mut forward_transitions: HashMap<u64, HashMap<u64, f64>> = HashMap::new();
        let mut backward_transitions: HashMap<u64, HashMap<u64, f64>> = HashMap::new();

        for (token, transitions) in forward_transitions_counter {
            let total = transitions.values().sum::<u64>() as f64;

            let token_transitions = HashMap::from_iter(transitions.iter()
                .map(|(key, value)| (*key, *value as f64 / total)));

            forward_transitions.insert(token, token_transitions);
        }

        for (token, transitions) in backward_transitions_counter {
            let total = transitions.values().sum::<u64>() as f64;

            let token_transitions = HashMap::from_iter(transitions.iter()
                .map(|(key, value)| (*key, *value as f64 / total)));

            backward_transitions.insert(token, token_transitions);
        }

        Self {
            forward_transitions,
            backward_transitions
        }
    }

    #[inline]
    pub fn forward_len(&self) -> usize {
        self.forward_transitions.len()
    }

    #[inline]
    pub fn backward_len(&self) -> usize {
        self.backward_transitions.len()
    }

    #[inline]
    pub fn get_forward_transitions(&self, token: u64) -> Option<impl Iterator<Item = (&'_ u64, &'_ f64)>> {
        self.forward_transitions.get(&token)
            .map(|transitions| transitions.iter())
    }

    #[inline]
    pub fn get_backward_transitions(&self, token: u64) -> Option<impl Iterator<Item = (&'_ u64, &'_ f64)>> {
        self.backward_transitions.get(&token)
            .map(|transitions| transitions.iter())
    }

    pub fn calculate_complexity(&self) -> u64 {
        let mut complexity = 0;

        for continuations in self.forward_transitions.values() {
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

        assert_eq!(chains.forward_len(), 2);
        assert_eq!(chains.backward_len(), 2);

        assert_eq!(chains.get_forward_transitions(hello).map(|t| t.collect::<Vec<_>>()), Some(vec![(&world, &1.0)]));
        assert_eq!(chains.get_forward_transitions(example).map(|t| t.collect::<Vec<_>>()), Some(vec![(&text, &1.0)]));

        assert!(chains.get_forward_transitions(world).is_none());
        assert!(chains.get_forward_transitions(text).is_none());

        assert_eq!(chains.get_backward_transitions(world).map(|t| t.collect::<Vec<_>>()), Some(vec![(&hello, &1.0)]));
        assert_eq!(chains.get_backward_transitions(text).map(|t| t.collect::<Vec<_>>()), Some(vec![(&example, &1.0)]));

        assert!(chains.get_backward_transitions(hello).is_none());
        assert!(chains.get_backward_transitions(example).is_none());

        Ok(())
    }
}
