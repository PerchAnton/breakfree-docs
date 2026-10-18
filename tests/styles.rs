use breakfree_docs::{commands::Editor, model::*, storage};

fn effective(p: &Paragraph, at: usize) -> Style {
    p.styles[at].resolve(p.paragraph_style.character_style())
}

#[test]
fn catalog_inheritance_and_explicit_false_have_distinct_meanings() {
    let heading = ParagraphStyle::Heading1;
    assert_eq!(heading.parent(), Some(ParagraphStyle::Normal));
    assert_eq!(
        ParagraphStyle::NoSpacing.parent(),
        Some(ParagraphStyle::Normal)
    );
    let base = heading.character_style();
    assert!(base.bold);
    assert_eq!(base.font_size.to_string(), "18");
    assert_eq!(CharacterFormat::default().resolve(base), base);
    let direct = CharacterFormat {
        bold: Some(false),
        italic: Some(true),
        ..Default::default()
    };
    assert!(!direct.resolve(base).bold);
    assert!(direct.resolve(base).italic);
    assert_eq!(direct.resolve(base).font_size, base.font_size);
    assert_eq!(
        ParagraphStyle::NoSpacing.character_style(),
        Style::default()
    );
    assert_eq!(ParagraphStyle::NoSpacing.line_height(), 1.);
    assert_eq!(ParagraphStyle::NoSpacing.space_after_points(), 0.);
}

#[test]
fn paragraph_application_preserves_overrides_selection_alignment_and_history() {
    let mut e = Editor::default();
    e.replace(0, 0, "Title\nУкраїна e\u{301}\nРусский", "Input")
        .unwrap();
    e.emphasize(0, 1, true, false).unwrap();
    e.underline(1, 2, true).unwrap();
    e.align(0, 5, Alignment::Center).unwrap();
    let before = e.document.clone();
    let pending = e.typing_style;
    let n = e.undo_stack.len();
    e.apply_paragraph_style(1, 6, ParagraphStyle::Heading1)
        .unwrap();
    let after = e.document.clone();
    assert_eq!(e.selection, (1, 6));
    assert_eq!(e.typing_style, pending);
    assert_eq!(e.undo_stack.len(), n + 1);
    assert_eq!(after.paragraphs[1..], before.paragraphs[1..]); // exclusive next start
    assert_eq!(after.paragraphs[0].alignment, Alignment::Center);
    assert!(!effective(&after.paragraphs[0], 0).bold);
    assert!(effective(&after.paragraphs[0], 1).underline);
    assert!(effective(&after.paragraphs[0], 2).bold);
    assert_eq!(after.paragraphs[0].styles, before.paragraphs[0].styles);
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
    e.apply_paragraph_style(1, 6, ParagraphStyle::Heading1)
        .unwrap();
    assert_eq!(e.undo_stack.len(), n + 1);
    e.selection = (0, e.document.character_count());
    assert_eq!(e.selected_paragraph_style(), None);
}

#[test]
fn clear_returns_to_each_paragraph_base_and_grow_uses_resolved_size() {
    let mut e = Editor::default();
    e.replace(0, 0, "Heading\nBody", "Input").unwrap();
    e.apply_paragraph_style(0, 7, ParagraphStyle::Heading1)
        .unwrap();
    let inherited = e.document.clone();
    e.set_font_size(0, 12, "30".parse().unwrap()).unwrap();
    e.emphasize(0, 12, true, false).unwrap();
    let direct = e.document.clone();
    let n = e.undo_stack.len();
    e.clear_character_formatting(0, 12).unwrap();
    assert_eq!(e.document, inherited);
    assert_eq!(e.undo_stack.len(), n + 1);
    assert_eq!(e.selected_font_size(), None);
    e.undo();
    assert_eq!(e.document, direct);
    e.redo();
    assert_eq!(e.document, inherited);
    e.step_font_size(0, 12, true).unwrap();
    assert_eq!(
        effective(&e.document.paragraphs[0], 0)
            .font_size
            .to_string(),
        "20"
    );
    assert_eq!(
        effective(&e.document.paragraphs[1], 0)
            .font_size
            .to_string(),
        "12"
    );
}

