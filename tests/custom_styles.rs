use breakfree_docs::{commands::Editor, model::*, storage};

fn based_on(parent: ParagraphStyle) -> StyleDefinition {
    StyleDefinition {
        parent: Some(parent),
        ..Default::default()
    }
}

#[test]
fn create_apply_is_one_exact_undo_with_unicode_selection_and_direct_overrides() {
    let mut e = Editor::default();
    e.replace(0, 0, "English café\nУкраїна\nРусский", "Input")
        .unwrap();
    e.emphasize(0, 1, true, false).unwrap();
    e.align(0, 0, Alignment::Center).unwrap();
    e.selection = (0, 21); // exclusive start of the third paragraph
    let before = e.document.clone();
    let typing = e.typing_style;
    let depth = e.undo_stack.len();
    let id = e
        .create_style("Назва café", based_on(ParagraphStyle::Heading1), true)
        .unwrap();
    let after = e.document.clone();
    assert_eq!(id, ParagraphStyle::Custom(1));
    assert_eq!(e.undo_stack.len(), depth + 1);
    assert_eq!(e.selection, (0, 21));
    assert_eq!(e.typing_style, typing);
    for i in 0..2 {
        assert_eq!(after.paragraphs[i].paragraph_style, id);
        assert_eq!(after.paragraphs[i].styles, before.paragraphs[i].styles);
        assert_eq!(
            after.paragraphs[i].alignment,
            before.paragraphs[i].alignment
        );
    }
    assert_eq!(after.paragraphs[2], before.paragraphs[2]);
    assert!(after.style_catalog.resolve(id).character.bold);
    assert!(
        !after.paragraphs[0].styles[0]
            .resolve(after.style_catalog.resolve(id).character)
            .bold
    );
    e.undo();
    assert_eq!(e.document, before);
    assert_eq!(e.selection, (0, 21));
    assert_eq!(e.typing_style, typing);
    e.redo();
    assert_eq!(e.document, after);
    assert_eq!(e.selection, (0, 21));
}

#[test]
fn definition_only_empty_paragraph_and_custom_parents_share_graph_rules() {
    let mut e = Editor::default();
    let before = e.document.paragraphs.clone();
    let parent = e
        .create_style("Body", based_on(ParagraphStyle::NoSpacing), false)
        .unwrap();
    assert_eq!(e.document.paragraphs, before);
    let child = e.create_style("Child", based_on(parent), true).unwrap();
    assert_eq!(e.selected_paragraph_style(), Some(child));
    let mut definition = *e.document.style_catalog.get(parent).unwrap();
    definition.character.font_size = Some("20".parse().unwrap());
    definition.character.italic = Some(true);
    e.modify_style(parent, definition).unwrap();
    assert_eq!(e.selected_font_size().unwrap().to_string(), "20");
    assert_eq!(
        e.document.style_catalog.resolve(child).line_height_percent,
        100
    );
    let valid = e.document.clone();
    definition.parent = Some(child);
    assert!(e.modify_style(parent, definition).is_err());
    assert_eq!(e.document, valid);
    assert!(
        e.modify_style(ParagraphStyle::Custom(99), definition)
            .is_err()
    );
    assert!(
        e.apply_paragraph_style(0, 0, ParagraphStyle::Custom(99))
            .is_err()
    );
    e.undo();
    assert_eq!(e.selected_font_size().unwrap().to_string(), "11");
    e.undo();
    assert_eq!(e.document.paragraphs, before);
    assert!(e.document.style_catalog.get(child).is_none());
    e.redo();
    assert_eq!(e.selected_paragraph_style(), Some(child));
}

