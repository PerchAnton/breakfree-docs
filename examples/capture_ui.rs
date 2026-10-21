//! Development-only native screenshot harness; never part of the editor CLI.
use breakfree_docs::{
    gui::{Action, DocsApp},
    session::Session,
    storage,
};
use eframe::egui;

struct Capture {
    app: DocsApp,
    path: String,
    frames: usize,
    mode: String,
    started: Option<f64>,
    requested: bool,
    _preferences_dir: Option<tempfile::TempDir>,
}

impl eframe::App for Capture {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, input: &mut egui::RawInput) {
        if matches!(
            self.mode.as_str(),
            "tab-keyboard"
                | "tab-delete"
                | "tab-mixed"
                | "tab-create"
                | "tab-numeric"
                | "tab-create-center"
                | "tab-create-right"
        ) {
            input.focused = true;
            if matches!(self.frames, 3 | 5) {
                input.events.push(egui::Event::Key {
                    key: if self.frames == 3 {
                        egui::Key::F8
                    } else {
                        if self.mode == "tab-delete" {
                            egui::Key::Delete
                        } else {
                            egui::Key::ArrowRight
                        }
                    },
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: if self.frames == 3 {
                        egui::Modifiers::CTRL
                            | egui::Modifiers::ALT
                            | if matches!(
                                self.mode.as_str(),
                                "tab-create"
                                    | "tab-numeric"
                                    | "tab-create-center"
                                    | "tab-create-right"
                            ) {
                                egui::Modifiers::SHIFT
                            } else {
                                egui::Modifiers::NONE
                            }
                    } else {
                        egui::Modifiers::SHIFT
                    },
                });
            }
        }
        if matches!(self.mode.as_str(), "tab-create-center" | "tab-create-right")
            && (self.frames == 1 || (self.mode == "tab-create-right" && self.frames == 2))
        {
            input.events.push(egui::Event::Key {
                key: egui::Key::F7,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::CTRL | egui::Modifiers::ALT,
            });
        }
        if self.mode == "tab-numeric" {
            if self.frames == 7 {
                input.events.push(egui::Event::Key {
                    key: egui::Key::F2,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                });
            }
            if self.frames == 9 {
                input.events.push(egui::Event::Text("48.5".into()));
            }
        }
        if self.mode == "tab-mixed" && self.frames == 3 {
            self.app.execute(Action::SelectAll, _ctx);
        }
        // Deterministic render fixture, including focus; this does not prove OS input.
        if self.mode == "ruler-sources" {
            input.focused = true;
            if let Some(r) = self.app.regions.iter().find(|r| r.name == "ruler-first") {
                input
                    .events
                    .push(egui::Event::PointerMoved(r.rect.center()));
            }
        }
        if self.mode.starts_with("keytips")
            || matches!(
                self.mode.as_str(),
                "ruler-keyboard"
                    | "ruler-reset"
                    | "ruler-reset-single"
                    | "ruler-source-keyboard"
                    | "ruler-source-list"
                    | "ruler-source-wheel"
                    | "ruler-source-menu"
                    | "ruler-source-context"
                    | "ruler-keyboard-menu"
            )
            || self.mode == "temporary"
            || self.mode == "replace-all"
            || self.mode.starts_with("styles")
            || self.mode.starts_with("clear-formatting")
            || self.mode.starts_with("navigation")
        {
            input.focused = true;
        }
        if matches!(
            self.mode.as_str(),
            "ruler-keyboard"
                | "ruler-reset"
                | "ruler-reset-single"
                | "ruler-source-keyboard"
                | "ruler-source-list"
                | "ruler-source-wheel"
                | "ruler-source-menu"
                | "ruler-source-context"
                | "ruler-keyboard-menu"
        ) && !matches!(
            self.mode.as_str(),
            "ruler-source-menu" | "ruler-source-context" | "ruler-keyboard-menu"
        ) && matches!(self.frames, 3 | 5)
        {
            let (key, modifiers) = if self.frames == 3 {
                (
                    egui::Key::F8,
                    egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                )
            } else {
                if matches!(
                    self.mode.as_str(),
                    "ruler-source-keyboard"
                        | "ruler-source-list"
                        | "ruler-source-wheel"
                        | "ruler-source-menu"
                        | "ruler-source-context"
                        | "ruler-keyboard-menu"
                ) {
                    (egui::Key::F1, egui::Modifiers::NONE)
                } else if self.mode == "ruler-reset-single" {
                    (egui::Key::Delete, egui::Modifiers::SHIFT)
                } else if self.mode == "ruler-reset" {
                    (egui::Key::Delete, egui::Modifiers::NONE)
                } else {
                    (egui::Key::ArrowRight, egui::Modifiers::SHIFT)
                }
            };
            for pressed in [true, false] {
                input.events.push(egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers,
                });
            }
        }
        if self.mode == "ruler-source-list" && self.frames == 7 {
            input.events.push(egui::Event::Key {
                key: egui::Key::End,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            });
        }
        if self.mode == "ruler-source-wheel"
            && self.frames == 7
            && let Some(r) = self.app.regions.iter().find(|r| r.name == "ruler-sources")
        {
            input
                .events
                .push(egui::Event::PointerMoved(r.rect.center()));
            input.events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Line,
                delta: egui::vec2(0., -40.),
                modifiers: egui::Modifiers::NONE,
            });
        }
        if matches!(
            self.mode.as_str(),
            "ruler-source-menu" | "ruler-source-context"
        ) {
            let target = if self.frames == 3 {
                Some(("ruler", egui::PointerButton::Secondary))
            } else if self.frames == 6 && self.mode == "ruler-source-context" {
                Some(("ruler-sources-first", egui::PointerButton::Primary))
            } else {
                None
            };
            if let Some((name, button)) = target
                && let Some(r) = self.app.regions.iter().find(|r| r.name == name)
            {
                let pos = r.rect.center();
                input.events.push(egui::Event::PointerMoved(pos));
                for pressed in [true, false] {
                    input.events.push(egui::Event::PointerButton {
                        pos,
                        button,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
            }
        }
        if self.mode == "ruler-keyboard-menu" && matches!(self.frames, 3 | 5) {
            let (key, modifiers) = if self.frames == 3 {
                (
                    egui::Key::F8,
                    egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                )
            } else {
                (egui::Key::F10, egui::Modifiers::SHIFT)
            };
            for pressed in [true, false] {
                input.events.push(egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers,
                });
            }
        }
        if self.frames == 2 && self.mode.starts_with("keytips") {
            let mut keys = vec![egui::Key::F10];
            match self.mode.as_str() {
                "keytips-qat"
                | "keytips-qat-below"
                | "keytips-settings"
                | "keytips-settings-reset"
                | "keytips-settings-order" => keys.push(egui::Key::Num0),
                "keytips-home" => keys.push(egui::Key::H),
                "keytips-prefix" => keys.extend([egui::Key::H, egui::Key::F]),
                "keytips-tabs" => keys.push(egui::Key::T),
                "keytips-view" => keys.push(egui::Key::W),
                "keytips-zoom" => keys.extend([egui::Key::W, egui::Key::Q]),
                _ => {}
            }
            for key in keys {
                for pressed in [true, false] {
                    input.events.push(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
            }
        }
        if self.frames == 2
            && ["styles-create", "styles-rename", "styles-character-create"]
                .contains(&self.mode.as_str())
        {
            input.events.push(egui::Event::Text("Report Note".into()));
        }
        if self.mode == "styles-menu" {
            if self.frames == 2
                && let Some(region) = self
                    .app
                    .regions
                    .iter()
                    .find(|r| r.name == "home-menu-Styles" || r.name == "home-overflow")
            {
                let pos = region.rect.center();
                input.events.push(egui::Event::PointerMoved(pos));
                for pressed in [true, false] {
                    input.events.push(egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
            }
            if self.frames == 4 {
                for pressed in [true, false] {
                    input.events.push(egui::Event::Key {
                        key: egui::Key::F3,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
            }
            if self.frames == 6
                && let Some(region) = self
                    .app
                    .regions
                    .iter()
                    .find(|r| r.name == "style-filter-used")
            {
                let pos = region.rect.center();
                input.events.push(egui::Event::PointerMoved(pos));
                for pressed in [true, false] {
                    input.events.push(egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
            }
        }
        if self.mode == "styles-menu"
            && self.frames == 8
            && let Some(region) = self
                .app
                .regions
                .iter()
                .find(|r| r.name == "style-jump-Style(Normal)-Style(Heading1)")
        {
            let pos = region.rect.center();
            input.events.push(egui::Event::PointerMoved(pos));
            for pressed in [true, false] {
                input.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                });
            }
        }
        if self.frames == 2 && self.mode.starts_with("navigation") {
            input.events.push(egui::Event::Text(
                if self.mode == "navigation-options" {
                    "breakfree"
                } else {
                    "BreakFree"
                }
                .into(),
            ));
        }
        if self.frames == 2 && self.mode.starts_with("clear-formatting") {
            let mut keys = vec![egui::Key::F10, egui::Key::H, egui::Key::F];
            if self.mode == "clear-formatting" {
                keys.push(egui::Key::C);
            }
            for key in keys {
                for pressed in [true, false] {
                    input.events.push(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
            }
        }
        if self.mode == "replace-all" {
            if self.frames == 2 {
                input.events.push(egui::Event::Text("BreakFree".into()));
            }
            if self.frames == 10 {
                input.events.push(egui::Event::Text("Local".into()));
            }
            let key = match self.frames {
                4..=9 | 11 | 12 => Some(egui::Key::Tab),
                13 => Some(egui::Key::Enter),
                _ => None,
            };
            if let Some(key) = key {
                for pressed in [true, false] {
                    input.events.push(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
            }
        }
        if self.mode == "navigation-options" {
            let key = match self.frames {
                4 | 5 | 7 => Some(egui::Key::Tab),
                6 | 8 => Some(egui::Key::Space),
                _ => None,
            };
            if let Some(key) = key {
                for pressed in [true, false] {
                    input.events.push(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
            }
        }
        if self.mode == "keytips-qat-below" {
            let key = match self.frames {
                4..=8 => Some(egui::Key::ArrowDown),
                10 => Some(egui::Key::Enter),
                _ => None,
            };
            if let Some(key) = key {
                for pressed in [true, false] {
                    input.events.push(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
            }
        }
        if self.mode == "keytips-settings-reset" || self.mode == "keytips-settings-order" {
            let frame = self.frames
                - usize::from(self.mode == "keytips-settings-order" && self.frames >= 10);
            let key = match frame {
                4..=9 => Some(egui::Key::ArrowDown),
                11 => Some(egui::Key::Enter),
                _ => None,
            };
            if let Some(key) = key {
                for pressed in [true, false] {
                    input.events.push(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
            }
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.mode == "ruler-sources" {
            ctx.style_mut(|s| s.interaction.tooltip_delay = 0.);
        }
        self.app.update_ui(ctx);
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                let rgba: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                image::save_buffer(
                    &self.path,
                    &rgba,
                    image.width() as u32,
                    image.height() as u32,
                    image::ColorType::Rgba8,
                )
                .expect("Save native capture");
                // Exit only this development harness; the editor correctly blocks
                // ordinary window-close requests while its Font draft is open.
                std::process::exit(0);
            }
        }
        // Allow both measurement passes and popup fade-in to settle. Frame count
        // alone can race the animation on a fast native renderer.
        self.frames += 1;
        let now = ctx.input(|i| i.time);
        let started = *self.started.get_or_insert(now);
        if self.frames >= 16 && now - started >= 0.4 && !self.requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            self.requested = true;
        }
        if !self.requested {
            ctx.request_repaint();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3
        || ![
            "home",
            "tabs",
            "explicit-tabs",
            "aligned-tabs",
            "tab-leaders",
            "typed-tabs-dialog",
            "tab-keyboard",
            "tab-delete",
            "tab-mixed",
            "tab-create",
            "tab-create-center",
            "tab-create-right",
            "tab-numeric",
            "tab-positions",
            "tab-dialog",
            "styles",
            "styles-menu",
            "styles-character",
            "styles-character-create",
            "styles-character-custom",
            "styles-character-modify",
            "styles-character-delete",
            "styles-modify",
            "styles-following",
            "styles-create",
            "styles-rename",
            "styles-delete",
            "file",
            "navigation",
            "navigation-options",
            "replace-all",
            "clear-formatting",
            "clear-formatting-tips",
            "formatting",
            "font",
            "paragraph",
            "indent-layout",
            "ruler-keyboard",
            "ruler-mixed",
            "ruler-reset",
            "ruler-reset-single",
            "ruler-sources",
            "ruler-source-keyboard",
            "ruler-source-list",
            "ruler-source-wheel",
            "ruler-source-menu",
            "ruler-source-context",
            "ruler-keyboard-menu",
            "collapsed",
            "temporary",
            "view",
            "view-fit",
            "keytips-qat",
            "keytips-qat-below",
            "keytips-settings",
            "keytips-settings-reset",
            "keytips-settings-order",
            "settings-error",
            "keytips-root",
            "keytips-home",
            "keytips-prefix",
            "keytips-tabs",
            "keytips-view",
            "keytips-zoom",
        ]
        .contains(&args[2].as_str())
    {
        return Err("Usage: capture_ui <width-DIP> <height-DIP> <home|tabs|styles|styles-menu|styles-modify|styles-following|styles-create|styles-rename|styles-delete|view|view-fit|keytips-qat|keytips-qat-below|file|navigation|navigation-options|replace-all|clear-formatting|clear-formatting-tips|formatting|font|paragraph|indent-layout|ruler-keyboard|ruler-mixed|ruler-reset|ruler-reset-single|ruler-sources|ruler-source-keyboard|ruler-source-list|ruler-source-wheel|ruler-source-menu|ruler-source-context|ruler-keyboard-menu|collapsed|temporary|keytips-root|keytips-home|keytips-prefix|keytips-tabs|keytips-view|keytips-zoom>; set BREAKFREE_CAPTURE_TO".into());
    }
    let path = std::env::var("BREAKFREE_CAPTURE_TO")
        .map_err(|_| "Set BREAKFREE_CAPTURE_TO to an output PNG path")?;
    if std::env::var_os("EFRAME_SCREENSHOT_TO").is_some() {
        return Err("Unset EFRAME_SCREENSHOT_TO; it captures before modal layout settles".into());
    }
    let width: f32 = args[0].parse()?;
    let height: f32 = args[1].parse()?;
    if !(640.0..=1920.0).contains(&width) || !(480.0..=1080.0).contains(&height) {
        return Err("Capture dimensions outside 640–1920 × 480–1080 DIP".into());
    }
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        if [
            "styles-character-custom",
            "styles-character-modify",
            "styles-character-delete",
        ]
        .contains(&args[2].as_str())
        {
            "examples/NamedCharacterStyles.bfdoc"
        } else if args[2] == "styles-character" || args[2] == "styles-character-create" {
            "examples/CharacterStyles.bfdoc"
        } else if args[2] == "styles-following" {
            "examples/FollowingStyles.bfdoc"
        } else if args[2] == "styles-delete" {
            "examples/StyleDeletion.bfdoc"
        } else if ["styles-create", "styles-rename"].contains(&args[2].as_str()) {
            "examples/CustomStyles.bfdoc"
        } else if args[2].starts_with("styles") {
            "examples/Styles.bfdoc"
        } else if ["formatting", "font"].contains(&args[2].as_str()) {
            "examples/Formatting.bfdoc"
        } else if args[2] == "tab-leaders" {
            "examples/TabLeaders.bfdoc"
        } else if matches!(args[2].as_str(), "aligned-tabs" | "typed-tabs-dialog") {
            "examples/AlignedTabs.bfdoc"
        } else {
            "examples/Welcome.bfdoc"
        },
    );
    let session = if matches!(
        args[2].as_str(),
        "tabs"
            | "tab-dialog"
            | "explicit-tabs"
            | "tab-keyboard"
            | "tab-delete"
            | "tab-mixed"
            | "tab-create"
            | "tab-create-center"
            | "tab-create-right"
            | "tab-numeric"
            | "tab-positions"
    ) {
        let mut session = Session::default();
        session.editor.replace(0, 0, "Name\t\tValue\nEnglish\t\t36 pt grid\nУкраїна\t\tТабуляція\nРусский\t\tТабуляция\nCombining e\u{301}\t\tEnd", "Tab fixture")?;
        if matches!(
            args[2].as_str(),
            "explicit-tabs"
                | "tab-keyboard"
                | "tab-delete"
                | "tab-mixed"
                | "tab-create"
                | "tab-create-center"
                | "tab-create-right"
                | "tab-numeric"
                | "tab-positions"
        ) {
            let end = session.editor.document.text().chars().count();
            session
                .editor
                .set_left_tab_stops(0, end, vec![144, 288, 432])?;
        }
        if args[2] == "tab-mixed" {
            session.editor.set_left_tab_stops(0, 0, vec![])?;
            let start = session.editor.document.text().find("Україна").unwrap();
            session
                .editor
                .set_left_tab_stops(start, start, vec![180, 288])?;
            session.editor.selection = (0, session.editor.document.text().chars().count());
        }
        session
    } else if matches!(
        args[2].as_str(),
        "indent-layout"
            | "ruler-keyboard"
            | "ruler-mixed"
            | "ruler-reset"
            | "ruler-reset-single"
            | "ruler-sources"
            | "ruler-source-keyboard"
            | "ruler-source-list"
            | "ruler-source-wheel"
            | "ruler-source-menu"
            | "ruler-source-context"
            | "ruler-keyboard-menu"
    ) {
        let mut session = Session::default();
        let sample =
            "English Україна Русский cafe\u{301}. Text wraps without changing stored characters.  ";
        let text = format!(
            "First line. {}\nHanging. {}\nJustified. {}",
            sample.repeat(3),
            sample.repeat(3),
            sample.repeat(3)
        );
        session.editor.replace(0, 0, &text, "Layout fixture")?;
        for (index, p) in session.editor.document.paragraphs.iter_mut().enumerate() {
            p.spacing.left_indent_half_points = Some(36);
            p.spacing.right_indent_half_points = Some(24);
            p.spacing.first_line_half_points = Some(if index == 0 { 72 } else { -72 });
            p.alignment = if index == 2 {
                breakfree_docs::model::Alignment::Justify
            } else {
                breakfree_docs::model::Alignment::Left
            };
        }
        if matches!(
            args[2].as_str(),
            "ruler-sources"
                | "ruler-source-keyboard"
                | "ruler-source-list"
                | "ruler-source-wheel"
                | "ruler-source-menu"
                | "ruler-source-context"
                | "ruler-keyboard-menu"
        ) {
            session
                .editor
                .document
                .style_catalog
                .normal
                .left_indent_half_points = Some(36);
            session.editor.document.paragraphs[0]
                .spacing
                .left_indent_half_points = None;
            session.editor.document.paragraphs[0].paragraph_style =
                breakfree_docs::model::ParagraphStyle::Heading1;
        }
        if matches!(
            args[2].as_str(),
            "ruler-source-list"
                | "ruler-source-wheel"
                | "ruler-source-menu"
                | "ruler-source-context"
                | "ruler-keyboard-menu"
        ) {
            session = Session::default();
            session.editor.replace(
                0,
                0,
                &vec!["Source inspection fixture"; 32].join("\n"),
                "Fixture",
            )?;
            for i in 0..32 {
                let id = session.editor.create_style(
                    &format!("Source {i:02} — inherited paragraph geometry"),
                    breakfree_docs::model::StyleDefinition {
                        left_indent_half_points: Some(i as u16),
                        ..Default::default()
                    },
                    false,
                )?;
                session.editor.document.paragraphs[i].paragraph_style = id;
            }
        }
        session.editor.document.validate()?;
        session.editor.selection = (0, 0);
        session
    } else if args[2] == "styles-menu" {
        let mut session = Session::default();
        session.editor.apply_paragraph_style(
            0,
            0,
            breakfree_docs::model::ParagraphStyle::Heading1,
        )?;
        session
    } else if args[2] == "replace-all" {
        let mut session = Session::default();
        session.editor.replace(0, 0, "BreakFree Docs\nBreakFree writes locally.\nBreakFree preserves Unicode: Україна Русский e\u{301}.", "Fixture")?;
        session.editor.emphasize(0, 9, true, true)?;
        session
    } else if args[2].starts_with("clear-formatting") {
        Session::new(
            storage::load(std::path::Path::new("examples/Formatting.bfdoc"))?,
            None,
        )
    } else if args[2] == "file" {
        Session::default()
    } else {
        Session::new(storage::load(&fixture)?, Some(fixture))
    };
    let mode = args[2].clone();
    eframe::run_native(
        "BreakFree Docs capture",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([width, height])
                .with_decorations(false),
            ..Default::default()
        },
        Box::new(move |cc| {
            let preferences_dir = mode
                .contains("settings")
                .then(|| tempfile::tempdir().expect("Isolated capture preferences"));
            let mut app = if let Some(dir) = &preferences_dir {
                let path = dir.path().join("settings.json");
                if mode == "settings-error" {
                    std::fs::write(&path, b"{broken settings").expect("Error fixture");
                } else {
                    use breakfree_docs::preferences::{Placement, QuickCommand, Store, Toolbar};
                    Store::open(Ok(path.clone()))
                        .0
                        .save(&Toolbar {
                            commands: if mode == "keytips-settings-order" {
                                vec![
                                    QuickCommand::New,
                                    QuickCommand::Open,
                                    QuickCommand::Save,
                                    QuickCommand::Undo,
                                    QuickCommand::Redo,
                                ]
                            } else {
                                vec![
                                    QuickCommand::Save,
                                    QuickCommand::Undo,
                                    QuickCommand::Redo,
                                    QuickCommand::Open,
                                ]
                            },
                            placement: Placement::Below,
                        })
                        .expect("Preferences fixture");
                }
                DocsApp::with_preferences(session, &cc.egui_ctx, Ok(path))
            } else {
                DocsApp::new(session, &cc.egui_ctx)
            };
            match mode.as_str() {
                "tab-positions" | "typed-tabs-dialog" => {
                    app.execute(Action::TabPositions, &cc.egui_ctx)
                }
                "tab-dialog" => app.execute(Action::TabInterval, &cc.egui_ctx),
                "view" | "view-fit" | "keytips-qat-below" => {
                    app.tab = 8;
                    if mode == "view-fit" {
                        app.execute(Action::ZoomWidth, &cc.egui_ctx);
                    }
                }
                "replace-all" => app.execute(Action::Replace, &cc.egui_ctx),
                "clear-formatting" | "clear-formatting-tips" => {
                    app.execute(Action::SelectAll, &cc.egui_ctx)
                }
                "file" => app.execute(Action::File, &cc.egui_ctx),
                "navigation" | "navigation-options" => app.execute(Action::Find, &cc.egui_ctx),
                "collapsed" | "keytips-root" | "keytips-home" | "keytips-prefix" => {
                    app.ribbon_collapsed = true
                }
                "temporary" => {
                    app.ribbon_collapsed = true;
                    app.ribbon_temporary = true;
                }
                "styles-modify" | "styles-rename" | "styles-following" => {
                    app.execute(Action::ModifyStyle, &cc.egui_ctx)
                }
                "styles-character-delete" => {
                    app.execute(
                        Action::CharacterStyle(Some(
                            breakfree_docs::model::CharacterStyle::Custom(1),
                        )),
                        &cc.egui_ctx,
                    );
                    app.execute(Action::DeleteCharacterStyle, &cc.egui_ctx);
                }
                "styles-character-modify" => {
                    app.execute(
                        Action::CharacterStyle(Some(
                            breakfree_docs::model::CharacterStyle::Custom(1),
                        )),
                        &cc.egui_ctx,
                    );
                    app.execute(Action::ModifyCharacterStyle, &cc.egui_ctx);
                }
                "styles-character-create" => {
                    app.execute(Action::CreateCharacterStyle, &cc.egui_ctx)
                }
                "styles-create" => app.execute(Action::CreateStyle, &cc.egui_ctx),
                "styles-delete" => app.execute(Action::DeleteStyle, &cc.egui_ctx),
                "formatting" => app.execute(Action::Formatting, &cc.egui_ctx),
                "indent-layout"
                | "ruler-keyboard"
                | "ruler-mixed"
                | "ruler-reset"
                | "ruler-reset-single"
                | "ruler-sources"
                | "ruler-source-keyboard"
                | "ruler-source-list"
                | "ruler-source-wheel"
                | "ruler-source-menu"
                | "ruler-source-context"
                | "ruler-keyboard-menu" => {
                    if mode == "ruler-mixed"
                        || matches!(
                            mode.as_str(),
                            "ruler-sources"
                                | "ruler-source-keyboard"
                                | "ruler-source-list"
                                | "ruler-source-wheel"
                                | "ruler-source-menu"
                                | "ruler-source-context"
                                | "ruler-keyboard-menu"
                        )
                    {
                        app.execute(Action::SelectAll, &cc.egui_ctx);
                    }
                    if width < 800. {
                        app.zoom = 50.;
                    }
                }
                "paragraph" => app.execute(Action::ParagraphSpacing, &cc.egui_ctx),
                "font" => {
                    app.execute(Action::SelectAll, &cc.egui_ctx);
                    app.execute(Action::FontDialog, &cc.egui_ctx);
                }
                _ => {}
            }
            Ok(Box::new(Capture {
                app,
                path,
                frames: 0,
                started: None,
                requested: false,
                mode,
                _preferences_dir: preferences_dir,
            }))
        }),
    )?;
    Ok(())
}
