//! Native toolbar controls and the adapter to GUI-independent local preferences.
use super::*;

pub(super) const DEFAULT: &[Action] = &[Action::Save, Action::Undo, Action::Redo];

impl DocsApp {
    /// The production entry point uses local preferences. `new` stays isolated for previews/tests.
    pub fn with_preferences(session: Session, ctx: &egui::Context, path: Result<PathBuf>) -> Self {
        use crate::preferences::{Placement, QuickCommand, Store};
        let (store, toolbar, warning) = Store::open(path);
        let mut app = Self::new(session, ctx);
        app.quick_access = toolbar
            .commands
            .into_iter()
            .map(|command| match command {
                QuickCommand::New => Action::New,
                QuickCommand::Open => Action::Open,
                QuickCommand::Save => Action::Save,
                QuickCommand::Undo => Action::Undo,
                QuickCommand::Redo => Action::Redo,
            })
            .collect();
        app.quick_access_below = toolbar.placement == Placement::Below;
        app.preferences = Some(store);
        if let Some(warning) = warning {
            app.preferences_failed(warning);
        }
        app
    }

    fn toolbar_preferences(&self) -> crate::preferences::Toolbar {
        use crate::preferences::{Placement, QuickCommand, Toolbar};
        Toolbar {
            commands: self
                .quick_access
                .iter()
                .map(|action| match action {
                    Action::New => QuickCommand::New,
                    Action::Open => QuickCommand::Open,
                    Action::Save => QuickCommand::Save,
                    Action::Undo => QuickCommand::Undo,
                    Action::Redo => QuickCommand::Redo,
                    _ => {
                        unreachable!("Only file/history actions can be configured in this toolbar")
                    }
                })
                .collect(),
            placement: if self.quick_access_below {
                Placement::Below
            } else {
                Placement::Above
            },
        }
    }

    fn preferences_failed(&mut self, reason: String) {
        let path = self
            .preferences
            .as_ref()
            .and_then(|s| s.path())
            .map(|p| format!("\n{}", p.display()))
            .unwrap_or_default();
        let warning = format!(
            "Toolbar settings could not be saved or loaded. Changes apply only to this window. The previous file was not replaced.\n\n{reason}{path}\n\nUse Retry Saving after fixing the cause, or Reset Toolbar to keep a backup and restore defaults."
        );
        self.preferences_warning = Some(warning.clone());
        self.error = Some(warning);
    }

    pub(super) fn save_quick_preferences(&mut self) {
        let toolbar = self.toolbar_preferences();
        if let Some(store) = &mut self.preferences {
            let result = if self.preferences_reset_pending {
                store.replace_with_backup(&toolbar)
            } else {
                store.save(&toolbar).map(|_| None)
            };
            match result {
                Ok(backup) => {
                    if self.preferences_reset_pending {
                        self.preferences_backup = backup;
                        self.preferences_reset_pending = false;
                    }
                    self.preferences_warning = None;
                }
                Err(e) => self.preferences_failed(e),
            }
        }
    }

    pub(super) fn quick_reset_dialog(&mut self, ctx: &egui::Context) {
        if !self.quick_reset {
            return;
        }
        let mut reset = false;
        let mut cancel = false;
        egui::Modal::new(Id::new("quick-access-reset")).show(ctx, |ui| {
            ui.set_width(370.);
            ui.heading("Reset Quick Access Toolbar?");
            ui.label("Restore Save, Undo and Redo above the ribbon?");
            if self.preferences.is_some() {
                ui.label("The existing settings file will be kept as a backup. These defaults will apply the next time you open Docs.");
            }
            ui.add_space(10.);
            ui.horizontal(|ui| {
                let response = ui.button("Reset");
                self.record("qat-reset-confirm", response.rect);
                reset = response.clicked();
                let response = ui.button("Cancel");
                self.record("qat-reset-cancel", response.rect);
                cancel = response.clicked();
            });
        });
        cancel |= ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        if cancel {
            self.quick_reset = false;
            self.finish_quick_option(ctx);
        } else if reset {
            self.quick_reset = false;
            self.quick_access = DEFAULT.to_vec();
            self.quick_access_below = false;
            self.finish_quick_option(ctx);
            self.preferences_reset_pending = self.preferences.is_some();
            self.save_quick_preferences();
        }
    }

    pub(super) fn finish_quick_option(&mut self, ctx: &egui::Context) {
        egui::Popup::close_all(ctx);
        self.ribbon_menu_index = None;
        self.ribbon_focus = Some(RibbonFocus::QuickAccess("qat-options".into()));
        self.focus_document = false;
    }

    pub(super) fn open_quick_options(&mut self, ctx: &egui::Context) {
        if let Some(target) = self
            .quick_access_targets
            .iter()
            .find(|t| t.name == "qat-options")
            .cloned()
        {
            self.ribbon_focus = Some(RibbonFocus::QuickAccess(target.name.clone()));
            self.focus_document = false;
            self.ribbon_temporary = false;
            self.activate_target(&target, ctx);
        }
    }

