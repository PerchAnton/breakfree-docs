//! Event-loop tests on the actual Rust widgets, without a native window or OS clipboard.
use super::*;
use crate::model::{CharacterFormat, Document};
use eframe::egui::Pos2;

fn frame(
    ctx: &egui::Context,
    app: &mut DocsApp,
    events: Vec<egui::Event>,
    physical: [f32; 2],
    dpi: f32,
) {
    let mut input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            vec2(physical[0] / dpi, physical[1] / dpi),
        )),
        events,
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .native_pixels_per_point = Some(dpi);
    let _ = ctx.run(input, |ctx| app.update_ui(ctx));
}
fn setup() -> (egui::Context, DocsApp) {
    let ctx = egui::Context::default();
    let mut app = DocsApp::new(Session::default(), &ctx);
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    }
    (ctx, app)
}
fn key(key: Key, command: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers {
            command,
            ctrl: command,
            ..Modifiers::NONE
        },
    }
}

#[test]
fn native_widget_edit_format_history_save_reopen_cycle() {
    let (ctx, mut app) = setup();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste(
            "English\nУкраїнська\nРусский e\u{301} 🙂".into(),
        )],
        [1366., 900.],
        1.,
    );
    assert_eq!(
        app.session.editor.document.text(),
        "English\nУкраїнська\nРусский e\u{301} 🙂"
    );
    assert_eq!(app.session.editor.undo_stack.len(), 1);
    app.execute(Action::SelectAll, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    frame(&ctx, &mut app, vec![key(Key::B, true)], [1366., 900.], 1.);
    assert!(
        app.session
            .editor
            .document
            .paragraphs
            .iter()
            .flat_map(|p| &p.styles)
            .all(|s| s.bold.unwrap_or(false))
    );
    let bold = app.session.editor.document.clone();
    app.execute(Action::Center, &ctx);
    let centered = app.session.editor.document.clone();
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    frame(&ctx, &mut app, vec![key(Key::Z, true)], [1366., 900.], 1.);
    assert_eq!(app.session.editor.document, bold);
    frame(&ctx, &mut app, vec![key(Key::Y, true)], [1366., 900.], 1.);
    assert_eq!(app.session.editor.document, centered);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("GUI cycle.bfdoc");
    app.session.path = Some(path.clone());
    app.execute(Action::Save, &ctx);
    assert!(!app.session.dirty());
    app.finish(Pending::New, &ctx);
    app.finish(Pending::Open(path), &ctx);
    assert_eq!(app.session.editor.document, centered);
    app.execute(Action::SelectAll, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Replacement".into())],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.session.editor.document.text(), "Replacement");
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, centered);
}

#[test]
fn ime_preedit_is_transient_and_commit_is_one_model_command() {
    let (ctx, mut app) = setup();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Ime(egui::ImeEvent::Enabled)],
        [1366., 900.],
        1.,
    );
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Ime(egui::ImeEvent::Preedit("ї".into()))],
        [1366., 900.],
        1.,
    );
    assert!(app.composing);
    assert_eq!(app.buffer, "ї");
    assert_eq!(app.session.editor.document, Document::default());
    assert!(!app.session.dirty());
    assert!(!app.enabled(Action::Save));
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Ime(egui::ImeEvent::Commit("ї".into()))],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.session.editor.document.text(), "ї");
    assert_eq!(app.session.editor.undo_stack.len(), 1);
    app.execute(Action::Undo, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    assert_eq!(app.session.editor.document.text(), "");
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Ime(egui::ImeEvent::Enabled)],
        [1366., 900.],
        1.,
    );
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Ime(egui::ImeEvent::Preedit("cancel".into()))],
        [1366., 900.],
        1.,
    );
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Ime(egui::ImeEvent::Commit("".into()))],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.buffer, "");
    assert!(!app.session.dirty());
}

#[test]
fn grapheme_navigation_and_deletion_preserve_unicode() {
    let (ctx, mut app) = setup();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste("e\u{301}🙂".into())],
        [1366., 900.],
        1.,
    );
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Backspace, false)],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.session.editor.document.text(), "e\u{301}");
    frame(
        &ctx,
        &mut app,
        vec![key(Key::ArrowLeft, false)],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.session.editor.selection, (0, 0));
    frame(
        &ctx,
        &mut app,
        vec![key(Key::ArrowRight, false)],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.session.editor.selection, (2, 2));
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Backspace, false)],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.session.editor.document.text(), "");
}

#[test]
fn galley_alignment_and_hit_testing_keep_scalar_indices() {
    let (ctx, _app) = setup();
    let mut editor = crate::commands::Editor::default();
    editor
        .replace(0, 0, "Hello\nВітаю e\u{301}", "Input")
        .unwrap();
    editor.align(0, 5, Alignment::Center).unwrap();
    editor.emphasize(6, 11, true, true).unwrap();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        let g = layout::galley(
            ctx,
            &editor.document,
            &editor.document.text(),
            CharacterFormat::default(),
            0,
            500.,
            16.,
        );
        assert_eq!(g.text(), editor.document.text());
        assert!(g.pos_from_cursor(CCursor::new(0)).left() > 100.);
        assert!(g.pos_from_cursor(CCursor::new(6)).left() < 2.);
        for i in [0, 2, 5, 6, 8, 11] {
            let p = g.pos_from_cursor(CCursor::new(i));
            assert_eq!(
                g.cursor_from_pos(p.center().to_vec2()).index,
                i,
                "scalar {i}"
            );
        }
    });
}

#[test]
fn ribbon_canvas_and_status_remain_separate_at_synthetic_dpi() {
    for dpi in [1., 1.25, 1.5] {
        for physical in [[960., 900.], [1366., 900.], [1920., 1080.]] {
            let ctx = egui::Context::default();
            let mut app = DocsApp::new(Session::default(), &ctx);
            for _ in 0..3 {
                frame(&ctx, &mut app, vec![], physical, dpi);
            }
            let region = |name: &str| app.regions.iter().find(|r| r.name == name).unwrap().rect;
            let title = region("title");
            let tabs = region("tabs");
            let ribbon = region("ribbon");
            let workspace = region("workspace");
            let status = region("status");
            assert!(title.bottom() <= tabs.top() + 1.);
            assert!(tabs.bottom() <= ribbon.top() + 1.);
            assert!(ribbon.bottom() <= workspace.top() + 1.);
            assert!(workspace.bottom() <= status.top() + 1.);
            for name in HOME_GROUPS {
                if let Some(group) = app.regions.iter().find(|r| r.name == *name) {
                    assert!(
                        group.rect.right() <= physical[0] / dpi + 1.,
                        "{name} overflows at {dpi}: {:?}",
                        group.rect
                    );
                }
            }
            app.ribbon_collapsed = true;
            frame(&ctx, &mut app, vec![], physical, dpi);
            assert!(!app.regions.iter().any(|r| r.name == "ribbon"));
        }
    }
}

#[test]
fn dirty_new_and_close_require_a_decision_and_failed_save_retains_session() {
    let (ctx, mut app) = setup();
    app.session
        .editor
        .replace(0, 0, "keep me", "Input")
        .unwrap();
    app.request(Pending::New, &ctx);
    assert!(app.pending.is_some());
    assert_eq!(app.session.editor.document.text(), "keep me");
    app.pending = None;
    assert!(app.session.dirty());
    let dir = tempfile::tempdir().unwrap();
    app.session.path = Some(dir.path().join("missing/file.bfdoc"));
    assert!(!app.choose_save(false));
    assert!(app.session.dirty());
    assert!(app.error.is_some());
    app.finish(Pending::Close, &ctx);
    assert!(app.allow_close);
}

#[test]
fn search_selects_real_model_ranges_and_failed_open_keeps_edits() {
    let (ctx, mut app) = setup();
    app.session
        .editor
        .replace(0, 0, "Hello Україна Hello", "Input")
        .unwrap();
    app.find = "Україна".into();
    app.sync_view();
    app.execute(Action::Find, &ctx);
    click_region(&ctx, &mut app, "find-next");
    assert_eq!(
        selected_text(&app.session.editor.document, app.session.editor.selection),
        "Україна"
    );
    let before = app.session.editor.document.clone();
    app.finish(Pending::Open(PathBuf::from("/missing.bfdoc")), &ctx);
    assert_eq!(app.session.editor.document, before);
    assert!(app.error.is_some());
}

#[test]
fn navigation_search_keeps_focus_wraps_past_forty_and_never_edits() {
    let (ctx, mut app) = setup();
    let text = "Україна e\u{301}\n".repeat(65);
    app.session.editor.replace(0, 0, &text, "Input").unwrap();
    app.sync_view();
    let before = app.session.editor.document.clone();
    let history = app.session.editor.undo_stack.len();
    app.execute(Action::Find, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Україна".into())],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.session.editor.selection, (0, 7));
    for _ in 0..50 {
        frame(
            &ctx,
            &mut app,
            vec![key(Key::Enter, false)],
            [1366., 900.],
            1.,
        );
        assert!(ctx.memory(|m| m.has_focus(Id::new("find-text"))));
    }
    assert_eq!(app.session.editor.selection, (550, 557));
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    assert!(app.regions.iter().any(|r| r.name == "find-result-50"));
    let previous = egui::Event::Key {
        key: Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::SHIFT,
    };
    frame(&ctx, &mut app, vec![previous], [1366., 900.], 1.);
    assert_eq!(app.session.editor.selection, (539, 546));
    frame(
        &ctx,
        &mut app,
        vec![key(Key::ArrowDown, false)],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.session.editor.selection, (550, 557));
    assert_eq!(app.session.editor.document, before);
    assert_eq!(app.session.editor.undo_stack.len(), history);
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Escape, false)],
        [1366., 900.],
        1.,
    );
    assert!(!app.navigation);
    assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
    assert_eq!(app.session.editor.document, before);
}

#[test]
fn navigation_replace_rejects_partial_graphemes_and_refreshes_after_history() {
    let (ctx, mut app) = setup();
    app.session
        .editor
        .replace(0, 0, "e\u{301} e\u{301}", "Input")
        .unwrap();
    app.session.editor.emphasize(0, 2, true, true).unwrap();
    app.sync_view();
    let before = app.session.editor.document.clone();
    let history = app.session.editor.undo_stack.len();
    app.execute(Action::Replace, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("e".into())],
        [1366., 900.],
        1.,
    );
    app.replacement = "ї".into();
    click_region(&ctx, &mut app, "find-replace");
    assert_eq!(app.session.editor.document, before);
    assert_eq!(app.session.editor.undo_stack.len(), history);
    click_region(&ctx, &mut app, "find-clear");
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("e\u{301}".into())],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.session.editor.selection, (0, 2));
    click_region(&ctx, &mut app, "find-replace");
    assert_eq!(app.session.editor.document.text(), "ї e\u{301}");
    assert!(
        app.session.editor.document.paragraphs[0].styles[0]
            .bold
            .unwrap_or(false)
    );
    assert_eq!(app.session.editor.undo_stack.len(), history + 1);
    let changed = app.session.editor.document.clone();
    app.execute(Action::Undo, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    assert_eq!(app.session.editor.document, before);
    assert!(app.regions.iter().any(|r| r.name == "find-result-1"));
    app.execute(Action::Redo, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    assert_eq!(app.session.editor.document, changed);
    assert!(!app.regions.iter().any(|r| r.name == "find-result-1"));
}

#[test]
fn search_ime_confirmation_is_not_a_next_result_command() {
    let (ctx, mut app) = setup();
    app.session.editor.replace(0, 0, "ї ї", "Input").unwrap();
    app.sync_view();
    app.execute(Action::Find, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    frame(
        &ctx,
        &mut app,
        vec![
            egui::Event::Ime(egui::ImeEvent::Enabled),
            egui::Event::Ime(egui::ImeEvent::Preedit("ї".into())),
        ],
        [1366., 900.],
        1.,
    );
    let selection = app.session.editor.selection;
    frame(
        &ctx,
        &mut app,
        vec![
            egui::Event::Ime(egui::ImeEvent::Commit("ї".into())),
            key(Key::Enter, false),
        ],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.session.editor.selection, selection);
    assert_eq!(app.session.editor.document.text(), "ї ї");
    assert_eq!(app.session.editor.undo_stack.len(), 1);
}

#[test]
fn navigation_controls_fit_narrow_and_scaled_windows() {
    for dpi in [1., 1.25, 1.5] {
        for size in [[640., 480.], [1024., 768.], [1366., 900.]] {
            let (ctx, mut app) = setup();
            app.session
                .editor
                .replace(0, 0, &"Русский Україна e\u{301}\n".repeat(80), "Input")
                .unwrap();
            app.sync_view();
            app.find = "Україна".into();
            app.execute(Action::Replace, &ctx);
            for _ in 0..4 {
                frame(&ctx, &mut app, vec![], size, dpi);
            }
            let pane = app
                .regions
                .iter()
                .find(|r| r.name == "navigation")
                .unwrap()
                .rect;
            let workspace = app
                .regions
                .iter()
                .find(|r| r.name == "workspace")
                .unwrap()
                .rect;
            assert!(pane.right() <= workspace.left());
            for name in [
                "find-input",
                "find-clear",
                "find-case",
                "find-words",
                "find-close",
                "find-next",
                "find-previous",
                "find-replace",
                "find-replace-all",
            ] {
                let rect = app.regions.iter().find(|r| r.name == name).unwrap().rect;
                assert!(
                    pane.expand(1.).contains_rect(rect),
                    "{name} {size:?} {dpi}: {pane:?} {rect:?}"
                );
            }
            assert!(
                app.regions
                    .iter()
                    .filter(|r| r.name.starts_with("find-result-"))
                    .count()
                    < 80
            );
        }
    }
}

fn click_region(ctx: &egui::Context, app: &mut DocsApp, name: &str) {
    click_region_at(ctx, app, name, [1366., 900.]);
}

fn click_region_at(ctx: &egui::Context, app: &mut DocsApp, name: &str, size: [f32; 2]) {
    for _ in 0..2 {
        frame(ctx, app, vec![], size, 1.);
    }
    let pos = app
        .regions
        .iter()
        .find(|r| r.name == name)
        .unwrap_or_else(|| panic!("Missing region {name}"))
        .rect
        .center();
    for pressed in [true, false] {
        frame(
            ctx,
            app,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
            size,
            1.,
        );
    }
}

#[test]
fn ribbon_pointer_and_all_unsaved_dialog_choices_are_real() {
    let (ctx, mut app) = setup();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Keep this".into())],
        [1366., 900.],
        1.,
    );
    app.execute(Action::SelectAll, &ctx);
    click_region(&ctx, &mut app, "command-Bold");
    assert!(
        app.session.editor.document.paragraphs[0]
            .styles
            .iter()
            .all(|s| s.bold.unwrap_or(false))
    );
    app.request(Pending::New, &ctx);
    click_region(&ctx, &mut app, "unsaved-cancel");
    assert!(app.pending.is_none());
    assert_eq!(app.session.editor.document.text(), "Keep this");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("saved.bfdoc");
    app.session.path = Some(path.clone());
    app.request(Pending::New, &ctx);
    click_region(&ctx, &mut app, "unsaved-save");
    assert_eq!(storage::load(&path).unwrap().text(), "Keep this");
    assert_eq!(app.session.editor.document.text(), "");
    for _ in 0..2 {
        frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    }
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Discard this".into())],
        [1366., 900.],
        1.,
    );
    app.request(Pending::New, &ctx);
    click_region(&ctx, &mut app, "unsaved-discard");
    assert_eq!(app.session.editor.document.text(), "");
    assert_eq!(storage::load(&path).unwrap().text(), "Keep this");
}

#[test]
fn shift_grapheme_selection_replaces_without_splitting_combining_text() {
    let (ctx, mut app) = setup();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste("a e\u{301}".into())],
        [1366., 900.],
        1.,
    );
    let mut event = key(Key::ArrowLeft, false);
    if let egui::Event::Key { modifiers, .. } = &mut event {
        modifiers.shift = true;
    }
    frame(&ctx, &mut app, vec![event], [1366., 900.], 1.);
    assert_eq!(
        selected_text(&app.session.editor.document, app.session.editor.selection),
        "e\u{301}"
    );
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("ї".into())],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.session.editor.document.text(), "a ї");
}

#[test]
fn windows_plain_text_paste_normalizes_line_endings() {
    let (ctx, mut app) = setup();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste("English\r\nУкраїнська\rРусский".into())],
        [1366., 900.],
        1.,
    );
    assert_eq!(
        app.session.editor.document.text(),
        "English\nУкраїнська\nРусский"
    );
    assert!(app.error.is_none());
    assert_eq!(app.session.editor.undo_stack.len(), 1);
}

#[test]
fn dense_controls_fit_each_group_and_window_at_breakpoints() {
    for dpi in [1., 1.25, 1.5] {
        for width in [640., 720., 899., 900., 916., 1195., 1196., 1366., 1920.] {
            let ctx = egui::Context::default();
            let mut app = DocsApp::new(Session::default(), &ctx);
            for _ in 0..3 {
                frame(&ctx, &mut app, vec![], [width * dpi, 900. * dpi], dpi);
            }
            let get = |name: &str| app.regions.iter().find(|r| r.name == name).unwrap().rect;
            assert!(get("document-title").right() <= get("command-search").left());
            assert!(get("command-search").right() <= get("window-0").left());
            let ribbon = get("ribbon");
            if let Some(more) = app.regions.iter().find(|r| r.name == "home-overflow") {
                assert!(
                    ribbon.contains_rect(more.rect),
                    "overflow at {width}/{dpi}: {:?}",
                    more.rect
                );
                assert!(get("Paragraph").right() <= more.rect.left());
            }
            assert!(
                (get("title").height() - TITLE_HEIGHT).abs() < 0.1,
                "title height {:?}",
                get("title")
            );
            let mut group: Option<&Region> = None;
            let mut controls = Vec::<&Region>::new();
            for region in &app.regions {
                if HOME_GROUPS.contains(&region.name.as_str()) {
                    group = Some(region);
                    controls.clear();
                } else if let Some(parent) = group {
                    if region.name == "ribbon" {
                        break;
                    }
                    if region.name.starts_with("command-")
                        || region.name.starts_with("unavailable-")
                        || region.name.starts_with("style-")
                        || region.name.starts_with("font-")
                        || region.name.starts_with("home-menu-")
                    {
                        assert!(
                            parent.rect.expand(0.1).contains_rect(region.rect),
                            "{} outside {} at {width}/{dpi}: {:?}",
                            region.name,
                            parent.name,
                            region.rect
                        );
                        assert!(ribbon.expand(0.1).contains_rect(region.rect));
                        for previous in &controls {
                            assert!(
                                !previous
                                    .rect
                                    .shrink(0.1)
                                    .intersects(region.rect.shrink(0.1)),
                                "{} overlaps {} at {width}/{dpi}",
                                previous.name,
                                region.name
                            );
                        }
                        controls.push(region);
                    }
                }
            }
        }
    }
}

#[test]
fn home_group_gutters_separate_controls_at_every_density_and_scale() {
    for dpi in [1., 1.25, 1.5] {
        for width in [640., 720., 899., 900., 916., 1195., 1196., 1366., 1920.] {
            for temporary in [false, true] {
                let (ctx, mut app) = setup();
                app.ribbon_collapsed = temporary;
                app.ribbon_temporary = temporary;
                for _ in 0..3 {
                    frame(&ctx, &mut app, vec![], [width * dpi, 900. * dpi], dpi);
                }
                let groups: Vec<_> = app
                    .regions
                    .iter()
                    .filter(|r| HOME_GROUPS.contains(&r.name.as_str()))
                    .collect();
                assert!(groups.len() >= 3);
                for pair in groups.windows(2) {
                    let gap = pair[1].rect.left() - pair[0].rect.right();
                    assert!(
                        (gap - 8.).abs() < 0.1,
                        "{} / {} gutter is {gap} at {width}/{dpi}, temporary={temporary}",
                        pair[0].name,
                        pair[1].name
                    );
                    let separator = app
                        .regions
                        .iter()
                        .find(|r| r.name == format!("home-separator-{}", pair[0].name))
                        .unwrap()
                        .rect;
                    assert!(separator.left() > pair[0].rect.right());
                    assert!(separator.right() < pair[1].rect.left());
                    assert!(
                        (separator.center().x - (pair[0].rect.right() + pair[1].rect.left()) / 2.)
                            .abs()
                            < 0.1
                    );
                }
                for separator in app
                    .regions
                    .iter()
                    .filter(|r| r.name.starts_with("home-separator-"))
                {
                    for target in app.regions.iter().filter(|r| {
                        r.name.starts_with("command-")
                            || r.name.starts_with("unavailable-")
                            || r.name.starts_with("font-")
                            || r.name.starts_with("style-")
                            || r.name.starts_with("home-menu-")
                            || r.name == "home-overflow"
                    }) {
                        assert!(
                            !separator.rect.intersects(target.rect),
                            "{} crosses {} at {width}/{dpi}, temporary={temporary}",
                            separator.name,
                            target.name
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn revised_shell_controls_preserve_selection_and_real_document_state() {
    let (ctx, mut app) = setup();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste("Hello\nУкраїна e\u{301}".into())],
        [1366., 900.],
        1.,
    );
    app.execute(Action::SelectAll, &ctx);
    click_region(&ctx, &mut app, "command-Italic");
    click_region(&ctx, &mut app, "command-Right");
    let styled = app.session.editor.document.clone();
    assert!(
        styled
            .paragraphs
            .iter()
            .all(|p| p.alignment == Alignment::Right
                && p.styles.iter().all(|s| s.italic.unwrap_or(false)))
    );
    click_region(&ctx, &mut app, "unavailable-Strikethrough");
    assert_eq!(app.session.editor.document, styled);
    click_region(&ctx, &mut app, "qat-Undo");
    assert!(
        app.session
            .editor
            .document
            .paragraphs
            .iter()
            .all(|p| p.alignment == Alignment::Left)
    );
    click_region(&ctx, &mut app, "qat-Redo");
    assert_eq!(app.session.editor.document, styled);
    click_region(&ctx, &mut app, "zoom-in");
    assert_eq!(app.zoom, 110.);
    click_region(&ctx, &mut app, "zoom-out");
    assert_eq!(app.zoom, 100.);
    app.zoom = 150.;
    click_region(&ctx, &mut app, "zoom-status");
    assert!(app.zoom_dialog.is_some());
    click_region(&ctx, &mut app, "zoom-preset-100");
    click_region(&ctx, &mut app, "zoom-ok");
    assert_eq!(app.zoom, 100.);
    app.backstage = true;
    click_region(&ctx, &mut app, "file-options");
    assert!(app.settings);
    click_region(&ctx, &mut app, "file-home");
    assert!(!app.settings);
    click_region(&ctx, &mut app, "file-blank");
    assert!(app.pending.is_some());
    click_region(&ctx, &mut app, "unsaved-cancel");
    assert_eq!(app.session.editor.document, styled);
    click_region(&ctx, &mut app, "file-back");
    assert!(!app.backstage);
    click_region(&ctx, &mut app, "tab-View");
    assert_eq!(app.tab, 8);
    click_region(&ctx, &mut app, "tab-Home");
    assert_eq!(app.tab, 0);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    ctx.memory_mut(|m| m.request_focus(Id::new("document-text")));
    frame(&ctx, &mut app, vec![key(Key::Z, true)], [1366., 900.], 1.);
    assert!(
        app.session
            .editor
            .document
            .paragraphs
            .iter()
            .all(|p| p.alignment == Alignment::Left)
    );
}

#[test]
fn icon_control_size_does_not_depend_on_platform_font_height() {
    for dpi in [1., 1.25, 1.5] {
        let ctx = egui::Context::default();
        let _app = DocsApp::new(Session::default(), &ctx);
        ctx.set_pixels_per_point(dpi);
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Exaggerate the metric variation that exposed the Windows failure.
                ui.style_mut()
                    .text_styles
                    .insert(egui::TextStyle::Button, FontId::proportional(36.));
                let rect = Rect::from_min_size(pos2(100., 80.), vec2(15., 16.));
                let response =
                    chrome::control(ui, rect, Icon::Launch, "", "Formatting pane", true, false);
                assert!(
                    rect.expand(0.1).contains_rect(response.rect),
                    "{dpi}: {:?}",
                    response.rect
                );
            });
        });
    }
}

fn edit_size(ctx: &egui::Context, app: &mut DocsApp, text: &str, finish: Key) {
    let mut shortcut = key(Key::P, true);
    if let egui::Event::Key { modifiers, .. } = &mut shortcut {
        modifiers.shift = true;
    }
    frame(ctx, app, vec![shortcut], [1366., 900.], 1.);
    assert!(ctx.memory(|m| m.has_focus(Id::new(FONT_SIZE_ID))));
    frame(
        ctx,
        app,
        vec![egui::Event::Text(text.into())],
        [1366., 900.],
        1.,
    );
    frame(ctx, app, vec![key(finish, false)], [1366., 900.], 1.);
    frame(ctx, app, vec![], [1366., 900.], 1.);
}

#[test]
fn home_size_and_underline_preserve_selection_history_and_saved_content() {
    let (ctx, mut app) = setup();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste(
            "English\nУкраїнська\nРусский e\u{301}".into(),
        )],
        [1366., 900.],
        1.,
    );
    app.execute(Action::SelectAll, &ctx);
    let selection = app.session.editor.selection;
    click_region(&ctx, &mut app, "command-Underline");
    assert!(app.checked(Action::Underline));
    let underlined = app.session.editor.document.clone();
    edit_size(&ctx, &mut app, "24.5", Key::Enter);
    assert!(app.error.is_none(), "{:?}", app.error);
    assert_eq!(app.session.editor.selection, selection);
    assert_eq!(app.font_size_text, "24.5");
    assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
    let formatted = app.session.editor.document.clone();
    assert!(
        formatted
            .paragraphs
            .iter()
            .flat_map(|p| &p.styles)
            .all(|s| s.underline.unwrap_or(false)
                && s.font_size.unwrap_or_default().to_string() == "24.5")
    );
    frame(&ctx, &mut app, vec![key(Key::Z, true)], [1366., 900.], 1.);
    assert_eq!(app.session.editor.document, underlined);
    frame(&ctx, &mut app, vec![key(Key::Y, true)], [1366., 900.], 1.);
    assert_eq!(app.session.editor.document, formatted);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("font-controls.bfdoc");
    app.session.path = Some(path.clone());
    app.execute(Action::Save, &ctx);
    app.finish(Pending::New, &ctx);
    app.finish(Pending::Open(path), &ctx);
    assert_eq!(app.session.editor.document, formatted);
    app.execute(Action::SelectAll, &ctx);
    click_region(&ctx, &mut app, "font-size-menu");
    click_region(&ctx, &mut app, "size-preset-18");
    assert_eq!(
        app.session.editor.selected_font_size().unwrap().to_string(),
        "18"
    );
    assert!(app.checked(Action::Underline));
    assert_eq!(app.session.editor.selection, selection);
}

#[test]
fn size_field_cancel_invalid_mixed_and_blur_do_not_lose_document_text() {
    let (ctx, mut app) = setup();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Hello ї e\u{301}".into())],
        [1366., 900.],
        1.,
    );
    app.execute(Action::SelectAll, &ctx);
    let original = app.session.editor.document.clone();
    let undo = app.session.editor.undo_stack.len();
    edit_size(&ctx, &mut app, "72", Key::Escape);
    assert_eq!(app.session.editor.document, original);
    assert_eq!(app.session.editor.undo_stack.len(), undo);
    assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
    edit_size(&ctx, &mut app, "0.5", Key::Enter);
    assert!(app.error.as_ref().unwrap().contains("1–400"));
    assert_eq!(app.session.editor.document, original);
    assert_eq!(app.session.editor.undo_stack.len(), undo);
    app.error = None;
    app.session
        .editor
        .set_font_size(0, 5, "18".parse().unwrap())
        .unwrap();
    app.execute(Action::SelectAll, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    assert!(app.font_size_text.is_empty());
    edit_size(&ctx, &mut app, "14", Key::Tab);
    assert!(app.error.is_none());
    assert_eq!(
        app.session.editor.selected_font_size().unwrap().to_string(),
        "14"
    );
    assert_eq!(app.session.editor.document.text(), original.text());
}

#[test]
fn caret_font_commands_and_ime_use_the_same_pending_style() {
    let (ctx, mut app) = setup();
    frame(&ctx, &mut app, vec![key(Key::U, true)], [1366., 900.], 1.);
    edit_size(&ctx, &mut app, "18", Key::Enter);
    assert!(!app.session.dirty());
    assert!(app.session.editor.undo_stack.is_empty());
    let pending = app.session.editor.typing_style;
    assert_eq!(pending.underline, Some(true));
    assert_eq!(pending.font_size.unwrap_or_default().to_string(), "18");
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text(
            "English\nУкраїнська\nРусский e\u{301}".into(),
        )],
        [1366., 900.],
        1.,
    );
    frame(
        &ctx,
        &mut app,
        vec![
            egui::Event::Ime(egui::ImeEvent::Enabled),
            egui::Event::Ime(egui::ImeEvent::Preedit("ї".into())),
        ],
        [1366., 900.],
        1.,
    );
    assert!(!app.enabled(Action::Underline));
    assert!(!app.enabled(Action::FontSize));
    let before = app.session.editor.document.clone();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Ime(egui::ImeEvent::Commit("ї".into()))],
        [1366., 900.],
        1.,
    );
    assert!(
        app.session
            .editor
            .document
            .paragraphs
            .iter()
            .flat_map(|p| &p.styles)
            .all(|s| *s == pending)
    );
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, before);
    assert_eq!(app.session.editor.typing_style, pending);
}

#[test]
fn mixed_font_layout_changes_real_geometry_and_preserves_unicode_hits() {
    let (ctx, _) = setup();
    let mut editor = crate::commands::Editor::default();
    editor
        .replace(0, 0, "small\nЇ e\u{301} ё", "Input")
        .unwrap();
    editor.set_font_size(6, 12, "28".parse().unwrap()).unwrap();
    editor.underline(6, 12, true).unwrap();
    for zoom in [0.5, 1., 2.] {
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            let g = layout::galley(
                ctx,
                &editor.document,
                &editor.document.text(),
                CharacterFormat::default(),
                0,
                600.,
                14.6667 * zoom,
            );
            assert!(g.rows[1].row.size.y > g.rows[0].row.size.y * 2.);
            assert!(g.job.sections.iter().any(|s| s.format.underline.width > 0.
                && (s.format.font_id.size - 28. * 4. / 3. * zoom).abs() < 0.01));
            for i in [0, 3, 5, 6, 7, 8, 10, 11, 12] {
                let p = g.pos_from_cursor(CCursor::new(i));
                assert_eq!(g.cursor_from_pos(p.center().to_vec2()).index, i);
            }
        });
    }
}

#[test]
fn save_and_close_resolve_pending_size_before_operating_on_document() {
    let (ctx, mut app) = setup();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Keep formatting".into())],
        [1366., 900.],
        1.,
    );
    app.execute(Action::SelectAll, &ctx);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("pending-size.bfdoc");
    app.session.path = Some(path.clone());
    app.execute(Action::Save, &ctx);
    app.execute(Action::FontSize, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("32".into())],
        [1366., 900.],
        1.,
    );
    assert!(app.font_size_dirty);
    frame(&ctx, &mut app, vec![key(Key::S, true)], [1366., 900.], 1.);
    let saved = storage::load(&path).unwrap();
    assert!(
        saved.paragraphs[0]
            .styles
            .iter()
            .all(|s| s.font_size.unwrap_or_default().to_string() == "32")
    );
    assert!(!app.session.dirty());
    app.execute(Action::FontSize, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("401".into())],
        [1366., 900.],
        1.,
    );
    frame(&ctx, &mut app, vec![key(Key::S, true)], [1366., 900.], 1.);
    assert!(app.error.is_some());
    assert_eq!(storage::load(&path).unwrap(), saved);
    app.error = None;
    app.execute(Action::FontSize, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("24".into())],
        [1366., 900.],
        1.,
    );
    let mut close = egui::RawInput::default();
    close
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .events
        .push(egui::ViewportEvent::Close);
    let output = ctx.run(close, |ctx| app.update_ui(ctx));
    assert!(app.pending.is_some());
    assert!(
        output.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .contains(&ViewportCommand::CancelClose)
    );
    assert_eq!(
        app.session.editor.selected_font_size().unwrap().to_string(),
        "24"
    );
    assert_eq!(storage::load(&path).unwrap(), saved);
}