#[test]
fn names_limits_missing_parents_and_grapheme_errors_preserve_redo() {
    let mut e = Editor::default();
    e.replace(0, 0, "café", "Input").unwrap();
    e.create_style("Україна Русский", StyleDefinition::default(), false)
        .unwrap();
    e.create_style("Undo me", StyleDefinition::default(), false)
        .unwrap();
    e.undo();
    let before = e.document.clone();
    let depth = e.undo_stack.len();
    for name in [
        "",
        " normal",
        "Normal",
        "heading 1",
        "No Spacing",
        "УКРАЇНА РУССКИЙ",
        "a\n",
        "a\u{2028}b",
        &"x".repeat(65),
    ] {
        assert!(
            e.create_style(name, StyleDefinition::default(), true)
                .is_err(),
            "{name:?}"
        );
        assert_eq!(e.document, before);
        assert_eq!(e.undo_stack.len(), depth);
        assert_eq!(e.redo_stack.len(), 1);
    }
    for parent in [ParagraphStyle::Custom(999), ParagraphStyle::Custom(2)] {
        assert!(
            e.create_style("Valid name", based_on(parent), false)
                .is_err()
        );
    }
    e.selection = (4, 4); // inside e + combining accent
    assert!(
        e.create_style("Split", StyleDefinition::default(), true)
            .is_err()
    );
    assert_eq!(e.document, before);
    e.selection = (0, 0);
    assert!(
        e.create_style(&"Ї".repeat(64), StyleDefinition::default(), false)
            .is_ok()
    );
    for i in 2..MAX_CUSTOM_STYLES {
        e.create_style(&format!("Style {i}"), StyleDefinition::default(), false)
            .unwrap();
    }
    let full = e.document.clone();
    assert!(
        e.create_style("Too many", StyleDefinition::default(), false)
            .is_err()
    );
    assert_eq!(e.document, full);
    e.undo();
    let id = e
        .create_style("Replacement", StyleDefinition::default(), false)
        .unwrap();
    assert_eq!(id, ParagraphStyle::Custom(64));
    assert!(e.redo_stack.is_empty());
}

#[test]
fn native_v6_roundtrip_rejects_malformed_ids_names_graphs_and_preserves_previous_file() {
    let mut e = Editor::default();
    e.replace(0, 0, "English\nУкраїна café\nРусский", "Input")
        .unwrap();
    let id = e
        .create_style("Custom", based_on(ParagraphStyle::Heading1), true)
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("custom.bfdoc");
    storage::save(&e.document, &path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), e.document);
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["content_version"], 6);
    for (pointer, value) in [
        ("/style_catalog/custom/0/id", serde_json::json!(0)),
        ("/style_catalog/custom/0/name", serde_json::json!("Normal")),
        (
            "/style_catalog/custom/0/definition/parent",
            serde_json::json!({"custom":1}),
        ),
        (
            "/style_catalog/custom/0/definition/parent",
            serde_json::json!({"custom":99}),
        ),
        (
            "/paragraphs/0/paragraph_style",
            serde_json::json!({"custom":99}),
        ),
        ("/content_version", serde_json::json!(7)),
        ("/format", serde_json::json!("breakfree.sheets.native")),
    ] {
        let mut bad = json.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(
            storage::decode(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "{pointer}"
        );
    }
    let mut duplicate = json.clone();
    duplicate["style_catalog"]["custom"]
        .as_array_mut()
        .unwrap()
        .push(json["style_catalog"]["custom"][0].clone());
    assert!(storage::decode(&serde_json::to_vec(&duplicate).unwrap()).is_err());
    for (from, to) in [
        ("\"id\": 1", "\"id\": 1, \"id\": 1"),
        (
            "\"name\": \"Custom\"",
            "\"name\": \"Custom\", \"extra\": true",
        ),
    ] {
        assert!(
            storage::decode(
                String::from_utf8(bytes.clone())
                    .unwrap()
                    .replace(from, to)
                    .as_bytes()
            )
            .is_err()
        );
    }
    e.modify_style(id, based_on(ParagraphStyle::NoSpacing))
        .unwrap();
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
    assert_eq!(
        storage::load(&path)
            .unwrap()
            .style_catalog
            .get(id)
            .unwrap()
            .parent,
        Some(ParagraphStyle::Heading1)
    );
}

#[test]
fn v4_import_preserves_definitions_without_rewriting_and_rejects_custom_extensions() {
    let mut e = Editor::default();
    let mut definition = e.document.style_catalog.normal;
    definition.character.italic = Some(true);
    e.modify_style(ParagraphStyle::Normal, definition).unwrap();
    let mut v4: serde_json::Value =
        serde_json::from_slice(&storage::encode(&e.document).unwrap()).unwrap();
    v4["content_version"] = 4.into();
    for id in ["normal", "no_spacing", "heading1"] {
        v4["style_catalog"][id]
            .as_object_mut()
            .unwrap()
            .remove("next_style");
    }
    v4["style_catalog"]
        .as_object_mut()
        .unwrap()
        .remove("custom");
    let bytes = serde_json::to_vec(&v4).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("old.bfdoc");
    std::fs::write(&path, &bytes).unwrap();
    assert_eq!(storage::load(&path).unwrap(), e.document);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    v4["style_catalog"]["custom"] = serde_json::json!([]);
    assert!(storage::decode(&serde_json::to_vec(&v4).unwrap()).is_err());
    v4["style_catalog"]
        .as_object_mut()
        .unwrap()
        .remove("custom");
    v4["paragraphs"][0]["paragraph_style"] = serde_json::json!({"custom":1});
    assert!(storage::decode(&serde_json::to_vec(&v4).unwrap()).is_err());
}

