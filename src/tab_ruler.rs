//! Toolkit-independent creation, movement and removal of explicit left tab stops.
use crate::{Result, commands::Editor, model::Document};

pub struct Draft {
    original: Document,
    selection: (usize, usize),
    source: Option<u16>,
    positions: Vec<u16>,
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
        let positions = editor.document.paragraphs[first]
            .spacing
            .left_tab_stops_half_points
            .clone();
        if source.is_some_and(|source| !positions.contains(&source)) {
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
            position: source.unwrap_or(1),
            deleting: false,
        })
    }
    pub fn move_to(&mut self, half_points: f32) -> Result<()> {
        self.deleting = false;
        self.valid = false;
        self.preview = self.original.clone();
        if !half_points.is_finite() {
            return Err("Invalid tab position.".into());
        }
        let position = half_points.round().clamp(1., 2880.) as u16;
        if Some(position) != self.source && self.positions.contains(&position) {
            return Err(
                "A tab already occupies this position. Choose another position or cancel.".into(),
            );
        }
        let mut positions = self.positions.clone();
        positions.retain(|p| Some(*p) != self.source);
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
    pub fn remove(&mut self) -> Result<()> {
        self.valid = false;
        let source = self.source.ok_or("Cannot remove an uncommitted new tab.")?;
        self.preview = self.original.clone();
        for paragraph in &mut self.preview.paragraphs[self.first..=self.last] {
            paragraph
                .spacing
                .left_tab_stops_half_points
                .retain(|p| *p != source);
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
