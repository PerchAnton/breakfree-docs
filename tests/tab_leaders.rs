use breakfree_docs::{
    commands::Editor,
    model::{TabKind as K, TabLeader as L},
    storage,
    tab_ruler::Draft,
};

#[test]
fn leader_history_roundtrip_boundary_and_enter() {
    for kind in [K::Left, K::Center, K::Right] {
        for leader in [L::Dots, L::Dashes, L::Line] {
            let mut e = Editor::default();
            e.replace(0, 0, "a\tb\nc\td\nlast", "Fixture").unwrap();
            e.set_tab_positions(0, 8, vec![(144, kind)]).unwrap();
            let before = e.document.clone();
            let n = e.undo_stack.len();
            e.set_tab_leader(0, 8, 144, Some(leader)).unwrap();
            let after = e.document.clone();
            assert_eq!(e.undo_stack.len(), n + 1);
            assert_eq!(after.paragraphs[1].spacing.tab_leader(144), Some(leader));
            assert_eq!(after.paragraphs[2], before.paragraphs[2]);
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("leaders.bfdoc");
            storage::save(&after, &path).unwrap();
            assert_eq!(storage::load(&path).unwrap(), after);
            e.undo();
            assert_eq!(e.document, before);
            e.redo();
            assert_eq!(e.document, after);
            e.insert_paragraph_break(1, 1).unwrap();
            assert_eq!(
                e.document.paragraphs[1].spacing.tab_leader(144),
                Some(leader)
            );
        }
    }
}
#[test]
fn leader_moves_and_deletes_with_each_owner_in_one_transaction() {
    let mut e = Editor::default();
    e.replace(0, 0, "a\nb", "Fixture").unwrap();
    e.set_tab_positions(0, 3, vec![(144, K::Center), (288, K::Right)])
        .unwrap();
    e.set_tab_leader(0, 0, 144, Some(L::Dots)).unwrap();
    e.set_tab_leader(2, 2, 144, Some(L::Dashes)).unwrap();
    e.set_tab_leader(0, 3, 288, Some(L::Line)).unwrap();
    e.selection = (0, 3);
    let before = e.document.clone();
    let n = e.undo_stack.len();
    let mut d = Draft::new(&e, 144).unwrap();
    assert!(d.move_to(288.).is_err());
    assert_eq!(d.preview, before);
    d.move_to(400.).unwrap();
    d.commit(&mut e).unwrap();
    let after = e.document.clone();
    assert_eq!(after.paragraphs[0].spacing.tab_leader(400), Some(L::Dots));
    assert_eq!(after.paragraphs[1].spacing.tab_leader(400), Some(L::Dashes));
    assert_eq!(after.paragraphs[0].spacing.tab_leader(288), Some(L::Line));
    assert_eq!(e.undo_stack.len(), n + 1);
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
    let mut d = Draft::new(&e, 400).unwrap();
    d.remove().unwrap();
    d.commit(&mut e).unwrap();
    assert_eq!(e.document.paragraphs[0].spacing.tab_leader(400), None);
    e.undo();
    assert_eq!(e.document, after);
    e.set_left_tab_stops(0, 3, vec![72]).unwrap();
    assert_eq!(
        e.document.paragraphs[0].spacing.tab_leader(400),
        Some(L::Dots)
    );
    e.set_tab_positions(0, 3, vec![(288, K::Left)]).unwrap();
    assert_eq!(e.document.paragraphs[0].spacing.tab_leader(400), None);
    assert_eq!(
        e.document.paragraphs[0].spacing.tab_leader(288),
        Some(L::Line)
    );
}
#[test]
fn leader_missing_position_rejects_atomically_and_noop_retains_redo() {
    let mut e = Editor::default();
    e.replace(0, 0, "a\nb", "Fixture").unwrap();
    e.set_tab_positions(0, 0, vec![(144, K::Left)]).unwrap();
    let before = e.document.clone();
    let n = e.undo_stack.len();
    assert!(e.set_tab_leader(0, 3, 144, Some(L::Dots)).is_err());
    assert_eq!(e.document, before);
    assert_eq!(e.undo_stack.len(), n);
    e.set_tab_leader(0, 0, 144, Some(L::Dots)).unwrap();
    let after = e.document.clone();
    e.undo();
    e.set_tab_leader(0, 0, 144, None).unwrap();
    e.redo();
    assert_eq!(e.document, after);
    e.set_tab_leader(0, 0, 144, None).unwrap();
    assert_eq!(e.document, before);
    e.undo();
    assert_eq!(e.document, after);
}
#[test]
fn leader_storage_rejects_invalid_data_and_preserves_legacy_reading() {
    let mut e = Editor::default();
    e.set_tab_positions(0, 0, vec![(144, K::Left), (288, K::Right)])
        .unwrap();
    let base: serde_json::Value =
        serde_json::from_slice(&storage::encode(&e.document).unwrap()).unwrap();
    assert_eq!(base["content_version"], 17);
    for bad in [
        serde_json::json!(null),
        serde_json::json!([{"position_half_points":144,"leader":"unknown"}]),
        serde_json::json!([{"position_half_points":144,"leader":"dots","extra":1}]),
        serde_json::json!([{"position_half_points":145,"leader":"dots"}]),
        serde_json::json!([{"position_half_points":144,"leader":"dots"},{"position_half_points":144,"leader":"line"}]),
        serde_json::json!([{"position_half_points":288,"leader":"line"},{"position_half_points":144,"leader":"dots"}]),
    ] {
        let mut j = base.clone();
        j["paragraphs"][0]["spacing"]["tab_leaders"] = bad;
        assert!(storage::decode(&serde_json::to_vec(&j).unwrap()).is_err());
    }
    for v in 1..=16 {
        for bad in [
            serde_json::json!([]),
            serde_json::json!(null),
            serde_json::json!([{"position_half_points":144,"leader":"dots"}]),
        ] {
            let mut j = base.clone();
            j["content_version"] = v.into();
            j["paragraphs"][0]["spacing"]["tab_leaders"] = bad;
            assert!(storage::decode(&serde_json::to_vec(&j).unwrap()).is_err());
        }
    }
    let mut old = base.clone();
    old["content_version"] = 16.into();
    assert_eq!(
        storage::decode(&serde_json::to_vec(&old).unwrap()).unwrap(),
        e.document
    );
    old["content_version"] = 18.into();
    assert!(storage::decode(&serde_json::to_vec(&old).unwrap()).is_err());
}

#[test]
fn leader_fixture_loads_and_failed_save_keeps_previous_bytes() {
    let doc = storage::decode(include_bytes!("../examples/TabLeaders.bfdoc")).unwrap();
    assert_eq!(doc.paragraphs.len(), 4);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("leaders.bfdoc");
    storage::save(&doc, &path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let mut bad = doc.clone();
    bad.paragraphs[0].spacing.tab_leaders[0].position_half_points = 1;
    assert!(storage::save(&bad, &path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(storage::load(&path).unwrap(), doc);
}
