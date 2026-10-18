use breakfree_docs::{commands::Editor, formatting, model::*, storage};

#[test]
fn spacing_transaction_inheritance_selection_and_history() {
    let mut e = Editor::default();
    e.replace(0, 0, "e\u{301}\nУкраїна\nРусский", "Fixture")
        .unwrap();
    e.document.style_catalog.normal.line_height_percent = Some(125);
    e.document.style_catalog.normal.space_after_half_points = Some(11);
    let before = e.document.clone();
    let depth = e.undo_stack.len();
    // Exclusive end at the start of paragraph two excludes it.
    e.set_paragraph_spacing(0, 3, Some(Some(200)), Some(Some(0)))
        .unwrap();
    assert_eq!(e.undo_stack.len(), depth + 1);
    assert_eq!(e.document.paragraphs[1..], before.paragraphs[1..]);
    let p = &e.document.paragraphs[0];
    assert_eq!(e.document.resolved_paragraph(p).space_after_half_points, 0);
    assert_eq!(
        formatting::inspect_paragraph(&e.document, p)[1].source,
        "Direct paragraph formatting"
    );
    let after = e.document.clone();
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
    e.set_paragraph_spacing(0, 0, Some(None), Some(None))
        .unwrap();
    assert_eq!(e.document, before);
    e.undo();
    assert_eq!(e.document, after);
    e.redo();
    assert_eq!(e.document, before);
    let redo = e.redo_stack.len();
    let depth = e.undo_stack.len();
    e.set_paragraph_spacing(0, 0, None, None).unwrap();
    assert_eq!((e.undo_stack.len(), e.redo_stack.len()), (depth, redo));
}

#[test]
fn spacing_validation_is_transactional_and_preserves_redo() {
    let mut e = Editor::default();
    e.set_paragraph_spacing(0, 0, Some(Some(150)), None)
        .unwrap();
    e.undo();
    let before = e.document.clone();
    for (line, after) in [(99, 0), (301, 0), (150, 401)] {
        assert!(
            e.set_paragraph_spacing(0, 0, Some(Some(line)), Some(Some(after)))
                .is_err()
        );
        assert_eq!(e.document, before);
        assert_eq!(e.redo_stack.len(), 1);
    }
    e.set_paragraph_spacing(0, 0, None, None).unwrap();
    assert_eq!(e.redo_stack.len(), 1);
    e.replace(0, 0, "e\u{301}", "Fixture").unwrap();
    let before = e.document.clone();
    assert!(
        e.set_paragraph_spacing(1, 1, Some(Some(150)), None)
            .is_err()
    );
    assert_eq!(e.document, before);
}

#[test]
fn spacing_survives_enter_style_change_join_and_batch_replacement() {
    let mut e = Editor::default();
    e.replace(0, 0, "ab\ncd", "Fixture").unwrap();
    e.set_paragraph_spacing(0, 0, Some(Some(180)), Some(Some(17)))
        .unwrap();
    let spacing = e.document.paragraphs[0].spacing;
    e.apply_paragraph_style(0, 0, ParagraphStyle::Heading1)
        .unwrap();
    e.insert_paragraph_break(1, 1).unwrap();
    assert_eq!(e.document.paragraphs[0].spacing, spacing);
    assert_eq!(e.document.paragraphs[1].spacing, spacing);
    e.replace(1, 2, "", "Join").unwrap();
    assert_eq!(e.document.paragraphs[0].spacing, spacing);
    let before = e.document.clone();
    let plan = e.preview_replace_all(&[(0, 1)], "a\nx").unwrap();
    e.apply_replace_all(plan).unwrap();
    assert_eq!(e.document.paragraphs[0].spacing, spacing);
    assert_eq!(e.document.paragraphs[1].spacing, spacing);
    assert_eq!(
        e.document.paragraphs[2].spacing,
        ParagraphSpacing::default()
    );
    e.undo();
    assert_eq!(e.document, before);
}