#[test]
fn font_step_controls_keep_mixed_selection_at_wide_and_minimum_width() {
    for width in [640., 1366.] {
        let (ctx, mut app) = setup();
        app.session
            .editor
            .replace(0, 0, "A ї e\u{301}", "Type")
            .unwrap();
        app.session
            .editor
            .set_font_size(0, 1, "28".parse().unwrap())
            .unwrap();
        app.session.editor.underline(0, 6, true).unwrap();
        app.execute(Action::SelectAll, &ctx);
        let before = app.session.editor.document.clone();
        let count = app.session.editor.undo_stack.len();
        click_region_at(&ctx, &mut app, "command-GrowFont", [width, 900.]);
        let after = app.session.editor.document.clone();
        assert_eq!(app.session.editor.selection, (0, 6));
        assert_eq!(app.session.editor.undo_stack.len(), count + 1);
        assert_eq!(
            after.paragraphs[0].styles[0]
                .font_size
                .unwrap_or_default()
                .to_string(),
            "36"
        );
        assert_eq!(
            after.paragraphs[0].styles[5]
                .font_size
                .unwrap_or_default()
                .to_string(),
            "12"
        );
        assert!(
            after.paragraphs[0]
                .styles
                .iter()
                .all(|s| s.underline.unwrap_or(false))
        );
        app.execute(Action::Undo, &ctx);
        assert_eq!(app.session.editor.document, before);
        app.execute(Action::Redo, &ctx);
        assert_eq!(app.session.editor.document, after);
        click_region_at(&ctx, &mut app, "command-ShrinkFont", [width, 900.]);
        for (a, b) in app.session.editor.document.paragraphs[0]
            .styles
            .iter()
            .zip(&before.paragraphs[0].styles)
        {
            assert_eq!(
                a.resolve(crate::model::Style::default()),
                b.resolve(crate::model::Style::default())
            );
        }
        assert_eq!(
            app.session.editor.document.paragraphs[0].styles[5].font_size,
            Some(FontSize::default())
        );
        frame(&ctx, &mut app, vec![], [width, 900.], 1.);
        let mut grow = key(Key::Period, true);
        if let egui::Event::Key { modifiers, .. } = &mut grow {
            modifiers.shift = true;
        }
        frame(&ctx, &mut app, vec![grow], [width, 900.], 1.);
        assert_eq!(app.session.editor.document, after);
        app.composing = true;
        click_region_at(&ctx, &mut app, "command-ShrinkFont", [width, 900.]);
        assert_eq!(app.session.editor.document, after);
    }
}

#[test]
fn home_menus_cancel_without_edits_and_dispatch_real_commands() {
    let (ctx, mut app) = setup();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Hello ї".into())],
        [1366., 900.],
        1.,
    );
    let before = app.session.editor.document.clone();
    let history = app.session.editor.undo_stack.len();
    click_region(&ctx, &mut app, "home-menu-Paste");
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    assert!(
        app.regions
            .iter()
            .any(|r| r.name == "command-Paste" && r.rect.width() >= 180.)
    );
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Escape, false)],
        [1366., 900.],
        1.,
    );
    click_region(&ctx, &mut app, "home-menu-Select");
    click_region(&ctx, &mut app, "command-SelectAll");
    assert_eq!(app.session.editor.selection, (0, 7));
    click_region(&ctx, &mut app, "command-Find");
    assert!(app.navigation);
    assert!(ctx.memory(|m| m.has_focus(Id::new("find-text"))));
    assert_eq!(app.session.editor.document, before);
    assert_eq!(app.session.editor.undo_stack.len(), history);
    app.navigation = false;
    click_region(&ctx, &mut app, "home-menu-Find");
    click_region(&ctx, &mut app, "command-Navigation");
    assert!(app.navigation);
    click_region_at(&ctx, &mut app, "home-overflow", [640., 900.]);
    click_region_at(&ctx, &mut app, "command-Replace", [640., 900.]);
    assert!(app.replace_mode);
    assert_eq!(app.session.editor.document, before);
}

#[test]
fn font_dialog_drafts_cancel_validate_and_apply_one_shared_transaction() {
    for width in [640., 1366.] {
        let (ctx, mut app) = setup();
        app.session
            .editor
            .replace(0, 0, "English\nЇ e\u{301} ё", "Paste")
            .unwrap();
        app.session.editor.emphasize(0, 7, true, true).unwrap();
        app.session
            .editor
            .set_font_size(0, 7, "28".parse().unwrap())
            .unwrap();
        app.execute(Action::SelectAll, &ctx);
        let before = app.session.editor.document.clone();
        let selection = app.session.editor.selection;
        let history = app.session.editor.undo_stack.len();
        click_region_at(&ctx, &mut app, "command-FontDialog", [width, 480.]);
        assert!(app.font_dialog.is_some());
        let bounds = app
            .regions
            .iter()
            .find(|r| r.name == "font-dialog")
            .unwrap()
            .rect;
        assert!(
            Rect::from_min_size(Pos2::ZERO, vec2(width, 480.)).contains_rect(bounds),
            "{bounds:?}"
        );
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::Text("19.5".into())],
            [width, 480.],
            1.,
        );
        click_region_at(
            &ctx,
            &mut app,
            "font-dialog-style-Bold Italic",
            [width, 480.],
        );
        click_region_at(&ctx, &mut app, "font-dialog-underline", [width, 480.]);
        frame(
            &ctx,
            &mut app,
            vec![key(Key::N, true), key(Key::Z, true), key(Key::B, true)],
            [width, 480.],
            1.,
        );
        assert_eq!(app.session.editor.document, before);
        click_region_at(&ctx, &mut app, "font-dialog-cancel", [width, 480.]);
        assert!(app.font_dialog.is_none());
        assert_eq!(app.session.editor.document, before);
        assert_eq!(app.session.editor.undo_stack.len(), history);
        frame(&ctx, &mut app, vec![], [width, 480.], 1.);
        frame(&ctx, &mut app, vec![key(Key::D, true)], [width, 480.], 1.);
        assert!(app.font_dialog.is_some());
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::Text("401".into())],
            [width, 480.],
            1.,
        );
        click_region_at(&ctx, &mut app, "font-dialog-ok", [width, 480.]);
        assert!(app.font_dialog.is_some());
        assert_eq!(app.session.editor.document, before);
        // The invalid field is selected again; a replacement plus Enter applies.
        frame(&ctx, &mut app, vec![], [width, 480.], 1.);
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::Text("19.5".into())],
            [width, 480.],
            1.,
        );
        click_region_at(&ctx, &mut app, "font-dialog-underline", [width, 480.]);
        frame(
            &ctx,
            &mut app,
            vec![key(Key::Enter, false)],
            [width, 480.],
            1.,
        );
        assert!(app.font_dialog.is_none());
        assert_eq!(app.session.editor.selection, selection);
        assert_eq!(app.session.editor.undo_stack.len(), history + 1);
        let after = app.session.editor.document.clone();
        for (old, new) in before.paragraphs.iter().zip(&after.paragraphs) {
            for (a, b) in old.styles.iter().zip(&new.styles) {
                assert_eq!(a.bold, b.bold);
                assert_eq!(a.italic, b.italic);
                assert_eq!(b.underline, Some(true));
                assert_eq!(b.font_size.unwrap_or_default().to_string(), "19.5");
            }
        }
        app.execute(Action::Undo, &ctx);
        assert_eq!(app.session.editor.document, before);
        app.execute(Action::Redo, &ctx);
        assert_eq!(app.session.editor.document, after);
        app.execute(Action::FontDialog, &ctx);
        frame(&ctx, &mut app, vec![], [width, 480.], 1.);
        frame(
            &ctx,
            &mut app,
            vec![key(Key::Escape, false)],
            [width, 480.],
            1.,
        );
        assert!(app.font_dialog.is_none());
        assert_eq!(app.session.editor.document, after);
    }
}

#[test]
fn font_dialog_keyboard_default_action_modal_close_and_dpi_bounds() {
    for dpi in [1., 1.25, 1.5] {
        let (ctx, mut app) = setup();
        let screen = [640. * dpi, 480. * dpi];
        app.execute(Action::FontDialog, &ctx);
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], screen, dpi);
        }
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::Text("400".into())],
            screen,
            dpi,
        );
        for _ in 0..2 {
            frame(&ctx, &mut app, vec![], screen, dpi);
        }
        let bounds = app
            .regions
            .iter()
            .find(|r| r.name == "font-dialog")
            .unwrap()
            .rect;
        assert!(
            Rect::from_min_size(Pos2::ZERO, vec2(640., 480.)).contains_rect(bounds),
            "{bounds:?}"
        );
        for region in app
            .regions
            .iter()
            .filter(|r| r.name.starts_with("font-dialog-"))
        {
            assert!(
                bounds.contains_rect(region.rect),
                "{} {:?}",
                region.name,
                region.rect
            );
        }
        let mut input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(640., 480.))),
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .events
            .push(egui::ViewportEvent::Close);
        let result = ctx.run(input, |ctx| app.update_ui(ctx));
        assert!(
            result.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .contains(&ViewportCommand::CancelClose)
        );
        assert!(app.font_dialog.is_some());
        assert_eq!(
            app.session
                .editor
                .typing_style
                .font_size
                .unwrap_or_default(),
            FontSize::default()
        );
        frame(&ctx, &mut app, vec![key(Key::Enter, false)], screen, dpi);
        assert!(app.font_dialog.is_none());
        assert_eq!(
            app.session
                .editor
                .typing_style
                .font_size
                .unwrap_or_default()
                .to_string(),
            "400"
        );
        assert!(!app.session.dirty());
        assert!(app.session.editor.undo_stack.is_empty());
        app.composing = true;
        app.execute(Action::FontDialog, &ctx);
        assert!(app.font_dialog.is_none());
        app.composing = false;
        app.pending = Some(Pending::New);
        app.execute(Action::FontDialog, &ctx);
        assert!(app.font_dialog.is_none());
        app.pending = None;
        app.error = Some("Cannot open document".into());
        app.execute(Action::FontDialog, &ctx);
        assert!(app.font_dialog.is_none());
    }
}

fn ribbon_key(
    ctx: &egui::Context,
    app: &mut DocsApp,
    key: Key,
    modifiers: Modifiers,
    size: [f32; 2],
) {
    for pressed in [true, false] {
        frame(
            ctx,
            app,
            vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            }],
            size,
            1.,
        );
    }
}

#[test]
fn temporary_ribbon_overlays_without_reflow_and_dismisses_after_real_command() {
    for width in [640., 1366.] {
        let (ctx, mut app) = setup();
        let size = [width, 900.];
        app.session
            .editor
            .replace(0, 0, "English\nУкраїнська\nРусский e\u{301}", "Paste")
            .unwrap();
        app.execute(Action::SelectAll, &ctx);
        app.execute(Action::Ribbon, &ctx);
        frame(&ctx, &mut app, vec![], size, 1.);
        let workspace = app
            .regions
            .iter()
            .find(|r| r.name == "workspace")
            .unwrap()
            .rect;
        let selection = app.session.editor.selection;
        click_region_at(&ctx, &mut app, "tab-Home", size);
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], size, 1.);
        }
        assert!(app.ribbon_collapsed && app.ribbon_temporary);
        assert_eq!(
            workspace,
            app.regions
                .iter()
                .find(|r| r.name == "workspace")
                .unwrap()
                .rect
        );
        let ribbon = app
            .regions
            .iter()
            .find(|r| r.name == "ribbon")
            .unwrap()
            .rect;
        assert!((ribbon.height() - RIBBON_HEIGHT).abs() <= 1., "{ribbon:?}");
        assert!(ribbon.right() <= width + 1.);
        click_region_at(&ctx, &mut app, "command-Bold", size);
        assert!(!app.ribbon_temporary && app.ribbon_collapsed);
        assert_eq!(app.session.editor.selection, selection);
        assert!(app.checked(Action::Bold));
        app.execute(Action::Undo, &ctx);
        assert!(!app.checked(Action::Bold));
        // A genuine click below the overlay closes it and keeps the document usable.
        click_region_at(&ctx, &mut app, "tab-Home", size);
        frame(&ctx, &mut app, vec![], size, 1.);
        click_region_at(&ctx, &mut app, "workspace", size);
        assert!(!app.ribbon_temporary && app.ribbon_collapsed);
    }
}

#[test]
fn home_keyboard_preserves_selection_history_and_roundtrip() {
    let (ctx, mut app) = setup();
    let size = [1366., 900.];
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste(
            "English\nУкраїнська\nРусский e\u{301}".into(),
        )],
        size,
        1.,
    );
    app.execute(Action::SelectAll, &ctx);
    app.execute(Action::Ribbon, &ctx);
    frame(&ctx, &mut app, vec![], size, 1.);
    let selection = app.session.editor.selection;
    ribbon_key(&ctx, &mut app, Key::F10, Modifiers::NONE, size);
    assert_eq!(app.ribbon_focus, Some(RibbonFocus::Tab));
    assert!(!app.ribbon_temporary);
    ribbon_key(&ctx, &mut app, Key::Tab, Modifiers::NONE, size);
    assert!(app.ribbon_temporary);
    assert_eq!(
        app.ribbon_focus,
        Some(RibbonFocus::Control("command-Paste".into()))
    );
    ribbon_key(&ctx, &mut app, Key::ArrowRight, Modifiers::CTRL, size);
    assert_eq!(
        app.ribbon_focus,
        Some(RibbonFocus::Control("font-size".into()))
    );
    for _ in 0..4 {
        ribbon_key(&ctx, &mut app, Key::Tab, Modifiers::NONE, size);
    }
    assert_eq!(
        app.ribbon_focus,
        Some(RibbonFocus::Control("command-ClearFormatting".into()))
    );
    ribbon_key(&ctx, &mut app, Key::Tab, Modifiers::NONE, size);
    assert_eq!(
        app.ribbon_focus,
        Some(RibbonFocus::Control("command-Bold".into()))
    );
    ribbon_key(&ctx, &mut app, Key::Space, Modifiers::NONE, size);
    assert!(app.checked(Action::Bold));
    assert!(!app.ribbon_temporary && app.ribbon_collapsed);
    assert_eq!(app.session.editor.selection, selection);
    assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
    let bold = app.session.editor.document.clone();
    app.execute(Action::Undo, &ctx);
    assert!(!app.checked(Action::Bold));
    app.execute(Action::Redo, &ctx);
    assert_eq!(app.session.editor.document, bold);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("Keyboard.bfdoc");
    app.session.save(&path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), bold);
}

#[test]
fn ribbon_menus_escape_overflow_and_keyboard_resize_retain_focus() {
    let (ctx, mut app) = setup();
    let wide = [1366., 900.];
    app.execute(Action::Ribbon, &ctx);
    ribbon_key(&ctx, &mut app, Key::H, Modifiers::ALT, wide);
    ribbon_key(&ctx, &mut app, Key::ArrowRight, Modifiers::CTRL, wide);
    ribbon_key(&ctx, &mut app, Key::ArrowDown, Modifiers::NONE, wide);
    assert!(egui::Popup::is_any_open(&ctx));
    assert!(app.ribbon_temporary);
    assert_eq!(app.ribbon_menu_index, Some(0));
    ribbon_key(&ctx, &mut app, Key::ArrowDown, Modifiers::NONE, wide);
    assert_eq!(app.ribbon_menu_index, Some(1));
    ribbon_key(&ctx, &mut app, Key::Escape, Modifiers::NONE, wide);
    assert!(!egui::Popup::is_any_open(&ctx));
    assert!(app.ribbon_temporary);
    assert_eq!(
        app.ribbon_focus,
        Some(RibbonFocus::Control("font-size-menu".into()))
    );
    // Move Font -> Paragraph -> Styles -> Editing; a narrow window uses More.
    for _ in 0..3 {
        ribbon_key(&ctx, &mut app, Key::ArrowRight, Modifiers::CTRL, wide);
    }
    assert_eq!(
        app.ribbon_focus,
        Some(RibbonFocus::Control("command-Find".into()))
    );
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    }
    assert_eq!(
        app.ribbon_focus,
        Some(RibbonFocus::Control("home-overflow".into()))
    );
    ribbon_key(
        &ctx,
        &mut app,
        Key::ArrowDown,
        Modifiers::NONE,
        [640., 480.],
    );
    assert!(egui::Popup::is_any_open(&ctx));
    let find_index = app
        .ribbon_menu_targets
        .iter()
        .position(|t| matches!(t.kind, TargetKind::Action(Action::Find)))
        .unwrap();
    for _ in 0..find_index {
        ribbon_key(
            &ctx,
            &mut app,
            Key::ArrowDown,
            Modifiers::NONE,
            [640., 480.],
        );
    }
    ribbon_key(&ctx, &mut app, Key::Enter, Modifiers::NONE, [640., 480.]);
    assert!(
        app.navigation,
        "menu index {:?}; style {:?}",
        app.ribbon_menu_index,
        app.session.editor.selected_paragraph_style()
    );
    assert!(!app.ribbon_temporary);
    assert!(ctx.memory(|m| m.has_focus(Id::new("find-text"))));
}

#[test]
fn ribbon_size_cancel_commit_and_f6_visible_zones() {
    let (ctx, mut app) = setup();
    let size = [1366., 900.];
    app.execute(Action::Ribbon, &ctx);
    edit_size(&ctx, &mut app, "19.5", Key::Escape);
    assert!(app.ribbon_collapsed && !app.ribbon_temporary);
    assert_eq!(
        app.session
            .editor
            .typing_style
            .font_size
            .unwrap_or_default()
            .to_string(),
        "11"
    );
    edit_size(&ctx, &mut app, "19.5", Key::Enter);
    assert!(app.ribbon_collapsed && !app.ribbon_temporary);
    assert_eq!(
        app.session
            .editor
            .typing_style
            .font_size
            .unwrap_or_default()
            .to_string(),
        "19.5"
    );
    ribbon_key(&ctx, &mut app, Key::F6, Modifiers::NONE, size);
    assert!(!app.navigation);
    assert!(ctx.memory(|m| m.focused() == app.status_focus));
    ribbon_key(&ctx, &mut app, Key::F6, Modifiers::NONE, size);
    assert_eq!(app.ribbon_focus, Some(RibbonFocus::Tab));
    ribbon_key(&ctx, &mut app, Key::F6, Modifiers::SHIFT, size);
    assert!(ctx.memory(|m| m.focused() == app.status_focus));
    ribbon_key(&ctx, &mut app, Key::F6, Modifiers::SHIFT, size);
    assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
    click_region(&ctx, &mut app, "zoom-in");
    ribbon_key(&ctx, &mut app, Key::F6, Modifiers::NONE, size);
    assert_eq!(app.ribbon_focus, Some(RibbonFocus::Tab));
    ribbon_key(&ctx, &mut app, Key::F6, Modifiers::NONE, size);
    assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
    app.navigation = true;
    app.formatting = true;
    frame(&ctx, &mut app, vec![], size, 1.);
    ribbon_key(&ctx, &mut app, Key::F6, Modifiers::NONE, size);
    assert!(ctx.memory(|m| m.has_focus(Id::new("find-text"))));
    ribbon_key(&ctx, &mut app, Key::F6, Modifiers::NONE, size);
    assert!(ctx.memory(|m| m.focused() == app.formatting_focus));
    ribbon_key(&ctx, &mut app, Key::F6, Modifiers::NONE, size);
    assert!(ctx.memory(|m| m.focused() == app.status_focus));
}

#[test]
fn temporary_home_geometry_at_synthetic_dpi_and_pin_shortcut() {
    for dpi in [1., 1.25, 1.5] {
        for width in [640., 900., 1366.] {
            let (ctx, mut app) = setup();
            app.ribbon_collapsed = true;
            app.ribbon_temporary = true;
            for _ in 0..4 {
                frame(&ctx, &mut app, vec![], [width * dpi, 480. * dpi], dpi);
            }
            let ribbon = app
                .regions
                .iter()
                .find(|r| r.name == "ribbon")
                .unwrap()
                .rect;
            assert!(
                ribbon.left() >= 0. && ribbon.right() <= width + 1.,
                "{width}/{dpi}: {ribbon:?}"
            );
            for target in &app.ribbon_targets {
                assert!(ribbon.expand(1.).contains_rect(target.rect));
            }
            frame(
                &ctx,
                &mut app,
                vec![key(Key::F1, true)],
                [width * dpi, 480. * dpi],
                dpi,
            );
            assert!(!app.ribbon_collapsed && !app.ribbon_temporary);
            let workspace = app
                .regions
                .iter()
                .find(|r| r.name == "workspace")
                .unwrap()
                .rect;
            assert!(workspace.top() >= TITLE_HEIGHT + TABS_HEIGHT + RIBBON_HEIGHT - 1.);
        }
    }
}

#[test]
fn keyboard_preset_activation_arrows_and_escape_do_not_insert_into_document() {
    let (ctx, mut app) = setup();
    let size = [1366., 900.];
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste("Українська e\u{301}".into())],
        size,
        1.,
    );
    app.execute(Action::SelectAll, &ctx);
    app.execute(Action::Ribbon, &ctx);
    let selection = app.session.editor.selection;
    ribbon_key(&ctx, &mut app, Key::H, Modifiers::ALT, size);
    ribbon_key(&ctx, &mut app, Key::ArrowRight, Modifiers::NONE, size);
    assert_eq!(
        app.ribbon_focus,
        Some(RibbonFocus::Control("command-Cut".into()))
    );
    ribbon_key(&ctx, &mut app, Key::ArrowDown, Modifiers::NONE, size);
    assert_eq!(
        app.ribbon_focus,
        Some(RibbonFocus::Control("command-Copy".into()))
    );
    ribbon_key(&ctx, &mut app, Key::Tab, Modifiers::SHIFT, size);
    assert_eq!(
        app.ribbon_focus,
        Some(RibbonFocus::Control("command-Cut".into()))
    );
    ribbon_key(&ctx, &mut app, Key::ArrowRight, Modifiers::CTRL, size);
    ribbon_key(&ctx, &mut app, Key::ArrowDown, Modifiers::ALT, size);
    ribbon_key(&ctx, &mut app, Key::ArrowDown, Modifiers::NONE, size);
    ribbon_key(&ctx, &mut app, Key::Enter, Modifiers::NONE, size);
    assert!(!app.ribbon_temporary && app.ribbon_collapsed);
    assert_eq!(app.session.editor.selection, selection);
    assert_eq!(app.session.editor.document.text(), "Українська e\u{301}");
    assert_eq!(
        app.session.editor.selected_font_size().unwrap().to_string(),
        "9"
    );
    ribbon_key(&ctx, &mut app, Key::H, Modifiers::ALT, size);
    ribbon_key(&ctx, &mut app, Key::Escape, Modifiers::NONE, size);
    assert!(!app.ribbon_temporary);
    assert_eq!(app.ribbon_focus, Some(RibbonFocus::Tab));
    ribbon_key(&ctx, &mut app, Key::Escape, Modifiers::NONE, size);
    assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
}

fn modifier_frame(ctx: &egui::Context, app: &mut DocsApp, modifiers: Modifiers, focused: bool) {
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366., 900.))),
        modifiers,
        focused,
        ..Default::default()
    };
    let _ = ctx.run(input, |ctx| app.update_ui(ctx));
}
fn alt_tap(ctx: &egui::Context, app: &mut DocsApp) {
    modifier_frame(ctx, app, Modifiers::ALT, true);
    modifier_frame(ctx, app, Modifiers::NONE, true);
}
fn tip_events(keys: &[Key]) -> Vec<egui::Event> {
    keys.iter()
        .flat_map(|k| {
            let mut events = vec![key(*k, false)];
            if k.name().len() == 1 {
                events.push(egui::Event::Text(k.name().to_lowercase()));
            }
            events.push(egui::Event::Key {
                key: *k,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: Modifiers::NONE,
            });
            events
        })
        .collect()
}
fn tip_sequence(ctx: &egui::Context, app: &mut DocsApp, keys: &[Key], size: [f32; 2]) {
    frame(ctx, app, tip_events(keys), size, 1.);
    // Native areas and their popups each need a measurement pass.
    for _ in 0..3 {
        frame(ctx, app, vec![], size, 1.);
    }
}

#[test]
fn keytips_bare_alt_toggle_chords_focus_loss_and_ime_guards() {
    use keytips::Level;
    let (ctx, mut app) = setup();
    alt_tap(&ctx, &mut app);
    assert_eq!(app.keytips.level, Some(Level::Root));
    assert!(app.regions.iter().any(|r| r.name == "keytip-H"));
    alt_tap(&ctx, &mut app);
    assert_eq!(app.keytips.level, None);
    assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
    for mods in [
        Modifiers::ALT | Modifiers::SHIFT,
        Modifiers::ALT | Modifiers::CTRL,
    ] {
        modifier_frame(&ctx, &mut app, Modifiers::ALT, true);
        modifier_frame(&ctx, &mut app, mods, true);
        modifier_frame(&ctx, &mut app, Modifiers::NONE, true);
        assert_eq!(app.keytips.level, None);
    }
    modifier_frame(&ctx, &mut app, Modifiers::ALT, true);
    modifier_frame(&ctx, &mut app, Modifiers::ALT, false);
    modifier_frame(&ctx, &mut app, Modifiers::NONE, true);
    assert_eq!(app.keytips.level, None);
    ribbon_key(
        &ctx,
        &mut app,
        Key::H,
        Modifiers::ALT | Modifiers::CTRL,
        [1366., 900.],
    );
    assert_eq!(app.keytips.level, None);
    assert!(!app.navigation && !app.replace_mode);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("ł".into())],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.session.editor.document.text(), "ł");
    frame(
        &ctx,
        &mut app,
        vec![
            egui::Event::Ime(egui::ImeEvent::Enabled),
            key(Key::F10, false),
        ],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.keytips.level, None);
    assert!(app.composing);
    alt_tap(&ctx, &mut app);
    assert_eq!(app.keytips.level, None);
}

#[test]
fn keytips_sequences_preserve_unicode_selection_history_and_roundtrip() {
    let (ctx, mut app) = setup();
    let text = "English e\u{301}\nУкраїнська\nРусский 🙂";
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste(text.into())],
        [1366., 900.],
        1.,
    );
    app.execute(Action::SelectAll, &ctx);
    app.ribbon_collapsed = true;
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    let original = app.session.editor.document.clone();
    let selection = app.session.editor.selection;
    let count = app.session.editor.undo_stack.len();
    // Native integrations may deliver several access keys and paired text in one frame.
    tip_sequence(
        &ctx,
        &mut app,
        &[Key::F10, Key::H, Key::Num1],
        [1366., 900.],
    );
    let bold = app.session.editor.document.clone();
    assert_eq!(bold.text(), text);
    assert!(
        bold.paragraphs
            .iter()
            .flat_map(|p| &p.styles)
            .all(|s| s.bold.unwrap_or(false))
    );
    assert_eq!(app.session.editor.selection, selection);
    assert_eq!(app.session.editor.undo_stack.len(), count + 1);
    assert!(app.ribbon_collapsed && !app.ribbon_temporary);
    assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, original);
    app.execute(Action::Redo, &ctx);
    assert_eq!(app.session.editor.document, bold);
    tip_sequence(
        &ctx,
        &mut app,
        &[Key::F10, Key::H, Key::A, Key::C],
        [1366., 900.],
    );
    assert!(
        app.session
            .editor
            .document
            .paragraphs
            .iter()
            .all(|p| p.alignment == Alignment::Center)
    );
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("KeyTips.bfdoc");
    app.session.path = Some(path.clone());
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::F, Key::S], [1366., 900.]);
    assert!(!app.session.dirty());
    assert_eq!(storage::load(&path).unwrap(), app.session.editor.document);
}

#[test]
fn keytips_prefix_backspace_escape_size_entry_and_font_modal() {
    use keytips::Level;
    let (ctx, mut app) = setup();
    app.ribbon_collapsed = true;
    tip_sequence(
        &ctx,
        &mut app,
        &[Key::F10, Key::H, Key::F, Key::Z],
        [1366., 900.],
    );
    assert_eq!(app.keytips.prefix, "F");
    assert!(app.ribbon_temporary);
    assert!(app.regions.iter().any(|r| r.name == "keytip-FS"));
    assert!(!app.regions.iter().any(|r| r.name == "keytip-1"));
    tip_sequence(
        &ctx,
        &mut app,
        &[Key::Backspace, Key::F, Key::S],
        [1366., 900.],
    );
    assert_eq!(app.keytips.level, None);
    assert!(ctx.memory(|m| m.has_focus(Id::new(FONT_SIZE_ID))));
    edit_size(&ctx, &mut app, "19.5", Key::Enter);
    assert_eq!(
        app.session
            .editor
            .typing_style
            .font_size
            .unwrap_or_default(),
        FontSize::try_from(39).unwrap()
    );
    assert!(app.session.editor.document.text().is_empty());
    tip_sequence(
        &ctx,
        &mut app,
        &[Key::F10, Key::H, Key::F, Key::N],
        [1366., 900.],
    );
    assert!(app.font_dialog.is_some());
    alt_tap(&ctx, &mut app);
    assert_eq!(app.keytips.level, None);
    ribbon_key(&ctx, &mut app, Key::Escape, Modifiers::NONE, [1366., 900.]);
    assert!(app.font_dialog.is_none());
    tip_sequence(
        &ctx,
        &mut app,
        &[Key::F10, Key::H, Key::F, Key::Escape],
        [1366., 900.],
    );
    assert_eq!(app.keytips.level, Some(Level::Home));
    assert!(app.keytips.prefix.is_empty());
    tip_sequence(&ctx, &mut app, &[Key::Escape], [1366., 900.]);
    assert_eq!(app.keytips.level, Some(Level::Root));
    assert!(!app.ribbon_temporary);
    tip_sequence(&ctx, &mut app, &[Key::Escape], [1366., 900.]);
    assert_eq!(app.keytips.level, None);
    assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
}

#[test]
fn keytips_menus_narrow_overflow_resize_and_pointer_dismissal() {
    use keytips::Level;
    let (ctx, mut app) = setup();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Select me".into())],
        [640., 480.],
        1.,
    );
    app.ribbon_collapsed = true;
    tip_sequence(
        &ctx,
        &mut app,
        &[Key::F10, Key::H, Key::S, Key::L],
        [640., 480.],
    );
    assert_eq!(app.keytips.level, Some(Level::Select));
    assert!(egui::Popup::is_any_open(&ctx));
    frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    assert!(app.regions.iter().any(|r| r.name == "keytip-A"));
    tip_sequence(&ctx, &mut app, &[Key::A], [640., 480.]);
    assert_eq!(app.session.editor.selection, (0, 9));
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::H, Key::E], [640., 480.]);
    assert_eq!(app.keytips.level, Some(Level::Editing));
    tip_sequence(&ctx, &mut app, &[Key::F, Key::D], [640., 480.]);
    assert!(app.navigation);
    assert!(ctx.memory(|m| m.has_focus(Id::new("find-text"))));
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::H, Key::V], [1366., 900.]);
    assert!(egui::Popup::is_any_open(&ctx));
    tip_sequence(&ctx, &mut app, &[Key::Escape], [1366., 900.]);
    assert_eq!(app.keytips.level, Some(Level::Home));
    assert!(!egui::Popup::is_any_open(&ctx));
    click_region_at(&ctx, &mut app, "document-text", [1366., 900.]);
    assert_eq!(app.keytips.level, None);
    assert!(!app.ribbon_temporary);
}

