use breakfree_docs::{
    commands::Editor,
    model::TabKind as K,
    storage,
    tab_ruler::{Draft, common_kind, positions},
};
#[test]
fn typed_dialog_transaction_validates_and_preserves_history() {
    let mut e = Editor::default();
    e.replace(0, 0, "a\tb\nc\td\nlast", "Fixture").unwrap();
    let before = e.document.clone();
    let count = e.undo_stack.len();
    let stops = vec![(72, K::Left), (144, K::Center), (288, K::Right)];
    e.set_tab_positions(0, 8, stops.clone()).unwrap();
    let after = e.document.clone();
    assert_eq!(e.undo_stack.len(), count + 1);
    assert_eq!(after.paragraphs[0].spacing.tab_positions(), stops);
    assert_eq!(after.paragraphs[1].spacing.tab_positions(), stops);
    assert_eq!(after.paragraphs[2], before.paragraphs[2]);
    assert_eq!(
        storage::decode(&storage::encode(&after).unwrap()).unwrap(),
        after
    );
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
    for bad in [
        vec![(72, K::Left), (72, K::Right)],
        vec![(144, K::Right), (72, K::Left)],
        vec![(0, K::Center)],
        (1..34).map(|p| (p, K::Right)).collect(),
    ] {
        assert!(e.set_tab_positions(0, 8, bad).is_err());
        assert_eq!(e.document, after);
    }
    e.undo();
    e.set_tab_positions(0, 8, vec![]).unwrap();
    e.redo();
    assert_eq!(e.document, after);
}
#[test]
fn mixed_kind_ruler_preserves_types_and_owners_in_one_transaction() {
    let mut e = Editor::default();
    e.replace(0, 0, "a\nb\nc", "Fixture").unwrap();
    e.set_tab_positions(0, 0, vec![(144, K::Left), (300, K::Right)])
        .unwrap();
    e.set_tab_positions(2, 2, vec![(144, K::Center), (400, K::Left)])
        .unwrap();
    e.set_tab_positions(4, 4, vec![(200, K::Right)]).unwrap();
    e.selection = (0, 5);
    let before = e.document.clone();
    let count = e.undo_stack.len();
    assert_eq!(common_kind(&before, e.selection, 144), None);
    assert_eq!(
        positions(&before, e.selection).unwrap(),
        vec![(144, 2), (200, 1), (300, 1), (400, 1)]
    );
    let mut d = Draft::new(&e, 144).unwrap();
    d.move_to_text("100").unwrap();
    assert_eq!(d.preview.paragraphs[2], before.paragraphs[2]);
    d.commit(&mut e).unwrap();
    let after = e.document.clone();
    assert_eq!(
        after.paragraphs[0].spacing.tab_positions(),
        vec![(200, K::Left), (300, K::Right)]
    );
    assert_eq!(
        after.paragraphs[1].spacing.tab_positions(),
        vec![(200, K::Center), (400, K::Left)]
    );
    assert_eq!(e.undo_stack.len(), count + 1);
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
    let mut d = Draft::new(&e, 200).unwrap();
    d.remove().unwrap();
    d.commit(&mut e).unwrap();
    assert!(e.document.paragraphs[2].spacing.tab_positions().is_empty());
    e.undo();
    assert_eq!(e.document, after);
}
#[test]
fn typed_ruler_conflict_cancel_and_stale_draft_are_atomic() {
    let mut e = Editor::default();
    e.set_tab_positions(0, 0, vec![(144, K::Center), (288, K::Right)])
        .unwrap();
    let before = e.document.clone();
    let count = e.undo_stack.len();
    let mut d = Draft::new(&e, 144).unwrap();
    assert_eq!(d.kind(), Some(K::Center));
    assert!(d.move_to(288.).is_err());
    assert_eq!(d.preview, before);
    assert!(d.commit(&mut e).is_err());
    assert_eq!(e.document, before);
    assert_eq!(e.undo_stack.len(), count);
    let mut d = Draft::new(&e, 288).unwrap();
    d.move_to(300.).unwrap();
    drop(d);
    assert_eq!(e.document, before);
    let mut d = Draft::new(&e, 288).unwrap();
    d.move_to(300.).unwrap();
    e.set_tab_positions(0, 0, vec![]).unwrap();
    assert!(d.commit(&mut e).is_err());
    e.undo();
    assert_eq!(e.document, before);
}