#[test]
fn empty_paragraph_split_merge_and_caret_keep_semantic_style() {
    let mut e = Editor::default();
    e.apply_paragraph_style(0, 0, ParagraphStyle::Heading1)
        .unwrap();
    let empty = e.document.clone();
    assert_eq!(e.selected_font_size().unwrap().to_string(), "18");
    e.emphasize(0, 0, true, false).unwrap();
    assert_eq!(e.document, empty);
    e.clear_character_formatting(0, 0).unwrap();
    assert!(e.emphasis_selected(true));
    e.replace(0, 0, "Ї e\u{301}\nРусский\n", "Input").unwrap();
    assert!(
        e.document
            .paragraphs
            .iter()
            .all(|p| p.paragraph_style == ParagraphStyle::Heading1)
    );
    let typed = e.document.clone();
    let split = e.document.paragraphs[0].text.chars().count();
    e.replace(split, split + 1, "", "Merge").unwrap();
    assert_eq!(
        e.document.paragraphs[0].paragraph_style,
        ParagraphStyle::Heading1
    );
    e.undo();
    assert_eq!(e.document, typed);
    for range in [(3, 4), (99, 99)] {
        assert!(
            e.apply_paragraph_style(range.0, range.1, ParagraphStyle::Normal)
                .is_err()
        );
        assert_eq!(e.document, typed);
    }
    assert_eq!(
        storage::decode(&storage::encode(&empty).unwrap()).unwrap(),
        empty
    );
}

