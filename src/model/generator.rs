use std::iter::FusedIterator;

use rayon::prelude::*;

use crate::prelude::{
    Ngram,
    GenerationParams,
    SmoothingAlgorithm,
    Model
};

pub struct Generator<'a, const NGRAM_SIZE: usize> {
    pub(crate) chain: Vec<Ngram<NGRAM_SIZE>>,
    pub(crate) params: &'a GenerationParams,
    pub(crate) model: &'a Model<NGRAM_SIZE>
}

impl<'a, const NGRAM_SIZE: usize> Iterator for Generator<'a, NGRAM_SIZE> {
    type Item = anyhow::Result<Ngram<NGRAM_SIZE>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get current token from the chain history
        let current = self.chain.last().copied()?;

        // Get possible continuations for the current token
        let mut forward_transitions = self.model.transitions.get_forward_transitions(current)?
            .collect::<Vec<_>>();

        // Find offset according to the normal distribution
        let offset = ((1.0 - self.params.k_normal) * forward_transitions.len() as f64).floor() as usize / 2;

        // If there's less possible variants than expected
        if forward_transitions.len() <= offset * 2 {
            // Stop tokens generation
            return None;
        }

        // Remove most and least probable variants
        forward_transitions = forward_transitions[offset..forward_transitions.len() - offset].to_vec();

        // If there are no continuations
        if forward_transitions.is_empty() {
            // Stop tokens generation
            return None;
        }

        // Apply smoothing function to the possible continuations
        let mut forward_transitions = match &self.params.smoothing {
            Some(SmoothingAlgorithm::AbsoluteDiscounting) => {
                forward_transitions.into_par_iter()
                    .flat_map(|(k, _)| {
                        self.model.transitions.calc_absolute_discounting_smoothing(*k)
                            .map(|prob| (*k, prob))
                    })
                    .collect::<Vec<_>>()
            },

            Some(SmoothingAlgorithm::KneserNay) => unimplemented!(),

            None => {
                forward_transitions.into_par_iter()
                    .flat_map(|(k, _)| {
                        self.model.transitions.get_forward_probability(current, *k)
                            .map(|prob| (*k, prob))
                    })
                    .collect::<Vec<_>>()
            }
        };

        // // Get the context window from the chain history
        // let chain_window = &self.chain[self.chain.len().saturating_sub(self.params.context_window)..];

        // // Update probabilities for each continuation
        // for continuation in &mut continuations {
        //     // Iterate over the context window
        //     for i in 1..chain_window.len() {
        //         // Multiply the probability by the continuation's probability
        //         continuation.1 *= self.model.chains.get_probability(chain_window[i - 1], chain_window[i])?;
        //     }
        // }

        // Sort the continuations by probability
        forward_transitions.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // dbg!(&forward_transitions[forward_transitions.len() - 3..]);

        // While there are continuations
        while forward_transitions.len() > 1 {
            // Get random seed from 0.0 to 1.0
            let random_seed = rand::random::<u32>() as f64 / u32::MAX as f64;

            // Get the next most probable token
            let next = forward_transitions.last().unwrap().0;

            // Find all the repeats of the next token
            let repeats = self.chain.iter()
                .filter(|token| **token == next)
                .count();

            // If the next token is repeated
            if repeats > 0 {
                // If the random seed is lower than the repeat penalty
                // 
                // repeat_penalty: 0.5 -> 0.25 -> 0.125 -> 0.0625 -> ...
                // 
                // lower repeat_penalty => lower chance that the if statement works
                // => higher chance that the next token is skipped
                if random_seed < self.params.repeat_penalty.powi(repeats as i32) {
                    // Keep current token as the next one
                    break;
                }
            }

            // Calculate the temperature
            let temperature = self.params.temperature * self.params.temperature_alpha.powi(self.chain.len() as i32);

            // If the random seed is lower than the temperature
            // 
            // temperature: 0.5 -> 0.25 -> 0.125 -> 0.0625 -> ...
            // 
            // lower temperature => lower chance that the if statement works
            // => higher chance that the next token is skipped
            if random_seed < temperature {
                // Keep current token as the next one
                break;
            }

            // Remove current most probable token
            forward_transitions.pop();
        }

        // Get the most probable token
        let next = forward_transitions.last().unwrap().0;

        // If the chain's length is greater than the minimum length
        if self.chain.len() > self.params.min_length {
            // If the chain's length is greater than the maximum length
            if self.chain.len() > self.params.max_len {
                // Stop tokens generation
                return None;
            }

            // If the next ngram is an end of the text
            if next.is_part_end() {
                // Stop tokens generation
                return None;
            }
        }

        // Add the most probable token to the chain
        self.chain.push(next);

        // Return the most probable token
        Some(Ok(next))
    }
}

impl<'a, const NGRAM_SIZE: usize> FusedIterator for Generator<'a, NGRAM_SIZE> {}
