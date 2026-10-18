use breakfree_docs::{
    commands::{Editor, FontPatch},
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
    e.underline(0, 18, true).unwrap();
    e.set_font_size(8, 18, "18.5".parse().unwrap()).unwrap();
    e.align(8, 18, Alignment::Center).unwrap();
    e
}

#[test]
fn font_patch_preserves_mixed_properties_and_roundtrips_as_one_operation() {
    let mut e = sample();
    let end = e.document.character_count();
    e.selection = (0, end);
    let before = e.document.clone();
    let typing = e.typing_style;
    let history = e.undo_stack.len();
    assert_eq!(e.selected_font().bold, None);
    assert_eq!(e.selected_font().font_size, None);
    let patch = FontPatch {
        italic: Some(true),
        font_size: Some("23.5".parse().unwrap()),
        ..Default::default()
    };
    e.set_font(0, end, patch).unwrap();
    let after = e.document.clone();
    assert_eq!(e.undo_stack.len(), history + 1);
    assert_eq!(e.undo_stack.last().unwrap().label, "Font");
    for (old, new) in before.paragraphs.iter().zip(&after.paragraphs) {
        assert_eq!(old.text, new.text);
        assert_eq!(old.alignment, new.alignment);
        for (a, b) in old.styles.iter().zip(&new.styles) {
            assert_eq!(a.bold, b.bold);
            assert_eq!(a.underline, b.underline);
            assert_eq!(b.italic, Some(true));
            assert_eq!(b.font_size.unwrap_or_default().to_string(), "23.5");
        }
    }
    let after_typing = e.typing_style;
    e.undo();
    assert_eq!(e.document, before);
    assert_eq!(e.typing_style, typing);
    assert_eq!(e.selection, (0, end));
    e.redo();
    assert_eq!(e.document, after);
    assert_eq!(e.typing_style, after_typing);
    e.set_font(0, end, patch).unwrap();
    assert_eq!(e.undo_stack.len(), history + 1);
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("Font.bfdoc");
    storage::save(&e.document, &path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), after);
}

