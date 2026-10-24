//! Bounded read-only details shared by paragraph dialogs and the ruler.
use super::*;

pub(super) struct TabSourcesDialog {
    report: crate::tab_sources::Inspection,
    title: String,
    focus: bool,
    pub return_focus: Option<Id>,
    pub(super) scroll: f32,
}
impl TabSourcesDialog {
    pub fn new(report: crate::tab_sources::Inspection, title: &str, ctx: &egui::Context) -> Self {
        Self {
            report,
            title: title.into(),
            focus: true,
            return_focus: ctx.memory(|m| m.focused()),
            scroll: 0.,
        }
    }
    pub fn show(&mut self, ctx: &egui::Context, regions: &mut Vec<Region>) -> bool {
        let mut close = false;
        let id = Id::new("tab-sources-dialog");
        let modal = egui::Modal::new(id).area(egui::Modal::default_area(id).fade_in(false)).show(ctx, |ui| {
            ui.set_width((ctx.content_rect().width() - 64.).clamp(240., 420.));
            ui.heading(&self.title);
            ui.small(format!("{} paragraph(s) · read-only", self.report.count));
            if self.report.absent > 0 { ui.small(format!("Stop absent in {} paragraph(s).", self.report.absent)); }
            ui.separator();
            let height = (ctx.content_rect().height() - 180.).clamp(90., 300.);
            ui.input_mut(|i| {
                for (key, delta) in [(Key::ArrowDown, 24.), (Key::ArrowUp, -24.), (Key::PageDown, height - 24.), (Key::PageUp, 24. - height)] {
                    if i.consume_key(Modifiers::NONE, key) { self.scroll = (self.scroll + delta).max(0.); }
                }
                if i.consume_key(Modifiers::NONE, Key::Home) { self.scroll = 0.; }
                if i.consume_key(Modifiers::NONE, Key::End) { self.scroll = f32::MAX; }
            });
            let scroll = egui::ScrollArea::vertical().id_salt("tab-source-list").max_height(height).min_scrolled_height(60.).vertical_scroll_offset(self.scroll).show(ui, |ui| {
                for (name, property) in [("Tab interval", &self.report.interval), ("Tab positions", &self.report.positions)] {
                    ui.strong(name);
                    for group in &property.groups {
                        ui.add(egui::Label::new(format!("{} paragraph(s) — {}", group.count, group.label)).wrap());
                        ui.add(egui::Label::new(&group.value).wrap());
                        ui.add_space(4.);
                    }
                }
            });
            self.scroll = scroll.state.offset.y;
            regions.push(Region { name: "tab-sources-list".into(), rect: scroll.inner_rect });
            ui.separator();
            let response = ui.button("Close").on_hover_text("Escape closes. Arrows, Page Up/Down, Home/End or the wheel scroll the source list.");
            regions.push(Region { name: "tab-sources-close".into(), rect: response.rect });
            if self.focus { response.request_focus(); self.focus = false; }
            close = response.clicked();
        });
        regions.push(Region {
            name: "tab-sources-dialog".into(),
            rect: modal.response.rect,
        });
        close || ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape))
    }
    pub fn restore_focus(&self, ctx: &egui::Context) {
        if let Some(id) = self.return_focus {
            // The destination is hidden for this frame. Request it only when its UI
            // is rebuilt, otherwise AccessKit receives a focus ID without a node.
            ctx.data_mut(|d| d.insert_temp(Id::new("tab-sources-return-focus"), id));
            ctx.request_repaint();
        }
    }
}

impl DocsApp {
    pub(super) fn open_tab_sources(&mut self, ctx: &egui::Context, stop: Option<u16>) {
        // Live document provenance, never materialized preview overrides.
        match crate::tab_sources::inspect(
            &self.session.editor.document,
            self.session.editor.selection,
            stop,
        ) {
            Ok(report) => {
                self.tab_sources_dialog =
                    Some(TabSourcesDialog::new(report, "Tabulation sources", ctx));
                self.focus_document = false;
            }
            Err(error) => self.error = Some(error),
        }
    }
}
