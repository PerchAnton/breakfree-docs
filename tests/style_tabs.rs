use breakfree_docs::{commands::Editor, model::*, storage, tab_ruler};
fn stop(position: u16, kind: TabKind, leader: Option<TabLeader>) -> StyleTabStop {
    StyleTabStop {
        position_half_points: position,
        kind,
        leader,
    }
}
fn fixture() -> Editor {
    let mut e = Editor::default();
    e.replace(
        0,
        0,
        "English\tText\nУкраїнська\tе\u{301}\nРусский\tтекст",
        "Fixture",
    )
    .unwrap();
    e.document.style_catalog.normal.tab_interval_half_points = Some(96);
    e.document.style_catalog.normal.tab_stops = Some(vec![
        stop(144, TabKind::Center, Some(TabLeader::Dots)),
        stop(288, TabKind::Right, Some(TabLeader::Line)),
    ]);
    e.document.paragraphs[0].paragraph_style = ParagraphStyle::Heading1;
    e.document.paragraphs[1].paragraph_style = ParagraphStyle::NoSpacing;
    e.document.validate().unwrap();
    e
}
fn tabs(e: &Editor, index: usize) -> ParagraphSpacing {
    e.document.resolved_tabs(&e.document.paragraphs[index])
}

#[test]
fn nearest_ancestor_resolves_independent_interval_and_complete_stop_list() {
    let mut e = fixture();
    assert_eq!(tabs(&e, 0).tab_interval_half_points, Some(96));
    assert_eq!(
        tabs(&e, 0).tab_positions(),
        vec![(144, TabKind::Center), (288, TabKind::Right)]
    );
    e.document.style_catalog.heading1.tab_interval_half_points = Some(120);
    e.document.style_catalog.heading1.tab_stops = Some(vec![]);
    assert_eq!(tabs(&e, 0).tab_interval_half_points, Some(120));
    assert!(tabs(&e, 0).tab_positions().is_empty());
    assert_eq!(tabs(&e, 1).tab_positions().len(), 2);
    e.document.style_catalog.heading1.tab_stops = Some(vec![stop(72, TabKind::Left, None)]);
    assert_eq!(tabs(&e, 0).tab_positions(), [(72, TabKind::Left)]);
    e.document.style_catalog.heading1.parent = None;
    e.document.style_catalog.heading1.tab_interval_half_points = None;
    assert_eq!(tabs(&e, 0).tab_interval_half_points, Some(72));
}

#[test]
fn direct_settings_win_and_independent_resets_are_single_undo_steps() {
    let mut e = fixture();
    e.set_tab_interval(0, 0, Some(72)).unwrap();
    e.set_tab_settings(0, 0, vec![(80, TabKind::Left)], vec![])
        .unwrap();
    let before = e.document.clone();
    let depth = e.undo_stack.len();
    e.inherit_tab_stops(0, 0).unwrap();
    assert_eq!(e.undo_stack.len(), depth + 1);
    assert_eq!(tabs(&e, 0).tab_positions().len(), 2);
    assert_eq!(tabs(&e, 0).tab_interval_half_points, Some(72));
    let after = e.document.clone();
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
    e.set_tab_interval(0, 0, None).unwrap();
    assert_eq!(tabs(&e, 0).tab_interval_half_points, Some(96));
    e.undo();
    assert_eq!(e.document, after);
    e.redo();
    assert_eq!(tabs(&e, 0).tab_interval_half_points, Some(96));
    e.set_tab_positions(0, 0, vec![]).unwrap();
    assert!(tabs(&e, 0).tab_positions().is_empty());
    assert!(e.document.paragraphs[0].spacing.clear_tab_stops);
    e.inherit_tab_stops(0, 0).unwrap();
    assert_eq!(tabs(&e, 0).tab_positions().len(), 2);
}

#[test]
fn modify_style_preserves_overrides_selection_and_history() {
    let mut e = fixture();
    e.set_tab_positions(0, 0, vec![(60, TabKind::Left)])
        .unwrap();
    e.selection = (0, 3);
    let before = e.document.clone();
    let depth = e.undo_stack.len();
    let mut definition = before.style_catalog.normal.clone();
    definition.tab_interval_half_points = Some(40);
    definition.tab_stops = Some(vec![stop(200, TabKind::Right, Some(TabLeader::Dashes))]);
    e.modify_style(ParagraphStyle::Normal, definition).unwrap();
    let after = e.document.clone();
    assert_eq!(e.undo_stack.len(), depth + 1);
    assert_eq!(e.selection, (0, 3));
    assert_eq!(after.paragraphs, before.paragraphs);
    assert_eq!(tabs(&e, 0).tab_positions(), [(60, TabKind::Left)]);
    assert_eq!(tabs(&e, 1).tab_leader(200), Some(TabLeader::Dashes));
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
}