#[test]
fn keytips_repeat_does_not_dispatch_twice_or_insert_paired_text() {
    let (ctx, mut app) = setup();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Words".into())],
        [1366., 900.],
        1.,
    );
    app.execute(Action::SelectAll, &ctx);
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::H], [1366., 900.]);
    let count = app.session.editor.undo_stack.len();
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Num1, false), egui::Event::Text("1".into())],
        [1366., 900.],
        1.,
    );
    let mut repeat = key(Key::Num1, false);
    if let egui::Event::Key { repeat: r, .. } = &mut repeat {
        *r = true;
    }
    frame(
        &ctx,
        &mut app,
        vec![repeat, egui::Event::Text("1".into())],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.session.editor.undo_stack.len(), count + 1);
    assert_eq!(app.session.editor.document.text(), "Words");
    ribbon_key(&ctx, &mut app, Key::Num1, Modifiers::NONE, [1366., 900.]);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Replacement".into())],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.session.editor.document.text(), "Replacement");
}

#[test]
fn keytips_badges_fit_real_controls_at_three_widths_and_scales() {
    for width in [640., 900., 1366.] {
        for dpi in [1., 1.25, 1.5] {
            let (ctx, mut app) = setup();
            app.ribbon_collapsed = true;
            frame(
                &ctx,
                &mut app,
                tip_events(&[Key::F10, Key::H]),
                [width * dpi, 900. * dpi],
                dpi,
            );
            for _ in 0..3 {
                frame(&ctx, &mut app, vec![], [width * dpi, 900. * dpi], dpi);
            }
            let tips: Vec<_> = app
                .regions
                .iter()
                .filter(|r| r.name.starts_with("keytip-"))
                .collect();
            assert!(tips.len() >= 13, "{width} {dpi}: {}", tips.len());
            let screen = Rect::from_min_size(Pos2::ZERO, vec2(width, 900.));
            for (index, tip) in tips.iter().enumerate() {
                assert!(
                    screen.contains_rect(tip.rect),
                    "{} {:?}",
                    tip.name,
                    tip.rect
                );
                for other in &tips[index + 1..] {
                    assert!(
                        !tip.rect.intersects(other.rect),
                        "{width} {dpi}: {} overlaps {}",
                        tip.name,
                        other.name
                    );
                }
            }
        }
    }
}

#[test]
fn keytips_restore_search_focus_and_ignore_altgr_command_flags() {
    let (ctx, mut app) = setup();
    ribbon_key(&ctx, &mut app, Key::Q, Modifiers::ALT, [1366., 900.]);
    assert!(ctx.memory(|m| m.has_focus(Id::new(SEARCH_ID))));
    alt_tap(&ctx, &mut app);
    tip_sequence(&ctx, &mut app, &[Key::Escape], [1366., 900.]);
    assert!(ctx.memory(|m| m.has_focus(Id::new(SEARCH_ID))));
    app.focus_document = true;
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    alt_tap(&ctx, &mut app);
    modifier_frame(&ctx, &mut app, Modifiers::NONE, false);
    assert!(app.keytips.level.is_none() && app.ribbon_focus.is_none());
    modifier_frame(&ctx, &mut app, Modifiers::NONE, true);
    app.focus_document = true;
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    let mut altgr = key(Key::H, true);
    if let egui::Event::Key { modifiers, .. } = &mut altgr {
        modifiers.alt = true;
    }
    frame(
        &ctx,
        &mut app,
        vec![altgr, egui::Event::Text("ł".into())],
        [1366., 900.],
        1.,
    );
    assert!(!app.navigation && !app.replace_mode);
    assert!(app.keytips.level.is_none());
    assert_eq!(app.session.editor.document.text(), "ł");
    assert_eq!(app.session.editor.undo_stack.len(), 1);
}

#[test]
fn qat_keytips_keep_fixed_numbers_and_use_shared_history_and_save() {
    use keytips::Level;
    let (ctx, mut app) = setup();
    tip_sequence(
        &ctx,
        &mut app,
        &[Key::F10, Key::Num2, Key::Num3],
        [1366., 900.],
    );
    assert_eq!(app.keytips.level, Some(Level::Root));
    for code in ["1", "2", "3"] {
        assert!(
            app.regions
                .iter()
                .any(|r| r.name == format!("keytip-{code}"))
        );
    }
    assert!(app.session.editor.document.text().is_empty());
    tip_sequence(&ctx, &mut app, &[Key::Escape], [1366., 900.]);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste(
            "English e\u{301}\nУкраїнська\nРусский 🙂".into(),
        )],
        [1366., 900.],
        1.,
    );
    app.execute(Action::SelectAll, &ctx);
    let original = app.session.editor.document.clone();
    tip_sequence(
        &ctx,
        &mut app,
        &[Key::F10, Key::H, Key::Num1],
        [1366., 900.],
    );
    let bold = app.session.editor.document.clone();
    app.ribbon_collapsed = true;
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::Num2], [1366., 900.]);
    assert_eq!(app.session.editor.document, original);
    ribbon_key(&ctx, &mut app, Key::Num3, Modifiers::ALT, [1366., 900.]);
    assert_eq!(app.session.editor.document, bold);
    assert!(app.ribbon_collapsed && !app.ribbon_temporary);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("Quick access.bfdoc");
    app.session.path = Some(path.clone());
    ribbon_key(&ctx, &mut app, Key::Num1, Modifiers::ALT, [1366., 900.]);
    assert_eq!(storage::load(&path).unwrap(), bold);
    assert!(!app.session.dirty());
    assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
}

#[test]
fn all_tab_keytips_keep_a_scope_and_escape_without_inserting_text() {
    use keytips::Level;
    let (ctx, mut app) = setup();
    app.ribbon_collapsed = true;
    for (keys, index) in [
        (vec![Key::N], 1),
        (vec![Key::J, Key::I], 2),
        (vec![Key::G], 3),
        (vec![Key::P], 4),
        (vec![Key::S], 5),
        (vec![Key::M], 6),
        (vec![Key::R], 7),
        (vec![Key::W], 8),
        (vec![Key::Y, Key::Num1], 9),
    ] {
        tip_sequence(&ctx, &mut app, &[Key::F10], [640., 480.]);
        tip_sequence(&ctx, &mut app, &keys, [640., 480.]);
        assert_eq!(app.tab, index);
        assert_eq!(app.keytips.level, Some(Level::Tab(index)));
        assert!(app.ribbon_temporary);
        tip_sequence(&ctx, &mut app, &[Key::Z], [640., 480.]);
        assert!(app.session.editor.document.text().is_empty());
        tip_sequence(&ctx, &mut app, &[Key::Escape], [640., 480.]);
        assert_eq!(app.keytips.level, Some(Level::Root));
        assert!(!app.ribbon_temporary);
        tip_sequence(&ctx, &mut app, &[Key::Escape], [640., 480.]);
        assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
    }
    assert!(app.session.editor.undo_stack.is_empty());
    ribbon_key(&ctx, &mut app, Key::J, Modifiers::ALT, [1366., 900.]);
    assert_eq!(app.keytips.prefix, "J");
    ribbon_key(&ctx, &mut app, Key::I, Modifiers::ALT, [1366., 900.]);
    assert_eq!(app.tab, 2);
    assert_eq!(app.keytips.level, Some(Level::Tab(2)));
}

#[test]
fn more_tabs_hints_show_hidden_routes_and_recover_after_resize() {
    use keytips::Level;
    let (ctx, mut app) = setup();
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::T], [640., 480.]);
    assert_eq!(app.keytips.level, Some(Level::Tabs));
    assert!(egui::Popup::is_any_open(&ctx));
    for code in ["M", "R", "W", "Y1"] {
        assert!(
            app.regions
                .iter()
                .any(|r| r.name == format!("keytip-{code}"))
        );
    }
    tip_sequence(&ctx, &mut app, &[Key::Escape], [640., 480.]);
    assert_eq!(app.keytips.level, Some(Level::Root));
    assert!(!egui::Popup::is_any_open(&ctx));
    tip_sequence(&ctx, &mut app, &[Key::T, Key::Y, Key::Num1], [640., 480.]);
    assert_eq!(app.tab, 9);
    assert!(!egui::Popup::is_any_open(&ctx));
    tip_sequence(&ctx, &mut app, &[Key::Escape, Key::T], [640., 480.]);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    assert_eq!(app.keytips.level, Some(Level::Root));
    assert!(!egui::Popup::is_any_open(&ctx));
    assert!(app.regions.iter().any(|r| r.name == "keytip-Y1"));
    assert!(app.session.editor.document.text().is_empty());
}

#[test]
fn view_keytips_toggle_real_state_and_zoom_draft_preserves_document() {
    let (ctx, mut app) = setup();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste("Українська e\u{301}".into())],
        [1366., 900.],
        1.,
    );
    app.execute(Action::SelectAll, &ctx);
    let document = app.session.editor.document.clone();
    let selection = app.session.editor.selection;
    let history = app.session.editor.undo_stack.len();
    let rulers = app.rulers;
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::W, Key::K], [1366., 900.]);
    assert!(app.navigation && app.checked(Action::Navigation));
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::W, Key::R], [1366., 900.]);
    assert_eq!(app.rulers, !rulers);
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::W, Key::Q], [1366., 900.]);
    assert!(app.zoom_dialog.is_some());
    assert!(ctx.memory(|m| m.has_focus(Id::new("zoom-percent"))));
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("125".into())],
        [1366., 900.],
        1.,
    );
    ribbon_key(&ctx, &mut app, Key::Escape, Modifiers::NONE, [1366., 900.]);
    assert_eq!(app.zoom, 100.);
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::W, Key::Q], [1366., 900.]);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("201".into())],
        [1366., 900.],
        1.,
    );
    ribbon_key(&ctx, &mut app, Key::Enter, Modifiers::NONE, [1366., 900.]);
    assert!(app.zoom_dialog.is_some());
    assert_eq!(app.zoom, 100.);
    // Modal shortcuts and KeyTips cannot reach the document behind the draft.
    tip_sequence(&ctx, &mut app, &[Key::F10], [1366., 900.]);
    frame(
        &ctx,
        &mut app,
        vec![key(Key::N, true), key(Key::B, true)],
        [1366., 900.],
        1.,
    );
    assert!(app.pending.is_none() && app.keytips.level.is_none());
    frame(&ctx, &mut app, vec![key(Key::A, true)], [1366., 900.], 1.);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("125".into())],
        [1366., 900.],
        1.,
    );
    ribbon_key(&ctx, &mut app, Key::Enter, Modifiers::NONE, [1366., 900.]);
    assert!(app.zoom_dialog.is_none());
    assert_eq!(app.zoom, 125.);
    assert_eq!(app.session.editor.document, document);
    assert_eq!(app.session.editor.selection, selection);
    assert_eq!(app.session.editor.undo_stack.len(), history);
    assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
}

#[test]
fn view_keytips_handoff_to_directional_navigation_and_collapsed_mode() {
    let (ctx, mut app) = setup();
    app.ribbon_collapsed = true;
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::W], [640., 480.]);
    ribbon_key(
        &ctx,
        &mut app,
        Key::ArrowRight,
        Modifiers::NONE,
        [640., 480.],
    );
    assert!(app.keytips.level.is_none());
    ribbon_key(
        &ctx,
        &mut app,
        Key::ArrowRight,
        Modifiers::CTRL,
        [640., 480.],
    );
    assert_eq!(
        app.ribbon_focus,
        Some(RibbonFocus::Control("command-Zoom".into()))
    );
    ribbon_key(&ctx, &mut app, Key::Enter, Modifiers::NONE, [640., 480.]);
    assert!(app.zoom_dialog.is_some());
    assert!(app.ribbon_collapsed && !app.ribbon_temporary);
    ribbon_key(&ctx, &mut app, Key::Escape, Modifiers::NONE, [640., 480.]);
    assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
}

#[test]
fn qat_tab_view_and_overflow_badges_fit_three_widths_and_scales() {
    for width in [640., 900., 1366.] {
        for dpi in [1., 1.25, 1.5] {
            for sequence in [
                vec![Key::F10],
                vec![Key::F10, Key::W],
                vec![Key::F10, Key::T],
            ] {
                let (ctx, mut app) = setup();
                frame(
                    &ctx,
                    &mut app,
                    tip_events(&sequence),
                    [width * dpi, 900. * dpi],
                    dpi,
                );
                for _ in 0..4 {
                    frame(&ctx, &mut app, vec![], [width * dpi, 900. * dpi], dpi);
                }
                let tips: Vec<_> = app
                    .regions
                    .iter()
                    .filter(|r| r.name.starts_with("keytip-"))
                    .collect();
                assert!(!tips.is_empty());
                let screen = Rect::from_min_size(Pos2::ZERO, vec2(width, 900.));
                for (index, tip) in tips.iter().enumerate() {
                    assert!(
                        screen.contains_rect(tip.rect),
                        "{} {:?}",
                        tip.name,
                        tip.rect
                    );
                    for other in &tips[index + 1..] {
                        assert!(
                            !tip.rect.intersects(other.rect),
                            "{width} {dpi}: {} overlaps {}",
                            tip.name,
                            other.name
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn zoom_dialog_bounds_cancel_and_close_guard_at_synthetic_scales() {
    for dpi in [1., 1.25, 1.5] {
        let (ctx, mut app) = setup();
        frame(
            &ctx,
            &mut app,
            tip_events(&[Key::F10, Key::W, Key::Q]),
            [640. * dpi, 480. * dpi],
            dpi,
        );
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], [640. * dpi, 480. * dpi], dpi);
        }
        let modal = app
            .regions
            .iter()
            .find(|r| r.name == "zoom-dialog")
            .unwrap();
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(640., 480.));
        assert!(screen.contains_rect(modal.rect));
        for control in app.regions.iter().filter(|r| {
            r.name.starts_with("zoom-preset-")
                || matches!(r.name.as_str(), "zoom-percent" | "zoom-ok" | "zoom-cancel")
        }) {
            assert!(modal.rect.contains_rect(control.rect));
        }
        let mut input = egui::RawInput::default();
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .events
            .push(egui::ViewportEvent::Close);
        let output = ctx.run(input, |ctx| app.update_ui(ctx));
        assert!(
            output.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .iter()
                .any(|c| matches!(c, ViewportCommand::CancelClose))
        );
        assert!(app.zoom_dialog.is_some());
        assert!(!app.allow_close);
        ribbon_key(&ctx, &mut app, Key::Escape, Modifiers::NONE, [640., 480.]);
        assert_eq!(app.zoom, 100.);
        assert!(!app.session.dirty());
    }
}

#[test]
fn customizable_quick_access_numbers_follow_visible_order_and_reset() {
    let (ctx, mut app) = setup();
    let before = app.session.editor.document.clone();
    click_region(&ctx, &mut app, "qat-options");
    click_region(&ctx, &mut app, "qat-toggle-New");
    assert_eq!(
        app.quick_access,
        [Action::Save, Action::Undo, Action::Redo, Action::New]
    );
    click_region(&ctx, &mut app, "qat-options");
    click_region(&ctx, &mut app, "qat-toggle-Save");
    assert_eq!(app.quick_access, [Action::Undo, Action::Redo, Action::New]);
    tip_sequence(&ctx, &mut app, &[Key::F10], [1366., 900.]);
    let tip = app.regions.iter().find(|r| r.name == "keytip-1").unwrap();
    let undo = app.regions.iter().find(|r| r.name == "qat-Undo").unwrap();
    assert!((tip.rect.center().x - undo.rect.center().x).abs() < 1.);
    tip_sequence(&ctx, &mut app, &[Key::Num1], [1366., 900.]);
    assert!(app.keytips.level.is_some()); // Disabled Undo keeps its number.
    tip_sequence(&ctx, &mut app, &[Key::Num0], [1366., 900.]);
    assert!(egui::Popup::is_any_open(&ctx));
    click_region(&ctx, &mut app, "qat-reset");
    click_region(&ctx, &mut app, "qat-reset-confirm");
    assert_eq!(app.quick_access, quick_access::DEFAULT);
    assert!(!app.quick_access_below);
    assert_eq!(app.session.editor.document, before);
    assert!(app.session.editor.undo_stack.is_empty());
}

#[test]
fn quick_access_keyboard_skips_disabled_and_menu_returns_to_toolbar() {
    let (ctx, mut app) = setup();
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::ArrowUp], [1366., 900.]);
    assert_eq!(
        app.ribbon_focus,
        Some(RibbonFocus::QuickAccess("qat-Save".into()))
    );
    ribbon_key(
        &ctx,
        &mut app,
        Key::ArrowRight,
        Modifiers::NONE,
        [1366., 900.],
    );
    assert_eq!(
        app.ribbon_focus,
        Some(RibbonFocus::QuickAccess("qat-options".into()))
    );
    ribbon_key(
        &ctx,
        &mut app,
        Key::ArrowDown,
        Modifiers::NONE,
        [1366., 900.],
    );
    ribbon_key(&ctx, &mut app, Key::Space, Modifiers::NONE, [1366., 900.]);
    assert!(app.quick_access.contains(&Action::New));
    assert!(!egui::Popup::is_any_open(&ctx));
    ribbon_key(&ctx, &mut app, Key::Escape, Modifiers::NONE, [1366., 900.]);
    assert_eq!(app.ribbon_focus, Some(RibbonFocus::Tab));
    ribbon_key(&ctx, &mut app, Key::Escape, Modifiers::NONE, [1366., 900.]);
    assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
    assert!(app.session.editor.document.text().is_empty());
}

#[test]
fn quick_access_below_ribbon_preserves_canvas_and_supports_empty_toolbar() {
    let (ctx, mut app) = setup();
    click_region(&ctx, &mut app, "qat-options");
    click_region(&ctx, &mut app, "qat-position");
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    }
    assert!(app.quick_access_below);
    let qat = app
        .regions
        .iter()
        .find(|r| r.name == "qat-Save")
        .unwrap()
        .rect;
    let ribbon = app
        .regions
        .iter()
        .find(|r| r.name == "ribbon")
        .unwrap()
        .rect;
    let workspace = app
        .regions
        .iter()
        .find(|r| r.name == "workspace")
        .unwrap()
        .rect;
    assert!(qat.top() >= ribbon.bottom() && qat.bottom() <= workspace.top());
    app.quick_access.clear();
    app.ribbon_collapsed = true;
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    }
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::Num0], [640., 480.]);
    assert!(egui::Popup::is_any_open(&ctx));
    click_region_at(&ctx, &mut app, "qat-reset", [640., 480.]);
    click_region_at(&ctx, &mut app, "qat-reset-confirm", [640., 480.]);
    assert_eq!(app.quick_access, quick_access::DEFAULT);
    assert!(!app.quick_access_below);
}

#[test]
fn canvas_width_tracks_workspace_and_manual_zoom_leaves_fit_without_history() {
    let (ctx, mut app) = setup();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste(
            "English e\u{301}\nУкраїнська\nРусский".into(),
        )],
        [1366., 900.],
        1.,
    );
    app.execute(Action::SelectAll, &ctx);
    app.execute(Action::Bold, &ctx);
    let before = app.session.editor.document.clone();
    let history = app.session.editor.undo_stack.len();
    let selection = app.session.editor.selection;
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::W, Key::C], [1366., 900.]);
    assert!(app.zoom_width);
    let wide = app.zoom;
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![], [960., 720.], 1.);
    }
    assert!(app.zoom < wide);
    let narrow = app.zoom;
    app.execute(Action::Navigation, &ctx);
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![], [960., 720.], 1.);
    }
    assert!(app.zoom < narrow);
    app.execute(Action::Zoom, &ctx);
    ribbon_key(&ctx, &mut app, Key::Escape, Modifiers::NONE, [960., 720.]);
    assert!(app.zoom_width);
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::W, Key::J], [960., 720.]);
    assert_eq!(app.zoom, 100.);
    assert!(!app.zoom_width);
    app.execute(Action::ZoomWidth, &ctx);
    click_region_at(&ctx, &mut app, "zoom-in", [960., 720.]);
    assert!(!app.zoom_width);
    assert_eq!(app.session.editor.document, before);
    assert_eq!(app.session.editor.undo_stack.len(), history);
    assert_eq!(app.session.editor.selection, selection);
    let dir = tempfile::tempdir().unwrap();
    app.session.path = Some(dir.path().join("view.bfdoc"));
    app.execute(Action::Save, &ctx);
    assert_eq!(
        storage::load(app.session.path.as_ref().unwrap()).unwrap(),
        before
    );
}

#[test]
fn view_and_custom_toolbar_bounds_at_synthetic_scales() {
    for width in [640., 960., 1366.] {
        for dpi in [1., 1.25, 1.5] {
            let (ctx, mut app) = setup();
            app.tab = 8;
            app.quick_access.extend([Action::New, Action::Open]);
            for below in [false, true] {
                app.quick_access_below = below;
                for _ in 0..4 {
                    frame(&ctx, &mut app, vec![], [width * dpi, 900. * dpi], dpi);
                }
                let screen = Rect::from_min_size(Pos2::ZERO, vec2(width, 900.));
                let targets: Vec<_> = app
                    .quick_access_targets
                    .iter()
                    .chain(app.ribbon_targets.iter())
                    .collect();
                for (i, target) in targets.iter().enumerate() {
                    assert!(
                        screen.contains_rect(target.rect),
                        "{} {:?}",
                        target.name,
                        target.rect
                    );
                    for other in &targets[i + 1..] {
                        assert!(
                            !target.rect.intersects(other.rect),
                            "{} / {}",
                            target.name,
                            other.name
                        );
                    }
                }
                let ruler = app
                    .regions
                    .iter()
                    .find(|r| r.name == "command-Rulers")
                    .unwrap()
                    .rect;
                let navigation = app
                    .regions
                    .iter()
                    .find(|r| r.name == "command-Navigation")
                    .unwrap()
                    .rect;
                assert!((ruler.left() - navigation.left()).abs() < 0.1);
                let title = app
                    .regions
                    .iter()
                    .find(|r| r.name == "document-title")
                    .unwrap()
                    .rect;
                for target in &app.quick_access_targets {
                    assert!(!title.intersects(target.rect));
                }
                let work = app
                    .regions
                    .iter()
                    .find(|r| r.name == "workspace")
                    .unwrap()
                    .rect;
                for target in targets {
                    assert!(!work.intersects(target.rect));
                }
            }
        }
    }
}

#[test]
fn below_toolbar_stays_accessible_with_temporary_ribbon_and_direct_number_entry() {
    let (ctx, mut app) = setup();
    app.quick_access_below = true;
    app.quick_access.push(Action::New);
    app.ribbon_collapsed = true;
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste("Keep this text".into())],
        [1366., 900.],
        1.,
    );
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::H], [1366., 900.]);
    let ribbon = app
        .regions
        .iter()
        .find(|r| r.name == "ribbon")
        .unwrap()
        .rect;
    for target in &app.quick_access_targets {
        assert!(!ribbon.intersects(target.rect));
    }
    tip_sequence(
        &ctx,
        &mut app,
        &[Key::Escape, Key::ArrowDown],
        [1366., 900.],
    );
    assert_eq!(
        app.ribbon_focus,
        Some(RibbonFocus::QuickAccess("qat-Save".into()))
    );
    ribbon_key(&ctx, &mut app, Key::Num4, Modifiers::ALT, [1366., 900.]);
    assert!(app.pending.is_some());
    click_region(&ctx, &mut app, "unsaved-cancel");
    assert_eq!(app.session.editor.document.text(), "Keep this text");
}

fn setup_preferences(path: &std::path::Path) -> (egui::Context, DocsApp) {
    let ctx = egui::Context::default();
    let mut app = DocsApp::with_preferences(Session::default(), &ctx, Ok(path.to_owned()));
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    }
    (ctx, app)
}

#[test]
fn toolbar_changes_survive_new_app_with_numbered_hints_and_leave_document_history_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let (ctx, mut app) = setup_preferences(&path);
    assert!(!path.exists());
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste(
            "English e\u{301}\nУкраїнська\nРусский".into(),
        )],
        [1366., 900.],
        1.,
    );
    app.execute(Action::SelectAll, &ctx);
    app.execute(Action::Bold, &ctx);
    let before = app.session.editor.document.clone();
    let selection = app.session.editor.selection;
    let history = app.session.editor.undo_stack.len();
    for target in [
        "qat-toggle-Save",
        "qat-toggle-Open",
        "qat-toggle-Save",
        "qat-position",
    ] {
        click_region(&ctx, &mut app, "qat-options");
        click_region(&ctx, &mut app, target);
    }
    assert!(app.preferences_warning.is_none());
    assert_eq!(app.session.editor.document, before);
    assert_eq!(app.session.editor.selection, selection);
    assert_eq!(app.session.editor.undo_stack.len(), history);
    let (ctx, mut reopened) = setup_preferences(&path);
    assert_eq!(
        reopened.quick_access,
        [Action::Undo, Action::Redo, Action::Open, Action::Save]
    );
    assert!(reopened.quick_access_below);
    assert!(!reopened.session.dirty());
    tip_sequence(&ctx, &mut reopened, &[Key::F10], [1366., 900.]);
    assert!(reopened.regions.iter().any(|r| r.name == "keytip-4"));
    assert_eq!(app.quick_access, reopened.quick_access);
}

#[test]
fn toolbar_reset_confirmation_blocks_editing_cancels_and_restores_persistent_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let (ctx, mut app) = setup_preferences(&path);
    click_region(&ctx, &mut app, "qat-options");
    click_region(&ctx, &mut app, "qat-toggle-New");
    let original = std::fs::read(&path).unwrap();
    click_region(&ctx, &mut app, "qat-options");
    click_region(&ctx, &mut app, "qat-reset");
    assert!(app.quick_reset);
    frame(
        &ctx,
        &mut app,
        vec![
            key(Key::B, true),
            key(Key::N, true),
            egui::Event::Text("blocked".into()),
        ],
        [1366., 900.],
        1.,
    );
    assert!(app.session.editor.document.text().is_empty());
    assert!(!app.session.editor.typing_style.bold.unwrap_or(false));
    assert!(app.quick_reset);
    ribbon_key(&ctx, &mut app, Key::Escape, Modifiers::NONE, [1366., 900.]);
    assert!(!app.quick_reset);
    assert_eq!(std::fs::read(&path).unwrap(), original);
    click_region(&ctx, &mut app, "qat-options");
    click_region(&ctx, &mut app, "qat-reset");
    click_region(&ctx, &mut app, "qat-reset-confirm");
    assert_eq!(app.quick_access, quick_access::DEFAULT);
    assert_eq!(
        std::fs::read(app.preferences_backup.as_ref().unwrap()).unwrap(),
        original
    );
    let (_, reopened) = setup_preferences(&path);
    assert_eq!(reopened.quick_access, quick_access::DEFAULT);
    assert!(!reopened.quick_access_below);
    assert!(!reopened.session.dirty());
}

#[test]
fn corrupt_settings_show_error_and_reset_backs_up_original_with_bounded_modal() {
    for dpi in [1., 1.25, 1.5] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, b"broken preferences").unwrap();
        let (ctx, mut app) = setup_preferences(&path);
        assert!(app.error.is_some());
        click_region(&ctx, &mut app, "error-ok");
        assert!(app.preferences_warning.is_some());
        click_region_at(&ctx, &mut app, "qat-options", [640., 480.]);
        click_region_at(&ctx, &mut app, "qat-reset", [640., 480.]);
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], [640. * dpi, 480. * dpi], dpi);
        }
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(640., 480.));
        for name in ["qat-reset-confirm", "qat-reset-cancel"] {
            assert!(
                screen.contains_rect(app.regions.iter().find(|r| r.name == name).unwrap().rect)
            );
        }
        click_region_at(&ctx, &mut app, "qat-reset-confirm", [640., 480.]);
        assert!(app.error.is_none());
        assert!(app.preferences_warning.is_none());
        assert_eq!(
            std::fs::read(app.preferences_backup.as_ref().unwrap()).unwrap(),
            b"broken preferences"
        );
    }
}

#[test]
fn toolbar_save_failure_is_visible_and_retry_persists_without_changing_document() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let lock = dir.path().join(".toolbar-settings.lock");
    let (ctx, mut app) = setup_preferences(&path);
    std::fs::write(&lock, b"busy").unwrap();
    click_region(&ctx, &mut app, "qat-options");
    click_region(&ctx, &mut app, "qat-toggle-Open");
    assert!(app.error.is_some());
    assert!(app.quick_access.contains(&Action::Open));
    assert!(!path.exists());
    click_region(&ctx, &mut app, "error-ok");
    std::fs::remove_file(lock).unwrap();
    click_region(&ctx, &mut app, "qat-options");
    click_region(&ctx, &mut app, "qat-retry");
    assert!(app.error.is_none());
    assert!(app.preferences_warning.is_none());
    assert!(!app.session.dirty());
    let (_, reopened) = setup_preferences(&path);
    assert_eq!(app.quick_access, reopened.quick_access);
}

#[test]
fn failed_reset_retry_keeps_backup_requirement_and_current_toolbar_draft() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let lock = dir.path().join(".toolbar-settings.lock");
    let (ctx, mut app) = setup_preferences(&path);
    click_region(&ctx, &mut app, "qat-options");
    click_region(&ctx, &mut app, "qat-toggle-Open");
    let original = std::fs::read(&path).unwrap();
    std::fs::write(&lock, b"busy").unwrap();
    click_region(&ctx, &mut app, "qat-options");
    click_region(&ctx, &mut app, "qat-reset");
    click_region(&ctx, &mut app, "qat-reset-confirm");
    assert!(app.preferences_reset_pending);
    assert!(app.error.is_some());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    click_region(&ctx, &mut app, "error-ok");
    // A further toolbar change must not turn a failed reset into a normal save.
    click_region(&ctx, &mut app, "qat-options");
    click_region(&ctx, &mut app, "qat-toggle-New");
    assert!(app.error.is_some());
    assert!(app.preferences_reset_pending);
    click_region(&ctx, &mut app, "error-ok");
    std::fs::remove_file(lock).unwrap();
    click_region(&ctx, &mut app, "qat-options");
    click_region(&ctx, &mut app, "qat-retry");
    assert!(app.error.is_none());
    assert!(!app.preferences_reset_pending);
    assert_eq!(
        std::fs::read(app.preferences_backup.as_ref().unwrap()).unwrap(),
        original
    );
    let (_, reopened) = setup_preferences(&path);
    assert_eq!(
        reopened.quick_access,
        [Action::Save, Action::Undo, Action::Redo, Action::New]
    );
    assert!(!app.session.dirty());
}

fn open_order(ctx: &egui::Context, app: &mut DocsApp) {
    click_region(ctx, app, "qat-options");
    click_region(ctx, app, "qat-order");
    assert!(app.quick_order.is_some());
}

