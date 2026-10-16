//! Shared validated commands: desktop input and tests never implement separate edits.
use crate::{Result, model::*};

#[derive(Clone, Debug)]
pub struct Operation {
    pub label: String,
    before: Document,
    after: Document,
    before_selection: (usize, usize),
    after_selection: (usize, usize),
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
            document,
            ..Self::default()
        }
    }

    fn commit(&mut self, document: Document, label: &str, selection: (usize, usize)) {
        if document != self.document {
            self.undo_stack.push(Operation {
                label: label.into(),
                before: self.document.clone(),
                after: document.clone(),
                before_selection: self.selection,
                after_selection: selection,
            });
            if self.undo_stack.len() > 100 {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
            self.document = document;
        }
        self.selection = selection;
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
        self.commit(result, label, (caret, caret));
        Ok(())
    }

    pub fn emphasize(&mut self, start: usize, end: usize, bold: bool, enabled: bool) -> Result<()> {
        validate_range(&self.document.text(), start, end)?;
        if start == end {
            if bold {
                self.typing_style.bold = enabled;
            } else {
                self.typing_style.italic = enabled;
            }
            return Ok(());
        }
        let mut result = self.document.clone();
        let mut offset = 0;
        for p in &mut result.paragraphs {
            for (i, style) in p.styles.iter_mut().enumerate() {
                if (start..end).contains(&(offset + i)) {
                    if bold {
                        style.bold = enabled;
                    } else {
                        style.italic = enabled;
                    }
                }
            }
            offset += p.styles.len() + 1;
        }
        self.selection = (start, end);
        self.commit(result, if bold { "Bold" } else { "Italic" }, (start, end));
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
        self.commit(result, "Align paragraphs", (start, end));
        Ok(())
    }

    pub fn emphasis_selected(&self, bold: bool) -> bool {
        let (start, end) = self.selection;
        if start == end {
            return if bold {
                self.typing_style.bold
            } else {
                self.typing_style.italic
            };
        }
        let mut offset = 0;
        let mut selected = Vec::new();
        for p in &self.document.paragraphs {
            selected.extend(
                p.styles
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| (start..end).contains(&(offset + i)))
                    .map(|(_, s)| if bold { s.bold } else { s.italic }),
            );
            offset += p.styles.len() + 1;
        }
        !selected.is_empty() && selected.iter().all(|s| *s)
    }

    pub fn undo(&mut self) {
        if let Some(op) = self.undo_stack.pop() {
            self.document = op.before.clone();
            self.selection = op.before_selection;
            self.typing_style = self.document.style_at(self.selection.0);
            self.redo_stack.push(op);
        }
    }
    pub fn redo(&mut self) {
        if let Some(op) = self.redo_stack.pop() {
            self.document = op.after.clone();
            self.selection = op.after_selection;
            self.typing_style = self.document.style_at(self.selection.0);
            self.undo_stack.push(op);
        }
    }
}
