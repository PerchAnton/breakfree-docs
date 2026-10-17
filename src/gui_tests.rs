//! Event-loop tests on the actual Rust widgets, without a native window or OS clipboard.
use super::*;
use crate::model::{Document, Style};
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
            .all(|s| s.bold)
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
            Style::default(),
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
    assert!(app.session.editor.document.paragraphs[0].styles[0].bold);
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
            .all(|s| s.bold)
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
            .all(|p| p.alignment == Alignment::Right && p.styles.iter().all(|s| s.italic))
    );
    click_region(&ctx, &mut app, "unavailable-Strikethrough");
    click_region(&ctx, &mut app, "style-Heading 1");
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
            .all(|s| s.underline && s.font_size.to_string() == "24.5")
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
    assert!(pending.underline);
    assert_eq!(pending.font_size.to_string(), "18");
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
                Style::default(),
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
            .all(|s| s.font_size.to_string() == "32")
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
        assert_eq!(after.paragraphs[0].styles[0].font_size.to_string(), "36");
        assert_eq!(after.paragraphs[0].styles[5].font_size.to_string(), "12");
        assert!(after.paragraphs[0].styles.iter().all(|s| s.underline));
        app.execute(Action::Undo, &ctx);
        assert_eq!(app.session.editor.document, before);
        app.execute(Action::Redo, &ctx);
        assert_eq!(app.session.editor.document, after);
        click_region_at(&ctx, &mut app, "command-ShrinkFont", [width, 900.]);
        assert_eq!(app.session.editor.document, before);
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
                assert!(b.underline);
                assert_eq!(b.font_size.to_string(), "19.5");
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
            app.session.editor.typing_style.font_size,
            FontSize::default()
        );
        frame(&ctx, &mut app, vec![key(Key::Enter, false)], screen, dpi);
        assert!(app.font_dialog.is_none());
        assert_eq!(app.session.editor.typing_style.font_size.to_string(), "400");
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
    // Move Font -> Paragraph -> Editing; a narrow window moves Editing into More.
    for _ in 0..2 {
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
    ribbon_key(&ctx, &mut app, Key::Enter, Modifiers::NONE, [640., 480.]);
    assert!(app.navigation);
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
    assert_eq!(app.session.editor.typing_style.font_size.to_string(), "11");
    edit_size(&ctx, &mut app, "19.5", Key::Enter);
    assert!(app.ribbon_collapsed && !app.ribbon_temporary);
    assert_eq!(
        app.session.editor.typing_style.font_size.to_string(),
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
            .all(|s| s.bold)
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
        app.session.editor.typing_style.font_size,
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
    assert!(!app.session.editor.typing_style.bold);
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
        assert!(!app.session.editor.document.paragraphs[0].styles[0].bold);
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
    assert!(!app.session.editor.typing_style.bold);
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
    assert!(after.paragraphs[0].styles[0].bold);
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
    assert!(after.paragraphs[0].styles[0].bold);
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
                    && p.styles.iter().all(|s| *s == Style::default()))
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
    assert!(app.session.editor.typing_style.bold);
    click_region(&ctx, &mut app, "command-ClearFormatting");
    assert_eq!(app.session.editor.typing_style, Style::default());
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
            .all(|s| *s == Style::default())
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
    assert!(app.session.editor.typing_style.bold);
    click_region(&ctx, &mut app, "command-ClearFormatting");
    assert!(app.session.editor.typing_style.bold);
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
    );
}