#[test]
fn toolbar_order_apply_roundtrips_and_renumbers_real_history_actions() {
    for below in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let (ctx, mut app) = setup_preferences(&path);
        app.quick_access_below = below;
        click_region(&ctx, &mut app, "qat-options");
        click_region(&ctx, &mut app, "qat-toggle-Open");
        let saved = std::fs::read(&path).unwrap();
        app.leave_ribbon(&ctx);
        frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::Paste(
                "English e\u{301}\nУкраїнська\nРусский".into(),
            )],
            [1366., 900.],
            1.,
        );
        app.execute(Action::SelectAll, &ctx);
        app.execute(Action::Bold, &ctx);
        let document = app.session.editor.document.clone();
        let selection = app.session.editor.selection;
        let history = app.session.editor.undo_stack.len();
        open_order(&ctx, &mut app);
        click_region(&ctx, &mut app, "qat-order-Open");
        click_region(&ctx, &mut app, "qat-order-up");
        click_region(&ctx, &mut app, "qat-order-up");
        assert_eq!(std::fs::read(&path).unwrap(), saved);
        assert_eq!(
            app.quick_access,
            [Action::Save, Action::Undo, Action::Redo, Action::Open]
        );
        click_region(&ctx, &mut app, "qat-order-ok");
        assert!(app.quick_order.is_none() && app.preferences_warning.is_none());
        assert_eq!(
            app.quick_access,
            [Action::Save, Action::Open, Action::Undo, Action::Redo]
        );
        assert_eq!(
            app.ribbon_focus,
            Some(RibbonFocus::QuickAccess("qat-options".into()))
        );
        let (_, restored) = setup_preferences(&path);
        assert_eq!(restored.quick_access, app.quick_access);
        assert_eq!(restored.quick_access_below, below);
        assert_eq!(app.session.editor.document, document);
        assert_eq!(app.session.editor.selection, selection);
        assert_eq!(app.session.editor.undo_stack.len(), history);
        let doc_path = dir.path().join("order.bfdoc");
        app.session.save(&doc_path).unwrap();
        assert_eq!(storage::load(&doc_path).unwrap(), document);
        tip_sequence(&ctx, &mut app, &[Key::F10], [1366., 900.]);
        let get = |name: &str| app.regions.iter().find(|r| r.name == name).unwrap().rect;
        assert!((get("keytip-3").center().x - get("qat-Undo").center().x).abs() < 1.);
        tip_sequence(&ctx, &mut app, &[Key::Num3], [1366., 900.]);
        assert!(
            !app.session.editor.document.paragraphs[0].styles[0]
                .bold
                .unwrap_or(false)
        );
        assert_eq!(app.session.editor.undo_stack.len(), history - 1);
    }
}

#[test]
fn toolbar_order_keyboard_keeps_selection_focus_and_cancel_blocks_background_edits() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let (ctx, mut app) = setup_preferences(&path);
    tip_sequence(&ctx, &mut app, &[Key::F10, Key::Num0], [1366., 900.]);
    for _ in 0..7 {
        ribbon_key(
            &ctx,
            &mut app,
            Key::ArrowDown,
            Modifiers::NONE,
            [1366., 900.],
        );
    }
    ribbon_key(&ctx, &mut app, Key::Enter, Modifiers::NONE, [1366., 900.]);
    assert!(app.quick_order.is_some());
    ribbon_key(
        &ctx,
        &mut app,
        Key::ArrowDown,
        Modifiers::NONE,
        [1366., 900.],
    );
    let undo_focus = ctx.memory(|m| m.focused());
    ribbon_key(
        &ctx,
        &mut app,
        Key::ArrowDown,
        Modifiers::ALT,
        [1366., 900.],
    );
    assert_eq!(ctx.memory(|m| m.focused()), undo_focus);
    ribbon_key(&ctx, &mut app, Key::Tab, Modifiers::NONE, [1366., 900.]);
    ribbon_key(&ctx, &mut app, Key::Enter, Modifiers::NONE, [1366., 900.]);
    assert!(app.quick_order.is_some()); // Enter activates Move Up, not default OK.
    ribbon_key(&ctx, &mut app, Key::Enter, Modifiers::NONE, [1366., 900.]);
    assert_eq!(ctx.memory(|m| m.focused()), undo_focus); // Boundary returns to selected row.
    frame(
        &ctx,
        &mut app,
        vec![
            key(Key::B, true),
            key(Key::N, true),
            key(Key::S, true),
            key(Key::F10, false),
            egui::Event::Paste("blocked".into()),
            egui::Event::Text("blocked".into()),
        ],
        [1366., 900.],
        1.,
    );
    assert!(app.session.editor.document.text().is_empty());
    assert!(!app.session.editor.typing_style.bold.unwrap_or(false));
    assert!(app.session.editor.undo_stack.is_empty());
    assert!(app.pending.is_none() && app.keytips.level.is_none());
    let mut input = egui::RawInput::default();
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .events
        .push(egui::ViewportEvent::Close);
    let output = ctx.run(input, |ctx| app.update_ui(ctx));
    assert!(
        output.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .contains(&ViewportCommand::CancelClose)
    );
    ribbon_key(&ctx, &mut app, Key::Escape, Modifiers::NONE, [1366., 900.]);
    assert!(app.quick_order.is_none());
    assert_eq!(app.quick_access, quick_access::DEFAULT);
    assert!(!path.exists());
    assert_eq!(
        app.ribbon_focus,
        Some(RibbonFocus::QuickAccess("qat-options".into()))
    );
}

#[test]
fn toolbar_order_failed_save_preserves_previous_file_and_retry_keeps_new_order() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let (ctx, mut app) = setup_preferences(&path);
    click_region(&ctx, &mut app, "qat-options");
    click_region(&ctx, &mut app, "qat-toggle-Open");
    let original = std::fs::read(&path).unwrap();
    let lock = dir.path().join(".toolbar-settings.lock");
    std::fs::write(&lock, b"busy").unwrap();
    open_order(&ctx, &mut app);
    click_region(&ctx, &mut app, "qat-order-down");
    click_region(&ctx, &mut app, "qat-order-ok");
    assert!(app.error.is_some() && app.preferences_warning.is_some());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(
        app.quick_access,
        [Action::Undo, Action::Save, Action::Redo, Action::Open]
    );
    click_region(&ctx, &mut app, "error-ok");
    std::fs::remove_file(lock).unwrap();
    click_region(&ctx, &mut app, "qat-options");
    click_region(&ctx, &mut app, "qat-retry");
    let (_, reopened) = setup_preferences(&path);
    assert_eq!(reopened.quick_access, app.quick_access);
    assert!(app.error.is_none() && app.preferences_warning.is_none());
    assert!(!app.session.dirty());
}

#[test]
fn toolbar_order_empty_single_and_all_commands_fit_without_noop_writes() {
    for dpi in [1., 1.25, 1.5] {
        for width in [640., 1366.] {
            for commands in [
                vec![],
                vec![Action::Save],
                vec![
                    Action::New,
                    Action::Open,
                    Action::Save,
                    Action::Undo,
                    Action::Redo,
                ],
            ] {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("settings.json");
                let (ctx, mut app) = setup_preferences(&path);
                app.quick_access = commands.clone();
                open_order(&ctx, &mut app);
                for _ in 0..3 {
                    frame(&ctx, &mut app, vec![], [width * dpi, 480. * dpi], dpi);
                }
                let screen = Rect::from_min_size(Pos2::ZERO, vec2(width, 480.));
                let bounds = app
                    .regions
                    .iter()
                    .find(|r| r.name == "qat-order-dialog")
                    .unwrap()
                    .rect;
                assert!(screen.contains_rect(bounds), "{width}/{dpi}: {bounds:?}");
                for r in app
                    .regions
                    .iter()
                    .filter(|r| r.name.starts_with("qat-order-"))
                {
                    assert!(bounds.contains_rect(r.rect), "{} {width}/{dpi}", r.name);
                }
                ribbon_key(&ctx, &mut app, Key::ArrowUp, Modifiers::ALT, [width, 480.]);
                ribbon_key(&ctx, &mut app, Key::Enter, Modifiers::NONE, [width, 480.]);
                assert!(app.quick_order.is_none());
                assert_eq!(app.quick_access, commands);
                assert!(!path.exists());
            }
        }
    }
}

#[test]
fn toolbar_order_cancel_and_unchanged_ok_preserve_existing_profile_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let (ctx, mut app) = setup_preferences(&path);
    click_region(&ctx, &mut app, "qat-options");
    click_region(&ctx, &mut app, "qat-toggle-New");
    let original = std::fs::read(&path).unwrap();
    let commands = app.quick_access.clone();
    open_order(&ctx, &mut app);
    click_region(&ctx, &mut app, "qat-order-down");
    click_region(&ctx, &mut app, "qat-order-cancel");
    assert_eq!(app.quick_access, commands);
    assert_eq!(std::fs::read(&path).unwrap(), original);
    open_order(&ctx, &mut app);
    ribbon_key(
        &ctx,
        &mut app,
        Key::ArrowDown,
        Modifiers::ALT,
        [1366., 900.],
    );
    // Reverse traversal from the list reaches Cancel, not OK.
    ribbon_key(&ctx, &mut app, Key::Tab, Modifiers::SHIFT, [1366., 900.]);
    ribbon_key(&ctx, &mut app, Key::Enter, Modifiers::NONE, [1366., 900.]);
    assert!(app.quick_order.is_none());
    assert_eq!(app.quick_access, commands);
    assert_eq!(std::fs::read(&path).unwrap(), original);
    // An unchanged OK must not even attempt a write under an existing lock.
    std::fs::write(dir.path().join(".toolbar-settings.lock"), b"busy").unwrap();
    open_order(&ctx, &mut app);
    click_region(&ctx, &mut app, "qat-order-ok");
    assert!(app.error.is_none() && app.preferences_warning.is_none());
    assert_eq!(std::fs::read(&path).unwrap(), original);
}

#[test]
fn search_options_keyboard_recompute_without_editing_and_escape_returns_to_document() {
    let (ctx, mut app) = setup();
    app.session
        .editor
        .replace(0, 0, "cat CAT cats", "Input")
        .unwrap();
    app.sync_view();
    let before = app.session.editor.document.clone();
    let history = app.session.editor.undo_stack.len();
    app.execute(Action::Find, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("cat".into())],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.find_query.search(&app.buffer).matches.len(), 2);
    for _ in 0..2 {
        frame(
            &ctx,
            &mut app,
            vec![key(Key::Tab, false)],
            [1366., 900.],
            1.,
        );
    }
    assert!(ctx.memory(|m| m.has_focus(app.find_option_ids[0].unwrap())));
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Space, false)],
        [1366., 900.],
        1.,
    );
    assert!(!app.find_options.match_case);
    assert_eq!(app.find_query.search(&app.buffer).matches.len(), 3);
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Tab, false)],
        [1366., 900.],
        1.,
    );
    assert!(ctx.memory(|m| m.has_focus(app.find_option_ids[1].unwrap())));
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Space, false)],
        [1366., 900.],
        1.,
    );
    assert!(app.find_options.whole_words);
    assert_eq!(app.find_query.search(&app.buffer).matches.len(), 2);
    assert_eq!(app.session.editor.document, before);
    assert_eq!(app.session.editor.undo_stack.len(), history);
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Escape, false)],
        [1366., 900.],
        1.,
    );
    assert!(!app.navigation);
    assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
    app.execute(Action::Find, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    click_region(&ctx, &mut app, "find-clear");
    assert!(!app.find_options.match_case && app.find_options.whole_words);
    assert!(app.find_query.search(&app.buffer).matches.is_empty());
    assert_eq!(app.session.editor.document, before);
}

#[test]
fn search_options_replacement_uses_original_ranges_and_history_refresh() {
    let (ctx, mut app) = setup();
    app.session.editor.replace(0, 0, "K Kx k", "Input").unwrap();
    app.session.editor.emphasize(0, 1, true, true).unwrap();
    app.sync_view();
    let before = app.session.editor.document.clone();
    let history = app.session.editor.undo_stack.len();
    app.execute(Action::Replace, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("k".into())],
        [1366., 900.],
        1.,
    );
    click_region(&ctx, &mut app, "find-case");
    click_region(&ctx, &mut app, "find-words");
    assert_eq!(app.session.editor.selection, (0, 1));
    assert_eq!(app.find_query.search(&app.buffer).matches.len(), 2);
    app.replacement = "e\u{301}".into();
    click_region(&ctx, &mut app, "find-replace");
    let after = app.session.editor.document.clone();
    assert_eq!(after.text(), "e\u{301} Kx k");
    assert!(after.paragraphs[0].styles[0].bold.unwrap_or(false));
    assert_eq!(app.session.editor.undo_stack.len(), history + 1);
    app.execute(Action::Undo, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    assert_eq!(app.session.editor.document, before);
    assert_eq!(app.find_query.search(&app.buffer).matches.len(), 2);
    app.execute(Action::Redo, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    assert_eq!(app.session.editor.document, after);
    assert_eq!(app.find_query.search(&app.buffer).matches.len(), 1);
}

#[test]
fn oversized_search_is_reported_without_replacing_or_truncating_input() {
    let (ctx, mut app) = setup();
    app.session
        .editor
        .replace(0, 0, "Україна", "Input")
        .unwrap();
    app.sync_view();
    let before = app.session.editor.document.clone();
    app.execute(Action::Replace, &ctx);
    app.find = "ї".repeat(2049);
    app.replacement = "wrong".into();
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    assert!(app.find_query.error().is_some());
    assert_eq!(app.find.len(), 4098);
    click_region(&ctx, &mut app, "find-replace");
    assert_eq!(app.session.editor.document, before);
    assert_eq!(app.session.editor.undo_stack.len(), 1);
}

#[test]
fn replace_all_preview_cancel_keyboard_apply_and_single_undo_are_real() {
    let (ctx, mut app) = setup();
    app.session
        .editor
        .replace(0, 0, "cat CAT cats\nУкраїна e\u{301}", "Input")
        .unwrap();
    app.session.editor.emphasize(0, 3, true, true).unwrap();
    app.sync_view();
    app.execute(Action::Replace, &ctx);
    app.find = "cat".into();
    app.replacement = "ї".into();
    app.find_options = crate::search::Options {
        match_case: false,
        whole_words: true,
    };
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    let original = app.session.editor.document.clone();
    let selection = app.session.editor.selection;
    let history = app.session.editor.undo_stack.len();
    click_region(&ctx, &mut app, "find-replace-all");
    assert_eq!(
        app.replace_all_dialog.as_ref().unwrap().plan.change_count(),
        2
    );
    assert_eq!(app.session.editor.document, original);
    assert_eq!(app.session.editor.selection, selection);
    // Initial focus is Cancel: Enter must not accidentally replace the document.
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Enter, false)],
        [1366., 900.],
        1.,
    );
    assert!(app.replace_all_dialog.is_none());
    assert_eq!(app.session.editor.document, original);
    click_region(&ctx, &mut app, "find-replace-all");
    for command in [Key::B, Key::Z, Key::N, Key::S] {
        frame(&ctx, &mut app, vec![key(command, true)], [1366., 900.], 1.);
    }
    assert_eq!(app.session.editor.document, original);
    assert_eq!(app.session.editor.undo_stack.len(), history);
    assert!(app.pending.is_none());
    assert!(app.session.path.is_none());
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Escape, false)],
        [1366., 900.],
        1.,
    );
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    assert!(app.replace_all_dialog.is_none());
    assert!(ctx.memory(|m| m.has_focus(Id::new("find-text"))));
    click_region(&ctx, &mut app, "find-replace-all");
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Tab, false)],
        [1366., 900.],
        1.,
    );
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Enter, false)],
        [1366., 900.],
        1.,
    );
    assert!(app.replace_all_dialog.is_none());
    let after = app.session.editor.document.clone();
    assert_eq!(after.text(), "ї ї cats\nУкраїна e\u{301}");
    assert!(after.paragraphs[0].styles[0].bold.unwrap_or(false));
    assert_eq!(app.session.editor.undo_stack.len(), history + 1);
    assert_eq!(
        app.replace_all_notice.as_deref(),
        Some("Replaced 2 matches.")
    );
    app.execute(Action::Undo, &ctx);
    assert!(app.replace_all_notice.is_none());
    assert_eq!(app.session.editor.document, original);
    app.execute(Action::Redo, &ctx);
    assert_eq!(app.session.editor.document, after);
}

#[test]
fn replace_all_preview_stale_and_size_errors_preserve_current_document() {
    let (ctx, mut app) = setup();
    app.session.editor.replace(0, 0, "a a", "Input").unwrap();
    app.sync_view();
    app.execute(Action::Replace, &ctx);
    app.find = "a".into();
    app.replacement = "b".into();
    click_region(&ctx, &mut app, "find-replace-all");
    app.session.editor.emphasize(0, 1, true, true).unwrap();
    let current = app.session.editor.document.clone();
    let count = app.session.editor.undo_stack.len();
    click_region(&ctx, &mut app, "replace-all-apply");
    assert!(app.error.as_ref().unwrap().contains("document changed"));
    assert_eq!(app.session.editor.document, current);
    assert_eq!(app.session.editor.undo_stack.len(), count);
    click_region(&ctx, &mut app, "error-ok");
    app.replacement = "x".repeat(crate::model::MAX_CHARACTERS);
    click_region(&ctx, &mut app, "find-replace-all");
    assert!(app.error.as_ref().unwrap().contains("limit"));
    assert!(app.replace_all_dialog.is_none());
    assert_eq!(app.session.editor.document, current);
    assert_eq!(app.session.editor.undo_stack.len(), count);
}

#[test]
fn replace_all_preview_bounds_virtual_rows_noops_and_close_guard() {
    for dpi in [1., 1.25, 1.5] {
        for size in [[640., 480.], [1024., 768.], [1366., 900.]] {
            let (ctx, mut app) = setup();
            app.session
                .editor
                .replace(0, 0, &"Україна e\u{301}\n".repeat(80), "Input")
                .unwrap();
            app.sync_view();
            app.replace_all_dialog = Some(
                ReplaceAllDialog::new(
                    &app.session.editor,
                    "Україна",
                    "Україна",
                    Default::default(),
                )
                .unwrap(),
            );
            for _ in 0..4 {
                frame(&ctx, &mut app, vec![], size, dpi);
            }
            let modal = app
                .regions
                .iter()
                .find(|r| r.name == "replace-all-preview")
                .unwrap()
                .rect;
            assert!(
                Rect::from_min_size(Pos2::ZERO, vec2(size[0] / dpi, size[1] / dpi))
                    .expand(1.)
                    .contains_rect(modal),
                "{size:?}/{dpi}: {modal:?}"
            );
            for name in ["replace-all-cancel", "replace-all-apply"] {
                assert!(
                    modal.contains_rect(app.regions.iter().find(|r| r.name == name).unwrap().rect)
                );
            }
            assert!(
                app.regions
                    .iter()
                    .filter(|r| r.name.starts_with("replace-all-row-"))
                    .count()
                    < 80
            );
            assert_eq!(
                app.replace_all_dialog.as_ref().unwrap().plan.change_count(),
                0
            );
            let mut input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    Pos2::ZERO,
                    vec2(size[0] / dpi, size[1] / dpi),
                )),
                ..Default::default()
            };
            input
                .viewports
                .get_mut(&egui::ViewportId::ROOT)
                .unwrap()
                .events
                .push(egui::ViewportEvent::Close);
            let output = ctx.run(input, |ctx| app.update_ui(ctx));
            assert!(
                output.viewport_output[&egui::ViewportId::ROOT]
                    .commands
                    .contains(&ViewportCommand::CancelClose)
            );
            assert!(app.replace_all_dialog.is_some());
            assert_eq!(app.session.editor.undo_stack.len(), 1);
        }
    }
}

#[test]
fn replacement_ime_cannot_open_replace_all_during_confirmation() {
    let (ctx, mut app) = setup();
    app.session.editor.replace(0, 0, "a a", "Input").unwrap();
    app.sync_view();
    app.execute(Action::Replace, &ctx);
    app.find = "a".into();
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    ctx.memory_mut(|m| m.request_focus(Id::new("replace-text")));
    frame(
        &ctx,
        &mut app,
        vec![
            egui::Event::Ime(egui::ImeEvent::Enabled),
            egui::Event::Ime(egui::ImeEvent::Preedit("ї".into())),
        ],
        [1366., 900.],
        1.,
    );
    assert!(app.replace_composing);
    let before = app.session.editor.document.clone();
    frame(
        &ctx,
        &mut app,
        vec![
            egui::Event::Ime(egui::ImeEvent::Commit("ї".into())),
            key(Key::Enter, false),
        ],
        [1366., 900.],
        1.,
    );
    assert!(app.replace_all_dialog.is_none());
    assert_eq!(app.session.editor.document, before);
    assert_eq!(app.session.editor.undo_stack.len(), 1);
}

#[test]
fn clear_formatting_pointer_keytips_and_search_share_one_history_operation() {
    for width in [640., 960., 1366.] {
        let (ctx, mut app) = setup();
        app.session
            .editor
            .replace(0, 0, "English e\u{301}\nУкраїна Русский", "Type")
            .unwrap();
        let end = app.session.editor.document.character_count();
        app.session.editor.emphasize(0, end, true, true).unwrap();
        app.session.editor.underline(0, end, true).unwrap();
        app.session
            .editor
            .set_font_size(0, 7, "24".parse().unwrap())
            .unwrap();
        app.session.editor.align(0, end, Alignment::Center).unwrap();
        app.execute(Action::SelectAll, &ctx);
        let before = app.session.editor.document.clone();
        let history = app.session.editor.undo_stack.len();
        click_region_at(&ctx, &mut app, "command-ClearFormatting", [width, 900.]);
        let after = app.session.editor.document.clone();
        assert_eq!(before.text(), after.text());
        assert_eq!(app.session.editor.selection, (0, end));
        assert_eq!(app.session.editor.undo_stack.len(), history + 1);
        assert!(
            after
                .paragraphs
                .iter()
                .all(|p| p.alignment == Alignment::Center
                    && p.styles.iter().all(|s| *s == CharacterFormat::default()))
        );
        app.execute(Action::Undo, &ctx);
        assert_eq!(app.session.editor.document, before);
        app.ribbon_collapsed = true;
        tip_sequence(
            &ctx,
            &mut app,
            &[Key::F10, Key::H, Key::F, Key::C],
            [width, 900.],
        );
        assert_eq!(app.session.editor.document, after);
        assert!(!app.ribbon_temporary);
        app.execute(Action::Undo, &ctx);
        ribbon_key(&ctx, &mut app, Key::Q, Modifiers::ALT, [width, 900.]);
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::Text("clear character".into())],
            [width, 900.],
            1.,
        );
        frame(
            &ctx,
            &mut app,
            vec![key(Key::Enter, false)],
            [width, 900.],
            1.,
        );
        assert_eq!(app.session.editor.document, after);
        assert!(!app.palette);
        assert_eq!(app.session.editor.undo_stack.len(), history + 1);
        app.execute(Action::ClearFormatting, &ctx);
        assert_eq!(app.session.editor.undo_stack.len(), history + 1);
    }
}

#[test]
fn clear_formatting_control_space_respects_focus_modifiers_and_modal_guards() {
    for command in [false, true] {
        let (ctx, mut app) = setup();
        app.session.editor.replace(0, 0, "Styled", "Type").unwrap();
        app.session.editor.emphasize(0, 6, true, true).unwrap();
        app.execute(Action::SelectAll, &ctx);
        frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
        let before = app.session.editor.document.clone();
        let ctrl = Modifiers {
            ctrl: true,
            command,
            ..Modifiers::NONE
        };
        ribbon_key(&ctx, &mut app, Key::Space, ctrl, [1366., 900.]);
        assert_eq!(
            app.session.editor.selected_font(),
            crate::commands::FontPatch {
                character_style: None,
                bold: Some(false),
                italic: Some(false),
                underline: Some(false),
                font_size: Some(FontSize::default())
            }
        );
        assert_eq!(app.session.editor.document.text(), "Styled");
        app.execute(Action::Undo, &ctx);
        for mods in [
            Modifiers { alt: true, ..ctrl },
            Modifiers {
                shift: true,
                ..ctrl
            },
            Modifiers {
                mac_cmd: true,
                command: true,
                ..Modifiers::NONE
            },
        ] {
            frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
            ribbon_key(&ctx, &mut app, Key::Space, mods, [1366., 900.]);
            assert_eq!(app.session.editor.document, before);
        }
        app.execute(Action::Find, &ctx);
        frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
        ribbon_key(&ctx, &mut app, Key::Space, ctrl, [1366., 900.]);
        assert_eq!(app.session.editor.document, before);
        app.execute(Action::FontDialog, &ctx);
        ribbon_key(&ctx, &mut app, Key::Space, ctrl, [1366., 900.]);
        app.execute(Action::ClearFormatting, &ctx);
        assert_eq!(app.session.editor.document, before);
        assert!(app.font_dialog.is_some());
    }
}

#[test]
fn clear_formatting_caret_and_composition_keep_pending_input_and_clean_document() {
    let (ctx, mut app) = setup();
    app.execute(Action::Bold, &ctx);
    assert!(app.session.editor.typing_style.bold.unwrap_or(false));
    click_region(&ctx, &mut app, "command-ClearFormatting");
    assert_eq!(app.session.editor.typing_style, CharacterFormat::default());
    assert!(!app.session.dirty());
    assert!(app.session.editor.undo_stack.is_empty());
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("ї e\u{301}".into())],
        [1366., 900.],
        1.,
    );
    assert!(
        app.session.editor.document.paragraphs[0]
            .styles
            .iter()
            .all(|s| *s == CharacterFormat::default())
    );
    app.execute(Action::Bold, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    let mut clear = key(Key::Space, false);
    if let egui::Event::Key { modifiers, .. } = &mut clear {
        modifiers.ctrl = true;
    }
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Ime(egui::ImeEvent::Enabled), clear.clone()],
        [1366., 900.],
        1.,
    );
    assert!(app.composing);
    assert!(app.session.editor.typing_style.bold.unwrap_or(false));
    click_region(&ctx, &mut app, "command-ClearFormatting");
    assert!(app.session.editor.typing_style.bold.unwrap_or(false));
    app.focus_document = true;
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Ime(egui::ImeEvent::Commit("字".into())), clear],
        [1366., 900.],
        1.,
    );
    assert!(
        app.session.editor.document.paragraphs[0]
            .styles
            .last()
            .unwrap()
            .bold
            .unwrap_or(false)
    );
}

#[test]
fn styles_gallery_menu_keytips_and_history_use_the_shared_engine() {
    for width in [1366., 1000., 640.] {
        let (ctx, mut app) = setup();
        let size = [width, 900.];
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::Paste(
                "Title\nУкраїна e\u{301}\nРусский".into(),
            )],
            size,
            1.,
        );
        app.session.editor.selection = (0, 5);
        app.session.editor.typing_style = CharacterFormat::default();
        app.sync_view();
        let before = app.session.editor.document.clone();
        let n = app.session.editor.undo_stack.len();
        if width == 1366. {
            click_region_at(&ctx, &mut app, "command-Style(Heading1)", size);
        } else {
            click_region_at(
                &ctx,
                &mut app,
                if width == 640. {
                    "home-overflow"
                } else {
                    "home-menu-Styles"
                },
                size,
            );
            click_region_at(&ctx, &mut app, "command-Style(Heading1)", size);
        }
        assert_eq!(
            app.session.editor.selected_paragraph_style(),
            Some(ParagraphStyle::Heading1)
        );
        assert_eq!(
            app.session.editor.selected_font_size().unwrap().to_string(),
            "18"
        );
        assert!(app.checked(Action::Bold));
        assert_eq!(app.session.editor.selection, (0, 5));
        assert_eq!(app.session.editor.undo_stack.len(), n + 1);
        let after = app.session.editor.document.clone();
        app.execute(Action::Undo, &ctx);
        assert_eq!(app.session.editor.document, before);
        app.execute(Action::Redo, &ctx);
        assert_eq!(app.session.editor.document, after);
        // The temporary ribbon exposes the same menu routes; Escape/selection closes it.
        app.execute(Action::Ribbon, &ctx);
        for k in [
            Key::F10,
            Key::H,
            if width == 640. { Key::E } else { Key::L },
            Key::S,
        ] {
            ribbon_key(&ctx, &mut app, k, Modifiers::NONE, size);
        }
        assert_eq!(
            app.session.editor.selected_paragraph_style(),
            Some(ParagraphStyle::NoSpacing)
        );
        assert!(app.ribbon_collapsed && !app.ribbon_temporary);
        assert!(ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Gallery.bfdoc");
        app.session.save(&path).unwrap();
        assert_eq!(storage::load(&path).unwrap(), app.session.editor.document);
    }
}

#[test]
fn inherited_layout_tracks_real_size_spacing_empty_paragraphs_and_hit_testing() {
    let ctx = egui::Context::default();
    layout::configure_fonts(&ctx);
    let mut e = crate::commands::Editor::default();
    e.replace(0, 0, "Heading\nBody\n\nEnd", "Input").unwrap();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        let render = |e: &crate::commands::Editor| {
            layout::galley(
                ctx,
                &e.document,
                &e.document.text(),
                e.typing_style,
                e.selection.0,
                500.,
                14.6667,
            )
        };
        let normal = render(&e);
        let gap = normal.rows[1].pos.y - normal.rows[0].pos.y - normal.rows[0].row.size.y;
        assert!(
            (gap - 14.6667 * 0.5).abs() < 0.6,
            "Normal paragraph gap: {gap}"
        );
        e.apply_paragraph_style(0, 7, ParagraphStyle::Heading1)
            .unwrap();
        e.apply_paragraph_style(13, 13, ParagraphStyle::Heading1)
            .unwrap();
        let heading = render(&e);
        assert!(heading.rows[0].row.size.y > normal.rows[0].row.size.y * 1.5);
        assert!(heading.rows[2].row.size.y > normal.rows[2].row.size.y * 1.5);
        for i in [0, 3, 7, 8, 11, 12, 13, 14, 16] {
            let pos = heading.pos_from_cursor(CCursor::new(i));
            assert_eq!(heading.cursor_from_pos(pos.center().to_vec2()).index, i);
        }
        e.apply_paragraph_style(0, e.document.character_count(), ParagraphStyle::NoSpacing)
            .unwrap();
        let tight = render(&e);
        assert!(tight.size().y < normal.size().y);
        let gap = tight.rows[1].pos.y - tight.rows[0].pos.y - tight.rows[0].row.size.y;
        assert!(gap.abs() < 0.6, "No Spacing paragraph gap: {gap}");
        assert_eq!(tight.text(), normal.text());
        e.selection = (13, 13);
        e.set_font_size(13, 13, "30".parse().unwrap()).unwrap();
        let pending = render(&e);
        assert!(pending.rows[2].row.size.y > tight.rows[2].row.size.y * 2.);
        assert_eq!(pending.rows[0].row.size.y, tight.rows[0].row.size.y);
    });
}

#[test]
fn style_actions_respect_composition_dialogs_and_command_search() {
    let (ctx, mut app) = setup();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste("Text".into())],
        [1366., 900.],
        1.,
    );
    let before = app.session.editor.document.clone();
    app.composing = true;
    app.execute(Action::Style(ParagraphStyle::Heading1), &ctx);
    assert_eq!(app.session.editor.document, before);
    app.composing = false;
    app.execute(Action::FontDialog, &ctx);
    app.execute(Action::Style(ParagraphStyle::Heading1), &ctx);
    assert_eq!(app.session.editor.document, before);
    app.font_dialog = None;
    ribbon_key(&ctx, &mut app, Key::Q, Modifiers::ALT, [1366., 900.]);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Heading 1".into())],
        [1366., 900.],
        1.,
    );
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Enter, false)],
        [1366., 900.],
        1.,
    );
    assert_eq!(
        app.session.editor.selected_paragraph_style(),
        Some(ParagraphStyle::Heading1)
    );
    assert_eq!(app.session.editor.document.text(), "Text");
}