#[test]
fn ruler_reads_inherited_stops_and_only_materializes_changed_owners() {
    let mut e = fixture();
    let end = e.document.paragraphs[0].text.chars().count() + 1;
    e.selection = (0, end); // Exclusive next paragraph is not edited.
    assert_eq!(
        tab_ruler::positions(&e.document, e.selection).unwrap(),
        [(144, 1), (288, 1)]
    );
    assert_eq!(
        tab_ruler::leader_summary(&e.document, e.selection, 144)
            .unwrap()
            .common(),
        Some(Some(TabLeader::Dots))
    );
    let before = e.document.clone();
    let depth = e.undo_stack.len();
    let mut draft = tab_ruler::Draft::new(&e, 144).unwrap();
    draft.move_to(180.).unwrap();
    assert_eq!(e.document, before); // Cancel by dropping the draft.
    drop(draft);
    let mut draft = tab_ruler::Draft::new(&e, 144).unwrap();
    draft.move_to(144.).unwrap();
    draft.commit(&mut e).unwrap();
    assert_eq!(e.document, before);
    assert_eq!(e.undo_stack.len(), depth);
    let mut draft = tab_ruler::Draft::new(&e, 144).unwrap();
    draft.move_to(180.).unwrap();
    draft.commit(&mut e).unwrap();
    let after = e.document.clone();
    assert_eq!(e.undo_stack.len(), depth + 1);
    assert_eq!(tabs(&e, 0).tab_leader(180), Some(TabLeader::Dots));
    assert_eq!(after.paragraphs[1..], before.paragraphs[1..]);
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, after);
}

#[test]
fn mixed_owner_removal_leader_edits_and_stale_draft_are_safe() {
    let mut e = fixture();
    e.document.style_catalog.no_spacing.tab_stops = Some(vec![]);
    e.selection = (0, e.document.character_count());
    let before = e.document.clone();
    e.set_owned_tab_leader(0, e.document.character_count(), 144, Some(TabLeader::Line))
        .unwrap();
    assert_eq!(e.document.paragraphs[1], before.paragraphs[1]);
    assert_eq!(tabs(&e, 0).tab_leader(144), Some(TabLeader::Line));
    e.undo();
    assert_eq!(e.document, before);
    let mut draft = tab_ruler::Draft::new(&e, 144).unwrap();
    draft.remove().unwrap();
    draft.commit(&mut e).unwrap();
    assert_eq!(e.document.paragraphs[1], before.paragraphs[1]);
    e.undo();
    assert_eq!(e.document, before);
    let draft = tab_ruler::Draft::new(&e, 144).unwrap();
    let mut d = e.document.style_catalog.normal.clone();
    d.tab_interval_half_points = Some(60);
    e.modify_style(ParagraphStyle::Normal, d).unwrap();
    let changed = e.document.clone();
    assert!(draft.commit(&mut e).is_err());
    assert_eq!(e.document, changed);
}

#[test]
fn invalid_style_lists_and_cycles_leave_redo_intact() {
    let mut e = fixture();
    e.set_tab_interval(0, 0, Some(48)).unwrap();
    e.undo();
    let before = e.document.clone();
    let depth = e.redo_stack.len();
    for list in [
        vec![stop(0, TabKind::Left, None)],
        vec![stop(2881, TabKind::Right, None)],
        vec![
            stop(80, TabKind::Left, None),
            stop(80, TabKind::Right, None),
        ],
        vec![stop(90, TabKind::Left, None), stop(80, TabKind::Left, None)],
        (1..=33).map(|i| stop(i, TabKind::Left, None)).collect(),
    ] {
        let mut d = before.style_catalog.normal.clone();
        d.tab_stops = Some(list);
        assert!(e.modify_style(ParagraphStyle::Normal, d).is_err());
    }
    let mut d = before.style_catalog.normal.clone();
    d.parent = Some(ParagraphStyle::Heading1);
    assert!(e.modify_style(ParagraphStyle::Normal, d).is_err());
    let mut d = before.style_catalog.normal.clone();
    d.tab_interval_half_points = Some(289);
    assert!(e.modify_style(ParagraphStyle::Normal, d).is_err());
    assert_eq!(e.document, before);
    assert_eq!(e.redo_stack.len(), depth);
}

