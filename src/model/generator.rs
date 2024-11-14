use std::iter::FusedIterator;

use crate::prelude::{
    Unigram,
    Bigram,
    Trigram,
    GenerationParams,
    Model,
    END_TOKEN
};

pub struct Generator<'a> {
    pub(crate) chain: Vec<u64>,
    pub(crate) params: &'a GenerationParams,
    pub(crate) model: &'a Model
}

impl<'a> Iterator for Generator<'a> {
    type Item = anyhow::Result<u64>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut continuations = None;

        // Get initial predictions from the trigram
        if !self.params.no_trigrams {
            let trigram = Trigram::construct_tailless(&self.chain);

            if let Some(trigram) = trigram.last() {
                if let Some(trigram_continuations) = self.model.transitions.for_trigram(trigram) {
                    let trigram_continuations = trigram_continuations
                        .filter(|(token, _)| !token.is_end())
                        .map(|(token, number)| (token.token(), *number))
                        .collect::<Vec<_>>();

                    if !trigram_continuations.is_empty() {
                        continuations = Some(trigram_continuations);
                    }
                }
            }
        }

        // If there are no continuations from the trigram - try to get them from the bigram
        if !self.params.no_bigrams && continuations.is_none() {
            let bigram = Bigram::construct_tailless(&self.chain);

            if let Some(bigram) = bigram.last() {
                if let Some(bigram_continuations) = self.model.transitions.for_bigram(bigram) {
                    let bigram_continuations = bigram_continuations
                        .filter(|(token, _)| !token.is_end())
                        .map(|(token, number)| (token.token(), *number))
                        .collect::<Vec<_>>();

                    if !bigram_continuations.is_empty() {
                        continuations = Some(bigram_continuations);
                    }
                }
            }
        }

        // If there are no continuations from the bigram - try to get them from the unigram
        if continuations.is_none() {
            let unigram = Unigram::construct_tailless(&self.chain);

            if let Some(unigram) = unigram.last() {
                if let Some(unigram_continuations) = self.model.transitions.for_unigram(unigram) {
                    let unigram_continuations = unigram_continuations
                        .filter(|(token, _)| !token.is_end())
                        .map(|(token, number)| (token.token(), *number))
                        .collect::<Vec<_>>();

                    if !unigram_continuations.is_empty() {
                        continuations = Some(unigram_continuations);
                    }
                }
            }
        }

        // Stop generation if there are no continuations
        let mut continuations = continuations?;

        // Find offset according to the normal distribution
        let offset = ((1.0 - self.params.k_normal) * continuations.len() as f64).floor() as usize / 2;

        // If there's less possible variants than expected
        if continuations.len() <= offset * 2 {
            // Stop tokens generation
            return None;
        }

        // Remove most and least probable variants
        continuations = continuations[offset..continuations.len() - offset].to_vec();

        // If there are no continuations
        if continuations.is_empty() {
            // Stop tokens generation
            return None;
        }

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
        continuations.sort_by(|a, b| a.1.cmp(&b.1));

        // dbg!(&continuations);

        // While there are continuations
        while continuations.len() > 1 {
            // Get random seed from 0.0 to 1.0
            let random_seed = rand::random::<u32>() as f64 / u32::MAX as f64;

            // Get the next most probable token
            let next = continuations.last().unwrap().0;

            // Find last repeats of the next token
            let repeats = self.chain.iter()
                .rev()
                .take(self.params.repeat_penalty_window)
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

            // Otherwise
            else {
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
            }

            // Remove current most probable token
            continuations.pop();
        }

        // Get the most probable token
        let next = continuations.last().unwrap().0;

        // If the chain's length is greater than the minimum length
        if self.chain.len() > self.params.min_len {
            // If the chain's length is greater than the maximum length
            if self.chain.len() > self.params.max_len {
                // Stop tokens generation
                return None;
            }
        }

        // If the next token is an end of the text
        if next == END_TOKEN {
            // Stop tokens generation
            return None;
        }

        // Add the most probable token to the chain
        self.chain.push(next);

        // Return the most probable token
        Some(Ok(next))
    }
}

impl<'a> FusedIterator for Generator<'a> {}
