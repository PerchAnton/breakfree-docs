use breakfree_docs::{
    commands::Editor,
    model::*,
    search::{Options, Query},
    storage,
};

fn ranges(editor: &Editor, find: &str, options: Options) -> Vec<(usize, usize)> {
    Query::new(find, options)
        .search(&editor.document.text())
        .matches
        .iter()
        .map(|m| m.range)
        .collect()
}

#[test]
fn preview_is_read_only_and_apply_is_one_exact_styled_undo_roundtrip() {
    let mut e = Editor::default();
    e.replace(0, 0, "cat CAT cats\nУкраїна Русский e\u{301}", "Input")
        .unwrap();
    e.emphasize(0, 3, true, true).unwrap();
    e.underline(4, 7, true).unwrap();
    e.align(12, e.document.character_count(), Alignment::Right)
        .unwrap();
    e.selection = (1, 3);
    e.typing_style.italic = true;
    let original = e.document.clone();
    let selection = e.selection;
    let typing = e.typing_style;
    let count = e.undo_stack.len();
    let found = ranges(
        &e,
        "cat",
        Options {
            match_case: false,
            whole_words: true,
        },
    );
    let plan = e.preview_replace_all(&found, "ї").unwrap();
    assert_eq!((plan.match_count(), plan.change_count()), (2, 2));
    assert_eq!(e.document, original);
    assert_eq!(e.undo_stack.len(), count);
    let expected = plan.result().clone();
    assert_eq!(expected.text(), "ї ї cats\nУкраїна Русский e\u{301}");
    assert!(expected.paragraphs[0].styles[0].bold);
    assert!(expected.paragraphs[0].styles[2].underline);
    assert_eq!(expected.paragraphs[1], original.paragraphs[1]);
    assert_eq!(e.apply_replace_all(plan).unwrap(), 2);
    assert_eq!(e.document, expected);
    assert_eq!(e.undo_stack.len(), count + 1);
    assert_eq!(e.undo_stack.last().unwrap().label, "Replace all");
    e.undo();
    assert_eq!(e.document, original);
    assert_eq!(e.selection, selection);
    assert_eq!(e.typing_style, typing);
    e.redo();
    assert_eq!(e.document, expected);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bulk.bfdoc");
    storage::save(&e.document, &path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), expected);
}

#[test]
fn noops_preserve_mixed_styles_selection_and_redo() {
    let mut e = Editor::default();
    e.replace(0, 0, "cat cat", "Input").unwrap();
    e.emphasize(0, 1, true, true).unwrap();
    e.underline(4, 5, true).unwrap();
    e.undo();
    let before = e.document.clone();
    let state = (
        e.selection,
        e.typing_style,
        e.undo_stack.len(),
        e.redo_stack.len(),
    );
    let plan = e.preview_replace_all(&[(0, 3), (4, 7)], "cat").unwrap();
    assert_eq!(plan.change_count(), 0);
    assert_eq!(plan.result(), &before);
    assert_eq!(e.apply_replace_all(plan).unwrap(), 0);
    assert_eq!(e.document, before);
    assert_eq!(
        (
            e.selection,
            e.typing_style,
            e.undo_stack.len(),
            e.redo_stack.len()
        ),
        state
    );
    let empty = e.preview_replace_all(&[], "anything").unwrap();
    assert_eq!(e.apply_replace_all(empty).unwrap(), 0);
}

#[test]
fn delete_and_multiline_replacement_keep_paragraph_policy_and_original_ranges() {
    let mut e = Editor::default();
    e.replace(0, 0, "A\nx\nB\ny\nC", "Input").unwrap();
    e.align(0, 1, Alignment::Center).unwrap();
    e.align(4, 5, Alignment::Right).unwrap();
    e.align(8, 9, Alignment::Justify).unwrap();
    let before = e.document.clone();
    let plan = e.preview_replace_all(&[(1, 4), (5, 8)], "!\n?").unwrap();
    assert_eq!(plan.result().text(), "A!\n?B!\n?C");
    assert!(
        plan.result()
            .paragraphs
            .iter()
            .all(|p| p.alignment == Alignment::Center)
    );
    e.apply_replace_all(plan).unwrap();
    e.undo();
    assert_eq!(e.document, before);
    let plan = e.preview_replace_all(&[(0, 9)], "").unwrap();
    assert_eq!(plan.result().paragraphs.len(), 1);
    assert_eq!(plan.result().text(), "");
    assert_eq!(plan.result().paragraphs[0].alignment, Alignment::Center);
    e.apply_replace_all(plan).unwrap();
    assert_eq!(e.selection, (0, 0));
    e.undo();
    assert_eq!(e.document, before);
}

