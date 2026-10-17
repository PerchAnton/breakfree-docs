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
pub const TITLE_HEIGHT: f32 = 44.0;
pub const TABS_HEIGHT: f32 = 32.0;
pub const RIBBON_HEIGHT: f32 = 104.0;
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
        RibbonDensity::Full => [86., 304., 246., 350., 116.],
        RibbonDensity::Compact => [82., 246., 218., 140., 104.],
        RibbonDensity::Overflow => [82., 232., 206., 0., 0.],
    }
}
pub fn selected_text(document: &Document, range: (usize, usize)) -> String {
    crate::model::slice(&document.text(), range.0, range.1)
}
