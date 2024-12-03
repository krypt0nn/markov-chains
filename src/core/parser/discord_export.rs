use super::*;

// TODO: add option to load whole JSON into RAM and decode it normally.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiscordExportParserOptions {
    /// Amount of bytes to read from the JSON file at once.
    pub block_size: usize,

    /// Format of the exported message.
    ///
    /// Supported keys:
    /// - `%person_id%` - id of the person.
    /// - `%person_name%` - nickname of the person.
    /// - `%message_content%` - content of the message.
    ///
    /// Default format: `<%person_name%>: %message_content%`
    pub format: String,

    /// When enabled exported messages will be formatted
    /// according to `format` option.
    pub format_export: bool
}

impl Default for DiscordExportParserOptions {
    #[inline]
    fn default() -> Self {
        Self {
            block_size: 4096,
            format: String::from("<%person_name%>: %message_content%"),
            format_export: false
        }
    }
}

pub struct DiscordExportParserIter {
    reader: Box<dyn Read>,
    buf: Vec<u8>,
    begin_ptr: usize,
    options: DiscordExportParserOptions
}

impl Iterator for DiscordExportParserIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            while self.begin_ptr + 4 < self.buf.len() {
                if &self.buf[self.begin_ptr..self.begin_ptr + 4] == b"\n  {" {
                    self.begin_ptr += 3;

                    let mut j = self.begin_ptr + 1;

                    loop {
                        while j + 4 < self.buf.len() {
                            if &self.buf[j..j + 4] == b"\n  }" {
                                j += 4;

                                let json = &self.buf[self.begin_ptr..j];

                                let message = serde_json::from_slice::<serde_json::Value>(json).ok()?;

                                self.buf = self.buf[j..].to_vec();
                                self.begin_ptr = 0;

                                let content = message["content"].as_str()?;

                                if !self.options.format_export {
                                    return Some(content.to_string());
                                }

                                let person_id = message["author"]["id"].as_str()?;
                                let person_name = message["author"]["name"].as_str()?;

                                let format = self.options.format
                                    .replace("%person_id%", person_id)
                                    .replace("%person_name%", person_name)
                                    .replace("%message_content%", content);

                                return Some(format);
                            }

                            else {
                                j += 1;
                            }
                        }

                        let mut buf = vec![0; self.options.block_size];

                        let n = self.reader.read(&mut buf).ok()?;

                        self.buf.extend(&buf[..n]);
                    }
                }

                else {
                    self.begin_ptr += 1;
                }
            }

            self.buf = self.buf[self.begin_ptr..].to_vec();
            self.begin_ptr = 0;

            let mut buf = vec![0; self.options.block_size];

            let n = self.reader.read(&mut buf).ok()?;

            if n == 0 {
                return None;
            }

            self.buf.extend(&buf[..n]);
        }
    }
}

impl FusedIterator for DiscordExportParserIter {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Parse exported discord messages history from JSON array
/// by reading it in small chunks.
///
/// This parser will split input file into smaller blocks
/// of fixed size, read them into the RAM and search for
/// JSON object patterns, dropping parsed blocks.
/// It is necessary for these patterns to be in special format.
pub struct DiscordExportParser;

impl Parser for DiscordExportParser {
    type Options = DiscordExportParserOptions;
    type Iter = DiscordExportParserIter;

    #[inline]
    fn parse(reader: Box<dyn Read>, options: Self::Options) -> Self::Iter {
        assert!(options.block_size > 0, "Block size must be larger than 0");

        DiscordExportParserIter {
            reader,
            buf: Vec::with_capacity(options.block_size),
            begin_ptr: 0,
            options
        }
    }
}
