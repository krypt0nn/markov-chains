use std::io::Read;
use std::iter::FusedIterator;

use crate::prelude::*;

pub mod newline;
pub mod discord_export;

pub mod prelude {
    pub use super::{
        Parser,
        ParsedMessagesStorage
    };

    pub use super::newline::{
        NewlineParser,
        NewlineParserOptions
    };
}

/// Parsers are used to convert input plaintext data
/// into separate messages. Meaning of "message" depends
/// on an actual implementation of splitter.
pub trait Parser {
    /// Options of the splitter.
    type Options;

    /// Result iterator of the splitter.
    type Iter: Iterator<Item = String> + FusedIterator;

    /// Create iterator of parsed messages over the given
    /// input plaintext reader.
    fn parse(reader: Box<dyn Read>, options: Self::Options) -> Self::Iter;
}

/// Storage of the parsed messages.
///
/// ```
/// use markov_chains::prelude::*;
///
/// let mut storage = ParsedMessagesStorage::create(
///     "parser.storage",
///     Compression::Zstd,
///     128
/// ).unwrap();
///
/// storage.insert("Message 1").unwrap();
/// storage.insert("Message 2").unwrap();
/// storage.insert("Message 3").unwrap();
///
/// // Storage keeps inserted values in RAM buffer and writes it on disk
/// // when buffer is full or when the struct is dropped. Here we flush
/// // this buffer on disk to write all the inserted messages and read them
/// // immeadiately a line later.
/// storage.flush().unwrap();
///
/// let mut reader = storage.read().unwrap();
///
/// assert_eq!(reader.next(), Some(String::from("Message 1")));
/// assert_eq!(reader.next(), Some(String::from("Message 2")));
/// assert_eq!(reader.next(), Some(String::from("Message 3")));
/// assert_eq!(reader.next(), None);
///
/// std::fs::remove_file("parser.storage").unwrap();
/// ```
pub type ParsedMessagesStorage = Storage<String>;
