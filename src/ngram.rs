use crate::prelude::{
    START_TOKEN,
    END_TOKEN
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ngram<const SIZE: usize>([u64; SIZE]);

impl<const SIZE: usize> Ngram<SIZE> {
    pub const SIZE: usize = SIZE;

    #[inline]
    pub fn new(tokens: [u64; SIZE]) -> Self {
        Self(tokens)
    }

    #[inline]
    pub fn start() -> Self {
        Self::new([START_TOKEN; SIZE])
    }

    #[inline]
    pub fn end() -> Self {
        Self::new([END_TOKEN; SIZE])
    }

    #[inline]
    pub fn is_start(&self) -> bool {
        self.0 == [START_TOKEN; SIZE]
    }

    #[inline]
    pub fn is_end(&self) -> bool {
        self.0 == [END_TOKEN; SIZE]
    }

    #[inline]
    pub fn is_part_start(&self) -> bool {
        self.0.contains(&START_TOKEN)
    }

    #[inline]
    pub fn is_part_end(&self) -> bool {
        self.0.contains(&END_TOKEN)
    }

    #[inline]
    pub fn token(&self) -> u64 {
        self.0[SIZE - 1]
    }

    /// Construct list of ngrams from list of tokens
    pub fn construct(tokens: &[u64]) -> Vec<Self> {
        let mut extended_tokens = Vec::with_capacity(tokens.len() + SIZE + 1);
        let mut ngrams = Vec::with_capacity(extended_tokens.len());

        extended_tokens.extend([START_TOKEN; SIZE]);
        extended_tokens.extend(tokens);
        extended_tokens.extend([END_TOKEN; SIZE]);

        let n = extended_tokens.len();

        for i in 0..n - SIZE + 1 {
            let mut ngram = [0; SIZE];

            ngram.copy_from_slice(&extended_tokens[i..i + SIZE]);

            ngrams.push(Self::new(ngram));
        }

        ngrams
    }

    /// Construct list of ngrams from list of tokens without the ending tail
    pub fn construct_tailless(tokens: &[u64]) -> Vec<Self> {
        let mut extended_tokens = Vec::with_capacity(tokens.len() + SIZE + 1);
        let mut ngrams = Vec::with_capacity(extended_tokens.len());

        extended_tokens.extend([START_TOKEN; SIZE]);
        extended_tokens.extend(tokens);

        let n = extended_tokens.len();

        for i in 0..n - SIZE + 1 {
            let mut ngram = [0; SIZE];

            ngram.copy_from_slice(&extended_tokens[i..i + SIZE]);

            ngrams.push(Self::new(ngram));
        }

        ngrams
    }

    /// Deconstruct list of ngrams into list of tokens
    pub fn deconstruct(ngrams: &[Self]) -> Vec<u64> {
        let mut tokens = Vec::with_capacity(ngrams.len());

        for ngram in ngrams.iter().take(ngrams.len() - SIZE) {
            if ngram.is_start() {
                continue;
            }

            else if ngram.is_end() {
                break;
            }

            else {
                tokens.push(ngram.0[SIZE - 1]);
            }
        }

        tokens
    }
}

impl<const SIZE: usize> serde::Serialize for Ngram<SIZE> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        self.0.as_ref().serialize(serializer)
    }
}

impl<'de, const SIZE: usize> serde::Deserialize<'de> for Ngram<SIZE> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let tokens = Vec::<u64>::deserialize(deserializer)?;

        if tokens.len() != SIZE {
            return Err(serde::de::Error::custom(format!("Expected {} tokens for ngram, got {}", SIZE, tokens.len())));
        }

        let mut ngram = [0; SIZE];

        ngram.copy_from_slice(&tokens[..SIZE]);

        Ok(Self::new(ngram))
    }
}

pub type Unigram = Ngram<1>;
pub type Bigram  = Ngram<2>;
pub type Trigram = Ngram<3>;

