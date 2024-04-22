use std::collections::{HashMap, HashSet};

use crate::prelude::Dataset;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Chains {
    /// (token, [(next token, percent of occurences)])
    pub(crate) chains: HashMap<u32, Vec<(u32, f32)>>,
    pub(crate) beginnings: HashSet<u32>,
    pub(crate) endings: HashSet<u32>
}

impl Chains {
    pub fn build_from_dataset(dataset: &Dataset) -> Self {
        let mut raw_chains: HashMap<u32, Vec<(u32, u32)>> = HashMap::new();
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

        for (token, mut continuations_raw) in raw_chains {
            let mut continuations_sized = Vec::new();

            while let Some((token, weight)) = continuations_raw.pop() {
                let len = continuations_raw.len();

                continuations_raw.retain(|(t, _)| *t != token);

                let amount = (len - continuations_raw.len() + 1) as u32;

                continuations_sized.push((token, amount * weight));
            }

            let mut continuations = Vec::with_capacity(continuations_sized.len());

            let total_tokens = continuations_sized.iter()
                .map(|(_, num)| *num)
                .sum::<u32>();

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
    pub fn is_beginning(&self, token: u32) -> bool {
        self.beginnings.contains(&token)
    }

    #[inline]
    pub fn is_ending(&self, token: u32) -> bool {
        self.endings.contains(&token)
    }

    #[inline]
    pub fn get_continuations(&self, token: u32) -> Option<&Vec<(u32, f32)>> {
        self.chains.get(&token)
    }

    pub fn get_probability(&self, first_token: u32, next_token: u32) -> Option<f32> {
        let continuations = self.get_continuations(first_token)?;

        let prob = continuations.iter()
            .find(|(token, _)| *token == next_token)
            .map(|(_, prob)| *prob)?;

        Some(prob)
    }

    pub fn calculate_complexity(&self) -> u32 {
        let mut complexity = 0;

        for continuations in self.chains.values() {
            complexity += continuations.len() as u32;
        }

        complexity
    }
}
