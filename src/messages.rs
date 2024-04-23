use std::io::BufRead;
use std::path::Path;
use std::collections::HashSet;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Messages {
    pub(crate) messages: HashSet<Vec<String>>
}

impl Messages {
    #[inline]
    pub fn parse_from_messages(file: impl AsRef<Path>) -> anyhow::Result<Self> {
        Self::parse_from_messages_with_filter(file, |word| word.to_lowercase())
    }

    pub fn parse_from_messages_with_filter(file: impl AsRef<Path>, filter: impl Fn(&str) -> String) -> anyhow::Result<Self> {
        let file = std::fs::File::open(file)?;

        let lines = std::io::BufReader::new(file)
            .lines()
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self::parse_from_lines_with_filter(&lines, filter))
    }

    #[inline]
    pub fn parse_from_lines(lines: &[String]) -> Self {
        Self::parse_from_lines_with_filter(lines, |word| word.to_lowercase())
    }

    pub fn parse_from_lines_with_filter(lines: &[String], filter: impl Fn(&str) -> String) -> Self {
        let mut messages = HashSet::new();

        for line in lines {
            let line = line.trim().to_string();

            let line = serde_json::from_str::<String>(&line)
                .unwrap_or(line);

            let words = line.split_whitespace()
                .filter(|word| !word.is_empty())
                .map(&filter)
                .collect::<Vec<_>>();

            if !words.is_empty() {
                messages.insert(words);
            }
        }

        Self {
            messages
        }
    }

    #[inline]
    pub fn messages(&self) -> &HashSet<Vec<String>> {
        &self.messages
    }

    #[inline]
    pub fn merge(mut self, messages: Messages) -> Self {
        self.messages.extend(messages.messages);

        self
    }
}

mod tests {
    #[test]
    fn parse() {
        use super::Messages;

        let messages = Messages::parse_from_lines(&[
            String::from("Hello, World!"),
            String::from("Example text")
        ]);

        assert!(messages.messages().contains(&vec![
            String::from("hello,"),
            String::from("world!")
        ]));

        assert!(messages.messages().contains(&vec![
            String::from("example"),
            String::from("text")
        ]));
    }

    #[test]
    fn merging() {
        use super::Messages;

        let messages = Messages::default()
            .merge(Messages::parse_from_lines(&[
                String::from("Hello, World!")
            ]))
            .merge(Messages::parse_from_lines(&[
                String::from("Example text")
            ]));

        assert!(messages.messages().contains(&vec![
            String::from("hello,"),
            String::from("world!")
        ]));

        assert!(messages.messages().contains(&vec![
            String::from("example"),
            String::from("text")
        ]));
    }
}
