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
    app.find_next();
    assert_eq!(
        selected_text(&app.session.editor.document, app.session.editor.selection),
        "Україна"
    );
    let before = app.session.editor.document.clone();
    app.finish(Pending::Open(PathBuf::from("/missing.bfdoc")), &ctx);
    assert_eq!(app.session.editor.document, before);
    assert!(app.error.is_some());
}

fn click_region(ctx: &egui::Context, app: &mut DocsApp, name: &str) {
    for _ in 0..2 {
        frame(ctx, app, vec![], [1366., 900.], 1.);
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
            [1366., 900.],
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
    click_region(&ctx, &mut app, "unavailable-Underline");
    click_region(&ctx, &mut app, "style-Heading 1");
    assert_eq!(app.session.editor.document, styled);
    click_region(&ctx, &mut app, "command-Undo");
    assert!(
        app.session
            .editor
            .document
            .paragraphs
            .iter()
            .all(|p| p.alignment == Alignment::Left)
    );
    click_region(&ctx, &mut app, "command-Redo");
    assert_eq!(app.session.editor.document, styled);
    click_region(&ctx, &mut app, "zoom-in");
    assert_eq!(app.zoom, 110.);
    click_region(&ctx, &mut app, "zoom-out");
    assert_eq!(app.zoom, 100.);
    app.zoom = 150.;
    click_region(&ctx, &mut app, "zoom-reset");
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
