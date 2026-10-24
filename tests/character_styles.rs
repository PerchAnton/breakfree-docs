mod support;
use breakfree_docs::{commands::Editor, model::*, session::Session, storage};

#[test]
fn cascade_preserves_unspecified_values_and_explicit_false_wins() {
    let paragraph = Style {
        bold: true,
        underline: true,
        font_size: "24".parse().unwrap(),
        ..Default::default()
    };
    let mut format = CharacterFormat {
        character_style: Some(CharacterStyle::Emphasis),
        ..Default::default()
    };
    let resolved = format.resolve(paragraph);
    assert!(resolved.bold && resolved.italic && resolved.underline);
    assert_eq!(resolved.font_size, paragraph.font_size);
    format.italic = Some(false);
    assert!(!format.resolve(paragraph).italic);
    format.character_style = Some(CharacterStyle::Strong);
    format.bold = Some(false);
    assert!(!format.resolve(paragraph).bold);
}

#[test]
fn selection_apply_clear_remove_are_distinct_single_undo_operations() {
    let mut e = Editor::default();
    e.replace(0, 0, "English e\u{301}\nУкраїна\nРусский 🙂", "Input")
        .unwrap();
    let end = e.document.character_count();
    e.emphasize(0, end, true, false).unwrap();
    e.apply_paragraph_style(0, end, ParagraphStyle::Heading1)
        .unwrap();
    let before = e.document.clone();
    let n = e.undo_stack.len();
    e.apply_character_style(0, end, Some(CharacterStyle::Strong))
        .unwrap();
    let named = e.document.clone();
    assert_eq!(e.undo_stack.len(), n + 1);
    assert_eq!(
        e.selected_character_style(),
        Some(Some(CharacterStyle::Strong))
    );
    for p in &named.paragraphs {
        for f in &p.styles {
            assert!(!f.resolve(ParagraphStyle::Heading1.character_style()).bold);
        }
    }
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, named);
    e.apply_character_style(0, end, Some(CharacterStyle::Strong))
        .unwrap();
    assert_eq!(e.undo_stack.len(), n + 1);
    e.clear_character_formatting(0, end).unwrap();
    assert!(
        e.document.paragraphs[0].styles[0]
            .resolve(Style::default())
            .bold
    );
    assert_eq!(
        e.document.paragraphs[0].styles[0].character_style,
        Some(CharacterStyle::Strong)
    );
    e.undo();
    assert_eq!(e.document, named);
    e.apply_character_style(0, end, None).unwrap();
    assert_eq!(e.document, before);
    e.undo();
    assert_eq!(e.document, named);
    e.redo();
    assert_eq!(e.document, before);
}

#[test]
fn caret_style_is_undoable_without_dirtying_document_and_survives_enter() {
    let mut s = Session::default();
    s.editor
        .apply_character_style(0, 0, Some(CharacterStyle::Emphasis))
        .unwrap();
    assert!(!s.dirty());
    assert_eq!(s.editor.undo_stack.len(), 1);
    s.editor.undo();
    assert_eq!(s.editor.typing_style.character_style, None);
    s.editor.redo();
    assert_eq!(
        s.editor.typing_style.character_style,
        Some(CharacterStyle::Emphasis)
    );
    s.editor.replace(0, 0, "a", "Input").unwrap();
    s.editor.insert_paragraph_break(1, 1).unwrap();
    s.editor.replace(2, 2, "b", "Input").unwrap();
    assert!(
        s.editor
            .document
            .paragraphs
            .iter()
            .all(|p| p.styles[0].character_style == Some(CharacterStyle::Emphasis))
    );
    let after = s.editor.document.clone();
    s.editor.undo();
    s.editor.redo();
    assert_eq!(s.editor.document, after);
}

#[test]
fn mixed_selection_unicode_rejection_and_noop_preserve_history() {
    let mut e = Editor::default();
    e.replace(0, 0, "e\u{301}🙂\nxyz", "Input").unwrap();
    let before = e.document.clone();
    let n = e.undo_stack.len();
    assert!(
        e.apply_character_style(1, 2, Some(CharacterStyle::Strong))
            .is_err()
    );
    assert_eq!(e.document, before);
    assert_eq!(e.undo_stack.len(), n);
    e.apply_character_style(0, 2, Some(CharacterStyle::Strong))
        .unwrap();
    e.selection = (0, 3);
    assert_eq!(e.selected_character_style(), None);
    e.selection = (3, 4);
    assert_eq!(e.selected_character_style(), None);
    e.undo();
    e.apply_character_style(0, 2, None).unwrap();
    assert_eq!(e.redo_stack.len(), 1);
    e.redo();
    assert_eq!(
        e.document.paragraphs[0].styles[0].character_style,
        Some(CharacterStyle::Strong)
    );
}