#[test]
fn modify_style_draft_cancel_invalid_parent_apply_and_history_are_modal() {
    let (ctx, mut app) = setup();
    let size = [1366., 900.];
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste(
            "Normal\nHeading café Україна Русский".into(),
        )],
        size,
        1.,
    );
    app.session
        .editor
        .apply_paragraph_style(7, 7, ParagraphStyle::Heading1)
        .unwrap();
    app.session.editor.selection = (0, 6);
    app.sync_view();
    let before = app.session.editor.document.clone();
    let depth = app.session.editor.undo_stack.len();
    app.execute(Action::ModifyStyle, &ctx);
    frame(&ctx, &mut app, vec![], size, 1.);
    let dialog = app.style_dialog.as_mut().unwrap();
    dialog.definition.parent = Some(ParagraphStyle::Heading1);
    dialog.size = "16".into();
    frame(&ctx, &mut app, vec![key(Key::Enter, false)], size, 1.);
    assert!(app.style_dialog.is_some());
    let mut close_input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366., 900.))),
        ..Default::default()
    };
    close_input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .events
        .push(egui::ViewportEvent::Close);
    let closed = ctx.run(close_input, |ctx| app.update_ui(ctx));
    assert!(
        closed.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .contains(&ViewportCommand::CancelClose)
    );
    assert!(app.style_dialog.is_some());

    assert_eq!(app.session.editor.document, before);
    assert!(!app.enabled(Action::Undo) && !app.enabled(Action::New));
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Z, true), egui::Event::Paste("blocked".into())],
        size,
        1.,
    );
    assert_eq!(app.session.editor.document, before);
    frame(&ctx, &mut app, vec![key(Key::Escape, false)], size, 1.);
    assert!(app.style_dialog.is_none());
    assert_eq!(app.session.editor.undo_stack.len(), depth);
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::ModifyStyle, &ctx);
    let dialog = app.style_dialog.as_mut().unwrap();
    dialog.size = "16".into();
    dialog.definition.character.italic = Some(true);
    dialog.line = "150".into();
    dialog.after = "8.5".into();
    dialog.left = "18".into();
    dialog.right = "12.5".into();
    dialog.first_line = "18.5".into();
    dialog.hanging = true;
    frame(&ctx, &mut app, vec![], size, 1.);
    click_region(&ctx, &mut app, "style-dialog-ok");
    assert!(app.style_dialog.is_none());
    assert_eq!(app.session.editor.undo_stack.len(), depth + 1);
    let after = app.session.editor.document.clone();
    assert_eq!(after.paragraphs, before.paragraphs);
    assert!(
        after
            .style_catalog
            .resolve(ParagraphStyle::Heading1)
            .character
            .italic
    );
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::Redo, &ctx);
    assert_eq!(app.session.editor.document, after);
    assert_eq!(
        storage::decode(&storage::encode(&after).unwrap()).unwrap(),
        after
    );
}

#[test]
fn modify_style_keytips_minimum_bounds_keyboard_and_composition_guards() {
    for dpi in [1., 1.25, 1.5] {
        let (ctx, mut app) = setup();
        let screen = [640. * dpi, 480. * dpi];
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], screen, dpi);
        }
        app.composing = true;
        app.execute(Action::ModifyStyle, &ctx);
        assert!(app.style_dialog.is_none());
        app.composing = false;
        app.pending = Some(Pending::New);
        app.execute(Action::ModifyStyle, &ctx);
        assert!(app.style_dialog.is_none());
        app.pending = None;
        // F10, Home, More, Modify. No selection or paragraph-style application needed.
        for k in [Key::F10, Key::H, Key::E, Key::M] {
            frame(&ctx, &mut app, vec![key(k, false)], screen, dpi);
        }
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], screen, dpi);
        }
        assert!(app.style_dialog.is_some());
        let bounds = app
            .regions
            .iter()
            .find(|r| r.name == "style-dialog")
            .unwrap()
            .rect;
        assert!(
            Rect::from_min_size(Pos2::ZERO, vec2(640., 480.)).contains_rect(bounds),
            "{bounds:?}"
        );
        for region in app
            .regions
            .iter()
            .filter(|r| r.name.starts_with("style-dialog-"))
        {
            assert!(
                bounds.contains_rect(region.rect),
                "{} {:?}",
                region.name,
                region.rect
            );
        }
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::Text("14.5".into())],
            screen,
            dpi,
        );
        frame(&ctx, &mut app, vec![key(Key::Enter, false)], screen, dpi);
        assert!(app.style_dialog.is_none());
        assert_eq!(
            app.session
                .editor
                .document
                .style_catalog
                .resolve(ParagraphStyle::Normal)
                .character
                .font_size
                .to_string(),
            "14.5"
        );
        assert_eq!(app.session.editor.undo_stack.len(), 1);
        app.execute(Action::Undo, &ctx);
        assert!(!app.session.dirty());
    }
}

#[test]
fn modified_catalog_drives_layout_and_selection_without_flattening_direct_values() {
    let (ctx, mut app) = setup();
    app.session
        .editor
        .replace(0, 0, "Body\nHeading", "Input")
        .unwrap();
    app.session
        .editor
        .apply_paragraph_style(5, 5, ParagraphStyle::Heading1)
        .unwrap();
    let doc = app.session.editor.document.clone();
    let before = layout::galley(
        &ctx,
        &doc,
        &doc.text(),
        CharacterFormat::default(),
        0,
        500.,
        15.,
    );
    let mut definition = doc.style_catalog.normal;
    definition.character.font_size = Some("30".parse().unwrap());
    definition.space_after_half_points = Some(60);
    app.session
        .editor
        .modify_style(ParagraphStyle::Normal, definition)
        .unwrap();
    let doc = &app.session.editor.document;
    let after = layout::galley(
        &ctx,
        doc,
        &doc.text(),
        CharacterFormat::default(),
        0,
        500.,
        15.,
    );
    assert!(after.rows[0].size.y > before.rows[0].size.y * 2.);
    assert!(after.rows[1].pos.y > before.rows[1].pos.y + 30.);
    app.session.editor.selection = (0, 4);
    assert_eq!(
        app.session.editor.selected_font_size().unwrap().to_string(),
        "30"
    );
    app.session.editor.clear_character_formatting(0, 4).unwrap();
    assert!(
        app.session.editor.document.paragraphs[0]
            .styles
            .iter()
            .all(|s| *s == CharacterFormat::default())
    );
}

#[test]
fn style_parent_combo_closes_after_selection_and_escape_only_closes_open_combo() {
    let (ctx, mut app) = setup();
    app.open_style_dialog(ParagraphStyle::Heading1, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    click_region(&ctx, &mut app, "style-dialog-parent");
    assert!(egui::Popup::is_any_open(&ctx));
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Escape, false)],
        [1366., 900.],
        1.,
    );
    assert!(!egui::Popup::is_any_open(&ctx));
    assert!(app.style_dialog.is_some());
    click_region(&ctx, &mut app, "style-dialog-parent");
    click_region(&ctx, &mut app, "style-parent-NoSpacing");
    assert!(!egui::Popup::is_any_open(&ctx));
    assert!(app.style_dialog.is_some());
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Enter, false)],
        [1366., 900.],
        1.,
    );
    assert!(app.style_dialog.is_none());
    assert_eq!(
        app.session.editor.document.style_catalog.heading1.parent,
        Some(ParagraphStyle::NoSpacing)
    );
    assert_eq!(app.session.editor.undo_stack.len(), 1);
    app.execute(Action::Undo, &ctx);
    assert!(!app.session.dirty());
}

#[test]
fn create_style_keytips_name_input_parent_choice_cancel_and_one_undo_at_three_scales() {
    for dpi in [1., 1.25, 1.5] {
        let (ctx, mut app) = setup();
        let screen = [640. * dpi, 480. * dpi];
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], screen, dpi);
        }
        app.composing = true;
        app.execute(Action::CreateStyle, &ctx);
        assert!(app.style_dialog.is_none());
        app.composing = false;
        for k in [Key::F10, Key::H, Key::E, Key::C] {
            frame(&ctx, &mut app, vec![key(k, false)], screen, dpi);
        }
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], screen, dpi);
        }
        assert!(app.style_dialog.as_ref().unwrap().creating);
        let bounds = app
            .regions
            .iter()
            .find(|r| r.name == "style-dialog")
            .unwrap()
            .rect;
        assert!(
            Rect::from_min_size(Pos2::ZERO, vec2(640., 480.)).contains_rect(bounds),
            "{bounds:?}"
        );
        for region in app
            .regions
            .iter()
            .filter(|r| r.name.starts_with("style-dialog-"))
        {
            assert!(
                bounds.contains_rect(region.rect),
                "{} {:?}",
                region.name,
                region.rect
            );
        }
        frame(&ctx, &mut app, vec![key(Key::Enter, false)], screen, dpi);
        assert!(app.style_dialog.is_some()); // empty name cannot accept
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::Text("Україна café".into())],
            screen,
            dpi,
        );
        assert_eq!(app.style_dialog.as_ref().unwrap().name, "Україна café");
        assert!(!app.enabled(Action::Undo));
        frame(&ctx, &mut app, vec![key(Key::Enter, false)], screen, dpi);
        assert!(app.style_dialog.is_none());
        assert_eq!(app.session.editor.undo_stack.len(), 1);
        assert_eq!(
            app.session.editor.selected_paragraph_style(),
            Some(ParagraphStyle::Custom(1))
        );
        let saved = app.session.editor.document.clone();
        app.execute(Action::Undo, &ctx);
        assert!(!app.session.dirty());
        app.execute(Action::CreateStyle, &ctx);
        frame(&ctx, &mut app, vec![], screen, dpi);
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::Text("Cancelled".into())],
            screen,
            dpi,
        );
        frame(&ctx, &mut app, vec![key(Key::Escape, false)], screen, dpi);
        assert!(!app.session.dirty());
        assert_eq!(app.session.editor.redo_stack.len(), 1);
        app.execute(Action::Redo, &ctx);
        assert_eq!(app.session.editor.document, saved);
    }
}

#[test]
fn create_style_custom_parent_duplicate_name_preview_and_search_are_real() {
    let (ctx, mut app) = setup();
    let parent = app
        .session
        .editor
        .create_style(
            "Parent",
            crate::model::StyleDefinition {
                parent: Some(ParagraphStyle::Heading1),
                ..Default::default()
            },
            false,
        )
        .unwrap();
    app.sync_view();
    app.execute(Action::CreateStyle, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    app.style_dialog.as_mut().unwrap().name = "parent".into();
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Enter, false)],
        [1366., 900.],
        1.,
    );
    assert!(app.style_dialog.is_some());
    app.style_dialog.as_mut().unwrap().name = "Child".into();
    click_region(&ctx, &mut app, "style-dialog-parent");
    click_region(&ctx, &mut app, "style-parent-Custom(1)");
    assert_eq!(
        app.style_dialog.as_ref().unwrap().definition.parent,
        Some(parent)
    );
    assert!(!egui::Popup::is_any_open(&ctx));
    click_region(&ctx, &mut app, "style-dialog-apply");
    click_region(&ctx, &mut app, "style-dialog-ok");
    assert!(app.style_dialog.is_none());
    assert_eq!(
        app.session.editor.selected_paragraph_style(),
        Some(ParagraphStyle::Normal)
    );
    assert_eq!(app.session.editor.undo_stack.len(), 2);
    app.palette = true;
    app.command_search = "Child".into();
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Enter, false)],
        [1366., 900.],
        1.,
    );
    assert_eq!(
        app.session.editor.selected_paragraph_style(),
        Some(ParagraphStyle::Custom(2))
    );
    assert!(app.session.editor.emphasis_selected(true));
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    assert!(
        app.regions
            .iter()
            .any(|r| r.name == "command-Style(Custom(2))")
    );
    app.execute(Action::ModifyStyle, &ctx);
    assert_eq!(app.style_dialog.as_ref().unwrap().name, "Child");
}

#[test]
fn custom_style_long_names_and_scrollable_menu_stay_bounded_and_keyboard_reachable() {
    let (ctx, mut app) = setup();
    for i in 1..=crate::model::MAX_CUSTOM_STYLES {
        app.session
            .editor
            .create_style(
                &format!("{i:02} {}", "W".repeat(61)),
                crate::model::StyleDefinition::default(),
                true,
            )
            .unwrap();
    }
    assert!(!app.enabled(Action::CreateStyle));
    app.sync_view();
    let size = [640., 480.];
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![], size, 1.);
    }
    for k in [Key::F10, Key::H, Key::E] {
        frame(&ctx, &mut app, vec![key(k, false)], size, 1.);
    }
    assert!(egui::Popup::is_any_open(&ctx));
    // The first Down enters Normal; walk through all built-ins and custom entries.
    for _ in 0..67 {
        ribbon_key(&ctx, &mut app, Key::ArrowDown, Modifiers::NONE, size);
    }
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![], size, 1.);
    }
    let last = app
        .regions
        .iter()
        .find(|r| r.name == "command-Style(Custom(64))")
        .unwrap()
        .rect;
    assert!(
        Rect::from_min_size(Pos2::ZERO, vec2(640., 480.)).contains_rect(last),
        "{last:?}"
    );
    ribbon_key(&ctx, &mut app, Key::Enter, Modifiers::NONE, size);
    assert_eq!(
        app.session.editor.selected_paragraph_style(),
        Some(ParagraphStyle::Custom(64))
    );
    assert!(!egui::Popup::is_any_open(&ctx));
    app.open_style_dialog(ParagraphStyle::Custom(64), &ctx);
    app.style_dialog.as_mut().unwrap().definition.parent = Some(ParagraphStyle::Custom(63));
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![], size, 1.);
    }
    let bounds = app
        .regions
        .iter()
        .find(|r| r.name == "style-dialog")
        .unwrap()
        .rect;
    assert!(
        Rect::from_min_size(Pos2::ZERO, vec2(640., 480.)).contains_rect(bounds),
        "{bounds:?}"
    );
}

#[test]
fn rename_custom_style_selects_name_and_keeps_one_history_step_at_three_scales() {
    for dpi in [1., 1.25, 1.5] {
        let (ctx, mut app) = setup();
        app.session = Session::new(
            storage::decode(include_bytes!("../examples/CustomStyles.bfdoc")).unwrap(),
            None,
        );
        app.sync_view();
        let before = app.session.editor.document.clone();
        let screen = [640. * dpi, 480. * dpi];
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], screen, dpi);
        }
        for k in [Key::F10, Key::H, Key::E, Key::M] {
            frame(&ctx, &mut app, vec![key(k, false)], screen, dpi);
        }
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], screen, dpi);
        }
        assert_eq!(
            app.style_dialog.as_ref().unwrap().id,
            ParagraphStyle::Custom(1)
        );
        let bounds = app
            .regions
            .iter()
            .find(|r| r.name == "style-dialog")
            .unwrap()
            .rect;
        assert!(Rect::from_min_size(Pos2::ZERO, vec2(640., 480.)).contains_rect(bounds));
        assert!(
            app.regions
                .iter()
                .filter(|r| r.name.starts_with("style-dialog-"))
                .all(|r| bounds.contains_rect(r.rect))
        );
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::Text("Новий café".into())],
            screen,
            dpi,
        );
        assert_eq!(app.style_dialog.as_ref().unwrap().name, "Новий café");
        assert_eq!(app.session.editor.document, before);
        frame(&ctx, &mut app, vec![key(Key::Enter, false)], screen, dpi);
        assert!(app.style_dialog.is_none());
        let after = app.session.editor.document.clone();
        assert_eq!(
            after.style_catalog.name(ParagraphStyle::Custom(1)),
            "Новий café"
        );
        assert_eq!(before.paragraphs, after.paragraphs);
        assert_eq!(app.session.editor.undo_stack.len(), 1);
        app.execute(Action::Undo, &ctx);
        assert_eq!(app.session.editor.document, before);
        app.execute(Action::Redo, &ctx);
        assert_eq!(app.session.editor.document, after);
        assert_eq!(
            storage::decode(&storage::encode(&after).unwrap()).unwrap(),
            after
        );
    }
}

#[test]
fn rename_dialog_invalid_cancel_noop_and_combined_changes_are_transactional() {
    let (ctx, mut app) = setup();
    let screen = [1366., 900.];
    app.session = Session::new(
        storage::decode(include_bytes!("../examples/CustomStyles.bfdoc")).unwrap(),
        None,
    );
    app.session
        .editor
        .replace(0, 0, "Undo me", "Input")
        .unwrap();
    app.session.editor.undo();
    app.sync_view();
    let before = app.session.editor.document.clone();
    app.execute(Action::ModifyStyle, &ctx);
    frame(&ctx, &mut app, vec![], screen, 1.);
    frame(&ctx, &mut app, vec![key(Key::Enter, false)], screen, 1.);
    assert!(app.style_dialog.is_none());
    assert_eq!(app.session.editor.redo_stack.len(), 1);
    app.execute(Action::ModifyStyle, &ctx);
    frame(&ctx, &mut app, vec![], screen, 1.);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Normal".into())],
        screen,
        1.,
    );
    frame(&ctx, &mut app, vec![key(Key::Enter, false)], screen, 1.);
    assert!(app.style_dialog.is_some());
    assert_eq!(app.session.editor.document, before);
    frame(&ctx, &mut app, vec![key(Key::Escape, false)], screen, 1.);
    assert_eq!(app.session.editor.redo_stack.len(), 1);
    app.execute(Action::ModifyStyle, &ctx);
    frame(&ctx, &mut app, vec![], screen, 1.);
    let dialog = app.style_dialog.as_mut().unwrap();
    dialog.name = "New title".into();
    dialog.definition.parent = Some(ParagraphStyle::NoSpacing);
    dialog.size = "24".into();
    click_region(&ctx, &mut app, "style-dialog-ok");
    assert!(app.style_dialog.is_none());
    assert_eq!(app.session.editor.undo_stack.len(), 1);
    assert!(app.session.editor.redo_stack.is_empty());
    assert_eq!(
        app.session
            .editor
            .document
            .style_catalog
            .name(ParagraphStyle::Custom(1)),
        "New title"
    );
    assert_eq!(
        app.session
            .editor
            .document
            .style_catalog
            .get(ParagraphStyle::Custom(1))
            .unwrap()
            .parent,
        Some(ParagraphStyle::NoSpacing)
    );
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, before);
    // Built-in names remain fixed and keep the previous size-field focus behavior.
    app.open_style_dialog(ParagraphStyle::Normal, &ctx);
    frame(&ctx, &mut app, vec![], screen, 1.);
    assert!(!app.regions.iter().any(|r| r.name == "style-dialog-name"));
}

#[test]
fn renamed_style_search_and_gallery_use_current_names_and_preserve_layout() {
    let (ctx, mut app) = setup();
    app.session = Session::new(
        storage::decode(include_bytes!("../examples/CustomStyles.bfdoc")).unwrap(),
        None,
    );
    let screen = [1366., 900.];
    app.sync_view();
    let old = app.session.editor.document.clone();
    frame(&ctx, &mut app, vec![], screen, 1.);
    click_region(&ctx, &mut app, "command-search");
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Modify style".into())],
        screen,
        1.,
    );
    frame(&ctx, &mut app, vec![key(Key::Enter, false)], screen, 1.);
    assert!(app.style_dialog.is_some());
    app.style_dialog.as_mut().unwrap().name = "Renamed heading".into();
    frame(&ctx, &mut app, vec![], screen, 1.);
    click_region(&ctx, &mut app, "style-dialog-ok");
    let doc = &app.session.editor.document;
    let mut expected = old.clone();
    expected.style_catalog.custom[0].name = "Renamed heading".into();
    assert_eq!(doc, &expected);
    let layout_before = layout::galley(
        &ctx,
        &old,
        &old.text(),
        CharacterFormat::default(),
        0,
        500.,
        15.,
    );
    let layout_after = layout::galley(
        &ctx,
        doc,
        &doc.text(),
        CharacterFormat::default(),
        0,
        500.,
        15.,
    );
    assert_eq!(layout_before.size(), layout_after.size());
    assert_eq!(
        layout_before.rows.iter().map(|r| r.pos).collect::<Vec<_>>(),
        layout_after.rows.iter().map(|r| r.pos).collect::<Vec<_>>()
    );
    assert_eq!(
        app.command_name(Action::Style(ParagraphStyle::Custom(1))),
        "Renamed heading"
    );
    app.execute(Action::Style(ParagraphStyle::Normal), &ctx);
    let depth = app.session.editor.undo_stack.len();
    frame(&ctx, &mut app, vec![], screen, 1.);
    click_region(&ctx, &mut app, "command-search");
    app.palette = true;
    app.command_search = "Report Heading".into();
    frame(&ctx, &mut app, vec![key(Key::Enter, false)], screen, 1.);
    assert_eq!(app.session.editor.undo_stack.len(), depth);
    app.command_search = "Renamed heading".into();
    frame(&ctx, &mut app, vec![key(Key::Enter, false)], screen, 1.);
    assert_eq!(
        app.session.editor.selected_paragraph_style(),
        Some(ParagraphStyle::Custom(1))
    );
    frame(&ctx, &mut app, vec![], screen, 1.);
    assert!(
        app.regions
            .iter()
            .any(|r| r.name == "command-Style(Custom(1))")
    );
    app.execute(Action::Undo, &ctx); // apply
    app.execute(Action::Undo, &ctx); // Normal
    app.execute(Action::Undo, &ctx); // rename
    assert_eq!(
        app.command_name(Action::Style(ParagraphStyle::Custom(1))),
        "Report Heading"
    );
    assert_eq!(app.session.editor.document, old);
}

#[test]
fn unused_style_list_context_menu_renames_without_applying_to_document() {
    let (ctx, mut app) = setup();
    let id = app
        .session
        .editor
        .create_style("Unused", crate::model::StyleDefinition::default(), false)
        .unwrap();
    let before = app.session.editor.document.clone();
    let depth = app.session.editor.undo_stack.len();
    let screen = [1366., 900.];
    app.sync_view();
    frame(&ctx, &mut app, vec![], screen, 1.);
    // Filter the target into view; usage captions make each scroll-list entry taller.
    app.style_filter = "Unused".into();
    click_region(&ctx, &mut app, "home-menu-Styles");
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![], screen, 1.);
    }
    let pos = app
        .regions
        .iter()
        .find(|r| r.name == "command-Style(Custom(1))")
        .unwrap()
        .rect
        .center();
    for pressed in [true, false] {
        frame(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Secondary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
            screen,
            1.,
        );
    }
    for _ in 0..2 {
        frame(&ctx, &mut app, vec![], screen, 1.);
    }
    click_region(&ctx, &mut app, "style-modify-Custom(1)");
    assert_eq!(app.style_dialog.as_ref().unwrap().id, id);
    assert!(!egui::Popup::is_any_open(&ctx));
    frame(&ctx, &mut app, vec![], screen, 1.);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Unused renamed".into())],
        screen,
        1.,
    );
    frame(&ctx, &mut app, vec![key(Key::Enter, false)], screen, 1.);
    assert_eq!(
        app.session.editor.document.style_catalog.name(id),
        "Unused renamed"
    );
    assert_eq!(app.session.editor.document.paragraphs, before.paragraphs);
    assert_eq!(app.session.editor.undo_stack.len(), depth + 1);
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, before);
}

fn delete_style_setup() -> (egui::Context, DocsApp) {
    let (ctx, mut app) = setup();
    let mut doc = storage::decode(include_bytes!("../examples/CustomStyles.bfdoc")).unwrap();
    doc.style_catalog.custom[1].definition.parent = Some(ParagraphStyle::Custom(1));
    app.session = Session::new(doc, None);
    app.sync_view();
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    (ctx, app)
}

#[test]
fn delete_style_widgets_replace_distinct_references_and_undo_once() {
    let (ctx, mut app) = delete_style_setup();
    let before = app.session.editor.document.clone();
    app.execute(Action::DeleteStyle, &ctx);
    click_region(&ctx, &mut app, "delete-style-paragraphs");
    click_region(&ctx, &mut app, "delete-style-paragraphs-NoSpacing");
    click_region(&ctx, &mut app, "delete-style-children");
    click_region(&ctx, &mut app, "delete-style-children-Heading1");
    assert_eq!(app.session.editor.document, before);
    assert!(!egui::Popup::is_any_open(&ctx));
    click_region(&ctx, &mut app, "delete-style-apply");
    assert!(app.delete_style_dialog.is_none());
    let after = app.session.editor.document.clone();
    assert_eq!(
        after.paragraphs[0].paragraph_style,
        ParagraphStyle::NoSpacing
    );
    assert_eq!(after.style_catalog.custom[0].id, 2);
    assert_eq!(
        after.style_catalog.custom[0].definition.parent,
        Some(ParagraphStyle::Heading1)
    );
    assert_eq!(app.session.editor.undo_stack.len(), 1);
    assert_eq!(
        app.session.editor.selected_paragraph_style(),
        Some(ParagraphStyle::NoSpacing)
    );
    assert_eq!(
        storage::decode(&storage::encode(&after).unwrap()).unwrap(),
        after
    );
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::Redo, &ctx);
    assert_eq!(app.session.editor.document, after);
}

#[test]
fn delete_style_cycle_cancel_and_modal_guards_preserve_redo() {
    let (ctx, mut app) = delete_style_setup();
    let screen = [1366., 900.];
    app.session.editor.replace(0, 0, "x", "Input").unwrap();
    app.session.editor.undo();
    let before = app.session.editor.document.clone();
    app.execute(Action::DeleteStyle, &ctx);
    click_region(&ctx, &mut app, "delete-style-children");
    click_region(&ctx, &mut app, "delete-style-children-Custom(2)");
    click_region(&ctx, &mut app, "delete-style-apply"); // invalid self-parent remains a draft
    assert!(app.delete_style_dialog.is_some());
    for action in [
        Action::New,
        Action::Save,
        Action::Undo,
        Action::CreateStyle,
        Action::ModifyStyle,
        Action::DeleteStyle,
    ] {
        assert!(!app.enabled(action));
        app.execute(action, &ctx);
    }
    frame(
        &ctx,
        &mut app,
        vec![
            key(Key::S, true),
            key(Key::N, true),
            egui::Event::Text("blocked".into()),
        ],
        screen,
        1.,
    );
    let mut input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366., 900.))),
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .events
        .push(egui::ViewportEvent::Close);
    let output = ctx.run(input, |ctx| app.update_ui(ctx));
    assert!(
        output.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .contains(&ViewportCommand::CancelClose)
    );
    assert!(app.pending.is_none());
    click_region(&ctx, &mut app, "delete-style-children");
    frame(&ctx, &mut app, vec![key(Key::Escape, false)], screen, 1.);
    assert!(app.delete_style_dialog.is_some());
    frame(&ctx, &mut app, vec![key(Key::Escape, false)], screen, 1.);
    assert!(app.delete_style_dialog.is_none());
    assert_eq!(app.session.editor.document, before);
    assert_eq!(app.session.editor.redo_stack.len(), 1);
    app.execute(Action::Redo, &ctx);
    assert!(app.session.editor.document.text().starts_with('x'));
}

#[test]
fn delete_style_keytips_long_name_bounds_cancel_and_keyboard_commit_at_three_scales() {
    for dpi in [1., 1.25, 1.5] {
        let (ctx, mut app) = delete_style_setup();
        app.session.editor.document.style_catalog.custom[0].name = "Ї".repeat(64);
        let before = app.session.editor.document.clone();
        let screen = [640. * dpi, 480. * dpi];
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], screen, dpi);
        }
        for keypress in [Key::F10, Key::H, Key::E, Key::D] {
            frame(&ctx, &mut app, vec![key(keypress, false)], screen, dpi);
        }
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], screen, dpi);
        }
        assert!(app.delete_style_dialog.is_some());
        let bounds = app
            .regions
            .iter()
            .find(|r| r.name == "delete-style-dialog")
            .unwrap()
            .rect;
        assert!(
            Rect::from_min_size(Pos2::ZERO, vec2(640., 480.)).contains_rect(bounds),
            "{bounds:?}"
        );
        for r in app
            .regions
            .iter()
            .filter(|r| r.name.starts_with("delete-style-"))
        {
            assert!(bounds.contains_rect(r.rect), "{} {:?}", r.name, r.rect);
        }
        // Cancel is initially focused: the opening Enter must not delete anything.
        frame(&ctx, &mut app, vec![key(Key::Enter, false)], screen, dpi);
        assert!(app.delete_style_dialog.is_none());
        assert_eq!(app.session.editor.document, before);
        app.execute(Action::DeleteStyle, &ctx);
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], screen, dpi);
        }
        // Cancel precedes Delete in the modal focus order; Tab selects Delete.
        frame(&ctx, &mut app, vec![key(Key::Tab, false)], screen, dpi);
        frame(&ctx, &mut app, vec![key(Key::Enter, false)], screen, dpi);
        assert!(app.delete_style_dialog.is_none());
        assert_eq!(app.session.editor.document.style_catalog.custom.len(), 1);
        app.execute(Action::Undo, &ctx);
        assert_eq!(app.session.editor.document, before);
    }
}

#[test]
fn delete_unused_style_context_target_search_and_composition_guards() {
    let (ctx, mut app) = setup();
    let screen = [1366., 900.];
    assert!(!app.enabled(Action::DeleteStyle));
    app.execute(Action::DeleteStyle, &ctx);
    assert!(app.delete_style_dialog.is_none());
    let id = app
        .session
        .editor
        .create_style("Unused", crate::model::StyleDefinition::default(), false)
        .unwrap();
    let before = app.session.editor.document.clone();
    app.composing = true;
    app.open_delete_style_dialog(id, &ctx);
    assert!(app.delete_style_dialog.is_none());
    app.composing = false;
    app.pending = Some(Pending::New);
    app.open_delete_style_dialog(id, &ctx);
    assert!(app.delete_style_dialog.is_none());
    app.pending = None;
    // Filter the target into view; usage captions make each scroll-list entry taller.
    app.style_filter = "Unused".into();
    click_region(&ctx, &mut app, "home-menu-Styles");
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![], screen, 1.);
    }
    let pos = app
        .regions
        .iter()
        .find(|r| r.name == "command-Style(Custom(1))")
        .unwrap()
        .rect
        .center();
    for pressed in [true, false] {
        frame(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Secondary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
            screen,
            1.,
        );
    }
    click_region(&ctx, &mut app, "style-delete-Custom(1)");
    assert_eq!(app.delete_style_dialog.as_ref().unwrap().id, id);
    click_region(&ctx, &mut app, "delete-style-apply");
    assert!(app.session.editor.document.style_catalog.custom.is_empty());
    assert_eq!(app.session.editor.document.paragraphs, before.paragraphs);
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::Style(id), &ctx);
    frame(&ctx, &mut app, vec![], screen, 1.);
    click_region(&ctx, &mut app, "command-search");
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Delete style".into())],
        screen,
        1.,
    );
    frame(&ctx, &mut app, vec![key(Key::Enter, false)], screen, 1.);
    assert!(app.delete_style_dialog.is_some());
    frame(&ctx, &mut app, vec![key(Key::Escape, false)], screen, 1.);
    assert!(app.delete_style_dialog.is_none());
}

#[test]
fn following_style_selector_enter_keyboard_batches_and_paste_history() {
    let (ctx, mut app) = setup();
    app.open_style_dialog(ParagraphStyle::Normal, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    click_region(&ctx, &mut app, "style-dialog-next");
    click_region(&ctx, &mut app, "style-next-Heading1");
    click_region(&ctx, &mut app, "style-dialog-ok");
    assert_eq!(
        app.session.editor.document.style_catalog.normal.next_style,
        Some(ParagraphStyle::Heading1)
    );
    app.execute(Action::Undo, &ctx);
    assert_eq!(
        app.session.editor.document.style_catalog.normal.next_style,
        None
    );
    app.execute(Action::Redo, &ctx);
    app.focus_document = true;
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    let before = app.session.editor.document.clone();
    let depth = app.session.editor.undo_stack.len();
    frame(
        &ctx,
        &mut app,
        vec![
            egui::Event::Text("Україна café".into()),
            key(Key::Enter, false),
        ],
        [1366., 900.],
        1.,
    );
    assert_eq!(app.session.editor.document.paragraphs.len(), 2);
    assert_eq!(
        app.session.editor.document.paragraphs[1].paragraph_style,
        ParagraphStyle::Heading1
    );
    assert_eq!(app.session.editor.undo_stack.len(), depth + 1);
    let after = app.session.editor.document.clone();
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::Redo, &ctx);
    assert_eq!(app.session.editor.document, after);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste("x\ny".into())],
        [1366., 900.],
        1.,
    );
    assert_eq!(
        app.session
            .editor
            .document
            .paragraphs
            .last()
            .unwrap()
            .paragraph_style,
        ParagraphStyle::Heading1
    );
    assert_eq!(
        storage::decode(&storage::encode(&app.session.editor.document).unwrap()).unwrap(),
        app.session.editor.document
    );
}

