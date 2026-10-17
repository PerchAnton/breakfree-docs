//! Literal search over original graphemes, with optional Unicode case/word matching.
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Match {
    pub range: (usize, usize),
    pub paragraph: usize,
    bytes: std::ops::Range<usize>,
}

pub struct Results<'a> {
    text: &'a str,
    pub matches: Vec<Match>,
}

/// A compiled literal query, reusable until the text or options change.
/// Case-insensitive matching uses Unicode simple folding, never normalization.
pub struct Query {
    text: String,
    options: Options,
    folded: Option<regex::Regex>,
    error: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    pub match_case: bool,
    pub whole_words: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            match_case: true,
            whole_words: false,
        }
    }
}

impl Query {
    pub fn new(text: &str, options: Options) -> Self {
        let mut query = Self {
            text: text.into(),
            options,
            folded: None,
            error: None,
        };
        if text.len() > 4096 {
            query.error = Some("Search text is too long. Use a shorter query.");
        } else if !text.is_empty() && !options.match_case {
            match regex::RegexBuilder::new(&regex::escape(text))
                .case_insensitive(true)
                .build()
            {
                Ok(pattern) => query.folded = Some(pattern),
                Err(_) => query.error = Some("Search could not be prepared. Use a shorter query."),
            }
        }
        query
    }

    pub fn is_current(&self, text: &str, options: Options) -> bool {
        self.text == text && self.options == options
    }

    pub fn error(&self) -> Option<&'static str> {
        self.error
    }

    pub fn search<'a>(&self, text: &'a str) -> Results<'a> {
        let mut matches = Vec::new();
        if self.text.is_empty() || self.error.is_some() {
            return Results { text, matches };
        }
        let edges: Vec<_> = text
            .grapheme_indices(true)
            .map(|(byte, _)| byte)
            .chain(std::iter::once(text.len()))
            .collect();
        let words: Vec<_> = if self.options.whole_words {
            text.unicode_word_indices()
                .map(|(start, word)| (start, start + word.len()))
                .collect()
        } else {
            Vec::new()
        };
        let (mut offset, mut previous, mut scalar, mut paragraph) = (0, 0, 0, 1);
        while offset < text.len() {
            let candidate = if let Some(pattern) = &self.folded {
                pattern.find_at(text, offset).map(|m| m.range())
            } else {
                text[offset..].find(&self.text).map(|byte| {
                    let start = offset + byte;
                    start..start + self.text.len()
                })
            };
            let Some(bytes) = candidate else {
                break;
            };
            // Walk each prefix once, retaining original scalar/paragraph positions.
            for ch in text[previous..bytes.start].chars() {
                scalar += 1;
                paragraph += usize::from(ch == '\n');
            }
            previous = bytes.start;
            let complete = edges.binary_search(&bytes.start).is_ok()
                && edges.binary_search(&bytes.end).is_ok();
            let whole = !self.options.whole_words
                || (words.binary_search_by_key(&bytes.start, |w| w.0).is_ok()
                    && words.binary_search_by_key(&bytes.end, |w| w.1).is_ok());
            if complete && whole {
                offset = bytes.end;
                let length = text[bytes.clone()].chars().count();
                matches.push(Match {
                    range: (scalar, scalar + length),
                    paragraph,
                    bytes,
                });
            } else {
                // A rejected partial match must not hide a later overlapping valid one.
                offset = edges[edges.partition_point(|&edge| edge <= bytes.start)];
            }
        }
        Results { text, matches }
    }
}

impl<'a> Results<'a> {
    pub fn new(text: &'a str, query: &str) -> Self {
        Query::new(query, Options::default()).search(text)
    }

    pub fn active(&self, selection: (usize, usize)) -> Option<usize> {
        let range = (selection.0.min(selection.1), selection.0.max(selection.1));
        self.matches.iter().position(|m| m.range == range)
    }

    /// Wrap through all matches, or start at the current document selection.
    pub fn adjacent(&self, selection: (usize, usize), previous: bool) -> Option<usize> {
        let count = self.matches.len();
        if count == 0 {
            return None;
        }
        if let Some(index) = self.active(selection) {
            return Some(if previous {
                (index + count - 1) % count
            } else {
                (index + 1) % count
            });
        }
        Some(if previous {
            self.matches
                .iter()
                .rposition(|m| m.range.1 <= selection.0.min(selection.1))
                .unwrap_or(count - 1)
        } else {
            self.matches
                .iter()
                .position(|m| m.range.0 >= selection.0.max(selection.1))
                .unwrap_or(0)
        })
    }

    /// Bounded display context. Never slice a grapheme or change document text.
    pub fn excerpt(&self, index: usize) -> (String, String, String) {
        let found = &self.matches[index];
        let left: Vec<_> = self.text[..found.bytes.start]
            .graphemes(true)
            .rev()
            .take(18)
            .collect();
        let prefix: String = left.into_iter().rev().collect();
        let matched: String = self.text[found.bytes.clone()]
            .graphemes(true)
            .take(32)
            .collect();
        let suffix: String = self.text[found.bytes.end..]
            .graphemes(true)
            .take(24)
            .collect();
        let before = format!(
            "{}{}",
            if prefix.len() < found.bytes.start {
                "…"
            } else {
                ""
            },
            prefix
        );
        let middle = format!(
            "{}{}",
            matched,
            if matched.len() < found.bytes.len() {
                "…"
            } else {
                ""
            }
        );
        let after = format!(
            "{}{}",
            suffix,
            if suffix.len() < self.text.len() - found.bytes.end {
                "…"
            } else {
                ""
            }
        );
        let display = |s: String| s.replace('\n', " / ").replace('\t', "  ");
        (display(before), display(middle), display(after))
    }
}
