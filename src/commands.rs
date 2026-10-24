//! Shared validated commands: desktop input and tests never implement separate edits.
use crate::{Result, model::*};

#[path = "replace_all.rs"]
mod replace_all;
pub use replace_all::ReplacementPlan;

/// A partial character-format change; unspecified properties retain their override.
pub type FontPatch = CharacterFormat;

#[derive(Clone, Debug)]
pub struct Operation {
    pub label: String,
    before: Document,
    after: Document,
    before_selection: (usize, usize),
    after_selection: (usize, usize),
    before_typing_style: CharacterFormat,
    after_typing_style: CharacterFormat,
}

#[derive(Default)]
pub struct Editor {
    pub document: Document,
    pub selection: (usize, usize),
    pub typing_style: CharacterFormat,
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
        typing: CharacterFormat,
    ) {
        self.commit_inner(document, label, selection, typing, false);
    }

    fn commit_inner(
        &mut self,
        document: Document,
        label: &str,
        selection: (usize, usize),
        typing: CharacterFormat,
        pending: bool,
    ) {
        if document != self.document || (pending && typing != self.typing_style) {
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
        self.replace_inner(start, end, text, label, false)
    }

    /// Insert a paragraph break and apply the source style's explicit successor.
    pub fn insert_paragraph_break(&mut self, start: usize, end: usize) -> Result<()> {
        self.replace_inner(start, end, "\n", "Paragraph break", true)
    }

    /// A widget keyboard batch may contain text and more than one Enter.
    pub fn replace_keyboard_text(&mut self, start: usize, end: usize, text: &str) -> Result<()> {
        self.replace_inner(start, end, text, "Paragraph break", true)
    }

    fn replace_inner(
        &mut self,
        start: usize,
        end: usize,
        text: &str,
        label: &str,
        following: bool,
    ) -> Result<()> {
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
        let styles: Vec<CharacterFormat> = a.styles[..left]
            .iter()
            .copied()
            .chain(std::iter::repeat_n(self.typing_style, text.chars().count()))
            .chain(b.styles[right..].iter().copied())
            .collect();
        let mut paragraphs = self.document.paragraphs[..first].to_vec();
        let mut offset = 0;
        let mut paragraph_style = a.paragraph_style;
        for (index, part) in joined.split('\n').enumerate() {
            if following && index > 0 {
                paragraph_style = self
                    .document
                    .style_catalog
                    .get(paragraph_style)
                    .and_then(|d| d.next_style)
                    .unwrap_or(paragraph_style);
            }
            let len = part.chars().count();
            paragraphs.push(Paragraph {
                text: part.into(),
                styles: styles[offset..offset + len].to_vec(),
                alignment: a.alignment,
                spacing: a.spacing.clone(),
                paragraph_style,
            });
            offset += len + 1;
        }
        paragraphs.extend_from_slice(&self.document.paragraphs[last + 1..]);
        let result = Document {
            paragraphs,
            style_catalog: self.document.style_catalog.clone(),
            character_styles: self.document.character_styles.clone(),
        };
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
        self.format_range(
            start,
            end,
            if bold { "Bold" } else { "Italic" },
            |style, _base| {
                if bold {
                    style.bold = Some(enabled);
                } else {
                    style.italic = Some(enabled);
                }
            },
        )
    }

    pub fn underline(&mut self, start: usize, end: usize, enabled: bool) -> Result<()> {
        self.format_range(start, end, "Underline", |style, _base| {
            style.underline = Some(enabled)
        })
    }

    pub fn set_font_size(&mut self, start: usize, end: usize, size: FontSize) -> Result<()> {
        self.format_range(start, end, "Font size", |style, _base| {
            style.font_size = Some(size)
        })
    }

    /// Reset supported direct character properties, preserving paragraph formatting.
    /// At a caret, reset only the pending insertion style, like other font commands.
    pub fn clear_character_formatting(&mut self, start: usize, end: usize) -> Result<()> {
        self.format_range(start, end, "Clear character formatting", |style, _base| {
            *style = CharacterFormat {
                character_style: style.character_style,
                ..Default::default()
            };
        })
    }

    /// Create and optionally apply a document-owned definition in one transaction.
    pub fn create_character_style(
        &mut self,
        name: &str,
        format: CharacterFormat,
        apply: bool,
    ) -> Result<CharacterStyle> {
        self.create_character_style_with_parent(name, format, None, apply)
    }

    pub fn create_character_style_with_parent(
        &mut self,
        name: &str,
        format: CharacterFormat,
        parent: Option<CharacterStyle>,
        apply: bool,
    ) -> Result<CharacterStyle> {
        let (start, end) = self.selection;
        validate_range(&self.document.text(), start, end)?;
        let mut candidate = Editor::new(self.document.clone());
        candidate.selection = self.selection;
        candidate.typing_style = self.typing_style;
        let id = candidate.document.add_character_style(name, format)?;
        candidate.document.set_character_parent(id, parent)?;
        if apply {
            candidate.apply_character_style(start, end, Some(id))?;
        }
        self.commit(
            candidate.document,
            "Create character style",
            self.selection,
            candidate.typing_style,
        );
        Ok(id)
    }

    /// Modify a definition in one history transaction without rewriting runs.
    pub fn modify_character_style(
        &mut self,
        id: CharacterStyle,
        format: CharacterFormat,
    ) -> Result<()> {
        let mut document = self.document.clone();
        document.modify_character_style(id, format)?;
        self.commit(
            document,
            "Modify character style",
            self.selection,
            self.typing_style,
        );
        Ok(())
    }

    /// Rename and edit a definition as one operation, retaining its stable ID.
    pub fn update_character_style(
        &mut self,
        id: CharacterStyle,
        name: &str,
        format: CharacterFormat,
    ) -> Result<()> {
        let mut document = self.document.clone();
        document.update_character_style(id, name, format)?;
        self.commit(
            document,
            "Modify character style",
            self.selection,
            self.typing_style,
        );
        Ok(())
    }

    /// Commit name, parent and properties as one validated history transaction.
    pub fn update_character_style_with_parent(
        &mut self,
        id: CharacterStyle,
        name: &str,
        format: CharacterFormat,
        parent: Option<CharacterStyle>,
    ) -> Result<()> {
        let mut document = self.document.clone();
        document.update_character_style(id, name, format)?;
        document.set_character_parent(id, parent)?;
        self.commit(
            document,
            "Modify character style",
            self.selection,
            self.typing_style,
        );
        Ok(())
    }

    /// Delete and remap document/pending references together in one undo step.
    pub fn delete_character_style(
        &mut self,
        id: CharacterStyle,
        replacement: Option<CharacterStyle>,
    ) -> Result<()> {
        let mut document = self.document.clone();
        document.delete_character_style(id, replacement)?;
        let mut typing = self.typing_style;
        if typing.character_style == Some(id) {
            typing.character_style = replacement;
        }
        self.commit(document, "Delete character style", self.selection, typing);
        Ok(())
    }

    /// Apply/remove a semantic character style, retaining all direct overrides.
    pub fn apply_character_style(
        &mut self,
        start: usize,
        end: usize,
        style: Option<CharacterStyle>,
    ) -> Result<()> {
        if style.is_some_and(|id| !self.document.character_style_exists(id)) {
            return Err("Unknown character style reference.".into());
        }
        validate_range(&self.document.text(), start, end)?;
        if start == end {
            let typing = CharacterFormat {
                character_style: style,
                ..self.typing_style
            };
            self.commit_inner(
                self.document.clone(),
                "Character style",
                (start, end),
                typing,
                true,
            );
            return Ok(());
        }
        self.format_range(start, end, "Character style", |format, _| {
            format.character_style = style
        })
    }

    /// None means mixed/separators; Some(None) means no named character style.
    pub fn selected_character_style(&self) -> Option<Option<CharacterStyle>> {
        let (start, end) = self.selection;
        if start == end {
            return Some(self.typing_style.character_style);
        }
        let mut offset = 0;
        let mut found = None;
        for p in &self.document.paragraphs {
            for (i, format) in p.styles.iter().enumerate() {
                if (start..end).contains(&(offset + i)) {
                    let value = format.character_style;
                    if found.is_some_and(|prior| prior != value) {
                        return None;
                    }
                    found = Some(value);
                }
            }
            offset += p.styles.len() + 1;
        }
        found
    }

    /// Apply a dialog's changes together, with one undo boundary and no live edits.
    pub fn set_font(&mut self, start: usize, end: usize, patch: FontPatch) -> Result<()> {
        validate_range(&self.document.text(), start, end)?;
        if patch == FontPatch::default() {
            return Ok(());
        }
        self.format_range(start, end, "Font", |style, _base| {
            if let Some(value) = patch.bold {
                style.bold = Some(value);
            }
            if let Some(value) = patch.italic {
                style.italic = Some(value);
            }
            if let Some(value) = patch.underline {
                style.underline = Some(value);
            }
            if let Some(value) = patch.font_size {
                style.font_size = Some(value);
            }
        })
    }

    /// Uniform values for the selection; None means mixed or only separators.
    pub fn selected_font(&self) -> FontPatch {
        FontPatch {
            character_style: None,
            bold: self.selected_value(|s| s.bold),
            italic: self.selected_value(|s| s.italic),
            underline: self.selected_value(|s| s.underline),
            font_size: self.selected_font_size(),
        }
    }

    pub fn step_font_size(&mut self, start: usize, end: usize, grow: bool) -> Result<()> {
        let catalog = self.document.character_styles.clone();
        self.format_range(
            start,
            end,
            if grow { "Grow font" } else { "Shrink font" },
            |style, _base| {
                style.font_size = Some(style.resolve_with(_base, &catalog).font_size.stepped(grow));
            },
        )
    }

    fn format_range(
        &mut self,
        start: usize,
        end: usize,
        label: &str,
        change: impl Fn(&mut CharacterFormat, Style),
    ) -> Result<()> {
        validate_range(&self.document.text(), start, end)?;
        if start == end {
            let base = self
                .document
                .style_catalog
                .resolve(self.document.paragraphs[self.document.locate(start)?.0].paragraph_style)
                .character;
            change(&mut self.typing_style, base);
            return Ok(());
        }
        let mut result = self.document.clone();
        let mut offset = 0;
        let mut typing = None;
        for p in &mut result.paragraphs {
            let base = self
                .document
                .style_catalog
                .resolve(p.paragraph_style)
                .character;
            for (i, style) in p.styles.iter_mut().enumerate() {
                if (start..end).contains(&(offset + i)) {
                    change(style, base);
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

    /// None leaves the property unchanged; Some(None) restores inheritance.
    pub fn set_paragraph_spacing(
        &mut self,
        start: usize,
        end: usize,
        line: Option<Option<u16>>,
        after: Option<Option<u16>>,
    ) -> Result<()> {
        self.set_paragraph_spacing_with_before(start, end, line, None, after)
    }

    /// Set any subset of paragraph spacing as one validated history operation.
    pub fn set_paragraph_spacing_with_before(
        &mut self,
        start: usize,
        end: usize,
        line: Option<Option<u16>>,
        before: Option<Option<u16>>,
        after: Option<Option<u16>>,
    ) -> Result<()> {
        self.set_paragraph_geometry(
            start,
            end,
            ParagraphGeometryPatch {
                line,
                before,
                after,
                ..Default::default()
            },
        )
    }

    /// One atomic partial change for spacing and left/right indentation.
    pub fn set_paragraph_geometry(
        &mut self,
        start: usize,
        end: usize,
        patch: ParagraphGeometryPatch,
    ) -> Result<()> {
        let ParagraphGeometryPatch {
            line,
            before,
            after,
            left,
            right,
            first_line,
        } = patch;
        validate_range(&self.document.text(), start, end)?;
        let (first, _) = self.document.locate(start)?;
        let (mut last, offset) = self.document.locate(end)?;
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        let mut result = self.document.clone();
        for p in &mut result.paragraphs[first..=last] {
            if let Some(value) = first_line {
                p.spacing.first_line_half_points = value;
            }
            if let Some(value) = left {
                p.spacing.left_indent_half_points = value;
            }
            if let Some(value) = right {
                p.spacing.right_indent_half_points = value;
            }
            if let Some(value) = line {
                p.spacing.line_height_percent = value;
            }
            if let Some(value) = before {
                p.spacing.space_before_half_points = value;
            }
            if let Some(value) = after {
                p.spacing.space_after_half_points = value;
            }
        }
        result.validate()?;
        self.commit(result, "Paragraph spacing", (start, end), self.typing_style);
        Ok(())
    }

    /// Change each selected paragraph's resolved left indent by 36 pt.
    /// Clamped paragraphs retain their authored override (including inheritance).
    pub fn step_paragraph_indent(
        &mut self,
        start: usize,
        end: usize,
        increase: bool,
    ) -> Result<()> {
        validate_range(&self.document.text(), start, end)?;
        let (first, _) = self.document.locate(start)?;
        let (mut last, offset) = self.document.locate(end)?;
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        let mut result = self.document.clone();
        for index in first..=last {
            let current = self
                .document
                .resolved_paragraph(&self.document.paragraphs[index])
                .left_indent_half_points;
            let next = if increase {
                current.saturating_add(72).min(288)
            } else {
                current.saturating_sub(72)
            };
            if next != current {
                result.paragraphs[index].spacing.left_indent_half_points = Some(next);
            }
        }
        result.validate()?;
        self.commit(
            result,
            if increase {
                "Increase indent"
            } else {
                "Decrease indent"
            },
            (start, end),
            self.typing_style,
        );
        Ok(())
    }

    /// Set/clear a leader on one explicit position in every selected paragraph.
    /// Missing positions reject the complete transaction rather than creating stops.
    pub fn set_tab_leader(
        &mut self,
        start: usize,
        end: usize,
        position: u16,
        leader: Option<crate::model::TabLeader>,
    ) -> Result<()> {
        self.change_tab_leader(start, end, position, leader, false)
    }
    /// Change only selected paragraphs owning this stop, preserving nonowners.
    pub fn set_owned_tab_leader(
        &mut self,
        start: usize,
        end: usize,
        position: u16,
        leader: Option<crate::model::TabLeader>,
    ) -> Result<()> {
        self.change_tab_leader(start, end, position, leader, true)
    }
    fn change_tab_leader(
        &mut self,
        start: usize,
        end: usize,
        position: u16,
        leader: Option<crate::model::TabLeader>,
        owners_only: bool,
    ) -> Result<()> {
        validate_range(&self.document.text(), start, end)?;
        let (first, _) = self.document.locate(start)?;
        let (mut last, offset) = self.document.locate(end)?;
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        let mut result = self.document.clone();
        for p in &mut result.paragraphs[first..=last] {
            let original = p.spacing.clone();
            let effective = self.document.resolved_tabs(p);
            p.spacing.copy_tab_stops(&effective);
            if !p.spacing.tab_positions().iter().any(|s| s.0 == position) {
                if owners_only {
                    p.spacing = original;
                    continue;
                }
                return Err("The selected paragraph has no explicit tab at this position.".into());
            }
            p.spacing
                .tab_leaders
                .retain(|s| s.position_half_points != position);
            if let Some(leader) = leader {
                p.spacing.tab_leaders.push(crate::model::TabLeaderStop {
                    position_half_points: position,
                    leader,
                });
                p.spacing
                    .tab_leaders
                    .sort_by_key(|s| s.position_half_points);
            }
            if p.spacing.style_tab_stops() == effective.style_tab_stops() {
                p.spacing = original;
            }
        }
        result.validate()?;
        self.commit(result, "Tab leader", (start, end), self.typing_style);
        Ok(())
    }

    /// Replace center/right stops, retaining left stops and other paragraph properties.
    pub fn set_aligned_tab_stops(
        &mut self,
        start: usize,
        end: usize,
        stops: Vec<crate::model::AlignedTabStop>,
    ) -> Result<()> {
        validate_range(&self.document.text(), start, end)?;
        let (first, _) = self.document.locate(start)?;
        let (mut last, offset) = self.document.locate(end)?;
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        let mut result = self.document.clone();
        for paragraph in &mut result.paragraphs[first..=last] {
            let effective = self.document.resolved_tabs(paragraph);
            let mut next = effective.clone();
            next.aligned_tab_stops = stops.clone();
            next.retain_tab_leaders();
            next.validate()?;
            if next.style_tab_stops() != effective.style_tab_stops() {
                paragraph.spacing.copy_tab_stops(&next);
            }
        }
        result.validate()?;
        self.commit(result, "Aligned tab stops", (start, end), self.typing_style);
        Ok(())
    }

    /// Replace explicit left stops on selected paragraphs as one history entry.
    pub fn set_left_tab_stops(
        &mut self,
        start: usize,
        end: usize,
        positions: Vec<u16>,
    ) -> Result<()> {
        validate_range(&self.document.text(), start, end)?;
        ParagraphSpacing {
            left_tab_stops_half_points: positions.clone(),
            ..Default::default()
        }
        .validate()?;
        let (first, _) = self.document.locate(start)?;
        let (mut last, offset) = self.document.locate(end)?;
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        let mut result = self.document.clone();
        for p in &mut result.paragraphs[first..=last] {
            let original = p.spacing.clone();
            let effective = self.document.resolved_tabs(p);
            p.spacing.copy_tab_stops(&effective);
            p.spacing.left_tab_stops_half_points = positions.clone();
            p.spacing.retain_tab_leaders();
            p.spacing.clear_tab_stops = p.spacing.tab_positions().is_empty();
            if p.spacing.style_tab_stops() == effective.style_tab_stops() {
                p.spacing = original;
            }
        }
        result.validate()?;
        self.commit(result, "Left tab stops", (start, end), self.typing_style);
        Ok(())
    }

    /// Replace all explicit types through a single validated history operation.
    pub fn set_tab_positions(
        &mut self,
        start: usize,
        end: usize,
        stops: Vec<(u16, crate::model::TabKind)>,
    ) -> Result<()> {
        validate_range(&self.document.text(), start, end)?;
        let (first, _) = self.document.locate(start)?;
        let (mut last, offset) = self.document.locate(end)?;
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        self.set_paragraph_tab_lists(start, end, vec![stops; last - first + 1])
    }

    /// Explicit whole-list replacement including leaders, committed once.
    pub fn set_tab_settings(
        &mut self,
        start: usize,
        end: usize,
        positions: Vec<(u16, crate::model::TabKind)>,
        leaders: Vec<crate::model::TabLeaderStop>,
    ) -> Result<()> {
        validate_range(&self.document.text(), start, end)?;
        let (first, _) = self.document.locate(start)?;
        let (mut last, offset) = self.document.locate(end)?;
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        let count = last - first + 1;
        self.set_paragraph_tab_settings(
            start,
            end,
            vec![positions; count],
            Some(vec![leaders; count]),
        )
    }

    /// Commit distinct selected-paragraph lists together without flattening geometry.
    pub(crate) fn set_paragraph_tab_lists(
        &mut self,
        start: usize,
        end: usize,
        lists: Vec<Vec<(u16, crate::model::TabKind)>>,
    ) -> Result<()> {
        self.set_paragraph_tab_settings(start, end, lists, None)
    }
    pub(crate) fn set_paragraph_tab_settings(
        &mut self,
        start: usize,
        end: usize,
        lists: Vec<Vec<(u16, crate::model::TabKind)>>,
        leaders: Option<Vec<Vec<crate::model::TabLeaderStop>>>,
    ) -> Result<()> {
        validate_range(&self.document.text(), start, end)?;
        let (first, _) = self.document.locate(start)?;
        let (mut last, offset) = self.document.locate(end)?;
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        if lists.len() != last - first + 1 {
            return Err("Tab list selection mismatch.".into());
        }
        let mut result = self.document.clone();
        if leaders
            .as_ref()
            .is_some_and(|values| values.len() != lists.len())
        {
            return Err("Tab leader selection mismatch.".into());
        }
        for (index, (paragraph, positions)) in result.paragraphs[first..=last]
            .iter_mut()
            .zip(lists)
            .enumerate()
        {
            let effective = self.document.resolved_tabs(paragraph);
            let mut next = effective.clone();
            next.set_tab_positions(&positions)?;
            if let Some(leaders) = &leaders {
                next.tab_leaders = leaders[index].clone();
            }
            next.validate()?;
            if next.style_tab_stops() != effective.style_tab_stops() {
                paragraph.spacing.copy_tab_stops(&next);
            }
        }
        result.validate()?;
        self.commit(result, "Tab positions", (start, end), self.typing_style);
        Ok(())
    }

    /// Remove direct stop lists while retaining the independently authored interval.
    pub fn inherit_tab_stops(&mut self, start: usize, end: usize) -> Result<()> {
        validate_range(&self.document.text(), start, end)?;
        let (first, _) = self.document.locate(start)?;
        let (mut last, offset) = self.document.locate(end)?;
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        let mut result = self.document.clone();
        for p in &mut result.paragraphs[first..=last] {
            p.spacing.left_tab_stops_half_points.clear();
            p.spacing.aligned_tab_stops.clear();
            p.spacing.tab_leaders.clear();
            p.spacing.clear_tab_stops = false;
        }
        result.validate()?;
        self.commit(
            result,
            "Inherit tab positions",
            (start, end),
            self.typing_style,
        );
        Ok(())
    }

    /// Set a direct tab interval, or inherit the style grid, in one transaction.
    pub fn set_tab_interval(
        &mut self,
        start: usize,
        end: usize,
        interval: Option<u16>,
    ) -> Result<()> {
        validate_range(&self.document.text(), start, end)?;
        ParagraphSpacing {
            tab_interval_half_points: interval,
            ..Default::default()
        }
        .validate()?;
        let (first, _) = self.document.locate(start)?;
        let (mut last, offset) = self.document.locate(end)?;
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        let mut result = self.document.clone();
        for p in &mut result.paragraphs[first..=last] {
            p.spacing.tab_interval_half_points = interval;
        }
        result.validate()?;
        self.commit(result, "Tab interval", (start, end), self.typing_style);
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

    /// Apply to intersected paragraphs, preserving every direct character override.
    pub fn apply_paragraph_style(
        &mut self,
        start: usize,
        end: usize,
        style: ParagraphStyle,
    ) -> Result<()> {
        if self.document.style_catalog.get(style).is_none() {
            return Err("Unknown paragraph style.".into());
        }
        validate_range(&self.document.text(), start, end)?;
        let (first, _) = self.document.locate(start)?;
        let (mut last, offset) = self.document.locate(end)?;
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        let mut result = self.document.clone();
        for p in &mut result.paragraphs[first..=last] {
            p.paragraph_style = style;
        }
        self.selection = (start, end);
        self.commit(
            result,
            "Apply paragraph style",
            (start, end),
            self.typing_style,
        );
        Ok(())
    }

    /// Replace a definition and its parent together, without flattening descendants.
    pub fn modify_style(&mut self, id: ParagraphStyle, definition: StyleDefinition) -> Result<()> {
        let mut result = self.document.clone();
        *result
            .style_catalog
            .get_mut(id)
            .ok_or("Unknown paragraph style.")? = definition;
        result.validate()?;
        self.commit(result, "Modify style", self.selection, self.typing_style);
        Ok(())
    }

    /// Rename and optionally modify a custom style in one validated transaction.
    pub fn modify_custom_style(
        &mut self,
        id: ParagraphStyle,
        name: &str,
        definition: StyleDefinition,
    ) -> Result<()> {
        let mut result = self.document.clone();
        result
            .style_catalog
            .update_custom(id, name, definition.clone())?;
        result.validate()?;
        let label = if self.document.style_catalog.get(id) == Some(&definition) {
            "Rename style"
        } else {
            "Modify style"
        };
        self.commit(result, label, self.selection, self.typing_style);
        Ok(())
    }

    /// Remove a custom definition and replace its direct references in one undo step.
    pub fn delete_custom_style(
        &mut self,
        id: ParagraphStyle,
        paragraph_replacement: ParagraphStyle,
        parent_replacement: Option<ParagraphStyle>,
    ) -> Result<()> {
        let catalog = self.document.style_catalog.without_custom_style(
            id,
            paragraph_replacement,
            parent_replacement,
        )?;
        let mut result = self.document.clone();
        result.style_catalog = catalog;
        for paragraph in &mut result.paragraphs {
            if paragraph.paragraph_style == id {
                paragraph.paragraph_style = paragraph_replacement;
            }
        }
        result.validate()?;
        self.commit(result, "Delete style", self.selection, self.typing_style);
        Ok(())
    }

    /// Create and optionally apply a named definition in one history transaction.
    pub fn create_style(
        &mut self,
        name: &str,
        definition: StyleDefinition,
        apply: bool,
    ) -> Result<ParagraphStyle> {
        let (start, end) = self.selection;
        validate_range(&self.document.text(), start, end)?;
        let mut result = self.document.clone();
        let id = result.style_catalog.add(name, definition)?;
        if apply {
            let (first, _) = result.locate(start)?;
            let (mut last, offset) = result.locate(end)?;
            if end > start && offset == 0 && last > first {
                last -= 1;
            }
            for p in &mut result.paragraphs[first..=last] {
                p.paragraph_style = id;
            }
        }
        result.validate()?;
        self.commit(result, "Create style", self.selection, self.typing_style);
        Ok(id)
    }

    pub fn selected_paragraph_style(&self) -> Option<ParagraphStyle> {
        let (start, end) = self.selection;
        let (first, _) = self.document.locate(start).ok()?;
        let (mut last, offset) = self.document.locate(end).ok()?;
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        let style = self.document.paragraphs[first].paragraph_style;
        self.document.paragraphs[first..=last]
            .iter()
            .all(|p| p.paragraph_style == style)
            .then_some(style)
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
            let p = self.document.locate(start).ok()?.0;
            return Some(value(
                self.typing_style.resolve_with(
                    self.document
                        .style_catalog
                        .resolve(self.document.paragraphs[p].paragraph_style)
                        .character,
                    &self.document.character_styles,
                ),
            ));
        }
        let mut offset = 0;
        let mut selected = Vec::new();
        for p in &self.document.paragraphs {
            let base = self
                .document
                .style_catalog
                .resolve(p.paragraph_style)
                .character;
            selected.extend(
                p.styles
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| (start..end).contains(&(offset + i)))
                    .map(|(_, s)| value(s.resolve_with(base, &self.document.character_styles))),
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

/// Outer None keeps a property; inner None restores style inheritance.
#[derive(Clone, Copy, Debug, Default)]
pub struct ParagraphGeometryPatch {
    pub line: Option<Option<u16>>,
    pub before: Option<Option<u16>>,
    pub after: Option<Option<u16>>,
    pub left: Option<Option<u16>>,
    pub right: Option<Option<u16>>,
    pub first_line: Option<Option<i16>>,
}
