use breakfree_docs::{commands::Editor, model::*, search::Results, storage};

#[test]
fn literal_matches_preserve_unicode_and_require_complete_graphemes() {
    let text = "English English\nУкраїнська Русский\ne\u{301} é 👩‍💻 ї";
    for query in [
        "English",
        "Українська",
        "Русский",
        "e\u{301}",
        "é",
        "👩‍💻",
        "ї",
    ] {
        let found = Results::new(text, query);
        assert!(!found.matches.is_empty(), "{query}");
        for result in &found.matches {
            assert_eq!(slice(text, result.range.0, result.range.1), query);
            assert!(validate_range(text, result.range.0, result.range.1).is_ok());
        }
    }
    for query in [
        "", "english", "e", "\u{301}", "👩", "💻", "\u{200d}", "missing",
    ] {
        assert!(Results::new(text, query).matches.is_empty(), "{query:?}");
    }
    let crossing = Results::new(text, "Русский\ne\u{301}");
    assert_eq!(crossing.matches.len(), 1);
    assert_eq!(crossing.matches[0].paragraph, 2);
    assert_eq!(Results::new("aaaaa", "aa").matches.len(), 2);
}

#[test]
fn navigation_wraps_from_selection_or_caret_without_a_result_limit() {
    let text = "ї ".repeat(50000);
    let results = Results::new(&text, "ї");
    assert_eq!(results.matches.len(), 50000);
    assert_eq!(results.adjacent((0, 0), false), Some(0));
    assert_eq!(results.adjacent((0, 0), true), Some(49999));
    assert_eq!(results.adjacent((1, 0), false), Some(1));
    assert_eq!(results.adjacent((99998, 99999), false), Some(0));
    assert_eq!(results.adjacent((0, 1), true), Some(49999));
    assert_eq!(results.adjacent((50, 60), false), Some(30));
    assert_eq!(results.adjacent((50, 60), true), Some(24));
    assert_eq!(Results::new(&text, "absent").adjacent((0, 0), false), None);
}

#[test]
fn excerpts_are_bounded_and_do_not_split_combining_or_joined_text() {
    let text = format!("{}match{}", "e\u{301}".repeat(40), "👩‍💻".repeat(50));
    let results = Results::new(&text, "match");
    let (left, matched, right) = results.excerpt(0);
    assert_eq!(left, format!("…{}", "e\u{301}".repeat(18)));
    assert_eq!(matched, "match");
    assert_eq!(right, format!("{}…", "👩‍💻".repeat(24)));
    let long = "ї".repeat(100);
    assert_eq!(
        Results::new(&long, &long).excerpt(0).1,
        format!("{}…", "ї".repeat(32))
    );
}

#[test]
fn searched_replacement_uses_existing_history_format_and_storage_rules() {
    let mut editor = Editor::default();
    editor
        .replace(0, 0, "English\nУкраїнська e\u{301}\nРусский", "Input")
        .unwrap();
    editor.emphasize(8, 17, true, true).unwrap();
    editor.align(8, 17, Alignment::Right).unwrap();
    let original = editor.document.clone();
    let text = editor.document.text();
    let found = Results::new(&text, "Українська");
    let (start, end) = found.matches[0].range;
    editor.selection = (start, end);
    editor.typing_style = editor.document.style_at(start);
    editor
        .replace(start, end, "Україна", "Replace match")
        .unwrap();
    let changed = editor.document.clone();
    assert_eq!(changed.text(), "English\nУкраїна e\u{301}\nРусский");
    assert!(changed.paragraphs[1].styles[0].bold);
    assert_eq!(changed.paragraphs[1].alignment, Alignment::Right);
    editor.undo();
    assert_eq!(editor.document, original);
    assert_eq!(editor.selection, (start, end));
    editor.redo();
    assert_eq!(editor.document, changed);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("search.bfdoc");
    storage::save(&editor.document, &path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), changed);
}
