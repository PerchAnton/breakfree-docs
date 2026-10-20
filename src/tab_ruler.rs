//! Toolkit-independent preview of one existing explicit left tab stop.
use crate::{Result, commands::Editor, model::Document};

pub struct Draft {
    original: Document,
    selection: (usize, usize),
    source: u16,
    positions: Vec<u16>,
    first: usize,
    last: usize,
    valid: bool,
    pub preview: Document,
    pub position: u16,
}
impl Draft {
    pub fn new(editor: &Editor, source: u16) -> Result<Self> {
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
        let positions = editor.document.paragraphs[first]
            .spacing
            .left_tab_stops_half_points
            .clone();
        if !positions.contains(&source) {
            return Err("Tab position no longer exists.".into());
        }
        if editor.document.paragraphs[first..=last]
            .iter()
            .any(|p| p.spacing.left_tab_stops_half_points != positions)
        {
            return Err("Different tab lists: use the Tab positions dialog.".into());
        }
        Ok(Self {
            original: editor.document.clone(),
            preview: editor.document.clone(),
            selection: editor.selection,
            source,
            positions,
            first,
            last,
            valid: true,
            position: source,
        })
    }
    pub fn move_to(&mut self, half_points: f32) -> Result<()> {
        self.valid = false;
        self.preview = self.original.clone();
        if !half_points.is_finite() {
            return Err("Invalid tab position.".into());
        }
        let position = half_points.round().clamp(1., 2880.) as u16;
        if position != self.source && self.positions.contains(&position) {
            return Err(
                "A tab already occupies this position. Choose another position or cancel.".into(),
            );
        }
        let mut positions = self.positions.clone();
        positions.retain(|p| *p != self.source);
        positions.push(position);
        positions.sort_unstable();
        for p in &mut self.preview.paragraphs[self.first..=self.last] {
            p.spacing.left_tab_stops_half_points = positions.clone();
        }
        self.preview.validate()?;
        self.position = position;
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
        editor.set_left_tab_stops(
            self.selection.0,
            self.selection.1,
            self.preview.paragraphs[self.first]
                .spacing
                .left_tab_stops_half_points
                .clone(),
        )
    }
}
