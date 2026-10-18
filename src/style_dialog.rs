//! A local definition draft. Only OK submits one model command.
use super::*;
use crate::model::{StyleCatalog, StyleDefinition};

pub(super) enum StyleOutcome {
    Open,
    Cancel,
    Apply(StyleDefinition),
}
pub(super) struct StyleDialog {
    pub id: ParagraphStyle,
    pub creating: bool,
    pub name: String,
    pub apply: bool,
    catalog: StyleCatalog,
    pub definition: StyleDefinition,
    pub size: String,
    pub line: String,
    pub after: String,
    focus: bool,
}
impl StyleDialog {
    pub fn new(id: ParagraphStyle, catalog: &StyleCatalog) -> Self {
        let definition = catalog.get(id).copied().unwrap_or_default();
        Self {
            id,
            creating: false,
            name: catalog.name(id).into(),
            apply: true,
            catalog: catalog.clone(),
            definition,
            size: definition
                .character
                .font_size
                .map(|v| v.to_string())
                .unwrap_or_default(),
            line: definition
                .line_height_percent
                .map(|v| v.to_string())
                .unwrap_or_default(),
            after: definition
                .space_after_half_points
                .map(|v| format!("{}{}", v / 2, if v % 2 == 1 { ".5" } else { "" }))
                .unwrap_or_default(),
            focus: true,
        }
    }
    pub fn create(parent: ParagraphStyle, catalog: &StyleCatalog) -> Self {
        let mut dialog = Self::new(ParagraphStyle::Custom(catalog.next_custom_id()), catalog);
        dialog.creating = true;
        dialog.name.clear();
        dialog.definition.parent = Some(parent);
        dialog
    }
    fn draft(&self) -> Result<(StyleDefinition, StyleCatalog)> {
        let mut definition = self.definition;
        definition.character.font_size = if self.size.trim().is_empty() {
            None
        } else {
            Some(self.size.parse()?)
        };
        definition.line_height_percent = if self.line.trim().is_empty() {
            None
        } else {
            Some(
                self.line
                    .trim()
                    .parse()
                    .map_err(|_| "Line spacing must be a whole percentage (100–300).")?,
            )
        };
        definition.space_after_half_points = if self.after.trim().is_empty() {
            None
        } else if matches!(self.after.trim(), "0" | "0.0") {
            Some(0)
        } else if self.after.trim() == "0.5" {
            Some(1)
        } else {
            Some(u16::from(self.after.parse::<FontSize>()?))
        };
        let mut catalog = self.catalog.clone();
        if self.creating {
            catalog.add(&self.name, definition)?;
        } else if matches!(self.id, ParagraphStyle::Custom(_)) {
            catalog.update_custom(self.id, &self.name, definition)?;
        } else {
            *catalog.get_mut(self.id).ok_or("Unknown paragraph style.")? = definition;
        }
        catalog.validate()?;
        Ok((definition, catalog))
    }
    pub fn show(&mut self, ctx: &egui::Context, regions: &mut Vec<Region>) -> StyleOutcome {
        let popup_was_open = egui::Popup::is_any_open(ctx);
        let mut outcome = StyleOutcome::Open;
        let mut accept = false;
        let record = |regions: &mut Vec<Region>, name: &str, rect| {
            regions.push(Region {
                name: name.into(),
                rect,
            })
        };
        let id = Id::new("style-dialog");
        let modal = egui::Modal::new(id)
            .area(egui::Modal::default_area(id).fade_in(false))
            .show(ctx, |ui| {
                ui.set_width(460.);
                ui.horizontal(|ui| {
                    ui.heading(if self.creating {
                        "Create Style"
                    } else {
                        "Modify Style"
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .button("×")
                            .on_hover_text("Cancel style changes")
                            .clicked()
                        {
                            outcome = StyleOutcome::Cancel;
                        }
                    });
                });
                ui.separator();
                egui::Grid::new("style-properties")
                    .num_columns(2)
                    .spacing([14., 7.])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        if self.creating || matches!(self.id, ParagraphStyle::Custom(_)) {
                            let output = egui::TextEdit::singleline(&mut self.name)
                                .id_salt("style-name")
                                .desired_width(290.)
                                .hint_text("Style name")
                                .show(ui);
                            let response = output.response;
                            response.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::WidgetType::TextEdit,
                                    true,
                                    "Style name",
                                )
                            });
                            record(regions, "style-dialog-name", response.rect);
                            if self.focus
                                || (response.lost_focus()
                                    && ctx.input(|i| i.key_pressed(Key::Enter)))
                            {
                                response.request_focus();
                                if self.focus && !self.creating {
                                    let mut state = output.state;
                                    state.cursor.set_char_range(Some(CCursorRange::two(
                                        CCursor::new(0),
                                        CCursor::new(self.name.chars().count()),
                                    )));
                                    state.store(ctx, response.id);
                                }
                                self.focus = false;
                            }
                        } else {
                            ui.add_sized(
                                [290., 20.],
                                egui::Label::new(RichText::new(&self.name).strong()).truncate(),
                            )
                            .on_hover_text(&self.name);
                        }
                        ui.end_row();
                        ui.label("Style based on:");
                        let parent = egui::ComboBox::from_id_salt("style-parent")
                            .width(290.)
                            .height(150.)
                            .truncate()
                            .selected_text(
                                self.definition
                                    .parent
                                    .map(|id| self.catalog.name(id))
                                    .unwrap_or("(No style)"),
                            )
                            .show_ui(ui, |ui| {
                                let root = ui.selectable_value(
                                    &mut self.definition.parent,
                                    None,
                                    "(No style)",
                                );
                                record(regions, "style-parent-none", root.rect);
                                if root.clicked() {
                                    ui.close();
                                }
                                ui.set_max_width(290.);
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                                for parent in self.catalog.ids() {
                                    if parent != self.id {
                                        let response = ui.selectable_value(
                                            &mut self.definition.parent,
                                            Some(parent),
                                            self.catalog.name(parent),
                                        );
                                        record(
                                            regions,
                                            &format!("style-parent-{parent:?}"),
                                            response.rect,
                                        );
                                        if response.clicked() {
                                            ui.close();
                                        }
                                    }
                                }
                            })
                            .response;
                        record(regions, "style-dialog-parent", parent.rect);
                        ui.end_row();
                        ui.label("Formatting:");
                        ui.horizontal(|ui| {
                            for (name, value) in [
                                ("Bold", &mut self.definition.character.bold),
                                ("Italic", &mut self.definition.character.italic),
                                ("Underline", &mut self.definition.character.underline),
                            ] {
                                let response =
                                    egui::ComboBox::from_id_salt(("style-property", name))
                                        .width(90.)
                                        .selected_text(format!(
                                            "{name}: {}",
                                            match value {
                                                None => "Inherit",
                                                Some(true) => "On",
                                                Some(false) => "Off",
                                            }
                                        ))
                                        .show_ui(ui, |ui| {
                                            for (label, option) in [
                                                ("Inherit", None),
                                                ("On", Some(true)),
                                                ("Off", Some(false)),
                                            ] {
                                                if ui
                                                    .selectable_value(value, option, label)
                                                    .clicked()
                                                {
                                                    ui.close();
                                                }
                                            }
                                        })
                                        .response;
                                record(regions, &format!("style-dialog-{name}"), response.rect);
                            }
                        });
                        ui.end_row();
                        for (name, label, value) in [
                            ("size", "Font size (pt):", &mut self.size),
                            ("line", "Line spacing (%):", &mut self.line),
                            ("after", "Space after (pt):", &mut self.after),
                        ] {
                            ui.label(label);
                            let output = egui::TextEdit::singleline(&mut *value)
                                .id_salt(("style-input", name))
                                .desired_width(290.)
                                .hint_text("Inherit")
                                .char_limit(8)
                                .show(ui);
                            let response = output.response;
                            response.widget_info(|| {
                                egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, label)
                            });
                            record(regions, &format!("style-dialog-{name}"), response.rect);
                            if name == "size" && self.focus {
                                response.request_focus();
                                let mut state = output.state;
                                state.cursor.set_char_range(Some(CCursorRange::two(
                                    CCursor::new(0),
                                    CCursor::new(value.chars().count()),
                                )));
                                state.store(ctx, response.id);
                                self.focus = false;
                            }
                            ui.end_row();
                        }
                    });
                ui.add_space(6.);
                ui.small("Blank fields inherit. No parent uses regular 11 pt, 125%, 5.5 pt after.");
                let draft = self.draft();
                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.0_f32, chrome::LINE))
                    .inner_margin(6)
                    .show(ui, |ui| {
                        let (rect, _) = ui.allocate_exact_size(vec2(448., 48.), Sense::hover());
                        if let Ok((_, catalog)) = &draft {
                            let style = catalog.resolve(self.id).character;
                            let mut text =
                                RichText::new("AaBbCc Україна Русский café").font(FontId::new(
                                    style.font_size.points() * 4. / 3.,
                                    egui::FontFamily::Name(
                                        if style.bold { "body-bold" } else { "body" }.into(),
                                    ),
                                ));
                            if style.italic {
                                text = text.italics();
                            }
                            if style.underline {
                                text = text.underline();
                            }
                            let galley = egui::WidgetText::from(text).into_galley(
                                ui,
                                Some(egui::TextWrapMode::Extend),
                                f32::INFINITY,
                                egui::TextStyle::Body,
                            );
                            ui.painter().with_clip_rect(rect).galley(
                                rect.left_center() - vec2(0., galley.size().y / 2.),
                                galley,
                                INK,
                            );
                        }
                        record(regions, "style-dialog-preview", rect);
                    });
                if self.creating {
                    let apply = ui.checkbox(&mut self.apply, "Apply to selected paragraphs");
                    record(regions, "style-dialog-apply", apply.rect);
                    ui.small("This document only. Creation and application use one undo step.");
                } else if matches!(self.id, ParagraphStyle::Custom(_)) {
                    ui.small("Renaming keeps all style references. Changes use one undo step.");
                } else {
                    ui.small("This document only. Updates descendants; direct formatting stays.");
                }
                if let Err(error) = &draft {
                    ui.colored_label(Color32::from_rgb(165, 30, 30), error);
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let cancel = ui.add_sized([82., 27.], Button::new("Cancel"));
                    record(regions, "style-dialog-cancel", cancel.rect);
                    if cancel.clicked() {
                        outcome = StyleOutcome::Cancel;
                    }
                    let ok =
                        ui.add_enabled(draft.is_ok(), Button::new("OK").min_size(vec2(82., 27.)));
                    record(regions, "style-dialog-ok", ok.rect);
                    accept = ok.clicked();
                });
            });
        record(regions, "style-dialog", modal.response.rect);
        // Let an open combo consume Enter/Escape before the containing dialog.
        if !popup_was_open && !egui::Popup::is_any_open(ctx) {
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
                return StyleOutcome::Cancel;
            }
            if !matches!(outcome, StyleOutcome::Cancel)
                && (accept || ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)))
                && let Ok((definition, _)) = self.draft()
            {
                return StyleOutcome::Apply(definition);
            }
        }
        outcome
    }
}
