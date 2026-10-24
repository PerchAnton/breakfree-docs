mod support;
use breakfree_docs::{commands::Editor, storage};

#[test]
fn literal_tabs_interval_history_roundtrip_and_selection() {
    let mut e = Editor::default();
    e.replace(0, 0, "English\tУкраїна\nРусский\te\u{301}\t", "Paste")
        .unwrap();
    let before = e.document.clone();
    let end = before.paragraphs[0].text.chars().count() + 1;
    e.set_tab_interval(0, end, Some(96)).unwrap();
    assert_eq!(
        e.document.paragraphs[0].spacing.tab_interval_half_points,
        Some(96)
    );
    assert_eq!(e.document.paragraphs[1], before.paragraphs[1]);
    let after = e.document.clone();
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tabs.bfdoc");
    storage::save(&after, &path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), after);
    e.insert_paragraph_break(1, 1).unwrap();
    assert_eq!(
        e.document.paragraphs[1].spacing.tab_interval_half_points,
        Some(96)
    );
}

#[test]
fn tab_interval_validation_and_strict_legacy_import() {
    let mut e = Editor::default();
    e.replace(0, 0, "a\tb", "Fixture").unwrap();
    let before = e.document.clone();
    for bad in [0, 289, u16::MAX] {
        assert!(e.set_tab_interval(0, 0, Some(bad)).is_err());
        assert_eq!(e.document, before);
    }
    let mut value: serde_json::Value =
        serde_json::from_slice(&storage::encode(&before).unwrap()).unwrap();
    assert_eq!(value["content_version"], storage::CONTENT_VERSION);
    value["content_version"] = 13.into();
    support::strip_structure_for_legacy(&mut value);
    assert_eq!(
        storage::decode(&serde_json::to_vec(&value).unwrap()).unwrap(),
        before
    );
    value["paragraphs"][0]["spacing"]["tab_interval_half_points"] = serde_json::Value::Null;
    assert!(storage::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    value["content_version"] = 14.into();
    support::strip_structure_for_legacy(&mut value);
    value["paragraphs"][0]["spacing"]["tab_interval_half_points"] = 0.into();
    assert!(storage::decode(&serde_json::to_vec(&value).unwrap()).is_err());
}
