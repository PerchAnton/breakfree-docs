use breakfree_docs::{commands::Editor, storage};

#[test]
fn positions_history_roundtrip_and_exclusive_selection() {
    let mut e = Editor::default();
    e.replace(0, 0, "English\tУкраїна\nРусский\te\u{301}\nlast", "Fixture")
        .unwrap();
    e.emphasize(0, 3, true, true).unwrap();
    let before = e.document.clone();
    let end = before.paragraphs[..2]
        .iter()
        .map(|p| p.text.chars().count() + 1)
        .sum();
    e.set_left_tab_stops(0, end, vec![1, 97, 2880]).unwrap();
    let after = e.document.clone();
    e.set_tab_interval(0, end, Some(96)).unwrap();
    assert_eq!(
        e.document.paragraphs[0].spacing.left_tab_stops_half_points,
        [1, 97, 2880]
    );
    e.undo();
    assert_eq!(e.document, after);
    for p in &after.paragraphs[..2] {
        assert_eq!(p.spacing.left_tab_stops_half_points, [1, 97, 2880]);
    }
    assert_eq!(after.paragraphs[2], before.paragraphs[2]);
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("stops.bfdoc");
    storage::save(&after, &path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), after);
    e.set_left_tab_stops(0, end, vec![]).unwrap();
    e.undo();
    assert_eq!(e.document, after);
    // A no-op must leave the pending clear operation available to Redo.
    e.set_left_tab_stops(0, end, vec![1, 97, 2880]).unwrap();
    e.redo();
    assert!(
        e.document.paragraphs[0]
            .spacing
            .left_tab_stops_half_points
            .is_empty()
    );
}

#[test]
fn invalid_positions_are_transactional_and_legacy_key_is_rejected() {
    let mut e = Editor::default();
    let before = e.document.clone();
    for bad in [
        vec![0],
        vec![2881],
        vec![72, 72],
        vec![144, 72],
        (1..=33).collect(),
    ] {
        assert!(e.set_left_tab_stops(0, 0, bad.clone()).is_err());
        assert_eq!(e.document, before);
        let mut json: serde_json::Value =
            serde_json::from_slice(&storage::encode(&before).unwrap()).unwrap();
        json["paragraphs"][0]["spacing"]["left_tab_stops_half_points"] = serde_json::json!(bad);
        assert!(storage::decode(&serde_json::to_vec(&json).unwrap()).is_err());
    }
    let mut json: serde_json::Value =
        serde_json::from_slice(&storage::encode(&before).unwrap()).unwrap();
    assert_eq!(json["content_version"], 15);
    json["content_version"] = 14.into();
    assert_eq!(
        storage::decode(&serde_json::to_vec(&json).unwrap()).unwrap(),
        before
    );
    for version in 1..=14 {
        json["content_version"] = version.into();
        for bad in [
            serde_json::json!([]),
            serde_json::Value::Null,
            serde_json::json!([72]),
        ] {
            json["paragraphs"][0]["spacing"]["left_tab_stops_half_points"] = bad;
            assert!(storage::decode(&serde_json::to_vec(&json).unwrap()).is_err());
        }
    }
    json["content_version"] = 15.into();
    for bad in [
        serde_json::Value::Null,
        serde_json::json!([1.5]),
        serde_json::json!([-1]),
        serde_json::json!(["72"]),
    ] {
        json["paragraphs"][0]["spacing"]["left_tab_stops_half_points"] = bad;
        assert!(storage::decode(&serde_json::to_vec(&json).unwrap()).is_err());
    }
}

#[test]
fn positions_follow_paragraph_editing_without_flattening() {
    let mut e = Editor::default();
    e.replace(0, 0, "a\tb\nc\td", "Fixture").unwrap();
    e.set_left_tab_stops(0, 0, vec![97, 192]).unwrap();
    e.set_left_tab_stops(4, 4, vec![144]).unwrap();
    let before = e.document.clone();
    e.insert_paragraph_break(1, 1).unwrap();
    assert_eq!(
        e.document.paragraphs[1].spacing.left_tab_stops_half_points,
        [97, 192]
    );
    assert_eq!(
        e.document.paragraphs[2].spacing.left_tab_stops_half_points,
        [144]
    );
    e.undo();
    assert_eq!(e.document, before);
    let plan = e.preview_replace_all(&[(0, 1)], "x\ny").unwrap();
    e.apply_replace_all(plan).unwrap();
    assert_eq!(
        e.document.paragraphs[1].spacing.left_tab_stops_half_points,
        [97, 192]
    );
    assert_eq!(
        e.document.paragraphs[2].spacing.left_tab_stops_half_points,
        [144]
    );
    e.undo();
    assert_eq!(e.document, before);
    e.replace(3, 4, "", "Join").unwrap();
    assert_eq!(
        e.document.paragraphs[0].spacing.left_tab_stops_half_points,
        [97, 192]
    );
    e.undo();
    assert_eq!(e.document, before);
}