#[test]
fn font_patch_validates_graphemes_and_preserves_caret_history() {
    let mut e = Editor::default();
    e.replace(0, 0, "e\u{301} ї", "Type").unwrap();
    let original = e.document.clone();
    let history = e.undo_stack.len();
    let patch = FontPatch {
        bold: Some(true),
        underline: Some(true),
        ..Default::default()
    };
    assert!(e.set_font(0, 1, patch).is_err());
    assert_eq!(e.document, original);
    e.set_font(4, 4, patch).unwrap();
    assert_eq!(e.document, original);
    assert_eq!(e.undo_stack.len(), history);
    let typing = e.typing_style;
    e.set_font(0, 4, FontPatch::default()).unwrap();
    assert_eq!(e.typing_style, typing);
    e.replace(4, 4, "ё", "Type").unwrap();
    assert_eq!(e.document.paragraphs[0].styles[4], typing);
    e.undo();
    assert_eq!(e.document, original);
    assert_eq!(e.typing_style, typing);
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
                assert!(e.underline(0, i, true).is_err());
                assert!(e.set_font_size(0, i, "24".parse().unwrap()).is_err());
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
    assert!(
        e.document.paragraphs[0]
            .styles
            .iter()
            .all(|s| s.bold.unwrap_or(false))
    );
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
    assert!(
        doc.paragraphs[0]
            .styles
            .iter()
            .all(|s| s.bold.unwrap_or(false))
    );
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
        ("content_version", serde_json::json!(6)),
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
        "\"content_version\": 5",
        "\"content_version\": 5, \"content_version\": 5",
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

#[test]
fn size_and_underline_history_preserve_other_styles_and_selection() {
    let mut e = sample();
    let before = e.document.clone();
    let before_typing = e.typing_style;
    let range = (0, e.document.character_count());
    e.set_font_size(range.0, range.1, "24.5".parse().unwrap())
        .unwrap();
    let sized = e.document.clone();
    for (old, new) in before
        .paragraphs
        .iter()
        .flat_map(|p| &p.styles)
        .zip(sized.paragraphs.iter().flat_map(|p| &p.styles))
    {
        assert_eq!(
            (old.bold, old.italic, old.underline),
            (new.bold, new.italic, new.underline)
        );
        assert_eq!(new.font_size.unwrap_or_default().to_string(), "24.5");
    }
    assert_eq!(e.selection, range);
    e.underline(range.0, range.1, true).unwrap();
    let styled = e.document.clone();
    assert!(e.underline_selected());
    e.undo();
    assert_eq!(e.document, sized);
    e.undo();
    assert_eq!(e.document, before);
    assert_eq!(e.typing_style, before_typing);
    e.redo();
    assert_eq!(e.document, sized);
    e.redo();
    assert_eq!(e.document, styled);
    assert_eq!(e.selection, range);
    let count = e.undo_stack.len();
    e.underline(range.0, range.1, true).unwrap();
    e.set_font_size(range.0, range.1, "24.5".parse().unwrap())
        .unwrap();
    assert_eq!(
        e.undo_stack.len(),
        count,
        "No-op formatting must not consume undo entries"
    );
    e.replace(range.0, range.1, "ї e\u{301}", "Replace")
        .unwrap();
    assert!(
        e.document.paragraphs[0]
            .styles
            .iter()
            .all(|s| s.underline.unwrap_or(false)
                && s.font_size.unwrap_or_default().to_string() == "24.5")
    );
    e.undo();
    assert_eq!(e.document, styled);
}

#[test]
fn pending_size_and_underline_survive_typing_history_and_mixed_selection() {
    let mut e = Editor::default();
    e.set_font_size(0, 0, "14.5".parse().unwrap()).unwrap();
    e.underline(0, 0, true).unwrap();
    let pending = e.typing_style;
    assert!(e.undo_stack.is_empty());
    assert_eq!(e.document, Document::default());
    e.replace(0, 0, "English\nУкраїнська\nРусский e\u{301}", "Input")
        .unwrap();
    assert!(
        e.document
            .paragraphs
            .iter()
            .flat_map(|p| &p.styles)
            .all(|s| *s == pending)
    );
    let typed = e.document.clone();
    e.undo();
    assert_eq!(e.typing_style, pending);
    e.redo();
    assert_eq!(e.document, typed);
    assert_eq!(e.typing_style, pending);
    e.set_font_size(0, 7, "18".parse().unwrap()).unwrap();
    e.underline(0, 7, false).unwrap();
    e.selection = (0, e.document.character_count());
    assert_eq!(e.selected_font_size(), None);
    assert!(!e.underline_selected());
    e.underline(e.selection.0, e.selection.1, true).unwrap();
    assert!(e.underline_selected());
    e.selection = (7, 8); // Only a paragraph separator, no character style to report.
    assert_eq!(e.selected_font_size(), None);
}

#[test]
fn font_size_input_is_exact_and_bounded() {
    for text in ["1", "1.5", "11", "12.5", "399.5", "400", " 24.0 "] {
        let size: FontSize = text.parse().unwrap();
        assert_eq!(size.to_string().parse::<FontSize>().unwrap(), size);
    }
    for text in [
        "", "0", "0.5", "400.5", "401", "NaN", "inf", "1.25", "12e1", "-12", "+12", ".5", "999999",
        "11.50", "12,5",
    ] {
        assert!(text.parse::<FontSize>().is_err(), "accepted {text}");
    }
    assert!(FontSize::try_from(0).is_err());
    assert!(FontSize::try_from(801).is_err());
}

#[test]
fn v1_load_is_read_only_and_v5_roundtrip_preserves_new_styles() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v1.bfdoc");
    let old = include_bytes!("../examples/Welcome.bfdoc");
    fs::write(&path, old).unwrap();
    let document = storage::load(&path).unwrap();
    assert_eq!(fs::read(&path).unwrap(), old);
    assert!(
        document
            .paragraphs
            .iter()
            .flat_map(|p| &p.styles)
            .all(|s| !s.underline.unwrap_or(false) && s.font_size == Some(FontSize::default()))
    );
    let mut s = Session::new(document, Some(path.clone()));
    s.editor.underline(0, 7, true).unwrap();
    s.editor
        .set_font_size(0, 7, "28.5".parse().unwrap())
        .unwrap();
    s.save(&path).unwrap();
    let new = fs::read(&path).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&new).unwrap();
    assert_eq!(json["content_version"], 5);
    assert_eq!(storage::load(&path).unwrap(), s.editor.document);
    assert!(!s.dirty());
}