#[test]
fn custom_ids_survive_text_edits_bulk_replacement_and_clear_with_stale_preview_rejection() {
    let mut e = Editor::default();
    e.replace(0, 0, "one\none", "Input").unwrap();
    let stale = e.preview_replace_all(&[(0, 3), (4, 7)], "two").unwrap();
    e.selection = (0, 7);
    let id = e
        .create_style("Both", based_on(ParagraphStyle::Heading1), true)
        .unwrap();
    assert!(e.apply_replace_all(stale).is_err());
    let before = e.document.clone();
    let plan = e
        .preview_replace_all(&[(0, 3), (4, 7)], "two\nthree")
        .unwrap();
    e.apply_replace_all(plan).unwrap();
    assert!(
        e.document
            .paragraphs
            .iter()
            .all(|p| p.paragraph_style == id)
    );
    assert_eq!(e.document.style_catalog, before.style_catalog);
    e.undo();
    assert_eq!(e.document, before);
    e.emphasize(0, 3, true, false).unwrap();
    e.clear_character_formatting(0, 3).unwrap();
    assert_eq!(e.document, before);
    e.replace(1, 1, "\n", "Split").unwrap();
    assert!(
        e.document
            .paragraphs
            .iter()
            .all(|p| p.paragraph_style == id)
    );
    assert_eq!(
        storage::decode(&storage::encode(&e.document).unwrap()).unwrap(),
        e.document
    );
}

#[test]
fn published_custom_fixture_reopens_with_names_inheritance_and_direct_formatting() {
    let d = storage::decode(include_bytes!("../examples/CustomStyles.bfdoc")).unwrap();
    assert_eq!(
        d.style_catalog.name(ParagraphStyle::Custom(1)),
        "Report Heading"
    );
    assert!(
        d.style_catalog
            .resolve(ParagraphStyle::Custom(1))
            .character
            .bold
    );
    assert!(
        d.style_catalog
            .resolve(ParagraphStyle::Custom(2))
            .character
            .italic
    );
    assert!(d.paragraphs[2].styles[0].bold.unwrap());
    assert_eq!(d.paragraphs[2].alignment, Alignment::Center);
    assert_eq!(storage::decode(&storage::encode(&d).unwrap()).unwrap(), d);
}

#[test]
fn rename_parent_preserves_ids_descendants_direct_values_and_exact_history() {
    let doc = storage::decode(include_bytes!("../examples/CustomStyles.bfdoc")).unwrap();
    let mut e = Editor::new(doc);
    let parent = ParagraphStyle::Custom(1);
    let child = ParagraphStyle::Custom(2);
    let mut child_definition = *e.document.style_catalog.get(child).unwrap();
    child_definition.parent = Some(parent);
    e.modify_style(child, child_definition).unwrap();
    e.selection = (0, e.document.character_count());
    e.typing_style.underline = Some(true);
    let before = e.document.clone();
    let selection = e.selection;
    let pending = e.typing_style;
    let depth = e.undo_stack.len();
    let definition = *before.style_catalog.get(parent).unwrap();
    e.modify_custom_style(parent, "Заголовок café Русский", definition)
        .unwrap();
    let after = e.document.clone();
    assert_eq!(e.undo_stack.len(), depth + 1);
    assert_eq!(after.paragraphs, before.paragraphs);
    assert_eq!(
        after.style_catalog.get(child),
        before.style_catalog.get(child)
    );
    assert_eq!(
        after.style_catalog.get(parent),
        before.style_catalog.get(parent)
    );
    assert_eq!(after.style_catalog.ids(), before.style_catalog.ids());
    assert_eq!(
        after.style_catalog.resolve(child),
        before.style_catalog.resolve(child)
    );
    assert_eq!(after.style_catalog.name(parent), "Заголовок café Русский");
    e.undo();
    assert_eq!(e.document, before);
    assert_eq!((e.selection, e.typing_style), (selection, pending));
    e.redo();
    assert_eq!(e.document, after);
    assert_eq!((e.selection, e.typing_style), (selection, pending));
}

