use super::*;

// TODO: encapsulated text reader.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NewlineParserOptions {
    /// Amout of bytes to read at once.
    pub read_chunk: usize
}

impl Default for NewlineParserOptions {
    #[inline]
    fn default() -> Self {
        Self {
            read_chunk: 4096
        }
    }
}

pub struct NewlineParserIter {
    reader: Box<dyn Read>,
    options: NewlineParserOptions,
    buf: Vec<u8>,
    begin_ptr: usize,
    search_ptr: usize
}

impl Iterator for NewlineParserIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        #[inline]
        fn extend_buf(buf: &mut Vec<u8>, reader: &mut Box<dyn Read>, options: &NewlineParserOptions) -> bool {
            let mut chunk = vec![0; options.read_chunk];

            match reader.read(&mut chunk) {
                Ok(0) | Err(_) => false,

                Ok(n) => {
                    buf.extend_from_slice(&chunk[..n]);

                    true
                }
            }
        }

        #[inline]
        const fn is_newline(byte: u8) -> bool {
            byte == b'\r' || byte == b'\n'
        }

        // There's no more sentences if the buffer is empty and we can't extend it.
        if self.buf.is_empty() && !extend_buf(&mut self.buf, &mut self.reader, &self.options) {
            return None;
        }

        let mut n = self.buf.len();

        while n > 0 {
            // Skip initial new line bytes.
            while self.begin_ptr < n && is_newline(self.buf[self.begin_ptr]) {
                self.begin_ptr += 1;
            }

            if self.search_ptr < self.begin_ptr {
                self.search_ptr = self.begin_ptr + 1;
            }

            // Find first non-initial new line byte.
            while self.search_ptr < n {
                if is_newline(self.buf[self.search_ptr]) {
                    // Parse string between new lines.
                    let result = String::from_utf8_lossy(&self.buf[self.begin_ptr..self.search_ptr])
                        .to_string();

                    // Skip all the further new lines.
                    while self.search_ptr < n && is_newline(self.buf[self.search_ptr]) {
                        self.search_ptr += 1;
                    }

                    self.buf = self.buf[self.search_ptr..].to_vec();

                    self.begin_ptr = 0;
                    self.search_ptr = 0;

                    // Return found string.
                    return Some(result);
                }

                self.search_ptr += 1;
            }

            // Extend the buffer if we couldn't find a string between new lines.
            if !extend_buf(&mut self.buf, &mut self.reader, &self.options) {
                // Return remaining string if we couldn't extend the buffer.
                if self.search_ptr > self.begin_ptr {
                    let result = String::from_utf8_lossy(&self.buf[self.begin_ptr..self.search_ptr])
                        .to_string();

                    self.buf = self.buf[self.search_ptr..].to_vec();

                    self.begin_ptr = 0;
                    self.search_ptr = 0;

                    return Some(result);
                }

                // Otherwise indicate that there's no more strings.
                else {
                    return None;
                }
            }

            n = self.buf.len();
        }

        None
    }
}

impl FusedIterator for NewlineParserIter {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Split input text into messages using new lines as delimiter.
///
/// ```
/// use markov_chains::prelude::*;
///
/// let text = "
/// Text 1
/// Text 2
/// Text 3
/// ";
///
/// let mut iter = NewlineParser::parse(Box::new(text.as_bytes()), NewlineParserOptions::default());
///
/// assert_eq!(iter.next(), Some(String::from("Text 1")));
/// assert_eq!(iter.next(), Some(String::from("Text 2")));
/// assert_eq!(iter.next(), Some(String::from("Text 3")));
/// assert_eq!(iter.next(), None);
/// ```
pub struct NewlineParser;

impl Parser for NewlineParser {
    type Options = NewlineParserOptions;
    type Iter = NewlineParserIter;

    #[inline]
    fn parse(reader: Box<dyn Read>, options: Self::Options) -> Self::Iter {
        assert!(options.read_chunk > 0, "Read chunk size must be greater than 0");

        NewlineParserIter {
            reader,
            options,
            buf: Vec::with_capacity(options.read_chunk),
            begin_ptr: 0,
            search_ptr: 0
        }
    }
}

#[test]
fn test() {
    use super::*;

    let text = "\rExample text 1\nExample text 2\rExample text 3\r\nExample text 4\n\n\r\nExample text 5";

    let mut iter = NewlineParser::parse(Box::new(text.as_bytes()), NewlineParserOptions {
        read_chunk: 5
    });

    assert_eq!(iter.next(), Some(String::from("Example text 1")));
    assert_eq!(iter.next(), Some(String::from("Example text 2")));
    assert_eq!(iter.next(), Some(String::from("Example text 3")));
    assert_eq!(iter.next(), Some(String::from("Example text 4")));
    assert_eq!(iter.next(), Some(String::from("Example text 5")));
    assert_eq!(iter.next(), None);
}