#[test]
fn invalid_ranges_controls_and_oversize_fail_without_any_live_mutation() {
    let mut e = Editor::default();
    e.replace(0, 0, "e\u{301} cat cat", "Input").unwrap();
    let before = e.document.clone();
    let state = (e.selection, e.typing_style, e.undo_stack.len());
    for bad in [
        vec![(0, 1)],
        vec![(0, 0)],
        vec![(3, 6), (4, 8)],
        vec![(7, 10), (3, 6)],
        vec![(0, 500)],
    ] {
        assert!(e.preview_replace_all(&bad, "x").is_err());
    }
    assert!(e.preview_replace_all(&[(3, 6)], "\0").is_err());
    assert!(
        e.preview_replace_all(&[(3, 6), (7, 10)], &"x".repeat(MAX_CHARACTERS))
            .is_err()
    );
    assert_eq!(e.document, before);
    assert_eq!((e.selection, e.typing_style, e.undo_stack.len()), state);
}

#[test]
fn stale_text_or_format_preview_cannot_overwrite_new_edits() {
    for format_only in [false, true] {
        let mut e = Editor::default();
        e.replace(0, 0, "cat cat", "Input").unwrap();
        let plan = e.preview_replace_all(&[(0, 3), (4, 7)], "dog").unwrap();
        if format_only {
            e.emphasize(0, 3, true, true).unwrap();
        } else {
            e.replace(0, 3, "new", "Edit").unwrap();
        }
        let before = e.document.clone();
        let count = e.undo_stack.len();
        assert!(
            e.apply_replace_all(plan)
                .unwrap_err()
                .contains("document changed")
        );
        assert_eq!(e.document, before);
        assert_eq!(e.undo_stack.len(), count);
    }
}

#[test]
fn dense_replacements_do_not_rescan_insertions_or_create_per_match_history() {
    let mut growing = Editor::default();
    growing.replace(0, 0, "a a", "Input").unwrap();
    let plan = growing
        .preview_replace_all(&[(0, 1), (2, 3)], "aa")
        .unwrap();
    growing.apply_replace_all(plan).unwrap();
    assert_eq!(growing.document.text(), "aa aa");
    assert_eq!(growing.undo_stack.len(), 2);
    growing.undo();
    assert_eq!(growing.document.text(), "a a");
    let mut e = Editor::default();
    e.replace(0, 0, &"a ".repeat(50000), "Input").unwrap();
    let found = ranges(&e, "a", Options::default());
    let plan = e.preview_replace_all(&found, "a").unwrap();
    assert_eq!((plan.match_count(), plan.change_count()), (50000, 0));
    let plan = e.preview_replace_all(&found, "b").unwrap();
    e.apply_replace_all(plan).unwrap();
    assert_eq!(e.document.text(), "b ".repeat(50000));
    assert_eq!(e.undo_stack.len(), 2);
    e.undo();
    assert_eq!(e.document.text(), "a ".repeat(50000));
    let plan = e.preview_replace_all(&[(0, 1)], "aa").unwrap_err();
    assert!(plan.contains("limit"));
}

#[test]
fn combining_replacements_and_mixed_noops_keep_valid_carets_and_exact_counts() {
    let mut e = Editor::default();
    e.replace(0, 0, "A a Ax", "Input").unwrap();
    e.emphasize(2, 3, true, true).unwrap();
    let found = ranges(
        &e,
        "a",
        Options {
            match_case: false,
            whole_words: true,
        },
    );
    let plan = e.preview_replace_all(&found, "a").unwrap();
    assert_eq!((plan.match_count(), plan.change_count()), (2, 1));
    assert!(plan.result().paragraphs[0].styles[2].bold);
    e.apply_replace_all(plan).unwrap();
    let plan = e.preview_replace_all(&[(1, 2)], "\u{301}").unwrap();
    e.apply_replace_all(plan).unwrap();
    assert_eq!(e.document.text(), "a\u{301}a Ax");
    assert!(validate_range(&e.document.text(), e.selection.0, e.selection.1).is_ok());
}
