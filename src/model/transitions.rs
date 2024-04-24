use std::collections::HashMap;

use rayon::prelude::*;

use crate::prelude::{
    Dataset,
    Unigram,
    Bigram,
    Trigram
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Transitions {
    /// count = forward_transitions\[current_ngram\]\[next_ngram\]
    pub(crate) unigrams: HashMap<Unigram, HashMap<Unigram, u64>>,

    /// count = forward_transitions\[current_ngram\]\[next_ngram\]
    pub(crate) bigrams: Option<HashMap<Bigram, HashMap<Bigram, u64>>>,

    /// count = forward_transitions\[current_ngram\]\[next_ngram\]
    pub(crate) trigrams: Option<HashMap<Trigram, HashMap<Trigram, u64>>>
}

impl Transitions {
    pub fn build_from_dataset(dataset: &Dataset, build_bigrams: bool, build_trigrams: bool) -> Self {
        let mut unigrams = HashMap::<Unigram, HashMap<Unigram, u64>>::new();

        let mut bigrams = if build_bigrams {
            Some(HashMap::<Bigram, HashMap<Bigram, u64>>::new())
        } else {
            None
        };

        let mut trigrams = if build_trigrams {
            Some(HashMap::<Trigram, HashMap<Trigram, u64>>::new())
        } else {
            None
        };

        for (messages, weight) in dataset.messages() {
            for message in messages.messages() {
                let unigram = Unigram::construct(message);

                for i in 0..unigram.len() - 1 {
                    *unigrams.entry(unigram[i])
                        .or_default()
                        .entry(unigram[i + 1])
                        .or_default() += *weight;
                }

                if let Some(bigrams) = &mut bigrams {
                    let bigram = Bigram::construct(message);

                    for i in 0..bigram.len() - 1 {
                        *bigrams.entry(bigram[i])
                            .or_default()
                            .entry(bigram[i + 1])
                            .or_default() += *weight;
                    }
                }

                if let Some(trigrams) = &mut trigrams {
                    let trigram = Trigram::construct(message);

                    for i in 0..trigram.len() - 1 {
                        *trigrams.entry(trigram[i])
                            .or_default()
                            .entry(trigram[i + 1])
                            .or_default() += *weight;
                    }
                }
            }
        }

        Self {
            unigrams,
            bigrams,
            trigrams
        }
    }

    #[inline]
    pub fn unigrams_len(&self) -> usize {
        self.unigrams.len()
    }

    #[inline]
    pub fn bigrams_len(&self) -> Option<usize> {
        Some(self.bigrams.as_ref()?.len())
    }

    #[inline]
    pub fn trigrams_len(&self) -> Option<usize> {
        Some(self.trigrams.as_ref()?.len())
    }

    #[inline]
    pub fn for_unigram(&self, unigram: &Unigram) -> Option<impl Iterator<Item = (&'_ Unigram, &'_ u64)>> {
        self.unigrams.get(unigram).map(|transitions| transitions.iter())
    }

    #[inline]
    pub fn for_bigram(&self, bigram: &Bigram) -> Option<impl Iterator<Item = (&'_ Bigram, &'_ u64)>> {
        self.bigrams.as_ref()?.get(bigram).map(|transitions| transitions.iter())
    }

    #[inline]
    pub fn for_trigram(&self, trigram: &Trigram) -> Option<impl Iterator<Item = (&'_ Trigram, &'_ u64)>> {
        self.trigrams.as_ref()?.get(trigram).map(|transitions| transitions.iter())
    }

    #[inline]
    /// Get probability of the (current_ngram -> next_ngram)
    pub fn calc_unigram_probability(&self, current_ngram: &Unigram, next_ngram: &Unigram) -> Option<f64> {
        self.unigrams.get(current_ngram)
            .and_then(|transitions| {
                transitions.get(next_ngram).map(|count| (count, transitions.len()))
            })
            .map(|(count, total)| *count as f64 / total as f64)
    }

    #[inline]
    /// Get probability of the (current_ngram -> next_ngram)
    pub fn calc_bigram_probability(&self, current_ngram: &Bigram, next_ngram: &Bigram) -> Option<f64> {
        self.bigrams.as_ref()?
            .get(current_ngram)
            .and_then(|transitions| {
                transitions.get(next_ngram).map(|count| (count, transitions.len()))
            })
            .map(|(count, total)| *count as f64 / total as f64)
    }

    #[inline]
    /// Get probability of the (current_ngram -> next_ngram)
    pub fn calc_trigram_probability(&self, current_ngram: &Trigram, next_ngram: &Trigram) -> Option<f64> {
        self.trigrams.as_ref()?
            .get(current_ngram)
            .and_then(|transitions| {
                transitions.get(next_ngram).map(|count| (count, transitions.len()))
            })
            .map(|(count, total)| *count as f64 / total as f64)
    }

    #[inline]
    /// Calculate average amount of paths per unigram
    pub fn calc_avg_unigram_paths(&self) -> f64 {
        let paths = self.unigrams.par_iter()
            .filter(|(k, _)| !k.is_start() && !k.is_end())
            .map(|(_, transitions)| transitions.par_iter())
            .map(|transitions| transitions.filter(|(k, _)| !k.is_start() && !k.is_end()))
            .map(|transitions| transitions.count() as u64)
            .sum::<u64>();

        paths as f64 / self.unigrams_len() as f64
    }

    #[inline]
    /// Calculate average amount of paths per bigram
    pub fn calc_avg_bigram_paths(&self) -> Option<f64> {
        let paths = self.bigrams.as_ref()?
            .par_iter()
            .filter(|(k, _)| !k.is_start() && !k.is_end())
            .map(|(_, transitions)| transitions.par_iter())
            .map(|transitions| transitions.filter(|(k, _)| !k.is_start() && !k.is_end()))
            .map(|transitions| transitions.count() as u64)
            .sum::<u64>();

        Some(paths as f64 / self.bigrams_len()? as f64)
    }

    #[inline]
    /// Calculate average amount of paths per trigram
    pub fn calc_avg_trigram_paths(&self) -> Option<f64> {
        let paths = self.trigrams.as_ref()?
            .par_iter()
            .filter(|(k, _)| !k.is_start() && !k.is_end())
            .map(|(_, transitions)| transitions.par_iter())
            .map(|transitions| transitions.filter(|(k, _)| !k.is_start() && !k.is_end()))
            .map(|transitions| transitions.count() as u64)
            .sum::<u64>();

        Some(paths as f64 / self.trigrams_len()? as f64)
    }

    #[inline]
    /// Calculate variety of the unigrams chain
    pub fn calc_unigram_variety(&self) -> f64 {
        let avg_paths = self.calc_avg_unigram_paths();

        let more_than_avg_paths = self.unigrams.par_iter()
            .filter(|(k, _)| !k.is_start() && !k.is_end())
            .map(|(_, transitions)| transitions.keys())
            .map(|ngrams| ngrams.filter(|ngram| !ngram.is_start() && !ngram.is_end()))
            .map(|ngrams| ngrams.count() as f64)
            .filter(|count| *count > avg_paths)
            .count();

        more_than_avg_paths as f64 / self.unigrams_len() as f64
    }

    #[inline]
    /// Calculate variety of the unigrams chain
    pub fn calc_bigram_variety(&self) -> Option<f64> {
        let avg_paths = self.calc_avg_bigram_paths()?;

        let more_than_avg_paths = self.bigrams.as_ref()?
            .par_iter()
            .filter(|(k, _)| !k.is_start() && !k.is_end())
            .map(|(_, transitions)| transitions.keys())
            .map(|ngrams| ngrams.filter(|ngram| !ngram.is_start() && !ngram.is_end()))
            .map(|ngrams| ngrams.count() as f64)
            .filter(|count| *count > avg_paths)
            .count();

        Some(more_than_avg_paths as f64 / self.bigrams_len()? as f64)
    }

    #[inline]
    /// Calculate variety of the trigrams chain
    pub fn calc_trigram_variety(&self) -> Option<f64> {
        let avg_paths = self.calc_avg_trigram_paths()?;

        let more_than_avg_paths = self.trigrams.as_ref()?
            .par_iter()
            .filter(|(k, _)| !k.is_start() && !k.is_end())
            .map(|(_, transitions)| transitions.keys())
            .map(|ngrams| ngrams.filter(|ngram| !ngram.is_start() && !ngram.is_end()))
            .map(|ngrams| ngrams.count() as f64)
            .filter(|count| *count > avg_paths)
            .count();

        Some(more_than_avg_paths as f64 / self.trigrams_len()? as f64)
    }
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
        let transitions = dataset.build_transitions(false, false);

        let hello = dataset.tokens.find_token("hello,").unwrap();
        let world = dataset.tokens.find_token("world!").unwrap();
        let example = dataset.tokens.find_token("example").unwrap();
        let text = dataset.tokens.find_token("text").unwrap();

        let hello = Unigram::new([hello]);
        let world = Unigram::new([world]);
        let example = Unigram::new([example]);
        let text = Unigram::new([text]);

        assert_eq!(transitions.for_unigram(&hello).map(|t| t.collect::<Vec<_>>()), Some(vec![(&world, &1)]));
        assert_eq!(transitions.for_unigram(&example).map(|t| t.collect::<Vec<_>>()), Some(vec![(&text, &1)]));

        assert_eq!(transitions.calc_unigram_probability(&hello, &world), Some(1.0));
        assert_eq!(transitions.calc_unigram_probability(&example, &text), Some(1.0));

        Ok(())
    }
}