#[test]
fn styles_survive_preview_replacement_and_save_failure_without_loss() {
    let mut e = Editor::default();
    e.replace(0, 0, "one\none\none", "Input").unwrap();
    e.apply_paragraph_style(0, 3, ParagraphStyle::Heading1)
        .unwrap();
    e.apply_paragraph_style(4, 7, ParagraphStyle::NoSpacing)
        .unwrap();
    e.emphasize(4, 7, true, false).unwrap();
    let before = e.document.clone();
    let plan = e
        .preview_replace_all(&[(0, 3), (4, 7), (8, 11)], "two\ne\u{301}")
        .unwrap();
    let expected = plan.result().clone();
    assert_eq!(e.document, before);
    e.apply_replace_all(plan).unwrap();
    for (i, style) in [
        ParagraphStyle::Heading1,
        ParagraphStyle::NoSpacing,
        ParagraphStyle::Normal,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(e.document.paragraphs[2 * i].paragraph_style, style);
        assert_eq!(e.document.paragraphs[2 * i + 1].paragraph_style, style);
    }
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, expected);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Styles.bfdoc");
    storage::save(&before, &path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    for stage in [
        storage::SaveStage::Write,
        storage::SaveStage::Sync,
        storage::SaveStage::Replace,
    ] {
        assert!(
            storage::save_checked(&expected, &path, |s| if s == stage {
                Err("fault".into())
            } else {
                Ok(())
            })
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    storage::save(&expected, &path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), expected);
}

#[test]
fn native_rejects_unknown_styles_and_bad_overrides_legacy_import_keeps_appearance() {
    let mut e = Editor::default();
    e.replace(0, 0, "Text", "Input").unwrap();
    let encoded = storage::encode(&e.document).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    for (pointer, value) in [
        ("/content_version", serde_json::json!(12)),
        ("/format", serde_json::json!("breakfree.sheets.native")),
        ("/paragraphs/0/paragraph_style", serde_json::json!("custom")),
        ("/paragraphs/0/runs/0/direct/bold", serde_json::json!(1)),
        (
            "/paragraphs/0/runs/0/direct/font_size",
            serde_json::json!(801),
        ),
    ] {
        let mut bad = json.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(storage::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    for (from, to) in [
        (
            "\"paragraph_style\": \"normal\"",
            "\"paragraph_style\": \"normal\", \"paragraph_style\": \"heading1\"",
        ),
        ("\"bold\": null", "\"bold\": null, \"bold\": true"),
        ("\"direct\": {", "\"direct\": {\"unknown\": true,"),
    ] {
        let bad = String::from_utf8(encoded.clone())
            .unwrap()
            .replacen(from, to, 1);
        assert_ne!(bad.as_bytes(), encoded);
        assert!(storage::decode(bad.as_bytes()).is_err());
    }
    for bytes in [
        include_bytes!("../examples/Welcome.bfdoc").as_slice(),
        include_bytes!("../examples/Formatting.bfdoc").as_slice(),
    ] {
        let doc = storage::decode(bytes).unwrap();
        let p = &doc.paragraphs[0];
        let original = effective(p, 0);
        assert!(p.styles[0].bold.is_some() && p.styles[0].font_size.is_some());
        let mut e = Editor::new(doc);
        e.apply_paragraph_style(0, 0, ParagraphStyle::Heading1)
            .unwrap();
        assert_eq!(effective(&e.document.paragraphs[0], 0), original);
        e.clear_character_formatting(0, 1).unwrap();
        assert_eq!(
            effective(&e.document.paragraphs[0], 0),
            ParagraphStyle::Heading1.character_style()
        );
        assert_eq!(
            storage::decode(&storage::encode(&e.document).unwrap()).unwrap(),
            e.document
        );
    }
}

#[test]
fn modify_parent_and_properties_updates_descendants_in_one_exact_undo() {
    let mut e = Editor::default();
    e.replace(0, 0, "Normal\nУкраїна café\nРусский", "Input")
        .unwrap();
    e.apply_paragraph_style(7, 20, ParagraphStyle::NoSpacing)
        .unwrap();
    e.apply_paragraph_style(21, 21, ParagraphStyle::Heading1)
        .unwrap();
    e.emphasize(0, 1, true, false).unwrap();
    e.selection = (7, 20);
    let before = e.document.clone();
    let typing = e.typing_style;
    let depth = e.undo_stack.len();
    let definition = StyleDefinition {
        character: CharacterFormat {
            bold: Some(true),
            italic: Some(true),
            font_size: Some("16".parse().unwrap()),
            ..Default::default()
        },
        line_height_percent: Some(150),
        space_after_half_points: Some(20),
        ..Default::default()
    };
    e.modify_style(ParagraphStyle::Normal, definition).unwrap();
    let after = e.document.clone();
    assert_eq!(after.paragraphs, before.paragraphs);
    assert_eq!(e.selection, (7, 20));
    assert_eq!(e.typing_style, typing);
    assert_eq!(e.undo_stack.len(), depth + 1);
    let body = after.style_catalog.resolve(ParagraphStyle::NoSpacing);
    assert!(body.character.bold && body.character.italic);
    assert_eq!(body.character.font_size.to_string(), "16");
    assert_eq!(
        (body.line_height_percent, body.space_after_half_points),
        (100, 0)
    );
    let heading = after.style_catalog.resolve(ParagraphStyle::Heading1);
    assert_eq!(heading.character.font_size.to_string(), "18");
    assert!(heading.character.italic);
    assert_eq!(
        (heading.line_height_percent, heading.space_after_half_points),
        (150, 20)
    );
    assert!(
        !after.paragraphs[0].styles[0]
            .resolve(
                after
                    .style_catalog
                    .resolve(ParagraphStyle::Normal)
                    .character
            )
            .bold
    );
    e.undo();
    assert_eq!(e.document, before);
    assert_eq!(e.selection, (7, 20));
    e.redo();
    assert_eq!(e.document, after);
    assert_eq!(e.typing_style, typing);
    let definition = StyleDefinition {
        parent: Some(ParagraphStyle::NoSpacing),
        character: CharacterFormat {
            bold: Some(false),
            ..Default::default()
        },
        ..Default::default()
    };
    e.modify_style(ParagraphStyle::Heading1, definition)
        .unwrap();
    let resolved = e.document.style_catalog.resolve(ParagraphStyle::Heading1);
    assert!(!resolved.character.bold);
    assert!(resolved.character.italic);
    assert_eq!(resolved.character.font_size.to_string(), "16");
    assert_eq!(
        (
            resolved.line_height_percent,
            resolved.space_after_half_points
        ),
        (100, 0)
    );
    e.undo();
    assert_eq!(e.document, after);
}

#[test]
fn invalid_cycles_and_noop_definitions_preserve_document_selection_and_redo() {
    let mut e = Editor::default();
    e.replace(0, 0, "Test", "Input").unwrap();
    e.undo();
    let before = e.document.clone();
    for (id, parent) in [
        (ParagraphStyle::Normal, ParagraphStyle::Normal),
        (ParagraphStyle::Normal, ParagraphStyle::Heading1),
    ] {
        let mut definition = *before.style_catalog.get(id).unwrap();
        definition.parent = Some(parent);
        assert!(
            e.modify_style(id, definition)
                .unwrap_err()
                .contains("inherit")
        );
        assert_eq!(e.document, before);
        assert!(e.undo_stack.is_empty());
        assert_eq!(e.redo_stack.len(), 1);
    }
    e.modify_style(ParagraphStyle::Normal, before.style_catalog.normal)
        .unwrap();
    assert!(e.undo_stack.is_empty());
    assert_eq!(e.redo_stack.len(), 1);
    let mut catalog = before.style_catalog.clone();
    catalog.normal.parent = Some(ParagraphStyle::Heading1);
    catalog.heading1.parent = Some(ParagraphStyle::NoSpacing);
    assert!(catalog.validate().is_err());
    // A valid non-default root is permitted, including reparenting Normal.
    catalog.no_spacing.parent = None;
    assert!(catalog.validate().is_ok());
}

#[test]
fn editable_catalog_survives_text_bulk_edits_and_invalidates_old_preview() {
    let mut e = Editor::default();
    e.replace(0, 0, "one\none", "Input").unwrap();
    let plan = e.preview_replace_all(&[(0, 3), (4, 7)], "two").unwrap();
    let definition = StyleDefinition {
        character: CharacterFormat {
            italic: Some(true),
            ..Default::default()
        },
        ..Default::default()
    };
    e.modify_style(ParagraphStyle::Normal, definition).unwrap();
    assert!(e.apply_replace_all(plan).is_err());
    let catalog = e.document.style_catalog.clone();
    e.replace(0, 0, "new\n", "Input").unwrap();
    assert_eq!(e.document.style_catalog, catalog);
    let plan = e
        .preview_replace_all(&[(4, 7), (8, 11)], "two\nthree")
        .unwrap();
    e.apply_replace_all(plan).unwrap();
    assert_eq!(e.document.style_catalog, catalog);
    assert_eq!(
        storage::decode(&storage::encode(&e.document).unwrap()).unwrap(),
        e.document
    );
}

#[test]
fn catalog_validation_and_failed_save_preserve_previous_document() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("styles.bfdoc");
    let mut e = Editor::default();
    e.modify_style(
        ParagraphStyle::Heading1,
        StyleDefinition {
            parent: Some(ParagraphStyle::NoSpacing),
            character: CharacterFormat {
                underline: Some(true),
                bold: Some(false),
                font_size: Some("12.5".parse().unwrap()),
                ..Default::default()
            },
            line_height_percent: Some(175),
            space_before_half_points: None,
            space_after_half_points: Some(1),
            next_style: None,
        },
    )
    .unwrap();
    storage::save(&e.document, &path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), e.document);
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    for (pointer, value) in [
        (
            "/style_catalog/normal/parent",
            serde_json::json!("heading1"),
        ),
        (
            "/style_catalog/heading1/parent",
            serde_json::json!("unknown"),
        ),
        (
            "/style_catalog/no_spacing/line_height_percent",
            serde_json::json!(99),
        ),
        (
            "/style_catalog/normal/space_after_half_points",
            serde_json::json!(401),
        ),
        (
            "/style_catalog/normal/character/font_size",
            serde_json::json!(0),
        ),
        ("/style_catalog", serde_json::json!({})),
    ] {
        let mut bad = json.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(
            storage::decode(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "{pointer}"
        );
    }
    for (from, to) in [
        ("\"normal\": {", "\"normal\": {}, \"normal\": {"),
        ("\"parent\": null", "\"parent\": null, \"parent\": null"),
        (
            "\"style_catalog\": {",
            "\"style_catalog\": {\"custom\": {},",
        ),
    ] {
        assert!(
            storage::decode(
                String::from_utf8(bytes.clone())
                    .unwrap()
                    .replacen(from, to, 1)
                    .as_bytes()
            )
            .is_err()
        );
    }
    let mut bad = json.clone();
    bad.as_object_mut().unwrap().remove("style_catalog");
    assert!(storage::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    e.document.style_catalog.normal.character.italic = Some(true);
    for stage in [
        storage::SaveStage::Write,
        storage::SaveStage::Sync,
        storage::SaveStage::Replace,
    ] {
        assert!(
            storage::save_checked(&e.document, &path, |at| if at == stage {
                Err("Injected failure".into())
            } else {
                Ok(())
            })
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    e.document.style_catalog.normal.parent = Some(ParagraphStyle::Normal);
    assert!(storage::save(&e.document, &path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}

#[test]
fn v3_import_freezes_defaults_and_does_not_modify_source_bytes() {
    let bytes = include_bytes!("../examples/Styles.bfdoc");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v3.bfdoc");
    std::fs::write(&path, bytes).unwrap();
    let doc = storage::load(&path).unwrap();
    assert_eq!(doc.style_catalog, StyleCatalog::default());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let mut v3: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    v3["style_catalog"] = serde_json::json!({});
    assert!(storage::decode(&serde_json::to_vec(&v3).unwrap()).is_err());
    assert_eq!(
        storage::decode(&storage::encode(&doc).unwrap()).unwrap(),
        doc
    );
}
