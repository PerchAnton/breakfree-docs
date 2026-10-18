use breakfree_docs::{commands::Editor, model::*, session::Session, storage};

fn format() -> CharacterFormat {
    CharacterFormat {
        bold: Some(false),
        italic: Some(true),
        underline: Some(true),
        font_size: Some("24".parse().unwrap()),
        ..Default::default()
    }
}

#[test]
fn create_apply_selection_is_one_exact_transaction_and_preserves_direct_overrides() {
    let mut e = Editor::default();
    e.replace(0, 0, "English e\u{301}\nУкраїна Русский 🙂", "Input")
        .unwrap();
    e.apply_paragraph_style(0, 0, ParagraphStyle::Heading1)
        .unwrap();
    e.emphasize(0, 2, true, true).unwrap();
    e.selection = (0, e.document.character_count());
    let before = e.document.clone();
    let pending = e.typing_style;
    let n = e.undo_stack.len();
    let id = e
        .create_character_style("Наголос e\u{301}", format(), true)
        .unwrap();
    assert_eq!(id, CharacterStyle::Custom(1));
    assert_eq!(e.undo_stack.len(), n + 1);
    assert_eq!(e.selected_character_style(), Some(Some(id)));
    let after = e.document.clone();
    let p = &after.paragraphs[0];
    let base = after.style_catalog.resolve(p.paragraph_style).character;
    assert!(p.styles[0].resolve_with(base, &after.character_styles).bold);
    let resolved = p.styles[2].resolve_with(base, &after.character_styles);
    assert!(!resolved.bold && resolved.italic && resolved.underline);
    assert_eq!(resolved.font_size.to_string(), "24");
    e.undo();
    assert_eq!(e.document, before);
    assert_eq!(e.typing_style, pending);
    e.redo();
    assert_eq!(e.document, after);
    assert_eq!(
        storage::decode(&storage::encode(&after).unwrap()).unwrap(),
        after
    );
}

#[test]
fn definition_only_and_caret_creation_dirty_checkpoints_and_enter_history() {
    let mut s = Session::default();
    let original = s.editor.document.clone();
    let id = s
        .editor
        .create_character_style("Note", format(), false)
        .unwrap();
    assert!(s.dirty());
    assert_eq!(s.editor.typing_style.character_style, None);
    assert_eq!(s.editor.document.text(), "");
    s.editor.undo();
    assert_eq!(s.editor.document, original);
    assert!(!s.dirty());
    s.editor.redo();
    s.editor.apply_character_style(0, 0, Some(id)).unwrap();
    s.editor.replace(0, 0, "A", "Input").unwrap();
    s.editor.insert_paragraph_break(1, 1).unwrap();
    s.editor.replace(2, 2, "B", "Input").unwrap();
    assert_eq!(s.editor.document.character_styles.len(), 1);
    assert!(
        s.editor
            .document
            .paragraphs
            .iter()
            .all(|p| p.styles[0].character_style == Some(id))
    );
    let mut e = Editor::default();
    let id = e.create_character_style("Caret", format(), true).unwrap();
    assert_eq!(e.typing_style.character_style, Some(id));
    assert_eq!(e.undo_stack.len(), 1);
    e.undo();
    assert!(e.document.character_styles.is_empty());
    assert_eq!(e.typing_style.character_style, None);
    e.redo();
    assert_eq!(e.typing_style.character_style, Some(id));
}

#[test]
fn invalid_names_references_and_catalog_limits_preserve_history() {
    let mut e = Editor::default();
    e.create_character_style("Existing", format(), false)
        .unwrap();
    e.replace(0, 0, "x", "Input").unwrap();
    e.undo();
    let before = e.document.clone();
    let history = (e.undo_stack.len(), e.redo_stack.len());
    for name in [
        "",
        " Existing",
        "Existing ",
        "existing",
        "STRONG",
        "Emphasis",
        "Default paragraph font",
        "a\nb",
        "a\u{2028}b",
        &"x".repeat(65),
    ] {
        assert!(
            e.create_character_style(name, format(), true).is_err(),
            "{name}"
        );
        assert_eq!(e.document, before);
        assert_eq!((e.undo_stack.len(), e.redo_stack.len()), history);
    }
    assert!(
        e.apply_character_style(0, 0, Some(CharacterStyle::Custom(999)))
            .is_err()
    );
    assert!(
        e.create_character_style(
            "Nested",
            CharacterFormat {
                character_style: Some(CharacterStyle::Strong),
                ..format()
            },
            true
        )
        .is_err()
    );
    for i in 1..64 {
        e.create_character_style(&format!("Style {i}"), format(), false)
            .unwrap();
    }
    assert!(
        e.create_character_style("Overflow", format(), false)
            .is_err()
    );
    let mut exhausted = Document::default();
    exhausted.character_styles.push(NamedCharacterStyle {
        id: u32::MAX,
        name: "Last".into(),
        format: format(),
    });
    assert!(exhausted.add_character_style("Next", format()).is_err());
}

