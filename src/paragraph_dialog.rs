//! Transactional direct spacing draft; unchecked properties stay unchanged.
use super::*;

pub(super) enum ParagraphOutcome {
    Open,
    Cancel,
    Apply(Option<Option<u16>>, Option<Option<u16>>),
}
#[derive(Default)]
pub(super) struct ParagraphDialog {
    line: String,
    after: String,
    change_line: bool,
    change_after: bool,
    focused: bool,
    error: Option<String>,
}
type SpacingValues = (Option<Option<u16>>, Option<Option<u16>>);

impl ParagraphDialog {
    fn values(&self) -> Result<SpacingValues> {
        let line = if !self.change_line {
            None
        } else if self.line.trim().is_empty() {
            Some(None)
        } else {
            Some(Some(
                self.line
                    .trim()
                    .parse::<u16>()
                    .map_err(|_| "Enter a whole percentage (100–300).")?,
            ))
        };
        let after = if !self.change_after {
            None
        } else if self.after.trim().is_empty() {
            Some(None)
        } else {
            let value = self.after.trim();
            let (whole, fraction) = value.split_once('.').unwrap_or((value, "0"));
            let half = match fraction {
                "0" | "" => 0,
                "5" => 1,
                _ => return Err("Enter 0–200 pt in half-point steps.".into()),
            };
            let whole: u16 = whole
                .parse()
                .map_err(|_| "Enter 0–200 pt in half-point steps.")?;
            if whole > 200 || (whole == 200 && half != 0) {
                return Err("Enter 0–200 pt in half-point steps.".into());
            }
            Some(Some(whole * 2 + half))
        };
        crate::model::ParagraphSpacing {
            line_height_percent: line.flatten(),
            space_after_half_points: after.flatten(),
        }
        .validate()?;
        Ok((line, after))
    }
    pub fn show(&mut self, ctx: &egui::Context, regions: &mut Vec<Region>) -> ParagraphOutcome {
        let mut cancel = false;
        let mut accept = false;
        let record = |regions: &mut Vec<Region>, name: &str, rect| {
            regions.push(Region {
                name: name.into(),
                rect,
            })
        };
        let id = Id::new("paragraph-dialog");
        let modal = egui::Modal::new(id)
            .area(egui::Modal::default_area(id).fade_in(false))
            .show(ctx, |ui| {
                ui.set_width(370.);
                ui.heading("Paragraph spacing");
                ui.separator();
                ui.label("Change only the checked properties in selected paragraphs.");
                ui.small("Leave a checked field blank to inherit from its paragraph style.");
                for (key, label, changed, text) in [
                    (
                        "line",
                        "Line spacing (%)",
                        &mut self.change_line,
                        &mut self.line,
                    ),
                    (
                        "after",
                        "Space after (pt)",
                        &mut self.change_after,
                        &mut self.after,
                    ),
                ] {
                    ui.horizontal(|ui| {
                        let check = ui.add_sized([155., 24.], egui::Checkbox::new(changed, label));
                        record(regions, &format!("paragraph-{key}-change"), check.rect);
                        let input = ui.add(
                            egui::TextEdit::singleline(text)
                                .id_salt(format!("paragraph-{key}"))
                                .desired_width(90.)
                                .char_limit(8)
                                .hint_text(if *changed { "From style" } else { "Unchanged" }),
                        );
                        record(regions, &format!("paragraph-{key}"), input.rect);
                        if input.changed() {
                            *changed = true;
                            self.error = None;
                        }
                    });
                }
                let reset = ui.button("Use style spacing");
                record(regions, "paragraph-reset", reset.rect);
                if reset.clicked() {
                    self.change_line = true;
                    self.change_after = true;
                    self.line.clear();
                    self.after.clear();
                    self.error = None;
                }
                ui.small("Line: 100–300%. After: 0–200 pt, in 0.5 pt steps.");
                if let Some(error) = &self.error {
                    ui.colored_label(Color32::from_rgb(165, 30, 30), error);
                }
                ui.separator();
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let response = ui.add_sized([82., 27.], Button::new("Cancel"));
                    record(regions, "paragraph-cancel", response.rect);
                    if !self.focused {
                        response.request_focus();
                        self.focused = true;
                    }
                    cancel = response.clicked();
                    let response = ui.add_sized([82., 27.], Button::new("OK"));
                    record(regions, "paragraph-ok", response.rect);
                    accept = response.clicked();
                });
            });
        record(regions, "paragraph-dialog", modal.response.rect);
        if cancel || ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            return ParagraphOutcome::Cancel;
        }
        // Standard focused-button Enter/Space activation; no global Enter override.
        if accept {
            match self.values() {
                Ok((line, after)) => return ParagraphOutcome::Apply(line, after),
                Err(error) => self.error = Some(error),
            }
        }
        ParagraphOutcome::Open
    }
}
