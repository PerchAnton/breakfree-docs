//! Read-only preview of a validated replacement transaction.
use super::*;
use crate::{
    commands::ReplacementPlan,
    search::{Options, Query},
};
use unicode_segmentation::UnicodeSegmentation;

pub(super) enum ReplaceAllOutcome {
    Open,
    Cancel,
    Apply,
}

pub(super) struct ReplaceAllDialog {
    pub plan: ReplacementPlan,
    source: String,
    query: Query,
    find: String,
    replacement: String,
    options: Options,
    focus_cancel: bool,
}

fn bounded(text: &str, limit: usize) -> String {
    let mut value: String = text.graphemes(true).take(limit).collect();
    if value.len() < text.len() {
        value.push('…');
    }
    value.replace('\n', " / ").replace('\t', "  ")
}

impl ReplaceAllDialog {
    pub fn new(
        editor: &crate::commands::Editor,
        find: &str,
        replacement: &str,
        options: Options,
    ) -> Result<Self> {
        let source = editor.document.text();
        let query = Query::new(find, options);
        if let Some(error) = query.error() {
            return Err(error.into());
        }
        let ranges: Vec<_> = query
            .search(&source)
            .matches
            .iter()
            .map(|m| m.range)
            .collect();
        let plan = editor.preview_replace_all(&ranges, replacement)?;
        Ok(Self {
            plan,
            source,
            query,
            find: find.into(),
            replacement: replacement.into(),
            options,
            focus_cancel: true,
        })
    }

    pub fn show(&mut self, ctx: &egui::Context, regions: &mut Vec<Region>) -> ReplaceAllOutcome {
        let mut outcome = ReplaceAllOutcome::Open;
        let results = self.query.search(&self.source);
        let record = |regions: &mut Vec<Region>, name: &str, rect| {
            regions.push(Region {
                name: name.into(),
                rect,
            })
        };
        let id = Id::new("replace-all-preview");
        let modal = egui::Modal::new(id).area(egui::Modal::default_area(id).fade_in(false)).show(ctx, |ui| {
            ui.set_width((ctx.content_rect().width() - 48.).clamp(240., 560.));
            ui.heading("Replace All — preview");
            ui.label(format!("{} match{} · {} text change{}", self.plan.match_count(), if self.plan.match_count() == 1 { "" } else { "es" }, self.plan.change_count(), if self.plan.change_count() == 1 { "" } else { "s" }));
            ui.add(egui::Label::new(format!("Find: {}", bounded(&self.find, 48))).truncate());
            ui.add(egui::Label::new(format!("Replace with: {}", if self.replacement.is_empty() { "(delete)".into() } else { bounded(&self.replacement, 48) })).truncate());
            ui.small(format!("{} · {}", if self.options.match_case { "Match case" } else { "Ignore case" }, if self.options.whole_words { "Whole words" } else { "Any text" }));
            ui.separator();
            egui::ScrollArea::vertical().id_salt("replace-all-rows").max_height((ctx.content_rect().height() - 270.).clamp(40., 260.))
                .auto_shrink([false, false]).show_rows(ui, 66., results.matches.len(), |ui, rows| {
                    for index in rows {
                        let (before, matched, after) = results.excerpt(index);
                        let original = &results.matches[index];
                        let unchanged = crate::model::slice(&self.source, original.range.0, original.range.1) == self.replacement;
                        let before = format!("{} · Paragraph {}  {}[{}]{}", index + 1, original.paragraph, before, matched, after);
                        let after = if unchanged { "Unchanged — original formatting retained".into() }
                            else if self.replacement.is_empty() { "After: (delete matched text)".into() }
                            else { format!("After: {}", bounded(&self.replacement, 64)) };
                        let response = ui.allocate_ui(vec2(ui.available_width(), 66.), |ui| {
                            ui.set_min_height(66.);
                            ui.add(egui::Label::new(RichText::new(before).font(FontId::new(13., egui::FontFamily::Name("body".into())))).truncate());
                            ui.add(egui::Label::new(RichText::new(after).color(ACCENT).font(FontId::new(13., egui::FontFamily::Name("body".into())))).truncate());
                            ui.separator();
                        }).response;
                        record(regions, &format!("replace-all-row-{index}"), response.rect);
                    }
                });
            ui.small("Inserted text uses each match's first character formatting. One Undo restores all changes.");
            if self.plan.change_count() == 0 { ui.label("No text would change."); }
            ui.separator();
            ui.horizontal(|ui| {
                let cancel = ui.add_sized([80., 27.], Button::new("Cancel"));
                record(regions, "replace-all-cancel", cancel.rect);
                if self.focus_cancel { cancel.request_focus(); self.focus_cancel = false; }
                if cancel.clicked() { outcome = ReplaceAllOutcome::Cancel; }
                let apply = ui.add_enabled(self.plan.change_count() != 0, Button::new(format!("Apply {} change{}", self.plan.change_count(), if self.plan.change_count() == 1 { "" } else { "s" })).min_size(vec2(128., 27.)));
                record(regions, "replace-all-apply", apply.rect);
                if apply.clicked() { outcome = ReplaceAllOutcome::Apply; }
            });
        });
        record(regions, "replace-all-preview", modal.response.rect);
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            ReplaceAllOutcome::Cancel
        } else {
            outcome
        }
    }
}
