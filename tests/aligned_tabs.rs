use breakfree_docs::{
    commands::Editor,
    model::{AlignedTabStop, TabStopAlignment},
    storage,
};
fn stop(position: u16, alignment: TabStopAlignment) -> AlignedTabStop {
    AlignedTabStop {
        position_half_points: position,
        alignment,
    }
}
#[test]
fn aligned_tabs_transaction_range_roundtrip_and_paragraph_break() {
    let mut e = Editor::default();
    e.replace(0, 0, "a\tb\nc\td\nlast", "Fixture").unwrap();
    e.set_left_tab_stops(0, 8, vec![72]).unwrap();
    let before = e.document.clone();
    let count = e.undo_stack.len();
    let stops = vec![
        stop(240, TabStopAlignment::Center),
        stop(480, TabStopAlignment::Right),
    ];
    e.set_aligned_tab_stops(0, 8, stops.clone()).unwrap();
    assert_eq!(e.undo_stack.len(), count + 1);
    let after = e.document.clone();
    assert_eq!(after.paragraphs[0].spacing.aligned_tab_stops, stops);
    assert_eq!(after.paragraphs[1].spacing.aligned_tab_stops, stops);
    assert_eq!(after.paragraphs[2], before.paragraphs[2]);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("aligned.bfdoc");
    storage::save(&after, &path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), after);
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
    e.insert_paragraph_break(1, 1).unwrap();
    assert_eq!(e.document.paragraphs[1].spacing.aligned_tab_stops, stops);
}
#[test]
fn aligned_tabs_reject_conflicts_capacity_and_malformed_storage() {
    let mut e = Editor::default();
    e.set_left_tab_stops(0, 0, vec![72]).unwrap();
    let before = e.document.clone();
    let count = e.undo_stack.len();
    for stops in [
        vec![stop(72, TabStopAlignment::Right)],
        vec![stop(0, TabStopAlignment::Center)],
        vec![stop(2881, TabStopAlignment::Right)],
        vec![
            stop(100, TabStopAlignment::Right),
            stop(100, TabStopAlignment::Center),
        ],
        vec![
            stop(200, TabStopAlignment::Center),
            stop(100, TabStopAlignment::Right),
        ],
        (100..132)
            .map(|p| stop(p, TabStopAlignment::Right))
            .collect(),
    ] {
        assert!(e.set_aligned_tab_stops(0, 0, stops).is_err());
        assert_eq!(e.document, before);
        assert_eq!(e.undo_stack.len(), count);
    }
    let base: serde_json::Value =
        serde_json::from_slice(&storage::encode(&before).unwrap()).unwrap();
    assert_eq!(base["content_version"], storage::CONTENT_VERSION);
    for value in [
        serde_json::json!(null),
        serde_json::json!([{"position_half_points":100,"alignment":"decimal"}]),
        serde_json::json!([{"position_half_points":100,"alignment":"center","unknown":1}]),
        serde_json::json!([{"position_half_points":1.5,"alignment":"right"}]),
    ] {
        let mut json = base.clone();
        json["paragraphs"][0]["spacing"]["aligned_tab_stops"] = value;
        assert!(storage::decode(&serde_json::to_vec(&json).unwrap()).is_err());
    }
    for version in 1..=15 {
        let mut json = base.clone();
        json["content_version"] = version.into();
        for value in [serde_json::json!([]), serde_json::json!(null)] {
            json["paragraphs"][0]["spacing"]["aligned_tab_stops"] = value;
            assert!(storage::decode(&serde_json::to_vec(&json).unwrap()).is_err());
        }
    }
    let mut legacy = base;
    legacy["content_version"] = 15.into();
    assert_eq!(
        storage::decode(&serde_json::to_vec(&legacy).unwrap()).unwrap(),
        before
    );
    e.set_aligned_tab_stops(0, 0, vec![stop(144, TabStopAlignment::Right)])
        .unwrap();
    let before = e.document.clone();
    assert!(e.set_left_tab_stops(0, 0, vec![144]).is_err());
    assert_eq!(e.document, before);
}
#[test]
fn aligned_tabs_example_and_left_ruler_preserve_typed_positions() {
    let document = storage::decode(include_bytes!("../examples/AlignedTabs.bfdoc")).unwrap();
    assert_eq!(document.paragraphs.len(), 5);
    assert_eq!(
        storage::decode(&storage::encode(&document).unwrap()).unwrap(),
        document
    );
    let mut e = Editor::default();
    e.set_aligned_tab_stops(0, 0, vec![stop(144, TabStopAlignment::Right)])
        .unwrap();
    assert!(breakfree_docs::tab_ruler::Draft::create(&e, 144).is_err());
    let before = e.document.clone();
    breakfree_docs::tab_ruler::Draft::create(&e, 72)
        .unwrap()
        .commit(&mut e)
        .unwrap();
    assert_eq!(
        e.document.paragraphs[0].spacing.aligned_tab_stops,
        before.paragraphs[0].spacing.aligned_tab_stops
    );
    e.undo();
    assert_eq!(e.document, before);
}
