use breakfree_docs::{
    commands::Editor,
    model::*,
    presentation,
    session::{LeaveChoice, Session},
    storage::{self, SaveStage},
};
use std::fs;

fn sample() -> Editor {
    let mut e = Editor::default();
    e.replace(
        0,
        0,
        "English\nУкраїнська ї є ґ\nРусский ё\ne\u{301} 🙂 👩‍💻\n",
        "Type",
    )
    .unwrap();
    e.emphasize(0, 7, true, true).unwrap();
    e.emphasize(8, 18, false, true).unwrap();
    e.align(8, 18, Alignment::Center).unwrap();
    e
}

#[test]
fn edits_and_formatting_restore_exact_snapshots() {
    let mut e = Editor::default();
    let mut states = vec![e.document.clone()];
    e.replace(0, 0, "Hello\nВітаю\nПривет", "Paste").unwrap();
    states.push(e.document.clone());
    e.emphasize(0, 5, true, true).unwrap();
    states.push(e.document.clone());
    e.emphasize(6, 11, false, true).unwrap();
    states.push(e.document.clone());
    e.align(6, 11, Alignment::Right).unwrap();
    states.push(e.document.clone());
    e.replace(0, 5, "Welcome", "Replace").unwrap();
    states.push(e.document.clone());
    e.replace(7, 8, "", "Delete").unwrap();
    states.push(e.document.clone());
    for state in states[..states.len() - 1].iter().rev() {
        e.undo();
        assert_eq!(&e.document, state);
    }
    for state in &states[1..] {
        e.redo();
        assert_eq!(&e.document, state);
    }
}
#[test]
fn unicode_scalar_and_grapheme_boundaries() {
    for text in [
        "English",
        "Українська ї є ґ",
        "Русский ё",
        "e\u{301}",
        "🙂",
        "👩‍💻",
        "🇺🇦",
    ] {
        let mut e = Editor::default();
        e.replace(0, 0, text, "Input").unwrap();
        for i in 0..=text.chars().count() {
            if !boundaries(text).contains(&i) {
                assert!(e.replace(i, i, "x", "Input").is_err());
                assert!(e.emphasize(0, i, true, true).is_err());
            }
        }
        e.replace(0, text.chars().count(), "new", "Replace")
            .unwrap();
        e.undo();
        assert_eq!(e.document.text(), text);
    }
    assert_eq!(presentation::snap("e\u{301}", 1, 2), (0, 2));
    let mut e = Editor::default();
    e.replace(0, 0, "👩💻", "Input").unwrap();
    e.replace(1, 1, "\u{200d}", "Input").unwrap();
    assert_eq!(e.selection, (3, 3));
}
#[test]
fn invalid_edit_retains_document_and_history() {
    let mut e = sample();
    let before = e.document.clone();
    let len = e.undo_stack.len();
    for text in ["\0", "\r", "\u{2028}", "\u{2029}"] {
        assert!(e.replace(0, 0, text, "Input").is_err());
        assert_eq!(e.document, before);
        assert_eq!(e.undo_stack.len(), len);
    }
    assert!(e.replace(99999, 99999, "x", "Input").is_err());
}
#[test]
fn typing_style_branching_and_operation_boundaries() {
    let mut e = Editor::default();
    e.emphasize(0, 0, true, true).unwrap();
    assert!(e.undo_stack.is_empty());
    e.replace(0, 0, "a", "Type").unwrap();
    e.replace(1, 1, "b", "Type").unwrap();
    assert!(e.document.paragraphs[0].styles.iter().all(|s| s.bold));
    e.undo();
    assert_eq!(e.document.text(), "a");
    e.replace(1, 1, "c", "Type").unwrap();
    assert!(e.redo_stack.is_empty());
}
#[test]
fn paragraph_alignment_has_exclusive_selection_end() {
    let mut e = Editor::default();
    e.replace(0, 0, "one\ntwo\nthree", "Paste").unwrap();
    e.align(0, 4, Alignment::Right).unwrap();
    assert_eq!(e.document.paragraphs[1].alignment, Alignment::Left);
    e.replace(1, 7, "A\nB", "Replace").unwrap();
    assert_eq!(e.document.text(), "oA\nB\nthree");
    assert_eq!(e.document.paragraphs[1].alignment, Alignment::Right);
}
#[test]
fn native_roundtrip_preserves_full_supported_document() {
    let e = sample();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("round trip.bfdoc");
    storage::save(&e.document, &path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), e.document);
    assert_eq!(
        storage::decode(&storage::encode(&e.document).unwrap()).unwrap(),
        e.document
    );
}
#[test]
fn opens_existing_python_iteration_fixture() {
    let doc = storage::decode(include_bytes!("../examples/Welcome.bfdoc")).unwrap();
    assert_eq!(doc.paragraphs.len(), 4);
    assert!(doc.paragraphs[0].styles.iter().all(|s| s.bold));
    assert_eq!(doc.paragraphs[1].alignment, Alignment::Center);
    assert!(doc.text().contains("Українська · Русский · e\u{301}"));
}
#[test]
fn all_alignments_survive_native_format() {
    let mut e = sample();
    for alignment in [
        Alignment::Left,
        Alignment::Center,
        Alignment::Right,
        Alignment::Justify,
    ] {
        let len = e.document.character_count();
        e.align(0, len, alignment).unwrap();
        assert_eq!(
            storage::decode(&storage::encode(&e.document).unwrap()).unwrap(),
            e.document
        );
    }
}
#[test]
fn rejects_corrupt_foreign_unknown_and_duplicate_fields() {
    for data in [
        b"bad JSON".as_slice(),
        b"\xff",
        b"[]",
        b"{}",
        b"{\"format\":1,\"format\":2}",
    ] {
        assert!(storage::decode(data).is_err());
    }
    let original =
        serde_json::from_slice::<serde_json::Value>(&storage::encode(&sample().document).unwrap())
            .unwrap();
    for (key, value) in [
        ("format", serde_json::json!("breakfree.sheets.native")),
        ("content_version", serde_json::json!(2)),
        ("content_version", serde_json::json!(true)),
        ("unknown", serde_json::json!("preserve")),
        ("paragraphs", serde_json::json!([])),
    ] {
        let mut data = original.clone();
        data[key] = value;
        assert!(storage::decode(&serde_json::to_vec(&data).unwrap()).is_err());
    }
    let mut duplicate = String::from_utf8(storage::encode(&sample().document).unwrap()).unwrap();
    duplicate = duplicate.replacen(
        "\"content_version\": 1",
        "\"content_version\": 1, \"content_version\": 1",
        1,
    );
    assert!(storage::decode(duplicate.as_bytes()).is_err());
}
#[test]
fn failed_partial_write_sync_and_replace_preserve_original() {
    let e = sample();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("safe.bfdoc");
    storage::save(&e.document, &path).unwrap();
    let bytes = fs::read(&path).unwrap();
    let mut changed = sample();
    changed.replace(0, 0, "change ", "Edit").unwrap();
    for failure in [SaveStage::Write, SaveStage::Sync, SaveStage::Replace] {
        assert!(
            storage::save_checked(&changed.document, &path, |stage| if stage == failure {
                Err("injected disk failure".into())
            } else {
                Ok(())
            })
            .is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert_eq!(storage::load(&path).unwrap(), e.document);
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }
    assert!(storage::save(&changed.document, &dir.path().join("missing/new.bfdoc")).is_err());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    // Real persist failure: replacing an existing directory is rejected by the OS.
    let directory = dir.path().join("directory.bfdoc");
    fs::create_dir(&directory).unwrap();
    assert!(storage::save(&changed.document, &directory).is_err());
    assert!(directory.is_dir());
    assert_eq!(fs::read(&path).unwrap(), bytes);
}
#[test]
fn rejected_inputs_and_extensions_are_never_modified() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("broken.bfdoc");
    let bytes = b"do not overwrite me";
    fs::write(&path, bytes).unwrap();
    assert!(storage::load(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    let other = dir.path().join("foreign.docx");
    fs::write(&other, bytes).unwrap();
    assert!(storage::save(&sample().document, &other).is_err());
    assert_eq!(fs::read(&other).unwrap(), bytes);
}
#[test]
fn dirty_checkpoint_save_discard_cancel_and_failed_save() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("session.bfdoc");
    let mut s = Session::default();
    s.editor.replace(0, 0, "unsaved", "Input").unwrap();
    assert!(!s.can_leave(LeaveChoice::Cancel, None).unwrap());
    assert!(s.can_leave(LeaveChoice::Discard, None).unwrap());
    assert!(s.dirty());
    assert!(s.can_leave(LeaveChoice::Save, None).is_err());
    assert!(s.dirty());
    assert!(s.can_leave(LeaveChoice::Save, Some(&path)).unwrap());
    assert!(!s.dirty());
    s.editor.replace(0, 0, "new ", "Input").unwrap();
    s.editor.undo();
    assert!(!s.dirty());
    s.editor.redo();
    assert!(s.dirty());
    let old_path = s.path.clone();
    assert!(s.save(&dir.path().join("missing/new.bfdoc")).is_err());
    assert_eq!(s.path, old_path);
    assert!(s.dirty());
}
#[test]
fn projection_diff_keeps_clusters_and_actual_document_statistics() {
    assert_eq!(
        presentation::text_change("ae\u{301}b", "ae\u{301}!b"),
        (3, 3, "!".into())
    );
    assert_eq!(
        presentation::text_change("e", "e\u{301}"),
        (0, 1, "e\u{301}".into())
    );
    let e = sample();
    assert_eq!(e.document.paragraphs.len(), 5);
    assert!(e.document.word_count() > 0);
}
#[test]
fn ribbon_sizes_and_overflow_at_dpi_factors() {
    for dpi in [1.0, 1.25, 1.5] {
        for physical_width in [960., 1280., 1366., 1920.] {
            let width: f32 = physical_width / dpi;
            let groups = presentation::group_widths(width);
            let overflow = if groups[3] == 0. { 72. } else { 0. };
            assert!(
                groups.iter().sum::<f32>() + overflow + 48. <= width,
                "width {width} at dpi {dpi}"
            );
        }
    }
    assert_eq!(
        presentation::HOME_GROUPS,
        ["Clipboard", "Font", "Paragraph", "Styles", "Editing"]
    );
}