#[test]
fn v7_roundtrip_strict_references_legacy_migration_and_failed_save_preserve_bytes() {
    let mut e = Editor::default();
    e.replace(0, 0, "Same appearance", "Input").unwrap();
    e.apply_character_style(0, 4, Some(CharacterStyle::Strong))
        .unwrap();
    e.emphasize(5, 15, true, true).unwrap();
    let bytes = storage::encode(&e.document).unwrap();
    assert_eq!(storage::decode(&bytes).unwrap(), e.document);
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["content_version"], storage::CONTENT_VERSION);
    for pointer in [
        "/paragraphs/0/runs/0/character_style",
        "/paragraphs/0/runs/0/direct/character_style",
        "/style_catalog/normal/character/character_style",
    ] {
        let mut bad = value.clone();
        if pointer.ends_with("direct/character_style") {
            bad["paragraphs"][0]["runs"][0]["direct"]["character_style"] = "strong".into();
        } else if pointer.starts_with("/style") {
            bad["style_catalog"]["normal"]["character"]["character_style"] = "strong".into();
        } else {
            *bad.pointer_mut(pointer).unwrap() = "unknown".into();
        }
        assert!(
            storage::decode(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "{pointer}"
        );
    }
    let mut legacy = value.clone();
    legacy["content_version"] = 6.into();
    support::strip_structure_for_legacy(&mut legacy);
    legacy.as_object_mut().unwrap().remove("character_styles");
    assert!(storage::decode(&serde_json::to_vec(&legacy).unwrap()).is_err());
    for p in legacy["paragraphs"].as_array_mut().unwrap() {
        p.as_object_mut().unwrap().remove("spacing");
        for r in p["runs"].as_array_mut().unwrap() {
            r.as_object_mut().unwrap().remove("character_style");
        }
    }
    let migrated = storage::decode(&serde_json::to_vec(&legacy).unwrap()).unwrap();
    assert!(
        migrated.paragraphs[0]
            .styles
            .iter()
            .all(|f| f.character_style.is_none())
    );
    let mut invalid = e.document.clone();
    invalid.style_catalog.normal.character.character_style = Some(CharacterStyle::Strong);
    assert!(storage::encode(&invalid).is_err());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("styles.bfdoc");
    storage::save(&e.document, &path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), e.document);
    e.apply_character_style(0, 4, None).unwrap();
    for stage in [
        storage::SaveStage::Write,
        storage::SaveStage::Sync,
        storage::SaveStage::Replace,
    ] {
        assert!(
            storage::save_checked(&e.document, &path, |at| if at == stage {
                Err("Injected".into())
            } else {
                Ok(())
            })
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn parent_edits_and_replace_all_keep_semantic_character_references() {
    let mut e = Editor::default();
    e.replace(0, 0, "cat cat", "Input").unwrap();
    e.apply_character_style(0, 7, Some(CharacterStyle::Emphasis))
        .unwrap();
    let mut definition = e.document.style_catalog.normal.clone();
    definition.character.bold = Some(true);
    e.modify_style(ParagraphStyle::Normal, definition).unwrap();
    let base = e
        .document
        .style_catalog
        .resolve(ParagraphStyle::Normal)
        .character;
    assert!(e.document.paragraphs[0].styles[0].resolve(base).bold);
    assert!(e.document.paragraphs[0].styles[0].resolve(base).italic);
    let before = e.document.clone();
    let plan = e.preview_replace_all(&[(0, 3), (4, 7)], "dog").unwrap();
    e.apply_replace_all(plan).unwrap();
    assert_eq!(e.document.text(), "dog dog");
    assert!(
        e.document.paragraphs[0]
            .styles
            .iter()
            .all(|f| f.character_style == Some(CharacterStyle::Emphasis))
    );
    let after = e.document.clone();
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
}
