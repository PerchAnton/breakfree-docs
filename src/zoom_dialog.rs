//! View-only draft. Accepting zoom never creates a document history entry.
use super::*;

pub(super) enum ZoomOutcome {
    Open,
    Cancel,
    Apply(f32),
}

pub(super) struct ZoomDialog {
    percent: String,
    focus: bool,
    error: bool,
}

impl ZoomDialog {
    pub fn new(value: f32) -> Self {
        Self {
            percent: format!("{value:.0}"),
            focus: true,
            error: false,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, regions: &mut Vec<Region>) -> ZoomOutcome {
        let mut accept = false;
        let mut cancel = false;
        let mut record = |name: &str, rect| {
            regions.push(Region {
                name: name.into(),
                rect,
            })
        };
        let id = Id::new("zoom-dialog");
        let modal = egui::Modal::new(id)
            .area(egui::Modal::default_area(id).fade_in(false))
            .show(ctx, |ui| {
                ui.set_width(280.);
                ui.heading("Zoom");
                ui.separator();
                ui.label("Zoom to");
                ui.horizontal(|ui| {
                    for value in [75, 100, 150, 200] {
                        let response = ui.selectable_label(
                            self.percent == value.to_string(),
                            format!("{value}%"),
                        );
                        record(&format!("zoom-preset-{value}"), response.rect);
                        if response.clicked() {
                            self.percent = value.to_string();
                            self.error = false;
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Percent:");
                    let output = egui::TextEdit::singleline(&mut self.percent)
                        .id(Id::new("zoom-percent"))
                        .desired_width(65.)
                        .char_limit(8)
                        .show(ui);
                    record("zoom-percent", output.response.rect);
                    output.response.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::TextEdit,
                            true,
                            "Zoom percent, 50 to 200",
                        )
                    });
                    if self.focus {
                        output.response.request_focus();
                        let mut state = output.state;
                        state.cursor.set_char_range(Some(CCursorRange::two(
                            CCursor::new(0),
                            CCursor::new(self.percent.chars().count()),
                        )));
                        state.store(ctx, output.response.id);
                        self.focus = false;
                    }
                    ui.label("%");
                });
                ui.small("50–200% · continuous canvas");
                if self.error {
                    ui.colored_label(Color32::DARK_RED, "Enter a whole number from 50 to 200.");
                }
                ui.add_space(8.);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let response = ui.add_sized([78., 27.], Button::new("Cancel"));
                    record("zoom-cancel", response.rect);
                    cancel = response.clicked();
                    let response = ui.add_sized([78., 27.], Button::new("OK"));
                    record("zoom-ok", response.rect);
                    accept = response.clicked();
                });
            });
        record("zoom-dialog", modal.response.rect);
        if cancel || ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            return ZoomOutcome::Cancel;
        }
        if accept || ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)) {
            if let Ok(value @ 50..=200) = self.percent.trim().parse::<u16>() {
                return ZoomOutcome::Apply(f32::from(value));
            }
            self.error = true;
            self.focus = true;
        }
        ZoomOutcome::Open
    }
}
