//! Toolkit-independent creation, movement and removal of explicit typed tab stops.
use crate::{
    Result,
    commands::Editor,
    model::{Document, TabKind},
};

pub struct Draft {
    original: Document,
    selection: (usize, usize),
    source: Option<u16>,
    first: usize,
    last: usize,
    valid: bool,
    pub preview: Document,
    pub position: u16,
    pub deleting: bool,
}
impl Draft {
    pub fn new(editor: &Editor, source: u16) -> Result<Self> {
        Self::snapshot(editor, Some(source))
    }
    /// Uncommitted creation session; invalid until a position passes validation.
    pub fn creation_preview(editor: &Editor) -> Result<Self> {
        let mut draft = Self::snapshot(editor, None)?;
        draft.position = 72;
        draft.valid = false;
        Ok(draft)
    }
    pub fn kind(&self) -> Option<TabKind> {
        match self.source {
            Some(position) => common_kind(&self.original, self.selection, position),
            None => Some(TabKind::Left),
        }
    }
    pub fn is_creation(&self) -> bool {
        self.source.is_none()
    }
    pub fn create(editor: &Editor, position: u16) -> Result<Self> {
        if !(1..=2880).contains(&position) {
            return Err("Tab position must be 0.5–1440 pt.".into());
        }
        let mut draft = Self::snapshot(editor, None)?;
        draft.move_to(f32::from(position))?;
        Ok(draft)
    }
    fn snapshot(editor: &Editor, source: Option<u16>) -> Result<Self> {
        editor.document.validate()?;
        let (start, end) = editor.selection;
        if start > end {
            return Err("Invalid tab ruler selection.".into());
        }
        let (first, _) = editor.document.locate(start)?;
        let (mut last, offset) = editor.document.locate(end)?;
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        if source.is_some_and(|value| {
            !editor.document.paragraphs[first..=last]
                .iter()
                .any(|p| p.spacing.tab_positions().iter().any(|s| s.0 == value))
        }) {
            return Err("Tab position no longer exists.".into());
        }
        Ok(Self {
            original: editor.document.clone(),
            preview: editor.document.clone(),
            selection: editor.selection,
            source,
            first,
            last,
            valid: true,
            position: source.unwrap_or(1),
            deleting: false,
        })
    }
    /// Exact point input: never round or clamp a typed value, and invalidate stale previews.
    pub fn move_to_text(&mut self, text: &str) -> Result<()> {
        self.valid = false;
        self.deleting = false;
        self.preview = self.original.clone();
        let text = text.trim();
        let (whole, fraction) = text.split_once('.').unwrap_or((text, "0"));
        let half = match fraction {
            "" | "0" => 0,
            "5" => 1,
            _ => return Err("Use 0.5 pt steps, for example 48.5.".into()),
        };
        let value = whole
            .parse::<u16>()
            .ok()
            .and_then(|v| v.checked_mul(2))
            .and_then(|v| v.checked_add(half))
            .filter(|v| (1..=2880).contains(v))
            .ok_or("Enter 0.5–1440 pt.")?;
        self.move_to(f32::from(value))
    }
    pub fn move_to(&mut self, half_points: f32) -> Result<()> {
        self.deleting = false;
        self.valid = false;
        self.preview = self.original.clone();
        if !half_points.is_finite() {
            return Err("Invalid tab position.".into());
        }
        let position = half_points.round().clamp(1., 2880.) as u16;
        let mut preview = self.original.clone();
        let mut added = false;
        for paragraph in &mut preview.paragraphs[self.first..=self.last] {
            let mut positions = paragraph.spacing.tab_positions();
            let kind = if let Some(source) = self.source {
                let Some((_, kind)) = positions.iter().find(|s| s.0 == source).copied() else {
                    continue;
                };
                if position != source && positions.iter().any(|s| s.0 == position) {
                    return Err("A selected paragraph already has a tab at this destination. Choose another position or cancel.".into());
                }
                positions.retain(|s| s.0 != source);
                kind
            } else {
                if let Some((_, kind)) = positions.iter().find(|s| s.0 == position) {
                    if *kind == TabKind::Left {
                        continue;
                    }
                    return Err("A different tab type already uses this position.".into());
                }
                TabKind::Left
            };
            positions.push((position, kind));
            positions.sort_by_key(|s| s.0);
            paragraph.spacing.set_tab_positions(&positions)?;
            added = true;
        }
        if self.source.is_none() && !added {
            return Err("All selected paragraphs already have this tab.".into());
        }
        preview.validate()?;
        self.preview = preview;
        self.position = position;
        self.valid = true;
        Ok(())
    }
    pub fn remove(&mut self) -> Result<()> {
        self.valid = false;
        let source = self.source.ok_or("Cannot remove an uncommitted new tab.")?;
        self.preview = self.original.clone();
        for paragraph in &mut self.preview.paragraphs[self.first..=self.last] {
            let positions: Vec<_> = paragraph
                .spacing
                .tab_positions()
                .into_iter()
                .filter(|s| s.0 != source)
                .collect();
            paragraph.spacing.set_tab_positions(&positions)?;
        }
        self.preview.validate()?;
        self.deleting = true;
        self.valid = true;
        Ok(())
    }
    pub fn commit(self, editor: &mut Editor) -> Result<()> {
        if !self.valid {
            return Err("Invalid tab preview; drag cancelled.".into());
        }
        if editor.document != self.original || editor.selection != self.selection {
            return Err("Tab drag cancelled because the document or selection changed.".into());
        }
        editor.set_paragraph_tab_lists(
            self.selection.0,
            self.selection.1,
            self.preview.paragraphs[self.first..=self.last]
                .iter()
                .map(|p| p.spacing.tab_positions())
                .collect(),
        )
    }
}

/// Sorted union with coverage counts for the half-open paragraph selection.
pub fn positions(document: &Document, selection: (usize, usize)) -> Result<Vec<(u16, usize)>> {
    if selection.0 > selection.1 {
        return Err("Invalid tab ruler selection.".into());
    }
    let (first, _) = document.locate(selection.0)?;
    let (mut last, offset) = document.locate(selection.1)?;
    if selection.1 > selection.0 && offset == 0 && last > first {
        last -= 1;
    }
    let mut counts = std::collections::BTreeMap::new();
    for paragraph in &document.paragraphs[first..=last] {
        for (position, _) in paragraph.spacing.tab_positions() {
            *counts.entry(position).or_insert(0) += 1;
        }
    }
    Ok(counts.into_iter().collect())
}

/// Common kind among paragraphs owning a position; None denotes mixed kinds.
pub fn common_kind(
    document: &Document,
    selection: (usize, usize),
    position: u16,
) -> Option<TabKind> {
    if selection.0 > selection.1 {
        return None;
    }
    let (first, _) = document.locate(selection.0).ok()?;
    let (mut last, offset) = document.locate(selection.1).ok()?;
    if selection.1 > selection.0 && offset == 0 && last > first {
        last -= 1;
    }
    if first > last {
        return None;
    }
    let mut kinds = document.paragraphs[first..=last].iter().filter_map(|p| {
        p.spacing
            .tab_positions()
            .into_iter()
            .find(|s| s.0 == position)
            .map(|s| s.1)
    });
    let first = kinds.next()?;
    kinds.all(|k| k == first).then_some(first)
}
