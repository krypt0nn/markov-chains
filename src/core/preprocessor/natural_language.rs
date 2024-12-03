use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NaturalLanguagePreprocessorOptions {
    /// Convert all the characters into lowercase.
    pub lowercase: bool
}

pub struct NaturalLanguagePreprocessorIter {
    messages: Box<dyn Iterator<Item = String>>,
    options: NaturalLanguagePreprocessorOptions
}

impl Iterator for NaturalLanguagePreprocessorIter {
    type Item = Message;

    fn next(&mut self) -> Option<Self::Item> {
        let message = self.messages.next()?;

        let mut words = Vec::with_capacity(message.len() / 4);

        for word in message.split_whitespace() {
            let mut word = word.to_string();

            if self.options.lowercase {
                word = word.to_lowercase();
            }

            let mut part = String::with_capacity(word.len());

            for char in word.chars() {
                if char.is_ascii_punctuation() {
                    if !part.is_empty() {
                        words.push(MessageWord::new_raw(&part).ok()?);
                    }

                    words.push(MessageWord::new_raw(char.to_string()).ok()?);

                    part.clear();
                }

                else {
                    part.push(char);
                }
            }

            words.push(MessageWord::new_raw(" ").ok()?);
        }

        if let Some(word) = words.last() {
            if word.as_str() == " " {
                words.pop();
            }
        }

        Some(Message::from_iter(words))
    }
}

impl FusedIterator for NaturalLanguagePreprocessorIter {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Split messages into individual words
/// and punctuations.
///
/// `"Hello, World!" -> ["Hello", ",", " ", "World", "!"]`
///
/// ```
/// use markov_chains::prelude::*;
///
/// let message = NaturalLanguagePreprocessor::process(
///     Box::new([String::from("Hello, World!")].into_iter()),
///     NaturalLanguagePreprocessorOptions {
///         lowercase: false
///     }
/// ).next().unwrap();
///
/// let words = message.words();
///
/// assert_eq!(words.len(), 5);
/// assert_eq!(words[0].as_str(), "Hello");
/// assert_eq!(words[1].as_str(), ",");
/// assert_eq!(words[2].as_str(), " ");
/// assert_eq!(words[3].as_str(), "World");
/// assert_eq!(words[4].as_str(), "!");
/// ```
pub struct NaturalLanguagePreprocessor;

impl Preprocessor for NaturalLanguagePreprocessor {
    type Options = NaturalLanguagePreprocessorOptions;
    type Iter = NaturalLanguagePreprocessorIter;

    #[inline]
    fn process(messages: Box<dyn Iterator<Item = String>>, options: Self::Options) -> Self::Iter {
        NaturalLanguagePreprocessorIter {
            messages,
            options
        }
    }

    fn combine(words: impl IntoIterator<Item = MessageWord>) -> String {
        let mut message = String::new();
        let mut word = String::new();

        for curr_word in words.into_iter() {
            // Likely a punctuation.
            if curr_word.len() == 1 {
                let char = unsafe {
                    curr_word.as_str()
                        .chars()
                        .next()
                        .unwrap_unchecked()
                };

                if char.is_ascii_punctuation() {
                    word.push(char);

                    continue;
                }
            }

            message += &word;

            word = curr_word.to_string();
        }

        if !word.is_empty() {
            message += &word;
        }

        message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let message = NaturalLanguagePreprocessor::process(
            Box::new([String::from("Hello,  World! (te=st)")].into_iter()),
            NaturalLanguagePreprocessorOptions {
                lowercase: false
            }
        ).next().unwrap();

        let words = message.words();

        assert_eq!(words.len(), 11);
        assert_eq!(words[0].as_str(), "Hello");
        assert_eq!(words[1].as_str(), ",");
        assert_eq!(words[2].as_str(), " ");
        assert_eq!(words[3].as_str(), "World");
        assert_eq!(words[4].as_str(), "!");
        assert_eq!(words[5].as_str(), " ");
        assert_eq!(words[6].as_str(), "(");
        assert_eq!(words[7].as_str(), "te");
        assert_eq!(words[8].as_str(), "=");
        assert_eq!(words[9].as_str(), "st");
        assert_eq!(words[10].as_str(), ")");
    }

    #[test]
    fn lowercase() {
        let message = NaturalLanguagePreprocessor::process(
            Box::new([String::from("Hello,  World! . ")].into_iter()),
            NaturalLanguagePreprocessorOptions {
                lowercase: true
            }
        ).next().unwrap();

        let words = message.words();

        assert_eq!(words.len(), 7);
        assert_eq!(words[0].as_str(), "hello");
        assert_eq!(words[1].as_str(), ",");
        assert_eq!(words[2].as_str(), " ");
        assert_eq!(words[3].as_str(), "world");
        assert_eq!(words[4].as_str(), "!");
        assert_eq!(words[5].as_str(), " ");
        assert_eq!(words[6].as_str(), ".");
    }

    #[test]
    fn combine() {
        let message = NaturalLanguagePreprocessor::combine([
            MessageWord::new_raw("Hello").unwrap(),
            MessageWord::new_raw(",").unwrap(),
            MessageWord::new_raw(" ").unwrap(),
            MessageWord::new_raw("World").unwrap(),
            MessageWord::new_raw("!").unwrap()
        ]);

        assert_eq!(message, "Hello, World!");
    }
}
