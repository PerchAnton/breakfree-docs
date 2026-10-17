//! Shared validated commands: desktop input and tests never implement separate edits.
use crate::{Result, model::*};

#[path = "replace_all.rs"]
mod replace_all;
pub use replace_all::ReplacementPlan;

/// A partial character-format change. Unspecified properties keep each run's value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FontPatch {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub font_size: Option<FontSize>,
}

#[derive(Clone, Debug)]
pub struct Operation {
    pub label: String,
    before: Document,
    after: Document,
    before_selection: (usize, usize),
    after_selection: (usize, usize),
    before_typing_style: Style,
    after_typing_style: Style,
}

#[derive(Default)]
pub struct Editor {
    pub document: Document,
    pub selection: (usize, usize),
    pub typing_style: Style,
    pub undo_stack: Vec<Operation>,
    pub redo_stack: Vec<Operation>,
}

impl Editor {
    pub fn new(document: Document) -> Self {
        Self {
            typing_style: document.style_at(0),
            document,
            ..Self::default()
        }
    }

    fn commit(
        &mut self,
        document: Document,
        label: &str,
        selection: (usize, usize),
        typing: Style,
    ) {
        if document != self.document {
            self.undo_stack.push(Operation {
                label: label.into(),
                before: self.document.clone(),
                after: document.clone(),
                before_selection: self.selection,
                after_selection: selection,
                before_typing_style: self.typing_style,
                after_typing_style: typing,
            });
            if self.undo_stack.len() > 100 {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
            self.document = document;
        }
        self.selection = selection;
        self.typing_style = typing;
    }

    pub fn replace(&mut self, start: usize, end: usize, text: &str, label: &str) -> Result<()> {
        validate_range(&self.document.text(), start, end)?;
        validate_text(text)?;
        let (first, left) = self.document.locate(start)?;
        let (last, right) = self.document.locate(end)?;
        let a = &self.document.paragraphs[first];
        let b = &self.document.paragraphs[last];
        let joined = format!(
            "{}{}{}",
            slice(&a.text, 0, left),
            text,
            slice(&b.text, right, b.styles.len())
        );
        let styles: Vec<Style> = a.styles[..left]
            .iter()
            .copied()
            .chain(std::iter::repeat_n(self.typing_style, text.chars().count()))
            .chain(b.styles[right..].iter().copied())
            .collect();
        let mut paragraphs = self.document.paragraphs[..first].to_vec();
        let mut offset = 0;
        for part in joined.split('\n') {
            let len = part.chars().count();
            paragraphs.push(Paragraph {
                text: part.into(),
                styles: styles[offset..offset + len].to_vec(),
                alignment: a.alignment,
            });
            offset += len + 1;
        }
        paragraphs.extend_from_slice(&self.document.paragraphs[last + 1..]);
        let result = Document { paragraphs };
        result.validate()?;
        let after = start + text.chars().count();
        let caret = boundaries(&result.text())
            .into_iter()
            .find(|p| *p >= after)
            .unwrap_or(after);
        self.selection = (start, end);
        self.commit(result, label, (caret, caret), self.typing_style);
        Ok(())
    }

    pub fn emphasize(&mut self, start: usize, end: usize, bold: bool, enabled: bool) -> Result<()> {
        self.format_range(start, end, if bold { "Bold" } else { "Italic" }, |style| {
            if bold {
                style.bold = enabled;
            } else {
                style.italic = enabled;
            }
        })
    }

    pub fn underline(&mut self, start: usize, end: usize, enabled: bool) -> Result<()> {
        self.format_range(start, end, "Underline", |style| style.underline = enabled)
    }

    pub fn set_font_size(&mut self, start: usize, end: usize, size: FontSize) -> Result<()> {
        self.format_range(start, end, "Font size", |style| style.font_size = size)
    }

    /// Reset supported direct character properties, preserving paragraph formatting.
    /// At a caret, reset only the pending insertion style, like other font commands.
    pub fn clear_character_formatting(&mut self, start: usize, end: usize) -> Result<()> {
        self.format_range(start, end, "Clear character formatting", |style| {
            *style = Style::default();
        })
    }

    /// Apply a dialog's changes together, with one undo boundary and no live edits.
    pub fn set_font(&mut self, start: usize, end: usize, patch: FontPatch) -> Result<()> {
        validate_range(&self.document.text(), start, end)?;
        if patch == FontPatch::default() {
            return Ok(());
        }
        self.format_range(start, end, "Font", |style| {
            if let Some(value) = patch.bold {
                style.bold = value;
            }
            if let Some(value) = patch.italic {
                style.italic = value;
            }
            if let Some(value) = patch.underline {
                style.underline = value;
            }
            if let Some(value) = patch.font_size {
                style.font_size = value;
            }
        })
    }

    /// Uniform values for the selection; None means mixed or only separators.
    pub fn selected_font(&self) -> FontPatch {
        FontPatch {
            bold: self.selected_value(|s| s.bold),
            italic: self.selected_value(|s| s.italic),
            underline: self.selected_value(|s| s.underline),
            font_size: self.selected_font_size(),
        }
    }

    pub fn step_font_size(&mut self, start: usize, end: usize, grow: bool) -> Result<()> {
        self.format_range(
            start,
            end,
            if grow { "Grow font" } else { "Shrink font" },
            |style| {
                style.font_size = style.font_size.stepped(grow);
            },
        )
    }

    fn format_range(
        &mut self,
        start: usize,
        end: usize,
        label: &str,
        change: impl Fn(&mut Style),
    ) -> Result<()> {
        validate_range(&self.document.text(), start, end)?;
        if start == end {
            change(&mut self.typing_style);
            return Ok(());
        }
        let mut result = self.document.clone();
        let mut offset = 0;
        let mut typing = None;
        for p in &mut result.paragraphs {
            for (i, style) in p.styles.iter_mut().enumerate() {
                if (start..end).contains(&(offset + i)) {
                    change(style);
                    typing.get_or_insert(*style);
                }
            }
            offset += p.styles.len() + 1;
        }
        self.selection = (start, end);
        self.commit(
            result,
            label,
            (start, end),
            typing.unwrap_or(self.typing_style),
        );
        Ok(())
    }

    pub fn align(&mut self, start: usize, end: usize, alignment: Alignment) -> Result<()> {
        validate_range(&self.document.text(), start, end)?;
        let (first, _) = self.document.locate(start)?;
        let (mut last, offset) = self.document.locate(end)?;
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        let mut result = self.document.clone();
        for p in &mut result.paragraphs[first..=last] {
            p.alignment = alignment;
        }
        self.selection = (start, end);
        self.commit(result, "Align paragraphs", (start, end), self.typing_style);
        Ok(())
    }

    pub fn emphasis_selected(&self, bold: bool) -> bool {
        self.selected_value(|s| if bold { s.bold } else { s.italic }) == Some(true)
    }

    pub fn underline_selected(&self) -> bool {
        self.selected_value(|s| s.underline) == Some(true)
    }

    /// None represents mixed values or a selection containing only separators.
    pub fn selected_font_size(&self) -> Option<FontSize> {
        self.selected_value(|s| s.font_size)
    }

    fn selected_value<T: Copy + Eq>(&self, value: impl Fn(Style) -> T) -> Option<T> {
        let (start, end) = self.selection;
        if start == end {
            return Some(value(self.typing_style));
        }
        let mut offset = 0;
        let mut selected = Vec::new();
        for p in &self.document.paragraphs {
            selected.extend(
                p.styles
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| (start..end).contains(&(offset + i)))
                    .map(|(_, s)| value(*s)),
            );
            offset += p.styles.len() + 1;
        }
        selected
            .first()
            .copied()
            .filter(|first| selected.iter().all(|s| s == first))
    }

    pub fn undo(&mut self) {
        if let Some(op) = self.undo_stack.pop() {
            self.document = op.before.clone();
            self.selection = op.before_selection;
            self.typing_style = op.before_typing_style;
            self.redo_stack.push(op);
        }
    }
    pub fn redo(&mut self) {
        if let Some(op) = self.redo_stack.pop() {
            self.document = op.after.clone();
            self.selection = op.after_selection;
            self.typing_style = op.after_typing_style;
            self.undo_stack.push(op);
        }
    }
}