#[test]
fn following_style_draft_cancel_and_unknown_reference_preserve_redo() {
    let (ctx, mut app) = setup();
    app.session.editor.replace(0, 0, "abc", "Input").unwrap();
    app.session.editor.undo();
    app.sync_view();
    let before = app.session.editor.document.clone();
    app.open_style_dialog(ParagraphStyle::Normal, &ctx);
    app.style_dialog.as_mut().unwrap().definition.next_style = Some(ParagraphStyle::Custom(999));
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    click_region(&ctx, &mut app, "style-dialog-ok");
    assert!(app.style_dialog.is_some());
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Escape, false)],
        [1366., 900.],
        1.,
    );
    assert!(app.style_dialog.is_none());
    assert_eq!(app.session.editor.document, before);
    assert_eq!(app.session.editor.redo_stack.len(), 1);
}

#[test]
fn enter_before_existing_separator_uses_caret_not_ambiguous_diff() {
    let (ctx, mut app) = setup();
    app.session
        .editor
        .replace(0, 0, "Heading\nBody", "Input")
        .unwrap();
    app.session
        .editor
        .apply_paragraph_style(0, 0, ParagraphStyle::Heading1)
        .unwrap();
    let mut d = *app
        .session
        .editor
        .document
        .style_catalog
        .get(ParagraphStyle::Heading1)
        .unwrap();
    d.next_style = Some(ParagraphStyle::NoSpacing);
    app.session
        .editor
        .modify_style(ParagraphStyle::Heading1, d)
        .unwrap();
    for selection in [(7, 7), (7, 8)] {
        app.session.editor.selection = selection;
        app.sync_view();
        app.focus_document = true;
        frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
        let before = app.session.editor.document.clone();
        frame(
            &ctx,
            &mut app,
            vec![key(Key::Enter, false)],
            [1366., 900.],
            1.,
        );
        assert_eq!(
            app.session.editor.document.paragraphs[1].paragraph_style,
            ParagraphStyle::NoSpacing
        );
        let after = app.session.editor.document.clone();
        app.execute(Action::Undo, &ctx);
        assert_eq!(app.session.editor.document, before);
        assert_eq!(app.session.editor.selection, selection);
        app.execute(Action::Redo, &ctx);
        assert_eq!(app.session.editor.document, after);
        app.execute(Action::Undo, &ctx);
    }
}

#[test]
fn character_styles_keytips_apply_remove_and_undo_at_wide_and_narrow_widths() {
    for width in [640., 1366.] {
        let (ctx, mut app) = setup();
        app.session
            .editor
            .replace(0, 0, "English Україна e\u{301} Русский", "Input")
            .unwrap();
        app.sync_view();
        app.execute(Action::SelectAll, &ctx);
        let before = app.session.editor.document.clone();
        let n = app.session.editor.undo_stack.len();
        let menu = if width < 900. { Key::E } else { Key::L };
        tip_sequence(
            &ctx,
            &mut app,
            &[Key::F10, Key::H, menu, Key::I],
            [width, 900.],
        );
        assert_eq!(
            app.session.editor.selected_character_style(),
            Some(Some(CharacterStyle::Emphasis))
        );
        assert_eq!(app.session.editor.undo_stack.len(), n + 1);
        assert!(app.checked(Action::Italic));
        let after = app.session.editor.document.clone();
        app.execute(Action::Undo, &ctx);
        assert_eq!(app.session.editor.document, before);
        app.execute(Action::Redo, &ctx);
        assert_eq!(app.session.editor.document, after);
        tip_sequence(
            &ctx,
            &mut app,
            &[Key::F10, Key::H, menu, Key::Z],
            [width, 900.],
        );
        assert_eq!(app.session.editor.document, before);
        app.execute(Action::Undo, &ctx);
        assert_eq!(app.session.editor.document, after);
        assert_eq!(
            storage::decode(&storage::encode(&after).unwrap()).unwrap(),
            after
        );
    }
}

#[test]
fn character_styles_respect_ime_guard_and_pending_input_history() {
    let (ctx, mut app) = setup();
    app.composing = true;
    app.execute(Action::CharacterStyle(Some(CharacterStyle::Strong)), &ctx);
    assert!(app.session.editor.undo_stack.is_empty());
    app.composing = false;
    app.execute(Action::CharacterStyle(Some(CharacterStyle::Strong)), &ctx);
    assert!(!app.session.dirty());
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("a".into())],
        [1366., 900.],
        1.,
    );
    assert_eq!(
        app.session.editor.document.paragraphs[0].styles[0].character_style,
        Some(CharacterStyle::Strong)
    );
    app.execute(Action::Undo, &ctx);
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.typing_style.character_style, None);
    app.execute(Action::Redo, &ctx);
    app.execute(Action::Redo, &ctx);
    assert_eq!(app.session.editor.document.text(), "a");
}

#[test]
fn styles_scrolled_rows_do_not_paint_keytips_over_other_commands() {
    for width in [640., 1366.] {
        let (ctx, mut app) = setup();
        let size = [width, 480.];
        let menu = if width < 900. { Key::E } else { Key::L };
        tip_sequence(&ctx, &mut app, &[Key::F10, Key::H, menu], size);
        for _ in 0..4 {
            frame(&ctx, &mut app, vec![], size, 1.);
        }
        let tips: Vec<_> = app
            .regions
            .iter()
            .filter(|r| r.name.starts_with("keytip-"))
            .collect();
        for (i, tip) in tips.iter().enumerate() {
            assert!(Rect::from_min_size(Pos2::ZERO, vec2(width, 480.)).contains_rect(tip.rect));
            for other in &tips[i + 1..] {
                assert!(
                    !tip.rect.intersects(other.rect),
                    "{} / {}",
                    tip.name,
                    other.name
                );
            }
        }
        // Emphasis starts below the clipped list, but keyboard navigation still finds it.
        assert!(!app.regions.iter().any(|r| r.name == "keytip-I"));
        let index = app
            .ribbon_menu_targets
            .iter()
            .position(|t| {
                matches!(
                    t.kind,
                    TargetKind::Action(Action::CharacterStyle(Some(CharacterStyle::Emphasis)))
                )
            })
            .unwrap();
        for _ in 0..index {
            ribbon_key(&ctx, &mut app, Key::ArrowDown, Modifiers::NONE, size);
        }
        for _ in 0..4 {
            frame(&ctx, &mut app, vec![], size, 1.);
        }
        let row = app
            .regions
            .iter()
            .find(|r| r.name == "command-CharacterStyle(Some(Emphasis))")
            .unwrap();
        assert!(Rect::from_min_size(Pos2::ZERO, vec2(width, 480.)).contains_rect(row.rect));
        ribbon_key(&ctx, &mut app, Key::Enter, Modifiers::NONE, size);
        assert_eq!(
            app.session.editor.typing_style.character_style,
            Some(CharacterStyle::Emphasis)
        );
    }
}

#[test]
fn create_character_style_draft_applies_named_properties_and_one_undo() {
    let (ctx, mut app) = setup();
    app.session
        .editor
        .replace(0, 0, "Україна e\u{301} English", "Input")
        .unwrap();
    app.sync_view();
    app.execute(Action::SelectAll, &ctx);
    let before = app.session.editor.document.clone();
    let n = app.session.editor.undo_stack.len();
    tip_sequence(
        &ctx,
        &mut app,
        &[Key::F10, Key::H, Key::L, Key::K],
        [1366., 900.],
    );
    assert!(app.style_dialog.as_ref().unwrap().character);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Annotation".into())],
        [1366., 900.],
        1.,
    );
    let draft = app.style_dialog.as_mut().unwrap();
    draft.definition.character.italic = Some(true);
    draft.size = "24".into();
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    click_region(&ctx, &mut app, "style-dialog-ok");
    assert!(app.style_dialog.is_none());
    assert_eq!(app.session.editor.undo_stack.len(), n + 1);
    assert_eq!(
        app.session.editor.document.character_styles[0].name,
        "Annotation"
    );
    assert!(app.checked(Action::Italic));
    assert_eq!(
        app.session.editor.selected_font_size().unwrap().to_string(),
        "24"
    );
    assert_eq!(
        app.command_name(Action::CharacterStyle(Some(CharacterStyle::Custom(1)))),
        "Annotation"
    );
    let after = app.session.editor.document.clone();
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::Redo, &ctx);
    assert_eq!(app.session.editor.document, after);
    assert_eq!(
        storage::decode(&storage::encode(&after).unwrap()).unwrap(),
        after
    );
}

#[test]
fn character_creation_invalid_cancel_and_modal_guards_preserve_redo() {
    let (ctx, mut app) = setup();
    app.session.editor.replace(0, 0, "x", "Input").unwrap();
    app.session.editor.undo();
    app.sync_view();
    let before = app.session.editor.document.clone();
    app.execute(Action::CreateCharacterStyle, &ctx);
    frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    app.style_dialog.as_mut().unwrap().name = "Strong".into();
    frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    click_region_at(&ctx, &mut app, "style-dialog-ok", [640., 480.]);
    assert!(app.style_dialog.is_some());
    app.execute(Action::New, &ctx);
    assert_eq!(app.session.editor.document, before);
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Escape, false)],
        [640., 480.],
        1.,
    );
    assert!(app.style_dialog.is_none());
    assert_eq!(app.session.editor.redo_stack.len(), 1);
    app.composing = true;
    app.execute(Action::CreateCharacterStyle, &ctx);
    assert!(app.style_dialog.is_none());
}

#[test]
fn character_creation_dialog_bounds_at_minimum_width_and_scales() {
    for scale in [1., 1.25, 1.5] {
        let (ctx, mut app) = setup();
        app.execute(Action::CreateCharacterStyle, &ctx);
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], [640. * scale, 480. * scale], scale);
        }
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(640., 480.));
        for r in app
            .regions
            .iter()
            .filter(|r| r.name.starts_with("style-dialog"))
        {
            assert!(screen.contains_rect(r.rect), "{} {:?}", r.name, r.rect);
        }
        assert!(
            !app.regions
                .iter()
                .any(|r| r.name == "style-dialog-parent" || r.name == "style-dialog-next")
        );
    }
}

#[test]
fn custom_character_style_changes_actual_layout_and_named_search_action() {
    let (ctx, mut app) = setup();
    app.session.editor.replace(0, 0, "Named", "Input").unwrap();
    app.session.editor.selection = (0, 5);
    let id = app
        .session
        .editor
        .create_character_style(
            "Annotation",
            CharacterFormat {
                italic: Some(true),
                font_size: Some("24".parse().unwrap()),
                ..Default::default()
            },
            true,
        )
        .unwrap();
    app.sync_view();
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    let d = &app.session.editor.document;
    let galley = layout::galley(
        &ctx,
        d,
        &d.text(),
        app.session.editor.typing_style,
        0,
        500.,
        11.,
    );
    assert!(galley.job.sections[0].format.italics);
    assert_eq!(galley.job.sections[0].format.font_id.size, 24.);
    app.command_search = "Annotation".into();
    app.palette = true;
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    assert_eq!(
        app.command_name(Action::CharacterStyle(Some(id))),
        "Annotation"
    );
    app.palette = false;
    app.execute(Action::CharacterStyle(None), &ctx);
    app.execute(Action::CharacterStyle(Some(id)), &ctx);
    assert!(app.checked(Action::CharacterStyle(Some(id))));
}

#[test]
fn modify_character_dialog_commit_cancel_validation_and_layout() {
    let (ctx, mut app) = setup();
    app.session.editor.replace(0, 0, "Text", "Input").unwrap();
    app.session.editor.selection = (0, 4);
    let id = app
        .session
        .editor
        .create_character_style("Note", CharacterFormat::default(), true)
        .unwrap();
    app.sync_view();
    let before = app.session.editor.document.clone();
    let n = app.session.editor.undo_stack.len();
    app.execute(Action::ModifyCharacterStyle, &ctx);
    frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    app.style_dialog.as_mut().unwrap().size = "bad".into();
    click_region_at(&ctx, &mut app, "style-dialog-ok", [640., 480.]);
    assert!(app.style_dialog.is_some());
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Escape, false)],
        [640., 480.],
        1.,
    );
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::ModifyCharacterStyle, &ctx);
    let draft = app.style_dialog.as_mut().unwrap();
    assert_eq!(draft.character_id, Some(id));
    draft.size = "24".into();
    draft.definition.character.italic = Some(true);
    frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    click_region_at(&ctx, &mut app, "style-dialog-ok", [640., 480.]);
    assert!(app.style_dialog.is_none());
    assert_eq!(app.session.editor.undo_stack.len(), n + 1);
    assert_eq!(app.session.editor.document.paragraphs, before.paragraphs);
    let d = &app.session.editor.document;
    let galley = layout::galley(
        &ctx,
        d,
        &d.text(),
        app.session.editor.typing_style,
        0,
        500.,
        11.,
    );
    assert!(galley.job.sections[0].format.italics);
    assert_eq!(galley.job.sections[0].format.font_id.size, 24.);
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::Redo, &ctx);
    assert_eq!(
        app.session.editor.document.character_styles[0]
            .format
            .font_size
            .unwrap()
            .to_string(),
        "24"
    );
}

#[test]
fn modify_character_availability_unused_definition_and_bounds() {
    for scale in [1., 1.25, 1.5] {
        let (ctx, mut app) = setup();
        assert!(!app.enabled(Action::ModifyCharacterStyle));
        let id = app
            .session
            .editor
            .create_character_style("Unused", CharacterFormat::default(), false)
            .unwrap();
        assert!(!app.enabled(Action::ModifyCharacterStyle));
        app.open_character_style_dialog(id, &ctx);
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], [640. * scale, 480. * scale], scale);
        }
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(640., 480.));
        for r in app
            .regions
            .iter()
            .filter(|r| r.name.starts_with("style-dialog"))
        {
            assert!(screen.contains_rect(r.rect), "{} {:?}", r.name, r.rect);
        }
        assert!(
            !app.regions
                .iter()
                .any(|r| r.name == "style-dialog-parent" || r.name == "style-dialog-apply")
        );
        app.execute(Action::New, &ctx);
        assert!(app.style_dialog.is_some());
    }
}

#[test]
fn rename_character_draft_validation_cancel_and_one_commit() {
    let (ctx, mut app) = setup();
    let id = app
        .session
        .editor
        .create_character_style("Note", CharacterFormat::default(), true)
        .unwrap();
    app.sync_view();
    let before = app.session.editor.document.clone();
    app.execute(Action::ModifyCharacterStyle, &ctx);
    frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    assert!(app.regions.iter().any(|r| r.name == "style-dialog-name"));
    app.style_dialog.as_mut().unwrap().name = "Strong".into();
    frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    click_region_at(&ctx, &mut app, "style-dialog-ok", [640., 480.]);
    assert!(app.style_dialog.is_some());
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Escape, false)],
        [640., 480.],
        1.,
    );
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::ModifyCharacterStyle, &ctx);
    let draft = app.style_dialog.as_mut().unwrap();
    draft.name = "Примітка e\u{301}".into();
    draft.size = "18".into();
    frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    click_region_at(&ctx, &mut app, "style-dialog-ok", [640., 480.]);
    assert!(app.style_dialog.is_none());
    assert_eq!(
        app.command_name(Action::CharacterStyle(Some(id))),
        "Примітка e\u{301}"
    );
    let after = app.session.editor.document.clone();
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::Redo, &ctx);
    assert_eq!(app.session.editor.document, after);
}

#[test]
fn delete_character_dialog_cancel_replacement_and_exact_history() {
    let (ctx, mut app) = setup();
    app.session.editor.replace(0, 0, "Text", "Input").unwrap();
    app.session.editor.selection = (0, 4);
    let id = app
        .session
        .editor
        .create_character_style("Note", CharacterFormat::default(), true)
        .unwrap();
    app.sync_view();
    let before = app.session.editor.document.clone();
    let n = app.session.editor.undo_stack.len();
    app.execute(Action::DeleteCharacterStyle, &ctx);
    frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    app.execute(Action::New, &ctx);
    assert!(app.delete_style_dialog.is_some());
    frame(
        &ctx,
        &mut app,
        vec![key(Key::Escape, false)],
        [640., 480.],
        1.,
    );
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::DeleteCharacterStyle, &ctx);
    app.delete_style_dialog
        .as_mut()
        .unwrap()
        .character_replacement = Some(CharacterStyle::Emphasis);
    frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    click_region_at(&ctx, &mut app, "delete-style-apply", [640., 480.]);
    assert!(app.delete_style_dialog.is_none());
    assert!(!app.session.editor.document.character_style_exists(id));
    assert_eq!(app.session.editor.undo_stack.len(), n + 1);
    assert_eq!(
        app.session.editor.selected_character_style(),
        Some(Some(CharacterStyle::Emphasis))
    );
    let after = app.session.editor.document.clone();
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::Redo, &ctx);
    assert_eq!(app.session.editor.document, after);
}

#[test]
fn delete_character_unused_bounds_and_guards() {
    for scale in [1., 1.25, 1.5] {
        let (ctx, mut app) = setup();
        assert!(!app.enabled(Action::DeleteCharacterStyle));
        let id = app
            .session
            .editor
            .create_character_style("Unused", CharacterFormat::default(), false)
            .unwrap();
        app.composing = true;
        app.open_delete_character_style(id, &ctx);
        assert!(app.delete_style_dialog.is_none());
        app.composing = false;
        app.open_delete_character_style(id, &ctx);
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], [640. * scale, 480. * scale], scale);
        }
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(640., 480.));
        for r in app.regions.iter().filter(|r| r.name.starts_with("delete-")) {
            assert!(screen.contains_rect(r.rect), "{} {:?}", r.name, r.rect);
        }
        assert_eq!(
            app.delete_style_dialog.as_ref().unwrap().character_id,
            Some(id)
        );
        click_region_at(&ctx, &mut app, "delete-style-cancel", [640., 480.]);
        assert!(app.session.editor.document.character_style_exists(id));
    }
}

#[test]
fn character_parent_dialog_validation_preview_and_history() {
    let (ctx, mut app) = setup();
    app.session.editor.replace(0, 0, "Text", "Input").unwrap();
    app.session.editor.selection = (0, 4);
    let parent = app
        .session
        .editor
        .create_character_style(
            "Parent",
            CharacterFormat {
                italic: Some(true),
                ..Default::default()
            },
            false,
        )
        .unwrap();
    let child = app
        .session
        .editor
        .create_character_style("Child", CharacterFormat::default(), true)
        .unwrap();
    app.sync_view();
    let before = app.session.editor.document.clone();
    app.execute(Action::ModifyCharacterStyle, &ctx);
    app.style_dialog.as_mut().unwrap().character_parent = Some(child);
    frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    click_region_at(&ctx, &mut app, "style-dialog-ok", [640., 480.]);
    assert!(app.style_dialog.is_some());
    app.style_dialog.as_mut().unwrap().character_parent = Some(parent);
    frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    assert!(
        app.regions
            .iter()
            .any(|r| r.name == "style-dialog-character-parent")
    );
    click_region_at(&ctx, &mut app, "style-dialog-ok", [640., 480.]);
    assert!(app.style_dialog.is_none());
    let d = &app.session.editor.document;
    let galley = layout::galley(
        &ctx,
        d,
        &d.text(),
        app.session.editor.typing_style,
        0,
        500.,
        11.,
    );
    assert!(galley.job.sections[0].format.italics);
    let after = app.session.editor.document.clone();
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::Redo, &ctx);
    assert_eq!(app.session.editor.document, after);
}

#[test]
fn formatting_provenance_pane_is_read_only_at_narrow_sizes() {
    for scale in [1., 1.25, 1.5] {
        let (ctx, mut app) = setup();
        app.formatting = true;
        let before = app.session.editor.document.clone();
        frame(&ctx, &mut app, vec![], [960., 720.], scale);
        assert_eq!(app.session.editor.document, before);
        assert_eq!(
            crate::formatting::selection(&app.session.editor)[0].source,
            "Document default"
        );
    }
}

#[test]
fn paragraph_provenance_pane_updates_without_history_changes() {
    for scale in [1., 1.25, 1.5] {
        let (ctx, mut app) = setup();
        app.formatting = true;
        app.session.editor.align(0, 0, Alignment::Center).unwrap();
        let before = app.session.editor.document.clone();
        frame(&ctx, &mut app, vec![], [960., 720.], scale);
        assert_eq!(app.session.editor.document, before);
        assert_eq!(
            crate::formatting::paragraph_selection(&app.session.editor)[0].value,
            "Center"
        );
        app.session.editor.undo();
        frame(&ctx, &mut app, vec![], [960., 720.], scale);
        assert_eq!(
            crate::formatting::paragraph_selection(&app.session.editor)[0].value,
            "Left"
        );
    }
}

#[test]
fn style_filter_keyboard_preserves_document_and_applies_visible_result() {
    let (ctx, mut app) = setup();
    let size = [640., 480.];
    frame(&ctx, &mut app, vec![], size, 1.);
    for k in [Key::F10, Key::H, Key::E] {
        frame(&ctx, &mut app, vec![key(k, false)], size, 1.);
    }
    let before = app.session.editor.document.clone();
    ribbon_key(&ctx, &mut app, Key::F3, Modifiers::NONE, size);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("Heading".into())],
        size,
        1.,
    );
    frame(&ctx, &mut app, vec![], size, 1.);
    assert_eq!(app.style_filter, "Heading");
    assert_eq!(app.session.editor.document, before);
    assert!(!app.ribbon_menu_targets.iter().any(|t| matches!(
        t.kind,
        TargetKind::Action(Action::Style(ParagraphStyle::Normal))
    )));
    ribbon_key(&ctx, &mut app, Key::ArrowDown, Modifiers::NONE, size);
    ribbon_key(&ctx, &mut app, Key::Enter, Modifiers::NONE, size);
    assert_eq!(
        app.session.editor.selected_paragraph_style(),
        Some(ParagraphStyle::Heading1)
    );
    app.session.editor.undo();
    assert_eq!(app.session.editor.document, before);
}

#[test]
fn style_filter_empty_scope_and_reset_do_not_change_history() {
    let (ctx, mut app) = setup();
    app.style_filter = "missing".into();
    app.style_scope = crate::style_filter::Scope::Custom;
    let before = app.session.editor.document.clone();
    for k in [Key::F10, Key::H, Key::E] {
        frame(&ctx, &mut app, vec![key(k, false)], [640., 480.], 1.);
    }
    frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    assert!(!app.ribbon_menu_targets.iter().any(|t| matches!(
        t.kind,
        TargetKind::Action(Action::Style(_) | Action::CharacterStyle(_))
    )));
    assert_eq!(app.session.editor.document, before);
    click_region_at(&ctx, &mut app, "style-filter-reset", [640., 480.]);
    assert!(app.style_filter.is_empty());
    assert_eq!(app.style_scope, crate::style_filter::Scope::All);
    assert_eq!(app.session.editor.document, before);
    click_region_at(&ctx, &mut app, "style-filter-scope", [640., 480.]);
    assert_eq!(app.style_scope, crate::style_filter::Scope::Paragraph);
    frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    assert!(
        !app.ribbon_menu_targets
            .iter()
            .any(|t| matches!(t.kind, TargetKind::Action(Action::CharacterStyle(_))))
    );
    ribbon_key(&ctx, &mut app, Key::Escape, Modifiers::NONE, [640., 480.]);
    assert_eq!(app.session.editor.document, before);
}

#[test]
fn used_style_filter_intersects_scope_and_reset_preserves_document() {
    let (ctx, mut app) = setup();
    app.session
        .editor
        .apply_paragraph_style(0, 0, ParagraphStyle::Heading1)
        .unwrap();
    let before = app.session.editor.document.clone();
    let size = [640., 480.];
    frame(&ctx, &mut app, vec![], size, 1.);
    for k in [Key::F10, Key::H, Key::E] {
        frame(&ctx, &mut app, vec![key(k, false)], size, 1.);
    }
    click_region_at(&ctx, &mut app, "style-filter-used", size);
    frame(&ctx, &mut app, vec![], size, 1.);
    assert!(app.style_used_only);
    assert!(
        app.regions
            .iter()
            .any(|r| r.name == "style-consumers-Style(Normal)-Required by: Heading 1")
    );
    assert!(
        app.regions
            .iter()
            .any(|r| r.name == "style-usage-Style(Normal)-Parent dependency")
    );
    assert!(
        app.regions
            .iter()
            .any(|r| r.name == "style-usage-Style(Heading1)-Direct use")
    );
    assert!(app.ribbon_menu_targets.iter().any(|t| matches!(
        t.kind,
        TargetKind::Action(Action::Style(ParagraphStyle::Normal))
    )));
    assert!(!app.ribbon_menu_targets.iter().any(|t| matches!(
        t.kind,
        TargetKind::Action(Action::Style(ParagraphStyle::NoSpacing))
    )));
    app.style_filter = "Normal".into();
    frame(&ctx, &mut app, vec![], size, 1.);
    assert!(
        app.regions
            .iter()
            .any(|r| r.name == "style-consumers-Style(Normal)-Required by: Heading 1")
    );
    assert!(!app.ribbon_menu_targets.iter().any(|t| matches!(
        t.kind,
        TargetKind::Action(Action::Style(ParagraphStyle::Heading1))
    )));
    click_region_at(&ctx, &mut app, "style-filter-reset", size);
    assert!(!app.style_used_only);
    assert_eq!(app.session.editor.document, before);
    app.session.editor.undo();
    assert_eq!(
        app.session.editor.selected_paragraph_style(),
        Some(ParagraphStyle::Normal)
    );
}

#[test]
fn required_by_navigation_reveals_focuses_without_applying() {
    for keyboard in [false, true] {
        let (ctx, mut app) = setup();
        app.session
            .editor
            .apply_paragraph_style(0, 0, ParagraphStyle::Heading1)
            .unwrap();
        let before = app.session.editor.document.clone();
        let depth = app.session.editor.undo_stack.len();
        let size = [640., 480.];
        app.style_used_only = true;
        app.style_filter = "Normal".into();
        app.style_scope = crate::style_filter::Scope::Paragraph;
        frame(&ctx, &mut app, vec![], size, 1.);
        for k in [Key::F10, Key::H, Key::E] {
            frame(&ctx, &mut app, vec![key(k, false)], size, 1.);
        }
        frame(&ctx, &mut app, vec![], size, 1.);
        let name = "style-jump-Style(Normal)-Style(Heading1)";
        if keyboard {
            app.ribbon_menu_index = app.ribbon_menu_targets.iter().position(|t| t.name == name);
            assert!(app.ribbon_menu_index.is_some());
            frame(&ctx, &mut app, vec![], size, 1.);
            ribbon_key(&ctx, &mut app, Key::Enter, Modifiers::NONE, size);
        } else {
            click_region_at(&ctx, &mut app, name, size);
        }
        for _ in 0..12 {
            frame(&ctx, &mut app, vec![], size, 1.);
        }
        assert!(egui::Popup::is_any_open(&ctx));
        assert!(app.style_filter.is_empty());
        assert_eq!(app.style_scope, crate::style_filter::Scope::All);
        assert!(app.style_used_only);
        let target = &app.ribbon_menu_targets[app.ribbon_menu_index.unwrap()];
        assert!(matches!(
            target.kind,
            TargetKind::Action(Action::Style(ParagraphStyle::Heading1))
        ));
        assert!(ctx.memory(|m| m.has_focus(target.id)));
        assert!(
            app.regions
                .iter()
                .any(|r| r.name == "command-Style(Heading1)")
        );
        assert_eq!(app.session.editor.document, before);
        assert_eq!(app.session.editor.undo_stack.len(), depth);
        let back = app
            .regions
            .iter()
            .find(|r| r.name == "style-return-label-Back to: Normal")
            .unwrap();
        assert!(back.rect.width() <= 220.1, "{:?}", back.rect);
        app.style_used_only = false;
        app.style_filter = "No match".into();
        app.style_scope = crate::style_filter::Scope::Custom;
        frame(&ctx, &mut app, vec![], size, 1.);
        if keyboard {
            app.ribbon_menu_index = app
                .ribbon_menu_targets
                .iter()
                .position(|t| t.name == "style-return");
            assert!(app.ribbon_menu_index.is_some());
            frame(&ctx, &mut app, vec![], size, 1.);
            ribbon_key(&ctx, &mut app, Key::Space, Modifiers::NONE, size);
        } else {
            click_region_at(&ctx, &mut app, "style-return", size);
        }
        for _ in 0..12 {
            frame(&ctx, &mut app, vec![], size, 1.);
        }
        assert!(egui::Popup::is_any_open(&ctx));
        assert_eq!(app.style_filter, "Normal");
        assert_eq!(app.style_scope, crate::style_filter::Scope::Paragraph);
        assert!(app.style_used_only);
        assert!(app.style_return.is_none());
        let target = &app.ribbon_menu_targets[app.ribbon_menu_index.unwrap()];
        assert!(matches!(
            target.kind,
            TargetKind::Action(Action::Style(ParagraphStyle::Normal))
        ));
        assert!(ctx.memory(|m| m.has_focus(target.id)));
        assert!(
            app.regions
                .iter()
                .any(|r| r.name == "command-Style(Normal)")
        );
        assert_eq!(app.session.editor.document, before);
        assert_eq!(app.session.editor.undo_stack.len(), depth);
        ribbon_key(&ctx, &mut app, Key::Escape, Modifiers::NONE, size);
        assert!(!egui::Popup::is_any_open(&ctx));
    }
}

#[test]
fn dependency_return_keeps_original_source_through_chained_jumps_and_clears_on_close() {
    let (ctx, mut app) = setup();
    let source = Action::CharacterStyle(Some(CharacterStyle::Strong));
    let middle = Action::CharacterStyle(Some(CharacterStyle::Custom(1)));
    let leaf = Action::CharacterStyle(Some(CharacterStyle::Custom(2)));
    app.style_filter = "Strong".into();
    app.style_scope = crate::style_filter::Scope::Character;
    app.jump_to_style(source, middle);
    app.jump_to_style(middle, leaf);
    app.return_to_style();
    assert_eq!(app.style_jump, Some(source));
    assert_eq!(app.style_filter, "Strong");
    assert_eq!(app.style_scope, crate::style_filter::Scope::Character);
    assert!(!app.style_used_only);
    app.jump_to_style(source, middle);
    frame(&ctx, &mut app, vec![], [640., 480.], 1.);
    assert!(app.style_return.is_none());
    assert!(app.style_jump.is_none());
}

#[test]
fn return_name_tracks_long_custom_names_without_expanding_menu() {
    let (ctx, mut app) = setup();
    let long = "Український стиль e\u{301} ".repeat(2).trim().to_owned();
    let id = app
        .session
        .editor
        .document
        .add_character_style(&long, CharacterFormat::default())
        .unwrap();
    let size = [640., 480.];
    frame(&ctx, &mut app, vec![], size, 1.);
    for k in [Key::F10, Key::H, Key::E] {
        frame(&ctx, &mut app, vec![key(k, false)], size, 1.);
    }
    frame(&ctx, &mut app, vec![], size, 1.);
    app.style_return = Some((
        Action::CharacterStyle(Some(id)),
        String::new(),
        crate::style_filter::Scope::All,
        false,
    ));
    frame(&ctx, &mut app, vec![], size, 1.);
    let name = format!("style-return-label-Back to: {long}");
    let rect = app.regions.iter().find(|r| r.name == name).unwrap().rect;
    assert!(rect.width() <= 220.1, "{rect:?}");
    assert!(rect.right() <= size[0]);
    app.session
        .editor
        .document
        .update_character_style(id, "Renamed parent", CharacterFormat::default())
        .unwrap();
    frame(&ctx, &mut app, vec![], size, 1.);
    assert!(
        app.regions
            .iter()
            .any(|r| r.name == "style-return-label-Back to: Renamed parent")
    );
}

