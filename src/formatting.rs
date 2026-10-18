//! Read-only formatting provenance, independent of the GUI and history.
use crate::{
    commands::Editor,
    model::{CharacterFormat, CharacterStyle, Document, ParagraphStyle},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyInfo {
    pub name: &'static str,
    pub value: String,
    pub source: String,
}

fn values(f: CharacterFormat) -> [Option<String>; 4] {
    let boolean = |v: bool| if v { "On".into() } else { "Off".into() };
    [
        f.bold.map(boolean),
        f.italic.map(boolean),
        f.underline.map(boolean),
        f.font_size.map(|s| format!("{s} pt")),
    ]
}

/// Follow authored definitions, not equal-looking resolved values.
pub fn inspect(
    document: &Document,
    paragraph: ParagraphStyle,
    direct: CharacterFormat,
) -> Vec<PropertyInfo> {
    let base = document.style_catalog.resolve(paragraph).character;
    let resolved = direct.resolve_with(base, &document.character_styles);
    let mut rows: Vec<_> = ["Bold", "Italic", "Underline", "Font size"]
        .into_iter()
        .zip(values(CharacterFormat::explicit(resolved)))
        .map(|(name, value)| PropertyInfo {
            name,
            value: value.unwrap(),
            source: "Document default".into(),
        })
        .collect();
    let mut found = [false; 4];
    let mut assign = |format, source: String| {
        for (i, value) in values(format).into_iter().enumerate() {
            if value.is_some() && !found[i] {
                rows[i].source = source.clone();
                found[i] = true;
            }
        }
    };
    assign(direct, "Direct formatting".into());
    let mut next = direct.character_style;
    let mut seen = Vec::new();
    while let Some(id) = next {
        if seen.contains(&id) {
            break;
        }
        seen.push(id);
        let relation = if Some(id) == direct.character_style {
            "Character style"
        } else {
            "Inherited character style"
        };
        let format = match id {
            CharacterStyle::Strong => CharacterFormat {
                bold: Some(true),
                ..Default::default()
            },
            CharacterStyle::Emphasis => CharacterFormat {
                italic: Some(true),
                ..Default::default()
            },
            CharacterStyle::Custom(key) => document
                .character_styles
                .iter()
                .find(|s| s.id == key)
                .map(|s| s.format)
                .unwrap_or_default(),
        };
        assign(
            format,
            format!("{relation}: {}", document.character_style_name(id)),
        );
        next = document
            .character_styles
            .iter()
            .find(|s| id == CharacterStyle::Custom(s.id))
            .and_then(|s| s.parent);
    }
    let mut next = Some(paragraph);
    let mut seen = Vec::new();
    while let Some(id) = next {
        if seen.contains(&id) {
            break;
        }
        seen.push(id);
        let Some(definition) = document.style_catalog.get(id) else {
            break;
        };
        let relation = if id == paragraph {
            "Paragraph style"
        } else {
            "Inherited paragraph style"
        };
        assign(
            definition.character,
            format!("{relation}: {}", document.style_catalog.name(id)),
        );
        next = definition.parent;
    }
    rows
}

/// Empty selections describe future typing. Mixed values and sources are independent.
pub fn selection(editor: &Editor) -> Vec<PropertyInfo> {
    let (start, end) = editor.selection;
    if start == end {
        let p = editor.document.locate(start).map(|(p, _)| p).unwrap_or(0);
        return inspect(
            &editor.document,
            editor.document.paragraphs[p].paragraph_style,
            editor.typing_style,
        );
    }
    let mut result: Option<Vec<PropertyInfo>> = None;
    let mut offset = 0;
    let mut previous = None;
    for paragraph in &editor.document.paragraphs {
        // A selected paragraph separator has no character formatting of its own.
        for (i, format) in paragraph.styles.iter().enumerate() {
            if (start..end).contains(&(offset + i)) {
                let key = (paragraph.paragraph_style, *format);
                if previous == Some(key) {
                    continue;
                }
                previous = Some(key);
                let rows = inspect(&editor.document, paragraph.paragraph_style, *format);
                if let Some(result) = &mut result {
                    for (a, b) in result.iter_mut().zip(rows) {
                        if a.value != b.value {
                            a.value = "Mixed".into();
                        }
                        if a.source != b.source {
                            a.source = "Mixed sources".into();
                        }
                    }
                } else {
                    result = Some(rows);
                }
            }
        }
        offset += paragraph.styles.len() + 1;
    }
    result.unwrap_or_default()
}

/// Paragraph values use the same cascade as layout. Alignment is stored directly;
/// the current format has no inherited alignment or optional alignment override.
pub fn inspect_paragraph(
    document: &Document,
    paragraph: &crate::model::Paragraph,
) -> Vec<PropertyInfo> {
    use crate::model::Alignment;
    let resolved = document.resolved_paragraph(paragraph);
    let mut rows = vec![
        PropertyInfo {
            name: "Alignment",
            value: match paragraph.alignment {
                Alignment::Left => "Left",
                Alignment::Center => "Center",
                Alignment::Right => "Right",
                Alignment::Justify => "Justify",
            }
            .into(),
            source: "Paragraph value (not inherited)".into(),
        },
        PropertyInfo {
            name: "Line spacing",
            value: format!("{}%", resolved.line_height_percent),
            source: "Document default".into(),
        },
        PropertyInfo {
            name: "Space after",
            value: format!("{} pt", resolved.space_after_points()),
            source: "Document default".into(),
        },
    ];
    let mut found = [
        paragraph.spacing.line_height_percent.is_some(),
        paragraph.spacing.space_after_half_points.is_some(),
    ];
    for (i, direct) in found.iter().enumerate() {
        if *direct {
            rows[i + 1].source = "Direct paragraph formatting".into();
        }
    }
    let mut next = Some(paragraph.paragraph_style);
    let mut seen = Vec::new();
    while let Some(id) = next {
        if seen.contains(&id) {
            break;
        }
        seen.push(id);
        let Some(definition) = document.style_catalog.get(id) else {
            break;
        };
        let relation = if id == paragraph.paragraph_style {
            "Paragraph style"
        } else {
            "Inherited paragraph style"
        };
        for (i, supplied) in [
            definition.line_height_percent.is_some(),
            definition.space_after_half_points.is_some(),
        ]
        .into_iter()
        .enumerate()
        {
            if supplied && !found[i] {
                rows[i + 1].source = format!("{relation}: {}", document.style_catalog.name(id));
                found[i] = true;
            }
        }
        next = definition.parent;
    }
    rows
}

/// Match alignment/style commands: exclude a paragraph at the exclusive end,
/// but include the caret's paragraph and selected empty paragraphs/separators.
pub fn paragraph_selection(editor: &Editor) -> Vec<PropertyInfo> {
    let (start, end) = editor.selection;
    if start > end {
        return Vec::new();
    }
    let Ok((first, _)) = editor.document.locate(start) else {
        return Vec::new();
    };
    let Ok((mut last, offset)) = editor.document.locate(end) else {
        return Vec::new();
    };
    if end > start && offset == 0 && last > first {
        last -= 1;
    }
    let mut result = inspect_paragraph(&editor.document, &editor.document.paragraphs[first]);
    for paragraph in &editor.document.paragraphs[first + 1..=last] {
        for (a, b) in result
            .iter_mut()
            .zip(inspect_paragraph(&editor.document, paragraph))
        {
            if a.value != b.value {
                a.value = "Mixed".into();
            }
            if a.source != b.source {
                a.source = "Mixed sources".into();
            }
        }
    }
    result
}
