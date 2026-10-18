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
    assert_eq!(json["content_version"], 10);
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
