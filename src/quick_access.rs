//! Window-local toolbar preferences. No document data or persistence lives here.
use super::*;

pub(super) const DEFAULT: &[Action] = &[Action::Save, Action::Undo, Action::Redo];

impl DocsApp {
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
            && self.zoom_dialog.is_none();
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
            ui.small("Applies to this window");
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
