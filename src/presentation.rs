//! Layout-free selection, text diff and ribbon geometry policy.
use crate::model::{Document, boundaries};
pub const TABS: &[&str] = &[
    "Home",
    "Insert",
    "Draw",
    "Design",
    "Layout",
    "References",
    "Mailings",
    "Review",
    "View",
    "Help",
];
pub const HOME_GROUPS: &[&str] = &["Clipboard", "Font", "Paragraph", "Styles", "Editing"];
pub const TITLE_HEIGHT: f32 = 36.0;
pub const TABS_HEIGHT: f32 = 30.0;
pub const RIBBON_HEIGHT: f32 = 92.0;
pub const STATUS_HEIGHT: f32 = 24.0;
pub const CANVAS_WIDTH: f32 = 816.0;
pub const CANVAS_MARGIN: f32 = 96.0;

pub fn snap(text: &str, start: usize, end: usize) -> (usize, usize) {
    let edges = boundaries(text);
    let left = edges
        .iter()
        .copied()
        .take_while(|e| *e <= start)
        .last()
        .unwrap_or(0);
    let right = if start == end {
        left
    } else {
        edges
            .into_iter()
            .find(|e| *e >= end)
            .unwrap_or(text.chars().count())
    };
    (left, right)
}
pub fn text_change(old: &str, new: &str) -> (usize, usize, String) {
    let a: Vec<char> = old.chars().collect();
    let b: Vec<char> = new.chars().collect();
    let mut start = a.iter().zip(&b).take_while(|(a, b)| a == b).count();
    let (mut ae, mut be) = (a.len(), b.len());
    while ae > start && be > start && a[ae - 1] == b[be - 1] {
        ae -= 1;
        be -= 1;
    }
    let (ae_edges, be_edges) = (boundaries(old), boundaries(new));
    while !ae_edges.contains(&start) || !be_edges.contains(&start) {
        start -= 1;
    }
    while !ae_edges.contains(&ae) || !be_edges.contains(&be) {
        ae += 1;
        be += 1;
    }
    (start, ae, b[start..be].iter().collect())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RibbonDensity {
    Full,
    Compact,
    Overflow,
}
pub fn density(width: f32) -> RibbonDensity {
    if width >= 1180.0 {
        RibbonDensity::Full
    } else if width >= 900.0 {
        RibbonDensity::Compact
    } else {
        RibbonDensity::Overflow
    }
}
pub fn group_widths(width: f32) -> [f32; 5] {
    match density(width) {
        RibbonDensity::Full => [86., 284., 226., 390., 116.],
        RibbonDensity::Compact => [82., 246., 218., 140., 104.],
        RibbonDensity::Overflow => [82., 232., 206., 0., 0.],
    }
}
pub fn selected_text(document: &Document, range: (usize, usize)) -> String {
    let mut offset = 0;
    let mut out = String::new();
    for (index, p) in document.paragraphs.iter().enumerate() {
        let end = offset + p.styles.len();
        if range.0 < end && range.1 > offset {
            if p.structure.block.is_some() {
                out.push_str(&crate::structure::plain_paragraph(p));
            } else {
                out.push_str(&crate::model::slice(
                    &p.text,
                    range.0.saturating_sub(offset),
                    (range.1 - offset).min(p.styles.len()),
                ));
            }
        }
        if index + 1 < document.paragraphs.len() && range.0 <= end && range.1 > end {
            out.push('\n');
        }
        offset = end + 1;
    }
    out
}

/// Prefer the actual edit selection when equal adjacent characters make a diff ambiguous.
/// Fall back if navigation/deletion in the same batch changed the affected range.
pub fn text_change_from_selection(
    old: &str,
    new: &str,
    selection: (usize, usize),
) -> (usize, usize, String) {
    let (start, end) = selection;
    if crate::model::validate_range(old, start, end).is_ok() {
        let prefix = crate::model::slice(old, 0, start);
        let suffix = crate::model::slice(old, end, old.chars().count());
        if new.len() >= prefix.len() + suffix.len()
            && new.starts_with(&prefix)
            && new.ends_with(&suffix)
        {
            let inserted = &new[prefix.len()..new.len() - suffix.len()];
            let stop = start + inserted.chars().count();
            let edges = boundaries(new);
            if edges.contains(&start) && edges.contains(&stop) {
                return (start, end, inserted.into());
            }
        }
    }
    text_change(old, new)
}
