use breakfree_docs::{
    commands::Editor,
    model::*,
    search::{Options, Query, Results},
    storage,
};

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
    let query = Query::new(
        "Ї",
        Options {
            match_case: false,
            whole_words: true,
        },
    );
    assert_eq!(query.search(&text).matches, results.matches);
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

#[test]
fn case_matching_keeps_original_scalar_ranges_and_literal_syntax() {
    let options = Options {
        match_case: false,
        whole_words: false,
    };
    let text = "ENGLISH English english\nУКРАЇНА Україна україна\nРУССКИЙ Русский русский";
    for query in ["English", "україна", "русский"] {
        let found = Query::new(query, options).search(text);
        assert_eq!(found.matches.len(), 3);
        for result in found.matches {
            let original = slice(text, result.range.0, result.range.1);
            assert_eq!(original.to_lowercase(), query.to_lowercase());
            assert!(validate_range(text, result.range.0, result.range.1).is_ok());
        }
    }
    for query in ["[A-Z]+", "(?i)", "a.b", "a*b", "\\w", "(a)"] {
        let text = format!("before {query} after");
        let found = Query::new(query, options).search(&text);
        assert_eq!(found.matches.len(), 1);
        assert_eq!(
            slice(&text, found.matches[0].range.0, found.matches[0].range.1),
            query
        );
    }
    // Simple, locale-independent folding is explicit: no expansion or normalization.
    for (text, query, count) in [
        ("Σ σ ς", "σ", 3),
        ("K k K", "k", 3),
        ("S s ſ", "s", 3),
        ("ẞ ß SS", "ß", 2),
        ("ß", "ss", 0),
        ("I i İ ı", "i", 2),
        ("É é E\u{301} e\u{301}", "é", 2),
        ("E\u{301} e\u{301}", "e", 0),
        ("E\u{301} e\u{301}", "e\u{301}", 2),
        ("👩‍💻", "👩", 0),
    ] {
        let found = Query::new(query, options).search(text);
        assert_eq!(found.matches.len(), count, "{text:?} / {query:?}");
        for result in found.matches {
            assert!(validate_range(text, result.range.0, result.range.1).is_ok());
        }
    }
}

#[test]
fn whole_words_use_unicode_boundaries_including_apostrophes_and_phrases() {
    let options = Options {
        match_case: false,
        whole_words: true,
    };
    for (text, query, count) in [
        ("cat cats scatter CAT cat2 cat_name (cat) cat-dog", "cat", 4),
        ("don't don'ts don’t", "don", 0),
        ("don't don’t", "don't", 1),
        ("п’ять п'ять", "ять", 0),
        ("п’ять п'ять", "п’ять", 1),
        ("Україна УКРАЇНА України", "україна", 2),
        ("русский русский2 нерусский", "русский", 1),
        ("e\u{301} E\u{301} e\u{301}x", "e\u{301}", 2),
        ("cat dog! cat dogs", "cat dog", 1),
        ("cat\ndog!", "CAT\nDOG", 1),
        ("👩‍💻 !", "!", 0),
    ] {
        let found = Query::new(query, options).search(text);
        assert_eq!(found.matches.len(), count, "{text:?} / {query:?}");
        for result in found.matches {
            assert!(validate_range(text, result.range.0, result.range.1).is_ok());
        }
    }
    // A partial first candidate must not consume the valid overlapping candidate.
    assert_eq!(
        Query::new("a a", options).search("xa a a").matches[0].range,
        (3, 6)
    );
}

#[test]
fn search_limits_are_explicit_and_options_do_not_modify_source() {
    let text = "Hello hello hellos";
    for options in [
        Options::default(),
        Options {
            match_case: false,
            whole_words: true,
        },
    ] {
        let long = "ї".repeat(2049);
        let query = Query::new(&long, options);
        assert!(query.error().is_some());
        assert!(query.search(text).matches.is_empty());
        assert!(query.is_current(&long, options));
        assert!(!query.is_current("Hello", options));
        assert!(Query::new("", options).search(text).matches.is_empty());
        assert!(Query::new(&"a".repeat(4096), options).error().is_none());
    }
}

#[test]
fn folded_whole_word_replacement_roundtrips_style_and_history() {
    let mut editor = Editor::default();
    editor
        .replace(0, 0, "K Kx\nУкраїна УКРАЇНА", "Input")
        .unwrap();
    editor.emphasize(0, 1, true, true).unwrap();
    let before = editor.document.clone();
    let text = before.text();
    let query = Query::new(
        "k",
        Options {
            match_case: false,
            whole_words: true,
        },
    );
    let found = query.search(&text);
    assert_eq!(found.matches.len(), 1);
    let (a, b) = found.matches[0].range;
    assert_eq!((a, b), (0, 1)); // Kelvin sign has three bytes, one scalar.
    editor.selection = (a, b);
    editor.typing_style = editor.document.style_at(a);
    editor.replace(a, b, "e\u{301}", "Replace match").unwrap();
    let after = editor.document.clone();
    assert_eq!(after.text(), "e\u{301} Kx\nУкраїна УКРАЇНА");
    assert!(after.paragraphs[0].styles[0].bold);
    editor.undo();
    assert_eq!(editor.document, before);
    editor.redo();
    assert_eq!(editor.document, after);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("options.bfdoc");
    storage::save(&after, &path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), after);
}