#[test]
fn rename_noops_errors_and_combined_invalid_definition_preserve_redo() {
    let mut e =
        Editor::new(storage::decode(include_bytes!("../examples/CustomStyles.bfdoc")).unwrap());
    let id = ParagraphStyle::Custom(1);
    let definition = *e.document.style_catalog.get(id).unwrap();
    e.replace(0, 0, "Undo me", "Input").unwrap();
    e.undo();
    let before = e.document.clone();
    let selection = e.selection;
    let pending = e.typing_style;
    e.modify_custom_style(id, "Report Heading", definition)
        .unwrap();
    assert!(e.undo_stack.is_empty());
    assert_eq!(e.redo_stack.len(), 1);
    for name in [
        "",
        " report",
        "report ",
        "Normal",
        "no spacing",
        "REPORT BODY",
        "a\n",
        "a\u{2029}b",
        &"x".repeat(65),
    ] {
        assert!(
            e.modify_custom_style(id, name, definition).is_err(),
            "{name:?}"
        );
        assert_eq!(e.document, before);
        assert!(e.undo_stack.is_empty());
        assert_eq!(e.redo_stack.len(), 1);
    }
    for target in [
        ParagraphStyle::Normal,
        ParagraphStyle::NoSpacing,
        ParagraphStyle::Heading1,
        ParagraphStyle::Custom(999),
    ] {
        assert!(
            e.modify_custom_style(target, "Valid name", definition)
                .is_err()
        );
        assert_eq!(e.document, before);
    }
    let mut cycle = definition;
    cycle.parent = Some(id);
    assert!(e.modify_custom_style(id, "Still valid", cycle).is_err());
    let mut invalid = definition;
    invalid.line_height_percent = Some(99);
    assert!(e.modify_custom_style(id, "Still valid", invalid).is_err());
    assert_eq!(e.document, before);
    assert_eq!((e.selection, e.typing_style), (selection, pending));
    assert_eq!(e.redo_stack.len(), 1);
    // A case-only edit is valid: the target is not a duplicate of itself.
    e.modify_custom_style(id, "REPORT HEADING", definition)
        .unwrap();
    assert_eq!(e.undo_stack.len(), 1);
    assert!(e.redo_stack.is_empty());
    e.undo();
    assert_eq!(e.document, before);
}

#[test]
fn rename_and_definition_change_are_one_transaction_and_work_at_catalog_capacity() {
    let mut e = Editor::default();
    for i in 1..=MAX_CUSTOM_STYLES {
        e.create_style(
            &format!("Style {i}"),
            based_on(ParagraphStyle::Normal),
            false,
        )
        .unwrap();
    }
    let id = ParagraphStyle::Custom(64); // unused definition still needs to be renameable
    let before = e.document.clone();
    let depth = e.undo_stack.len();
    let definition = StyleDefinition {
        parent: Some(ParagraphStyle::Custom(1)),
        character: CharacterFormat {
            italic: Some(true),
            ..Default::default()
        },
        line_height_percent: Some(175),
        space_after_half_points: Some(17),
        next_style: None,
    };
    e.modify_custom_style(id, &"Ї".repeat(64), definition)
        .unwrap();
    assert_eq!(e.undo_stack.len(), depth + 1);
    assert_eq!(e.document.style_catalog.custom.len(), MAX_CUSTOM_STYLES);
    assert_eq!(e.document.paragraphs, before.paragraphs);
    assert_eq!(e.document.style_catalog.get(id), Some(&definition));
    let after = e.document.clone();
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
}

