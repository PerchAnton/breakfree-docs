//! A draft over the shared interval command; opening or cancelling is read-only.
use super::*;

pub(super) enum TabOutcome {
    Open,
    Cancel,
    Apply(Option<u16>),
}

pub(super) struct TabDialog {
    value: String,
    default: bool,
    focus: bool,
    error: bool,
    mixed: bool,
    explicit_positions: bool,
}

impl TabDialog {
    pub fn new(editor: &crate::commands::Editor) -> Self {
        let (start, end) = editor.selection;
        let (first, _) = editor.document.locate(start).unwrap();
        let (mut last, offset) = editor.document.locate(end).unwrap();
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        let values = &editor.document.paragraphs[first..=last];
        let value = values[0].spacing.tab_interval_half_points;
        let mixed = values
            .iter()
            .any(|p| p.spacing.tab_interval_half_points != value);
        Self {
            value: if mixed {
                String::new()
            } else {
                format!("{}", f32::from(value.unwrap_or(72)) / 2.)
            },
            default: !mixed && value.is_none(),
            focus: true,
            error: false,
            mixed,
            explicit_positions: values
                .iter()
                .any(|p| !p.spacing.left_tab_stops_half_points.is_empty()),
        }
    }

    fn value(&self) -> Option<Option<u16>> {
        if self.default {
            return Some(None);
        }
        let text = self.value.trim();
        let (whole, fraction) = text.split_once('.').unwrap_or((text, "0"));
        let half = match fraction {
            "0" | "" => 0,
            "5" => 1,
            _ => return None,
        };
        let value = whole
            .parse::<u16>()
            .ok()?
            .checked_mul(2)?
            .checked_add(half)?;
        (1..=288).contains(&value).then_some(Some(value))
    }

    pub fn show(&mut self, ctx: &egui::Context, regions: &mut Vec<Region>) -> TabOutcome {
        let mut accept = false;
        let mut cancel = false;
        let mut record = |name: &str, rect| {
            regions.push(Region {
                name: name.into(),
                rect,
            })
        };
        let id = Id::new("tab-dialog");
        let modal = egui::Modal::new(id)
            .area(egui::Modal::default_area(id).fade_in(false))
            .show(ctx, |ui| {
                ui.set_width(290.);
                ui.heading("Tab interval");
                ui.separator();
                ui.label("Applies to the selected paragraphs.");
                if self.mixed {
                    ui.small("Selection contains different interval settings.");
                }
                ui.horizontal(|ui| {
                    ui.label("Interval:");
                    let output = egui::TextEdit::singleline(&mut self.value)
                        .id(Id::new("tab-interval-value"))
                        .desired_width(80.)
                        .char_limit(10)
                        .show(ui);
                    record("tab-interval-value", output.response.rect);
                    output.response.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::TextEdit,
                            true,
                            "Tab interval in points, 0.5 to 144",
                        )
                    });
                    if output.response.changed() {
                        self.default = false;
                        self.error = false;
                    }
                    if self.focus {
                        output.response.request_focus();
                        let mut state = output.state;
                        state.cursor.set_char_range(Some(CCursorRange::two(
                            CCursor::new(0),
                            CCursor::new(self.value.chars().count()),
                        )));
                        state.store(ctx, output.response.id);
                        self.focus = false;
                    }
                    ui.label("pt");
                });
                let response = ui.checkbox(&mut self.default, "Use default interval (36 pt)");
                record("tab-default", response.rect);
                if response.changed() && self.default {
                    self.value = "36".into();
                    self.error = false;
                }
                ui.small("0.5–144 pt, in 0.5 pt steps. Left tab grid only.");
                if self.explicit_positions {
                    ui.small("Explicit positions take precedence over this interval.");
                }
                ui.small("Edit explicit positions through Paragraph → Tab positions.");
                if self.error {
                    ui.colored_label(Color32::DARK_RED, "Enter 0.5–144 pt in half-point steps.");
                }
                ui.separator();
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let r = ui.add_sized([78., 27.], Button::new("Cancel"));
                    record("tab-cancel", r.rect);
                    cancel = r.clicked();
                    let r = ui.add_sized([78., 27.], Button::new("OK"));
                    record("tab-ok", r.rect);
                    accept = r.clicked();
                });
            });
        record("tab-dialog", modal.response.rect);
        if cancel || ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            return TabOutcome::Cancel;
        }
        // Let focused buttons/checkboxes retain their normal Enter/Space semantics.
        if accept {
            if let Some(value) = self.value() {
                return TabOutcome::Apply(value);
            }
            self.error = true;
            self.focus = true;
        }
        TabOutcome::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interval_draft_requires_exact_half_points() {
        let mut draft = TabDialog::new(&crate::commands::Editor::default());
        draft.default = false;
        for invalid in [
            "",
            "0",
            "144.5",
            "-1",
            "NaN",
            "inf",
            "0.50000001",
            "1e1",
            "65535",
        ] {
            draft.value = invalid.into();
            assert_eq!(draft.value(), None, "{invalid}");
        }
        for (text, expected) in [("0.5", 1), ("144", 288), ("36.0", 72), ("48.5", 97)] {
            draft.value = text.into();
            assert_eq!(draft.value(), Some(Some(expected)));
        }
    }
}
