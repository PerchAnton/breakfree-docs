//! Literal, case-sensitive search over complete graphemes; no normalization.
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

impl<'a> Results<'a> {
    pub fn new(text: &'a str, query: &str) -> Self {
        let mut matches = Vec::new();
        if !query.is_empty() {
            let edges: Vec<_> = text
                .grapheme_indices(true)
                .map(|(byte, _)| byte)
                .chain(std::iter::once(text.len()))
                .collect();
            let (mut previous, mut scalar, mut paragraph) = (0, 0, 1);
            let length = query.chars().count();
            for (byte, found) in text.match_indices(query) {
                // Walk each prefix only once, including rejected partial matches.
                for ch in text[previous..byte].chars() {
                    scalar += 1;
                    paragraph += usize::from(ch == '\n');
                }
                previous = byte;
                let end = byte + found.len();
                if edges.binary_search(&byte).is_ok() && edges.binary_search(&end).is_ok() {
                    matches.push(Match {
                        range: (scalar, scalar + length),
                        paragraph,
                        bytes: byte..end,
                    });
                }
            }
        }
        Self { text, matches }
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
