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
