use std::iter::FusedIterator;

use crate::prelude::*;

pub mod whitespace;
pub mod natural_language;

pub mod prelude {
    pub use super::{
        Preprocessor,
        PreprocessedMessagesStorage
    };

    pub use super::whitespace::{
        WhitespacePreprocessor,
        WhitespacePreprocessorOptions
    };

    pub use super::natural_language::{
        NaturalLanguagePreprocessor,
        NaturalLanguagePreprocessorOptions
    };
}

/// Preprocessors are used to convert messages
/// into vectors of pre-processed words.
pub trait Preprocessor {
    /// Options of the preprocessor.
    type Options;

    /// Result iterator of the preprocessor.
    type Iter: Iterator<Item = Message> + FusedIterator;

    /// Create iterator of split words over the
    /// given input messages iterator.
    fn process(messages: Box<dyn Iterator<Item = String>>, options: Self::Options) -> Self::Iter;

    /// Combine list of words to the message.
    fn combine(words: impl IntoIterator<Item = MessageWord>) -> String {
        words.into_iter()
            .fold(String::new(), |acc, word| format!("{acc}{word} "))
            .trim_ascii_end()
            .to_string()
    }
}

/// Storage of the preprocessed messages.
///
/// ```
/// use markov_chains::prelude::*;
///
/// let mut storage = PreprocessedMessagesStorage::create(
///     "preprocessor.storage",
///     Compression::Zstd,
///     128
/// ).unwrap();
///
/// storage.insert(Message::new_raw("Message 1").unwrap()).unwrap();
/// storage.insert(Message::new_raw("Message 2").unwrap()).unwrap();
/// storage.insert(Message::new_raw("Message 3").unwrap()).unwrap();
///
/// // Storage keeps inserted values in RAM buffer and writes it on disk
/// // when buffer is full or when the struct is dropped. Here we flush
/// // this buffer on disk to write all the inserted messages and read them
/// // immeadiately a line later.
/// storage.flush().unwrap();
///
/// let mut reader = storage.read().unwrap();
///
/// assert_eq!(reader.next().map(|msg| msg.to_string()), Some(String::from("Message 1")));
/// assert_eq!(reader.next().map(|msg| msg.to_string()), Some(String::from("Message 2")));
/// assert_eq!(reader.next().map(|msg| msg.to_string()), Some(String::from("Message 3")));
/// assert_eq!(reader.next(), None);
///
/// std::fs::remove_file("preprocessor.storage").unwrap();
/// ```
pub type PreprocessedMessagesStorage = Storage<Message>;
