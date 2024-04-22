use std::io::BufRead;
use std::path::Path;
use std::collections::HashSet;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Messages {
    pub(crate) messages: HashSet<Vec<String>>
}

impl Messages {
    pub fn parse_from_messages(file: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut messages = HashSet::new();

        let file = std::fs::File::open(file)?;

        for line in std::io::BufReader::new(file).lines() {
            let line = line?.trim().to_string();

            let line = serde_json::from_str::<String>(&line)
                .unwrap_or(line);

            let words = line.split_whitespace()
                .filter(|word| !word.is_empty())
                .map(|word| word.to_lowercase()) // .to_string()
                .collect::<Vec<_>>();

            if !words.is_empty() {
                messages.insert(words);
            }
        }

        Ok(Self {
            messages
        })
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