#[test]
fn invalid_v2_styles_and_v1_extensions_are_rejected_without_modifying_files() {
    let original: serde_json::Value =
        serde_json::from_slice(include_bytes!("../examples/Formatting.bfdoc")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("invalid.bfdoc");
    let mut inputs = Vec::new();
    for (key, value) in [
        ("font_size_half_points", serde_json::json!(0)),
        ("font_size_half_points", serde_json::json!(801)),
        ("font_size_half_points", serde_json::json!(-1)),
        ("font_size_half_points", serde_json::json!(22.5)),
        ("font_size_half_points", serde_json::json!(true)),
        ("font_size_half_points", serde_json::json!(null)),
        ("underline", serde_json::json!("true")),
        ("underline", serde_json::json!(null)),
        ("unknown", serde_json::json!(true)),
    ] {
        let mut data = original.clone();
        data["paragraphs"][0]["runs"][0][key] = value;
        inputs.push(serde_json::to_vec(&data).unwrap());
    }
    for key in ["underline", "font_size_half_points"] {
        let mut data = original.clone();
        data["paragraphs"][0]["runs"][0]
            .as_object_mut()
            .unwrap()
            .remove(key);
        inputs.push(serde_json::to_vec(&data).unwrap());
    }
    let encoded = String::from_utf8(serde_json::to_vec(&original).unwrap()).unwrap();
    inputs.push(
        encoded
            .replacen(
                "\"underline\":true",
                "\"underline\": true, \"underline\": false",
                1,
            )
            .into_bytes(),
    );
    inputs.push(
        encoded
            .replacen("\"content_version\":2", "\"content_version\":1", 1)
            .into_bytes(),
    );
    let old = String::from_utf8(include_bytes!("../examples/Welcome.bfdoc").to_vec()).unwrap();
    inputs.push(
        old.replacen("\"bold\": true", "\"bold\": true, \"underline\": null", 1)
            .into_bytes(),
    );
    for bytes in inputs {
        fs::write(&path, &bytes).unwrap();
        assert!(
            storage::load(&path).is_err(),
            "accepted {}",
            String::from_utf8_lossy(&bytes)
        );
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn font_steps_preserve_mixed_runs_unicode_and_exact_history() {
    let mut e = sample();
    let before = e.document.clone();
    let typing = e.typing_style;
    let range = (0, e.document.character_count());
    let count = e.undo_stack.len();
    e.step_font_size(range.0, range.1, true).unwrap();
    let after = e.document.clone();
    assert_eq!(e.selection, range);
    assert_eq!(e.undo_stack.len(), count + 1);
    for (old, new) in before
        .paragraphs
        .iter()
        .flat_map(|p| &p.styles)
        .zip(after.paragraphs.iter().flat_map(|p| &p.styles))
    {
        assert_eq!(
            (old.bold, old.italic, old.underline),
            (new.bold, new.italic, new.underline)
        );
        assert_eq!(
            new.font_size.unwrap_or_default().to_string(),
            if old.font_size.unwrap_or_default().to_string() == "18.5" {
                "20"
            } else {
                "12"
            }
        );
    }
    assert_eq!(before.text(), after.text());
    e.undo();
    assert_eq!(e.document, before);
    assert_eq!(e.typing_style, typing);
    e.redo();
    assert_eq!(e.document, after);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("stepped.bfdoc");
    storage::save(&after, &path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), after);
    let accent = e
        .document
        .text()
        .chars()
        .position(|c| c == '\u{301}')
        .unwrap();
    assert!(e.step_font_size(accent, accent + 1, false).is_err());
    assert_eq!(e.document, after);
    assert_eq!(e.undo_stack.len(), count + 1);
    e.selection = (range.1, range.1);
    e.step_font_size(range.1, range.1, false).unwrap();
    assert_eq!(e.document, after);
    assert_eq!(e.undo_stack.len(), count + 1);
    assert_eq!(
        e.typing_style.font_size.unwrap_or_default().to_string(),
        "11"
    );
}

#[test]
fn font_steps_are_bounded_and_noops_do_not_consume_history() {
    for half in 2..=800 {
        let size = FontSize::try_from(half).unwrap();
        assert!(u16::from(size.stepped(true)) >= half);
        assert!(u16::from(size.stepped(false)) <= half);
        assert!(FontSize::try_from(u16::from(size.stepped(true))).is_ok());
        assert!(FontSize::try_from(u16::from(size.stepped(false))).is_ok());
    }
    for (from, grow, expected) in [
        ("18.5", false, "18"),
        ("28", true, "36"),
        ("72", true, "73"),
        ("100.5", false, "99.5"),
        ("8", false, "7"),
    ] {
        assert_eq!(
            from.parse::<FontSize>().unwrap().stepped(grow).to_string(),
            expected
        );
    }
    let mut e = Editor::default();
    e.replace(0, 0, "ї", "Type").unwrap();
    for (size, grow) in [("400", true), ("1", false)] {
        e.set_font_size(0, 1, size.parse().unwrap()).unwrap();
        let count = e.undo_stack.len();
        e.step_font_size(0, 1, grow).unwrap();
        assert_eq!(e.undo_stack.len(), count);
        assert_eq!(e.selected_font_size().unwrap().to_string(), size);
    }
}

#[test]
fn clear_character_formatting_preserves_text_paragraphs_and_one_exact_history_step() {
    let mut e = sample();
    let end = e.document.character_count() - 1;
    e.selection = (8, end);
    let before = e.document.clone();
    let typing = e.typing_style;
    let history = e.undo_stack.len();
    e.clear_character_formatting(8, end).unwrap();
    let after = e.document.clone();
    assert_eq!(after.text(), before.text());
    assert_eq!(after.paragraphs[0], before.paragraphs[0]);
    for (a, b) in after.paragraphs.iter().zip(&before.paragraphs).skip(1) {
        assert_eq!(a.alignment, b.alignment);
        assert!(a.styles.iter().all(|s| *s == CharacterFormat::default()));
    }
    assert_eq!(e.selection, (8, end));
    assert_eq!(e.typing_style, CharacterFormat::default());
    assert_eq!(e.undo_stack.len(), history + 1);
    assert_eq!(
        e.undo_stack.last().unwrap().label,
        "Clear character formatting"
    );
    e.undo();
    assert_eq!(e.document, before);
    assert_eq!(e.selection, (8, end));
    assert_eq!(e.typing_style, typing);
    e.redo();
    assert_eq!(e.document, after);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cleared.bfdoc");
    storage::save(&e.document, &path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), after);
}

#[test]
fn clear_at_caret_only_resets_pending_style_without_dirtying_or_losing_redo() {
    let mut e = sample();
    e.replace(0, 0, "x", "Type").unwrap();
    e.undo();
    e.selection = (2, 2);
    e.typing_style = CharacterFormat {
        bold: Some(true),
        italic: Some(true),
        underline: Some(true),
        font_size: Some("28".parse().unwrap()),
    };
    let before = e.document.clone();
    let history = (e.undo_stack.len(), e.redo_stack.len());
    e.clear_character_formatting(2, 2).unwrap();
    assert_eq!(e.document, before);
    assert_eq!(e.selection, (2, 2));
    assert_eq!(e.typing_style, CharacterFormat::default());
    assert_eq!((e.undo_stack.len(), e.redo_stack.len()), history);
    e.replace(2, 2, "їe\u{301}", "Type").unwrap();
    assert!(
        e.document.paragraphs[0].styles[2..5]
            .iter()
            .all(|s| *s == CharacterFormat::default())
    );
    assert!(e.document.paragraphs[0].styles[0].bold.unwrap_or(false));
    e.undo();
    assert_eq!(e.document, before);
    assert_eq!(e.typing_style, CharacterFormat::default());
}

#[test]
fn clear_rejects_partial_clusters_and_plain_or_separator_ranges_keep_history() {
    let mut e = Editor::default();
    e.replace(0, 0, "e\u{301} 👩‍💻\nї", "Type").unwrap();
    e.emphasize(0, 2, true, true).unwrap();
    e.undo();
    let before = e.document.clone();
    let state = (
        e.selection,
        e.typing_style,
        e.undo_stack.len(),
        e.redo_stack.len(),
    );
    for range in [(0, 1), (4, 6), (5, 5), (8, 7), (0, 50)] {
        assert!(e.clear_character_formatting(range.0, range.1).is_err());
        assert_eq!(
            (
                e.selection,
                e.typing_style,
                e.undo_stack.len(),
                e.redo_stack.len()
            ),
            state
        );
    }
    for range in [(0, 2), (6, 7), (0, 8)] {
        e.clear_character_formatting(range.0, range.1).unwrap();
        assert_eq!(e.document, before);
        assert_eq!((e.undo_stack.len(), e.redo_stack.len()), (state.2, state.3));
    }
}
