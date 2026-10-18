//! Transactional direct spacing draft; unchecked properties stay unchanged.
use super::*;

pub(super) enum ParagraphOutcome {
    Open,
    Cancel,
    Apply(crate::commands::ParagraphGeometryPatch),
}
#[derive(Default)]
pub(super) struct ParagraphDialog {
    line: String,
    after: String,
    before: String,
    left: String,
    right: String,
    first_line: String,
    hanging: bool,
    change_first_line: bool,
    change_left: bool,
    change_right: bool,
    change_line: bool,
    change_after: bool,
    change_before: bool,
    focused: bool,
    error: Option<String>,
}

impl ParagraphDialog {
    fn values(&self) -> Result<crate::commands::ParagraphGeometryPatch> {
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
        let before = if !self.change_before {
            None
        } else if self.before.trim().is_empty() {
            Some(None)
        } else {
            let value = self.before.trim();
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
            space_before_half_points: before.flatten(),
            ..Default::default()
        }
        .validate()?;
        let left = if self.change_left {
            Some(crate::model::parse_indent(&self.left)?)
        } else {
            None
        };
        let right = if self.change_right {
            Some(crate::model::parse_indent(&self.right)?)
        } else {
            None
        };
        Ok(crate::commands::ParagraphGeometryPatch {
            line,
            before,
            after,
            left,
            right,
            first_line: if self.change_first_line {
                Some(
                    crate::model::parse_indent(&self.first_line)?
                        .map(|v| (v as i16) * if self.hanging { -1 } else { 1 }),
                )
            } else {
                None
            },
        })
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
                ui.heading("Paragraph spacing and indents");
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
                        "before",
                        "Space before (pt)",
                        &mut self.change_before,
                        &mut self.before,
                    ),
                    (
                        "after",
                        "Space after (pt)",
                        &mut self.change_after,
                        &mut self.after,
                    ),
                    (
                        "left",
                        "Left indent (pt)",
                        &mut self.change_left,
                        &mut self.left,
                    ),
                    (
                        "right",
                        "Right indent (pt)",
                        &mut self.change_right,
                        &mut self.right,
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
                ui.horizontal(|ui| {
                    let check = ui.add_sized(
                        [155., 24.],
                        egui::Checkbox::new(&mut self.change_first_line, "Special indent"),
                    );
                    record(regions, "paragraph-first-change", check.rect);
                    let input = ui.add(
                        egui::TextEdit::singleline(&mut self.first_line)
                            .id_salt("paragraph-first")
                            .desired_width(90.)
                            .char_limit(8)
                            .hint_text(if self.change_first_line {
                                "From style"
                            } else {
                                "Unchanged"
                            }),
                    );
                    record(regions, "paragraph-first", input.rect);
                    if input.changed() {
                        self.change_first_line = true;
                        self.error = None;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    for (value, label, key) in [
                        (false, "First line", "first-line"),
                        (true, "Hanging", "hanging"),
                    ] {
                        let r = ui.selectable_value(&mut self.hanging, value, label);
                        record(regions, &format!("paragraph-{key}"), r.rect);
                        if r.changed() {
                            self.change_first_line = true;
                            self.error = None;
                        }
                    }
                    ui.small("Amount in pt");
                });
                let reset = ui.button("Use style spacing and indents");
                record(regions, "paragraph-reset", reset.rect);
                if reset.clicked() {
                    self.change_first_line = true;
                    self.first_line.clear();
                    self.change_left = true;
                    self.change_right = true;
                    self.left.clear();
                    self.right.clear();
                    self.change_line = true;
                    self.change_after = true;
                    self.change_before = true;
                    self.line.clear();
                    self.after.clear();
                    self.before.clear();
                    self.error = None;
                }
                ui.small("Line: 100–300%. Before/after: 0–200 pt, in 0.5 pt steps.");
                ui.small("Indents: 0–144 pt, in 0.5 pt steps.");
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
                Ok(patch) => return ParagraphOutcome::Apply(patch),
                Err(error) => self.error = Some(error),
            }
        }
        ParagraphOutcome::Open
    }
}
