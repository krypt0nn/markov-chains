use std::iter::FusedIterator;

use crate::prelude::{
    GenerationParams,
    Model
};

pub struct TokenGenerator<'a> {
    pub(crate) chain: Vec<u32>,
    pub(crate) params: &'a GenerationParams,
    pub(crate) model: &'a Model
}

impl<'a> Iterator for TokenGenerator<'a> {
    type Item = anyhow::Result<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get current token from the chain history
        let current = self.chain.last().copied()?;

        // Get random seed from 0.0 to 1.0
        let random_seed = rand::random::<u32>() as f32 / u32::MAX as f32;

        // If the chain's length is greater than the minimum length
        if self.chain.len() > self.params.min_length {
            // If the current token is an ending
            if self.model.chains.is_ending(current) {
                // If the random seed is greater than the end height
                if random_seed * self.params.end_weight >= self.params.end_height {
                    // Stop tokens generation
                    return None;
                }

                // If the chain's length is greater than the maximum length
                if self.chain.len() > self.params.max_len {
                    // Stop tokens generation
                    return None;
                }
            }

            // If the chain's length is greater than the force break length
            if self.chain.len() > self.params.force_break_len {
                // Stop tokens generation
                return None;
            }
        }

        // Get possible continuations for the current token
        let mut continuations = self.model.chains.get_continuations(current)?.clone();

        // If there are no continuations
        if continuations.is_empty() {
            // Stop tokens generation
            return None;
        }

        // Get the context window from the chain history
        let chain_window = &self.chain[self.chain.len().saturating_sub(self.params.context_window)..];

        // Update probabilities for each continuation
        for continuation in &mut continuations {
            // Iterate over the context window
            for i in 1..chain_window.len() {
                // Multiply the probability by the continuation's probability
                continuation.1 *= self.model.chains.get_probability(chain_window[i - 1], chain_window[i])?;
            }
        }

        // Sort the continuations by probability
        continuations.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // dbg!(&continuations);

        // While there are continuations
        while continuations.len() > 1 {
            // Get random seed from 0.0 to 1.0
            let random_seed = rand::random::<u32>() as f32 / u32::MAX as f32;

            // Get the next most probable token
            let next = continuations.last().unwrap().0;

            // Find all the repeats of the next token
            let repeats = self.chain.iter()
                .filter(|token| **token == next)
                .sum::<u32>();

            // Calculate the temperature
            let temperature = self.params.temperature *
                self.params.temperature_alpha.powi(self.chain.len() as i32) *
                self.params.repeat_penalty.powi(repeats as i32);

            // If the random seed is greater than the temperature
            if random_seed > temperature {
                // Stop tokens generation
                break;
            }

            // Remove current most probable token
            continuations.pop();
        }

        // Get the most probable token
        let next = continuations.last().unwrap().0;

        // Add the most probable token to the chain
        self.chain.push(next);

        // Return the most probable token
        Some(Ok(next))
    }
}

impl<'a> FusedIterator for TokenGenerator<'a> {}
