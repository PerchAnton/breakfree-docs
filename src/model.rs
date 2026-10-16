//! Unicode scalar indices, immutable command snapshots, and supported document values.
use crate::Result;
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

pub const MAX_CHARACTERS: usize = 100_000;

/// Exact half-point units: 1–400 pt, with no floating-point values in the model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct FontSize(u16);

impl Default for FontSize {
    fn default() -> Self {
        Self(22)
    }
}
impl TryFrom<u16> for FontSize {
    type Error = String;
    fn try_from(value: u16) -> Result<Self> {
        if (2..=800).contains(&value) {
            Ok(Self(value))
        } else {
            Err("Font size must be 1–400 pt in steps of 0.5 pt.".into())
        }
    }
}
impl From<FontSize> for u16 {
    fn from(value: FontSize) -> Self {
        value.0
    }
}
impl FontSize {
    pub fn points(self) -> f32 {
        f32::from(self.0) / 2.
    }
}
impl std::fmt::Display for FontSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.0 / 2,
            if self.0 % 2 == 1 { ".5" } else { "" }
        )
    }
}
impl std::str::FromStr for FontSize {
    type Err = String;
    fn from_str(text: &str) -> Result<Self> {
        let invalid = || "Font size must be 1–400 pt in steps of 0.5 pt.".to_owned();
        let (whole, fraction) = text.trim().split_once('.').unwrap_or((text.trim(), "0"));
        if whole.is_empty() || !whole.bytes().all(|b| b.is_ascii_digit()) {
            return Err(invalid());
        }
        let half = match fraction {
            "0" => 0,
            "5" => 1,
            _ => return Err(invalid()),
        };
        whole
            .parse::<u16>()
            .ok()
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| n.checked_add(half))
            .ok_or_else(invalid)?
            .try_into()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub font_size: FontSize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Paragraph {
    pub text: String,
    pub styles: Vec<Style>,
    pub alignment: Alignment,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub paragraphs: Vec<Paragraph>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            paragraphs: vec![Paragraph::default()],
        }
    }
}

pub fn validate_text(text: &str) -> Result<()> {
    if text.chars().count() > MAX_CHARACTERS {
        return Err("Document exceeds the 100,000 character limit.".into());
    }
    if let Some(ch) = text.chars().find(|ch| {
        (ch.is_control() && *ch != '\n' && *ch != '\t') || matches!(*ch, '\u{2028}' | '\u{2029}')
    }) {
        return Err(format!(
            "Unsupported control or separator U+{:04X}.",
            ch as u32
        ));
    }
    Ok(())
}

pub fn boundaries(text: &str) -> Vec<usize> {
    let mut count = 0;
    let mut edges = vec![0];
    for grapheme in text.graphemes(true) {
        count += grapheme.chars().count();
        edges.push(count);
    }
    edges
}

pub fn validate_range(text: &str, start: usize, end: usize) -> Result<()> {
    let edges = boundaries(text);
    if start > end || edges.binary_search(&start).is_err() || edges.binary_search(&end).is_err() {
        return Err("Selection is outside the document or splits a Unicode grapheme.".into());
    }
    Ok(())
}

pub fn slice(text: &str, start: usize, end: usize) -> String {
    text.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

impl Document {
    pub fn text(&self) -> String {
        self.paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn validate(&self) -> Result<()> {
        if self.paragraphs.is_empty() {
            return Err("A document needs a paragraph.".into());
        }
        for p in &self.paragraphs {
            validate_text(&p.text)?;
            if p.text.contains('\n') || p.styles.len() != p.text.chars().count() {
                return Err("Invalid paragraph text/style mapping.".into());
            }
        }
        validate_text(&self.text())
    }

    pub fn locate(&self, mut offset: usize) -> Result<(usize, usize)> {
        for (index, p) in self.paragraphs.iter().enumerate() {
            let len = p.styles.len();
            if offset <= len {
                return Ok((index, offset));
            }
            offset -= len + 1;
        }
        Err("Position is outside the document.".into())
    }

    pub fn style_at(&self, position: usize) -> Style {
        self.locate(position)
            .ok()
            .and_then(|(p, i)| self.paragraphs[p].styles.get(i.saturating_sub(1)).copied())
            .unwrap_or_default()
    }

    pub fn word_count(&self) -> usize {
        self.text().unicode_words().count()
    }
    pub fn character_count(&self) -> usize {
        self.text().chars().count()
    }
}
