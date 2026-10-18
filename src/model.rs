//! Unicode scalar indices, immutable command snapshots, and supported document values.
use crate::Result;
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

pub const MAX_CHARACTERS: usize = 100_000;
pub const FONT_SIZE_PRESETS: &[u16] =
    &[8, 9, 10, 11, 12, 14, 16, 18, 20, 22, 24, 26, 28, 36, 48, 72];

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

    /// Step each run independently through common presets. Outside the preset
    /// interval, move one point; preserve half points and clamp to model limits.
    pub fn stepped(self, grow: bool) -> Self {
        let value = if grow && (16..144).contains(&self.0) {
            FONT_SIZE_PRESETS
                .iter()
                .map(|p| p * 2)
                .find(|p| *p > self.0)
                .unwrap()
        } else if !grow && (16..=144).contains(&self.0) && self.0 > 16 {
            FONT_SIZE_PRESETS
                .iter()
                .rev()
                .map(|p| p * 2)
                .find(|p| *p < self.0)
                .unwrap()
        } else if grow {
            (self.0 + 2).min(800)
        } else {
            self.0.saturating_sub(2).max(2)
        };
        Self(value)
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

/// Direct character overrides. None inherits; Some(false) explicitly disables a property.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterFormat {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub font_size: Option<FontSize>,
}
impl CharacterFormat {
    pub fn resolve(self, base: Style) -> Style {
        Style {
            bold: self.bold.unwrap_or(base.bold),
            italic: self.italic.unwrap_or(base.italic),
            underline: self.underline.unwrap_or(base.underline),
            font_size: self.font_size.unwrap_or(base.font_size),
        }
    }
    /// Legacy files contain resolved values, with no reliable authored inheritance.
    pub fn explicit(style: Style) -> Self {
        Self {
            bold: Some(style.bold),
            italic: Some(style.italic),
            underline: Some(style.underline),
            font_size: Some(style.font_size),
        }
    }
}

/// Stable paragraph-style identifiers. Definitions belong to each document.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParagraphStyle {
    #[default]
    Normal,
    NoSpacing,
    Heading1,
    Custom(u32),
}
impl ParagraphStyle {
    pub const ALL: [Self; 3] = [Self::Normal, Self::NoSpacing, Self::Heading1];
    pub fn name(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::NoSpacing => "No Spacing",
            Self::Heading1 => "Heading 1",
            Self::Custom(_) => "Custom style",
        }
    }
    /// Frozen v3 baseline. Use the document catalog for actual editing/layout.
    pub fn parent(self) -> Option<Self> {
        StyleCatalog::default().get(self).and_then(|d| d.parent)
    }
    pub fn character_style(self) -> Style {
        StyleCatalog::default().resolve(self).character
    }
    pub fn line_height(self) -> f32 {
        StyleCatalog::default().resolve(self).line_height()
    }
    pub fn space_after_points(self) -> f32 {
        StyleCatalog::default().resolve(self).space_after_points()
    }
}

/// Optional properties inherit from the parent, or the frozen root defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StyleDefinition {
    /// None repeats this style; this behavior is not inherited.
    pub next_style: Option<ParagraphStyle>,
    pub parent: Option<ParagraphStyle>,
    pub character: CharacterFormat,
    pub line_height_percent: Option<u16>,
    pub space_after_half_points: Option<u16>,
}

