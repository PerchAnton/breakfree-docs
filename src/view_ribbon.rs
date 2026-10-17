//! Compact View groups for the continuous canvas; pagination commands stay disabled.
use super::*;

impl DocsApp {
    fn view_check(&mut self, ui: &mut Ui, rect: Rect, label: &str, action: Action) {
        let mut checked = self.checked(action);
        let response = ui
            .scope_builder(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(Layout::left_to_right(Align::Center)),
                |ui| {
                    ui.add_enabled(
                        self.enabled(action),
                        egui::Checkbox::new(&mut checked, label),
                    )
                },
            )
            .inner;
        let name = format!("command-{action:?}");
        self.record(&name, response.rect);
        self.track_ribbon(&response, &name, TargetKind::Action(action));
        if response.on_hover_text(action.label()).clicked() {
            self.execute(action, ui.ctx());
        }
    }

    pub(super) fn view_ribbon(&mut self, ui: &mut Ui) {
        let origin = ui.max_rect().min + vec2(8., 4.);
        let rect = |x, y, w, h| Rect::from_min_size(origin + vec2(x, y), vec2(w, h));
        for (x, w, label) in [
            (0., 120., "Views"),
            (128., 156., "Show"),
            (292., 264., "Zoom"),
        ] {
            let group = rect(x, 0., w, 88.);
            self.record(&format!("view-group-{label}"), group);
            ui.painter().text(
                group.center_bottom() - vec2(0., 7.),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(11.),
                Color32::from_gray(95),
            );
            if x > 0. {
                ui.painter().line_segment(
                    [origin + vec2(x - 4., 4.), origin + vec2(x - 4., 84.)],
                    Stroke::new(1.0_f32, chrome::LINE),
                );
            }
        }
        for (x, label, reason) in [
            (0., "Read Mode", "Read Mode is not implemented."),
            (
                60.,
                "Print Layout",
                "Pagination is not implemented. The document uses a continuous canvas.",
            ),
        ] {
            chrome::control(
                ui,
                rect(x, 0., 58., 66.),
                Icon::Document,
                if label == "Read Mode" {
                    "Read\nMode"
                } else {
                    "Print\nLayout"
                },
                label,
                false,
                false,
            )
            .on_disabled_hover_text(reason);
        }
        self.ribbon_group = Some(0);
        self.view_check(ui, rect(134., 2., 146., 23.), "Ruler", Action::Rulers);
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(rect(134., 26., 146., 23.))
                .layout(Layout::left_to_right(Align::Center)),
            |ui| ui.add_enabled(false, egui::Checkbox::new(&mut false, "Gridlines")),
        )
        .inner
        .on_disabled_hover_text("Gridlines are not implemented.");
        self.view_check(
            ui,
            rect(134., 50., 146., 23.),
            "Navigation Pane",
            Action::Navigation,
        );
        self.ribbon_group = Some(1);
        self.ribbon_control(
            ui,
            rect(294., 0., 52., 66.),
            Icon::Search,
            "Zoom",
            Action::Zoom,
        );
        self.ribbon_control(
            ui,
            rect(350., 0., 52., 66.),
            Icon::Document,
            "100%",
            Action::Zoom100,
        );
        for (y, label) in [(0., "One Page"), (24., "Multiple Pages")] {
            chrome::control(
                ui,
                rect(410., y, 144., 23.),
                Icon::Document,
                label,
                label,
                false,
                false,
            )
            .on_disabled_hover_text("Pagination is not implemented.");
        }
        self.ribbon_control(
            ui,
            rect(410., 48., 144., 23.),
            Icon::CanvasWidth,
            "Canvas Width",
            Action::ZoomWidth,
        );
        self.ribbon_group = None;
    }
}
