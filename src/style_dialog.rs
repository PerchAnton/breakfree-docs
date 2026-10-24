//! A local definition draft. Only OK submits one model command.
use super::*;
use crate::model::{
    CharacterFormat, Document, NamedCharacterStyle, Style, StyleCatalog, StyleDefinition,
};

pub(super) enum StyleOutcome {
    Open,
    Cancel,
    Apply(StyleDefinition),
}
pub(super) struct StyleDialog {
    pub id: ParagraphStyle,
    pub creating: bool,
    pub character: bool,
    pub character_id: Option<CharacterStyle>,
    pub character_parent: Option<CharacterStyle>,
    character_styles: Vec<NamedCharacterStyle>,
    preview_base: Style,
    pub name: String,
    pub apply: bool,
    catalog: StyleCatalog,
    pub definition: StyleDefinition,
    pub size: String,
    pub line: String,
    pub after: String,
    pub before: String,
    pub left: String,
    pub right: String,
    pub first_line: String,
    pub hanging: bool,
    focus: bool,
    tabs: Option<PositionsDialog>,
    interval: Option<TabDialog>,
    return_to_tabs: Option<bool>,
}
impl StyleDialog {
    pub fn new(id: ParagraphStyle, catalog: &StyleCatalog) -> Self {
        let definition = catalog.get(id).cloned().unwrap_or_default();
        Self {
            id,
            creating: false,
            character: false,
            character_id: None,
            character_parent: None,
            character_styles: Vec::new(),
            preview_base: Style::default(),
            name: catalog.name(id).into(),
            apply: true,
            catalog: catalog.clone(),
            definition: definition.clone(),
            size: definition
                .character
                .font_size
                .map(|v| v.to_string())
                .unwrap_or_default(),
            line: definition
                .line_height_percent
                .map(|v| v.to_string())
                .unwrap_or_default(),
            before: definition
                .space_before_half_points
                .map(|v| format!("{}{}", v / 2, if v % 2 == 1 { ".5" } else { "" }))
                .unwrap_or_default(),
            after: definition
                .space_after_half_points
                .map(|v| format!("{}{}", v / 2, if v % 2 == 1 { ".5" } else { "" }))
                .unwrap_or_default(),
            left: definition
                .left_indent_half_points
                .map(|v| (f32::from(v) / 2.).to_string())
                .unwrap_or_default(),
            right: definition
                .right_indent_half_points
                .map(|v| (f32::from(v) / 2.).to_string())
                .unwrap_or_default(),
            first_line: definition
                .first_line_half_points
                .map(|v| (f32::from(v.unsigned_abs()) / 2.).to_string())
                .unwrap_or_default(),
            hanging: definition.first_line_half_points.is_some_and(|v| v < 0),
            focus: true,
            tabs: None,
            interval: None,
            return_to_tabs: None,
        }
    }
    pub fn create(parent: ParagraphStyle, catalog: &StyleCatalog) -> Self {
        let mut dialog = Self::new(ParagraphStyle::Custom(catalog.next_custom_id()), catalog);
        dialog.creating = true;
        dialog.name.clear();
        dialog.definition.parent = Some(parent);
        dialog
    }
    pub fn create_character(document: &Document, paragraph: ParagraphStyle) -> Self {
        let mut dialog = Self::new(ParagraphStyle::Normal, &document.style_catalog);
        dialog.creating = true;
        dialog.character = true;
        dialog.name.clear();
        dialog.definition = StyleDefinition::default();
        dialog.size.clear();
        dialog.line.clear();
        dialog.after.clear();
        dialog.before.clear();
        dialog.left.clear();
        dialog.right.clear();
        dialog.first_line.clear();
        dialog.hanging = false;
        dialog.character_styles = document.character_styles.clone();
        dialog.preview_base = document.style_catalog.resolve(paragraph).character;
        dialog
    }
    pub fn modify_character(
        document: &Document,
        paragraph: ParagraphStyle,
        id: CharacterStyle,
    ) -> Option<Self> {
        let CharacterStyle::Custom(key) = id else {
            return None;
        };
        let style = document.character_styles.iter().find(|s| s.id == key)?;
        let mut dialog = Self::create_character(document, paragraph);
        dialog.creating = false;
        dialog.character_id = Some(id);
        dialog.name = style.name.clone();
        dialog.character_parent = style.parent;
        dialog.definition.character = style.format;
        dialog.size = style
            .format
            .font_size
            .map(|v| v.to_string())
            .unwrap_or_default();
        Some(dialog)
    }
    fn draft(&self) -> Result<(StyleDefinition, StyleCatalog)> {
        let mut definition = self.definition.clone();
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
        definition.space_before_half_points = if self.before.trim().is_empty() {
            None
        } else if matches!(self.before.trim(), "0" | "0.0") {
            Some(0)
        } else if self.before.trim() == "0.5" {
            Some(1)
        } else {
            Some(u16::from(self.before.parse::<FontSize>()?))
        };
        definition.left_indent_half_points = crate::model::parse_indent(&self.left)?;
        definition.right_indent_half_points = crate::model::parse_indent(&self.right)?;
        definition.first_line_half_points = crate::model::parse_indent(&self.first_line)?
            .map(|v| (v as i16) * if self.hanging { -1 } else { 1 });
        let mut catalog = self.catalog.clone();
        if self.character {
            let mut document = Document {
                character_styles: self.character_styles.clone(),
                ..Default::default()
            };
            let id = if let Some(id) = self.character_id {
                document.update_character_style(id, &self.name, definition.character)?;
                id
            } else {
                document.add_character_style(&self.name, definition.character)?
            };
            document.set_character_parent(id, self.character_parent)?;
        } else if self.creating {
            catalog.add(&self.name, definition.clone())?;
        } else if matches!(self.id, ParagraphStyle::Custom(_)) {
            catalog.update_custom(self.id, &self.name, definition.clone())?;
        } else {
            *catalog.get_mut(self.id).ok_or("Unknown paragraph style.")? = definition.clone();
        }
        catalog.validate()?;
        Ok((definition, catalog))
    }
    pub fn show(&mut self, ctx: &egui::Context, regions: &mut Vec<Region>) -> StyleOutcome {
        if let Some(mut dialog) = self.interval.take() {
            match dialog.show(ctx, regions) {
                TabOutcome::Open => self.interval = Some(dialog),
                TabOutcome::Cancel => self.return_to_tabs = Some(false),
                TabOutcome::Apply(value) => {
                    self.definition.tab_interval_half_points = value;
                    self.return_to_tabs = Some(false);
                }
            }
            return StyleOutcome::Open;
        }
        if let Some(mut dialog) = self.tabs.take() {
            match dialog.show(ctx, regions) {
                PositionsOutcome::Open => self.tabs = Some(dialog),
                PositionsOutcome::Cancel => self.return_to_tabs = Some(true),
                PositionsOutcome::Inherit => {
                    self.definition.tab_stops = None;
                    self.return_to_tabs = Some(true);
                }
                PositionsOutcome::Apply { positions, leaders } => {
                    self.definition.tab_stops = Some(
                        positions
                            .into_iter()
                            .map(|(position_half_points, kind)| crate::model::StyleTabStop {
                                position_half_points,
                                kind,
                                leader: leaders
                                    .iter()
                                    .find(|s| s.position_half_points == position_half_points)
                                    .map(|s| s.leader),
                            })
                            .collect(),
                    );
                    self.return_to_tabs = Some(true);
                }
            }
            return StyleOutcome::Open;
        }
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
                let compact = ctx.content_rect().height() < 600.;
                ui.horizontal(|ui| {
                    ui.heading(if self.character {
                        if self.creating {
                            "Create Character Style"
                        } else {
                            "Modify Character Style"
                        }
                    } else if self.creating {
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
                    .spacing([14., if compact { 1. } else { 7. }])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        if self.creating
                            || self.character_id.is_some()
                            || matches!(self.id, ParagraphStyle::Custom(_))
                        {
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
                        if self.character {
                            ui.label("Style based on:");
                            let name = |id: CharacterStyle| {
                                self.character_styles
                                    .iter()
                                    .find(|s| CharacterStyle::Custom(s.id) == id)
                                    .map_or(id.name(), |s| s.name.as_str())
                            };
                            let response = egui::ComboBox::from_id_salt("character-parent")
                                .width(290.)
                                .height(150.)
                                .truncate()
                                .selected_text(
                                    self.character_parent
                                        .map(name)
                                        .unwrap_or("(Paragraph style)"),
                                )
                                .show_ui(ui, |ui| {
                                    ui.set_max_width(290.);
                                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                                    let choices = [
                                        None,
                                        Some(CharacterStyle::Strong),
                                        Some(CharacterStyle::Emphasis),
                                    ]
                                    .into_iter()
                                    .chain(
                                        self.character_styles
                                            .iter()
                                            .map(|s| Some(CharacterStyle::Custom(s.id))),
                                    );
                                    for choice in
                                        choices.filter(|c| c.is_none() || *c != self.character_id)
                                    {
                                        if ui
                                            .selectable_value(
                                                &mut self.character_parent,
                                                choice,
                                                choice.map(name).unwrap_or("(Paragraph style)"),
                                            )
                                            .clicked()
                                        {
                                            ui.close();
                                        }
                                    }
                                })
                                .response;
                            record(regions, "style-dialog-character-parent", response.rect);
                            ui.end_row();
                        }
                        if !self.character {
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
                            ui.label("Following paragraph:");
                            let next = egui::ComboBox::from_id_salt("style-next")
                                .width(290.)
                                .height(150.)
                                .truncate()
                                .selected_text(
                                    self.definition
                                        .next_style
                                        .map(|id| self.catalog.name(id))
                                        .unwrap_or("(Same style)"),
                                )
                                .show_ui(ui, |ui| {
                                    let r = ui.selectable_value(
                                        &mut self.definition.next_style,
                                        None,
                                        "(Same style)",
                                    );
                                    record(regions, "style-next-same", r.rect);
                                    if r.clicked() {
                                        ui.close();
                                    }
                                    ui.set_max_width(290.);
                                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                                    for id in self.catalog.ids() {
                                        let r = ui.selectable_value(
                                            &mut self.definition.next_style,
                                            Some(id),
                                            self.catalog.name(id),
                                        );
                                        record(regions, &format!("style-next-{id:?}"), r.rect);
                                        if r.clicked() {
                                            ui.close();
                                        }
                                    }
                                })
                                .response;
                            record(regions, "style-dialog-next", next.rect);
                            ui.end_row();
                        }
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
                            ("before", "Space before (pt):", &mut self.before),
                            ("after", "Space after (pt):", &mut self.after),
                            ("left", "Left indent (pt):", &mut self.left),
                            ("right", "Right indent (pt):", &mut self.right),
                            ("first", "Special indent (pt):", &mut self.first_line),
                        ] {
                            if self.character && name != "size" {
                                continue;
                            }
                            if name == "first" {
                                ui.label(label);
                                ui.horizontal(|ui| {
                                    let r = ui.add(
                                        egui::TextEdit::singleline(&mut *value)
                                            .id_salt(("style-input", name))
                                            .desired_width(80.)
                                            .hint_text("Inherit")
                                            .char_limit(8),
                                    );
                                    record(regions, "style-dialog-first", r.rect);
                                    for (v, label, key) in [
                                        (false, "First line", "first-line"),
                                        (true, "Hanging", "hanging"),
                                    ] {
                                        let r = ui.selectable_value(&mut self.hanging, v, label);
                                        record(regions, &format!("style-dialog-{key}"), r.rect);
                                    }
                                });
                                ui.end_row();
                                continue;
                            }
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
                ui.small(if self.character {
                    "Unset properties inherit the parent, then paragraph. Direct formatting wins."
                } else {
                    "Blank fields inherit. No parent uses regular 11 pt, 125%, 5.5 pt after."
                });
                let draft = self.draft();
                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.0_f32, chrome::LINE))
                    .inner_margin(if compact { 3 } else { 6 })
                    .show(ui, |ui| {
                        let (rect, _) = ui.allocate_exact_size(
                            vec2(448., if compact { 24. } else { 48. }),
                            Sense::hover(),
                        );
                        if let Ok((definition, catalog)) = &draft {
                            let style = if self.character {
                                definition.character.resolve(
                                    CharacterFormat {
                                        character_style: self.character_parent,
                                        ..Default::default()
                                    }
                                    .resolve_with(self.preview_base, &self.character_styles),
                                )
                            } else {
                                catalog.resolve(self.id).character
                            };
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
                    let apply = ui.checkbox(
                        &mut self.apply,
                        if self.character {
                            "Apply to selection / future typing"
                        } else {
                            "Apply to selected paragraphs"
                        },
                    );
                    record(regions, "style-dialog-apply", apply.rect);
                    apply.on_hover_text(
                        "This document only. Creation and application use one undo step.",
                    );
                    if !compact {
                        ui.small("This document only. Creation and application use one undo step.");
                    }
                } else if self.character {
                    ui.small("Renaming keeps references; name and properties use one undo step.");
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
                    if !self.character {
                        for (positions, label, name) in [
                            (true, "Tab positions…", "style-tab-positions"),
                            (false, "Tab interval…", "style-tab-interval"),
                        ] {
                            let response = ui.add_enabled(
                                draft.is_ok(),
                                Button::new(label).min_size(vec2(0., 27.)),
                            );
                            let response = if let Ok((_, catalog)) = &draft {
                                let document = Document {
                                    style_catalog: catalog.clone(),
                                    ..Default::default()
                                };
                                let report = crate::tab_sources::style(&document, self.id);
                                let info = if positions {
                                    &report.positions
                                } else {
                                    &report.interval
                                };
                                let help = info.summary();
                                response.widget_info(|| {
                                    egui::WidgetInfo::labeled(
                                        egui::WidgetType::Button,
                                        response.enabled(),
                                        format!("{label}. {help}"),
                                    )
                                });
                                response.on_hover_text(help)
                            } else {
                                response
                            };
                            record(regions, name, response.rect);
                            if self.return_to_tabs == Some(positions) {
                                response.request_focus();
                                self.return_to_tabs = None;
                            }
                            if response.clicked()
                                && let Ok((definition, catalog)) = &draft
                            {
                                let mut editor = crate::commands::Editor::default();
                                editor.document.style_catalog = catalog.clone();
                                editor.document.paragraphs[0].paragraph_style = self.id;
                                editor.document.paragraphs[0]
                                    .spacing
                                    .tab_interval_half_points = definition.tab_interval_half_points;
                                if positions {
                                    self.tabs = Some(PositionsDialog::for_style(&editor));
                                } else {
                                    self.interval = Some(TabDialog::for_style(&editor));
                                }
                            }
                        }
                    }
                });
            });
        record(regions, "style-dialog", modal.response.rect);
        if self.tabs.is_some() || self.interval.is_some() {
            return StyleOutcome::Open;
        }
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