#[test]
fn v10_spacing_roundtrip_migration_strictness_and_failed_write() {
    let old = include_bytes!("../examples/StyleNavigation.bfdoc");
    let mut e = Editor::new(storage::decode(old).unwrap());
    assert!(
        e.document
            .paragraphs
            .iter()
            .all(|p| p.spacing == ParagraphSpacing::default())
    );
    e.set_paragraph_spacing(0, 0, Some(Some(175)), Some(Some(0)))
        .unwrap();
    let bytes = storage::encode(&e.document).unwrap();
    assert_eq!(storage::decode(&bytes).unwrap(), e.document);
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["content_version"], 12);
    for value in [
        serde_json::json!({"line_height_percent":99}),
        serde_json::json!({"space_after_half_points":401}),
        serde_json::json!({"unknown":0}),
    ] {
        let mut bad = json.clone();
        bad["paragraphs"][0]["spacing"] = value;
        assert!(storage::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let mut bad = json.clone();
    bad["content_version"] = 9.into();
    assert!(storage::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    let mut missing = json.clone();
    missing["paragraphs"][0]
        .as_object_mut()
        .unwrap()
        .remove("spacing");
    assert!(storage::decode(&serde_json::to_vec(&missing).unwrap()).is_err());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("spacing.bfdoc");
    storage::save(&e.document, &path).unwrap();
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
    assert_eq!(storage::load(&path).unwrap(), e.document);
}

#[test]
fn mixed_spacing_partial_patch_preserves_other_property_and_empty_paragraphs() {
    let mut e = Editor::default();
    e.replace(0, 0, "a\n\nb", "Fixture").unwrap();
    e.set_paragraph_spacing(0, 0, Some(Some(150)), Some(Some(10)))
        .unwrap();
    e.set_paragraph_spacing(2, 2, Some(Some(180)), Some(Some(20)))
        .unwrap();
    let before = e.document.clone();
    e.set_paragraph_spacing(0, 3, Some(None), None).unwrap();
    assert_eq!(e.document.paragraphs[0].spacing.line_height_percent, None);
    assert_eq!(e.document.paragraphs[1].spacing.line_height_percent, None);
    assert_eq!(
        e.document.paragraphs[0].spacing.space_after_half_points,
        Some(10)
    );
    assert_eq!(
        e.document.paragraphs[1].spacing.space_after_half_points,
        Some(20)
    );
    assert_eq!(e.document.paragraphs[2], before.paragraphs[2]);
    e.undo();
    assert_eq!(e.document, before);
}

#[test]
fn before_cascades_through_styles_direct_zero_and_single_history_reset() {
    let mut e = Editor::default();
    e.replace(0, 0, "English\nУкраїна e\u{301}\nРусский", "Fixture")
        .unwrap();
    let mut normal = e.document.style_catalog.normal;
    normal.space_before_half_points = Some(15);
    e.modify_style(ParagraphStyle::Normal, normal).unwrap();
    e.apply_paragraph_style(0, 0, ParagraphStyle::Heading1)
        .unwrap();
    assert_eq!(
        e.document
            .resolved_paragraph(&e.document.paragraphs[0])
            .space_before_half_points,
        15
    );
    assert_eq!(
        formatting::inspect_paragraph(&e.document, &e.document.paragraphs[0])[3].source,
        "Inherited paragraph style: Normal"
    );
    let before = e.document.clone();
    let depth = e.undo_stack.len();
    e.set_paragraph_spacing_with_before(0, 8, Some(Some(150)), Some(Some(0)), Some(Some(21)))
        .unwrap();
    assert_eq!(e.undo_stack.len(), depth + 1);
    assert_eq!(e.document.paragraphs[1..], before.paragraphs[1..]);
    assert_eq!(
        e.document
            .resolved_paragraph(&e.document.paragraphs[0])
            .space_before_half_points,
        0
    );
    assert_eq!(
        formatting::inspect_paragraph(&e.document, &e.document.paragraphs[0])[3].source,
        "Direct paragraph formatting"
    );
    let changed = e.document.clone();
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, changed);
    e.set_paragraph_spacing_with_before(0, 0, None, Some(None), None)
        .unwrap();
    assert_eq!(
        e.document
            .resolved_paragraph(&e.document.paragraphs[0])
            .space_before_half_points,
        15
    );
    assert_eq!(
        e.document.paragraphs[0].spacing.space_after_half_points,
        Some(21)
    );
    e.undo();
    assert_eq!(e.document, changed);
    e.redo();
    normal.space_before_half_points = Some(30);
    e.modify_style(ParagraphStyle::Normal, normal).unwrap();
    assert_eq!(
        e.document
            .resolved_paragraph(&e.document.paragraphs[0])
            .space_before_half_points,
        30
    );
}

#[test]
fn before_invalid_noop_empty_and_mixed_ranges_preserve_history() {
    let mut e = Editor::default();
    e.replace(0, 0, "a\n\nb", "Fixture").unwrap();
    e.set_paragraph_spacing_with_before(0, 0, None, Some(Some(10)), None)
        .unwrap();
    e.set_paragraph_spacing_with_before(2, 2, None, Some(Some(20)), None)
        .unwrap();
    let before = e.document.clone();
    let depth = e.undo_stack.len();
    e.set_paragraph_spacing_with_before(0, 3, None, Some(None), None)
        .unwrap();
    assert_eq!(e.undo_stack.len(), depth + 1);
    assert_eq!(
        e.document.paragraphs[0].spacing.space_before_half_points,
        None
    );
    assert_eq!(
        e.document.paragraphs[1].spacing.space_before_half_points,
        None
    );
    assert_eq!(e.document.paragraphs[2], before.paragraphs[2]);
    e.undo();
    assert_eq!(e.document, before);
    assert!(
        e.set_paragraph_spacing_with_before(0, 3, None, Some(Some(401)), None)
            .is_err()
    );
    assert_eq!(e.document, before);
    assert_eq!(e.redo_stack.len(), 1);
    e.set_paragraph_spacing_with_before(0, 0, None, Some(Some(10)), None)
        .unwrap();
    assert_eq!(e.redo_stack.len(), 1);
    let mut style = e.document.style_catalog.normal;
    style.space_before_half_points = Some(401);
    assert!(e.modify_style(ParagraphStyle::Normal, style).is_err());
    assert_eq!(e.document, before);
}

#[test]
fn before_survives_enter_join_replace_all_and_v11_save_failures() {
    let mut e = Editor::default();
    e.replace(0, 0, "ab\ncd", "Fixture").unwrap();
    e.set_paragraph_spacing_with_before(0, 0, None, Some(Some(17)), None)
        .unwrap();
    e.insert_paragraph_break(1, 1).unwrap();
    assert_eq!(
        e.document.paragraphs[1].spacing.space_before_half_points,
        Some(17)
    );
    e.replace(1, 2, "", "Join").unwrap();
    let before = e.document.clone();
    let plan = e.preview_replace_all(&[(0, 1)], "a\nx").unwrap();
    e.apply_replace_all(plan).unwrap();
    assert_eq!(
        e.document.paragraphs[1].spacing.space_before_half_points,
        Some(17)
    );
    assert_eq!(
        e.document.paragraphs[2].spacing.space_before_half_points,
        None
    );
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("before.bfdoc");
    storage::save(&e.document, &path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
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
fn legacy_spacing_migrates_with_zero_before_and_rejects_new_fields() {
    let mut old: serde_json::Value =
        serde_json::from_slice(include_bytes!("../examples/StyleNavigation.bfdoc")).unwrap();
    old["content_version"] = 10.into();
    for p in old["paragraphs"].as_array_mut().unwrap() {
        p["spacing"] = serde_json::json!({"line_height_percent":150,"space_after_half_points":17});
    }
    let bytes = serde_json::to_vec(&old).unwrap();
    let doc = storage::decode(&bytes).unwrap();
    assert!(
        doc.paragraphs
            .iter()
            .all(|p| doc.resolved_paragraph(p).space_before_half_points == 0
                && p.spacing.space_after_half_points == Some(17))
    );
    for version in 6..=10 {
        for path in ["style", "direct"] {
            let mut bad = old.clone();
            bad["content_version"] = version.into();
            if path == "style" {
                bad["style_catalog"]["normal"]["space_before_half_points"] =
                    serde_json::Value::Null;
            } else {
                bad["paragraphs"][0]["spacing"]["space_before_half_points"] = 0.into();
            }
            assert!(
                storage::decode(&serde_json::to_vec(&bad).unwrap())
                    .unwrap_err()
                    .contains("requires native content v11")
            );
        }
    }
    let mut current: serde_json::Value =
        serde_json::from_slice(&storage::encode(&doc).unwrap()).unwrap();
    assert_eq!(current["content_version"], 12);
    current["paragraphs"][0]["spacing"]["space_before_half_points"] = 401.into();
    assert!(storage::decode(&serde_json::to_vec(&current).unwrap()).is_err());
    current["paragraphs"][0]["spacing"]["space_before_half_points"] = 0.into();
    current["style_catalog"]["normal"]["space_before_half_points"] = 401.into();
    assert!(storage::decode(&serde_json::to_vec(&current).unwrap()).is_err());
}
