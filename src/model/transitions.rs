use std::collections::HashMap;

use crate::prelude::{
    Dataset,
    Ngram
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Transitions<const NGRAM_SIZE: usize> {
    /// count = forward_transitions\[current_ngram\]\[next_ngram\]
    pub(crate) forward_transitions: HashMap<Ngram<NGRAM_SIZE>, HashMap<Ngram<NGRAM_SIZE>, u64>>,

    /// count = backward_transitions\[current_ngram\]\[previous_ngram\]
    pub(crate) backward_transitions: HashMap<Ngram<NGRAM_SIZE>, HashMap<Ngram<NGRAM_SIZE>, u64>>,
}

impl<const NGRAM_SIZE: usize> Transitions<NGRAM_SIZE> {
    pub fn build_from_dataset(dataset: &Dataset) -> Self {
        let mut forward_transitions: HashMap<Ngram<NGRAM_SIZE>, HashMap<Ngram<NGRAM_SIZE>, u64>> = HashMap::new();
        let mut backward_transitions: HashMap<Ngram<NGRAM_SIZE>, HashMap<Ngram<NGRAM_SIZE>, u64>> = HashMap::new();

        for (messages, weight) in &dataset.messages {
            for message in &messages.messages {
                let message = Ngram::<NGRAM_SIZE>::construct(message);

                let n = message.len();

                for i in 0..n {
                    if i > 0 {
                        *backward_transitions.entry(message[i])
                            .or_default()
                            .entry(message[i - 1])
                            .or_default() += *weight;
                    }

                    if i < n - 1 {
                        *forward_transitions.entry(message[i])
                            .or_default()
                            .entry(message[i + 1])
                            .or_default() += *weight;
                    }
                }
            }
        }

        // let mut forward_transitions: HashMap<Ngram<NGRAM_SIZE>, HashMap<Ngram<NGRAM_SIZE>, f64>> = HashMap::new();
        // let mut backward_transitions: HashMap<Ngram<NGRAM_SIZE>, HashMap<Ngram<NGRAM_SIZE>, f64>> = HashMap::new();

        // for (ngram, transitions) in forward_transitions_counter {
        //     let total = transitions.values().sum::<u64>() as f64;

        //     let ngram_transitions = HashMap::from_iter(transitions.iter()
        //         .map(|(key, value)| (*key, *value as f64 / total)));

        //     forward_transitions.insert(ngram, ngram_transitions);
        // }

        // for (ngram, transitions) in backward_transitions_counter {
        //     let total = transitions.values().sum::<u64>() as f64;

        //     let ngram_transitions = HashMap::from_iter(transitions.iter()
        //         .map(|(key, value)| (*key, *value as f64 / total)));

        //     backward_transitions.insert(ngram, ngram_transitions);
        // }

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
    pub fn get_forward_transitions(&self, ngram: Ngram<NGRAM_SIZE>) -> Option<impl Iterator<Item = (&'_ Ngram<NGRAM_SIZE>, &'_ u64)>> {
        self.forward_transitions.get(&ngram)
            .map(|transitions| transitions.iter())
    }

    #[inline]
    pub fn get_backward_transitions(&self, ngram: Ngram<NGRAM_SIZE>) -> Option<impl Iterator<Item = (&'_ Ngram<NGRAM_SIZE>, &'_ u64)>> {
        self.backward_transitions.get(&ngram)
            .map(|transitions| transitions.iter())
    }

    #[inline]
    /// Calculate complexity of the chain
    /// 
    /// Complexity is the sum of the number of possible transitions for each ngram.
    pub fn calc_complexity(&self) -> u64 {
        self.forward_transitions.iter()
            .filter(|(k, _)| !k.is_part_start() && !k.is_part_end())
            .map(|(_, transitions)| transitions.iter())
            .map(|transitions| transitions.filter(|(k, _)| !k.is_part_start() && !k.is_part_end()))
            .map(|transitions| transitions.count() as u64)
            .sum()
    }

    #[inline]
    /// Calculate variety of the chain
    pub fn calc_variety(&self) -> f64 {
        let avg_paths = self.calc_complexity() as f64 / self.forward_len() as f64;

        let more_than_avg_paths = self.forward_transitions.iter()
            .filter(|(k, _)| !k.is_part_start() && !k.is_part_end())
            .map(|(_, transitions)| transitions.keys())
            .map(|ngrams| ngrams.filter(|ngram| !ngram.is_part_start() && !ngram.is_part_end()))
            .map(|ngrams| ngrams.count() as f64)
            .filter(|count| *count > avg_paths)
            .count();

        more_than_avg_paths as f64 / self.forward_len() as f64
    }

    #[inline]
    /// Get probability of the (current_ngram -> next_ngram)
    pub fn get_forward_probability(&self, current_ngram: Ngram<NGRAM_SIZE>, next_ngram: Ngram<NGRAM_SIZE>) -> Option<f64> {
        self.forward_transitions.get(&current_ngram)
            .and_then(|transitions| {
                transitions.get(&next_ngram).map(|count| (count, transitions.len()))
            })
            .map(|(count, total)| *count as f64 / total as f64)
    }

    #[inline]
    /// Get probability of the (previous_ngram <- current_ngram)
    pub fn get_backward_probability(&self, previous_ngram: Ngram<NGRAM_SIZE>, current_ngram: Ngram<NGRAM_SIZE>) -> Option<f64> {
        self.backward_transitions.get(&current_ngram)
            .and_then(|transitions| {
                transitions.get(&previous_ngram).map(|count| (count, transitions.len()))
            })
            .map(|(count, total)| *count as f64 / total as f64)
    }

    // pub fn calc_bayes_probability(&self, current_ngram: u64, next_ngram: u64) -> f64 {
        
    // }

    // pub fn calc_absolute_discounting_smoothing(&self) -> f64 {
        
    // }

    // pub fn calc_knesser_nay_smoothing(&self) -> f64 {

    // }
}

mod tests {
    #[test]
    fn build_transitions() -> anyhow::Result<()> {
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
        let transitions = dataset.build_transitions::<1>();

        let hello = dataset.tokens.find_token("hello,").unwrap();
        let world = dataset.tokens.find_token("world!").unwrap();
        let example = dataset.tokens.find_token("example").unwrap();
        let text = dataset.tokens.find_token("text").unwrap();

        let hello = Unigram::new([hello]);
        let world = Unigram::new([world]);
        let example = Unigram::new([example]);
        let text = Unigram::new([text]);

        assert_eq!(transitions.get_forward_transitions(hello).map(|t| t.collect::<Vec<_>>()), Some(vec![(&world, &1)]));
        assert_eq!(transitions.get_forward_transitions(example).map(|t| t.collect::<Vec<_>>()), Some(vec![(&text, &1)]));

        assert_eq!(transitions.get_forward_transitions(world).map(|t| t.collect::<Vec<_>>()), Some(vec![(&Unigram::end(), &1)]));
        assert_eq!(transitions.get_forward_transitions(text).map(|t| t.collect::<Vec<_>>()), Some(vec![(&Unigram::end(), &1)]));

        assert_eq!(transitions.get_backward_transitions(world).map(|t| t.collect::<Vec<_>>()), Some(vec![(&hello, &1)]));
        assert_eq!(transitions.get_backward_transitions(text).map(|t| t.collect::<Vec<_>>()), Some(vec![(&example, &1)]));

        assert_eq!(transitions.get_backward_transitions(hello).map(|t| t.collect::<Vec<_>>()), Some(vec![(&Unigram::start(), &1)]));
        assert_eq!(transitions.get_backward_transitions(example).map(|t| t.collect::<Vec<_>>()), Some(vec![(&Unigram::start(), &1)]));

        assert_eq!(transitions.get_forward_probability(hello, world), Some(1.0));
        assert_eq!(transitions.get_forward_probability(example, text), Some(1.0));
        assert_eq!(transitions.get_forward_probability(world, Unigram::end()), Some(1.0));
        assert_eq!(transitions.get_forward_probability(text, Unigram::end()), Some(1.0));

        assert_eq!(transitions.get_backward_probability(hello, world), Some(1.0));
        assert_eq!(transitions.get_backward_probability(example, text), Some(1.0));
        assert_eq!(transitions.get_backward_probability(Unigram::start(), hello), Some(1.0));
        assert_eq!(transitions.get_backward_probability(Unigram::start(), example), Some(1.0));

        Ok(())
    }
}