#[test]
fn dependency_return_handles_renamed_deleted_and_unused_sources() {
    for change in 0..3 {
        let (ctx, mut app) = setup();
        let id = app
            .session
            .editor
            .document
            .add_character_style("Original", CharacterFormat::default())
            .unwrap();
        app.session.editor.replace(0, 0, "x", "Fixture").unwrap();
        app.session
            .editor
            .apply_character_style(0, 1, Some(id))
            .unwrap();
        let source = Action::CharacterStyle(Some(id));
        let size = [640., 480.];
        frame(&ctx, &mut app, vec![], size, 1.);
        for k in [Key::F10, Key::H, Key::E] {
            ribbon_key(&ctx, &mut app, k, Modifiers::NONE, size);
        }
        app.style_filter = "Original".into();
        app.style_scope = crate::style_filter::Scope::Character;
        app.style_used_only = change == 2;
        app.jump_to_style(source, Action::CharacterStyle(Some(CharacterStyle::Strong)));
        match change {
            0 => app
                .session
                .editor
                .document
                .update_character_style(id, "Renamed", CharacterFormat::default())
                .unwrap(),
            1 => app
                .session
                .editor
                .document
                .delete_character_style(id, None)
                .unwrap(),
            _ => app
                .session
                .editor
                .apply_character_style(0, 1, None)
                .unwrap(),
        }
        let before = app.session.editor.document.clone();
        let depth = app.session.editor.undo_stack.len();
        app.return_to_style();
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], size, 1.);
        }
        assert_eq!(app.style_filter, "Original");
        assert_eq!(app.style_scope, crate::style_filter::Scope::Character);
        assert_eq!(app.style_used_only, change == 2);
        assert!(app.style_jump.is_none());
        assert!(app.style_return.is_none());
        assert!(app.style_navigation_notice.is_some());
        assert!(ctx.memory(|m| m.has_focus(Id::new("style-filter-input"))));
        assert_eq!(app.session.editor.document, before);
        assert_eq!(app.session.editor.undo_stack.len(), depth);
        // Reset must not resurrect a stale jump when its destination becomes visible.
        click_region_at(&ctx, &mut app, "style-filter-reset", size);
        assert!(app.style_jump.is_none());
    }
}

#[test]
fn dependency_navigation_long_reverse_catalog_chain_keeps_identity_and_history() {
    let (ctx, mut app) = setup();
    let mut ids = Vec::new();
    for n in 0..32 {
        ids.push(
            app.session
                .editor
                .document
                .add_character_style(&format!("Style {n}"), CharacterFormat::default())
                .unwrap(),
        );
    }
    for n in 0..31 {
        app.session
            .editor
            .document
            .set_character_parent(ids[n], Some(ids[n + 1]))
            .unwrap();
    }
    app.session.editor.replace(0, 0, "ab", "Fixture").unwrap();
    app.session
        .editor
        .apply_character_style(0, 1, Some(ids[0]))
        .unwrap();
    app.session
        .editor
        .apply_character_style(1, 2, Some(ids[16]))
        .unwrap();
    let source = Action::CharacterStyle(Some(ids[31]));
    let middle = Action::CharacterStyle(Some(ids[16]));
    let leaf = Action::CharacterStyle(Some(ids[0]));
    let before = app.session.editor.document.clone();
    let depth = app.session.editor.undo_stack.len();
    let size = [640., 480.];
    frame(&ctx, &mut app, vec![], size, 1.);
    for k in [Key::F10, Key::H, Key::E] {
        ribbon_key(&ctx, &mut app, k, Modifiers::NONE, size);
    }
    app.style_filter = "Style 31".into();
    app.style_scope = crate::style_filter::Scope::Custom;
    for (from, to) in [(source, middle), (middle, leaf)] {
        frame(&ctx, &mut app, vec![], size, 1.);
        app.ribbon_menu_index = app
            .ribbon_menu_targets
            .iter()
            .position(|t| matches!(t.kind, TargetKind::StyleJump(a, b) if a == from && b == to));
        assert!(app.ribbon_menu_index.is_some());
        frame(&ctx, &mut app, vec![], size, 1.);
        ribbon_key(&ctx, &mut app, Key::Enter, Modifiers::NONE, size);
        for _ in 0..12 {
            frame(&ctx, &mut app, vec![], size, 1.);
        }
        let target = &app.ribbon_menu_targets[app.ribbon_menu_index.unwrap()];
        assert!(matches!(target.kind, TargetKind::Action(action) if action == to));
        assert!(ctx.memory(|m| m.has_focus(target.id)));
        assert!(app.style_jump.is_none());
    }
    app.return_to_style();
    for _ in 0..12 {
        frame(&ctx, &mut app, vec![], size, 1.);
    }
    assert_eq!(app.style_filter, "Style 31");
    let target = &app.ribbon_menu_targets[app.ribbon_menu_index.unwrap()];
    assert!(matches!(target.kind, TargetKind::Action(action) if action == source));
    assert!(ctx.memory(|m| m.has_focus(target.id)));
    assert_eq!(app.session.editor.document, before);
    assert_eq!(app.session.editor.undo_stack.len(), depth);
}

#[test]
fn dependency_keyboard_navigation_scale_matrix() {
    for dpi in [1., 1.25, 1.5] {
        for logical in [[640., 480.], [1024., 768.], [1366., 900.]] {
            for collapsed in [false, true] {
                let (ctx, mut app) = setup();
                app.session
                    .editor
                    .apply_paragraph_style(0, 0, ParagraphStyle::Heading1)
                    .unwrap();
                app.ribbon_collapsed = collapsed;
                app.style_filter = "Normal".into();
                app.style_used_only = true;
                let before = app.session.editor.document.clone();
                let depth = app.session.editor.undo_stack.len();
                let screen = [logical[0] * dpi, logical[1] * dpi];
                let press = |app: &mut DocsApp, k, modifiers| {
                    for pressed in [true, false] {
                        frame(
                            &ctx,
                            app,
                            vec![egui::Event::Key {
                                key: k,
                                physical_key: Some(k),
                                pressed,
                                repeat: false,
                                modifiers,
                            }],
                            screen,
                            dpi,
                        );
                    }
                };
                for _ in 0..3 {
                    frame(&ctx, &mut app, vec![], screen, dpi);
                }
                if collapsed {
                    press(&mut app, Key::H, Modifiers::ALT);
                } else {
                    press(&mut app, Key::F10, Modifiers::NONE);
                    press(&mut app, Key::H, Modifiers::NONE);
                }
                press(
                    &mut app,
                    if logical[0] == 640. { Key::E } else { Key::L },
                    Modifiers::NONE,
                );
                press(&mut app, Key::F3, Modifiers::NONE);
                assert!(ctx.memory(|m| m.has_focus(Id::new("style-filter-input"))));
                press(&mut app, Key::ArrowDown, Modifiers::NONE);
                let link = "style-jump-Style(Normal)-Style(Heading1)";
                for _ in 0..20 {
                    if app
                        .ribbon_menu_index
                        .is_some_and(|i| app.ribbon_menu_targets[i].name == link)
                    {
                        break;
                    }
                    press(&mut app, Key::Tab, Modifiers::NONE);
                }
                let index = app.ribbon_menu_index.unwrap();
                assert_eq!(app.ribbon_menu_targets[index].name, link);
                press(&mut app, Key::Tab, Modifiers::SHIFT);
                press(&mut app, Key::Tab, Modifiers::NONE);
                assert_eq!(app.ribbon_menu_index, Some(index));
                press(&mut app, Key::Enter, Modifiers::NONE);
                for _ in 0..12 {
                    frame(&ctx, &mut app, vec![], screen, dpi);
                }
                let target = &app.ribbon_menu_targets[app.ribbon_menu_index.unwrap()];
                assert!(matches!(
                    target.kind,
                    TargetKind::Action(Action::Style(ParagraphStyle::Heading1))
                ));
                assert!(ctx.memory(|m| m.has_focus(target.id)));
                let back = app
                    .regions
                    .iter()
                    .find(|r| r.name == "style-return")
                    .unwrap();
                assert!(back.rect.min.x >= 0. && back.rect.max.x <= logical[0] + 0.1);
                assert!(back.rect.min.y >= 0. && back.rect.max.y <= logical[1] + 0.1);
                for _ in 0..20 {
                    if app
                        .ribbon_menu_index
                        .is_some_and(|i| app.ribbon_menu_targets[i].name == "style-return")
                    {
                        break;
                    }
                    press(&mut app, Key::ArrowUp, Modifiers::NONE);
                }
                assert_eq!(
                    app.ribbon_menu_targets[app.ribbon_menu_index.unwrap()].name,
                    "style-return"
                );
                press(&mut app, Key::Space, Modifiers::NONE);
                for _ in 0..12 {
                    frame(&ctx, &mut app, vec![], screen, dpi);
                }
                assert_eq!(app.style_filter, "Normal");
                assert!(app.style_used_only);
                assert!(app.style_return.is_none());
                let target = &app.ribbon_menu_targets[app.ribbon_menu_index.unwrap()];
                assert!(matches!(
                    target.kind,
                    TargetKind::Action(Action::Style(ParagraphStyle::Normal))
                ));
                assert!(ctx.memory(|m| m.has_focus(target.id)));
                assert_eq!(app.session.editor.document, before);
                assert_eq!(app.session.editor.undo_stack.len(), depth);
                press(&mut app, Key::Escape, Modifiers::NONE);
                assert!(!egui::Popup::is_any_open(&ctx));
            }
        }
    }
}

#[test]
fn paragraph_spacing_dialog_real_input_history_reset_and_layout() {
    let (ctx, mut app) = setup();
    app.session
        .editor
        .replace(0, 0, "Body\nNext", "Fixture")
        .unwrap();
    app.session.editor.selection = (0, 0);
    app.sync_view();
    let before = app.session.editor.document.clone();
    let depth = app.session.editor.undo_stack.len();
    let layout_before = layout::galley(
        &ctx,
        &before,
        &before.text(),
        CharacterFormat::default(),
        0,
        500.,
        15.,
    );
    app.execute(Action::ParagraphSpacing, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    click_region(&ctx, &mut app, "paragraph-line");
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("200".into())],
        [1366., 900.],
        1.,
    );
    click_region(&ctx, &mut app, "paragraph-after");
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("20.5".into())],
        [1366., 900.],
        1.,
    );
    click_region(&ctx, &mut app, "paragraph-before");
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("7.5".into())],
        [1366., 900.],
        1.,
    );
    for (field, text) in [("paragraph-left", "18"), ("paragraph-right", "12.5")] {
        click_region(&ctx, &mut app, field);
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::Text(text.into())],
            [1366., 900.],
            1.,
        );
    }
    click_region(&ctx, &mut app, "paragraph-first");
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("18.5".into())],
        [1366., 900.],
        1.,
    );
    click_region(&ctx, &mut app, "paragraph-hanging");
    app.execute(Action::New, &ctx); // Modal must guard background commands.
    assert_eq!(app.session.editor.document, before);
    click_region(&ctx, &mut app, "paragraph-ok");
    assert!(app.paragraph_dialog.is_none());
    assert_eq!(app.session.editor.undo_stack.len(), depth + 1);
    let after = app.session.editor.document.clone();
    assert_eq!(after.paragraphs[0].spacing.line_height_percent, Some(200));
    assert_eq!(
        after.paragraphs[0].spacing.space_after_half_points,
        Some(41)
    );
    assert_eq!(
        after.paragraphs[0].spacing.space_before_half_points,
        Some(15)
    );
    assert_eq!(
        after.paragraphs[0].spacing.left_indent_half_points,
        Some(36)
    );
    assert_eq!(
        after.paragraphs[0].spacing.right_indent_half_points,
        Some(25)
    );
    assert_eq!(
        after.paragraphs[0].spacing.first_line_half_points,
        Some(-37)
    );
    assert_eq!(after.paragraphs[1], before.paragraphs[1]);
    let layout_after = layout::galley(
        &ctx,
        &after,
        &after.text(),
        CharacterFormat::default(),
        0,
        500.,
        15.,
    );
    assert!(layout_after.rows[0].pos.y > layout_before.rows[0].pos.y);
    assert!(layout_after.rows[0].size.y > layout_before.rows[0].size.y);
    assert!(layout_after.rows[1].pos.y > layout_before.rows[1].pos.y);
    assert_eq!(
        storage::decode(&storage::encode(&after).unwrap()).unwrap(),
        after
    );
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::Redo, &ctx);
    assert_eq!(app.session.editor.document, after);
    app.execute(Action::ParagraphSpacing, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    click_region(&ctx, &mut app, "paragraph-reset");
    click_region(&ctx, &mut app, "paragraph-ok");
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, after);
}

#[test]
fn paragraph_spacing_dialog_invalid_and_cancel_leave_document_intact() {
    for invalid in ["NaN", "0.25", "201", "-1"] {
        let (ctx, mut app) = setup();
        let before = app.session.editor.document.clone();
        app.execute(Action::ParagraphSpacing, &ctx);
        frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
        click_region(&ctx, &mut app, "paragraph-after");
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::Text(invalid.into())],
            [1366., 900.],
            1.,
        );
        click_region(&ctx, &mut app, "paragraph-ok");
        assert!(app.paragraph_dialog.is_some());
        assert_eq!(app.session.editor.document, before);
        frame(
            &ctx,
            &mut app,
            vec![key(Key::Escape, false)],
            [1366., 900.],
            1.,
        );
        assert!(app.paragraph_dialog.is_none());
        assert!(app.session.editor.undo_stack.is_empty());
    }
}

#[test]
fn paragraph_spacing_keytips_and_scaled_modal_bounds() {
    for dpi in [1., 1.25, 1.5] {
        for logical in [[640., 480.], [1024., 768.], [1366., 900.]] {
            let (ctx, mut app) = setup();
            let screen = [logical[0] * dpi, logical[1] * dpi];
            for _ in 0..3 {
                frame(&ctx, &mut app, vec![], screen, dpi);
            }
            for k in [Key::F10, Key::H, Key::P, Key::S] {
                frame(&ctx, &mut app, tip_events(&[k]), screen, dpi);
            }
            frame(&ctx, &mut app, vec![], screen, dpi);
            assert!(app.paragraph_dialog.is_some());
            let rect = app
                .regions
                .iter()
                .find(|r| r.name == "paragraph-dialog")
                .unwrap()
                .rect;
            assert!(
                rect.min.x >= 0.
                    && rect.min.y >= 0.
                    && rect.max.x <= logical[0]
                    && rect.max.y <= logical[1],
                "{rect:?}"
            );
            for name in [
                "paragraph-line",
                "paragraph-before",
                "paragraph-left",
                "paragraph-right",
                "paragraph-first",
                "paragraph-hanging",
                "paragraph-after",
                "paragraph-ok",
                "paragraph-cancel",
            ] {
                assert!(
                    rect.contains_rect(app.regions.iter().find(|r| r.name == name).unwrap().rect)
                );
            }
            frame(&ctx, &mut app, vec![key(Key::Escape, false)], screen, dpi);
            assert!(app.paragraph_dialog.is_none());
            assert!(app.session.editor.undo_stack.is_empty());
        }
    }
}

#[test]
fn space_before_offsets_first_wrapped_and_empty_paragraphs_once() {
    let (ctx, mut app) = setup();
    app.session
        .editor
        .replace(
            0,
            0,
            "A long paragraph with wrapped words\n\nNext",
            "Fixture",
        )
        .unwrap();
    let before = app.session.editor.document.clone();
    let mut after = before.clone();
    after.paragraphs[0].spacing.space_before_half_points = Some(22);
    after.paragraphs[1].spacing.space_before_half_points = Some(44);
    after.paragraphs[2].spacing.space_before_half_points = Some(66);
    let render = |d: &crate::model::Document| {
        layout::galley(&ctx, d, &d.text(), CharacterFormat::default(), 0, 85., 11.)
    };
    let a = render(&before);
    let b = render(&after);
    assert_eq!(a.rows.len(), b.rows.len());
    assert!(a.rows.len() > 3);
    let mut offset = 11.;
    let mut paragraph = 0;
    for (x, y) in a.rows.iter().zip(&b.rows) {
        assert!((y.pos.y - x.pos.y - offset).abs() < 0.01);
        if x.ends_with_newline {
            paragraph += 1;
            offset += match paragraph {
                1 => 22.,
                2 => 33.,
                _ => 0.,
            };
        }
    }
    assert!((b.rect.height() - a.rect.height() - 66.).abs() < 0.01);
}

#[test]
fn paragraph_indents_wrap_align_and_hit_test_without_changing_text() {
    let (ctx, mut app) = setup();
    app.session
        .editor
        .replace(
            0,
            0,
            "English words wrap here and continue\n\nУкраїна e\u{301}\nРусский",
            "Fixture",
        )
        .unwrap();
    let mut doc = app.session.editor.document.clone();
    doc.style_catalog.normal.left_indent_half_points = Some(36);
    doc.style_catalog.normal.right_indent_half_points = Some(24);
    for alignment in [
        Alignment::Left,
        Alignment::Center,
        Alignment::Right,
        Alignment::Justify,
    ] {
        for p in &mut doc.paragraphs {
            p.alignment = alignment;
        }
        let g = layout::galley(
            &ctx,
            &doc,
            &doc.text(),
            CharacterFormat::default(),
            0,
            130.,
            11.,
        );
        assert_eq!(g.text(), doc.text());
        assert!(g.rows.len() > doc.paragraphs.len());
        for row in &g.rows {
            assert!(row.x_offset(0) >= 17.9);
            assert!(
                row.glyphs
                    .iter()
                    .filter(|g| !g.chr.is_whitespace())
                    .all(|g| g.max_x() <= 118.5),
                "{:?}: {}",
                alignment,
                row.size.x
            );
        }
        for i in [0, 3, 36, 37, 40, 46, 48] {
            let pos = g.pos_from_cursor(CCursor::new(i));
            assert_eq!(
                g.cursor_from_pos(pos.center().to_vec2()).index,
                i,
                "{alignment:?}, {i}"
            );
        }
    }
    for p in &mut doc.paragraphs {
        p.alignment = Alignment::Left;
    }
    doc.style_catalog.normal.left_indent_half_points = Some(288);
    doc.style_catalog.normal.right_indent_half_points = Some(288);
    let g = layout::galley(
        &ctx,
        &doc,
        &doc.text(),
        CharacterFormat::default(),
        0,
        80.,
        11.,
    );
    assert!(g.rows.iter().all(|r| r.size.x <= 56.5));
    assert_eq!(doc.style_catalog.normal.left_indent_half_points, Some(288));
}

#[test]
fn paragraph_indent_invalid_widget_input_is_transactional() {
    let (ctx, mut app) = setup();
    let before = app.session.editor.document.clone();
    app.execute(Action::ParagraphSpacing, &ctx);
    frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    click_region(&ctx, &mut app, "paragraph-left");
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Text("144.5".into())],
        [1366., 900.],
        1.,
    );
    click_region(&ctx, &mut app, "paragraph-ok");
    assert!(app.paragraph_dialog.is_some());
    assert_eq!(app.session.editor.document, before);
    click_region(&ctx, &mut app, "paragraph-cancel");
    assert!(app.paragraph_dialog.is_none());
    assert_eq!(app.session.editor.document, before);
}

#[test]
fn special_indents_shape_first_and_continuation_rows_without_fake_newlines() {
    use unicode_segmentation::UnicodeSegmentation;
    let (ctx, mut app) = setup();
    app.session.editor.replace(0,0,"English words wrap here and continue. Україна e\u{301} Русский. More words wrap here.\n\nLast", "Fixture").unwrap();
    // Mixed runs must retain their formats after the first-row/suffix split.
    app.session.editor.emphasize(8, 18, true, true).unwrap();
    let mut doc = app.session.editor.document.clone();
    for value in [36, -36, 0, 288, -288] {
        doc.style_catalog.normal.first_line_half_points = Some(value);
        for alignment in [
            Alignment::Left,
            Alignment::Center,
            Alignment::Right,
            Alignment::Justify,
        ] {
            for p in &mut doc.paragraphs {
                p.alignment = alignment;
            }
            let g = layout::galley(
                &ctx,
                &doc,
                &doc.text(),
                CharacterFormat::default(),
                0,
                145.,
                11.,
            );
            assert_eq!(g.text(), doc.text());
            assert_eq!(g.rows.iter().filter(|r| r.ends_with_newline).count(), 2);
            assert_eq!(
                g.rows
                    .iter()
                    .map(|r| r.char_count_excluding_newline())
                    .sum::<usize>(),
                doc.text().chars().filter(|c| *c != '\n').count()
            );
            assert!(g.rows.iter().all(|r| {
                r.glyphs
                    .iter()
                    .filter(|g| !g.chr.is_whitespace())
                    .all(|g| g.max_x() <= 145.5)
                    && r.height() > 0.
            }));
            if alignment == Alignment::Left && value.abs() == 36 {
                assert!((g.rows[0].x_offset(0) - if value > 0 { 18. } else { 0. }).abs() < 0.5);
                assert!((g.rows[1].x_offset(0) - if value < 0 { 18. } else { 0. }).abs() < 0.5);
            }
            let text = doc.text();
            let mut boundaries: Vec<usize> = text
                .grapheme_indices(true)
                .map(|(byte, _)| text[..byte].chars().count())
                .collect();
            boundaries.push(text.chars().count());
            let mut end = 0;
            for row in &g.rows {
                end += row.char_count_excluding_newline();
                assert!(boundaries.contains(&end), "Split grapheme at {end}");
                if row.ends_with_newline {
                    end += 1;
                }
            }
            for i in [0, 3, 8, 15, 20, 35] {
                let pos = g.pos_from_cursor(CCursor::new(i));
                assert_eq!(
                    g.cursor_from_pos(pos.center().to_vec2()).index,
                    i,
                    "{value} {alignment:?} {i}"
                );
            }
        }
    }
}

#[test]
fn extended_indent_layout_matrix_preserves_rows_metrics_and_unicode() {
    use unicode_segmentation::UnicodeSegmentation;
    let (ctx, mut app) = setup();
    let text = format!(
        "  {}\n\n{}\n",
        "English Україна Русский cafe\u{301}  ".repeat(12),
        "LongWordWithoutSpaces".repeat(8)
    );
    app.session.editor.replace(0, 0, &text, "Fixture").unwrap();
    app.session
        .editor
        .set_font_size(2, 9, "24".parse().unwrap())
        .unwrap();
    app.session.editor.emphasize(10, 17, true, true).unwrap();
    app.sync_view();
    let baseline = app.session.editor.document.clone();
    let boundaries = crate::model::boundaries(&text);
    for dpi in [1., 1.25, 1.5] {
        frame(&ctx, &mut app, vec![], [1366. * dpi, 900. * dpi], dpi);
        assert!((ctx.pixels_per_point() - dpi).abs() < 0.01);
        for width in [80., 145., 500.] {
            for special in [-288, 0, 288] {
                for align in [
                    Alignment::Left,
                    Alignment::Center,
                    Alignment::Right,
                    Alignment::Justify,
                ] {
                    let mut doc = baseline.clone();
                    doc.style_catalog.normal.left_indent_half_points = Some(60);
                    doc.style_catalog.normal.right_indent_half_points = Some(30);
                    doc.style_catalog.normal.first_line_half_points = Some(special);
                    for p in &mut doc.paragraphs {
                        p.alignment = align;
                    }
                    let before = doc.clone();
                    let g = layout::galley(
                        &ctx,
                        &doc,
                        &text,
                        CharacterFormat::default(),
                        0,
                        width,
                        15.,
                    );
                    assert_eq!(doc, before);
                    assert_eq!(g.text(), text);
                    let mut reconstructed = String::new();
                    let mut offset = 0;
                    for row in &g.rows {
                        reconstructed.extend(row.glyphs.iter().map(|glyph| glyph.chr));
                        offset += row.char_count_excluding_newline();
                        assert!(
                            boundaries.contains(&offset),
                            "split grapheme at {offset}; {dpi} {width} {special} {align:?}"
                        );
                        if row.ends_with_newline {
                            reconstructed.push('\n');
                            offset += 1;
                        }
                        assert!(row.pos.y.is_finite() && row.height() > 0.);
                        assert!(
                            row.max_y() <= g.rect.max.y + 0.51,
                            "height {dpi} {width} {special} {align:?}"
                        );
                    }
                    assert_eq!(reconstructed, text);
                    assert_eq!(
                        g.num_vertices,
                        g.rows
                            .iter()
                            .map(|r| r.visuals.mesh.vertices.len())
                            .sum::<usize>()
                    );
                    assert_eq!(
                        g.num_indices,
                        g.rows
                            .iter()
                            .map(|r| r.visuals.mesh.indices.len())
                            .sum::<usize>()
                    );
                    for pair in g.rows.windows(2) {
                        assert!(pair[0].max_y() <= pair[1].min_y() + 0.51);
                    }
                    for (byte, grapheme) in text.grapheme_indices(true).step_by(17) {
                        if grapheme.chars().all(char::is_whitespace) {
                            continue;
                        }
                        let index = text[..byte].chars().count();
                        let pos = g.pos_from_cursor(CCursor::new(index));
                        assert_eq!(
                            g.cursor_from_pos(pos.center().to_vec2()).index,
                            index,
                            "hit {dpi} {width} {special} {align:?} {index}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn indented_widget_selection_survives_zoom_edit_history_and_reopen() {
    let (ctx, mut app) = setup();
    let text = format!(
        "{}\n\nУкраїна e\u{301} Русский",
        "English wrapped words  ".repeat(20)
    );
    app.session.editor.replace(0, 0, &text, "Fixture").unwrap();
    app.session
        .editor
        .set_paragraph_geometry(
            0,
            text.chars().count(),
            crate::commands::ParagraphGeometryPatch {
                left: Some(Some(36)),
                right: Some(Some(24)),
                first_line: Some(Some(-51)),
                ..Default::default()
            },
        )
        .unwrap();
    let start = text.chars().count() - 7;
    app.session.editor.selection = (start, text.chars().count());
    app.sync_view();
    let before = app.session.editor.document.clone();
    for zoom in [50., 125., 200., 100.] {
        app.zoom = zoom;
        frame(&ctx, &mut app, vec![], [1024., 768.], 1.);
        assert_eq!(app.session.editor.document, before);
        assert_eq!(app.session.editor.selection, (start, text.chars().count()));
    }
    let depth = app.session.editor.undo_stack.len();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Paste("замена".into())],
        [1024., 768.],
        1.,
    );
    assert_eq!(app.session.editor.undo_stack.len(), depth + 1);
    let after = app.session.editor.document.clone();
    assert_eq!(
        after.paragraphs.last().unwrap().spacing,
        before.paragraphs.last().unwrap().spacing
    );
    app.execute(Action::Undo, &ctx);
    frame(&ctx, &mut app, vec![], [1024., 768.], 1.);
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::Redo, &ctx);
    frame(&ctx, &mut app, vec![], [1024., 768.], 1.);
    assert_eq!(app.session.editor.document, after);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("indent-layout.bfdoc");
    app.session.save(&path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), after);
}

#[test]
fn justified_trailing_spaces_keep_row_end_caret_and_projection_text() {
    let (ctx, mut app) = setup();
    let text = "alpha beta gamma delta  epsilon zeta  ";
    app.session.editor.replace(0, 0, text, "Fixture").unwrap();
    app.session.editor.align(0, 0, Alignment::Justify).unwrap();
    app.session
        .editor
        .set_paragraph_geometry(
            0,
            0,
            crate::commands::ParagraphGeometryPatch {
                first_line: Some(Some(-36)),
                ..Default::default()
            },
        )
        .unwrap();
    let doc = app.session.editor.document.clone();
    for display in [text.to_owned(), format!("{text}e\u{301} Україна")] {
        let g = layout::galley(
            &ctx,
            &doc,
            &display,
            CharacterFormat::default(),
            0,
            100.,
            15.,
        );
        let mut end = 0;
        for row in &g.rows {
            end += row.char_count_excluding_newline();
            let position = g.pos_from_cursor(CCursor::new(end));
            assert_eq!(g.cursor_from_pos(position.center().to_vec2()).index, end);
        }
        assert_eq!(end, display.chars().count());
        assert_eq!(g.text(), display);
        assert_eq!(app.session.editor.document, doc);
    }
}

#[test]
fn compact_style_creation_reserves_room_for_taller_interface_font_metrics() {
    for dpi in [1., 1.25, 1.5] {
        let (ctx, mut app) = setup();
        ctx.style_mut(|style| {
            for key in [
                egui::TextStyle::Body,
                egui::TextStyle::Small,
                egui::TextStyle::Button,
            ] {
                style.text_styles.get_mut(&key).unwrap().size += 1.;
            }
        });
        app.execute(Action::CreateStyle, &ctx);
        for _ in 0..4 {
            frame(&ctx, &mut app, vec![], [640. * dpi, 480. * dpi], dpi);
        }
        let bounds = app
            .regions
            .iter()
            .find(|r| r.name == "style-dialog")
            .unwrap()
            .rect;
        assert!(
            Rect::from_min_size(Pos2::ZERO, vec2(640., 480.)).contains_rect(bounds),
            "{dpi} {bounds:?}"
        );
        for region in app
            .regions
            .iter()
            .filter(|r| r.name.starts_with("style-dialog-"))
        {
            assert!(bounds.contains_rect(region.rect), "{}", region.name);
        }
        frame(
            &ctx,
            &mut app,
            vec![key(Key::Escape, false)],
            [640. * dpi, 480. * dpi],
            dpi,
        );
        assert!(app.style_dialog.is_none());
        assert!(app.session.editor.undo_stack.is_empty());
    }
}

fn ruler_pointer(pos: Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

#[test]
fn ruler_pointer_preview_release_and_escape_preserve_history() {
    for dpi in [1., 1.25, 1.5] {
        for zoom in [50., 100., 200.] {
            let (ctx, mut app) = setup();
            app.zoom = zoom;
            app.session
                .editor
                .replace(0, 0, "English cafe\u{301}\nУкраїна\nРусский", "Fixture")
                .unwrap();
            app.session.editor.selection = (0, app.session.editor.document.text().chars().count());
            app.sync_view();
            let size = [1366. * dpi, 900. * dpi];
            for _ in 0..3 {
                frame(&ctx, &mut app, vec![], size, dpi);
            }
            let original = app.session.editor.document.clone();
            let count = app.session.editor.undo_stack.len();
            let pos = app
                .regions
                .iter()
                .find(|r| r.name == "ruler-first")
                .unwrap()
                .rect
                .center();
            frame(
                &ctx,
                &mut app,
                vec![egui::Event::PointerMoved(pos), ruler_pointer(pos, true)],
                size,
                dpi,
            );
            assert!(app.ruler_drag.is_some(), "dpi={dpi}, zoom={zoom}");
            let to = pos + vec2(48. * zoom / 100., 0.);
            frame(
                &ctx,
                &mut app,
                vec![egui::Event::PointerMoved(to)],
                size,
                dpi,
            );
            assert_eq!(app.session.editor.document, original);
            assert_eq!(app.session.editor.undo_stack.len(), count);
            assert_ne!(app.ruler_drag.as_ref().unwrap().draft.preview, original);
            frame(&ctx, &mut app, vec![ruler_pointer(to, false)], size, dpi);
            assert!(app.ruler_drag.is_none());
            assert_eq!(app.session.editor.undo_stack.len(), count + 1);
            let after = app.session.editor.document.clone();
            assert!(
                after
                    .paragraphs
                    .iter()
                    .all(|p| p.spacing.first_line_half_points == Some(72))
            );
            app.execute(Action::Undo, &ctx);
            assert_eq!(app.session.editor.document, original);
            app.execute(Action::Redo, &ctx);
            assert_eq!(app.session.editor.document, after);
            for _ in 0..2 {
                frame(&ctx, &mut app, vec![], size, dpi);
            }
            let pos = app
                .regions
                .iter()
                .find(|r| r.name == "ruler-body")
                .unwrap()
                .rect
                .center();
            frame(
                &ctx,
                &mut app,
                vec![egui::Event::PointerMoved(pos), ruler_pointer(pos, true)],
                size,
                dpi,
            );
            frame(
                &ctx,
                &mut app,
                vec![egui::Event::PointerMoved(pos + vec2(30., 0.))],
                size,
                dpi,
            );
            frame(&ctx, &mut app, vec![key(Key::Escape, false)], size, dpi);
            assert!(app.ruler_drag.is_none());
            assert_eq!(app.session.editor.document, after);
            frame(&ctx, &mut app, vec![ruler_pointer(pos, false)], size, dpi);
            assert_eq!(app.session.editor.undo_stack.len(), count + 1);
        }
    }
}

#[test]
fn ruler_cancels_on_focus_loss_resize_and_blocks_input() {
    for resize in [false, true] {
        let (ctx, mut app) = setup();
        let original = app.session.editor.document.clone();
        let pos = app
            .regions
            .iter()
            .find(|r| r.name == "ruler-left")
            .unwrap()
            .rect
            .center();
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(pos), ruler_pointer(pos, true)],
            [1366., 900.],
            1.,
        );
        frame(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(pos + vec2(40., 0.)),
                egui::Event::Text("bad".into()),
                key(Key::S, true),
            ],
            [1366., 900.],
            1.,
        );
        assert!(app.ruler_drag.is_some());
        assert_eq!(app.session.editor.document, original);
        if resize {
            frame(&ctx, &mut app, vec![], [1000., 800.], 1.);
        } else {
            let mut input = egui::RawInput {
                focused: false,
                ..Default::default()
            };
            input.events.push(egui::Event::WindowFocused(false));
            let _ = ctx.run(input, |ctx| app.update_ui(ctx));
        }
        assert!(app.ruler_drag.is_none());
        assert_eq!(app.session.editor.document, original);
        assert!(app.session.editor.undo_stack.is_empty());
    }
}

