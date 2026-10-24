use breakfree_docs::{
    commands::Editor,
    pagination::{self, Measure},
    storage,
    structure::*,
};
fn editor(text: &str) -> Editor {
    let mut e = Editor::default();
    e.replace(0, 0, text, "Fixture").unwrap();
    e.undo_stack.clear();
    e
}
#[test]
fn lists_split_exit_levels_restart_and_history() {
    let mut e = editor("One\nДва\nТри");
    e.selection = (0, 7);
    e.set_list(Some(ListKind::Decimal)).unwrap();
    assert_eq!(
        labels(&e.document),
        vec![Some("1.".into()), Some("2.".into()), None]
    );
    let before = e.document.clone();
    e.selection = (4, 4);
    e.change_list_level(true).unwrap();
    assert_eq!(
        e.document.paragraphs[1]
            .structure
            .list
            .as_ref()
            .unwrap()
            .level,
        1
    );
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    e.restart_list(5).unwrap();
    assert_eq!(labels(&e.document)[1], Some("5.".into()));
    let mut e = editor("item");
    e.set_list(Some(ListKind::Bullet)).unwrap();
    e.insert_paragraph_break(4, 4).unwrap();
    assert!(e.document.paragraphs[1].structure.list.is_some());
    let before = e.document.clone();
    e.insert_paragraph_break(5, 5).unwrap();
    assert_eq!(e.document.text(), "item\n");
    assert!(e.document.paragraphs[1].structure.list.is_none());
    e.undo();
    assert_eq!(e.document, before);
}
#[test]
fn link_boundaries_unicode_edits_replace_all_and_history() {
    let mut e = editor("Read Україна e\u{301} now");
    e.selection = (5, 12);
    e.set_link(Some("https://example.com")).unwrap();
    let before = e.document.clone();
    e.replace(7, 8, "XY", "Edit").unwrap();
    assert_eq!(e.document.paragraphs[0].structure.links[0].end, 13);
    e.undo();
    assert_eq!(e.document, before);
    let plan = e.preview_replace_all(&[(5, 12)], "English").unwrap();
    e.apply_replace_all(plan).unwrap();
    assert_eq!(
        e.document.paragraphs[0].structure.links[0].target,
        "https://example.com"
    );
    e.undo();
    assert_eq!(e.document, before);
    e.selection = (13, 14);
    assert!(e.set_link(Some("https://example.com")).is_err());
    e.selection = (5, 12);
    for target in ["javascript:alert(1)", "file:///tmp/a", "https://bad host"] {
        assert!(e.set_link(Some(target)).is_err());
    }
    e.set_link(None).unwrap();
    assert!(e.document.paragraphs[0].structure.links.is_empty());
    e.undo();
    assert_eq!(e.document, before);
}
#[test]
fn table_insert_edit_undo_reopen_and_preserve_on_failed_save() {
    let mut e = editor("Report");
    let mut table = Table::new(2, 2).unwrap();
    table.rows[0] = vec!["English".into(), "Україна".into()];
    table.rows[1] = vec!["Русский".into(), "e\u{301}".into()];
    table.repeat_header = true;
    let original = e.document.clone();
    e.insert_block(Block::Table(table.clone())).unwrap();
    let inserted = e.document.clone();
    assert_eq!(e.document.word_count(), 5);
    e.undo();
    assert_eq!(e.document, original);
    e.redo();
    assert_eq!(e.document, inserted);
    let bytes = storage::encode(&e.document).unwrap();
    assert_eq!(storage::decode(&bytes).unwrap(), e.document);
    table.rows.push(vec!["Added".into(), "row".into()]);
    let depth = e.undo_stack.len();
    e.update_block(1, Block::Table(table)).unwrap();
    assert_eq!(e.undo_stack.len(), depth + 1);
    e.undo();
    assert_eq!(e.document, inserted);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("report.bfdoc");
    storage::save(&e.document, &path).unwrap();
    let saved = std::fs::read(&path).unwrap();
    for stage in [
        storage::SaveStage::Write,
        storage::SaveStage::Sync,
        storage::SaveStage::Replace,
    ] {
        assert!(
            storage::save_checked(&e.document, &path, |s| if s == stage {
                Err("injected".into())
            } else {
                Ok(())
            })
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), saved);
    }
    let mut bad: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    bad["paragraphs"][1]["structure"]["block"]["rows"][0] = serde_json::json!(["wrong length"]);
    assert!(storage::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
}
#[test]
fn metadata_survives_unrelated_replace_all_and_blocks_reject_partial_merges() {
    let mut e = editor("first\nsecond");
    e.selection = (6, 12);
    e.set_list(Some(ListKind::Bullet)).unwrap();
    e.set_flow(true, true, false).unwrap();
    e.insert_block(Block::Table(Table::new(1, 1).unwrap()))
        .unwrap();
    let original = e.document.clone();
    let plan = e.preview_replace_all(&[(0, 5)], "start").unwrap();
    assert!(plan.result().paragraphs[1].structure.page_break_before);
    assert!(plan.result().paragraphs[2].structure.block.is_some());
    e.apply_replace_all(plan).unwrap();
    e.undo();
    assert_eq!(e.document, original);
    let object_start = e.document.paragraphs[..2]
        .iter()
        .map(|p| p.styles.len() + 1)
        .sum();
    assert!(
        e.replace(object_start, object_start, "bad", "Input")
            .is_err()
    );
    assert_eq!(e.document, original);
    e.replace(object_start, object_start + 1, "plain", "Replace object")
        .unwrap();
    assert!(e.document.paragraphs[2].structure.block.is_none());
    e.undo();
    assert_eq!(e.document, original);
}
#[test]
fn page_settings_sections_roundtrip_validation_and_undo() {
    let mut e = editor("A\nB");
    let before = e.document.clone();
    let mut page = Section {
        width_points: 595,
        height_points: 842,
        page_numbers: true,
        header: "Report".into(),
        footer: "Local document".into(),
        ..Default::default()
    };
    e.set_page(page.clone(), false).unwrap();
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    e.selection = (2, 2);
    page.width_points = 842;
    page.height_points = 595;
    page.number_start = Some(10);
    e.set_page(page, true).unwrap();
    assert_eq!(pagination::sections(&e.document)[1].width_points, 842);
    assert_eq!(
        storage::decode(&storage::encode(&e.document).unwrap()).unwrap(),
        e.document
    );
    let before = e.document.clone();
    assert!(
        e.set_page(
            Section {
                left_points: 600,
                ..Default::default()
            },
            false
        )
        .is_err()
    );
    assert_eq!(before, e.document);
    e.remove_section().unwrap();
    e.undo();
    assert_eq!(before, e.document);
}
#[test]
fn strict_v19_gate_and_legacy_defaults() {
    let e = editor("text");
    let mut value: serde_json::Value =
        serde_json::from_slice(&storage::encode(&e.document).unwrap()).unwrap();
    assert_eq!(value["content_version"], storage::CONTENT_VERSION);
    value["content_version"] = 18.into();
    assert!(storage::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    value.as_object_mut().unwrap().remove("page");
    value["paragraphs"][0]
        .as_object_mut()
        .unwrap()
        .remove("structure");
    assert_eq!(
        storage::decode(&serde_json::to_vec(&value).unwrap()).unwrap(),
        e.document
    );
    value["content_version"] = 19.into();
    assert!(storage::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    assert!(
        Picture {
            png: vec![],
            width_points: 100,
            height_points: 100,
            alt: String::new()
        }
        .validate()
        .is_err()
    );
}
#[test]
fn pagination_splits_rows_keeps_widows_sections_and_repeated_headers() {
    let mut e = editor("a\nb");
    e.document.page = Section {
        height_points: 200,
        top_points: 25,
        bottom_points: 25,
        ..Default::default()
    };
    let m = vec![
        Measure {
            rows: vec![30.; 4],
            before: 0.,
            after: 0.,
        },
        Measure {
            rows: vec![30.; 3],
            before: 0.,
            after: 0.,
        },
    ];
    let plan = pagination::paginate(&e.document, &m).unwrap();
    assert_eq!(plan.pages.len(), 2);
    assert_eq!(plan.pages[0].items.len(), 4);
    assert_eq!(plan.pages[1].items.len(), 3);
    e.document.paragraphs[1].structure.section = Some(Section {
        number_start: Some(7),
        ..e.document.page.clone()
    });
    let plan = pagination::paginate(&e.document, &m).unwrap();
    assert_eq!(plan.pages[1].number, 7);
    let mut e = editor("");
    let mut table = Table::new(5, 1).unwrap();
    table.repeat_header = true;
    e.insert_block(Block::Table(table)).unwrap();
    e.document.page = Section {
        height_points: 200,
        top_points: 25,
        bottom_points: 25,
        ..Default::default()
    };
    let m = vec![
        Measure {
            rows: vec![12.],
            before: 0.,
            after: 0.,
        },
        Measure {
            rows: vec![40.; 5],
            before: 0.,
            after: 0.,
        },
        Measure {
            rows: vec![12.],
            before: 0.,
            after: 0.,
        },
    ];
    let plan = pagination::paginate(&e.document, &m).unwrap();
    assert_eq!(plan.pages.len(), 2);
    assert!(plan.pages[1].items[0].repeated_header);
    assert_eq!(
        plan.pages
            .iter()
            .flat_map(|p| &p.items)
            .filter(|p| p.paragraph == 1 && !p.repeated_header)
            .count(),
        5
    );
}

#[test]
fn keep_chain_respects_explicit_breaks_and_moves_as_a_group() {
    let mut e = editor("Previous\nHeading\nSubheading\nBody");
    e.document.page.height_points = 244;
    e.document.paragraphs[1].structure.keep_with_next = true;
    e.document.paragraphs[2].structure.keep_with_next = true;
    let m: Vec<_> = [60., 20., 20., 20.]
        .into_iter()
        .map(|h| Measure {
            rows: vec![h],
            before: 0.,
            after: 0.,
        })
        .collect();
    let plan = pagination::paginate(&e.document, &m).unwrap();
    assert_eq!(plan.pages.len(), 2);
    assert_eq!(plan.pages[0].items.len(), 1);
    assert_eq!(plan.pages[1].items.len(), 3);
    e.document.paragraphs[2].structure.page_break_before = true;
    let plan = pagination::paginate(&e.document, &m).unwrap();
    assert_eq!(plan.pages[0].items.len(), 2);
}

#[test]
fn outline_and_plain_object_copy_follow_document_order() {
    let mut e = editor("Heading\nBody");
    e.document.paragraphs[0].paragraph_style = breakfree_docs::model::ParagraphStyle::Heading1;
    assert_eq!(headings(&e.document), vec![(0, 7, "Heading".into())]);
    let mut t = Table::new(1, 2).unwrap();
    t.rows[0] = vec!["A".into(), "B".into()];
    e.selection = (8, 8);
    e.insert_block(Block::Table(t)).unwrap();
    let copied =
        breakfree_docs::presentation::selected_text(&e.document, (0, e.document.character_count()));
    assert!(copied.contains("A\tB"));
    assert!(!copied.contains(OBJECT));
}

#[test]
fn page_break_splits_at_caret_and_restores_selection_in_one_undo() {
    let mut e = editor("first second");
    e.selection = (6, 6);
    let before = e.document.clone();
    e.insert_page_break().unwrap();
    assert_eq!(e.document.text(), "first \nsecond");
    assert!(e.document.paragraphs[1].structure.page_break_before);
    assert_eq!(e.undo_stack.len(), 1);
    let after = e.document.clone();
    e.undo();
    assert_eq!(e.document, before);
    assert_eq!(e.selection, (6, 6));
    e.redo();
    assert_eq!(e.document, after);
    assert_eq!(e.selection, (7, 7));
}