#[test]
fn typed_creation_missing_only_conflict_and_one_history() {
    for kind in [K::Left, K::Center, K::Right] {
        let mut e = Editor::default();
        e.replace(0, 0, "a\nb\nlast", "Fixture").unwrap();
        e.set_tab_positions(0, 0, vec![(144, kind)]).unwrap();
        e.selection = (0, 4);
        let before = e.document.clone();
        let count = e.undo_stack.len();
        let mut d = Draft::creation_preview_typed(&e, kind).unwrap();
        assert_eq!(d.kind(), Some(kind));
        d.move_to_text("72").unwrap();
        assert_eq!(e.document, before);
        d.commit(&mut e).unwrap();
        let after = e.document.clone();
        assert_eq!(e.undo_stack.len(), count + 1);
        assert_eq!(
            after.paragraphs[1].spacing.tab_positions(),
            vec![(144, kind)]
        );
        assert_eq!(after.paragraphs[2], before.paragraphs[2]);
        assert_eq!(
            storage::decode(&storage::encode(&after).unwrap()).unwrap(),
            after
        );
        e.undo();
        assert_eq!(e.document, before);
        e.redo();
        assert_eq!(e.document, after);
        assert!(Draft::create_typed(&e, 144, kind).is_err());
        let other = if kind == K::Left { K::Right } else { K::Left };
        assert!(Draft::create_typed(&e, 144, other).is_err());
        assert_eq!(e.document, after);
    }
}
#[test]
fn typed_creation_capacity_invalid_and_cancel_preserve_redo() {
    let mut e = Editor::default();
    e.set_tab_positions(0, 0, (1..=32).map(|p| (p, K::Right)).collect())
        .unwrap();
    let before = e.document.clone();
    assert!(Draft::create_typed(&e, 100, K::Center).is_err());
    assert!(Draft::create_typed(&e, 0, K::Right).is_err());
    assert!(Draft::create_typed(&e, 2881, K::Center).is_err());
    let mut d = Draft::creation_preview_typed(&e, K::Right).unwrap();
    assert!(d.move_to_text("100").is_err());
    assert!(d.commit(&mut e).is_err());
    assert_eq!(e.document, before);
    e.undo();
    let empty = e.document.clone();
    let mut d = Draft::creation_preview_typed(&e, K::Center).unwrap();
    d.move_to_text("100").unwrap();
    drop(d);
    assert_eq!(e.document, empty);
    e.redo();
    assert_eq!(e.document, before);
}

#[test]
fn mixed_typed_draft_recovers_from_conflict_delete_and_numeric_error() {
    for kind in [K::Left, K::Center, K::Right] {
        let mut e = Editor::default();
        e.replace(0, 0, "English\nУкраїна\nРусский\ne\u{301}", "Fixture")
            .unwrap();
        e.set_tab_positions(0, 0, vec![(72, kind)]).unwrap();
        e.set_tab_positions(8, 8, vec![(72, K::Right), (73, K::Center)])
            .unwrap();
        e.selection = (0, 16); // Exclude the paragraph beginning at this boundary.
        let before = e.document.clone();
        let count = e.undo_stack.len();
        let mut d = Draft::new(&e, 72).unwrap();
        assert!(d.move_to(73.).is_err());
        assert_eq!(d.preview, before); // The first owner's valid edit must not leak.
        d.remove().unwrap();
        assert!(d.deleting);
        assert_eq!(
            d.preview.paragraphs[1].spacing.tab_positions(),
            vec![(73, K::Center)]
        );
        assert!(d.move_to_text("nan").is_err());
        assert!(!d.deleting);
        assert_eq!(d.preview, before);
        d.move_to_text("37").unwrap();
        d.commit(&mut e).unwrap();
        let after = e.document.clone();
        assert_eq!(
            after.paragraphs[0].spacing.tab_positions(),
            vec![(74, kind)]
        );
        assert_eq!(
            after.paragraphs[1].spacing.tab_positions(),
            vec![(73, K::Center), (74, K::Right)]
        );
        assert_eq!(after.paragraphs[2..], before.paragraphs[2..]);
        assert_eq!(e.undo_stack.len(), count + 1);
        assert_eq!(
            storage::decode(&storage::encode(&after).unwrap()).unwrap(),
            after
        );
        e.undo();
        assert_eq!(e.document, before);
        e.redo();
        assert_eq!(e.document, after);
    }
}

#[test]
fn typed_draft_selection_change_and_noop_preserve_redo() {
    for kind in [K::Left, K::Center, K::Right] {
        let mut e = Editor::default();
        e.replace(0, 0, "a\nb", "Fixture").unwrap();
        e.set_tab_positions(0, 0, vec![(72, kind)]).unwrap();
        e.selection = (0, 0);
        let before = e.document.clone();
        let mut d = Draft::new(&e, 72).unwrap();
        d.move_to(74.).unwrap();
        e.selection = (2, 2);
        assert!(d.commit(&mut e).is_err());
        assert_eq!(e.document, before);
        e.selection = (0, 0);
        let mut d = Draft::new(&e, 72).unwrap();
        d.move_to(74.).unwrap();
        d.commit(&mut e).unwrap();
        let after = e.document.clone();
        e.undo();
        let count = e.undo_stack.len();
        let mut d = Draft::new(&e, 72).unwrap();
        d.move_to(72.).unwrap();
        d.commit(&mut e).unwrap();
        assert_eq!(e.undo_stack.len(), count);
        assert_eq!(e.document, before);
        e.redo();
        assert_eq!(e.document, after);
    }
}
