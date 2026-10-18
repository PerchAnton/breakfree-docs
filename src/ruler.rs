//! Toolkit-independent ruler draft. Preview never changes the live editor/history.
use crate::{
    Result,
    commands::{Editor, ParagraphGeometryPatch},
    model::{Document, ResolvedParagraphStyle},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    Left,
    First,
    Body,
    Right,
}

impl Handle {
    /// Horizontal position in half points from the text origin (right is reversed).
    pub fn position(self, style: ResolvedParagraphStyle) -> i32 {
        let left = i32::from(style.left_indent_half_points);
        let special = i32::from(style.first_line_half_points);
        match self {
            Self::Left => left,
            Self::First => left + special.max(0),
            Self::Body => left + (-special).max(0),
            Self::Right => -i32::from(style.right_indent_half_points),
        }
    }
}

pub fn values(style: ResolvedParagraphStyle) -> String {
    format!(
        "Left {:.1} pt   Right {:.1} pt   First {:.1} pt   Hanging {:.1} pt",
        f32::from(style.left_indent_half_points) / 2.,
        f32::from(style.right_indent_half_points) / 2.,
        f32::from(style.first_line_half_points.max(0)) / 2.,
        f32::from((-style.first_line_half_points).max(0)) / 2.
    )
}

/// Resolved half-point range: identical appearance is common even with different sources.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValueRange {
    pub min: i32,
    pub max: i32,
}
impl ValueRange {
    fn new(value: i32) -> Self {
        Self {
            min: value,
            max: value,
        }
    }
    fn include(&mut self, value: i32) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }
    pub fn mixed(self) -> bool {
        self.min != self.max
    }
    fn display(self) -> String {
        if self.mixed() {
            "Mixed".into()
        } else {
            format!("{:.1} pt", self.min as f32 / 2.)
        }
    }
    fn detail(self) -> String {
        if self.mixed() {
            format!("{:.1}–{:.1} pt", self.min as f32 / 2., self.max as f32 / 2.)
        } else {
            self.display()
        }
    }
}
#[derive(Clone, Debug)]
pub struct SelectionValues {
    pub count: usize,
    /// Left, right, first-line amount, hanging amount (not absolute marker origins).
    pub amounts: [ValueRange; 4],
    /// Overall left, first-line origin, continuation origin, right inset.
    pub positions: [ValueRange; 4],
}
impl SelectionValues {
    pub fn read(document: &Document, selection: (usize, usize)) -> Result<Self> {
        if selection.0 > selection.1 {
            return Err("Invalid ruler selection.".into());
        }
        let (first, _) = document.locate(selection.0)?;
        let (mut last, offset) = document.locate(selection.1)?;
        if selection.1 > selection.0 && offset == 0 && last > first {
            last -= 1;
        }
        let collect = |style: ResolvedParagraphStyle| {
            (
                [
                    i32::from(style.left_indent_half_points),
                    i32::from(style.right_indent_half_points),
                    i32::from(style.first_line_half_points.max(0)),
                    i32::from((-style.first_line_half_points).max(0)),
                ],
                [
                    Handle::Left.position(style),
                    Handle::First.position(style),
                    Handle::Body.position(style),
                    -Handle::Right.position(style),
                ],
            )
        };
        let (amounts, positions) =
            collect(document.resolved_paragraph(&document.paragraphs[first]));
        let mut result = Self {
            count: last - first + 1,
            amounts: amounts.map(ValueRange::new),
            positions: positions.map(ValueRange::new),
        };
        for paragraph in &document.paragraphs[first + 1..last + 1] {
            let (amounts, positions) = collect(document.resolved_paragraph(paragraph));
            for i in 0..4 {
                result.amounts[i].include(amounts[i]);
                result.positions[i].include(positions[i]);
            }
        }
        Ok(result)
    }
    pub fn position(&self, handle: Handle) -> ValueRange {
        self.positions[match handle {
            Handle::Left => 0,
            Handle::First => 1,
            Handle::Body => 2,
            Handle::Right => 3,
        }]
    }
    pub fn mixed(&self) -> bool {
        self.amounts.iter().any(|v| v.mixed())
    }
    pub fn values(&self) -> String {
        ["Left", "Right", "First", "Hanging"]
            .into_iter()
            .zip(self.amounts)
            .map(|(label, value)| format!("{label} {}", value.display()))
            .collect::<Vec<_>>()
            .join("   ")
    }
    pub fn details(&self) -> String {
        ["Left", "Right", "First", "Hanging"]
            .into_iter()
            .zip(self.amounts)
            .map(|(label, value)| format!("{label}: {}", value.detail()))
            .collect::<Vec<_>>()
            .join("; ")
    }
    pub fn marker_details(&self, handle: Handle) -> String {
        format!(
            "{} paragraph(s). {} Marker {}: {}. Markers are anchored to the first selected paragraph; editing applies changed properties to the selection.",
            self.count,
            self.details(),
            if handle == Handle::Right {
                "inset from right"
            } else {
                "origin from left"
            },
            self.position(handle).detail()
        )
    }
}

