//! Validated batch replacement prepared without mutating the live editor.
use super::*;

#[derive(Debug)]
pub struct ReplacementPlan {
    source: Document,
    result: Document,
    matches: usize,
    changes: usize,
    caret: usize,
    typing: CharacterFormat,
}

impl ReplacementPlan {
    pub fn match_count(&self) -> usize {
        self.matches
    }
    pub fn change_count(&self) -> usize {
        self.changes
    }
    pub fn result(&self) -> &Document {
        &self.result
    }
}

#[derive(Clone)]
struct Unit {
    ch: char,
    style: CharacterFormat,
    next_alignment: Option<Alignment>,
    next_style: Option<ParagraphStyle>,
    next_spacing: Option<ParagraphSpacing>,
}

fn append(paragraphs: &mut Vec<Paragraph>, unit: Unit) {
    if unit.ch == '\n' {
        paragraphs.push(Paragraph {
            paragraph_style: unit
                .next_style
                .unwrap_or(paragraphs.last().unwrap().paragraph_style),
            spacing: unit
                .next_spacing
                .unwrap_or_else(|| paragraphs.last().unwrap().spacing.clone()),
            alignment: unit
                .next_alignment
                .unwrap_or(paragraphs.last().unwrap().alignment),
            ..Paragraph::default()
        });
    } else {
        let paragraph = paragraphs.last_mut().unwrap();
        paragraph.text.push(unit.ch);
        paragraph.styles.push(unit.style);
    }
}

impl Editor {
    /// Ranges are ordered, disjoint, nonempty and refer to the original document.
    /// Equal-text replacements preserve existing mixed formatting as true no-ops.
    pub fn preview_replace_all(
        &self,
        ranges: &[(usize, usize)],
        replacement: &str,
    ) -> Result<ReplacementPlan> {
        self.document.validate()?;
        validate_text(replacement)?;
        let text = self.document.text();
        let edges = boundaries(&text);
        let inserted: Vec<_> = replacement.chars().collect();
        let mut length = text.chars().count();
        let mut previous = 0;
        for &(start, end) in ranges {
            if start < previous
                || start >= end
                || edges.binary_search(&start).is_err()
                || edges.binary_search(&end).is_err()
            {
                return Err(
                    "Replacement ranges must be ordered, disjoint and contain complete characters."
                        .into(),
                );
            }
            length = length
                .checked_sub(end - start)
                .and_then(|n| n.checked_add(inserted.len()))
                .ok_or("Replacement size exceeds the document limit.")?;
            previous = end;
        }
        if length > MAX_CHARACTERS {
            return Err("Replacement would exceed the 100,000 character document limit.".into());
        }
        let mut units = Vec::with_capacity(text.chars().count());
        for (index, paragraph) in self.document.paragraphs.iter().enumerate() {
            units.extend(
                paragraph
                    .text
                    .chars()
                    .zip(&paragraph.styles)
                    .map(|(ch, &style)| Unit {
                        ch,
                        style,
                        next_alignment: None,
                        next_style: None,
                        next_spacing: None,
                    }),
            );
            if let Some(next) = self.document.paragraphs.get(index + 1) {
                units.push(Unit {
                    ch: '\n',
                    style: paragraph.styles.last().copied().unwrap_or_default(),
                    next_alignment: Some(next.alignment),
                    next_style: Some(next.paragraph_style),
                    next_spacing: Some(next.spacing.clone()),
                });
            }
        }
        let mut paragraphs = vec![Paragraph {
            alignment: self.document.paragraphs[0].alignment,
            spacing: self.document.paragraphs[0].spacing.clone(),
            paragraph_style: self.document.paragraphs[0].paragraph_style,
            ..Paragraph::default()
        }];
        let (mut offset, mut output_length, mut changes, mut caret) = (0, 0, 0, 0);
        let mut typing = self.typing_style;
        for &(start, end) in ranges {
            for unit in &units[offset..start] {
                append(&mut paragraphs, unit.clone());
            }
            output_length += start - offset;
            if units[start..end]
                .iter()
                .map(|u| u.ch)
                .eq(inserted.iter().copied())
            {
                for unit in &units[start..end] {
                    append(&mut paragraphs, unit.clone());
                }
                output_length += end - start;
            } else {
                typing = units[start].style;
                for &ch in &inserted {
                    append(
                        &mut paragraphs,
                        Unit {
                            ch,
                            style: typing,
                            next_alignment: None,
                            next_style: None,
                            next_spacing: None,
                        },
                    );
                }
                output_length += inserted.len();
                caret = output_length;
                changes += 1;
            }
            offset = end;
        }
        for unit in &units[offset..] {
            append(&mut paragraphs, unit.clone());
        }
        let result = Document {
            paragraphs,
            style_catalog: self.document.style_catalog.clone(),
            character_styles: self.document.character_styles.clone(),
        };
        result.validate()?;
        // Inserted marks/joiners can combine with neighboring source text.
        caret = boundaries(&result.text())
            .into_iter()
            .find(|&p| p >= caret)
            .unwrap_or(caret);
        Ok(ReplacementPlan {
            source: self.document.clone(),
            result,
            matches: ranges.len(),
            changes,
            caret,
            typing,
        })
    }

    /// Apply exactly the previewed snapshot, or reject a stale plan without edits.
    pub fn apply_replace_all(&mut self, plan: ReplacementPlan) -> Result<usize> {
        if self.document != plan.source {
            return Err(
                "The document changed. Close this preview and prepare Replace All again.".into(),
            );
        }
        if plan.changes != 0 {
            self.commit(
                plan.result,
                "Replace all",
                (plan.caret, plan.caret),
                plan.typing,
            );
        }
        Ok(plan.changes)
    }
}
