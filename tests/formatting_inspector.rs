use breakfree_docs::{commands::Editor, formatting, model::*};
#[test]
fn authored_sources_survive_equal_values_and_history() {
    let mut doc = Document::default();
    let parent = doc
        .add_character_style(
            "Parent",
            CharacterFormat {
                bold: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    let child = doc
        .add_character_style("Child", CharacterFormat::default())
        .unwrap();
    doc.set_character_parent(child, Some(parent)).unwrap();
    let direct = CharacterFormat {
        character_style: Some(child),
        ..Default::default()
    };
    let rows = formatting::inspect(&doc, ParagraphStyle::Heading1, direct);
    assert_eq!(rows[0].source, "Inherited character style: Parent");
    assert_eq!(rows[3].source, "Paragraph style: Heading 1");
    let rows = formatting::inspect(
        &doc,
        ParagraphStyle::Heading1,
        CharacterFormat {
            bold: Some(false),
            ..direct
        },
    );
    assert_eq!(
        (rows[0].value.as_str(), rows[0].source.as_str()),
        ("Off", "Direct formatting")
    );
}
#[test]
fn mixed_sources_caret_and_separators_are_read_only() {
    let mut doc = Document::default();
    doc.paragraphs[0].text = "aé".into();
    doc.paragraphs[0].styles = vec![
        CharacterFormat {
            bold: Some(false),
            ..Default::default()
        },
        CharacterFormat::default(),
    ];
    let mut editor = Editor::new(doc);
    editor.selection = (0, 2);
    let before = editor.document.clone();
    let rows = formatting::selection(&editor);
    assert_eq!(rows[0].value, "Off");
    assert_eq!(rows[0].source, "Mixed sources");
    assert_eq!(editor.document, before);
    editor.selection = (2, 2);
    editor.typing_style.bold = Some(true);
    assert_eq!(formatting::selection(&editor)[0].value, "On");
    editor.document.paragraphs.push(Paragraph::default());
    editor.selection = (2, 3);
    assert!(formatting::selection(&editor).is_empty());
}

#[test]
fn paragraph_parent_and_changed_definition_track_undo_redo() {
    let mut doc = Document::default();
    doc.style_catalog.normal.character.italic = Some(true);
    let rows = formatting::inspect(&doc, ParagraphStyle::NoSpacing, CharacterFormat::default());
    assert_eq!(rows[1].source, "Inherited paragraph style: Normal");
    assert_eq!(rows[1].value, "On");
    let id = doc
        .add_character_style(
            "Note",
            CharacterFormat {
                italic: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    let mut editor = Editor::new(doc);
    editor.typing_style.character_style = Some(id);
    let before = formatting::selection(&editor);
    editor
        .update_character_style_with_parent(
            id,
            "Note",
            CharacterFormat::default(),
            Some(CharacterStyle::Emphasis),
        )
        .unwrap();
    let after = formatting::selection(&editor);
    assert_eq!(after[1].source, "Inherited character style: Emphasis");
    assert_eq!(after[1].value, "On");
    editor.undo();
    assert_eq!(formatting::selection(&editor), before);
    editor.redo();
    assert_eq!(formatting::selection(&editor), after);
}

#[test]
fn paragraph_intervals_report_authored_sources_and_zero() {
    let mut doc = Document::default();
    let rows = formatting::inspect_paragraph(&doc, &doc.paragraphs[0]);
    assert_eq!(
        (rows[1].value.as_str(), rows[1].source.as_str()),
        ("125%", "Document default")
    );
    assert_eq!(rows[2].value, "5.5 pt");
    doc.style_catalog.normal.line_height_percent = Some(125);
    doc.style_catalog.normal.space_after_half_points = Some(0);
    doc.paragraphs[0].paragraph_style = ParagraphStyle::Heading1;
    doc.paragraphs[0].alignment = Alignment::Right;
    let rows = formatting::inspect_paragraph(&doc, &doc.paragraphs[0]);
    assert_eq!(rows[0].value, "Right");
    assert_eq!(rows[0].source, "Paragraph value (not inherited)");
    assert_eq!(rows[1].source, "Inherited paragraph style: Normal");
    assert_eq!(rows[2].value, "0 pt");
    doc.paragraphs[0].paragraph_style = ParagraphStyle::NoSpacing;
    let rows = formatting::inspect_paragraph(&doc, &doc.paragraphs[0]);
    assert_eq!(rows[2].source, "Paragraph style: No Spacing");
}

#[test]
fn paragraph_selection_matches_half_open_editing_and_empty_paragraphs() {
    let mut doc = Document::default();
    doc.paragraphs[0].text = "é".into();
    doc.paragraphs[0].styles = vec![CharacterFormat::default()];
    doc.paragraphs.push(Paragraph {
        paragraph_style: ParagraphStyle::NoSpacing,
        alignment: Alignment::Center,
        ..Default::default()
    });
    doc.paragraphs.push(Paragraph {
        paragraph_style: ParagraphStyle::Heading1,
        ..Default::default()
    });
    let mut editor = Editor::new(doc);
    editor.selection = (0, 2); // Ends at the empty second paragraph's start.
    assert_eq!(formatting::paragraph_selection(&editor)[0].value, "Left");
    editor.selection = (2, 2);
    assert_eq!(formatting::paragraph_selection(&editor)[0].value, "Center");
    editor.selection = (2, 3); // Select only the empty paragraph separator.
    assert_eq!(formatting::paragraph_selection(&editor)[2].value, "0 pt");
    editor.selection = (0, 3);
    let rows = formatting::paragraph_selection(&editor);
    assert_eq!(rows[0].value, "Mixed");
    assert_eq!(rows[0].source, "Paragraph value (not inherited)");
    assert_eq!(rows[1].value, "Mixed");
    assert_eq!(rows[1].source, "Mixed sources");
    editor.document.style_catalog.normal.line_height_percent = Some(100);
    let rows = formatting::paragraph_selection(&editor);
    assert_eq!(rows[1].value, "100%");
    assert_eq!(rows[1].source, "Mixed sources");
}

#[test]
fn paragraph_inspection_tracks_history_and_roundtrip_without_mutation() {
    let mut editor = Editor::new(Document::default());
    let before = formatting::paragraph_selection(&editor);
    editor.align(0, 0, Alignment::Justify).unwrap();
    let aligned = formatting::paragraph_selection(&editor);
    assert_eq!(aligned[0].value, "Justify");
    editor.undo();
    assert_eq!(formatting::paragraph_selection(&editor), before);
    editor.redo();
    assert_eq!(formatting::paragraph_selection(&editor), aligned);
    editor
        .apply_paragraph_style(0, 0, ParagraphStyle::NoSpacing)
        .unwrap();
    let expected = formatting::paragraph_selection(&editor);
    let bytes = breakfree_docs::storage::encode(&editor.document).unwrap();
    let reopened = Editor::new(breakfree_docs::storage::decode(&bytes).unwrap());
    assert_eq!(formatting::paragraph_selection(&reopened), expected);
    assert_eq!(
        breakfree_docs::storage::encode(&editor.document).unwrap(),
        bytes
    );
    editor.undo();
    assert_eq!(formatting::paragraph_selection(&editor), aligned);
    editor.redo();
    assert_eq!(formatting::paragraph_selection(&editor), expected);
}
