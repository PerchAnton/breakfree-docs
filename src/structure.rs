//! Document-owned structure and page policy, independent of GUI and file adapters.
use crate::{
    Result,
    model::{Document, Paragraph, boundaries, validate_text},
};
use serde::{Deserialize, Serialize};
pub const OBJECT: &str = "\u{fffc}";
pub const MAX_IMAGE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListKind {
    Bullet,
    Decimal,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListItem {
    pub kind: ListKind,
    pub level: u8,
    pub restart: Option<u16>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub start: usize,
    pub end: usize,
    pub target: String,
}
impl Link {
    pub fn validate(&self, text: &str) -> Result<()> {
        let edges = boundaries(text);
        if self.start >= self.end || !edges.contains(&self.start) || !edges.contains(&self.end) {
            return Err("Link boundaries must cover complete characters.".into());
        }
        validate_target(&self.target)
    }
}
pub fn validate_target(target: &str) -> Result<()> {
    if target.len() > 2048
        || target.chars().any(|c| c.is_control() || c.is_whitespace())
        || !["https://", "http://", "mailto:"]
            .iter()
            .any(|prefix| target.starts_with(prefix) && target.len() > prefix.len())
    {
        return Err(
            "Use an http://, https:// or mailto: link without spaces (maximum 2048 bytes).".into(),
        );
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    pub widths_points: Vec<u16>,
    pub rows: Vec<Vec<String>>,
    pub repeat_header: bool,
}
impl Table {
    pub fn new(rows: usize, columns: usize) -> Result<Self> {
        if !(1..=100).contains(&rows) || !(1..=12).contains(&columns) {
            return Err("Tables support 1–100 rows and 1–12 columns.".into());
        }
        Ok(Self {
            widths_points: vec![72; columns],
            rows: vec![vec![String::new(); columns]; rows],
            repeat_header: false,
        })
    }
    pub fn validate(&self) -> Result<()> {
        if !(1..=100).contains(&self.rows.len())
            || !(1..=12).contains(&self.widths_points.len())
            || self.widths_points.iter().any(|w| !(18..=720).contains(w))
        {
            return Err("Invalid table dimensions or column widths (18–720 pt).".into());
        }
        let mut total = 0;
        for row in &self.rows {
            if row.len() != self.widths_points.len() {
                return Err("Table rows must have the same number of cells.".into());
            }
            for cell in row {
                validate_text(cell)?;
                total += cell.chars().count();
            }
        }
        if total > 50_000 {
            return Err("Table exceeds 50,000 characters.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Picture {
    pub png: Vec<u8>,
    pub width_points: u16,
    pub height_points: u16,
    pub alt: String,
}
impl Picture {
    pub fn validate_pixels(&self) -> Result<()> {
        self.validate()?;
        image::load_from_memory_with_format(&self.png, image::ImageFormat::Png)
            .map(|_| ())
            .map_err(|e| format!("Invalid PNG pixels: {e}"))
    }
    pub fn validate(&self) -> Result<()> {
        if self.png.len() > MAX_IMAGE_BYTES
            || self.png.len() < 24
            || !self.png.starts_with(b"\x89PNG\r\n\x1a\n")
            || &self.png[12..16] != b"IHDR"
        {
            return Err("Pictures must be PNG images of at most 4 MiB.".into());
        }
        let mut at = 8usize;
        let mut ended = false;
        let mut data = false;
        while at < self.png.len() {
            if at + 12 > self.png.len() {
                return Err("Truncated PNG chunk.".into());
            }
            let length = u32::from_be_bytes(self.png[at..at + 4].try_into().unwrap()) as usize;
            let end = at
                .checked_add(length + 12)
                .filter(|end| *end <= self.png.len())
                .ok_or("Truncated PNG payload.")?;
            let kind = &self.png[at + 4..at + 8];
            if at == 8 && (kind != b"IHDR" || length != 13) {
                return Err("Invalid PNG header.".into());
            }
            let mut crc = !0u32;
            for byte in &self.png[at + 4..end - 4] {
                crc ^= u32::from(*byte);
                for _ in 0..8 {
                    crc = (crc >> 1) ^ (0xedb88320 & (0u32.wrapping_sub(crc & 1)));
                }
            }
            if !crc != u32::from_be_bytes(self.png[end - 4..end].try_into().unwrap()) {
                return Err("Damaged PNG checksum.".into());
            }
            data |= kind == b"IDAT";
            at = end;
            if kind == b"IEND" {
                ended = length == 0 && at == self.png.len();
                break;
            }
        }
        if !ended || !data {
            return Err("PNG requires image data and a final end chunk.".into());
        }
        let w = u32::from_be_bytes(self.png[16..20].try_into().unwrap());
        let h = u32::from_be_bytes(self.png[20..24].try_into().unwrap());
        if w == 0 || h == 0 || u64::from(w) * u64::from(h) > 16_000_000 || w > 8192 || h > 8192 {
            return Err("Image exceeds the 16 megapixel / 8192-pixel dimension limit.".into());
        }
        if !(12..=1440).contains(&self.width_points)
            || !(12..=1440).contains(&self.height_points)
            || self.alt.chars().count() > 1024
        {
            return Err("Invalid picture size (12–1440 pt) or alternative text.".into());
        }
        validate_text(&self.alt)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Block {
    Table(Table),
    Picture(Picture),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub width_points: u16,
    pub height_points: u16,
    pub left_points: u16,
    pub right_points: u16,
    pub top_points: u16,
    pub bottom_points: u16,
    pub header: String,
    pub footer: String,
    pub page_numbers: bool,
    pub number_start: Option<u16>,
}
impl Default for Section {
    fn default() -> Self {
        Self {
            width_points: 612,
            height_points: 792,
            left_points: 72,
            right_points: 72,
            top_points: 72,
            bottom_points: 72,
            header: String::new(),
            footer: String::new(),
            page_numbers: false,
            number_start: None,
        }
    }
}
impl Section {
    pub fn validate(&self) -> Result<()> {
        if !(144..=1440).contains(&self.width_points)
            || !(144..=1440).contains(&self.height_points)
            || u32::from(self.left_points) + u32::from(self.right_points) + 72
                > u32::from(self.width_points)
            || u32::from(self.top_points) + u32::from(self.bottom_points) + 72
                > u32::from(self.height_points)
            || [
                self.left_points,
                self.right_points,
                self.top_points,
                self.bottom_points,
            ]
            .iter()
            .any(|v| *v < 18)
            || self.number_start == Some(0)
        {
            return Err("Page dimensions must be 144–1440 pt; margins at least 18 pt, leaving a 72 pt text area; page numbering starts at 1 or later.".into());
        }
        for text in [&self.header, &self.footer] {
            validate_text(text)?;
            if text.chars().count() > 200 || text.contains(['\n', '\t']) {
                return Err("Headers and footers support up to 200 characters on one line.".into());
            }
        }
        Ok(())
    }
    pub fn text_width(&self) -> f32 {
        f32::from(self.width_points - self.left_points - self.right_points)
    }
    pub fn text_height(&self) -> f32 {
        f32::from(self.height_points - self.top_points - self.bottom_points)
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParagraphStructure {
    pub list: Option<ListItem>,
    pub links: Vec<Link>,
    pub block: Option<Block>,
    pub page_break_before: bool,
    pub keep_with_next: bool,
    pub keep_lines: bool,
    pub section: Option<Section>,
}
impl ParagraphStructure {
    pub fn validate(&self, text: &str) -> Result<()> {
        if let Some(list) = &self.list
            && (list.level > 8 || list.restart == Some(0))
        {
            return Err("List level must be 0–8 and numbering starts at 1 or later.".into());
        }
        let mut end = 0;
        for link in &self.links {
            link.validate(text)?;
            if link.start < end {
                return Err("Links cannot overlap.".into());
            }
            end = link.end;
        }
        if let Some(section) = &self.section {
            section.validate()?;
        }
        if let Some(block) = &self.block {
            if text != OBJECT || self.list.is_some() || !self.links.is_empty() {
                return Err(
                    "An object occupies its own paragraph and cannot carry list/link text.".into(),
                );
            }
            match block {
                Block::Table(t) => t.validate()?,
                Block::Picture(p) => p.validate()?,
            }
        } else if text.contains(OBJECT) {
            return Err("An object marker requires its stored object.".into());
        }
        Ok(())
    }
}
pub fn labels(document: &Document) -> Vec<Option<String>> {
    let mut counters = [0usize; 9];
    let mut prior = None;
    document
        .paragraphs
        .iter()
        .map(|p| {
            let Some(item) = &p.structure.list else {
                counters = [0; 9];
                prior = None;
                return None;
            };
            if prior != Some(item.kind) {
                counters = [0; 9];
            }
            prior = Some(item.kind);
            let level = usize::from(item.level);
            counters[level] = item.restart.map_or(counters[level] + 1, usize::from);
            for n in counters.iter_mut().skip(level + 1) {
                *n = 0;
            }
            Some(match item.kind {
                ListKind::Bullet => ["•", "◦", "▪"][level % 3].into(),
                ListKind::Decimal => format!("{}.", counters[level]),
            })
        })
        .collect()
}
pub fn paragraph_indices(
    document: &Document,
    range: (usize, usize),
) -> Result<std::ops::RangeInclusive<usize>> {
    crate::model::validate_range(&document.text(), range.0, range.1)?;
    let (first, _) = document.locate(range.0)?;
    let (mut last, offset) = document.locate(range.1)?;
    if range.1 > range.0 && offset == 0 && last > first {
        last -= 1;
    }
    Ok(first..=last)
}
pub fn plain_paragraph(p: &Paragraph) -> String {
    match &p.structure.block {
        Some(Block::Table(t)) => t
            .rows
            .iter()
            .map(|r| r.join("\t"))
            .collect::<Vec<_>>()
            .join("\n"),
        Some(Block::Picture(p)) => p.alt.clone(),
        None => p.text.clone(),
    }
}

/// Outline follows semantic Heading 1 ancestry, never guessed font sizes.
pub fn headings(document: &Document) -> Vec<(usize, usize, String)> {
    let mut offset = 0;
    let mut result = Vec::new();
    for p in &document.paragraphs {
        let mut style = Some(p.paragraph_style);
        for _ in 0..=document.style_catalog.custom.len() + 3 {
            let Some(id) = style else { break };
            if id == crate::model::ParagraphStyle::Heading1 {
                result.push((
                    offset,
                    offset + p.styles.len(),
                    if p.text.is_empty() {
                        "(Empty heading)".into()
                    } else {
                        p.text.clone()
                    },
                ));
                break;
            }
            style = document.style_catalog.get(id).and_then(|s| s.parent);
        }
        offset += p.styles.len() + 1;
    }
    result
}