/// Shared narrow-column projection, also used by the native text adapter.
pub fn indent_scale(style: ResolvedParagraphStyle, unit: f32, width: f32) -> f32 {
    let total = (f32::from(style.left_indent_half_points)
        + f32::from(style.right_indent_half_points)
        + f32::from(style.first_line_half_points).abs())
        * unit;
    if total > (width - 32.).max(0.) {
        (width - 32.).max(0.) / total
    } else {
        1.
    }
}

pub struct Draft {
    pub original: Document,
    pub preview: Document,
    pub selection: (usize, usize),
    pub handle: Handle,
    pub style: ResolvedParagraphStyle,
    patch: ParagraphGeometryPatch,
    pub resetting: bool,
}
impl Draft {
    pub fn new(editor: &Editor, handle: Handle) -> Result<Self> {
        let (index, _) = editor.document.locate(editor.selection.0)?;
        Ok(Self {
            original: editor.document.clone(),
            preview: editor.document.clone(),
            selection: editor.selection,
            handle,
            style: editor
                .document
                .resolved_paragraph(&editor.document.paragraphs[index]),
            patch: ParagraphGeometryPatch::default(),
            resetting: false,
        })
    }
    /// Remove only direct indent overrides; each paragraph keeps its own style chain.
    pub fn reset_indents(&mut self) -> Result<()> {
        let patch = ParagraphGeometryPatch {
            left: Some(None),
            right: Some(None),
            first_line: Some(None),
            ..Default::default()
        };
        let mut preview = Editor::new(self.original.clone());
        preview.set_paragraph_geometry(self.selection.0, self.selection.1, patch)?;
        self.preview = preview.document;
        self.patch = patch;
        self.resetting = true;
        Ok(())
    }
    /// Absolute displacement from press, in half points; rounding never accumulates.
    pub fn move_by(&mut self, delta: f32) -> Result<()> {
        if !delta.is_finite() {
            return Err("Invalid ruler displacement.".into());
        }
        let delta = delta.round().clamp(-576., 576.) as i32;
        let left = i32::from(self.style.left_indent_half_points);
        let right = i32::from(self.style.right_indent_half_points);
        let special = i32::from(self.style.first_line_half_points);
        let first = left + special.max(0);
        let body = left + (-special).max(0);
        let mut patch = ParagraphGeometryPatch::default();
        match self.handle {
            Handle::Left => {
                let value = (left + delta).clamp(0, 288);
                if value != left {
                    patch.left = Some(Some(value as u16));
                }
            }
            Handle::Right => {
                let value = (right - delta).clamp(0, 288);
                if value != right {
                    patch.right = Some(Some(value as u16));
                }
            }
            Handle::First | Handle::Body => {
                let (moving, fixed) = if self.handle == Handle::First {
                    (first, body)
                } else {
                    (body, first)
                };
                let max = if fixed > 288 { 288 } else { fixed + 288 };
                let value = (moving + delta).clamp((fixed - 288).max(0), max);
                if value != moving {
                    if value.min(fixed) != left {
                        patch.left = Some(Some(value.min(fixed) as u16));
                    }
                    patch.first_line = Some(Some(if self.handle == Handle::First {
                        value - fixed
                    } else {
                        fixed - value
                    } as i16));
                }
            }
        }
        let mut preview = Editor::new(self.original.clone());
        preview.set_paragraph_geometry(self.selection.0, self.selection.1, patch)?;
        self.preview = preview.document;
        self.patch = patch;
        Ok(())
    }
    pub fn commit(self, editor: &mut Editor) -> Result<()> {
        if editor.document != self.original || editor.selection != self.selection {
            return Err("Ruler drag cancelled because the document or selection changed.".into());
        }
        editor.set_paragraph_geometry(self.selection.0, self.selection.1, self.patch)?;
        if self.preview != self.original
            && let Some(operation) = editor.undo_stack.last_mut()
        {
            operation.label = if self.resetting {
                "Reset ruler indents"
            } else {
                "Ruler indent"
            }
            .into();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn editor() -> Editor {
        let mut e = Editor::default();
        e.replace(0, 0, "English cafe\u{301}\nУкраїна\nРусский", "Fixture")
            .unwrap();
        e.selection = (0, e.document.text().chars().count());
        e.undo_stack.clear();
        e
    }
    #[test]
    fn reset_inherits_each_style_preserves_spacing_and_roundtrips_one_history_entry() {
        let mut e = editor();
        e.document.style_catalog.normal.left_indent_half_points = Some(37);
        e.document.style_catalog.heading1.right_indent_half_points = Some(21);
        e.document.paragraphs[0].paragraph_style = crate::model::ParagraphStyle::Heading1;
        for p in &mut e.document.paragraphs {
            p.spacing.left_indent_half_points = Some(0);
            p.spacing.right_indent_half_points = Some(88);
            p.spacing.first_line_half_points = Some(-72);
            p.spacing.space_before_half_points = Some(11);
        }
        e.selection = (
            0,
            e.document.paragraphs[..2]
                .iter()
                .map(|p| p.text.chars().count() + 1)
                .sum(),
        );
        let before = e.document.clone();
        let mut draft = Draft::new(&e, Handle::Left).unwrap();
        draft.reset_indents().unwrap();
        assert_eq!(e.document, before);
        for p in &draft.preview.paragraphs[..2] {
            assert_eq!(p.spacing.left_indent_half_points, None);
            assert_eq!(p.spacing.right_indent_half_points, None);
            assert_eq!(p.spacing.first_line_half_points, None);
            assert_eq!(p.spacing.space_before_half_points, Some(11));
        }
        assert_eq!(draft.preview.paragraphs[2], before.paragraphs[2]);
        assert_eq!(
            draft
                .preview
                .resolved_paragraph(&draft.preview.paragraphs[0])
                .right_indent_half_points,
            21
        );
        assert_eq!(
            draft
                .preview
                .resolved_paragraph(&draft.preview.paragraphs[1])
                .left_indent_half_points,
            37
        );
        let after = draft.preview.clone();
        draft.commit(&mut e).unwrap();
        assert_eq!(e.undo_stack.len(), 1);
        assert_eq!(e.undo_stack[0].label, "Reset ruler indents");
        e.undo();
        assert_eq!(e.document, before);
        e.redo();
        assert_eq!(e.document, after);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reset.bfdoc");
        crate::storage::save(&e.document, &path).unwrap();
        assert_eq!(crate::storage::load(&path).unwrap(), after);
    }
    #[test]
    fn reset_cancel_and_noop_preserve_redo_and_stale_reset_is_rejected() {
        let mut e = editor();
        let mut edit = Draft::new(&e, Handle::Left).unwrap();
        edit.move_by(40.).unwrap();
        edit.commit(&mut e).unwrap();
        e.undo();
        let before = e.document.clone();
        let mut reset = Draft::new(&e, Handle::Left).unwrap();
        reset.reset_indents().unwrap();
        drop(reset);
        assert_eq!(e.document, before);
        assert_eq!(e.redo_stack.len(), 1);
        let mut reset = Draft::new(&e, Handle::Left).unwrap();
        reset.reset_indents().unwrap();
        reset.commit(&mut e).unwrap();
        assert_eq!(e.redo_stack.len(), 1);
        assert!(e.undo_stack.is_empty());
        let mut reset = Draft::new(&e, Handle::Left).unwrap();
        reset.reset_indents().unwrap();
        e.selection = (0, 0);
        assert!(reset.commit(&mut e).is_err());
    }
    #[test]
    fn draft_preview_commit_history_and_reopen() {
        let mut e = editor();
        let before = e.document.clone();
        let mut draft = Draft::new(&e, Handle::First).unwrap();
        for value in [1., 20., 72.] {
            draft.move_by(value).unwrap();
        }
        assert_eq!(e.document, before);
        assert!(e.undo_stack.is_empty());
        let preview = draft.preview.clone();
        draft.commit(&mut e).unwrap();
        assert_eq!(e.document, preview);
        assert_eq!(e.undo_stack.len(), 1);
        assert_eq!(e.undo_stack[0].label, "Ruler indent");
        e.undo();
        assert_eq!(e.document, before);
        e.redo();
        assert_eq!(e.document, preview);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ruler.bfdoc");
        crate::storage::save(&e.document, &path).unwrap();
        assert_eq!(crate::storage::load(&path).unwrap(), preview);
    }
    #[test]
    fn cancel_no_motion_and_return_to_origin_preserve_inheritance_and_redo() {
        let mut e = editor();
        e.set_paragraph_geometry(
            0,
            0,
            ParagraphGeometryPatch {
                left: Some(Some(40)),
                ..Default::default()
            },
        )
        .unwrap();
        e.undo();
        let before = e.document.clone();
        let mut draft = Draft::new(&e, Handle::Body).unwrap();
        draft.move_by(100.).unwrap();
        drop(draft);
        assert_eq!(e.document, before);
        assert_eq!(e.redo_stack.len(), 1);
        let mut draft = Draft::new(&e, Handle::Left).unwrap();
        draft.move_by(72.).unwrap();
        draft.move_by(0.).unwrap();
        draft.commit(&mut e).unwrap();
        assert_eq!(e.document, before);
        assert_eq!(e.redo_stack.len(), 1);
    }
    #[test]
    fn all_handles_clamp_and_keep_unrelated_properties_and_exclusive_end() {
        for handle in [Handle::First, Handle::Body, Handle::Left, Handle::Right] {
            for special in [-288, 0, 288] {
                for delta in [-10000., -72., 0., 72., 10000.] {
                    let mut e = editor();
                    e.document.paragraphs[0].spacing.left_indent_half_points = Some(144);
                    e.document.paragraphs[0].spacing.first_line_half_points = Some(special);
                    e.document.paragraphs[0].spacing.space_before_half_points = Some(30);
                    e.selection = (0, e.document.paragraphs[0].text.chars().count() + 1);
                    let before = e.document.clone();
                    let mut draft = Draft::new(&e, handle).unwrap();
                    draft.move_by(delta).unwrap();
                    draft.commit(&mut e).unwrap();
                    e.document.validate().unwrap();
                    assert_eq!(e.document.paragraphs[1..], before.paragraphs[1..]);
                    assert_eq!(
                        e.document.paragraphs[0].spacing.space_before_half_points,
                        Some(30)
                    );
                    assert_eq!(e.document.text(), before.text());
                    if !e.undo_stack.is_empty() {
                        e.undo();
                        assert_eq!(e.document, before);
                    }
                }
            }
        }
    }
    #[test]
    fn stale_draft_cannot_overwrite_an_edit() {
        let mut e = editor();
        let mut draft = Draft::new(&e, Handle::Right).unwrap();
        assert!(draft.move_by(f32::INFINITY).is_err());
        assert!(draft.move_by(f32::NAN).is_err());
        draft.move_by(-30.).unwrap();
        e.replace(0, 0, "X", "Edit").unwrap();
        let changed = e.document.clone();
        assert!(draft.commit(&mut e).is_err());
        assert_eq!(e.document, changed);
    }
    #[test]
    fn exact_values_distinguish_special_amounts_and_origins() {
        let mut e = editor();
        let p = &mut e.document.paragraphs[0];
        p.spacing.left_indent_half_points = Some(37);
        p.spacing.right_indent_half_points = Some(5);
        p.spacing.first_line_half_points = Some(-11);
        let style = e.document.resolved_paragraph(&e.document.paragraphs[0]);
        assert_eq!(
            values(style),
            "Left 18.5 pt   Right 2.5 pt   First 0.0 pt   Hanging 5.5 pt"
        );
        assert_eq!(Handle::First.position(style), 37);
        assert_eq!(Handle::Body.position(style), 48);
        assert_eq!(Handle::Right.position(style), -5);
    }
    #[test]
    fn mixed_selection_resolves_sources_and_exclusive_paragraph_end() {
        let mut e = editor();
        e.document.style_catalog.normal.left_indent_half_points = Some(37);
        e.document.paragraphs[0].paragraph_style = crate::model::ParagraphStyle::Heading1;
        e.document.paragraphs[1].spacing.left_indent_half_points = Some(37);
        e.document.paragraphs[2].spacing.left_indent_half_points = Some(0);
        let second_end = e.document.paragraphs[..2]
            .iter()
            .map(|p| p.text.chars().count() + 1)
            .sum();
        let common = SelectionValues::read(&e.document, (0, second_end)).unwrap();
        assert_eq!(common.count, 2);
        assert!(!common.mixed());
        assert!(common.values().contains("Left 18.5 pt"));
        let mixed = SelectionValues::read(&e.document, e.selection).unwrap();
        assert_eq!(mixed.count, 3);
        assert!(mixed.values().contains("Left Mixed"));
        assert!(mixed.details().contains("Left: 0.0–18.5 pt"));
        assert!(
            !SelectionValues::read(&e.document, (second_end, second_end))
                .unwrap()
                .mixed()
        );
        assert!(SelectionValues::read(&e.document, (5, 2)).is_err());
        assert!(SelectionValues::read(&e.document, (0, usize::MAX)).is_err());
    }
    #[test]
    fn mixed_amounts_and_marker_origins_are_distinct() {
        let mut e = Editor::default();
        e.replace(0, 0, "e\u{301}\n\n", "Fixture").unwrap();
        e.document.paragraphs[0].spacing.first_line_half_points = Some(20);
        e.document.paragraphs[1].spacing.left_indent_half_points = Some(20);
        let summary = SelectionValues::read(&e.document, (0, 4)).unwrap();
        assert_eq!(summary.count, 2);
        assert!(summary.mixed());
        assert!(!summary.position(Handle::First).mixed());
        assert!(summary.position(Handle::Body).mixed());
        assert_eq!(SelectionValues::read(&e.document, (4, 4)).unwrap().count, 1);
        e.document.paragraphs[1].spacing.first_line_half_points = Some(-20);
        let summary = SelectionValues::read(&e.document, (0, 4)).unwrap();
        assert!(summary.values().contains("First Mixed   Hanging Mixed"));
    }
    #[test]
    fn mixed_preview_cancel_commit_and_history_preserve_sources() {
        let mut e = editor();
        e.document.paragraphs[1].spacing.right_indent_half_points = Some(72);
        let before = e.document.clone();
        let count = e.undo_stack.len();
        assert!(
            SelectionValues::read(&e.document, e.selection)
                .unwrap()
                .position(Handle::Right)
                .mixed()
        );
        let mut draft = Draft::new(&e, Handle::Right).unwrap();
        draft.move_by(-10.).unwrap();
        assert!(
            !SelectionValues::read(&draft.preview, e.selection)
                .unwrap()
                .position(Handle::Right)
                .mixed()
        );
        drop(draft);
        assert_eq!(e.document, before);
        assert_eq!(e.undo_stack.len(), count);
        let mut draft = Draft::new(&e, Handle::Right).unwrap();
        draft.move_by(-10.).unwrap();
        draft.commit(&mut e).unwrap();
        assert_eq!(e.undo_stack.len(), count + 1);
        let after = e.document.clone();
        e.undo();
        assert_eq!(e.document, before);
        e.redo();
        assert_eq!(e.document, after);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mixed.bfdoc");
        crate::storage::save(&e.document, &path).unwrap();
        assert_eq!(crate::storage::load(&path).unwrap(), after);
    }
}
