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
        })
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
            operation.label = "Ruler indent".into();
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
}
