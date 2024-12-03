use tinyvec::TinyVec;
use intmap::IntMap;

/// General trait for representing different tokenized
/// enums in the same numeric format.
pub trait Token: Default {
    /// Encode token into 4 bytes.
    fn encode(&self) -> u32;

    /// Decode token from 4 bytes.
    fn decode(token: u32) -> Self;
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
/// Interpretation of a single tokenized message optimized
/// for fast in-ram operations. Contains vector of tokens.
pub struct TokenizedMessage<T: Token> {
    tokens: TinyVec<[T; 8]>
}

impl<T: Token> TokenizedMessage<T> {
    #[inline]
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    #[inline]
    pub fn tokens(&self) -> &[T] {
        &self.tokens
    }
}

impl<T: Token> FromIterator<T> for TokenizedMessage<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            tokens: iter.into_iter().collect::<TinyVec<_>>()
        }
    }
}
