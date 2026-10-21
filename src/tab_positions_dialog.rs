//! Explicit-position draft; only OK invokes the shared document transaction.
use super::*;

pub(super) enum PositionsOutcome {
    Open,
    Cancel,
    Apply(Vec<u16>),
}

pub(super) struct PositionsDialog {
    positions: Vec<u16>,
    selected: Option<u16>,
    input: String,
    input_dirty: bool,
    focus: bool,
    mixed: bool,
    changed: bool,
    error: Option<String>,
}

impl PositionsDialog {
    pub fn new(editor: &crate::commands::Editor) -> Self {
        let (start, end) = editor.selection;
        let (first, _) = editor.document.locate(start).unwrap();
        let (mut last, offset) = editor.document.locate(end).unwrap();
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        let paragraphs = &editor.document.paragraphs[first..=last];
        let positions = &paragraphs[0].spacing.left_tab_stops_half_points;
        let mixed = paragraphs
            .iter()
            .any(|p| &p.spacing.left_tab_stops_half_points != positions);
        Self {
            positions: if mixed { vec![] } else { positions.clone() },
            selected: None,
            input: String::new(),
            input_dirty: false,
            focus: true,
            mixed,
            changed: false,
            error: None,
        }
    }

    fn parse(&self) -> Result<u16> {
        let text = self.input.trim();
        let (whole, fraction) = text.split_once('.').unwrap_or((text, "0"));
        let half = match fraction {
            "" | "0" => 0,
            "5" => 1,
            _ => return Err("Use half-point steps (for example 48.5).".into()),
        };
        whole
            .parse::<u16>()
            .ok()
            .and_then(|v| v.checked_mul(2))
            .and_then(|v| v.checked_add(half))
            .filter(|v| (1..=2880).contains(v))
            .ok_or("Enter 0.5–1440 pt.".into())
    }

    fn set(&mut self, update: bool) -> Result<()> {
        let value = self.parse()?;
        if self.positions.contains(&value) && (!update || self.selected != Some(value)) {
            return Err("This position already exists.".into());
        }
        if !update && self.positions.len() == 32 {
            return Err("At most 32 positions are supported.".into());
        }
        if update {
            let old = self.selected.ok_or("Select a position to update.")?;
            self.positions.retain(|p| *p != old);
        }
        self.positions.push(value);
        self.positions.sort_unstable();
        self.selected = Some(value);
        self.input_dirty = false;
        self.changed = true;
        Ok(())
    }

    pub fn show(&mut self, ctx: &egui::Context, regions: &mut Vec<Region>) -> PositionsOutcome {
        let mut accept = false;
        let mut cancel = false;
        let mut record = |name: &str, rect| {
            regions.push(Region {
                name: name.into(),
                rect,
            })
        };
        let id = Id::new("tab-positions-dialog");
        let modal = egui::Modal::new(id)
            .area(egui::Modal::default_area(id).fade_in(false))
            .show(ctx, |ui| {
                ui.set_width(320.);
                ui.heading("Tab positions");
                ui.separator();
                ui.label("Left positions in points from the text-column edge.");
                if self.mixed {
                    ui.small("Different lists selected. Build a replacement or Clear all.");
                }
                ui.small("OK replaces left positions only; center/right stops are preserved.");
                egui::ScrollArea::vertical()
                    .id_salt("positions-list")
                    .max_height(115.)
                    .min_scrolled_height(40.)
                    .show(ui, |ui| {
                        ui.set_min_width(300.);
                        if self.positions.is_empty() {
                            ui.label(if self.mixed && !self.changed {
                                "Mixed positions"
                            } else {
                                "No explicit left positions"
                            });
                        }
                        for &value in &self.positions {
                            let r = ui.selectable_label(
                                self.selected == Some(value),
                                format!("{} pt", f32::from(value) / 2.),
                            );
                            record(&format!("tab-position-{value}"), r.rect);
                            if r.clicked() {
                                self.selected = Some(value);
                                self.input = format!("{}", f32::from(value) / 2.);
                                self.input_dirty = false;
                                self.error = None;
                            }
                        }
                    });
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    let out = egui::TextEdit::singleline(&mut self.input)
                        .id(Id::new("tab-position-input"))
                        .desired_width(90.)
                        .char_limit(10)
                        .show(ui);
                    record("tab-position-input", out.response.rect);
                    out.response.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::TextEdit,
                            true,
                            "Left tab position, 0.5 to 1440 points",
                        )
                    });
                    if out.response.changed() {
                        self.input_dirty = true;
                        self.error = None;
                    }
                    if self.focus {
                        out.response.request_focus();
                        self.focus = false;
                    }
                    ui.label("pt");
                });
                ui.horizontal(|ui| {
                    let r = ui.button("Add");
                    record("tab-position-add", r.rect);
                    if r.clicked() {
                        self.error = self.set(false).err();
                    }
                    let r = ui.add_enabled(self.selected.is_some(), Button::new("Update"));
                    record("tab-position-update", r.rect);
                    if r.clicked() {
                        self.error = self.set(true).err();
                    }
                    let r = ui.add_enabled(self.selected.is_some(), Button::new("Remove"));
                    record("tab-position-remove", r.rect);
                    if r.clicked() {
                        self.positions.retain(|p| Some(*p) != self.selected);
                        self.selected = None;
                        self.input.clear();
                        self.input_dirty = false;
                        self.changed = true;
                        self.error = None;
                    }
                    let r = ui.button("Clear all");
                    record("tab-position-clear", r.rect);
                    if r.clicked() {
                        self.positions.clear();
                        self.selected = None;
                        self.input.clear();
                        self.input_dirty = false;
                        self.changed = true;
                        self.error = None;
                    }
                });
                ui.small("0.5–1440 pt, in 0.5 pt steps; up to 32 positions.");
                if let Some(error) = &self.error {
                    ui.colored_label(Color32::DARK_RED, error);
                }
                ui.separator();
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let r = ui.add_sized([78., 27.], Button::new("Cancel"));
                    record("tab-positions-cancel", r.rect);
                    cancel = r.clicked();
                    let r = ui.add_enabled(
                        !self.mixed || self.changed,
                        Button::new("OK").min_size(vec2(78., 27.)),
                    );
                    record("tab-positions-ok", r.rect);
                    accept = r.clicked();
                });
            });
        record("tab-positions-dialog", modal.response.rect);
        if cancel || ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            return PositionsOutcome::Cancel;
        }
        if accept {
            if self.input_dirty && !self.input.trim().is_empty() {
                self.error = Some("Use Add or Update before applying the list.".into());
            } else {
                return PositionsOutcome::Apply(self.positions.clone());
            }
        }
        PositionsOutcome::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn draft_validation_limits_and_updates() {
        let mut d = PositionsDialog::new(&crate::commands::Editor::default());
        for bad in ["", "0", "1440.5", "65535", "NaN", "1e1", "1.50000001"] {
            d.input = bad.into();
            assert!(d.set(false).is_err());
            assert!(d.positions.is_empty());
        }
        for n in 1..=32 {
            d.input = n.to_string();
            d.set(false).unwrap();
        }
        d.input = "100".into();
        assert!(d.set(false).is_err());
        assert_eq!(d.positions.len(), 32);
        d.selected = Some(2);
        d.input = "32".into();
        assert!(d.set(true).is_err());
        assert_eq!(d.positions[0], 2);
        d.input = "1440".into();
        d.set(true).unwrap();
        assert_eq!(d.positions.last(), Some(&2880));
        d.input = "0.5".into();
        d.set(true).unwrap();
        assert_eq!(d.positions[0], 1);
    }
}
