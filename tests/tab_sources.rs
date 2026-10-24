use breakfree_docs::{
    commands::Editor,
    model::*,
    storage,
    tab_sources::{self, Source},
};
fn fixture() -> Editor {
    let mut e = Editor::default();
    e.replace(0, 0, "Україна\te\u{301}\nРусский\tтекст\nThird", "Fixture")
        .unwrap();
    e.document.style_catalog.normal.tab_interval_half_points = Some(96);
    e.document.style_catalog.normal.tab_stops = Some(vec![StyleTabStop {
        position_half_points: 180,
        kind: TabKind::Right,
        leader: Some(TabLeader::Dots),
    }]);
    e.document.paragraphs[0].paragraph_style = ParagraphStyle::Heading1;
    e.document.paragraphs[1].paragraph_style = ParagraphStyle::Heading1;
    e.selection = (0, e.document.text().chars().count());
    e
}
#[test]
fn tab_sources_resolve_independently_and_do_not_infer_from_equal_values() {
    let mut e = fixture();
    e.document.style_catalog.heading1.tab_interval_half_points = Some(96);
    e.document.paragraphs[1].spacing.tab_interval_half_points = Some(96);
    let r = tab_sources::inspect(&e.document, e.selection, None).unwrap();
    assert_eq!(r.interval.row("Interval").value, "48 pt");
    assert_eq!(r.interval.row("Interval").source, "Mixed sources");
    assert_eq!(r.interval.groups.len(), 3);
    assert_eq!(
        r.positions.groups[0].source,
        Source::Style(ParagraphStyle::Normal, true)
    );
    assert_eq!(r.positions.groups[0].count, 2);
    assert!(r.positions.groups[0].value.contains("90 pt · Right · Dots"));
}
#[test]
fn tab_sources_distinguish_explicit_empty_default_and_inherited_empty() {
    let mut e = fixture();
    e.document.style_catalog.heading1.tab_stops = Some(vec![]);
    e.document.paragraphs[1].spacing.clear_tab_stops = true;
    e.document.style_catalog.normal.tab_stops = None;
    let r = tab_sources::inspect(&e.document, e.selection, None).unwrap();
    assert_eq!(
        r.positions.row("Positions").value,
        "No explicit positions (interval grid only)"
    );
    assert_eq!(r.positions.groups.len(), 3);
    assert_eq!(
        r.positions.groups[0].source,
        Source::Style(ParagraphStyle::Heading1, false)
    );
    assert_eq!(r.positions.groups[1].source, Source::Direct);
    assert_eq!(r.positions.groups[2].source, Source::Default);
}
#[test]
fn tab_sources_respect_half_open_selection_empty_paragraphs_and_marker_owners() {
    assert_eq!(
        tab_sources::inspect(&Document::default(), (0, 0), None)
            .unwrap()
            .count,
        1
    );
    let mut e = fixture();
    let second = e.document.paragraphs[0].text.chars().count() + 1;
    assert_eq!(
        tab_sources::inspect(&e.document, (0, second), None)
            .unwrap()
            .count,
        1
    );
    assert_eq!(
        tab_sources::inspect(
            &e.document,
            (
                e.document.text().chars().count(),
                e.document.text().chars().count()
            ),
            None
        )
        .unwrap()
        .count,
        1
    );
    e.document.paragraphs[1].spacing.clear_tab_stops = true;
    let r = tab_sources::inspect(&e.document, e.selection, Some(180)).unwrap();
    assert_eq!((r.count, r.absent), (2, 1));
    assert_eq!(
        tab_sources::inspect(&e.document, e.selection, Some(999))
            .unwrap()
            .count,
        0
    );
    assert!(tab_sources::inspect(&e.document, (2, 1), None).is_err());
}
#[test]
fn tab_sources_follow_style_rename_history_roundtrip_without_writes() {
    let mut e = fixture();
    let parent = e
        .create_style(
            "Джерело e\u{301}",
            StyleDefinition {
                parent: Some(ParagraphStyle::Normal),
                tab_stops: Some(vec![]),
                ..Default::default()
            },
            true,
        )
        .unwrap();
    let before = tab_sources::inspect(&e.document, e.selection, None).unwrap();
    let def = e.document.style_catalog.get(parent).unwrap().clone();
    e.modify_custom_style(parent, "Source", def).unwrap();
    let after = tab_sources::inspect(&e.document, e.selection, None).unwrap();
    assert_ne!(
        before.positions.groups[0].label,
        after.positions.groups[0].label
    );
    let bytes = storage::encode(&e.document).unwrap();
    let depth = e.undo_stack.len();
    assert_eq!(
        tab_sources::inspect(&storage::decode(&bytes).unwrap(), e.selection, None).unwrap(),
        after
    );
    assert_eq!(e.undo_stack.len(), depth);
    e.undo();
    assert_eq!(
        tab_sources::inspect(&e.document, e.selection, None).unwrap(),
        before
    );
    e.redo();
    assert_eq!(
        tab_sources::inspect(&e.document, e.selection, None).unwrap(),
        after
    );
}
#[test]
fn tab_sources_default_cycle_guard_and_same_length_different_lists() {
    let mut e = fixture();
    e.document.paragraphs[1].spacing.left_tab_stops_half_points = vec![180];
    let r = tab_sources::inspect(&e.document, e.selection, None).unwrap();
    assert_eq!(r.positions.row("Positions").value, "Mixed");
    let mut doc = Document::default();
    doc.style_catalog.normal.parent = Some(ParagraphStyle::Heading1);
    assert_eq!(
        tab_sources::paragraph(&doc, &doc.paragraphs[0])
            .interval
            .groups[0]
            .source,
        Source::Default
    );
}
