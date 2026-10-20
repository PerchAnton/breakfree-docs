use breakfree_docs::{commands::Editor, model::*, storage};

fn configured() -> Editor {
    let mut e = Editor::default();
    e.replace(0, 0, "English Україна Русский cafe\u{301}", "Input")
        .unwrap();
    let mut d = *e
        .document
        .style_catalog
        .get(ParagraphStyle::Heading1)
        .unwrap();
    d.next_style = Some(ParagraphStyle::NoSpacing);
    e.modify_style(ParagraphStyle::Heading1, d).unwrap();
    e.apply_paragraph_style(0, 0, ParagraphStyle::Heading1)
        .unwrap();
    e
}
#[test]
fn enter_at_start_middle_end_and_selection_is_one_exact_transaction() {
    for (start, end) in [(0, 0), (8, 8), (8, 15), (0, 33), (33, 33)] {
        let mut e = configured();
        let len = e.document.text().chars().count();
        let (start, end) = (start.min(len), end.min(len));
        // Use the final grapheme boundary, never the combining mark boundary.
        let end = if end == len - 1 { len } else { end };
        let start = if start == len - 1 { len } else { start };
        e.selection = (start, end);
        e.typing_style.bold = Some(false);
        let before = e.document.clone();
        let depth = e.undo_stack.len();
        let pending = e.typing_style;
        e.insert_paragraph_break(start, end).unwrap();
        assert_eq!(e.undo_stack.len(), depth + 1);
        assert_eq!(
            e.document.paragraphs[0].paragraph_style,
            ParagraphStyle::Heading1
        );
        assert_eq!(
            e.document.paragraphs[1].paragraph_style,
            ParagraphStyle::NoSpacing
        );
        assert_eq!(e.typing_style, pending);
        let after = e.document.clone();
        assert_eq!(
            storage::decode(&storage::encode(&after).unwrap()).unwrap(),
            after
        );
        e.undo();
        assert_eq!(e.document, before);
        assert_eq!(e.selection, (start, end));
        e.redo();
        assert_eq!(e.document, after);
        assert_eq!(e.selection, (start + 1, start + 1));
    }
}
#[test]
fn successor_is_not_inherited_and_sequences_may_cycle() {
    let mut e = configured();
    let child = e
        .create_style(
            "Child",
            StyleDefinition {
                parent: Some(ParagraphStyle::Heading1),
                ..Default::default()
            },
            true,
        )
        .unwrap();
    e.insert_paragraph_break(e.selection.0, e.selection.1)
        .unwrap();
    assert_eq!(e.document.paragraphs.last().unwrap().paragraph_style, child);
    let mut d = *e.document.style_catalog.get(child).unwrap();
    d.next_style = Some(ParagraphStyle::Heading1);
    e.modify_style(child, d).unwrap();
    let mut h = *e
        .document
        .style_catalog
        .get(ParagraphStyle::Heading1)
        .unwrap();
    h.next_style = Some(child);
    e.modify_style(ParagraphStyle::Heading1, h).unwrap();
    e.insert_paragraph_break(e.selection.0, e.selection.1)
        .unwrap();
    assert_eq!(
        e.document.paragraphs[2].paragraph_style,
        ParagraphStyle::Heading1
    );
    e.insert_paragraph_break(e.selection.0, e.selection.1)
        .unwrap();
    assert_eq!(e.document.paragraphs[3].paragraph_style, child);
    let before = e.document.clone();
    e.undo();
    let redo = e.redo_stack.len();
    d.next_style = Some(ParagraphStyle::Custom(999));
    assert!(e.modify_style(child, d).is_err());
    assert_eq!(e.redo_stack.len(), redo);
    e.redo();
    assert_eq!(e.document, before);
}
#[test]
fn paste_does_not_apply_following_style_and_invalid_unicode_does_not_mutate() {
    let mut e = configured();
    e.replace(0, 0, "a\nb\n", "Paste").unwrap();
    assert!(
        e.document
            .paragraphs
            .iter()
            .all(|p| p.paragraph_style == ParagraphStyle::Heading1)
    );
    let before = e.document.clone();
    let depth = e.undo_stack.len();
    let bad = e.document.text().chars().count() - 1;
    assert!(e.insert_paragraph_break(bad, bad).is_err());
    assert_eq!(e.document, before);
    assert_eq!(e.undo_stack.len(), depth);
}
#[test]
fn migration_is_strict_and_deleted_successor_is_remapped_and_undoable() {
    let old = include_bytes!("../examples/StyleDeletion.bfdoc");
    let mut e = Editor::new(storage::decode(old).unwrap());
    assert!(e.document.style_catalog.ids().iter().all(|id| {
        e.document
            .style_catalog
            .get(*id)
            .unwrap()
            .next_style
            .is_none()
    }));
    let mut d = *e
        .document
        .style_catalog
        .get(ParagraphStyle::Normal)
        .unwrap();
    d.next_style = Some(ParagraphStyle::Custom(1));
    e.modify_style(ParagraphStyle::Normal, d).unwrap();
    let before = e.document.clone();
    e.delete_custom_style(
        ParagraphStyle::Custom(1),
        ParagraphStyle::NoSpacing,
        Some(ParagraphStyle::Heading1),
    )
    .unwrap();
    assert_eq!(
        e.document.style_catalog.normal.next_style,
        Some(ParagraphStyle::NoSpacing)
    );
    e.undo();
    assert_eq!(e.document, before);
    let bytes = storage::encode(&before).unwrap();
    let mut json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["content_version"], 14);
    json["content_version"] = 5.into();
    assert!(storage::decode(&serde_json::to_vec(&json).unwrap()).is_err());
    json["content_version"] = 13.into();
    json["style_catalog"]["normal"]["next_style"] = serde_json::json!({"custom":999});
    assert!(storage::decode(&serde_json::to_vec(&json).unwrap()).is_err());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("following.bfdoc");
    storage::save(&before, &path).unwrap();
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
        assert_eq!(storage::load(&path).unwrap(), before);
    }
}

#[test]
fn empty_and_cross_paragraph_enter_preserve_alignment_direct_overrides_and_dirty_checkpoint() {
    use breakfree_docs::session::Session;
    let mut s = Session::default();
    let mut d = s.editor.document.style_catalog.normal;
    d.next_style = Some(ParagraphStyle::Heading1);
    s.editor.modify_style(ParagraphStyle::Normal, d).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("checkpoint.bfdoc");
    s.save(&path).unwrap();
    assert!(!s.dirty());
    s.editor.insert_paragraph_break(0, 0).unwrap();
    assert!(s.dirty());
    assert_eq!(
        s.editor.document.paragraphs[1].paragraph_style,
        ParagraphStyle::Heading1
    );
    s.editor.undo();
    assert!(!s.dirty());
    s.editor.redo();
    assert!(s.dirty());
    s.editor.replace(0, 1, "abc\ndef", "Paste").unwrap();
    s.editor.document.paragraphs[0].alignment = Alignment::Center;
    let before = s.editor.document.clone();
    s.editor.insert_paragraph_break(1, 5).unwrap();
    assert_eq!(s.editor.document.text(), "a\nef");
    assert_eq!(
        s.editor.document.paragraphs[1].paragraph_style,
        ParagraphStyle::Heading1
    );
    assert_eq!(s.editor.document.paragraphs[1].alignment, Alignment::Center);
    s.editor.undo();
    assert_eq!(s.editor.document, before);
}
