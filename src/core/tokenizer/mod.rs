use std::iter::FusedIterator;

use crate::prelude::*;

pub mod hashing;

pub mod prelude {
    pub use super::Tokenizer;
    // pub use super::hashing::HashingTokenizer;
}

/// Tokenizers are used to convert vectors of words (messages)
/// into vector of another entities (numbers) to store and process
/// them more efficiently.
pub trait Tokenizer {
    /// Token type of the tokenizer.
    type Token: PartialEq + Eq;

    /// Iterator of tokens of the message.
    type TokenizeIter: Iterator<Item = Self::Token> + FusedIterator;

    /// Iterator of words of the tokenized message.
    type DetokenizeIter<'a>: Iterator<Item = MessageWord> + FusedIterator;

    /// Convert given list of words into vector of tokens.
    fn tokenize(&self, message: Message) -> Self::TokenizeIter;

    /// Convert given list of tokens back into vector of words.
    fn detokenize<'a>(&self, tokens: impl IntoIterator<Item = Self::Token> + 'a) -> Self::DetokenizeIter<'a>;
}