#[test]
fn native_roundtrip_strict_version_gates_and_failed_writes() {
    let mut e = fixture();
    e.set_tab_positions(0, 0, vec![]).unwrap();
    let bytes = storage::encode(&e.document).unwrap();
    assert_eq!(storage::decode(&bytes).unwrap(), e.document);
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["content_version"], 18);
    for v in 1..=17 {
        let mut old = value.clone();
        old["content_version"] = v.into();
        assert!(storage::decode(&serde_json::to_vec(&old).unwrap()).is_err());
    }
    for (pointer, bad) in [
        (
            "/style_catalog/normal/tab_interval_half_points",
            serde_json::json!(0),
        ),
        (
            "/style_catalog/normal/tab_stops",
            serde_json::json!([{"position_half_points":72,"kind":"decimal","leader":null}]),
        ),
        (
            "/style_catalog/normal/tab_stops",
            serde_json::json!([{"position_half_points":72,"kind":"left","leader":"unknown"}]),
        ),
        (
            "/paragraphs/0/spacing/clear_tab_stops",
            serde_json::json!(null),
        ),
        ("/content_version", serde_json::json!(19)),
        ("/format", serde_json::json!("foreign")),
    ] {
        let mut bad_file = value.clone();
        *bad_file.pointer_mut(pointer).unwrap() = bad;
        assert!(storage::decode(&serde_json::to_vec(&bad_file).unwrap()).is_err());
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tabs.bfdoc");
    storage::save(&e.document, &path).unwrap();
    let previous = std::fs::read(&path).unwrap();
    e.set_tab_interval(0, 0, Some(100)).unwrap();
    for stage in [
        storage::SaveStage::Write,
        storage::SaveStage::Sync,
        storage::SaveStage::Replace,
    ] {
        assert!(
            storage::save_checked(&e.document, &path, |at| if at == stage {
                Err("Injected failure".into())
            } else {
                Ok(())
            })
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), previous);
    }
    assert_eq!(
        storage::load(&path).unwrap(),
        storage::decode(&previous).unwrap()
    );
}

#[test]
fn legacy_v17_preserves_rendered_settings_and_accepts_future_style_edits() {
    let doc = storage::decode(include_bytes!("../examples/TabLeaders.bfdoc")).unwrap();
    for p in &doc.paragraphs {
        let tabs = doc.resolved_tabs(p);
        assert_eq!(tabs.tab_positions(), p.spacing.tab_positions());
        assert_eq!(tabs.tab_leaders, p.spacing.tab_leaders);
        assert_eq!(
            tabs.tab_interval_half_points,
            Some(p.spacing.tab_interval_half_points.unwrap_or(72))
        );
    }
    assert_eq!(
        storage::decode(&storage::encode(&doc).unwrap()).unwrap(),
        doc
    );
}

#[test]
fn enter_and_custom_style_creation_preserve_authored_inheritance() {
    let mut e = fixture();
    e.selection = (0, 0);
    let before = e.document.clone();
    let id = e
        .create_style(
            "Inherited tabs",
            StyleDefinition {
                parent: Some(ParagraphStyle::Normal),
                ..Default::default()
            },
            true,
        )
        .unwrap();
    let created = e.document.clone();
    assert_eq!(tabs(&e, 0).tab_leader(144), Some(TabLeader::Dots));
    assert!(
        e.document
            .style_catalog
            .get(id)
            .unwrap()
            .tab_stops
            .is_none()
    );
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document, created);
    e.insert_paragraph_break(3, 3).unwrap();
    assert_eq!(e.document.paragraphs[1].paragraph_style, id);
    assert!(!e.document.paragraphs[1].spacing.has_direct_tab_stops());
    assert_eq!(tabs(&e, 1).tab_positions(), tabs(&e, 0).tab_positions());
    e.undo();
    assert_eq!(e.document, created);
    let redo = e.redo_stack.len();
    e.inherit_tab_stops(0, 0).unwrap();
    e.set_tab_positions(0, 0, tabs(&e, 0).tab_positions())
        .unwrap();
    assert_eq!(e.redo_stack.len(), redo);
    assert_eq!(e.document, created);
}

#[test]
fn inherited_fixture_roundtrips_with_explicit_empty_and_direct_overrides() {
    let d = storage::decode(include_bytes!("../examples/InheritedTabs.bfdoc")).unwrap();
    assert_eq!(
        d.resolved_tabs(&d.paragraphs[0]).style_tab_stops(),
        d.resolved_tabs(&d.paragraphs[1]).style_tab_stops()
    );
    assert_ne!(
        d.resolved_tabs(&d.paragraphs[0]).style_tab_stops(),
        d.resolved_tabs(&d.paragraphs[2]).style_tab_stops()
    );
    assert!(d.resolved_tabs(&d.paragraphs[3]).tab_positions().is_empty());
    assert_eq!(storage::decode(&storage::encode(&d).unwrap()).unwrap(), d);
}
