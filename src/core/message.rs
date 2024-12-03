use tinyvec::TinyVec;
use strumbra::UniqueString;

use super::serialize::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Single message's word optimized for fast in-ram operations.
pub struct MessageWord {
    word: UniqueString
}

impl MessageWord {
    /// Create new message word from the given unicode string.
    /// This function will remove all the whitespace characters
    /// in beginning and end of the word. Use `new_raw` method
    /// if you don't want to do this.
    ///
    /// Returns Err if the given word can't fit into 2^32 bits
    /// or if it's empty.
    pub fn new(word: impl AsRef<str>) -> anyhow::Result<Self> {
        let word = word.as_ref().trim_ascii();

        if word.is_empty() {
            anyhow::bail!("Given message word is empty");
        }

        Self::new_raw(word)
    }

    /// Create new message word from the given unicode string
    /// without additional transformations (whitespace removing)
    /// and checks.
    ///
    /// Returns Err if the given word can't fit into 2^32 bits.
    pub fn new_raw(word: impl AsRef<str>) -> anyhow::Result<Self> {
        Ok(Self {
            word: UniqueString::try_from(word.as_ref())?
        })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.word.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.word.is_empty()
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.word.as_bytes()
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.word.as_str()
    }
}

impl Default for MessageWord {
    fn default() -> Self {
        // SAFETY: UniqueString::try_from only checks that the length
        // can fit into u32.
        unsafe {
            Self::new_raw("").unwrap_unchecked()
        }
    }
}

impl std::fmt::Display for MessageWord {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.word)
    }
}

impl Serialize for MessageWord {
    #[inline]
    fn to_bytes(&self) -> Vec<u8> {
        self.word.as_bytes().to_vec()
    }

    fn from_bytes(bytes: impl AsRef<[u8]>) -> Option<Self> where Self: Sized {
        let str = String::from_utf8(bytes.as_ref().to_vec()).ok();

        Some(Self {
            word: UniqueString::try_from(str?).ok()?
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
/// Interpretation of a single plaintext message optimized
/// for fast in-ram operations. Contains vector of words.
pub struct Message {
    // We've calculated that people on our discord server
    // send 5 words on average (median of all messages).
    words: TinyVec<[MessageWord; 8]>
}

impl Message {
    /// Parse message words from the given unicode text
    /// and apply the formatting function on it.
    pub fn new(message: impl AsRef<str>, formatter: impl Fn(&str) -> String + Copy + Send + Sync + 'static) -> anyhow::Result<Self> {
        let message = message.as_ref();

        Ok(Self {
            words: message.split_whitespace()
                .map(|word| {
                    MessageWord::new(formatter(word))
                })
                .collect::<Result<TinyVec<_>, _>>()?
        })
    }

    /// Parse message words from the given unicode text.
    pub fn new_raw(message: impl AsRef<str>) -> anyhow::Result<Self> {
        let message = message.as_ref();

        Ok(Self {
            words: message.split_whitespace()
                .map(MessageWord::new)
                .collect::<Result<TinyVec<_>, _>>()?
        })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.words.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    #[inline]
    pub fn words(&self) -> &[MessageWord] {
        self.words.as_slice()
    }
}

impl FromIterator<MessageWord> for Message {
    #[inline]
    fn from_iter<T: IntoIterator<Item = MessageWord>>(iter: T) -> Self {
        Self {
            words: iter.into_iter()
                .collect::<TinyVec<_>>()
        }
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = self.words.iter()
            .map(MessageWord::to_string)
            .fold(String::new(), |str, word| format!("{str}{word} "));

        f.write_str(message.trim_ascii_end())
    }
}

impl Serialize for Message {
    #[inline]
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        for word in self.words() {
            let word = word.as_bytes();

            assert!(word.len() <= u8::MAX as usize, "Words in messages must be shorter than 256 bytes");

            bytes.push(word.len() as u8);
            bytes.extend(word);
        }

        bytes
    }

    fn from_bytes(bytes: impl AsRef<[u8]>) -> Option<Self> where Self: Sized {
        let bytes = bytes.as_ref();

        let mut i = 0;
        let n = bytes.len();

        // Assume median length of a word is 4 chars.
        let mut words = Vec::with_capacity(n / 4);

        while i < n {
            let len = bytes[i] as usize;

            let word = String::from_utf8(bytes[i + 1..i + len + 1].to_vec()).ok()?;

            words.push(MessageWord::new_raw(word).ok()?);

            i += len + 1;
        }

        Some(Message::from_iter(words))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() -> anyhow::Result<()> {
        assert_eq!(Message::new_raw("Hello, World!")?.to_string(), "Hello, World!");
        assert_eq!(Message::new("Hello, World!", |word| word.to_ascii_lowercase())?.to_string(), "hello, world!");

        assert_eq!(Message::new_raw("a  b  c \n d \t\n\re")?.len(), 5);

        assert_eq!(MessageWord::new_raw("123")?, MessageWord::new_raw("123")?);
        assert_ne!(MessageWord::new_raw("123")?, MessageWord::new_raw("456")?);

        assert_eq!(Message::new_raw("Hello, World!")?, Message::new_raw("Hello, World!")?);
        assert_ne!(Message::new_raw("Hello, World!")?, Message::new_raw("Goodbye, World!")?);

        Ok(())
    }
}