#[test]
fn ruler_lost_pointer_cancels_and_release_outside_commits_once() {
    for gone in [false, true] {
        let (ctx, mut app) = setup();
        let original = app.session.editor.document.clone();
        let pos = app
            .regions
            .iter()
            .find(|r| r.name == "ruler-right")
            .unwrap()
            .rect
            .center();
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(pos), ruler_pointer(pos, true)],
            [1366., 900.],
            1.,
        );
        let to = pos + vec2(-48., 120.);
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(to)],
            [1366., 900.],
            1.,
        );
        frame(
            &ctx,
            &mut app,
            vec![if gone {
                egui::Event::PointerGone
            } else {
                ruler_pointer(to, false)
            }],
            [1366., 900.],
            1.,
        );
        assert!(app.ruler_drag.is_none());
        if gone {
            assert_eq!(app.session.editor.document, original);
            assert!(app.session.editor.undo_stack.is_empty());
        } else {
            assert_eq!(app.session.editor.undo_stack.len(), 1);
            assert_eq!(
                app.session.editor.document.paragraphs[0]
                    .spacing
                    .right_indent_half_points,
                Some(72)
            );
            app.execute(Action::Undo, &ctx);
            assert_eq!(app.session.editor.document, original);
        }
    }
}

fn ruler_key(key: Key, modifiers: Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}
#[test]
fn ruler_keyboard_preview_steps_tab_commit_escape_and_history() {
    for dpi in [1., 1.25, 1.5] {
        for zoom in [50., 100., 200.] {
            let (ctx, mut app) = setup();
            app.zoom = zoom;
            app.session
                .editor
                .replace(0, 0, "English cafe\u{301}\nУкраїна\nРусский", "Fixture")
                .unwrap();
            app.session.editor.selection = (0, app.session.editor.document.text().chars().count());
            app.sync_view();
            let size = [1366. * dpi, 900. * dpi];
            for _ in 0..3 {
                frame(&ctx, &mut app, vec![], size, dpi);
            }
            let before = app.session.editor.document.clone();
            let n = app.session.editor.undo_stack.len();
            frame(
                &ctx,
                &mut app,
                vec![ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT)],
                size,
                dpi,
            );
            assert_eq!(app.ruler_key_focus, Some(crate::ruler::Handle::First));
            frame(
                &ctx,
                &mut app,
                vec![
                    key(Key::ArrowRight, false),
                    ruler_key(Key::ArrowRight, Modifiers::SHIFT),
                ],
                size,
                dpi,
            );
            assert_eq!(app.session.editor.document, before);
            assert_eq!(
                app.ruler_drag.as_ref().unwrap().draft.preview.paragraphs[0]
                    .spacing
                    .first_line_half_points,
                Some(11)
            );
            frame(&ctx, &mut app, vec![key(Key::Enter, false)], size, dpi);
            assert_eq!(
                app.session.editor.undo_stack.len(),
                n + 1,
                "dpi={dpi} zoom={zoom} error={:?} focus={:?}",
                app.error,
                app.ruler_key_focus
            );
            assert!(
                app.session
                    .editor
                    .document
                    .paragraphs
                    .iter()
                    .all(|p| p.spacing.first_line_half_points == Some(11))
            );
            let committed = app.session.editor.document.clone();
            frame(
                &ctx,
                &mut app,
                vec![key(Key::Tab, false), key(Key::ArrowRight, false)],
                size,
                dpi,
            );
            assert_eq!(app.ruler_key_focus, Some(crate::ruler::Handle::Body));
            frame(&ctx, &mut app, vec![key(Key::Escape, false)], size, dpi);
            assert!(app.ruler_drag.is_none());
            assert!(app.ruler_key_focus.is_none());
            assert_eq!(app.session.editor.document, committed);
            app.execute(Action::Undo, &ctx);
            assert_eq!(app.session.editor.document, before);
            app.execute(Action::Redo, &ctx);
            assert_eq!(app.session.editor.document, committed);
        }
    }
}
#[test]
fn ruler_keyboard_bounds_reverse_tab_and_noop_preserve_redo() {
    let (ctx, mut app) = setup();
    let size = [1366., 900.];
    app.session
        .editor
        .set_paragraph_geometry(
            0,
            0,
            crate::commands::ParagraphGeometryPatch {
                right: Some(Some(288)),
                ..Default::default()
            },
        )
        .unwrap();
    frame(
        &ctx,
        &mut app,
        vec![
            ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT),
            ruler_key(Key::Tab, Modifiers::SHIFT),
        ],
        size,
        1.,
    );
    assert_eq!(app.ruler_key_focus, Some(crate::ruler::Handle::Right));
    let n = app.session.editor.undo_stack.len();
    frame(
        &ctx,
        &mut app,
        vec![
            key(Key::ArrowLeft, false),
            key(Key::ArrowLeft, false),
            key(Key::ArrowRight, false),
            key(Key::Tab, false),
        ],
        size,
        1.,
    );
    assert_eq!(
        app.session.editor.document.paragraphs[0]
            .spacing
            .right_indent_half_points,
        Some(287)
    );
    assert_eq!(app.session.editor.undo_stack.len(), n + 1);
    frame(&ctx, &mut app, vec![key(Key::Escape, false)], size, 1.);
    app.execute(Action::Undo, &ctx);
    frame(
        &ctx,
        &mut app,
        vec![
            ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT),
            key(Key::ArrowRight, false),
            key(Key::ArrowLeft, false),
            key(Key::Enter, false),
        ],
        size,
        1.,
    );
    assert_eq!(app.session.editor.redo_stack.len(), 1);
}
#[test]
fn ruler_keyboard_cancel_on_resize_focus_loss_and_guard_modal() {
    for resize in [false, true] {
        let (ctx, mut app) = setup();
        let size = [1366., 900.];
        let before = app.session.editor.document.clone();
        frame(
            &ctx,
            &mut app,
            vec![
                ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT),
                key(Key::ArrowRight, false),
                egui::Event::Text("ignored".into()),
                key(Key::S, true),
            ],
            size,
            1.,
        );
        assert!(app.ruler_drag.is_some());
        assert_eq!(app.session.editor.document, before);
        if resize {
            frame(&ctx, &mut app, vec![], [1000., 800.], 1.);
        } else {
            let input = egui::RawInput {
                focused: false,
                ..Default::default()
            };
            let _ = ctx.run(input, |ctx| app.update_ui(ctx));
        }
        assert!(app.ruler_drag.is_none());
        assert!(app.ruler_key_focus.is_none());
        assert_eq!(app.session.editor.document, before);
    }
    let (ctx, mut app) = setup();
    app.execute(Action::ParagraphSpacing, &ctx);
    frame(
        &ctx,
        &mut app,
        vec![ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT)],
        [1366., 900.],
        1.,
    );
    assert!(app.ruler_key_focus.is_none());
    assert!(app.paragraph_dialog.is_some());
}

#[test]
fn ruler_keyboard_reveals_hidden_ruler_pointer_takeover_and_value_bounds() {
    let (ctx, mut app) = setup();
    let size = [640., 480.];
    app.zoom = 50.;
    app.rulers = false;
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![], size, 1.);
    }
    frame(
        &ctx,
        &mut app,
        vec![
            ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT),
            key(Key::ArrowRight, false),
        ],
        size,
        1.,
    );
    assert!(app.rulers);
    assert!(app.ruler_drag.is_some());
    let before = app.session.editor.document.clone();
    let pos = app
        .regions
        .iter()
        .find(|r| r.name == "document-text")
        .unwrap()
        .rect
        .center();
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::PointerMoved(pos), ruler_pointer(pos, true)],
        size,
        1.,
    );
    assert!(app.ruler_drag.is_none());
    assert!(app.ruler_key_focus.is_none());
    assert_eq!(app.session.editor.document, before);
    frame(&ctx, &mut app, vec![ruler_pointer(pos, false)], size, 1.);
    let p = &mut app.session.editor.document.paragraphs[0];
    p.spacing.left_indent_half_points = Some(288);
    p.spacing.right_indent_half_points = Some(288);
    p.spacing.first_line_half_points = Some(-288);
    frame(&ctx, &mut app, vec![], size, 1.);
    frame(
        &ctx,
        &mut app,
        vec![ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT)],
        size,
        1.,
    );
    let style = app
        .session
        .editor
        .document
        .resolved_paragraph(&app.session.editor.document.paragraphs[0]);
    let width = ctx.fonts_mut(|f| {
        f.layout_no_wrap(crate::ruler::values(style), FontId::proportional(11.), INK)
            .size()
            .x
    });
    let rect = app
        .regions
        .iter()
        .find(|r| r.name == "ruler-values")
        .unwrap()
        .rect;
    assert!(width <= rect.width(), "{width} > {}", rect.width());
}

#[test]
fn ruler_keyboard_scrolls_focused_marker_into_view_without_cancelling_preview() {
    let (ctx, mut app) = setup();
    let size = [640., 480.];
    app.zoom = 200.;
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![], size, 1.);
    }
    frame(
        &ctx,
        &mut app,
        vec![ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT)],
        size,
        1.,
    );
    frame(
        &ctx,
        &mut app,
        vec![
            ruler_key(Key::Tab, Modifiers::SHIFT),
            key(Key::ArrowLeft, false),
        ],
        size,
        1.,
    );
    for _ in 0..30 {
        frame(&ctx, &mut app, vec![], size, 1.);
    }
    assert_eq!(app.ruler_key_focus, Some(crate::ruler::Handle::Right));
    assert!(app.ruler_drag.is_some());
    let target = app
        .regions
        .iter()
        .find(|r| r.name == "ruler-right")
        .unwrap()
        .rect;
    assert!(
        target.center().x > 0. && target.center().x < 640.,
        "{target:?}"
    );
    frame(&ctx, &mut app, vec![key(Key::Enter, false)], size, 1.);
    assert_eq!(
        app.session.editor.document.paragraphs[0]
            .spacing
            .right_indent_half_points,
        Some(1)
    );
}

#[test]
fn ruler_mixed_markers_follow_selection_preview_cancel_and_history() {
    for dpi in [1., 1.25, 1.5] {
        let (ctx, mut app) = setup();
        let size = [1366. * dpi, 900. * dpi];
        app.session
            .editor
            .replace(0, 0, "English cafe\u{301}\nУкраїна\nРусский", "Fixture")
            .unwrap();
        app.session.editor.document.paragraphs[1]
            .spacing
            .right_indent_half_points = Some(72);
        let first_end = app.session.editor.document.paragraphs[0]
            .text
            .chars()
            .count()
            + 1;
        app.session.editor.selection = (0, first_end);
        app.sync_view();
        for _ in 0..2 {
            frame(&ctx, &mut app, vec![], size, dpi);
        }
        assert!(!app.regions.iter().any(|r| r.name == "ruler-right-mixed"));
        app.execute(Action::SelectAll, &ctx);
        frame(&ctx, &mut app, vec![], size, dpi);
        assert!(app.regions.iter().any(|r| r.name == "ruler-right-mixed"));
        let before = app.session.editor.document.clone();
        let n = app.session.editor.undo_stack.len();
        frame(
            &ctx,
            &mut app,
            vec![
                ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT),
                ruler_key(Key::Tab, Modifiers::SHIFT),
                key(Key::ArrowLeft, false),
            ],
            size,
            dpi,
        );
        assert!(!app.regions.iter().any(|r| r.name == "ruler-right-mixed"));
        assert_eq!(app.session.editor.document, before);
        frame(&ctx, &mut app, vec![key(Key::Escape, false)], size, dpi);
        assert!(app.regions.iter().any(|r| r.name == "ruler-right-mixed"));
        assert_eq!(app.session.editor.undo_stack.len(), n);
        frame(
            &ctx,
            &mut app,
            vec![
                ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT),
                ruler_key(Key::Tab, Modifiers::SHIFT),
                key(Key::ArrowLeft, false),
                key(Key::Enter, false),
                key(Key::F6, false),
            ],
            size,
            dpi,
        );
        assert_eq!(app.session.editor.undo_stack.len(), n + 1);
        app.execute(Action::Undo, &ctx);
        frame(&ctx, &mut app, vec![], size, dpi);
        assert!(app.regions.iter().any(|r| r.name == "ruler-right-mixed"));
        app.execute(Action::Redo, &ctx);
        frame(&ctx, &mut app, vec![], size, dpi);
        assert!(!app.regions.iter().any(|r| r.name == "ruler-right-mixed"));
    }
}

#[test]
fn ruler_mixed_readout_fits_compact_and_does_not_edit() {
    let (ctx, mut app) = setup();
    app.zoom = 50.;
    let size = [640., 480.];
    app.session.editor.replace(0, 0, "a\nb", "Fixture").unwrap();
    let p = &mut app.session.editor.document.paragraphs[1];
    p.spacing.left_indent_half_points = Some(288);
    p.spacing.right_indent_half_points = Some(288);
    p.spacing.first_line_half_points = Some(-288);
    app.execute(Action::SelectAll, &ctx);
    let before = app.session.editor.document.clone();
    let n = app.session.editor.undo_stack.len();
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![], size, 1.);
    }
    assert!(!app.regions.iter().any(|r| r.name == "ruler-values"));
    frame(
        &ctx,
        &mut app,
        vec![ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT)],
        size,
        1.,
    );
    let summary =
        crate::ruler::SelectionValues::read(&before, app.session.editor.selection).unwrap();
    let width = ctx.fonts_mut(|f| {
        f.layout_no_wrap(summary.values(), FontId::proportional(11.), INK)
            .size()
            .x
    });
    assert!(
        width
            <= app
                .regions
                .iter()
                .find(|r| r.name == "ruler-values")
                .unwrap()
                .rect
                .width()
    );
    assert_eq!(app.session.editor.document, before);
    assert_eq!(app.session.editor.undo_stack.len(), n);
}

#[test]
fn ruler_axes_align_with_canvas_at_zoom_and_dpi_and_hide_together() {
    for dpi in [1., 1.25, 1.5] {
        for zoom in [50., 100., 200.] {
            let (ctx, mut app) = setup();
            app.zoom = zoom;
            for _ in 0..3 {
                frame(&ctx, &mut app, vec![], [1366. * dpi, 900. * dpi], dpi);
            }
            let rect = |name: &str| app.regions.iter().find(|r| r.name == name).unwrap().rect;
            let h = rect("ruler");
            let v = rect("ruler-vertical");
            let canvas = rect("continuous-canvas");
            assert_eq!(h.height(), 24.);
            assert_eq!(v.width(), 20.);
            assert!((h.left() - canvas.left()).abs() < 2.);
            assert!((v.top() - canvas.top()).abs() < 1.);
            assert_eq!(v.bottom(), canvas.bottom());
            assert!(v.right() < canvas.left());
            assert!(!app.regions.iter().any(|r| r.name == "ruler-values"));
            let before = app.session.editor.document.clone();
            app.execute(Action::Rulers, &ctx);
            frame(&ctx, &mut app, vec![], [1366. * dpi, 900. * dpi], dpi);
            assert!(!app.regions.iter().any(|r| r.name.starts_with("ruler")));
            assert_eq!(before, app.session.editor.document);
        }
    }
}

#[test]
fn ruler_vertical_tracks_long_canvas_while_horizontal_remains_visible() {
    let (ctx, mut app) = setup();
    let text = "A paragraph for scrolling.\n".repeat(200);
    app.session.editor.replace(0, 0, &text, "Fixture").unwrap();
    app.session.editor.selection = (0, 0);
    app.sync_view();
    for _ in 0..4 {
        frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    }
    let old = app
        .regions
        .iter()
        .find(|r| r.name == "continuous-canvas")
        .unwrap()
        .rect;
    frame(
        &ctx,
        &mut app,
        vec![
            egui::Event::PointerMoved(pos2(900., 600.)),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: vec2(0., -500.),
                modifiers: Modifiers::NONE,
            },
        ],
        [1366., 900.],
        1.,
    );
    for _ in 0..30 {
        frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    }
    let rect = |name: &str| app.regions.iter().find(|r| r.name == name).unwrap().rect;
    let canvas = rect("continuous-canvas");
    let v = rect("ruler-vertical");
    let h = rect("ruler");
    assert!(canvas.top() < old.top() - 100.);
    assert_eq!(v.top(), canvas.top());
    assert!(h.top() > 0. && h.bottom() < 400.);
    assert_eq!(app.session.editor.document.text(), text);
}

#[test]
fn ruler_reset_keyboard_preview_cancel_commit_and_history() {
    let (ctx, mut app) = setup();
    app.session.editor.document.paragraphs[0]
        .spacing
        .left_indent_half_points = Some(80);
    app.session
        .editor
        .document
        .style_catalog
        .normal
        .left_indent_half_points = Some(25);
    let before = app.session.editor.document.clone();
    let n = app.session.editor.undo_stack.len();
    for cancel in [true, false] {
        frame(
            &ctx,
            &mut app,
            vec![
                ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT),
                key(Key::Delete, false),
            ],
            [1366., 900.],
            1.,
        );
        assert!(app.ruler_drag.as_ref().unwrap().draft.resetting);
        assert_eq!(
            app.ruler_drag.as_ref().unwrap().draft.preview.paragraphs[0]
                .spacing
                .left_indent_half_points,
            None
        );
        assert_eq!(app.session.editor.document, before);
        frame(
            &ctx,
            &mut app,
            vec![key(if cancel { Key::Escape } else { Key::Enter }, false)],
            [1366., 900.],
            1.,
        );
        assert!(app.ruler_drag.is_none());
        if cancel {
            assert_eq!(app.session.editor.document, before);
            assert_eq!(app.session.editor.undo_stack.len(), n);
        }
    }
    assert_eq!(app.session.editor.undo_stack.len(), n + 1);
    assert_eq!(
        app.session.editor.document.paragraphs[0]
            .spacing
            .left_indent_half_points,
        None
    );
    frame(&ctx, &mut app, vec![key(Key::F6, false)], [1366., 900.], 1.);
    app.execute(Action::Undo, &ctx);
    assert_eq!(app.session.editor.document, before);
    app.execute(Action::Redo, &ctx);
    assert_eq!(
        app.session.editor.document.paragraphs[0]
            .spacing
            .left_indent_half_points,
        None
    );
}

#[test]
fn ruler_context_menu_reset_starts_cancellable_preview() {
    for menu_name in [
        "ruler-reset",
        "ruler-reset-left",
        "ruler-reset-right",
        "ruler-reset-first",
    ] {
        let (ctx, mut app) = setup();
        app.session.editor.document.paragraphs[0]
            .spacing
            .left_indent_half_points = Some(80);
        let before = app.session.editor.document.clone();
        let rect = app.regions.iter().find(|r| r.name == "ruler").unwrap().rect;
        let pos = rect.center();
        let event = |pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Secondary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(pos), event(true), event(false)],
            [1366., 900.],
            1.,
        );
        frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
        let pos = app
            .regions
            .iter()
            .find(|r| r.name == menu_name)
            .unwrap()
            .rect
            .center();
        frame(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(pos),
                ruler_pointer(pos, true),
                ruler_pointer(pos, false),
            ],
            [1366., 900.],
            1.,
        );
        frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
        assert!(app.ruler_drag.as_ref().unwrap().draft.resetting);
        assert_eq!(app.session.editor.document, before);
        frame(
            &ctx,
            &mut app,
            vec![key(Key::Escape, false)],
            [1366., 900.],
            1.,
        );
        assert!(app.ruler_drag.is_none());
        assert_eq!(app.session.editor.document, before);
    }
}

#[test]
fn ruler_shift_delete_resets_only_focused_property_with_cancel_and_history() {
    use crate::ruler::{Handle, ResetIndent};
    for (tabs, handle) in [
        (0, Handle::First),
        (1, Handle::Body),
        (2, Handle::Left),
        (3, Handle::Right),
    ] {
        let (ctx, mut app) = setup();
        let p = &mut app.session.editor.document.paragraphs[0];
        p.spacing.left_indent_half_points = Some(80);
        p.spacing.right_indent_half_points = Some(60);
        p.spacing.first_line_half_points = Some(-40);
        let before = app.session.editor.document.clone();
        let n = app.session.editor.undo_stack.len();
        for cancel in [true, false] {
            let mut events = vec![ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT)];
            events.extend((0..tabs).map(|_| key(Key::Tab, false)));
            events.push(ruler_key(Key::Delete, Modifiers::SHIFT));
            frame(&ctx, &mut app, events, [1366., 900.], 1.);
            let draft = &app.ruler_drag.as_ref().unwrap().draft;
            assert_eq!(draft.reset_scope, ResetIndent::for_handle(handle));
            let expected = draft.preview.clone();
            assert_eq!(app.session.editor.document, before);
            frame(
                &ctx,
                &mut app,
                vec![key(if cancel { Key::Escape } else { Key::Enter }, false)],
                [1366., 900.],
                1.,
            );
            assert_eq!(
                app.session.editor.document,
                if cancel { before.clone() } else { expected }
            );
        }
        assert_eq!(app.session.editor.undo_stack.len(), n + 1);
        frame(&ctx, &mut app, vec![key(Key::F6, false)], [1366., 900.], 1.);
        let after = app.session.editor.document.clone();
        app.execute(Action::Undo, &ctx);
        assert_eq!(app.session.editor.document, before);
        app.execute(Action::Redo, &ctx);
        assert_eq!(app.session.editor.document, after);
    }
}

#[test]
fn ruler_hover_displays_provenance_without_editing() {
    fn has_source(shape: &egui::Shape) -> bool {
        match shape {
            egui::Shape::Text(t) => t.galley.job.text.contains("Direct paragraph formatting"),
            egui::Shape::Vec(shapes) => shapes.iter().any(has_source),
            _ => false,
        }
    }
    let (ctx, mut app) = setup();
    ctx.style_mut(|s| {
        s.interaction.tooltip_delay = 0.;
        s.interaction.tooltip_grace_time = 0.;
    });
    app.session.editor.document.paragraphs[0]
        .spacing
        .right_indent_half_points = Some(24);
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![], [1366., 900.], 1.);
    }
    let pos = app
        .regions
        .iter()
        .find(|r| r.name == "ruler-right")
        .unwrap()
        .rect
        .center();
    let before = app.session.editor.document.clone();
    let n = app.session.editor.undo_stack.len();
    let mut found = false;
    for i in 0..5 {
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366., 900.))),
                time: Some(10. + i as f64),
                events: vec![egui::Event::PointerMoved(pos)],
                ..Default::default()
            },
            |ctx| app.update_ui(ctx),
        );
        found |= output.shapes.iter().any(|s| has_source(&s.shape));
    }
    assert!(found, "Ruler tooltip must render the source");
    assert_eq!(app.session.editor.document, before);
    assert_eq!(app.session.editor.undo_stack.len(), n);
}

#[test]
fn ruler_sources_keyboard_reads_each_marker_and_returns_without_edits() {
    for tabs in 0..4 {
        let (ctx, mut app) = setup();
        let before = app.session.editor.document.clone();
        let n = app.session.editor.undo_stack.len();
        let mut keys = vec![ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT)];
        keys.extend((0..tabs).map(|_| key(Key::Tab, false)));
        keys.push(key(Key::F1, false));
        frame(&ctx, &mut app, keys, [1366., 900.], 1.);
        let focused = app.ruler_key_focus;
        assert!(app.ruler_sources);
        assert!(app.regions.iter().any(|r| r.name == "ruler-sources"));
        frame(
            &ctx,
            &mut app,
            vec![
                key(Key::Tab, false),
                key(Key::ArrowRight, false),
                key(Key::Delete, false),
                egui::Event::Text("blocked".into()),
            ],
            [1366., 900.],
            1.,
        );
        assert_eq!(app.ruler_key_focus, focused);
        assert!(app.ruler_drag.is_none());
        assert_eq!(app.session.editor.document, before);
        assert_eq!(app.session.editor.undo_stack.len(), n);
        frame(
            &ctx,
            &mut app,
            vec![key(Key::Escape, false)],
            [1366., 900.],
            1.,
        );
        assert!(!app.ruler_sources);
        assert_eq!(app.ruler_key_focus, focused);
        frame(
            &ctx,
            &mut app,
            vec![key(Key::Escape, false)],
            [1366., 900.],
            1.,
        );
        assert!(app.ruler_key_focus.is_none());
    }
}

#[test]
fn ruler_sources_dismissal_preserves_preview_until_separate_commit_or_cancel() {
    for dismiss in [Key::Escape, Key::Enter, Key::F1] {
        let (ctx, mut app) = setup();
        let before = app.session.editor.document.clone();
        let n = app.session.editor.undo_stack.len();
        frame(
            &ctx,
            &mut app,
            vec![
                ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT),
                key(Key::ArrowRight, false),
                key(Key::F1, false),
            ],
            [1366., 900.],
            1.,
        );
        let preview = app.ruler_drag.as_ref().unwrap().draft.preview.clone();
        frame(
            &ctx,
            &mut app,
            vec![
                key(Key::ArrowDown, false),
                key(Key::End, false),
                key(Key::PageUp, false),
            ],
            [1366., 900.],
            1.,
        );
        frame(&ctx, &mut app, vec![key(dismiss, false)], [1366., 900.], 1.);
        assert!(!app.ruler_sources);
        assert_eq!(app.ruler_drag.as_ref().unwrap().draft.preview, preview);
        assert_eq!(app.session.editor.document, before);
        assert_eq!(app.session.editor.undo_stack.len(), n);
        frame(
            &ctx,
            &mut app,
            vec![key(Key::Enter, false)],
            [1366., 900.],
            1.,
        );
        assert_eq!(app.session.editor.document, preview);
        assert_eq!(app.session.editor.undo_stack.len(), n + 1);
    }
}

#[test]
fn ruler_sources_compact_bounds_and_focus_loss_cleanup() {
    for dpi in [1., 1.25, 1.5] {
        let (ctx, mut app) = setup();
        app.zoom = 50.;
        let size = [640. * dpi, 480. * dpi];
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], size, dpi);
        }
        frame(
            &ctx,
            &mut app,
            vec![
                ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT),
                key(Key::F1, false),
            ],
            size,
            dpi,
        );
        let r = app
            .regions
            .iter()
            .find(|r| r.name == "ruler-sources")
            .unwrap()
            .rect;
        assert!(
            r.left() >= 0. && r.right() <= 640. && r.bottom() <= 480.,
            "{r:?}"
        );
        let _ = ctx.run(
            egui::RawInput {
                focused: false,
                ..Default::default()
            },
            |ctx| app.update_ui(ctx),
        );
        assert!(!app.ruler_sources);
        assert!(app.ruler_key_focus.is_none());
    }
}

#[test]
fn full_ruler_sources_scroll_clamp_and_reopen_without_editing() {
    for dpi in [1., 1.25, 1.5] {
        let (ctx, mut app) = setup();
        app.zoom = 50.;
        let text = vec!["Sample"; 32].join("\n");
        app.session.editor.replace(0, 0, &text, "Fixture").unwrap();
        for i in 0..32 {
            let id = app
                .session
                .editor
                .create_style(
                    &format!("Source {i:02} {}", "W".repeat(40)),
                    crate::model::StyleDefinition {
                        left_indent_half_points: Some(i as u16),
                        ..Default::default()
                    },
                    false,
                )
                .unwrap();
            app.session.editor.document.paragraphs[i].paragraph_style = id;
        }
        app.session.editor.selection = (0, text.chars().count());
        app.sync_view();
        let size = [640. * dpi, 480. * dpi];
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![], size, dpi);
        }
        let before = app.session.editor.document.clone();
        let n = app.session.editor.undo_stack.len();
        frame(
            &ctx,
            &mut app,
            vec![
                ruler_key(Key::F8, Modifiers::CTRL | Modifiers::SHIFT),
                key(Key::F1, false),
            ],
            size,
            dpi,
        );
        assert_eq!(app.ruler_sources_scroll, 0.);
        frame(&ctx, &mut app, vec![key(Key::PageDown, false)], size, dpi);
        assert!(app.ruler_sources_scroll > 0.);
        frame(&ctx, &mut app, vec![key(Key::End, false)], size, dpi);
        let end = app.ruler_sources_scroll;
        assert!(end > app.ruler_sources_height);
        frame(&ctx, &mut app, vec![key(Key::ArrowDown, false)], size, dpi);
        assert_eq!(app.ruler_sources_scroll, end);
        frame(&ctx, &mut app, vec![key(Key::PageUp, false)], size, dpi);
        assert!(app.ruler_sources_scroll < end);
        frame(
            &ctx,
            &mut app,
            vec![key(Key::Home, false), key(Key::ArrowUp, false)],
            size,
            dpi,
        );
        assert_eq!(app.ruler_sources_scroll, 0.);
        let r = app
            .regions
            .iter()
            .find(|r| r.name == "ruler-sources")
            .unwrap()
            .rect;
        assert!(r.right() <= 640. && r.bottom() <= 480., "{r:?}");
        frame(&ctx, &mut app, vec![key(Key::End, false)], size, dpi);
        frame(&ctx, &mut app, vec![key(Key::Escape, false)], size, dpi);
        frame(&ctx, &mut app, vec![key(Key::F1, false)], size, dpi);
        assert_eq!(app.ruler_sources_scroll, 0.);
        assert_eq!(app.session.editor.document, before);
        assert_eq!(app.session.editor.undo_stack.len(), n);
        assert!(app.ruler_drag.is_none());
    }
}
