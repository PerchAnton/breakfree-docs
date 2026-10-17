//! Navigation widgets project the same grapheme-safe matches used by replacement.
use super::*;
use crate::search::Results;

const ROW_HEIGHT: f32 = 64.;

impl DocsApp {
    fn select_find_result(&mut self, results: &Results<'_>, index: usize) {
        self.session.editor.selection = results.matches[index].range;
        self.session.editor.typing_style = self
            .session
            .editor
            .document
            .style_at(self.session.editor.selection.0);
        self.sync_view();
        self.focus_document = false;
        self.scroll_to_selection = true;
        self.find_scroll = Some(index);
    }

    pub(super) fn navigation_panel(&mut self, ctx: &egui::Context) {
        if !self.navigation || self.backstage {
            return;
        }
        let panel = egui::SidePanel::left("navigation")
            .default_width(260.)
            .min_width(220.)
            .max_width(320.)
            .frame(egui::Frame::side_top_panel(&ctx.style()).fill(chrome::SURFACE))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Navigation");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let close = ui.button("×").on_hover_text("Close navigation (Escape from search)");
                        self.record("find-close", close.rect);
                        if close.clicked() {
                            self.navigation = false;
                            self.focus_document = true;
                        }
                    });
                });
                let id = Id::new("find-text");
                let focused = ui.memory(|m| m.has_focus(id));
                // Do not interpret IME confirmation as a navigation command.
                let ime_event = ui.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Ime(_))));
                if focused {
                    ui.input(|i| for event in &i.events {
                        match event {
                            egui::Event::Ime(egui::ImeEvent::Enabled | egui::ImeEvent::Preedit(_)) => self.find_composing = true,
                            egui::Event::Ime(egui::ImeEvent::Commit(_) | egui::ImeEvent::Disabled) => self.find_composing = false,
                            _ => {}
                        }
                    });
                } else {
                    self.find_composing = false;
                }
                let ready = !self.composing && !self.find_composing && !ime_event;
                let mut direction = None;
                if focused && ready {
                    ui.input_mut(|i| {
                        if i.consume_key(Modifiers::SHIFT, Key::Enter) || i.consume_key(Modifiers::NONE, Key::ArrowUp) {
                            direction = Some(true);
                        } else if i.consume_key(Modifiers::NONE, Key::Enter) || i.consume_key(Modifiers::NONE, Key::ArrowDown) {
                            direction = Some(false);
                        }
                    });
                }
                let mut changed = false;
                ui.horizontal(|ui| {
                    let response = ui.add_sized(
                        [ui.available_width() - 28., 26.],
                        egui::TextEdit::singleline(&mut self.find).id(id).hint_text("Search document"),
                    ).on_hover_text("Literal, case-sensitive search. Enter / Down: next; Shift+Enter / Up: previous.");
                    self.record("find-input", response.rect);
                    changed = response.changed();
                    if self.focus_find {
                        response.request_focus();
                        self.focus_find = false;
                    }
                    let clear = ui.add_enabled(!self.find.is_empty() && ready, Button::new("×")).on_hover_text("Clear search");
                    self.record("find-clear", clear.rect);
                    if clear.clicked() {
                        self.find.clear();
                        changed = true;
                        response.request_focus();
                    }
                });
                let mut text = self.session.editor.document.text();
                let mut results = Results::new(&text, &self.find);
                if changed && ready {
                    self.find_scroll = None;
                    if !results.matches.is_empty() {
                        self.select_find_result(&results, 0);
                    }
                }
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Results").strong());
                    Self::disabled(ui, "Headings", vec2(68., 23.), "Heading styles are not implemented.");
                    Self::disabled(ui, "Pages", vec2(52., 23.), "Pages require pagination, which is not implemented.");
                });
                ui.separator();
                ui.horizontal(|ui| {
                    let label = if self.find.is_empty() {
                        "Search document text".into()
                    } else if let Some(index) = results.active(self.session.editor.selection) {
                        format!("Result {} of {}", index + 1, results.matches.len())
                    } else {
                        format!("{} {}", results.matches.len(), if results.matches.len() == 1 { "result" } else { "results" })
                    };
                    ui.label(label);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        for (name, previous, hint) in [
                            ("find-next", false, "Next result (Enter / Down in search)"),
                            ("find-previous", true, "Previous result (Shift+Enter / Up in search)"),
                        ] {
                            let (rect, _) = ui.allocate_exact_size(vec2(24., 24.), Sense::hover());
                            let response = chrome::control(ui, rect, Icon::Chevron(previous), "", hint, ready && !results.matches.is_empty(), false);
                            self.record(name, response.rect);
                            if response.clicked() { direction = Some(previous); }
                        }
                    });
                });
                if ready && let Some(previous) = direction
                    && let Some(index) = results.adjacent(self.session.editor.selection, previous) {
                    self.select_find_result(&results, index);
                }
                if self.replace_mode {
                    ui.add_sized([ui.available_width(), 26.], egui::TextEdit::singleline(&mut self.replacement).id(Id::new("replace-text")).hint_text("Replace with"));
                    let replace = ui.add_enabled(ready && !results.matches.is_empty(), Button::new("Replace selection / next"));
                    self.record("find-replace", replace.rect);
                    if replace.clicked() {
                        if let Some(index) = results.active(self.session.editor.selection) {
                            let (a, b) = results.matches[index].range;
                            let result = self.session.editor.replace(a, b, &self.replacement, "Replace match");
                            self.fail(result);
                            self.sync_view();
                            self.focus_document = false;
                            text = self.session.editor.document.text();
                            results = Results::new(&text, &self.find);
                            self.find_scroll = None;
                        } else if let Some(index) = results.adjacent(self.session.editor.selection, false) {
                            self.select_find_result(&results, index);
                        }
                    }
                }
                ui.add_space(4.);
                ui.small("Exact text · case sensitive");
                if self.find.is_empty() {
                    ui.label("Type to find text in this document.");
                } else if results.matches.is_empty() {
                    ui.label("No results. Check spelling and case. Only complete characters match.");
                }
                let row_step = ROW_HEIGHT + ui.spacing().item_spacing.y;
                let mut scroll = egui::ScrollArea::vertical().id_salt("find-results").auto_shrink([false, false]);
                if let Some(index) = self.find_scroll.take() {
                    scroll = scroll.vertical_scroll_offset(index as f32 * row_step);
                }
                scroll.show_rows(ui, ROW_HEIGHT, results.matches.len(), |ui, rows| {
                    for index in rows {
                        let (before, matched, after) = results.excerpt(index);
                        let mut job = egui::text::LayoutJob::default();
                        job.wrap.max_rows = 3;
                        job.wrap.break_anywhere = true;
                        let format = |size, color| egui::TextFormat {font_id: FontId::new(size, egui::FontFamily::Name("body".into())), color, ..Default::default()};
                        job.append(&format!("{} · Paragraph {}\n", index + 1, results.matches[index].paragraph), 0., format(11., Color32::from_gray(100)));
                        job.append(&before, 0., format(13., INK));
                        job.append(&matched, 0., egui::TextFormat { background: chrome::SELECTION, font_id: FontId::new(13., egui::FontFamily::Name("body-bold".into())), ..format(13., INK) });
                        job.append(&after, 0., format(13., INK));
                        let selected = results.active(self.session.editor.selection) == Some(index);
                        let response = ui.add_sized([ui.available_width(), ROW_HEIGHT], Button::new(job).selected(selected).frame_when_inactive(false).wrap());
                        self.record(&format!("find-result-{index}"), response.rect);
                        if response.clicked() && ready { self.select_find_result(&results, index); }
                    }
                });
            });
        self.record("navigation", panel.response.rect);
    }
}
