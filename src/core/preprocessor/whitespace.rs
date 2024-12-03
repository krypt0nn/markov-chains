use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WhitespacePreprocessorOptions {
    /// Convert all the characters into lowercase.
    pub lowercase: bool,

    /// Remove all the punctuation characters.
    pub remove_punctuation: bool
}

impl Default for WhitespacePreprocessorOptions {
    #[inline]
    fn default() -> Self {
        Self {
            lowercase: false,
            remove_punctuation: false
        }
    }
}

pub struct WhitespacePreprocessorIter {
    messages: Box<dyn Iterator<Item = String>>,
    options: WhitespacePreprocessorOptions
}

impl Iterator for WhitespacePreprocessorIter {
    type Item = Message;

    fn next(&mut self) -> Option<Self::Item> {
        self.messages.next()
            .and_then(|message| {
                message.split_whitespace()
                    .map(|word| {
                        let mut word = word.to_string();

                        if self.options.lowercase {
                            word = word.to_lowercase();
                        }

                        if self.options.remove_punctuation {
                            #[inline]
                            fn not_punctuation(c: &char) -> bool {
                                c != &'.' && c != &',' && c != &'!' && c != &'?'
                            }

                            word = word.chars()
                                .filter(not_punctuation)
                                .collect::<String>()
                        }

                        word
                    })
                    .filter(|word| !word.is_empty())
                    .map(MessageWord::new_raw)
                    .collect::<Result<Message, _>>()
                    .ok()
            })
    }
}

impl FusedIterator for WhitespacePreprocessorIter {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Split input messages into separate words
/// using whitespaces as delimiter.
///
/// `"Hello, World!" -> ["Hello,", "World!"]`
///
/// ```
/// use markov_chains::prelude::*;
///
/// let message = WhitespacePreprocessor::process(
///     Box::new([String::from("Hello, World!")].into_iter()),
///     WhitespacePreprocessorOptions {
///         lowercase: false,
///         remove_punctuation: false
///     }
/// ).next().unwrap();
///
/// let words = message.words();
///
/// assert_eq!(words.len(), 2);
/// assert_eq!(words[0].as_str(), "Hello,");
/// assert_eq!(words[1].as_str(), "World!");
/// ```
pub struct WhitespacePreprocessor;

impl Preprocessor for WhitespacePreprocessor {
    type Options = WhitespacePreprocessorOptions;
    type Iter = WhitespacePreprocessorIter;

    #[inline]
    fn process(messages: Box<dyn Iterator<Item = String>>, options: Self::Options) -> Self::Iter {
        WhitespacePreprocessorIter {
            messages,
            options
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let message = WhitespacePreprocessor::process(
            Box::new([String::from("Hello,  World! ")].into_iter()),
            WhitespacePreprocessorOptions {
                lowercase: false,
                remove_punctuation: false
            }
        ).next().unwrap();

        let words = message.words();

        assert_eq!(words.len(), 2);
        assert_eq!(words[0].as_str(), "Hello,");
        assert_eq!(words[1].as_str(), "World!");
    }

    #[test]
    fn lowercase() {
        let message = WhitespacePreprocessor::process(
            Box::new([String::from("Hello,  World! . ")].into_iter()),
            WhitespacePreprocessorOptions {
                lowercase: true,
                remove_punctuation: false
            }
        ).next().unwrap();

        let words = message.words();

        assert_eq!(words.len(), 3);
        assert_eq!(words[0].as_str(), "hello,");
        assert_eq!(words[1].as_str(), "world!");
        assert_eq!(words[2].as_str(), ".");
    }

    #[test]
    fn remove_punctuation() {
        let message = WhitespacePreprocessor::process(
            Box::new([String::from("Hello,  World! . ")].into_iter()),
            WhitespacePreprocessorOptions {
                lowercase: false,
                remove_punctuation: true
            }
        ).next().unwrap();

        let words = message.words();

        assert_eq!(words.len(), 2);
        assert_eq!(words[0].as_str(), "Hello");
        assert_eq!(words[1].as_str(), "World");
    }
}
