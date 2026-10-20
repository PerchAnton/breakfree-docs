use breakfree_docs::{
    commands::{Editor, ParagraphGeometryPatch},
    formatting,
    model::*,
    storage,
};

#[test]
fn indent_cascade_partial_selection_reset_and_history() {
    let mut e = Editor::default();
    e.replace(0, 0, "e\u{301}\nУкраїна\nРусский", "Fixture")
        .unwrap();
    e.document.style_catalog.normal.left_indent_half_points = Some(36);
    e.document.style_catalog.normal.right_indent_half_points = Some(24);
    e.document.paragraphs[0].paragraph_style = ParagraphStyle::Heading1;
    let before = e.document.clone();
    let depth = e.undo_stack.len();
    assert_eq!(
        e.document
            .resolved_paragraph(&e.document.paragraphs[0])
            .left_indent_half_points,
        36
    );
    assert!(
        formatting::inspect_paragraph(&e.document, &e.document.paragraphs[0])[4]
            .source
            .contains("Inherited")
    );
    e.set_paragraph_geometry(
        0,
        3,
        ParagraphGeometryPatch {
            left: Some(Some(0)),
            right: Some(Some(51)),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(e.undo_stack.len(), depth + 1);
    assert_eq!(e.document.paragraphs[1..], before.paragraphs[1..]);
    let after = e.document.clone();
    assert_eq!(
        formatting::inspect_paragraph(&after, &after.paragraphs[0])[4].source,
        "Direct paragraph formatting"
    );
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
    e.set_paragraph_geometry(
        0,
        0,
        ParagraphGeometryPatch {
            left: Some(None),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        e.document
            .resolved_paragraph(&e.document.paragraphs[0])
            .left_indent_half_points,
        36
    );
    assert_eq!(
        e.document.paragraphs[0].spacing.right_indent_half_points,
        Some(51)
    );
    e.undo();
    assert_eq!(e.document, after);
    e.redo();
    let snapshot = e.document.clone();
    for (start, end, value) in [(0, 0, 289), (1, 2, 12), (0, 999, 12)] {
        assert!(
            e.set_paragraph_geometry(
                start,
                end,
                ParagraphGeometryPatch {
                    left: Some(Some(value)),
                    ..Default::default()
                }
            )
            .is_err()
        );
        assert_eq!(e.document, snapshot);
    }
    let mut definition = e.document.style_catalog.normal;
    definition.left_indent_half_points = Some(60);
    let depth = e.undo_stack.len();
    e.modify_style(ParagraphStyle::Normal, definition).unwrap();
    assert_eq!(e.undo_stack.len(), depth + 1);
    assert_eq!(
        e.document
            .resolved_paragraph(&e.document.paragraphs[0])
            .left_indent_half_points,
        60
    );
    e.undo();
    assert_eq!(e.document, snapshot);
    e.redo();
}

#[test]
fn indent_edit_preservation_roundtrip_and_legacy_gate() {
    let mut e = Editor::default();
    e.replace(0, 0, "One\nTwo", "Fixture").unwrap();
    e.set_paragraph_geometry(
        0,
        0,
        ParagraphGeometryPatch {
            left: Some(Some(37)),
            right: Some(Some(12)),
            ..Default::default()
        },
    )
    .unwrap();
    e.replace(1, 1, "\n", "Enter").unwrap();
    assert_eq!(
        e.document.paragraphs[0].spacing,
        e.document.paragraphs[1].spacing
    );
    e.replace(1, 2, "", "Join").unwrap();
    assert_eq!(
        e.document.paragraphs[0].spacing.left_indent_half_points,
        Some(37)
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("indents.bfdoc");
    storage::save(&e.document, &path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), e.document);
    let bytes = std::fs::read(&path).unwrap();
    for stage in [
        storage::SaveStage::Write,
        storage::SaveStage::Sync,
        storage::SaveStage::Replace,
    ] {
        assert!(
            storage::save_checked(&e.document, &path, |s| if s == stage {
                Err("injected".into())
            } else {
                Ok(())
            })
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    let mut json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["content_version"], 14);
    for version in 6..12 {
        json["content_version"] = version.into();
        assert!(storage::decode(&serde_json::to_vec(&json).unwrap()).is_err());
    }
    let mut old: serde_json::Value =
        serde_json::from_slice(&storage::encode(&Document::default()).unwrap()).unwrap();
    old["content_version"] = 11.into();
    let imported = storage::decode(&serde_json::to_vec(&old).unwrap()).unwrap();
    assert_eq!(
        imported
            .resolved_paragraph(&imported.paragraphs[0])
            .left_indent_half_points,
        0
    );
    old["style_catalog"]["normal"]["right_indent_half_points"] = serde_json::Value::Null;
    assert!(
        storage::decode(&serde_json::to_vec(&old).unwrap())
            .unwrap_err()
            .contains("require native content v12")
    );
    json["content_version"] = 13.into();
    json["paragraphs"][0]["spacing"]["left_indent_half_points"] = 289.into();
    assert!(storage::decode(&serde_json::to_vec(&json).unwrap()).is_err());
}

#[test]
fn indent_parser_and_noop_preserve_redo() {
    for s in ["-1", "NaN", "145", "144.5", "0.25", "999999", "1.2.3"] {
        assert!(parse_indent(s).is_err(), "{s}");
    }
    assert_eq!(parse_indent(" 12.5 ").unwrap(), Some(25));
    assert_eq!(parse_indent("").unwrap(), None);
    let mut e = Editor::default();
    e.set_paragraph_geometry(
        0,
        0,
        ParagraphGeometryPatch {
            left: Some(Some(1)),
            ..Default::default()
        },
    )
    .unwrap();
    e.undo();
    let redo = e.redo_stack.len();
    e.set_paragraph_geometry(0, 0, ParagraphGeometryPatch::default())
        .unwrap();
    assert_eq!(e.redo_stack.len(), redo);
}

#[test]
fn stepped_indents_preserve_mixed_geometry_inheritance_history_and_storage() {
    let mut e = Editor::default();
    e.replace(0, 0, "e\u{301}\nУкраїна\nРусский", "Fixture")
        .unwrap();
    e.document.style_catalog.normal.left_indent_half_points = Some(36);
    e.document.paragraphs[1].spacing.left_indent_half_points = Some(270);
    e.document.paragraphs[0].spacing.first_line_half_points = Some(-24);
    e.document.paragraphs[0].spacing.right_indent_half_points = Some(48);
    let before = e.document.clone();
    let n = e.undo_stack.len();
    let end = e.document.text().chars().count();
    e.step_paragraph_indent(0, end, true).unwrap();
    assert_eq!(
        e.document.paragraphs[0].spacing.left_indent_half_points,
        Some(108)
    );
    assert_eq!(
        e.document.paragraphs[1].spacing.left_indent_half_points,
        Some(288)
    );
    assert_eq!(
        e.document.paragraphs[0].spacing.first_line_half_points,
        Some(-24)
    );
    assert_eq!(
        e.document.paragraphs[0].spacing.right_indent_half_points,
        Some(48)
    );
    assert_eq!(e.undo_stack.len(), n + 1);
    let after = e.document.clone();
    assert_eq!(
        storage::decode(&storage::encode(&after).unwrap()).unwrap(),
        after
    );
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
    e.step_paragraph_indent(0, 3, false).unwrap();
    assert_eq!(
        e.document.paragraphs[0].spacing.left_indent_half_points,
        Some(36)
    );
    assert_eq!(e.document.paragraphs[1..], after.paragraphs[1..]);
}

#[test]
fn stepped_indent_noop_preserves_inheritance_redo_and_rejects_invalid_range() {
    let mut e = Editor::default();
    e.replace(0, 0, "text", "Fixture").unwrap();
    e.undo();
    let redo = e.redo_stack.len();
    e.step_paragraph_indent(0, 0, false).unwrap();
    assert_eq!(e.redo_stack.len(), redo);
    assert_eq!(
        e.document.paragraphs[0].spacing.left_indent_half_points,
        None
    );
    e.document.style_catalog.normal.left_indent_half_points = Some(288);
    e.step_paragraph_indent(0, 0, true).unwrap();
    assert_eq!(
        e.document.paragraphs[0].spacing.left_indent_half_points,
        None
    );
    let before = e.document.clone();
    assert!(e.step_paragraph_indent(1, 0, true).is_err());
    assert_eq!(e.document, before);
}
