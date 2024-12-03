use std::path::PathBuf;

use smol::io::AsyncWriteExt;

use super::message::Message;
use super::ngram::*;

#[derive(Debug)]
/// Struct that holds information about independent messages
/// and allows to split them into smaller datasets.
pub struct Dataset {
    path: PathBuf,
    file: smol::fs::File
}

impl Dataset {
    pub async fn open(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path: PathBuf = path.into();

        if !path.exists() {
            if let Some(parent) = path.parent() {
                smol::fs::create_dir_all(parent).await?;
            }
        }

        let file = smol::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .await?;

        Ok(Self {
            path,
            file
        })
    }

    #[inline]
    pub async fn insert(&mut self, message: Message, weight: f64) {
        let words = message.into_inner();

        let mut record = Vec::with_capacity(1024);
        let mut tokens = Vec::with_capacity(words.len());

        // Expect individual messages to have 65k words at max.
        let len = tokens.len() as u16;

        record.extend_from_slice(&len.to_be_bytes());

        for word in words {
            // Expect text words to be 255 chars long at max.
            let len = word.len() as u8;

            record.push(len);
            record.extend_from_slice(word.as_bytes());

            tokens.push(word.as_token());
        }

        let unigrams = Unigram::construct(&tokens);
        let bigrams = Bigram::construct(&tokens);
        let trigrams = Trigram::construct(&tokens);

        // There should not be more unigrams that words.
        let len = unigrams.len() as u16;

        record.extend_from_slice(&len.to_be_bytes());

        for unigram in unigrams {
            for token in unigram.into_inner() {
                record.extend_from_slice(&token.to_be_bytes());
            }
        }

        // There can be more bigrams and trigrams, so 4 bytes for length.
        let len = bigrams.len() as u32;

        record.extend_from_slice(&len.to_be_bytes());

        for bigram in bigrams {
            for token in bigram.into_inner() {
                record.extend_from_slice(&token.to_be_bytes());
            }
        }

        let len = trigrams.len() as u32;

        record.extend_from_slice(&len.to_be_bytes());

        for trigram in trigrams {
            for token in trigram.into_inner() {
                record.extend_from_slice(&token.to_be_bytes());
            }
        }

        self.file.write()

        self.file.write_all()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    #[inline]
    pub fn merge(&mut self, dataset: Self) {
        for message in dataset.messages {
            self.messages.insert(message);
        }
    }

    /// Split dataset into two different ones with provides fraction,
    /// randomly choosing messages from the current one.
    ///
    /// Fraction 0.8 means roughly 80% left, 20% right.
    ///
    /// Return None if invalid fraction provided.
    pub fn split(mut self, fraction: f32) -> Option<(Self, Self)> {
        if !(0.0..=1.0).contains(&fraction) {
            return None;
        }

        let mut left = HashSet::with_capacity((self.messages.len() as f32 * (1.0 - fraction)) as usize);
        let mut right = HashSet::with_capacity((self.messages.len() as f32 * fraction) as usize);

        for message in self.messages.drain() {
            if fastrand::f32() > fraction {
                right.insert(message);
            } else {
                left.insert(message);
            }
        }

        Some((Self { messages: left }, Self { messages: right }))
    }
}
