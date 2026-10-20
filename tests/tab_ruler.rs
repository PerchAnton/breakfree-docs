use breakfree_docs::{commands::Editor, tab_ruler::Draft};
fn editor() -> Editor {
    let mut e = Editor::default();
    e.replace(0, 0, "a\tb\nc\td\nlast", "Fixture").unwrap();
    e.set_left_tab_stops(0, 8, vec![72, 144, 216]).unwrap();
    e.selection = (0, 8);
    e
}
#[test]
fn preview_one_history_entry_and_half_open_selection() {
    let mut e = editor();
    let before = e.document.clone();
    let count = e.undo_stack.len();
    let mut d = Draft::new(&e, 72).unwrap();
    d.move_to(96.).unwrap();
    d.move_to(180.).unwrap();
    assert_eq!(e.document, before);
    assert_eq!(e.undo_stack.len(), count);
    assert_eq!(
        d.preview.paragraphs[0].spacing.left_tab_stops_half_points,
        [144, 180, 216]
    );
    assert_eq!(d.preview.paragraphs[2], before.paragraphs[2]);
    d.commit(&mut e).unwrap();
    let after = e.document.clone();
    assert_eq!(
        breakfree_docs::storage::decode(&breakfree_docs::storage::encode(&after).unwrap()).unwrap(),
        after
    );
    assert_eq!(e.undo_stack.len(), count + 1);
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
}
#[test]
fn duplicate_stale_mixed_cancel_and_noop_preserve_data() {
    let mut e = editor();
    let before = e.document.clone();
    let mut d = Draft::new(&e, 72).unwrap();
    d.move_to(96.).unwrap();
    assert!(d.move_to(144.).is_err());
    assert!(d.commit(&mut e).is_err());
    assert_eq!(e.document, before);
    let mut d = Draft::new(&e, 72).unwrap();
    d.move_to(96.).unwrap();
    drop(d);
    assert_eq!(e.document, before);
    let mut d = Draft::new(&e, 72).unwrap();
    d.move_to(96.).unwrap();
    e.selection = (0, 0);
    assert!(d.commit(&mut e).is_err());
    e.selection = (0, 8);
    let mut d = Draft::new(&e, 72).unwrap();
    d.move_to(f32::NAN).unwrap_err();
    assert!(d.commit(&mut e).is_err());
    let d = Draft::new(&e, 72).unwrap();
    e.set_tab_interval(0, 0, Some(96)).unwrap();
    assert!(d.commit(&mut e).is_err());
    e.undo();
    Draft::new(&e, 72).unwrap().commit(&mut e).unwrap();
    e.redo();
    assert_eq!(
        e.document.paragraphs[0].spacing.tab_interval_half_points,
        Some(96)
    );
    e.set_left_tab_stops(4, 4, vec![72, 288]).unwrap();
    e.selection = (0, 8);
    assert!(Draft::new(&e, 72).is_err());
}
#[test]
fn clamped_bounds_rounding_and_empty_source_rejection() {
    let e = editor();
    assert!(Draft::new(&e, 1).is_err());
    let mut d = Draft::new(&e, 72).unwrap();
    d.move_to(-100.).unwrap();
    assert_eq!(d.position, 1);
    d.move_to(4000.).unwrap();
    assert_eq!(d.position, 2880);
    d.move_to(96.4).unwrap();
    assert_eq!(d.position, 96);
}
