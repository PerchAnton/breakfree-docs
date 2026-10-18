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
