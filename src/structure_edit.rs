//! Transactional structural edits; GUI drafts commit through these operations.
use crate::{Result, commands::Editor, model::*, structure::*};

// Preserve paragraph identity metadata on untouched text; links survive edits by
// clipping deleted characters and shifting the retained suffix. Insertions strictly
// inside a link inherit its target; insertions at either boundary do not.
#[allow(clippy::too_many_arguments)]
pub(crate) fn replacement_structure(
    a: &Paragraph,
    b: &Paragraph,
    index: usize,
    count: usize,
    left: usize,
    right: usize,
    inserted: &str,
    part: &str,
) -> Result<ParagraphStructure> {
    let mut meta = a.structure.clone();
    if index > 0 {
        meta.page_break_before = false;
        meta.section = None;
        if let Some(list) = &mut meta.list {
            list.restart = None;
        }
    }
    meta.links.clear();
    meta.block = None;
    if part == OBJECT {
        meta.block = if index == 0 {
            a.structure.block.clone()
        } else if index + 1 == count {
            b.structure.block.clone()
        } else {
            None
        };
        meta.list = None;
    }
    if part.contains(OBJECT) && meta.block.is_none() {
        return Err(
            "Edit object contents through its editor, or select the whole object to replace it."
                .into(),
        );
    }
    let first_insert = inserted.split('\n').next().unwrap_or("").chars().count();
    let last_insert = inserted
        .split('\n')
        .next_back()
        .unwrap_or("")
        .chars()
        .count();
    if index == 0 {
        for link in &a.structure.links {
            let end = link.end.min(left);
            if link.start < end {
                meta.links.push(Link {
                    start: link.start,
                    end,
                    target: link.target.clone(),
                });
            }
        }
    }
    if count == 1 {
        for link in &a.structure.links {
            if link.start < left
                && link.end > right
                && !inserted.is_empty()
                && let Some(last) = meta
                    .links
                    .last_mut()
                    .filter(|l| l.target == link.target && l.end == left)
            {
                last.end += first_insert;
            }
        }
    }
    if index + 1 == count {
        let base = if count == 1 {
            left + first_insert
        } else {
            last_insert
        };
        for link in &b.structure.links {
            let start = link.start.max(right);
            if start < link.end {
                let span = Link {
                    start: base + start - right,
                    end: base + link.end - right,
                    target: link.target.clone(),
                };
                if let Some(last) = meta
                    .links
                    .last_mut()
                    .filter(|l| l.end == span.start && l.target == span.target)
                {
                    last.end = span.end;
                } else {
                    meta.links.push(span);
                }
            }
        }
    }
    // Combining-character edits may absorb a former boundary; never keep an
    // invalid half-grapheme annotation or alter the underlying input text.
    let edges = boundaries(part);
    meta.links
        .retain(|l| edges.contains(&l.start) && edges.contains(&l.end));
    Ok(meta)
}
impl Editor {
    pub fn insert_page_break(&mut self) -> Result<()> {
        let mut draft = Editor::new(self.document.clone());
        draft.selection = self.selection;
        draft.typing_style = self.typing_style;
        let (start, end) = self.selection;
        draft.replace(start, end, "\n", "Page break")?;
        let paragraph = draft.document.locate(start + 1)?.0;
        draft.document.paragraphs[paragraph]
            .structure
            .page_break_before = true;
        draft.document.validate()?;
        self.commit(
            draft.document,
            "Page break",
            draft.selection,
            draft.typing_style,
        );
        Ok(())
    }

