//! Modal draft UI. Only OK submits a single shared engine command.
use super::*;
use crate::commands::FontPatch;

pub(super) enum FontDialogOutcome {
    Open,
    Cancel,
    Apply(FontPatch),
}

pub(super) struct FontDialog {
    initial: FontPatch,
    patch: FontPatch,
    size: String,
    size_changed: bool,
    focus_size: bool,
    error: Option<String>,
}

impl FontDialog {
    pub fn new(initial: FontPatch) -> Self {
        Self {
            initial,
            patch: FontPatch::default(),
            size: initial.font_size.map(|s| s.to_string()).unwrap_or_default(),
            size_changed: false,
            focus_size: true,
            error: None,
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        family: &str,
        regions: &mut Vec<Region>,
    ) -> FontDialogOutcome {
        let mut outcome = FontDialogOutcome::Open;
        let mut accept = false;
        let record = |regions: &mut Vec<Region>, name: &str, rect| {
            regions.push(Region {
                name: name.into(),
                rect,
            });
        };
        let id = Id::new("font-dialog");
        let modal = egui::Modal::new(id)
            .area(egui::Modal::default_area(id).fade_in(false))
            .show(ctx, |ui| {
            ui.set_width(420.);
            ui.horizontal(|ui| {
                ui.heading("Font");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let close = ui.button("×").on_hover_text("Cancel font changes");
                    record(regions, "font-dialog-close", close.rect);
                    if close.clicked() { outcome = FontDialogOutcome::Cancel; }
                });
            });
            ui.separator();
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(vec2(172., 158.), Layout::top_down(Align::Min), |ui| {
                    ui.label("Font:");
                    let response = ui.add_enabled(false, Button::new(family).min_size(vec2(164., 25.)))
                        .on_disabled_hover_text("The document uses this host font. Font-family selection is not implemented.");
                    record(regions, "font-dialog-family", response.rect);
                    ui.add_space(7.);
                    ui.small("Font-family selection is planned.");
                });
                ui.allocate_ui_with_layout(vec2(136., 158.), Layout::top_down(Align::Min), |ui| {
                    ui.label("Font style:");
                    let bold = self.patch.bold.or(self.initial.bold);
                    let italic = self.patch.italic.or(self.initial.italic);
                    let styles = [("Regular", false, false), ("Italic", false, true),
                        ("Bold", true, false), ("Bold Italic", true, true)];
                    egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0_f32, chrome::LINE))
                        .inner_margin(3).show(ui, |ui| {
                            ui.set_width(122.);
                            for (name, b, i) in styles {
                                let response = ui.add_sized([122., 24.], egui::Button::selectable(
                                    bold == Some(b) && italic == Some(i), name));
                                record(regions, &format!("font-dialog-style-{name}"), response.rect);
                                if response.clicked() {
                                    self.patch.bold = Some(b);
                                    self.patch.italic = Some(i);
                                }
                            }
                        });
                    if bold.is_none() || italic.is_none() { ui.small("Mixed"); }
                });
                ui.allocate_ui_with_layout(vec2(92., 158.), Layout::top_down(Align::Min), |ui| {
                    ui.label("Size:");
                    let output = egui::TextEdit::singleline(&mut self.size)
                        .id(Id::new("font-dialog-size-input"))
                        .desired_width(76.).char_limit(8).hint_text("Mixed").show(ui);
                    output.response.widget_info(|| egui::WidgetInfo::labeled(
                        egui::WidgetType::TextEdit, true, "Font dialog size in points"));
                    record(regions, "font-dialog-size", output.response.rect);
                    if self.focus_size {
                        output.response.request_focus();
                        let mut state = output.state;
                        state.cursor.set_char_range(Some(CCursorRange::two(
                            CCursor::new(0), CCursor::new(self.size.chars().count()))));
                        state.store(ctx, Id::new("font-dialog-size-input"));
                        self.focus_size = false;
                    }
                    if output.response.changed() { self.size_changed = true; self.error = None; }
                    egui::ScrollArea::vertical().id_salt("font-dialog-sizes").max_height(106.)
                        .auto_shrink([false, false]).show(ui, |ui| {
                            ui.set_width(76.);
                            for size in FONT_SIZE_PRESETS {
                                let value = size.to_string();
                                let response = ui.selectable_label(self.size == value, &value);
                                if response.clicked() {
                                    self.size = value;
                                    self.size_changed = true;
                                    self.error = None;
                                }
                            }
                        });
                });
            });
            ui.separator();
            let underline = self.patch.underline.or(self.initial.underline);
            let mut value = underline.unwrap_or(false);
            let response = ui.add(egui::Checkbox::new(&mut value, "Underline")
                .indeterminate(underline.is_none()));
            record(regions, "font-dialog-underline", response.rect);
            if response.changed() { self.patch.underline = Some(value); }
            ui.add_space(4.);
            ui.label("Preview");
            egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0_f32, chrome::LINE))
                .show(ui, |ui| {
                    // Clip large sizes to the preview box; never scale the document draft.
                    let (rect, _) = ui.allocate_exact_size(vec2(418., 54.), Sense::hover());
                    let bold = self.patch.bold.or(self.initial.bold).unwrap_or(false);
                    let italic = self.patch.italic.or(self.initial.italic).unwrap_or(false);
                    let size = self.size.parse::<FontSize>().unwrap_or_default().points() * 4. / 3.;
                    let mut preview = RichText::new("AaBbCc Її").font(FontId::new(size,
                        egui::FontFamily::Name(if bold { "body-bold" } else { "body" }.into())));
                    if italic { preview = preview.italics(); }
                    if underline == Some(true) { preview = preview.underline(); }
                    let galley = egui::WidgetText::from(preview).into_galley(
                        ui, Some(egui::TextWrapMode::Extend), f32::INFINITY, egui::TextStyle::Body);
                    ui.painter().with_clip_rect(rect).galley(rect.center() - galley.size() / 2., galley, INK);
                });
            ui.small("Mixed values stay unchanged unless edited. Preview uses defaults for mixed values.");
            if let Some(error) = &self.error {
                ui.colored_label(Color32::from_rgb(165, 30, 30), error);
            }
            ui.add_space(6.);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let cancel = ui.add_sized([82., 27.], Button::new("Cancel"));
                record(regions, "font-dialog-cancel", cancel.rect);
                if cancel.clicked() { outcome = FontDialogOutcome::Cancel; }
                let ok = ui.add_sized([82., 27.], Button::new("OK"));
                record(regions, "font-dialog-ok", ok.rect);
                accept = ok.clicked();
            });
        });
        record(regions, "font-dialog", modal.response.rect);
        // Clicking the backdrop leaves the draft open. Escape always cancels.
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            return FontDialogOutcome::Cancel;
        }
        if matches!(outcome, FontDialogOutcome::Cancel) {
            return outcome;
        }
        if accept || ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)) {
            if self.size_changed {
                match self.size.parse::<FontSize>() {
                    Ok(size) => self.patch.font_size = Some(size),
                    Err(error) => {
                        self.error = Some(error);
                        self.focus_size = true;
                        return FontDialogOutcome::Open;
                    }
                }
            }
            return FontDialogOutcome::Apply(self.patch);
        }
        FontDialogOutcome::Open
    }
}