#[test]
fn custom_definition_survives_replacement_clear_and_font_step() {
    let mut e = Editor::default();
    e.replace(0, 0, "cat cat", "Input").unwrap();
    e.selection = (0, 7);
    let id = e.create_character_style("Note", format(), true).unwrap();
    assert_eq!(e.selected_font_size().unwrap().to_string(), "24");
    e.step_font_size(0, 7, true).unwrap();
    assert!(e.selected_font_size().unwrap().points() > 24.);
    e.clear_character_formatting(0, 7).unwrap();
    assert_eq!(e.selected_font_size().unwrap().to_string(), "24");
    let before = e.document.clone();
    let plan = e.preview_replace_all(&[(0, 3), (4, 7)], "dog").unwrap();
    e.apply_replace_all(plan).unwrap();
    assert_eq!(e.document.character_styles, before.character_styles);
    assert!(
        e.document.paragraphs[0]
            .styles
            .iter()
            .all(|f| f.character_style == Some(id))
    );
    e.undo();
    assert_eq!(e.document, before);
}

#[test]
fn native_v8_validation_legacy_reading_and_failed_writes_are_lossless() {
    let mut e = Editor::default();
    e.replace(0, 0, "Hello Україна", "Input").unwrap();
    e.selection = (0, e.document.character_count());
    e.create_character_style("Accent", format(), true).unwrap();
    let bytes = storage::encode(&e.document).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["content_version"], 8);
    for (pointer, bad) in [
        ("/character_styles/0/id", serde_json::json!(0)),
        ("/character_styles/0/name", serde_json::json!("Strong")),
        (
            "/character_styles/0/format/font_size",
            serde_json::json!(801),
        ),
        (
            "/paragraphs/0/runs/0/character_style",
            serde_json::json!({"custom":999}),
        ),
    ] {
        let mut json = value.clone();
        *json.pointer_mut(pointer).unwrap() = bad;
        assert!(storage::decode(&serde_json::to_vec(&json).unwrap()).is_err());
    }
    let mut bad = value.clone();
    bad["character_styles"] =
        serde_json::json!([value["character_styles"][0], value["character_styles"][0]]);
    assert!(storage::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    bad = value.clone();
    bad["content_version"] = 7.into();
    bad.as_object_mut().unwrap().remove("character_styles");
    assert!(storage::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    let old = include_bytes!("../examples/CharacterStyles.bfdoc");
    let legacy = storage::decode(old).unwrap();
    assert!(legacy.character_styles.is_empty());
    assert_eq!(
        storage::decode(&storage::encode(&legacy).unwrap()).unwrap(),
        legacy
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("named.bfdoc");
    storage::save(&e.document, &path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), e.document);
    for stage in [
        storage::SaveStage::Write,
        storage::SaveStage::Sync,
        storage::SaveStage::Replace,
    ] {
        assert!(
            storage::save_checked(&Document::default(), &path, |at| if at == stage {
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
fn modify_properties_preserves_identity_references_and_exact_history() {
    let mut e = Editor::default();
    e.replace(0, 0, "English e\u{301}\nУкраїна Русский 🙂", "Input")
        .unwrap();
    e.selection = (0, e.document.character_count());
    let id = e.create_character_style("Note", format(), true).unwrap();
    e.emphasize(0, 2, true, true).unwrap();
    let before = e.document.clone();
    let selection = e.selection;
    let typing = e.typing_style;
    let n = e.undo_stack.len();
    let changed = CharacterFormat {
        bold: Some(false),
        italic: Some(false),
        ..Default::default()
    };
    e.modify_character_style(id, changed).unwrap();
    assert_eq!(e.undo_stack.len(), n + 1);
    assert_eq!(e.document.paragraphs, before.paragraphs);
    assert_eq!(
        e.document.character_styles[0].id,
        before.character_styles[0].id
    );
    assert_eq!(e.document.character_styles[0].name, "Note");
    assert_eq!(e.selection, selection);
    assert_eq!(e.typing_style, typing);
    let p = &e.document.paragraphs[0];
    let base = e
        .document
        .style_catalog
        .resolve(p.paragraph_style)
        .character;
    assert!(
        p.styles[0]
            .resolve_with(base, &e.document.character_styles)
            .bold
    );
    assert!(
        !p.styles[2]
            .resolve_with(base, &e.document.character_styles)
            .italic
    );
    let after = e.document.clone();
    assert_eq!(
        storage::decode(&storage::encode(&after).unwrap()).unwrap(),
        after
    );
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
}

#[test]
fn modify_unused_invalid_and_noop_definitions_preserve_history() {
    let mut s = Session::default();
    let id = s
        .editor
        .create_character_style("Unused", format(), false)
        .unwrap();
    s.editor.replace(0, 0, "x", "Input").unwrap();
    s.editor.undo();
    let before = s.editor.document.clone();
    let n = s.editor.undo_stack.len();
    s.editor.modify_character_style(id, format()).unwrap();
    for bad in [
        CharacterStyle::Strong,
        CharacterStyle::Emphasis,
        CharacterStyle::Custom(999),
    ] {
        assert!(s.editor.modify_character_style(bad, format()).is_err());
    }
    assert!(
        s.editor
            .modify_character_style(
                id,
                CharacterFormat {
                    character_style: Some(id),
                    ..format()
                }
            )
            .is_err()
    );
    assert_eq!(s.editor.document, before);
    assert_eq!(s.editor.undo_stack.len(), n);
    assert_eq!(s.editor.redo_stack.len(), 1);
    s.editor
        .modify_character_style(id, CharacterFormat::default())
        .unwrap();
    assert!(s.editor.redo_stack.is_empty());
    assert!(s.dirty());
    s.editor.undo();
    assert_eq!(s.editor.document, before);
}

#[test]
fn rename_character_style_retains_references_and_combined_history() {
    let mut e = Editor::default();
    e.replace(0, 0, "English e\u{301}\nУкраїна Русский 🙂", "Input")
        .unwrap();
    e.selection = (0, e.document.character_count());
    let id = e.create_character_style("Note", format(), true).unwrap();
    let before = e.document.clone();
    let selection = e.selection;
    let typing = e.typing_style;
    let count = e.undo_stack.len();
    e.update_character_style(id, "Примітка e\u{301}", format())
        .unwrap();
    assert_eq!(e.document.paragraphs, before.paragraphs);
    assert_eq!(
        e.document.character_styles[0].id,
        before.character_styles[0].id
    );
    assert_eq!(e.typing_style, typing);
    assert_eq!(e.selection, selection);
    assert_eq!(e.undo_stack.len(), count + 1);
    let after = e.document.clone();
    assert_eq!(
        storage::decode(&storage::encode(&after).unwrap()).unwrap(),
        after
    );
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
    let changed = CharacterFormat {
        bold: Some(true),
        ..format()
    };
    e.update_character_style(id, "Renamed", changed).unwrap();
    e.undo();
    assert_eq!(e.document, after);
    e.redo();
    assert_eq!(e.document.character_styles[0].format, changed);
    assert_eq!(e.document.character_style_name(id), "Renamed");
}

#[test]
fn rename_invalid_noop_unused_and_case_only_edits() {
    let mut e = Editor::default();
    let id = e.create_character_style("Note", format(), false).unwrap();
    e.create_character_style("Other", format(), false).unwrap();
    e.replace(0, 0, "x", "Input").unwrap();
    e.undo();
    let before = e.document.clone();
    for name in [
        "",
        " Note",
        "Note ",
        "OTHER",
        "Strong",
        "a\nb",
        "a\u{2028}b",
        &"x".repeat(65),
    ] {
        assert!(e.update_character_style(id, name, format()).is_err());
    }
    for bad in [CharacterStyle::Strong, CharacterStyle::Custom(999)] {
        assert!(e.update_character_style(bad, "Valid", format()).is_err());
    }
    e.update_character_style(id, "Note", format()).unwrap();
    assert_eq!(e.document, before);
    assert_eq!(e.redo_stack.len(), 1);
    e.update_character_style(id, "NOTE", format()).unwrap();
    assert_eq!(e.document.character_style_name(id), "NOTE");
    e.undo();
    assert_eq!(e.document, before);
}
