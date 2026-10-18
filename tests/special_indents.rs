use breakfree_docs::{
    commands::{Editor, ParagraphGeometryPatch},
    formatting,
    model::*,
    storage,
};
fn patch(value: Option<i16>) -> ParagraphGeometryPatch {
    ParagraphGeometryPatch {
        first_line: Some(value),
        ..Default::default()
    }
}
#[test]
fn special_indent_inherits_overrides_resets_and_undoes_once() {
    let mut e = Editor::default();
    e.replace(0, 0, "e\u{301}\nУкраїна\nРусский", "Fixture")
        .unwrap();
    e.document.style_catalog.normal.first_line_half_points = Some(-36);
    e.document.paragraphs[0].paragraph_style = ParagraphStyle::Heading1;
    let before = e.document.clone();
    let depth = e.undo_stack.len();
    assert_eq!(
        e.document
            .resolved_paragraph(&e.document.paragraphs[0])
            .first_line_half_points,
        -36
    );
    assert!(
        formatting::inspect_paragraph(&e.document, &e.document.paragraphs[0])[6]
            .source
            .contains("Inherited")
    );
    e.set_paragraph_geometry(0, 3, patch(Some(0))).unwrap();
    assert_eq!(e.undo_stack.len(), depth + 1);
    assert_eq!(e.document.paragraphs[1..], before.paragraphs[1..]);
    assert_eq!(
        e.document
            .resolved_paragraph(&e.document.paragraphs[0])
            .first_line_half_points,
        0
    );
    assert_eq!(
        formatting::inspect_paragraph(&e.document, &e.document.paragraphs[0])[6].source,
        "Direct paragraph formatting"
    );
    let after = e.document.clone();
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
    e.set_paragraph_geometry(0, 0, patch(None)).unwrap();
    assert_eq!(e.document, before);
    e.undo();
    assert_eq!(e.document, after);
    e.redo();
    let mut style = e.document.style_catalog.normal;
    style.first_line_half_points = Some(51);
    let depth = e.undo_stack.len();
    e.modify_style(ParagraphStyle::Normal, style).unwrap();
    assert_eq!(e.undo_stack.len(), depth + 1);
    assert_eq!(
        e.document
            .resolved_paragraph(&e.document.paragraphs[0])
            .first_line_half_points,
        51
    );
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
}
#[test]
fn special_indent_validation_noop_empty_and_mixed_selection() {
    let mut e = Editor::default();
    e.replace(0, 0, "a\n\nb", "Fixture").unwrap();
    e.set_paragraph_geometry(0, 0, patch(Some(12))).unwrap();
    e.set_paragraph_geometry(2, 2, patch(Some(-12))).unwrap();
    let before = e.document.clone();
    e.set_paragraph_geometry(0, 3, patch(Some(-20))).unwrap();
    assert_eq!(e.document.paragraphs[2], before.paragraphs[2]);
    e.undo();
    assert_eq!(e.document, before);
    let depth = e.undo_stack.len();
    let redo = e.redo_stack.len();
    for v in [i16::MIN, -289, 289, i16::MAX] {
        assert!(e.set_paragraph_geometry(0, 0, patch(Some(v))).is_err());
        let mut d = e.document.style_catalog.normal;
        d.first_line_half_points = Some(v);
        assert!(e.modify_style(ParagraphStyle::Normal, d).is_err());
    }
    e.set_paragraph_geometry(0, 0, patch(Some(12))).unwrap();
    assert_eq!(e.undo_stack.len(), depth);
    assert_eq!(e.redo_stack.len(), redo);
    assert_eq!(e.document, before);
}
#[test]
fn special_indent_editing_roundtrip_legacy_and_failed_save() {
    let mut e = Editor::default();
    e.replace(0, 0, "One\nTwo", "Fixture").unwrap();
    e.set_paragraph_geometry(0, 0, patch(Some(-37))).unwrap();
    let original = e.document.clone();
    e.replace(1, 1, "\n", "Enter").unwrap();
    assert_eq!(
        e.document.paragraphs[0].spacing,
        e.document.paragraphs[1].spacing
    );
    e.undo();
    assert_eq!(e.document, original);
    e.redo();
    e.replace(1, 2, "", "Join").unwrap();
    assert_eq!(e.document, original);
    let preview = e.preview_replace_all(&[(0, 3)], "Alpha\nBeta").unwrap();
    e.apply_replace_all(preview).unwrap();
    assert_eq!(
        e.document.paragraphs[1].spacing.first_line_half_points,
        Some(-37)
    );
    assert_eq!(
        e.document.paragraphs[2].spacing.first_line_half_points,
        None
    );
    e.undo();
    assert_eq!(e.document, original);
    e.redo();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("special.bfdoc");
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
    assert_eq!(json["content_version"], 13);
    for version in 1..13 {
        json["content_version"] = version.into();
        assert!(
            storage::decode(&serde_json::to_vec(&json).unwrap())
                .unwrap_err()
                .contains("requires native content v13")
        );
    }
    json["content_version"] = 13.into();
    json["paragraphs"][0]["spacing"]["first_line_half_points"] = (-289).into();
    assert!(storage::decode(&serde_json::to_vec(&json).unwrap()).is_err());
    let mut old: serde_json::Value =
        serde_json::from_slice(&storage::encode(&Document::default()).unwrap()).unwrap();
    old["content_version"] = 12.into();
    let imported = storage::decode(&serde_json::to_vec(&old).unwrap()).unwrap();
    assert_eq!(
        imported
            .resolved_paragraph(&imported.paragraphs[0])
            .first_line_half_points,
        0
    );
    old["style_catalog"]["normal"]["first_line_half_points"] = serde_json::Value::Null;
    assert!(storage::decode(&serde_json::to_vec(&old).unwrap()).is_err());
}
