// use std::collections::HashMap;
// use std::rc::Rc;

// use strumbra::UniqueString;

// use super::*;

// pub struct HashingTokenizerIter<'a> {
//     message: Box<dyn Iterator<Item = UniqueString> + 'a>
// }

// impl<'a> Iterator for HashingTokenizerIter<'a> {
//     type Item = u32;

//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         self.message.next().map(|word| crc32fast::hash(word.as_bytes()))
//     }
// }

// impl<'a> FusedIterator for HashingTokenizerIter<'a> {}

// pub struct HashingDetokenizerIter<'a> {
//     message: Box<dyn Iterator<Item = u32> + 'a>,
//     values: Rc<HashMap<u32, UniqueString>>
// }

// impl<'a> Iterator for HashingDetokenizerIter<'a> {
//     type Item = UniqueString;

//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         self.message.next()
//             .and_then(|token| self.values.get(&token))
//             .cloned()
//     }
// }

// impl<'a> FusedIterator for HashingDetokenizerIter<'a> {}

// pub struct HashingTokenizer {
//     values: Rc<HashMap<u32, UniqueString>>
// }

// impl Tokenizer for HashingTokenizer {
//     type Token = u32;
//     type TokenizeIter<'a> = HashingTokenizerIter<'a>;
//     type DetokenizeIter<'a> = HashingDetokenizerIter<'a>;

//     #[inline]
//     fn tokenize<'a>(&self, message: impl IntoIterator<Item = UniqueString> + 'a) -> Self::TokenizeIter<'a> {
//         HashingTokenizerIter {
//             message: Box::new(message.into_iter())
//         }
//     }

//     #[inline]
//     fn detokenize<'a>(&self, tokens: impl IntoIterator<Item = Self::Token> + 'a) -> Self::DetokenizeIter<'a> {
//         HashingDetokenizerIter {
//             message: Box::new(tokens.into_iter()),
//             values: self.values.clone()
//         }
//     }
// }
