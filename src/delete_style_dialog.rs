//! Local deletion draft; the shared command owns reference replacement and history.
use super::*;
use crate::model::{Document, StyleCatalog};

pub(super) enum DeleteStyleOutcome {
    Open,
    Cancel,
    Apply,
}

pub(super) struct DeleteStyleDialog {
    pub id: ParagraphStyle,
    pub paragraph_replacement: ParagraphStyle,
    pub parent_replacement: Option<ParagraphStyle>,
    focus_cancel: bool,
}
impl DeleteStyleDialog {
    pub fn new(id: ParagraphStyle, catalog: &StyleCatalog) -> Option<Self> {
        if !matches!(id, ParagraphStyle::Custom(_)) {
            return None;
        }
        let parent = catalog.get(id)?.parent;
        Some(Self {
            id,
            paragraph_replacement: parent.unwrap_or(ParagraphStyle::Normal),
            parent_replacement: parent,
            focus_cancel: true,
        })
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        document: &Document,
        regions: &mut Vec<Region>,
    ) -> DeleteStyleOutcome {
        let catalog = &document.style_catalog;
        let popup_was_open = egui::Popup::is_any_open(ctx);
        let mut outcome = DeleteStyleOutcome::Open;
        let record = |regions: &mut Vec<Region>, name: &str, rect| {
            regions.push(Region {
                name: name.into(),
                rect,
            })
        };
        let id = Id::new("delete-style-dialog");
        let modal = egui::Modal::new(id)
            .area(egui::Modal::default_area(id).fade_in(false))
            .show(ctx, |ui| {
                ui.set_width(460.);
                ui.heading("Delete Style");
                ui.add(egui::Label::new(RichText::new(catalog.name(self.id)).strong()).truncate())
                    .on_hover_text(catalog.name(self.id));
                ui.separator();
                let paragraphs = document.paragraphs.iter().filter(|p| p.paragraph_style == self.id).count();
                let children = catalog.ids().into_iter().filter(|id| catalog.get(*id).is_some_and(|d| d.parent == Some(self.id))).count();
                let counts = ui.label(format!("Paragraphs: {paragraphs} · Direct child styles: {children}"));
                record(regions, "delete-style-counts", counts.rect);
                ui.add_space(6.);
                egui::Grid::new("delete-style-replacements").num_columns(2).spacing([14., 10.]).show(ui, |ui| {
                    for (key, label, value, allow_none) in [
                        ("paragraphs", "Replace in paragraphs:", Some(self.paragraph_replacement), false),
                        ("children", "New parent for children:", self.parent_replacement, true),
                    ] {
                        ui.label(label);
                        let mut chosen = value;
                        let response = egui::ComboBox::from_id_salt(("delete-style-choice", key))
                            .width(280.).height(150.).truncate()
                            .selected_text(chosen.map(|id| catalog.name(id)).unwrap_or("(No style)"))
                            .show_ui(ui, |ui| {
                                ui.set_max_width(280.);
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                                if allow_none {
                                    let response = ui.selectable_value(&mut chosen, None, "(No style)");
                                    record(regions, "delete-style-children-none", response.rect);
                                    if response.clicked() { ui.close(); }
                                }
                                for id in catalog.ids() {
                                    if id != self.id {
                                        let response = ui.selectable_value(&mut chosen, Some(id), catalog.name(id));
                                        record(regions, &format!("delete-style-{key}-{id:?}"), response.rect);
                                        if response.clicked() { ui.close(); }
                                    }
                                }
                            }).response;
                        record(regions, &format!("delete-style-{key}"), response.rect);
                        if allow_none { self.parent_replacement = chosen; }
                        else if let Some(id) = chosen { self.paragraph_replacement = id; }
                        ui.end_row();
                    }
                });
                ui.add_space(10.);
                ui.label("Inherited appearance may change. Text, alignment and direct formatting stay. Indirect descendants keep their parent links.");
                ui.small("This document only. One Undo restores the style and all references.");
                let validation = catalog.without_custom_style(self.id, self.paragraph_replacement, self.parent_replacement);
                if let Err(error) = &validation { ui.colored_label(Color32::from_rgb(165, 30, 30), error); }
                ui.add_space(8.);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let cancel = ui.add_sized([82., 27.], Button::new("Cancel"));
                    record(regions, "delete-style-cancel", cancel.rect);
                    if self.focus_cancel { cancel.request_focus(); self.focus_cancel = false; }
                    if cancel.clicked() { outcome = DeleteStyleOutcome::Cancel; }
                    let delete = ui.add_enabled(validation.is_ok(), Button::new("Delete").min_size(vec2(82., 27.)));
                    record(regions, "delete-style-apply", delete.rect);
                    if delete.clicked() { outcome = DeleteStyleOutcome::Apply; }
                });
            });
        record(regions, "delete-style-dialog", modal.response.rect);
        // A combo consumes Escape first. Enter only activates its focused control.
        if !popup_was_open
            && !egui::Popup::is_any_open(ctx)
            && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape))
        {
            DeleteStyleOutcome::Cancel
        } else {
            outcome
        }
    }
}
