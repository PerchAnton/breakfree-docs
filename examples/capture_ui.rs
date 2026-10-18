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
        // Deterministic render fixture, including focus; this does not prove OS input.
        if self.mode.starts_with("keytips")
            || self.mode == "temporary"
            || self.mode == "replace-all"
            || self.mode.starts_with("styles")
            || self.mode.starts_with("clear-formatting")
            || self.mode.starts_with("navigation")
        {
            input.focused = true;
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
        if self.frames == 2 && ["styles-create", "styles-rename"].contains(&self.mode.as_str()) {
            input.events.push(egui::Event::Text("Report Note".into()));
        }
        if self.frames == 2 && self.mode == "styles-menu" {
            for key in [egui::Key::F10, egui::Key::H, egui::Key::E] {
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
            "styles",
            "styles-menu",
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
        return Err("Usage: capture_ui <width-DIP> <height-DIP> <home|styles|styles-menu|styles-modify|styles-following|styles-create|styles-rename|styles-delete|view|view-fit|keytips-qat|keytips-qat-below|file|navigation|navigation-options|replace-all|clear-formatting|clear-formatting-tips|formatting|font|collapsed|temporary|keytips-root|keytips-home|keytips-prefix|keytips-tabs|keytips-view|keytips-zoom>; set BREAKFREE_CAPTURE_TO".into());
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
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(if args[2] == "styles-following" {
            "examples/FollowingStyles.bfdoc"
        } else if args[2] == "styles-delete" {
            "examples/StyleDeletion.bfdoc"
        } else if ["styles-create", "styles-rename"].contains(&args[2].as_str()) {
            "examples/CustomStyles.bfdoc"
        } else if args[2].starts_with("styles") {
            "examples/Styles.bfdoc"
        } else if ["formatting", "font"].contains(&args[2].as_str()) {
            "examples/Formatting.bfdoc"
        } else {
            "examples/Welcome.bfdoc"
        });
    let session = if args[2] == "replace-all" {
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
                "styles-create" => app.execute(Action::CreateStyle, &cc.egui_ctx),
                "styles-delete" => app.execute(Action::DeleteStyle, &cc.egui_ctx),
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