    pub fn set_list(&mut self, kind: Option<ListKind>) -> Result<()> {
        let mut doc = self.document.clone();
        for i in paragraph_indices(&doc, self.selection)? {
            let p = &mut doc.paragraphs[i];
            if p.structure.block.is_some() {
                return Err("Lists cannot be applied to table or picture blocks.".into());
            }
            p.structure.list = kind.map(|kind| ListItem {
                kind,
                level: p.structure.list.as_ref().map_or(0, |l| l.level),
                restart: None,
            });
        }
        self.commit_structure(doc, "List formatting")
    }
    pub fn change_list_level(&mut self, increase: bool) -> Result<()> {
        let mut doc = self.document.clone();
        for i in paragraph_indices(&doc, self.selection)? {
            let list = doc.paragraphs[i]
                .structure
                .list
                .as_mut()
                .ok_or("Select list paragraphs first.")?;
            list.level = if increase {
                (list.level + 1).min(8)
            } else {
                list.level.saturating_sub(1)
            };
        }
        self.commit_structure(doc, "List level")
    }
    pub fn restart_list(&mut self, start: u16) -> Result<()> {
        let mut doc = self.document.clone();
        let i = doc.locate(self.selection.0)?.0;
        doc.paragraphs[i]
            .structure
            .list
            .as_mut()
            .ok_or("Select a list paragraph.")?
            .restart = Some(start);
        self.commit_structure(doc, "Restart numbering")
    }
    pub fn set_link(&mut self, target: Option<&str>) -> Result<()> {
        let (a, b) = self.selection;
        validate_range(&self.document.text(), a, b)?;
        let (first, left) = self.document.locate(a)?;
        let (last, right) = self.document.locate(b)?;
        if first != last
            || left == right
            || self.document.paragraphs[first].structure.block.is_some()
        {
            return Err("Select text within one paragraph for a link.".into());
        }
        if let Some(target) = target {
            validate_target(target)?;
        }
        let mut doc = self.document.clone();
        let links = &mut doc.paragraphs[first].structure.links;
        let mut next = Vec::new();
        for link in links.drain(..) {
            if link.end <= left || link.start >= right {
                next.push(link);
            } else {
                if link.start < left {
                    next.push(Link {
                        end: left,
                        ..link.clone()
                    });
                }
                if link.end > right {
                    next.push(Link {
                        start: right,
                        ..link
                    });
                }
            }
        }
        if let Some(target) = target {
            next.push(Link {
                start: left,
                end: right,
                target: target.into(),
            });
        }
        next.sort_by_key(|l| l.start);
        *links = next;
        self.commit_structure(doc, "Hyperlink")
    }
    pub fn set_page(&mut self, settings: Section, new_section: bool) -> Result<()> {
        settings.validate()?;
        let mut doc = self.document.clone();
        if new_section {
            let i = doc.locate(self.selection.0)?.0;
            doc.paragraphs[i].structure.section = Some(settings);
        } else {
            doc.page = settings;
        }
        self.commit_structure(
            doc,
            if new_section {
                "Section settings"
            } else {
                "Page setup"
            },
        )
    }
    pub fn set_flow(&mut self, page_break: bool, keep_next: bool, keep_lines: bool) -> Result<()> {
        let mut doc = self.document.clone();
        for i in paragraph_indices(&doc, self.selection)? {
            let p = &mut doc.paragraphs[i].structure;
            p.page_break_before = page_break;
            p.keep_with_next = keep_next;
            p.keep_lines = keep_lines;
        }
        self.commit_structure(doc, "Paragraph pagination")
    }
    pub fn remove_section(&mut self) -> Result<()> {
        let mut doc = self.document.clone();
        let i = doc.locate(self.selection.0)?.0;
        doc.paragraphs[i].structure.section = None;
        self.commit_structure(doc, "Remove section break")
    }
    pub fn insert_block(&mut self, block: Block) -> Result<()> {
        if let Block::Picture(p) = &block {
            p.validate_pixels()?;
        }
        let mut doc = self.document.clone();
        let i = doc.locate(self.selection.0)?.0;
        let p = Paragraph {
            text: OBJECT.into(),
            styles: vec![CharacterFormat::default()],
            structure: ParagraphStructure {
                block: Some(block),
                ..Default::default()
            },
            ..Default::default()
        };
        // Objects are inserted as independent blocks after the current paragraph;
        // the selection is never silently overwritten.
        doc.paragraphs.insert(i + 1, p);
        doc.paragraphs.insert(i + 2, Paragraph::default());
        let offset = doc.paragraphs[..i + 1]
            .iter()
            .map(|p| p.styles.len() + 1)
            .sum::<usize>();
        doc.validate()?;
        self.commit(
            doc,
            "Insert object",
            (offset, offset + 1),
            self.typing_style,
        );
        Ok(())
    }
    pub fn update_block(&mut self, paragraph: usize, block: Block) -> Result<()> {
        if let Block::Picture(p) = &block {
            p.validate_pixels()?;
        }
        let mut doc = self.document.clone();
        let p = doc
            .paragraphs
            .get_mut(paragraph)
            .ok_or("Object no longer exists.")?;
        if p.structure.block.is_none() {
            return Err("Select a table or picture.".into());
        }
        p.structure.block = Some(block);
        self.commit_structure(doc, "Edit object")
    }
    pub fn commit_structure(&mut self, document: Document, label: &str) -> Result<()> {
        document.validate()?;
        self.commit(document, label, self.selection, self.typing_style);
        Ok(())
    }
}