mod tests {
    #[test]
    fn unigram() {
        use super::Unigram;

        assert_eq!(&Unigram::construct(&[]), &[
            Unigram::start(),
            Unigram::end()
        ]);

        assert_eq!(&Unigram::construct(&[1]), &[
            Unigram::start(),
            Unigram::new([1]),
            Unigram::end()
        ]);

        assert_eq!(&Unigram::construct(&[1, 2]), &[
            Unigram::start(),
            Unigram::new([1]),
            Unigram::new([2]),
            Unigram::end()
        ]);

        assert_eq!(&Unigram::construct(&[1, 2, 3]), &[
            Unigram::start(),
            Unigram::new([1]),
            Unigram::new([2]),
            Unigram::new([3]),
            Unigram::end()
        ]);
    }

    #[test]
    fn bigram() {
        use super::{Bigram, START_TOKEN, END_TOKEN};

        assert_eq!(&Bigram::construct(&[]), &[
            Bigram::start(),
            Bigram::new([START_TOKEN, END_TOKEN]),
            Bigram::end()
        ]);

        assert_eq!(&Bigram::construct(&[1]), &[
            Bigram::start(),
            Bigram::new([START_TOKEN, 1]),
            Bigram::new([1, END_TOKEN]),
            Bigram::end()
        ]);

        assert_eq!(&Bigram::construct(&[1, 2]), &[
            Bigram::start(),
            Bigram::new([START_TOKEN, 1]),
            Bigram::new([1, 2]),
            Bigram::new([2, END_TOKEN]),
            Bigram::end()
        ]);

        assert_eq!(&Bigram::construct(&[1, 2, 3]), &[
            Bigram::start(),
            Bigram::new([START_TOKEN, 1]),
            Bigram::new([1, 2]),
            Bigram::new([2, 3]),
            Bigram::new([3, END_TOKEN]),
            Bigram::end()
        ]);
    }

    #[test]
    fn trigram() {
        use super::{Trigram, START_TOKEN, END_TOKEN};

        assert_eq!(&Trigram::construct(&[]), &[
            Trigram::start(),
            Trigram::new([START_TOKEN, START_TOKEN, END_TOKEN]),
            Trigram::new([START_TOKEN, END_TOKEN, END_TOKEN]),
            Trigram::end()
        ]);

        assert_eq!(&Trigram::construct(&[1]), &[
            Trigram::start(),
            Trigram::new([START_TOKEN, START_TOKEN, 1]),
            Trigram::new([START_TOKEN, 1, END_TOKEN]),
            Trigram::new([1, END_TOKEN, END_TOKEN]),
            Trigram::end()
        ]);

        assert_eq!(&Trigram::construct(&[1, 2]), &[
            Trigram::start(),
            Trigram::new([START_TOKEN, START_TOKEN, 1]),
            Trigram::new([START_TOKEN, 1, 2]),
            Trigram::new([1, 2, END_TOKEN]),
            Trigram::new([2, END_TOKEN, END_TOKEN]),
            Trigram::end()
        ]);

        assert_eq!(&Trigram::construct(&[1, 2, 3]), &[
            Trigram::start(),
            Trigram::new([START_TOKEN, START_TOKEN, 1]),
            Trigram::new([START_TOKEN, 1, 2]),
            Trigram::new([1, 2, 3]),
            Trigram::new([2, 3, END_TOKEN]),
            Trigram::new([3, END_TOKEN, END_TOKEN]),
            Trigram::end()
        ]);
    }

    #[test]
    fn construct() {
        use super::{
            Unigram,
            Bigram,
            Trigram
        };

        let tokens = &[
            vec![],
            vec![1],
            vec![1, 2],
            vec![1, 2, 3],
            vec![1, 2, 3, 4],
            vec![1, 2, 3, 4, 5]
        ];

        for tokens in tokens {
            assert_eq!(&Unigram::deconstruct(&Unigram::construct(tokens)), tokens);
            assert_eq!(&Bigram::deconstruct(&Bigram::construct(tokens)), tokens);
            assert_eq!(&Trigram::deconstruct(&Trigram::construct(tokens)), tokens);
        }
    }
}