/// Bound catalog traversal and menu sizes independently of document length.
pub const MAX_CUSTOM_STYLES: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomStyle {
    pub id: u32,
    pub name: String,
    pub definition: StyleDefinition,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StyleCatalog {
    pub custom: Vec<CustomStyle>,
    pub normal: StyleDefinition,
    pub no_spacing: StyleDefinition,
    pub heading1: StyleDefinition,
}
impl Default for StyleCatalog {
    fn default() -> Self {
        Self {
            custom: Vec::new(),
            normal: StyleDefinition::default(),
            no_spacing: StyleDefinition {
                parent: Some(ParagraphStyle::Normal),
                line_height_percent: Some(100),
                space_after_half_points: Some(0),
                ..Default::default()
            },
            heading1: StyleDefinition {
                parent: Some(ParagraphStyle::Normal),
                character: CharacterFormat {
                    bold: Some(true),
                    font_size: Some(FontSize(36)),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedParagraphStyle {
    pub character: Style,
    pub line_height_percent: u16,
    pub space_after_half_points: u16,
}
impl ResolvedParagraphStyle {
    pub fn line_height(self) -> f32 {
        f32::from(self.line_height_percent) / 100.
    }
    pub fn space_after_points(self) -> f32 {
        f32::from(self.space_after_half_points) / 2.
    }
}
impl StyleCatalog {
    pub fn ids(&self) -> Vec<ParagraphStyle> {
        ParagraphStyle::ALL
            .into_iter()
            .chain(self.custom.iter().map(|s| ParagraphStyle::Custom(s.id)))
            .collect()
    }
    pub fn name(&self, id: ParagraphStyle) -> &str {
        match id {
            ParagraphStyle::Custom(id) => self
                .custom
                .iter()
                .find(|s| s.id == id)
                .map(|s| s.name.as_str())
                .unwrap_or("(Missing style)"),
            _ => id.name(),
        }
    }
    pub fn get(&self, id: ParagraphStyle) -> Option<&StyleDefinition> {
        match id {
            ParagraphStyle::Normal => Some(&self.normal),
            ParagraphStyle::NoSpacing => Some(&self.no_spacing),
            ParagraphStyle::Heading1 => Some(&self.heading1),
            ParagraphStyle::Custom(id) => self
                .custom
                .iter()
                .find(|s| s.id == id)
                .map(|s| &s.definition),
        }
    }
    pub fn get_mut(&mut self, id: ParagraphStyle) -> Option<&mut StyleDefinition> {
        match id {
            ParagraphStyle::Normal => Some(&mut self.normal),
            ParagraphStyle::NoSpacing => Some(&mut self.no_spacing),
            ParagraphStyle::Heading1 => Some(&mut self.heading1),
            ParagraphStyle::Custom(id) => self
                .custom
                .iter_mut()
                .find(|s| s.id == id)
                .map(|s| &mut s.definition),
        }
    }
    pub fn next_custom_id(&self) -> u32 {
        (1..=MAX_CUSTOM_STYLES as u32 + 1)
            .find(|id| !self.custom.iter().any(|s| s.id == *id))
            .unwrap_or(0)
    }
    pub fn add(&mut self, name: &str, definition: StyleDefinition) -> Result<ParagraphStyle> {
        let id = self.next_custom_id();
        let mut candidate = self.clone();
        candidate.custom.push(CustomStyle {
            id,
            name: name.into(),
            definition,
        });
        candidate.validate()?;
        *self = candidate;
        Ok(ParagraphStyle::Custom(id))
    }
    /// Edit a custom definition without changing its identity or references.
    pub fn update_custom(
        &mut self,
        id: ParagraphStyle,
        name: &str,
        definition: StyleDefinition,
    ) -> Result<()> {
        let ParagraphStyle::Custom(id) = id else {
            return Err("Built-in style names cannot be changed.".into());
        };
        let mut candidate = self.clone();
        let style = candidate
            .custom
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or("Unknown custom style.")?;
        style.name = name.into();
        style.definition = definition;
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }
    /// Validate all replacement references and return a catalog without one custom style.
    pub fn without_custom_style(
        &self,
        id: ParagraphStyle,
        paragraph_replacement: ParagraphStyle,
        parent_replacement: Option<ParagraphStyle>,
    ) -> Result<Self> {
        let ParagraphStyle::Custom(custom_id) = id else {
            return Err("Built-in styles cannot be deleted.".into());
        };
        if self.get(id).is_none() {
            return Err("Unknown custom style.".into());
        }
        if paragraph_replacement == id || self.get(paragraph_replacement).is_none() {
            return Err("Choose an existing replacement for paragraphs.".into());
        }
        if parent_replacement.is_some_and(|parent| parent == id || self.get(parent).is_none()) {
            return Err("Choose an existing parent or No style for child styles.".into());
        }
        let mut candidate = self.clone();
        candidate.custom.retain(|s| s.id != custom_id);
        for style in candidate.ids() {
            let definition = candidate.get_mut(style).ok_or("Unknown paragraph style.")?;
            if definition.next_style == Some(id) {
                definition.next_style = Some(paragraph_replacement);
            }
            if definition.parent == Some(id) {
                definition.parent = parent_replacement;
            }
        }
        candidate.validate()?;
        Ok(candidate)
    }

    pub fn validate(&self) -> Result<()> {
        if self.custom.len() > MAX_CUSTOM_STYLES {
            return Err("A document supports up to 64 custom styles.".into());
        }
        let mut names: Vec<String> = ParagraphStyle::ALL
            .iter()
            .map(|s| s.name().to_lowercase())
            .collect();
        let mut ids = Vec::new();
        for style in &self.custom {
            if style.id == 0 || ids.contains(&style.id) {
                return Err("Custom style IDs must be positive and unique.".into());
            }
            ids.push(style.id);
            let name = &style.name;
            if name.is_empty()
                || name.trim() != name
                || name.chars().count() > 64
                || name
                    .chars()
                    .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
            {
                return Err(
                    "Use a name of 1–64 characters, without outer spaces or control characters."
                        .into(),
                );
            }
            let key = name.to_lowercase();
            if names.contains(&key) {
                return Err("A style with this name already exists (ignoring letter case).".into());
            }
            names.push(key);
        }
        for id in self.ids() {
            let definition = self.get(id).ok_or("Unknown paragraph style.")?;
            if definition
                .next_style
                .is_some_and(|next| self.get(next).is_none())
            {
                return Err("Unknown following paragraph style.".into());
            }
            if definition
                .line_height_percent
                .is_some_and(|v| !(100..=300).contains(&v))
            {
                return Err("Line spacing must be 100–300 percent.".into());
            }
            if definition.space_after_half_points.is_some_and(|v| v > 400) {
                return Err("Space after must be 0–200 pt in half-point steps.".into());
            }
            let mut seen = Vec::new();
            let mut next = Some(id);
            while let Some(current) = next {
                if seen.contains(&current) {
                    return Err("A style cannot inherit from itself or its descendants.".into());
                }
                seen.push(current);
                next = self.get(current).ok_or("Unknown parent style.")?.parent;
            }
        }
        Ok(())
    }
    /// The bounded walk also keeps invalid external model values safe to inspect.
    /// Editing and storage validate the complete catalog before accepting it.
    pub fn resolve(&self, id: ParagraphStyle) -> ResolvedParagraphStyle {
        let mut chain = Vec::new();
        let mut next = Some(id);
        while let Some(current) = next {
            if chain.contains(&current) {
                break;
            }
            chain.push(current);
            next = self.get(current).and_then(|d| d.parent);
        }
        let mut resolved = ResolvedParagraphStyle {
            character: Style::default(),
            line_height_percent: 125,
            space_after_half_points: 11,
        };
        for id in chain.into_iter().rev() {
            let Some(definition) = self.get(id) else {
                continue;
            };
            resolved.character = definition.character.resolve(resolved.character);
            resolved.line_height_percent = definition
                .line_height_percent
                .unwrap_or(resolved.line_height_percent);
            resolved.space_after_half_points = definition
                .space_after_half_points
                .unwrap_or(resolved.space_after_half_points);
        }
        resolved
    }
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
    pub styles: Vec<CharacterFormat>,
    pub paragraph_style: ParagraphStyle,
    pub alignment: Alignment,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub style_catalog: StyleCatalog,
    pub paragraphs: Vec<Paragraph>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            paragraphs: vec![Paragraph::default()],
            style_catalog: StyleCatalog::default(),
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
        self.style_catalog.validate()?;
        if self.paragraphs.is_empty() {
            return Err("A document needs a paragraph.".into());
        }
        for p in &self.paragraphs {
            if self.style_catalog.get(p.paragraph_style).is_none() {
                return Err("Unknown paragraph style reference.".into());
            }
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

    pub fn style_at(&self, position: usize) -> CharacterFormat {
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