#[test]
fn rename_roundtrip_dirty_checkpoint_failed_write_and_stale_preview_keep_references() {
    use breakfree_docs::session::Session;
    let mut session = Session::new(
        storage::decode(include_bytes!("../examples/CustomStyles.bfdoc")).unwrap(),
        None,
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("renamed.bfdoc");
    session.save(&path).unwrap();
    let original = std::fs::read(&path).unwrap();
    let old = session.editor.document.clone();
    let stale = session
        .editor
        .preview_replace_all(&[(0, 1)], "Another")
        .unwrap();
    let id = ParagraphStyle::Custom(1);
    let definition = *old.style_catalog.get(id).unwrap();
    session
        .editor
        .modify_custom_style(id, "Renamed Україна", definition)
        .unwrap();
    assert!(session.dirty());
    assert!(session.editor.apply_replace_all(stale).is_err());
    session.editor.undo();
    assert!(!session.dirty());
    session.editor.redo();
    assert!(session.dirty());
    let renamed = session.editor.document.clone();
    for stage in [
        storage::SaveStage::Write,
        storage::SaveStage::Sync,
        storage::SaveStage::Replace,
    ] {
        assert!(
            storage::save_checked(&renamed, &path, |at| if at == stage {
                Err("Injected failure".into())
            } else {
                Ok(())
            })
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert!(session.dirty());
    }
    session.save(&path).unwrap();
    assert!(!session.dirty());
    assert_eq!(storage::load(&path).unwrap(), renamed);
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(value["content_version"], 6);
    assert_eq!(renamed.paragraphs, old.paragraphs);
    let plan = session
        .editor
        .preview_replace_all(&[(0, 1)], "Another")
        .unwrap();
    session.editor.apply_replace_all(plan).unwrap();
    assert_eq!(session.editor.document.style_catalog, renamed.style_catalog);
    assert_eq!(session.editor.document.paragraphs[0].paragraph_style, id);
    session.editor.undo();
    assert_eq!(session.editor.document, renamed);
}

fn deletion_fixture() -> Editor {
    let mut e =
        Editor::new(storage::decode(include_bytes!("../examples/CustomStyles.bfdoc")).unwrap());
    let mut child = *e
        .document
        .style_catalog
        .get(ParagraphStyle::Custom(2))
        .unwrap();
    child.parent = Some(ParagraphStyle::Custom(1));
    e.modify_style(ParagraphStyle::Custom(2), child).unwrap();
    e.create_style(
        "Онук café Русский",
        based_on(ParagraphStyle::Custom(2)),
        false,
    )
    .unwrap();
    e
}

#[test]
fn delete_replaces_paragraphs_and_direct_children_separately_in_one_exact_undo() {
    let mut e = deletion_fixture();
    e.selection = (0, e.document.character_count());
    e.typing_style.underline = Some(true);
    let before = e.document.clone();
    let selection = e.selection;
    let pending = e.typing_style;
    let depth = e.undo_stack.len();
    e.delete_custom_style(
        ParagraphStyle::Custom(1),
        ParagraphStyle::Heading1,
        Some(ParagraphStyle::NoSpacing),
    )
    .unwrap();
    let after = e.document.clone();
    let mut expected = before.clone();
    expected.style_catalog.custom.remove(0);
    expected.style_catalog.custom[0].definition.parent = Some(ParagraphStyle::NoSpacing);
    expected.paragraphs[0].paragraph_style = ParagraphStyle::Heading1;
    assert_eq!(after, expected);
    assert_eq!(
        after.style_catalog.custom[1].definition.parent,
        Some(ParagraphStyle::Custom(2))
    );
    assert_eq!(e.undo_stack.len(), depth + 1);
    assert_eq!((e.selection, e.typing_style), (selection, pending));
    e.undo();
    assert_eq!(e.document, before);
    assert_eq!((e.selection, e.typing_style), (selection, pending));
    e.redo();
    assert_eq!(e.document, after);
    assert_eq!((e.selection, e.typing_style), (selection, pending));
}

#[test]
fn deletion_detaches_builtin_children_and_can_replace_paragraphs_with_a_descendant() {
    let mut e = Editor::default();
    e.replace(0, 0, "Україна café\nРусский", "Input").unwrap();
    e.selection = (0, e.document.character_count());
    let id = e
        .create_style("Root", StyleDefinition::default(), true)
        .unwrap();
    e.modify_style(ParagraphStyle::Normal, based_on(id))
        .unwrap();
    let child = e.create_style("Child", based_on(id), false).unwrap();
    let before = e.document.clone();
    e.delete_custom_style(id, child, None).unwrap();
    assert_eq!(e.document.style_catalog.normal.parent, None);
    assert_eq!(e.document.style_catalog.get(child).unwrap().parent, None);
    assert_eq!(e.document.paragraphs[0].paragraph_style, child);
    assert_eq!(
        e.document.style_catalog.heading1.parent,
        Some(ParagraphStyle::Normal)
    );
    assert_eq!(e.document.text(), before.text());
    e.document.validate().unwrap();
    e.undo();
    assert_eq!(e.document, before);
}

#[test]
fn deletion_rejects_unknown_self_and_cyclic_replacements_without_losing_redo() {
    let mut e = deletion_fixture();
    e.replace(0, 0, "x", "Input").unwrap();
    e.undo();
    let before = e.document.clone();
    let depth = e.undo_stack.len();
    let selection = e.selection;
    let pending = e.typing_style;
    for (target, paragraphs, parent) in [
        (ParagraphStyle::Normal, ParagraphStyle::NoSpacing, None),
        (ParagraphStyle::Custom(999), ParagraphStyle::Normal, None),
        (ParagraphStyle::Custom(1), ParagraphStyle::Custom(1), None),
        (ParagraphStyle::Custom(1), ParagraphStyle::Custom(999), None),
        (
            ParagraphStyle::Custom(1),
            ParagraphStyle::Normal,
            Some(ParagraphStyle::Custom(1)),
        ),
        (
            ParagraphStyle::Custom(1),
            ParagraphStyle::Normal,
            Some(ParagraphStyle::Custom(999)),
        ),
        (
            ParagraphStyle::Custom(1),
            ParagraphStyle::Normal,
            Some(ParagraphStyle::Custom(2)),
        ),
        (
            ParagraphStyle::Custom(1),
            ParagraphStyle::Normal,
            Some(ParagraphStyle::Custom(3)),
        ),
    ] {
        assert!(e.delete_custom_style(target, paragraphs, parent).is_err());
        assert_eq!(e.document, before);
        assert_eq!(e.undo_stack.len(), depth);
        assert_eq!(e.redo_stack.len(), 1);
        assert_eq!((e.selection, e.typing_style), (selection, pending));
    }
    e.redo();
    assert!(e.document.text().starts_with('x'));
}

#[test]
fn deleting_unused_style_at_capacity_frees_identity_without_changing_survivors() {
    let mut e = Editor::default();
    for i in 1..=MAX_CUSTOM_STYLES {
        e.create_style(
            &format!("Style {i}"),
            based_on(ParagraphStyle::Normal),
            false,
        )
        .unwrap();
    }
    let before = e.document.clone();
    let depth = e.undo_stack.len();
    let id = ParagraphStyle::Custom(32);
    e.delete_custom_style(id, ParagraphStyle::Normal, None)
        .unwrap();
    let after = e.document.clone();
    assert_eq!(e.undo_stack.len(), depth + 1);
    assert_eq!(after.paragraphs, before.paragraphs);
    assert_eq!(after.style_catalog.custom.len(), 63);
    for style in &after.style_catalog.custom {
        assert_eq!(
            Some(style),
            before
                .style_catalog
                .custom
                .iter()
                .find(|s| s.id == style.id)
        );
    }
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
    assert_eq!(
        e.create_style("Style 32", based_on(ParagraphStyle::Heading1), false)
            .unwrap(),
        id
    );
    assert!(e.redo_stack.is_empty());
    e.undo();
    assert_eq!(e.document, after);
    e.undo();
    assert_eq!(e.document, before);
}

#[test]
fn deletion_save_reopen_failed_writes_dirty_checkpoint_and_bulk_preview_are_safe() {
    use breakfree_docs::session::Session;
    let mut session = Session::new(deletion_fixture().document, None);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("delete.bfdoc");
    session.save(&path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let before = session.editor.document.clone();
    let stale = session.editor.preview_replace_all(&[(0, 1)], "X").unwrap();
    session
        .editor
        .delete_custom_style(
            ParagraphStyle::Custom(1),
            ParagraphStyle::NoSpacing,
            Some(ParagraphStyle::Heading1),
        )
        .unwrap();
    let after = session.editor.document.clone();
    assert!(session.dirty());
    assert!(session.editor.apply_replace_all(stale).is_err());
    session.editor.undo();
    assert!(!session.dirty());
    assert_eq!(session.editor.document, before);
    session.editor.redo();
    assert!(session.dirty());
    for stage in [
        storage::SaveStage::Write,
        storage::SaveStage::Sync,
        storage::SaveStage::Replace,
    ] {
        assert!(
            storage::save_checked(&after, &path, |at| if at == stage {
                Err("Injected failure".into())
            } else {
                Ok(())
            })
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert!(session.dirty());
    }
    session.save(&path).unwrap();
    assert!(!session.dirty());
    assert_eq!(storage::load(&path).unwrap(), after);
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(value["content_version"], 6);
    let plan = session
        .editor
        .preview_replace_all(&[(0, 1)], "café")
        .unwrap();
    session.editor.apply_replace_all(plan).unwrap();
    assert_eq!(session.editor.document.style_catalog, after.style_catalog);
    assert_eq!(
        session.editor.document.paragraphs[0].paragraph_style,
        ParagraphStyle::NoSpacing
    );
    session.editor.undo();
    assert_eq!(session.editor.document, after);
}
