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
    click_region(&ctx, &mut app, "home-menu-Find");
    click_region(&ctx, &mut app, "command-Find");
    assert!(app.navigation);
    assert!(ctx.memory(|m| m.has_focus(Id::new("find-text"))));
    assert_eq!(app.session.editor.document, before);
    assert_eq!(app.session.editor.undo_stack.len(), history);
    click_region_at(&ctx, &mut app, "home-overflow", [640., 900.]);
    click_region_at(&ctx, &mut app, "command-Replace", [640., 900.]);
    assert!(app.replace_mode);
    assert_eq!(app.session.editor.document, before);
}
