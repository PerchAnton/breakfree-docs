//! A draft over the shared interval command; opening or cancelling is read-only.
use super::*;

pub(super) enum TabOutcome {
    Open,
    Cancel,
    Apply(Option<u16>),
}

pub(super) struct TabDialog {
    style_scope: bool,
    sources: crate::tab_sources::Inspection,
    source_dialog: Option<TabSourcesDialog>,
    value: String,
    default: bool,
    focus: bool,
    error: bool,
    mixed: bool,
    explicit_positions: bool,
}

impl TabDialog {
    pub fn for_style(editor: &crate::commands::Editor) -> Self {
        Self {
            style_scope: true,
            sources: crate::tab_sources::style(
                &editor.document,
                editor.document.paragraphs[0].paragraph_style,
            ),
            ..Self::new(editor)
        }
    }
    pub fn new(editor: &crate::commands::Editor) -> Self {
        let (start, end) = editor.selection;
        let (first, _) = editor.document.locate(start).unwrap();
        let (mut last, offset) = editor.document.locate(end).unwrap();
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        let values = &editor.document.paragraphs[first..=last];
        let value = values[0].spacing.tab_interval_half_points;
        let effective = editor
            .document
            .resolved_tabs(&values[0])
            .tab_interval_half_points
            .unwrap_or(72);
        let mixed = values.iter().any(|p| {
            p.spacing.tab_interval_half_points != value
                || editor.document.resolved_tabs(p).tab_interval_half_points != Some(effective)
        });
        Self {
            style_scope: false,
            sources: crate::tab_sources::inspect(&editor.document, editor.selection, None)
                .unwrap_or_default(),
            source_dialog: None,
            value: if mixed {
                String::new()
            } else {
                format!("{}", f32::from(effective) / 2.)
            },
            default: !mixed && value.is_none(),
            focus: true,
            error: false,
            mixed,
            explicit_positions: values
                .iter()
                .any(|p| !editor.document.resolved_tabs(p).tab_positions().is_empty()),
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
        if let Some(mut dialog) = self.source_dialog.take() {
            if dialog.show(ctx, regions) {
                dialog.restore_focus(ctx);
            } else {
                self.source_dialog = Some(dialog);
            }
            return TabOutcome::Open;
        }
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
                ui.horizontal(|ui| {
                    ui.heading(if self.style_scope {
                        "Style tab interval"
                    } else {
                        "Tab interval"
                    });
                    let source = ui.button("Sources…").on_hover_text(
                        "Inspect the settings at dialog entry; changes remain in this draft.",
                    );
                    record("tab-settings-sources", source.rect);
                    if source.clicked() {
                        source.request_focus();
                        self.source_dialog = Some(TabSourcesDialog::new(
                            self.sources.clone(),
                            if self.style_scope {
                                "Style tabulation sources at entry"
                            } else {
                                "Tabulation sources at entry"
                            },
                            ctx,
                        ));
                    }
                });
                ui.separator();
                ui.label(if self.style_scope {
                    "Changes stay in the style draft until its OK."
                } else {
                    "Applies to the selected paragraphs."
                });
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
                let response = ui.checkbox(
                    &mut self.default,
                    if self.style_scope {
                        "Inherit interval from parent"
                    } else {
                        "Inherit interval from style"
                    },
                );
                record("tab-default", response.rect);
                if response.changed() && self.default {
                    self.value.clear();
                    self.error = false;
                }
                ui.small("0.5–144 pt, in 0.5 pt steps. Left tab grid only.");
                if self.explicit_positions {
                    ui.small("Explicit positions take precedence over this interval.");
                }
                if !self.style_scope {
                    ui.small("Edit explicit positions through Paragraph → Tab positions.");
                }
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