    fn quick_option(
        &mut self,
        ui: &mut Ui,
        name: &str,
        label: &str,
        checked: Option<bool>,
        kind: TargetKind,
    ) {
        let response = if let Some(mut value) = checked {
            ui.checkbox(&mut value, label)
        } else {
            ui.add_sized([220., 24.], Button::new(label).frame(false))
        };
        self.record(name, response.rect);
        self.track_ribbon(&response, name, kind);
        if response.clicked() {
            let target = self.ribbon_menu_targets.last().unwrap().clone();
            self.activate_target(&target, ui.ctx());
        }
    }

    pub(super) fn quick_access_bar(&mut self, ui: &mut Ui, origin: egui::Pos2) {
        for (i, action) in self.quick_access.clone().into_iter().enumerate() {
            let icon = match action {
                Action::New => Icon::Document,
                Action::Open => Icon::Open,
                Action::Undo => Icon::Undo,
                Action::Redo => Icon::Redo,
                _ => Icon::Save,
            };
            let name = format!("qat-{action:?}");
            let rect = Rect::from_min_size(origin + vec2(i as f32 * 30., 0.), vec2(28., 28.));
            let response = chrome::control(
                ui,
                rect,
                icon,
                "",
                action.label(),
                self.enabled(action),
                false,
            );
            self.record(&name, rect);
            if response.enabled() {
                self.quick_access_targets.push(RibbonTarget {
                    name,
                    id: response.id,
                    rect,
                    group: 0,
                    kind: TargetKind::Action(action),
                });
            }
            let shortcut = match action {
                Action::Save => "Ctrl+S",
                Action::Undo => "Ctrl+Z",
                Action::Redo => "Ctrl+Y",
                Action::New => "Ctrl+N",
                _ => "Ctrl+O",
            };
            if response.on_hover_text(format!("{} ({shortcut})", action.label())).on_disabled_hover_text("No history operation available, or finish the open dialog/text composition first.").clicked() {
                self.execute(action, ui.ctx());
            }
        }
        let rect = Rect::from_min_size(
            origin + vec2(self.quick_access.len() as f32 * 30., 0.),
            vec2(24., 28.),
        );
        let enabled = !self.composing
            && self.pending.is_none()
            && self.error.is_none()
            && self.font_dialog.is_none()
            && self.paragraph_dialog.is_none()
            && self.style_dialog.is_none()
            && self.delete_style_dialog.is_none()
            && self.zoom_dialog.is_none()
            && !self.quick_reset
            && self.quick_order.is_none();
        let response = chrome::control(
            ui,
            rect,
            Icon::Chevron(false),
            "",
            "Customize Quick Access Toolbar",
            enabled,
            false,
        );
        self.record("qat-options", rect);
        if enabled {
            self.quick_access_targets.push(RibbonTarget {
                name: "qat-options".into(),
                id: response.id,
                rect,
                group: 0,
                kind: TargetKind::Menu,
            });
        }
        if response.clicked() {
            self.ribbon_focus = Some(RibbonFocus::QuickAccess("qat-options".into()));
            self.focus_document = false;
        }
        let response = response.on_hover_text("Customize Quick Access Toolbar (Alt, 0)");
        egui::Popup::menu(&response).show(|ui| {
            ui.set_min_width(230.);
            ui.strong("Customize Quick Access Toolbar");
            if let Some(warning) = &self.preferences_warning {
                ui.colored_label(Color32::DARK_RED, "Not saved · this window only")
                    .on_hover_text(warning);
            } else if self.preferences.is_some() {
                ui.small("Applies to all documents");
                if let Some(path) = &self.preferences_backup {
                    ui.small("Settings backup kept")
                        .on_hover_text(path.display().to_string());
                }
            } else {
                ui.small("Preview · settings are not saved");
            }
            ui.separator();
            self.ribbon_group = Some(0);
            self.collecting_menu = true;
            for action in [
                Action::New,
                Action::Open,
                Action::Save,
                Action::Undo,
                Action::Redo,
            ] {
                self.quick_option(
                    ui,
                    &format!("qat-toggle-{action:?}"),
                    action.label(),
                    Some(self.quick_access.contains(&action)),
                    TargetKind::QuickToggle(action),
                );
            }
            ui.separator();
            let position = if self.quick_access_below {
                "Show Above the Ribbon"
            } else {
                "Show Below the Ribbon"
            };
            self.quick_option(
                ui,
                "qat-position",
                position,
                None,
                TargetKind::QuickPosition,
            );
            self.quick_option(
                ui,
                "qat-reset",
                "Reset Toolbar",
                None,
                TargetKind::QuickReset,
            );
            if self.preferences_warning.is_some() {
                self.quick_option(
                    ui,
                    "qat-retry",
                    "Retry Saving",
                    None,
                    TargetKind::QuickRetry,
                );
            }
            ui.separator();
            self.quick_option(
                ui,
                "qat-order",
                "More Commands…",
                None,
                TargetKind::QuickOrder,
            );
            self.collecting_menu = false;
            self.ribbon_group = None;
            self.record("quick-access-menu", ui.min_rect());
        });
    }

    pub(super) fn quick_access_panel(&mut self, ctx: &egui::Context) {
        if self.quick_access_below && !self.backstage {
            egui::TopBottomPanel::top("quick-access-below")
                .exact_height(32.)
                .frame(egui::Frame::new().fill(CHROME).inner_margin(0))
                .show(ctx, |ui| {
                    self.quick_access_bar(ui, ui.max_rect().min + vec2(8., 2.));
                });
        }
    }
}
