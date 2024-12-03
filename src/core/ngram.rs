use super::tokens::Tokens;

pub type Unigram = Ngram<1>;
pub type Bigram  = Ngram<2>;
pub type Trigram = Ngram<3>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ngram<const SIZE: usize>([u64; SIZE]);

impl<const SIZE: usize> Ngram<SIZE> {
    pub const SIZE: usize = SIZE;

    #[inline]
    pub const fn new(tokens: [u64; SIZE]) -> Self {
        Self(tokens)
    }

    #[inline]
    pub const fn into_inner(self) -> [u64; SIZE] {
        self.0
    }

    #[inline]
    pub const fn start() -> Self {
        Self::new([Tokens::START_TOKEN; SIZE])
    }

    #[inline]
    pub fn is_start(&self) -> bool {
        self.0 == [Tokens::START_TOKEN; SIZE]
    }

    #[inline]
    pub fn is_end(&self) -> bool {
        self.0.contains(&Tokens::END_TOKEN)
    }

    #[inline]
    pub fn token(&self) -> u64 {
        if self.is_end() && SIZE > 1 {
            self.0[SIZE - 2]
        } else {
            self.0[SIZE - 1]
        }
    }

    #[inline]
    pub fn head(&self) -> &[u64] {
        &self.0[..SIZE - 1]
    }

    #[inline]
    pub fn tail(&self) -> &[u64] {
        &self.0[1..]
    }

    /// Construct list of ngrams from list of tokens
    pub fn construct(tokens: &[u64]) -> Vec<Self> {
        let mut extended_tokens = Vec::with_capacity(tokens.len() + SIZE + 1);
        let mut ngrams = Vec::with_capacity(extended_tokens.len());

        extended_tokens.extend([Tokens::START_TOKEN; SIZE]);
        extended_tokens.extend(tokens);
        extended_tokens.push(Tokens::END_TOKEN);

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
        let mut ngrams = Self::construct(tokens);

        ngrams.pop();

        ngrams
    }

    /// Deconstruct list of ngrams into list of tokens
    pub fn deconstruct(ngrams: &[Self]) -> Vec<u64> {
        let mut tokens = Vec::with_capacity(ngrams.len());

        for ngram in ngrams.iter().take(ngrams.len().saturating_sub(1)) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unigram() {
        assert_eq!(&Unigram::construct(&[]), &[
            Unigram::start(),
            Unigram::new([Tokens::END_TOKEN])
        ]);

        assert_eq!(&Unigram::construct(&[1]), &[
            Unigram::start(),
            Unigram::new([1]),
            Unigram::new([Tokens::END_TOKEN])
        ]);

        assert_eq!(&Unigram::construct(&[1, 2]), &[
            Unigram::start(),
            Unigram::new([1]),
            Unigram::new([2]),
            Unigram::new([Tokens::END_TOKEN])
        ]);

        assert_eq!(&Unigram::construct(&[1, 2, 3]), &[
            Unigram::start(),
            Unigram::new([1]),
            Unigram::new([2]),
            Unigram::new([3]),
            Unigram::new([Tokens::END_TOKEN])
        ]);
    }

    #[test]
    fn bigram() {
        assert_eq!(&Bigram::construct(&[]), &[
            Bigram::start(),
            Bigram::new([Tokens::START_TOKEN, Tokens::END_TOKEN])
        ]);

        assert_eq!(&Bigram::construct(&[1]), &[
            Bigram::start(),
            Bigram::new([Tokens::START_TOKEN, 1]),
            Bigram::new([1, Tokens::END_TOKEN])
        ]);

        assert_eq!(&Bigram::construct(&[1, 2]), &[
            Bigram::start(),
            Bigram::new([Tokens::START_TOKEN, 1]),
            Bigram::new([1, 2]),
            Bigram::new([2, Tokens::END_TOKEN])
        ]);

        assert_eq!(&Bigram::construct(&[1, 2, 3]), &[
            Bigram::start(),
            Bigram::new([Tokens::START_TOKEN, 1]),
            Bigram::new([1, 2]),
            Bigram::new([2, 3]),
            Bigram::new([3, Tokens::END_TOKEN])
        ]);
    }

    #[test]
    fn trigram() {
        assert_eq!(&Trigram::construct(&[]), &[
            Trigram::start(),
            Trigram::new([Tokens::START_TOKEN, Tokens::START_TOKEN, Tokens::END_TOKEN])
        ]);

        assert_eq!(&Trigram::construct(&[1]), &[
            Trigram::start(),
            Trigram::new([Tokens::START_TOKEN, Tokens::START_TOKEN, 1]),
            Trigram::new([Tokens::START_TOKEN, 1, Tokens::END_TOKEN])
        ]);

        assert_eq!(&Trigram::construct(&[1, 2]), &[
            Trigram::start(),
            Trigram::new([Tokens::START_TOKEN, Tokens::START_TOKEN, 1]),
            Trigram::new([Tokens::START_TOKEN, 1, 2]),
            Trigram::new([1, 2, Tokens::END_TOKEN])
        ]);

        assert_eq!(&Trigram::construct(&[1, 2, 3]), &[
            Trigram::start(),
            Trigram::new([Tokens::START_TOKEN, Tokens::START_TOKEN, 1]),
            Trigram::new([Tokens::START_TOKEN, 1, 2]),
            Trigram::new([1, 2, 3]),
            Trigram::new([2, 3, Tokens::END_TOKEN])
        ]);
    }

    #[test]
    fn construct() {
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
