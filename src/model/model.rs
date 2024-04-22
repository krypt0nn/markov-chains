use std::collections::{HashMap, HashSet};

use crate::prelude::{
    TokenizedMessages,
    GenerationParams,
    TokenGenerator
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Model {
    /// (token, [(next token, amount of occurences)])
    chains: HashMap<u32, Vec<(u32, u32)>>,
    beginnings: HashSet<u32>,
    endings: HashSet<u32>
}

impl Model {
    pub fn build(messages: TokenizedMessages) -> Self {
        let mut raw_chains: HashMap<u32, Vec<u32>> = HashMap::new();
        let mut beginnings = HashSet::new();
        let mut endings = HashSet::new();

        for message in messages.messages {
            let beginning = message[0];
            let ending = message[message.len() - 1];

            beginnings.insert(beginning);
            endings.insert(ending);

            for i in 0..message.len() - 1 {
                let mut continuations = raw_chains.get(&message[i]).cloned()
                    .unwrap_or_default();

                continuations.push(message[i + 1]);

                raw_chains.insert(message[i], continuations);
            }
        }

        let mut chains = HashMap::new();

        for (token, mut continuations_raw) in raw_chains {
            let mut continuations = Vec::new();

            while let Some(token) = continuations_raw.pop() {
                let len = continuations_raw.len();

                continuations_raw.retain(|t| *t != token);

                let amount = (len - continuations_raw.len() + 1) as u32;

                continuations.push((token, amount));
            }

            chains.insert(token, continuations);
        }

        Self {
            chains,
            beginnings,
            endings
        }
    }

    pub fn is_beginning(&self, token: u32) -> bool {
        self.beginnings.contains(&token)
    }

    pub fn is_ending(&self, token: u32) -> bool {
        self.endings.contains(&token)
    }

    pub fn get_continuations(&self, token: u32) -> anyhow::Result<Vec<(u32, f32)>> {
        let Some(continuations) = self.chains.get(&token) else {
            anyhow::bail!("Could not find continuations for token: {token}");
        };

        let mut percent_continuations = Vec::with_capacity(continuations.len());

        let total = continuations.iter().map(|(_, num)| *num).sum::<u32>() as f32;

        for (token, num) in continuations {
            percent_continuations.push((*token, *num as f32 / total));
        }

        Ok(percent_continuations)
    }

    pub fn get_probability(&self, first_token: u32, next_token: u32) -> anyhow::Result<f32> {
        let continuations = self.get_continuations(first_token)?;

        let prob = continuations.iter()
            .find(|(t, _)| *t == next_token)
            .map(|(_, p)| *p)
            .ok_or_else(|| anyhow::anyhow!("Could not find continuation for token: {next_token}"))?;

        Ok(prob)
    }

    pub fn complexity(&self) -> u32 {
        let mut complexity = 0;

        for chain in self.chains.values() {
            complexity += chain.iter().map(|(_, num)| *num).sum::<u32>();
        }

        complexity
    }

    #[allow(clippy::too_many_arguments)]
    pub fn generate<'a>(&'a self, beginning: impl Into<Vec<u32>>, params: &'a GenerationParams) -> TokenGenerator<'a> {
        TokenGenerator {
            chain: beginning.into(),
            params,
            model: self
        }
    }
}
