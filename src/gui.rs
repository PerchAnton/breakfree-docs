//! Custom native office shell. Document changes always go through commands::Editor.
use crate::{
    Result, VERSION,
    chrome::{self, Icon},
    layout,
    model::{Alignment, FONT_SIZE_PRESETS, FontSize, boundaries},
    presentation::*,
    session::Session,
    storage,
};
use eframe::egui::{
    self, Align, Align2, Button, Color32, FontId, Id, Key, Layout, Modifiers, Rect, RichText,
    Sense, Stroke, Ui, Vec2, ViewportCommand, pos2,
    text::{CCursor, CCursorRange},
    vec2,
};
use std::path::PathBuf;

#[path = "font_dialog.rs"]
mod font_dialog;
use font_dialog::{FontDialog, FontDialogOutcome};

#[path = "ribbon_navigation.rs"]
mod ribbon_navigation;
use ribbon_navigation::{RibbonFocus, RibbonTarget, TargetKind};

#[path = "keytips.rs"]
mod keytips;
use keytips::KeyTips;

#[path = "zoom_dialog.rs"]
mod zoom_dialog;
use zoom_dialog::{ZoomDialog, ZoomOutcome};

const INK: Color32 = Color32::from_rgb(35, 35, 35);
const ACCENT: Color32 = Color32::from_rgb(112, 71, 174);
const CHROME: Color32 = Color32::from_rgb(243, 243, 243);
const WORKSPACE: Color32 = Color32::from_rgb(232, 232, 232);
const EDIT_ID: &str = "document-text";
const SEARCH_ID: &str = "command-search";
const FONT_SIZE_ID: &str = "font-size-input";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    File,
    New,
    Open,
    Save,
    SaveAs,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Bold,
    Italic,
    Underline,
    FontSize,
    FontDialog,
    GrowFont,
    ShrinkFont,
    Left,
    Center,
    Right,
    Justify,
    Find,
    Replace,
    SelectAll,
    Ribbon,
    Navigation,
    Rulers,
    Zoom,
    Formatting,
}
impl Action {
    fn label(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::New => "New document",
            Self::Open => "Open document",
            Self::Save => "Save",
            Self::SaveAs => "Save As",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::Cut => "Cut",
            Self::Copy => "Copy",
            Self::Paste => "Paste plain text",
            Self::Bold => "Bold",
            Self::Italic => "Italic",
            Self::Underline => "Underline",
            Self::FontSize => "Font size",
            Self::FontDialog => "Font dialog",
            Self::GrowFont => "Grow font",
            Self::ShrinkFont => "Shrink font",
            Self::Left => "Align left",
            Self::Center => "Center",
            Self::Right => "Align right",
            Self::Justify => "Justify",
            Self::Find => "Find",
            Self::Replace => "Replace",
            Self::SelectAll => "Select all",
            Self::Ribbon => "Collapse or expand ribbon",
            Self::Navigation => "Navigation pane",
            Self::Rulers => "Canvas rulers",
            Self::Zoom => "Zoom",
            Self::Formatting => "Formatting pane",
        }
    }
}
const COMMANDS: &[Action] = &[
    Action::File,
    Action::New,
    Action::Open,
    Action::Save,
    Action::SaveAs,
    Action::Undo,
    Action::Redo,
    Action::Cut,
    Action::Copy,
    Action::Paste,
    Action::Bold,
    Action::Italic,
    Action::Underline,
    Action::FontSize,
    Action::FontDialog,
    Action::GrowFont,
    Action::ShrinkFont,
    Action::Left,
    Action::Center,
    Action::Right,
    Action::Justify,
    Action::Find,
    Action::Replace,
    Action::SelectAll,
    Action::Ribbon,
    Action::Navigation,
    Action::Rulers,
    Action::Zoom,
    Action::Formatting,
];

enum Pending {
    New,
    Open(PathBuf),
    Close,
}
#[derive(Clone, Debug)]
pub struct Region {
    pub name: String,
    pub rect: Rect,
}

pub struct DocsApp {
    pub session: Session,
    pub tab: usize,
    pub ribbon_collapsed: bool,
    pub ribbon_temporary: bool,
    ribbon_focus: Option<RibbonFocus>,
    keytips: KeyTips,
    more_tabs_id: Option<Id>,
    zoom_dialog: Option<ZoomDialog>,
    ribbon_targets: Vec<RibbonTarget>,
    ribbon_menu_targets: Vec<RibbonTarget>,
    ribbon_menu_index: Option<usize>,
    ribbon_group: Option<usize>,
    collecting_menu: bool,
    status_focus: Option<Id>,
    formatting_focus: Option<Id>,
    pub navigation: bool,
    pub rulers: bool,
    pub zoom: f32,
    pub regions: Vec<Region>,
    buffer: String,
    composing: bool,
    cursor_override: Option<CCursorRange>,
    focus_document: bool,
    focus_search: bool,
    focus_find: bool,
    focus_font_size: bool,
    font_size_text: String,
    font_size_dirty: bool,
    font_dialog: Option<FontDialog>,
    pending: Option<Pending>,
    error: Option<String>,
    backstage: bool,
    settings: bool,
    formatting: bool,
    command_search: String,
    palette: bool,
    find: String,
    replacement: String,
    replace_mode: bool,
    body_font: String,
    icon: Option<egui::TextureHandle>,
    focus_zone: usize,
    allow_close: bool,
}

impl DocsApp {
    pub fn new(session: Session, ctx: &egui::Context) -> Self {
        let font = layout::configure_fonts(ctx);
        ctx.set_visuals(egui::Visuals::light());
        ctx.style_mut(|s| {
            s.spacing.item_spacing = vec2(4., 3.);
            s.spacing.button_padding = vec2(6., 3.);
            s.spacing.interact_size.y = 24.;
            s.visuals.panel_fill = CHROME;
            s.visuals.widgets.inactive.bg_fill = Color32::from_gray(248);
            s.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_gray(218));
            s.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_gray(185));
            s.visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
            s.visuals.widgets.inactive.corner_radius = 3.into();
            s.visuals.widgets.hovered.corner_radius = 3.into();
            s.visuals.widgets.active.corner_radius = 3.into();
            s.visuals.override_text_color = Some(INK);
            s.visuals.selection.bg_fill = Color32::from_rgb(222, 210, 239);
            s.visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);
            s.visuals.widgets.hovered.bg_fill = Color32::from_rgb(231, 231, 231);
            s.visuals.widgets.active.bg_fill = Color32::from_rgb(215, 215, 215);
            s.text_styles
                .insert(egui::TextStyle::Body, FontId::proportional(13.));
            s.text_styles
                .insert(egui::TextStyle::Button, FontId::proportional(13.));
            s.text_styles
                .insert(egui::TextStyle::Small, FontId::proportional(11.));
        });
        let image = image::load_from_memory(include_bytes!(
            "../docs/brand/breakfree-docs-transparent.webp"
        ))
        .ok()
        .map(|i| i.into_rgba8());
        let icon = image.map(|i| {
            ctx.load_texture(
                "BreakFree Docs logo",
                egui::ColorImage::from_rgba_unmultiplied(
                    [i.width() as usize, i.height() as usize],
                    i.as_raw(),
                ),
                Default::default(),
            )
        });
        let buffer = session.editor.document.text();
        Self {
            session,
            tab: 0,
            ribbon_collapsed: false,
            ribbon_temporary: false,
            ribbon_focus: None,
            keytips: KeyTips::default(),
            more_tabs_id: None,
            zoom_dialog: None,
            ribbon_targets: Vec::new(),
            ribbon_menu_targets: Vec::new(),
            ribbon_menu_index: None,
            ribbon_group: None,
            collecting_menu: false,
            status_focus: None,
            formatting_focus: None,
            navigation: false,
            rulers: true,
            zoom: 100.,
            regions: Vec::new(),
            buffer,
            composing: false,
            cursor_override: None,
            focus_document: true,
            focus_search: false,
            focus_find: false,
            focus_font_size: false,
            font_size_text: String::new(),
            font_size_dirty: false,
            font_dialog: None,
            pending: None,
            error: None,
            backstage: false,
            settings: false,
            formatting: false,
            command_search: String::new(),
            palette: false,
            find: String::new(),
            replacement: String::new(),
            replace_mode: false,
            body_font: font,
            icon,
            focus_zone: 0,
            allow_close: false,
        }
    }

    fn record(&mut self, name: &str, rect: Rect) {
        self.regions.push(Region {
            name: name.into(),
            rect,
        });
    }
    fn enabled(&self, action: Action) -> bool {
        if self.font_dialog.is_some() || self.zoom_dialog.is_some() {
            return false;
        }
        if matches!(action, Action::FontDialog | Action::Zoom)
            && (self.pending.is_some() || self.error.is_some())
        {
            return false;
        }
        if self.composing {
            return matches!(action, Action::Copy | Action::Ribbon | Action::Rulers);
        }
        match action {
            Action::Undo => !self.session.editor.undo_stack.is_empty(),
            Action::Redo => !self.session.editor.redo_stack.is_empty(),
            Action::Cut | Action::Copy => {
                self.session.editor.selection.0 != self.session.editor.selection.1
            }
            _ => true,
        }
    }
    fn checked(&self, action: Action) -> bool {
        match action {
            Action::Navigation => self.navigation,
            Action::Rulers => self.rulers,
            Action::Bold => self.session.editor.emphasis_selected(true),
            Action::Italic => self.session.editor.emphasis_selected(false),
            Action::Underline => self.session.editor.underline_selected(),
            Action::Left | Action::Center | Action::Right | Action::Justify => {
                let p = self
                    .session
                    .editor
                    .document
                    .locate(self.session.editor.selection.0)
                    .map(|p| p.0)
                    .unwrap_or(0);
                let a = self.session.editor.document.paragraphs[p].alignment;
                a == match action {
                    Action::Center => Alignment::Center,
                    Action::Right => Alignment::Right,
                    Action::Justify => Alignment::Justify,
                    _ => Alignment::Left,
                }
            }
            _ => false,
        }
    }
    fn sync_view(&mut self) {
        self.buffer = self.session.editor.document.text();
        let (a, b) = self.session.editor.selection;
        self.cursor_override = Some(CCursorRange::two(CCursor::new(b), CCursor::new(a)));
        self.focus_document = true;
    }
    fn fail(&mut self, result: Result<()>) {
        if let Err(e) = result {
            self.error = Some(e);
        }
    }
    fn choose_save(&mut self, save_as: bool) -> bool {
        let target = if !save_as && self.session.path.is_some() {
            self.session.path.clone()
        } else {
            rfd::FileDialog::new()
                .set_title("Save BreakFree Docs document")
                .add_filter("BreakFree Docs", &["bfdoc"])
                .set_file_name(
                    self.session
                        .path
                        .as_ref()
                        .and_then(|p| p.file_name())
                        .and_then(|p| p.to_str())
                        .unwrap_or("Untitled.bfdoc"),
                )
                .save_file()
                .map(|mut p| {
                    if p.extension().is_none() {
                        p.set_extension("bfdoc");
                    }
                    p
                })
        };
        let Some(path) = target else {
            return false;
        };
        match self.session.save(&path) {
            Ok(()) => true,
            Err(e) => {
                self.error = Some(e);
                false
            }
        }
    }
    fn request(&mut self, pending: Pending, ctx: &egui::Context) {
        if self.composing {
            self.error =
                Some("Finish or cancel text composition before changing documents.".into());
            return;
        }
        if self.session.dirty() {
            self.pending = Some(pending);
        } else {
            self.finish(pending, ctx);
        }
    }
    fn finish(&mut self, pending: Pending, ctx: &egui::Context) {
        match pending {
            Pending::New => self.session = Session::default(),
            Pending::Open(path) => match storage::load(&path) {
                Ok(document) => self.session = Session::new(document, Some(path)),
                Err(e) => {
                    self.error = Some(e);
                    return;
                }
            },
            Pending::Close => {
                self.allow_close = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
                return;
            }
        }
        self.backstage = false;
        self.font_size_dirty = false;
        self.sync_view();
    }
    pub fn execute(&mut self, action: Action, ctx: &egui::Context) {
        if !self.enabled(action) {
            return;
        }
        // A keyboard Save may leave the size field focused. Commit it before
        // dispatch, so Save/New/Undo cannot silently operate on an older model.
        if action != Action::FontSize && !self.commit_font_size_input() {
            return;
        }
        let (start, end) = self.session.editor.selection;
        match action {
            Action::File => self.backstage = true,
            Action::New => self.request(Pending::New, ctx),
            Action::Open => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Open BreakFree Docs document")
                    .add_filter("BreakFree Docs", &["bfdoc"])
                    .pick_file()
                {
                    match storage::load(&path) {
                        Ok(_) => self.request(Pending::Open(path), ctx),
                        Err(e) => self.error = Some(e),
                    }
                }
            }
            Action::Save | Action::SaveAs => {
                self.choose_save(action == Action::SaveAs);
            }
            Action::Undo => {
                self.session.editor.undo();
                self.sync_view();
            }
            Action::Redo => {
                self.session.editor.redo();
                self.sync_view();
            }
            Action::Copy | Action::Cut => {
                ctx.copy_text(selected_text(&self.session.editor.document, (start, end)));
                if action == Action::Cut {
                    let result = self.session.editor.replace(start, end, "", "Cut");
                    self.fail(result);
                    self.sync_view();
                }
            }
            Action::Paste => match arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
                Ok(text) => {
                    let text = text.replace("\r\n", "\n").replace('\r', "\n");
                    let result = self.session.editor.replace(start, end, &text, "Paste");
                    self.fail(result);
                    self.sync_view();
                }
                Err(e) => self.error = Some(format!("Cannot read text from clipboard: {e}")),
            },
            Action::Bold | Action::Italic => {
                let enabled = !self.checked(action);
                let result =
                    self.session
                        .editor
                        .emphasize(start, end, action == Action::Bold, enabled);
                self.fail(result);
                self.sync_view();
            }
            Action::Underline => {
                let result = self
                    .session
                    .editor
                    .underline(start, end, !self.checked(action));
                self.fail(result);
                self.sync_view();
            }
            Action::FontSize => {
                self.tab = 0;
                self.backstage = false;
                self.ribbon_temporary = self.ribbon_collapsed;
                self.ribbon_focus = Some(RibbonFocus::Control("font-size".into()));
                self.focus_font_size = true;
                self.focus_document = false;
            }
            Action::FontDialog => {
                self.font_dialog = Some(FontDialog::new(self.session.editor.selected_font()));
                self.focus_document = false;
                self.focus_font_size = false;
                self.palette = false;
                ctx.memory_mut(|m| m.surrender_focus(Id::new(EDIT_ID)));
            }
            Action::GrowFont | Action::ShrinkFont => {
                let result =
                    self.session
                        .editor
                        .step_font_size(start, end, action == Action::GrowFont);
                self.fail(result);
                self.sync_view();
            }
            Action::Left | Action::Center | Action::Right | Action::Justify => {
                let a = match action {
                    Action::Center => Alignment::Center,
                    Action::Right => Alignment::Right,
                    Action::Justify => Alignment::Justify,
                    _ => Alignment::Left,
                };
                let result = self.session.editor.align(start, end, a);
                self.fail(result);
                self.sync_view();
            }
            Action::SelectAll => {
                self.session.editor.selection = (0, self.session.editor.document.character_count());
                self.sync_view();
            }
            Action::Find | Action::Replace => {
                self.navigation = true;
                self.replace_mode = action == Action::Replace;
                self.focus_find = true;
            }
            Action::Ribbon => {
                self.ribbon_collapsed = !self.ribbon_collapsed;
                self.ribbon_temporary = false;
                self.ribbon_focus = None;
                self.focus_document = true;
                egui::Popup::close_all(ctx);
            }
            Action::Zoom => {
                self.zoom_dialog = Some(ZoomDialog::new(self.zoom));
                self.focus_document = false;
                self.palette = false;
                ctx.memory_mut(|m| {
                    if let Some(id) = m.focused() {
                        m.surrender_focus(id);
                    }
                });
            }
            Action::Navigation => self.navigation = !self.navigation,
            Action::Rulers => self.rulers = !self.rulers,
            Action::Formatting => self.formatting = !self.formatting,
        }
        if action != Action::FontSize && action != Action::Ribbon {
            self.ribbon_temporary = false;
            self.ribbon_focus = None;
            self.ribbon_menu_index = None;
            egui::Popup::close_all(ctx);
            if !matches!(
                action,
                Action::Find | Action::Replace | Action::FontDialog | Action::Zoom | Action::File
            ) {
                self.focus_document = true;
            }
        }
    }

    fn command(&mut self, ui: &mut Ui, label: &str, action: Action, size: Vec2) {
        let enabled = self.enabled(action);
        let glyph = matches!(
            action,
            Action::Left | Action::Center | Action::Right | Action::Justify
        ) && size.x <= 32.0
            || matches!(action, Action::Undo | Action::Redo | Action::Ribbon) && size.x <= 32.0;
        let text = if glyph {
            RichText::new("")
        } else if action == Action::Bold && label == "B" {
            RichText::new(label).strong()
        } else if action == Action::Italic && label == "I" {
            RichText::new(label).italics()
        } else if action == Action::Underline {
            RichText::new(label).underline()
        } else {
            RichText::new(label)
        };
        let button = Button::new(text).selected(self.checked(action));
        let response = ui
            .add_enabled_ui(enabled, |ui| ui.add_sized(size, button))
            .inner;
        self.record(&format!("command-{action:?}"), response.rect);
        if action == Action::Bold && !self.collecting_menu {
            self.formatting_focus = Some(response.id);
        }
        if self.ribbon_group.is_some() {
            self.track_ribbon(
                &response,
                &format!("command-{action:?}"),
                TargetKind::Action(action),
            );
        }
        if glyph {
            let c = response.rect.center();
            let color = if enabled {
                INK
            } else {
                Color32::from_gray(170)
            };
            let stroke = Stroke::new(1.2_f32, color);
            let painter = ui.painter();
            match action {
                Action::Left | Action::Center | Action::Right | Action::Justify => {
                    for (i, length) in [14.0, 10.0, 14.0, 8.0].into_iter().enumerate() {
                        let len = if action == Action::Justify {
                            14.0
                        } else {
                            length
                        };
                        let x = match action {
                            Action::Center => c.x - len / 2.0,
                            Action::Right => c.x + 7.0 - len,
                            _ => c.x - 7.0,
                        };
                        let y = c.y - 5.0 + i as f32 * 3.5;
                        painter.line_segment([pos2(x, y), pos2(x + len, y)], stroke);
                    }
                }
                Action::Undo | Action::Redo => {
                    let d = if action == Action::Undo { 1.0 } else { -1.0 };
                    let points = vec![
                        c + vec2(6.0 * d, 5.0),
                        c + vec2(6.0 * d, -1.0),
                        c + vec2(2.0 * d, -5.0),
                        c + vec2(-6.0 * d, -5.0),
                    ];
                    painter.add(egui::Shape::line(points, stroke));
                    painter
                        .line_segment([c + vec2(-2.0 * d, -9.0), c + vec2(-6.0 * d, -5.0)], stroke);
                    painter
                        .line_segment([c + vec2(-2.0 * d, -1.0), c + vec2(-6.0 * d, -5.0)], stroke);
                }
                Action::Ribbon => {
                    let d = if self.ribbon_collapsed { -1.0 } else { 1.0 };
                    painter.add(egui::Shape::line(
                        vec![
                            c + vec2(-4.0, 2.0 * d),
                            c + vec2(0.0, -2.0 * d),
                            c + vec2(4.0, 2.0 * d),
                        ],
                        stroke,
                    ));
                }
                _ => {}
            }
        }
        let tip = if self.composing && !enabled {
            "Finish text composition first."
        } else {
            action.label()
        };
        if response
            .on_disabled_hover_text(tip)
            .on_hover_text(tip)
            .clicked()
        {
            self.execute(action, ui.ctx());
        }
    }
    fn disabled(ui: &mut Ui, label: &str, size: Vec2, why: &str) {
        ui.add_enabled_ui(false, |ui| ui.add_sized(size, Button::new(label)))
            .inner
            .on_disabled_hover_text(why);
    }

    fn title(&mut self, ctx: &egui::Context) {
        let panel = egui::TopBottomPanel::top("title")
            .exact_height(TITLE_HEIGHT)
            .frame(egui::Frame::new().fill(CHROME).inner_margin(0))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let w = rect.width();
                let drag = ui.interact(rect, Id::new("window-drag"), Sense::click_and_drag());
                if drag.drag_started() {
                    ctx.send_viewport_cmd(ViewportCommand::StartDrag);
                }
                if drag.double_clicked() {
                    let max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                    ctx.send_viewport_cmd(ViewportCommand::Maximized(!max));
                }
                if let Some(icon) = &self.icon {
                    ui.put(
                        Rect::from_min_size(rect.min + vec2(10., 10.), vec2(24., 24.)),
                        egui::Image::new(icon).fit_to_exact_size(vec2(24., 24.)),
                    );
                }
                for (i, (icon, action)) in [
                    (Icon::Save, Action::Save),
                    (Icon::Undo, Action::Undo),
                    (Icon::Redo, Action::Redo),
                ]
                .into_iter()
                .enumerate()
                {
                    let qat_rect = Rect::from_min_size(
                        rect.min + vec2(43. + i as f32 * 30., 8.),
                        vec2(28., 28.),
                    );
                    self.record(&format!("qat-{action:?}"), qat_rect);
                    self.ribbon_control(ui, qat_rect, icon, "", action);
                }
                let search_w = (w * 0.25).clamp(155., 350.);
                let search_x = (w * 0.56 - search_w / 2.).max(285.);
                let name = self
                    .session
                    .path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or("Untitled".into());
                let title = format!(
                    "{}{} — BreakFree Docs",
                    name,
                    if self.session.dirty() { " *" } else { "" }
                );
                let title_rect = Rect::from_min_max(
                    rect.min + vec2(147., 8.),
                    pos2(search_x - 16., rect.top() + 36.),
                );
                ui.put(title_rect, egui::Label::new(&title).truncate())
                    .on_hover_text(&title);
                self.record("document-title", title_rect);
                let search_rect = Rect::from_min_size(
                    rect.min + vec2(search_x, 8.),
                    vec2(search_w.min(w - search_x - 150.), 28.),
                );
                ui.painter()
                    .rect_filled(search_rect, 4., Color32::from_gray(250));
                ui.painter().rect_stroke(
                    search_rect,
                    4.,
                    Stroke::new(1.0_f32, Color32::from_gray(219)),
                    egui::StrokeKind::Inside,
                );
                chrome::paint(
                    ui,
                    Icon::Search,
                    search_rect.left_center() + vec2(14., 0.),
                    15.,
                    Color32::from_gray(95),
                );
                let response = ui.put(
                    Rect::from_min_max(
                        search_rect.min + vec2(28., 4.),
                        search_rect.max - vec2(5., 2.),
                    ),
                    egui::TextEdit::singleline(&mut self.command_search)
                        .id(Id::new(SEARCH_ID))
                        .frame(false)
                        .hint_text("Search (Alt+Q)"),
                );
                self.record("command-search", search_rect);
                if self.focus_search {
                    response.request_focus();
                    self.focus_search = false;
                }
                if response.changed() {
                    self.palette = !self.command_search.is_empty();
                }
                let max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                for (i, (icon, name)) in [
                    (Icon::Minimize, "Minimize"),
                    (Icon::Maximize(max), "Maximize / restore"),
                    (Icon::Close, "Close"),
                ]
                .into_iter()
                .enumerate()
                {
                    let r = Rect::from_min_size(
                        pos2(rect.right() - 138. + i as f32 * 46., rect.top()),
                        vec2(46., TITLE_HEIGHT),
                    );
                    let response = chrome::control(ui, r, icon, "", name, true, false);
                    self.record(&format!("window-{i}"), response.rect);
                    if response.on_hover_text(name).clicked() {
                        match i {
                            0 => ctx.send_viewport_cmd(ViewportCommand::Minimized(true)),
                            1 => ctx.send_viewport_cmd(ViewportCommand::Maximized(!max)),
                            _ => self.request(Pending::Close, ctx),
                        }
                    }
                }
            });
        self.record("title", panel.response.rect);
    }

    fn tabs(&mut self, ctx: &egui::Context) {
        let panel = egui::TopBottomPanel::top("tabs")
            .exact_height(TABS_HEIGHT)
            .frame(
                egui::Frame::new()
                    .fill(CHROME)
                    .inner_margin(egui::Margin::symmetric(8, 2)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let file = ui.add_sized(
                        [44., 26.],
                        Button::new(RichText::new("File").color(INK)).frame_when_inactive(false),
                    );
                    self.record("tab-File", file.rect);
                    if file.clicked() {
                        self.backstage = !self.backstage;
                    }
                    let full = ui.available_width() > 770.;
                    let visible = if full {
                        TABS.len()
                    } else {
                        ((ui.available_width() - 95.) / 70.).floor().max(1.) as usize
                    };
                    for (index, label) in TABS.iter().enumerate().take(visible) {
                        let response = ui.add_sized(
                            [label.len() as f32 * 6.5 + 16., 26.],
                            Button::new(RichText::new(*label).color(
                                if self.tab == index && !self.backstage {
                                    ACCENT
                                } else {
                                    INK
                                },
                            ))
                            .frame_when_inactive(false),
                        );
                        self.record(&format!("tab-{label}"), response.rect);
                        if self.ribbon_focus == Some(RibbonFocus::Tab) && self.tab == index {
                            response.request_focus();
                        }
                        if response.clicked() {
                            self.tab = index;
                            self.backstage = false;
                            self.ribbon_temporary = self.ribbon_collapsed;
                            self.ribbon_focus = Some(RibbonFocus::Tab);
                            self.focus_document = false;
                            response.request_focus();
                        }
                        if response.double_clicked() {
                            self.execute(Action::Ribbon, ctx);
                        }
                        response.context_menu(|ui| {
                            if ui
                                .button(if self.ribbon_collapsed {
                                    "Always show ribbon"
                                } else {
                                    "Show tabs only"
                                })
                                .clicked()
                            {
                                self.execute(Action::Ribbon, ctx);
                                ui.close();
                            }
                        });
                        if self.tab == index && !self.backstage {
                            let r = response.rect;
                            ui.painter().line_segment(
                                [r.left_bottom(), r.right_bottom()],
                                Stroke::new(2.0_f32, ACCENT),
                            );
                        }
                    }
                    if visible < TABS.len() {
                        let more = ui.menu_button("More tabs", |ui| {
                            for (index, label) in TABS.iter().enumerate().skip(visible) {
                                let response = ui.add_sized([150., 26.], Button::new(*label));
                                self.record(&format!("overflow-tab-{label}"), response.rect);
                                if response.clicked() {
                                    self.tab = index;
                                    self.backstage = false;
                                    self.ribbon_temporary = self.ribbon_collapsed;
                                    self.ribbon_focus = Some(RibbonFocus::Tab);
                                    self.focus_document = false;
                                    ui.close();
                                }
                            }
                        });
                        self.more_tabs_id = Some(more.response.id);
                        self.record("more-tabs", more.response.rect);
                        if self.tab >= visible && self.ribbon_focus == Some(RibbonFocus::Tab) {
                            more.response.request_focus();
                        }
                    } else {
                        self.more_tabs_id = None;
                        if self.keytips.level == Some(keytips::Level::Tabs) {
                            egui::Popup::close_all(ctx);
                            self.keytips.level = Some(keytips::Level::Root);
                        }
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        self.command(
                            ui,
                            if self.ribbon_collapsed { "⌄" } else { "⌃" },
                            Action::Ribbon,
                            vec2(26., 24.),
                        );
                    });
                });
            });
        self.record("tabs", panel.response.rect);
    }

    fn ribbon_control(&mut self, ui: &mut Ui, rect: Rect, icon: Icon, label: &str, action: Action) {
        let response = chrome::control(
            ui,
            rect,
            icon,
            label,
            action.label(),
            self.enabled(action),
            self.checked(action),
        );
        self.record(&format!("command-{action:?}"), response.rect);
        self.track_ribbon(
            &response,
            &format!("command-{action:?}"),
            TargetKind::Action(action),
        );
        let shortcut = match action {
            Action::Save => "Ctrl+S",
            Action::Undo => "Ctrl+Z",
            Action::Redo => "Ctrl+Y",
            Action::Cut => "Ctrl+X",
            Action::Copy => "Ctrl+C",
            Action::Paste => "Ctrl+V",
            Action::Bold => "Ctrl+B",
            Action::Italic => "Ctrl+I",
            Action::Underline => "Ctrl+U",
            Action::FontSize => "Ctrl+Shift+P",
            Action::FontDialog => "Ctrl+D",
            Action::GrowFont => "Ctrl+Shift+>",
            Action::ShrinkFont => "Ctrl+Shift+<",
            Action::Find => "Ctrl+F",
            Action::Replace => "Ctrl+H",
            Action::SelectAll => "Ctrl+A",
            Action::Left => "Ctrl+L",
            Action::Center => "Ctrl+E",
            Action::Right => "Ctrl+R",
            Action::Justify => "Ctrl+J",
            _ => "",
        };
        let shortcut = if cfg!(target_os = "macos") {
            shortcut.replace("Ctrl", "Command")
        } else {
            shortcut.to_owned()
        };
        let tip = format!(
            "{}{}{}",
            action.label(),
            if shortcut.is_empty() { "" } else { " · " },
            shortcut
        );
        if response
            .on_hover_text(&tip)
            .on_disabled_hover_text(if self.composing {
                "Finish text composition first."
            } else {
                &tip
            })
            .clicked()
        {
            self.execute(action, ui.ctx());
        }
    }

    fn unavailable(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        icon: Icon,
        label: &str,
        name: &str,
        why: &str,
    ) {
        let response =
            chrome::control(ui, rect, icon, label, name, false, false).on_disabled_hover_text(why);
        self.record(&format!("unavailable-{name}"), response.rect);
    }

    fn home_group(&mut self, ui: &mut Ui, index: usize, width: f32) {
        let (bounds, _) = ui.allocate_exact_size(vec2(width, RIBBON_HEIGHT - 16.), Sense::hover());
        self.record(HOME_GROUPS[index], bounds);
        self.ribbon_group = Some(index);
        ui.scope_builder(egui::UiBuilder::new().max_rect(bounds), |ui| {
            self.home_contents(ui, index, width, bounds);
        });
        self.ribbon_group = None;
        if index < 4 {
            let x = bounds.right() + 3.;
            ui.painter().line_segment(
                [pos2(x, bounds.top() + 3.), pos2(x, bounds.bottom() - 3.)],
                Stroke::new(1.0_f32, chrome::LINE),
            );
        }
    }

    fn commit_font_size_input(&mut self) -> bool {
        if !self.font_size_dirty {
            return true;
        }
        match self.font_size_text.parse::<FontSize>() {
            Ok(size) => {
                self.apply_font_size(size);
                true
            }
            Err(error) => {
                self.error = Some(error);
                self.font_size_dirty = false;
                false
            }
        }
    }

    fn apply_font_size(&mut self, size: FontSize) {
        if self.composing {
            return;
        }
        let (start, end) = self.session.editor.selection;
        let result = self.session.editor.set_font_size(start, end, size);
        self.fail(result);
        self.font_size_dirty = false;
        self.sync_view();
    }

    fn font_size_control(&mut self, ui: &mut Ui, rect: Rect) {
        let id = Id::new(FONT_SIZE_ID);
        let focused = ui.memory(|m| m.has_focus(id));
        if !focused && !self.font_size_dirty {
            self.font_size_text = self
                .session
                .editor
                .selected_font_size()
                .map(|size| size.to_string())
                .unwrap_or_default();
        }
        ui.painter().rect_filled(rect, 2., Color32::WHITE);
        ui.painter().rect_stroke(
            rect,
            2.,
            Stroke::new(1.0_f32, if focused { ACCENT } else { chrome::LINE }),
            egui::StrokeKind::Inside,
        );
        let edit_rect = Rect::from_min_max(rect.min, pos2(rect.right() - 18., rect.bottom()));
        let response = ui
            .add_enabled_ui(!self.composing, |ui| {
                ui.put(
                    edit_rect,
                    egui::TextEdit::singleline(&mut self.font_size_text)
                        .id(id)
                        .font(FontId::proportional(13.))
                        .desired_width(0.)
                        .frame(false)
                        .margin(vec2(4., 4.))
                        .hint_text("—")
                        .char_limit(8),
                )
            })
            .inner;
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::TextEdit,
                !self.composing,
                "Font size in points",
            )
        });
        self.record("font-size", response.rect);
        self.track_ribbon(&response, "font-size", TargetKind::Text);
        if self.focus_font_size && !self.composing {
            response.request_focus();
            let mut state = egui::TextEdit::load_state(ui.ctx(), id).unwrap_or_default();
            state.cursor.set_char_range(Some(CCursorRange::two(
                CCursor::new(0),
                CCursor::new(self.font_size_text.chars().count()),
            )));
            state.store(ui.ctx(), id);
            self.focus_font_size = false;
        }
        if response.changed() {
            self.font_size_dirty = true;
        }
        if !self.composing && self.font_size_dirty && response.lost_focus() {
            if self.commit_font_size_input() && !ui.input(|i| i.key_pressed(Key::Enter)) {
                // Preserve the destination of Tab or a pointer focus change.
                self.focus_document = false;
            }
        } else if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
            self.sync_view();
        }
        if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
            self.leave_ribbon(ui.ctx());
        }
        response.on_hover_text("Font size in points (1–400, steps of 0.5). Enter or leave the field to apply; Escape cancels. Blank means mixed sizes.")
            .on_disabled_hover_text("Finish text composition first.");
        let arrow = chrome::control(
            ui,
            Rect::from_min_max(pos2(rect.right() - 18., rect.top()), rect.max),
            Icon::Chevron(false),
            "",
            "Font size presets",
            !self.composing,
            false,
        );
        self.record("font-size-menu", arrow.rect);
        self.track_ribbon(&arrow, "font-size-menu", TargetKind::Menu);
        egui::Popup::menu(&arrow).show(|ui| {
            self.collecting_menu = true;
            egui::ScrollArea::vertical()
                .max_height(280.)
                .show(ui, |ui| {
                    for &points in FONT_SIZE_PRESETS {
                        let size = FontSize::try_from(points * 2).expect("valid font preset");
                        let response = ui.selectable_label(
                            self.session.editor.selected_font_size() == Some(size),
                            size.to_string(),
                        );
                        self.record(&format!("size-preset-{points}"), response.rect);
                        self.track_ribbon(
                            &response,
                            &format!("size-preset-{points}"),
                            TargetKind::Preset(size),
                        );
                        if response.clicked() {
                            self.apply_font_size(size);
                            self.leave_ribbon(ui.ctx());
                            ui.close();
                        }
                    }
                });
            self.collecting_menu = false;
        });
    }

    fn unavailable_menu(&mut self, ui: &mut Ui, rect: Rect, icon: Icon, name: &str, why: &str) {
        let response =
            chrome::dropdown(ui, rect, icon, "", name, false).on_disabled_hover_text(why);
        self.record(&format!("unavailable-{name}"), response.rect);
    }

    fn paste_control(&mut self, ui: &mut Ui, rect: Rect) {
        let enabled = self.enabled(Action::Paste);
        let color = if enabled {
            chrome::INK
        } else {
            Color32::from_gray(150)
        };
        let top = Rect::from_min_size(rect.min, vec2(rect.width(), 37.));
        let response = chrome::button(ui, top, Action::Paste.label(), enabled, false);
        self.record("command-Paste", response.rect);
        self.track_ribbon(
            &response,
            "command-Paste",
            TargetKind::Action(Action::Paste),
        );
        chrome::paint(ui, Icon::Paste, top.center(), 28., color);
        if response
            .on_hover_text("Paste plain text · Ctrl/Command+V")
            .on_disabled_hover_text("Finish text composition first.")
            .clicked()
        {
            self.execute(Action::Paste, ui.ctx());
        }
        let bottom = Rect::from_min_max(top.left_bottom(), rect.max);
        let menu = chrome::button(ui, bottom, "Paste options", enabled, false)
            .on_hover_text("Paste options");
        self.record("home-menu-Paste", menu.rect);
        self.track_ribbon(&menu, "home-menu-Paste", TargetKind::Menu);
        ui.painter().text(
            bottom.center_top() + vec2(0., 9.),
            Align2::CENTER_CENTER,
            "Paste",
            FontId::proportional(12.),
            color,
        );
        chrome::paint(
            ui,
            Icon::Chevron(false),
            bottom.center_bottom() - vec2(0., 6.),
            9.,
            color,
        );
        egui::Popup::menu(&menu).show(|ui| {
            self.collecting_menu = true;
            self.command(ui, "Paste plain text", Action::Paste, vec2(180., 27.));
            ui.separator();
            Self::disabled(ui, "Paste Special…", vec2(180., 27.),
                "Rich clipboard formats are not implemented. Paste inserts plain text using the current typing style.");
            self.collecting_menu = false;
        });
    }

    fn editing_menu(&mut self, ui: &mut Ui, rect: Rect, find: bool) {
        let name = if find { "Find" } else { "Select" };
        // Find's primary target opens navigation immediately; its arrow owns the menu.
        let rect = if find {
            let primary = Rect::from_min_max(rect.min, pos2(rect.right() - 18., rect.bottom()));
            self.ribbon_control(ui, primary, Icon::Search, "Find", Action::Find);
            Rect::from_min_max(primary.right_top(), rect.max)
        } else {
            rect
        };
        let response = if find {
            chrome::control(
                ui,
                rect,
                Icon::Chevron(false),
                "",
                "Find options",
                self.enabled(Action::Find),
                false,
            )
        } else {
            chrome::dropdown(
                ui,
                rect,
                Icon::Select,
                name,
                name,
                self.enabled(Action::SelectAll),
            )
        }
        .on_hover_text(if find {
            "Find and navigation"
        } else {
            "Selection commands"
        });
        self.record(&format!("home-menu-{name}"), response.rect);
        self.track_ribbon(&response, &format!("home-menu-{name}"), TargetKind::Menu);
        egui::Popup::menu(&response).show(|ui| {
            self.collecting_menu = true;
            if find {
                self.command(ui, "Find", Action::Find, vec2(160., 27.));
                self.command(ui, "Navigation pane", Action::Navigation, vec2(160., 27.));
            } else {
                self.command(ui, "Select all", Action::SelectAll, vec2(160., 27.));
                Self::disabled(
                    ui,
                    "Select Objects",
                    vec2(160., 27.),
                    "Document objects are not implemented.",
                );
            }
            self.collecting_menu = false;
        });
    }

    fn home_contents(&mut self, ui: &mut Ui, index: usize, width: f32, bounds: Rect) {
        let origin = bounds.min;
        let r =
            |x: f32, y: f32, w: f32, h: f32| Rect::from_min_size(origin + vec2(x, y), vec2(w, h));
        ui.set_clip_rect(bounds.intersect(ui.clip_rect()));
        match index {
            0 => {
                self.paste_control(ui, r(0., 1., 46., 69.));
                self.ribbon_control(ui, r(49., 0., 27., 23.), Icon::Cut, "", Action::Cut);
                self.ribbon_control(ui, r(49., 24., 27., 23.), Icon::Copy, "", Action::Copy);
                self.unavailable(
                    ui,
                    r(49., 48., 27., 23.),
                    Icon::Brush,
                    "",
                    "Format Painter",
                    "Format Painter is planned; use the supported direct-formatting controls.",
                );
            }
            1 => {
                let full = width >= 300.;
                let family = if full { 112. } else { width - 182. };
                let response = chrome::field(
                    ui,
                    r(2., 5., family, 25.),
                    &self.body_font,
                    "Current body font. Changing fonts is planned.",
                );
                self.record("font-family", response.rect);
                self.font_size_control(ui, r(family + 4., 5., 64., 25.));
                let x = family + 72.;
                self.ribbon_control(
                    ui,
                    r(x, 5., 25., 25.),
                    Icon::FontStep(true),
                    "",
                    Action::GrowFont,
                );
                self.ribbon_control(
                    ui,
                    r(x + 26., 5., 25., 25.),
                    Icon::FontStep(false),
                    "",
                    Action::ShrinkFont,
                );
                self.unavailable_menu(
                    ui,
                    r(x + 56., 5., 32., 25.),
                    Icon::Text("Aa"),
                    "Change case",
                    "Changing letter case is planned.",
                );
                if full {
                    self.unavailable(
                        ui,
                        r(x + 92., 5., 25., 25.),
                        Icon::ClearFormatting,
                        "",
                        "Clear formatting",
                        "Clearing all formatting is planned.",
                    );
                }
                self.ribbon_control(ui, r(0., 34., 24., 25.), Icon::Text("B"), "", Action::Bold);
                self.ribbon_control(
                    ui,
                    r(25., 34., 24., 25.),
                    Icon::Text("I"),
                    "",
                    Action::Italic,
                );
                self.ribbon_control(
                    ui,
                    r(50., 34., 24., 25.),
                    Icon::Text("U"),
                    "",
                    Action::Underline,
                );
                self.unavailable(
                    ui,
                    r(75., 34., 11., 25.),
                    Icon::Chevron(false),
                    "",
                    "Underline styles",
                    "Single underline is supported. Other underline styles are planned.",
                );
                for (i, (text, name)) in [
                    ("abc", "Strikethrough"),
                    ("x₂", "Subscript"),
                    ("x²", "Superscript"),
                ]
                .into_iter()
                .enumerate()
                {
                    self.unavailable(
                        ui,
                        r(89. + i as f32 * 26., 34., 25., 25.),
                        Icon::Text(text),
                        "",
                        name,
                        "This character formatting is not implemented.",
                    );
                }
                ui.painter().line_segment(
                    [origin + vec2(170., 36.), origin + vec2(170., 57.)],
                    Stroke::new(1.0_f32, chrome::LINE),
                );
                self.unavailable_menu(
                    ui,
                    r(176., 34., 32., 25.),
                    Icon::TextEffects,
                    "Text effects",
                    "Text effects are planned.",
                );
                if width >= 246. {
                    self.unavailable_menu(
                        ui,
                        r(212., 34., 32., 25.),
                        Icon::Highlight,
                        "Text highlight",
                        "Text highlighting is planned.",
                    );
                }
                if full {
                    self.unavailable_menu(
                        ui,
                        r(248., 34., 32., 25.),
                        Icon::FontColor,
                        "Font color",
                        "Font colors are planned.",
                    );
                }
            }
            2 => {
                for (i, (icon, name)) in [
                    (Icon::Bullets, "Bullets"),
                    (Icon::Numbers, "Numbering"),
                    (Icon::Multilevel, "Multilevel list"),
                ]
                .into_iter()
                .enumerate()
                {
                    self.unavailable_menu(
                        ui,
                        r(i as f32 * 31., 5., 30., 25.),
                        icon,
                        name,
                        "Lists are planned.",
                    );
                }
                ui.painter().line_segment(
                    [origin + vec2(95., 7.), origin + vec2(95., 28.)],
                    Stroke::new(1.0_f32, chrome::LINE),
                );
                for (i, (icon, name)) in [
                    (Icon::Indent(false), "Decrease indent"),
                    (Icon::Indent(true), "Increase indent"),
                    (Icon::Sort, "Sort"),
                    (Icon::Text("¶"), "Show marks"),
                ]
                .into_iter()
                .enumerate()
                {
                    self.unavailable(
                        ui,
                        r(99. + i as f32 * 26., 5., 25., 25.),
                        icon,
                        "",
                        name,
                        "Indentation, sorting and formatting marks are planned.",
                    );
                }
                for (i, action) in [Action::Left, Action::Center, Action::Right, Action::Justify]
                    .into_iter()
                    .enumerate()
                {
                    self.ribbon_control(
                        ui,
                        r(i as f32 * 25., 34., 24., 25.),
                        Icon::Align(i as u8),
                        "",
                        action,
                    );
                }
                self.unavailable_menu(
                    ui,
                    r(101., 34., 31., 25.),
                    Icon::Spacing,
                    "Line spacing",
                    "Paragraph spacing controls are planned.",
                );
                ui.painter().line_segment(
                    [origin + vec2(135., 36.), origin + vec2(135., 57.)],
                    Stroke::new(1.0_f32, chrome::LINE),
                );
                self.unavailable_menu(
                    ui,
                    r(139., 34., 31., 25.),
                    Icon::Shading,
                    "Shading",
                    "Paragraph shading is planned.",
                );
                self.unavailable_menu(
                    ui,
                    r(174., 34., 31., 25.),
                    Icon::Border,
                    "Borders",
                    "Paragraph borders are planned.",
                );
            }
            3 => {
                // Names are the preview, matching the reference's compact gallery.
                // No selected frame: named styles do not exist in the model yet.
                let gallery = r(0., 3., width - 1., 63.);
                ui.painter().rect_stroke(
                    gallery,
                    2.,
                    Stroke::new(1.0_f32, chrome::LINE),
                    egui::StrokeKind::Inside,
                );
                let count = if width >= 280. { 3 } else { 1 };
                let card = (width - 19.) / count as f32;
                for (i, label) in ["Normal", "No Spacing", "Heading 1"]
                    .into_iter()
                    .take(count)
                    .enumerate()
                {
                    let rect = r(2. + i as f32 * card, 5., card - 3., 59.);
                    let response = chrome::button(ui, rect, label, false, false)
                        .on_disabled_hover_text(
                            "Named styles are planned. This preview does not apply a style.",
                        );
                    self.record(&format!("style-{label}"), response.rect);
                    ui.painter().with_clip_rect(rect).text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        label,
                        FontId::new(
                            if i == 2 { 18. } else { 12. },
                            egui::FontFamily::Name("body".into()),
                        ),
                        Color32::from_gray(135),
                    );
                }
                for (i, (icon, name)) in [
                    (Icon::Chevron(true), "Previous styles"),
                    (Icon::Chevron(false), "Next styles"),
                    (Icon::Chevron(false), "More styles"),
                ]
                .into_iter()
                .enumerate()
                {
                    let rect = r(width - 17., 4. + i as f32 * 20., 15., 19.);
                    self.unavailable(ui, rect, icon, "", name, "Named styles are planned.");
                }
            }
            4 => {
                self.editing_menu(ui, r(0., 0., width - 3., 23.), true);
                self.ribbon_control(
                    ui,
                    r(0., 24., width - 3., 23.),
                    Icon::Replace,
                    "Replace",
                    Action::Replace,
                );
                self.editing_menu(ui, r(0., 48., width - 3., 23.), false);
            }
            _ => {}
        }
        ui.painter().text(
            pos2(origin.x + width / 2., origin.y + 78.),
            Align2::CENTER_CENTER,
            HOME_GROUPS[index],
            FontId::proportional(11.),
            Color32::from_gray(94),
        );
        if matches!(index, 1 | 2) {
            self.ribbon_control(
                ui,
                r(width - 16., 71., 15., 16.),
                Icon::Launch,
                "",
                if index == 1 {
                    Action::FontDialog
                } else {
                    Action::Formatting
                },
            );
        }
    }
    fn ribbon_contents(&mut self, ui: &mut Ui) {
        let shell = ui.max_rect();
        ui.painter().rect_filled(shell, 6., Color32::WHITE);
        ui.painter().rect_stroke(
            shell,
            6.,
            Stroke::new(1.0_f32, Color32::from_gray(225)),
            egui::StrokeKind::Inside,
        );
        ui.shrink_clip_rect(shell);
        ui.add_space(4.);
        if self.tab == 0 {
            let widths = group_widths(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.;
                ui.add_space(5.);
                for (i, w) in widths.into_iter().enumerate() {
                    if w > 0. {
                        self.home_group(ui, i, w);
                    }
                }
                if widths[3] == 0. {
                    self.ribbon_group = Some(4);
                    let menu = ui.menu_button("More", |ui| {
                        self.collecting_menu = true;
                        ui.label("Styles");
                        for name in ["Normal", "Heading 1"] {
                            Self::disabled(
                                ui,
                                name,
                                vec2(100., 24.),
                                "Styles are not implemented.",
                            );
                        }
                        ui.separator();
                        for action in [
                            Action::Find,
                            Action::Replace,
                            Action::SelectAll,
                            Action::Formatting,
                        ] {
                            self.command(ui, action.label(), action, vec2(145., 25.));
                        }
                        self.collecting_menu = false;
                    });
                    self.record("home-overflow", menu.response.rect);
                    self.track_ribbon(&menu.response, "home-overflow", TargetKind::Menu);
                    self.ribbon_group = None;
                }
            });
        } else if self.tab == 8 {
            self.ribbon_group = Some(0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    Self::disabled(
                        ui,
                        "Print Layout",
                        vec2(110., 30.),
                        "Pagination is not implemented. The document uses a continuous canvas.",
                    );
                    Self::disabled(
                        ui,
                        "Read Mode",
                        vec2(110., 30.),
                        "Read Mode is not implemented.",
                    );
                    ui.small("Views");
                });
                ui.separator();
                ui.vertical(|ui| {
                    self.command(ui, "Navigation Pane", Action::Navigation, vec2(145., 30.));
                    self.command(ui, "Canvas Rulers", Action::Rulers, vec2(145., 30.));
                    ui.small("Show");
                });
                ui.separator();
                self.ribbon_group = Some(1);
                ui.vertical(|ui| {
                    self.command(ui, "Zoom…", Action::Zoom, vec2(88., 30.));
                    ui.label(format!("{:.0}%", self.zoom));
                    ui.small("Zoom");
                });
            });
            self.ribbon_group = None;
        } else if self.tab == 9 {
            ui.label(format!("BreakFree Docs {VERSION}"));
            ui.label("Local .bfdoc documents · no pagination yet");
            ui.label("Ctrl+N/O/S · Ctrl+B/I/U · Ctrl+Shift+P · Ctrl+Z/Y · Ctrl+F");
            if self.keytips.level == Some(keytips::Level::Tab(9)) {
                ui.small("Help is informational. Esc returns to tab KeyTips.");
            }
        } else {
            let groups = placeholder_groups(self.tab);
            let width = ((ui.available_width() - 24.) / groups.len() as f32).min(130.);
            ui.horizontal(|ui| {
                for group in groups {
                    ui.vertical(|ui| {
                        Self::disabled(
                            ui,
                            group,
                            vec2(width, 53.),
                            "This document feature is not implemented. See the roadmap.",
                        );
                        ui.small(*group);
                    });
                }
            });
            ui.small(
                if self.keytips.level == Some(keytips::Level::Tab(self.tab)) {
                    "Commands in this tab are not implemented yet. Esc returns to tab KeyTips."
                } else {
                    "Commands in this tab are not implemented yet."
                },
            );
        }
    }

    fn ribbon(&mut self, ctx: &egui::Context) {
        if self.ribbon_collapsed || self.backstage {
            return;
        }
        let panel = egui::TopBottomPanel::top("ribbon")
            .exact_height(RIBBON_HEIGHT)
            .frame(
                egui::Frame::new()
                    .fill(CHROME)
                    .inner_margin(egui::Margin::symmetric(8, 4)),
            )
            .show(ctx, |ui| self.ribbon_contents(ui));
        self.record("ribbon", panel.response.rect);
    }

    fn temporary_ribbon(&mut self, ctx: &egui::Context) {
        if !self.ribbon_collapsed || !self.ribbon_temporary || self.backstage {
            return;
        }
        let width = ctx.content_rect().width();
        let area = egui::Area::new(Id::new("temporary-ribbon"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos2(ctx.content_rect().left(), TITLE_HEIGHT + TABS_HEIGHT))
            .fade_in(false)
            .movable(false)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(CHROME)
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 3],
                        blur: 6,
                        spread: 0,
                        color: Color32::from_black_alpha(35),
                    })
                    .show(ui, |ui| {
                        ui.set_min_size(vec2(width - 16., RIBBON_HEIGHT - 8.));
                        ui.set_max_size(vec2(width - 16., RIBBON_HEIGHT - 8.));
                        self.ribbon_contents(ui);
                    });
            });
        self.record("ribbon", area.response.rect);
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        // Modal drafts must never dispatch edits, Save or New into the background.
        if self.font_dialog.is_some() || self.zoom_dialog.is_some() {
            return;
        }
        self.ribbon_keys(ctx);
        let document_focus =
            self.ribbon_focus.is_none() && ctx.memory(|m| m.has_focus(Id::new(EDIT_ID)));
        if !self.composing {
            for (key, shift, action) in [
                (Key::N, false, Action::New),
                (Key::O, false, Action::Open),
                (Key::S, true, Action::SaveAs),
                (Key::S, false, Action::Save),
                (Key::F, false, Action::Find),
                (Key::H, false, Action::Replace),
                (Key::P, true, Action::FontSize),
                (Key::D, false, Action::FontDialog),
                (Key::F, true, Action::FontDialog),
            ] {
                let modifiers = Modifiers {
                    shift,
                    command: true,
                    ..Modifiers::NONE
                };
                if ctx.input_mut(|i| i.consume_key(modifiers, key)) {
                    self.execute(action, ctx);
                }
            }
            if document_focus {
                for (key, shift, action) in [
                    (Key::Z, true, Action::Redo),
                    (Key::Z, false, Action::Undo),
                    (Key::Y, false, Action::Redo),
                    (Key::B, false, Action::Bold),
                    (Key::I, false, Action::Italic),
                    (Key::U, false, Action::Underline),
                    (Key::Period, true, Action::GrowFont),
                    (Key::Comma, true, Action::ShrinkFont),
                    (Key::L, false, Action::Left),
                    (Key::E, false, Action::Center),
                    (Key::R, false, Action::Right),
                    (Key::J, false, Action::Justify),
                ] {
                    let modifiers = Modifiers {
                        shift,
                        command: true,
                        ..Modifiers::NONE
                    };
                    if ctx.input_mut(|i| i.consume_key(modifiers, key)) {
                        self.execute(action, ctx);
                    }
                }
                for key in [Key::Backspace, Key::Delete] {
                    if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, key)) {
                        let (mut a, mut b) = self.session.editor.selection;
                        if a == b {
                            let edges = boundaries(&self.session.editor.document.text());
                            let index = edges.binary_search(&a).unwrap_or(0);
                            if key == Key::Backspace && index > 0 {
                                a = edges[index - 1];
                            }
                            if key == Key::Delete && index + 1 < edges.len() {
                                b = edges[index + 1];
                            }
                        }
                        let result = self.session.editor.replace(a, b, "", "Delete");
                        self.fail(result);
                        self.sync_view();
                    }
                }
                for (key, forward) in [(Key::ArrowLeft, false), (Key::ArrowRight, true)] {
                    for shift in [true, false] {
                        if ctx.input_mut(|i| {
                            i.consume_key(
                                Modifiers {
                                    shift,
                                    ..Modifiers::NONE
                                },
                                key,
                            )
                        }) {
                            let state = egui::TextEdit::load_state(ctx, Id::new(EDIT_ID))
                                .unwrap_or_default();
                            let range = state.cursor.char_range().unwrap_or_else(|| {
                                CCursorRange::one(CCursor::new(self.session.editor.selection.1))
                            });
                            let edges = boundaries(&self.session.editor.document.text());
                            let p = range.primary.index;
                            let next = if !shift && !range.is_empty() {
                                if forward {
                                    range.as_sorted_char_range().end
                                } else {
                                    range.as_sorted_char_range().start
                                }
                            } else if forward {
                                edges
                                    .iter()
                                    .copied()
                                    .find(|e| *e > p)
                                    .unwrap_or(*edges.last().unwrap())
                            } else {
                                edges
                                    .iter()
                                    .copied()
                                    .take_while(|e| *e < p)
                                    .last()
                                    .unwrap_or(0)
                            };
                            let anchor = if shift { range.secondary.index } else { next };
                            self.session.editor.selection = (next.min(anchor), next.max(anchor));
                            self.session.editor.typing_style =
                                self.session.editor.document.style_at(next);
                            self.cursor_override =
                                Some(CCursorRange::two(CCursor::new(next), CCursor::new(anchor)));
                        }
                    }
                }
            }
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            if ctx.memory(|m| {
                m.has_focus(Id::new(FONT_SIZE_ID)) || m.had_focus_last_frame(Id::new(FONT_SIZE_ID))
            }) {
                self.font_size_dirty = false;
                ctx.memory_mut(|m| m.surrender_focus(Id::new(FONT_SIZE_ID)));
                self.sync_view();
            }
            if self.composing {
                self.composing = false;
                self.sync_view();
            }
            self.palette = false;
            self.backstage = false;
            if self.pending.is_some() {
                self.pending = None;
            }
        }
    }

    fn save_status(&self) -> &'static str {
        if self.session.dirty() {
            "Unsaved changes"
        } else if self.session.path.is_some() {
            "Saved locally"
        } else {
            "Not saved yet"
        }
    }

    fn status(&mut self, ctx: &egui::Context) {
        if self.backstage {
            return;
        }
        let panel=egui::TopBottomPanel::bottom("status").exact_height(STATUS_HEIGHT)
            .frame(egui::Frame::new().fill(CHROME).inner_margin(egui::Margin::symmetric(10,0))).show(ctx,|ui| {
                ui.horizontal_centered(|ui| {
                    let doc=&self.session.editor.document;
                    ui.small(format!("{} words",doc.word_count()));
                    ui.add_space(10.);ui.small(format!("{} paragraphs",doc.paragraphs.len()));
                    if ui.available_width()>520. {ui.add_space(10.);ui.small(self.save_status());}
                    ui.with_layout(Layout::right_to_left(Align::Center),|ui| {
                        let percent=ui.add_sized([42.,20.],Button::new(format!("{:.0}%",self.zoom)).frame_when_inactive(false));
                        self.record("zoom-reset",percent.rect);self.status_focus=Some(percent.id);if percent.clicked(){percent.request_focus();}
                        if percent.on_hover_text("Reset document zoom to 100%").clicked(){self.zoom=100.;}
                        let plus=ui.add_sized([22.,20.],Button::new("+").frame_when_inactive(false));self.record("zoom-in",plus.rect);if plus.clicked(){plus.request_focus();}
                        if plus.on_hover_text("Zoom in").clicked(){self.zoom=(self.zoom+10.).min(200.);}
                        ui.spacing_mut().slider_width=110.;
                        let slider=ui.add(egui::Slider::new(&mut self.zoom,50.0..=200.0).show_value(false).clamping(egui::SliderClamping::Always));self.record("zoom-slider",slider.rect);
                        let minus=ui.add_sized([22.,20.],Button::new("−").frame_when_inactive(false));self.record("zoom-out",minus.rect);if minus.clicked(){minus.request_focus();}
                        if minus.on_hover_text("Zoom out").clicked(){self.zoom=(self.zoom-10.).max(50.);}
                        if ui.available_width()>195. {ui.separator();ui.small("Continuous layout").on_hover_text("Pagination and Print Layout are not implemented. This canvas has no computed page count.");}
                    });
                });
            });
        self.record("status", panel.response.rect);
    }

    fn find_next(&mut self) {
        if self.find.is_empty() {
            return;
        }
        let text = self.session.editor.document.text();
        let end = self.session.editor.selection.1;
        let matches: Vec<_> = text
            .match_indices(&self.find)
            .map(|(byte, word)| (text[..byte].chars().count(), word.chars().count()))
            .collect();
        if let Some(&(start, len)) = matches
            .iter()
            .find(|(a, _)| *a >= end)
            .or_else(|| matches.first())
        {
            self.session.editor.selection = snap(&text, start, start + len);
            self.sync_view();
        }
    }

    fn navigation_panel(&mut self, ctx: &egui::Context) {
        if !self.navigation || self.backstage {
            return;
        }
        let panel = egui::SidePanel::left("navigation")
            .default_width(240.)
            .min_width(195.)
            .max_width(320.)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Navigation");
                    if ui.button("×").on_hover_text("Close navigation").clicked() {
                        self.navigation = false;
                    }
                });
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.find)
                        .id(Id::new("find-text"))
                        .hint_text("Search document"),
                );
                if self.focus_find {
                    response.request_focus();
                    self.focus_find = false;
                }
                let results = if self.find.is_empty() {
                    0
                } else {
                    self.session
                        .editor
                        .document
                        .text()
                        .match_indices(&self.find)
                        .count()
                };
                ui.label(format!("{results} matches"));
                if ui
                    .add_enabled(results > 0, Button::new("Next result"))
                    .clicked()
                    || response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter))
                {
                    self.find_next();
                }
                if self.replace_mode {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.replacement).hint_text("Replace with"),
                    );
                    if ui
                        .add_enabled(
                            results > 0 && !self.composing,
                            Button::new("Replace selection / next"),
                        )
                        .clicked()
                    {
                        if selected_text(
                            &self.session.editor.document,
                            self.session.editor.selection,
                        ) == self.find
                        {
                            let (a, b) = self.session.editor.selection;
                            let result = self.session.editor.replace(
                                a,
                                b,
                                &self.replacement,
                                "Replace match",
                            );
                            self.fail(result);
                            self.sync_view();
                        } else {
                            self.find_next();
                        }
                    }
                }
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Results").strong());
                    Self::disabled(
                        ui,
                        "Pages",
                        vec2(58., 23.),
                        "Pages require pagination, which is not implemented.",
                    );
                });
                let text = self.session.editor.document.text();
                if !self.find.is_empty() {
                    for (byte, _) in text.match_indices(&self.find.clone()).take(40) {
                        let start = text[..byte].chars().count();
                        let end = start + self.find.chars().count();
                        let snippet: String = text
                            .chars()
                            .skip(start.saturating_sub(12))
                            .take(35)
                            .collect();
                        if ui.selectable_label(false, snippet).clicked() {
                            self.session.editor.selection = snap(&text, start, end);
                            self.sync_view();
                        }
                    }
                }
            });
        self.record("navigation", panel.response.rect);
    }

    fn formatting_panel(&mut self, ctx: &egui::Context) {
        if !self.formatting || self.backstage {
            return;
        }
        let panel = egui::SidePanel::right("formatting")
            .default_width(220.)
            .min_width(195.)
            .max_width(300.)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Formatting");
                    if ui.button("×").clicked() {
                        self.formatting = false;
                    }
                });
                let (a, b) = self.session.editor.selection;
                ui.label(format!("{} selected characters", b - a));
                ui.separator();
                self.command(ui, "Bold", Action::Bold, vec2(150., 28.));
                self.command(ui, "Italic", Action::Italic, vec2(150., 28.));
                self.command(ui, "Underline", Action::Underline, vec2(150., 28.));
                let size = self
                    .session
                    .editor
                    .selected_font_size()
                    .map(|s| format!("{s} pt"))
                    .unwrap_or("Mixed sizes".into());
                self.command(
                    ui,
                    &format!("Font size: {size}"),
                    Action::FontSize,
                    vec2(180., 28.),
                );
                ui.separator();
                ui.label("Paragraph alignment");
                for a in [Action::Left, Action::Center, Action::Right, Action::Justify] {
                    self.command(ui, a.label(), a, vec2(150., 27.));
                }
            });
        self.record("formatting", panel.response.rect);
    }

    fn ruler(ui: &mut Ui, width: f32, zoom: f32) {
        let (rect, _) = ui.allocate_exact_size(vec2(width, 22.), Sense::hover());
        let margin = CANVAS_MARGIN * zoom;
        ui.painter().rect_filled(rect, 0., Color32::from_gray(214));
        ui.painter().rect_filled(
            Rect::from_min_max(rect.min + vec2(margin, 0.), rect.max - vec2(margin, 0.)),
            0.,
            Color32::from_gray(250),
        );
        let left = rect.left() + margin;
        let right = rect.right() - margin;
        for tick in 0..=((right - left) / (12. * zoom)) as usize {
            let x = left + tick as f32 * 12. * zoom;
            let large = tick % 8 == 0;
            ui.painter().line_segment(
                [
                    pos2(x, rect.bottom() - if large { 11. } else { 5. }),
                    pos2(x, rect.bottom() - 1.),
                ],
                Stroke::new(1.0_f32, Color32::from_gray(115)),
            );
            if large {
                ui.painter().text(
                    pos2(x, rect.top() + 6.),
                    Align2::CENTER_CENTER,
                    (tick / 8).to_string(),
                    FontId::proportional(10.),
                    Color32::from_gray(90),
                );
            }
        }
    }

    fn document(&mut self, ui: &mut Ui) {
        let zoom = self.zoom / 100.;
        let width = CANVAS_WIDTH * zoom;
        let margin = CANVAS_MARGIN * zoom;
        let text_width = width - 2. * margin;
        let font_size = 14.6667 * zoom;
        let model = self.session.editor.document.clone();
        let typing = self.session.editor.typing_style;
        let natural = layout::galley(
            ui.ctx(),
            &model,
            &self.buffer,
            typing,
            text_width,
            font_size,
        )
        .size()
        .y;
        let min_height = (ui.available_height() - 45.)
            .max(natural + 2. * margin)
            .max(300.);
        egui::ScrollArea::both().auto_shrink([false,false]).show(ui,|ui|{
            let space=((ui.available_width()-width-32.)/2.).max(12.);
            ui.add_space(10.);
            ui.horizontal_top(|ui|{
                ui.add_space(space);
                ui.vertical(|ui|{
                    if self.rulers {Self::ruler(ui,width,zoom);ui.add_space(7.);}
                    let frame=egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0_f32,Color32::from_gray(197))).shadow(egui::epaint::Shadow{offset:[0,2],blur:5,spread:0,color:Color32::from_black_alpha(24)}).inner_margin(0).show(ui,|ui|{
                        ui.set_min_size(vec2(width,min_height));
                        ui.add_space(margin);
                        ui.horizontal_top(|ui|{
                            ui.add_space(margin);
                            ui.allocate_ui_with_layout(vec2(text_width,natural.max(40.)),Layout::top_down(Align::Min),|ui|{
                                let id=Id::new(EDIT_ID);
                                let mut state=egui::TextEdit::load_state(ui.ctx(),id).unwrap_or_default();state.clear_undoer();
                                if let Some(range)=self.cursor_override.take(){state.cursor.set_char_range(Some(range));}state.store(ui.ctx(),id);
                                if ui.memory(|m|m.has_focus(id)) {
                                    ui.input_mut(|i|for event in &mut i.events {if let egui::Event::Paste(text)=event {*text=text.replace("\r\n","\n").replace('\r',"\n");}});
                                }
                                let events=ui.input(|i|i.events.clone());
                                let focused=ui.memory(|m|m.has_focus(id));
                                let mut commit=false;let mut cancel=false;
                                if focused {for event in &events {if let egui::Event::Ime(ime)=event {match ime {egui::ImeEvent::Enabled|egui::ImeEvent::Preedit(_)=>self.composing=true,egui::ImeEvent::Commit(text)=>{self.composing=false;if text.is_empty()||text=="\n"||text=="\r"{cancel=true;}else{commit=true;}},egui::ImeEvent::Disabled=>{if self.composing{cancel=true;}self.composing=false;}}}}}
                                let mut layouter=|ui:&Ui,text:&dyn egui::TextBuffer,wrap:f32|layout::galley(ui.ctx(),&model,text.as_str(),typing,wrap,font_size);
                                let mut output=egui::TextEdit::multiline(&mut self.buffer).id(id).frame(false).margin(0).desired_width(text_width).desired_rows(1).font(FontId::new(font_size,egui::FontFamily::Name("body".into()))).layouter(&mut layouter).show(ui);
                                if self.focus_document {
                                    if !output.response.has_focus() {output.response.request_focus();}
                                    ui.memory_mut(|m|m.set_focus_lock_filter(id,egui::EventFilter{horizontal_arrows:true,vertical_arrows:true,..Default::default()}));
                                    self.focus_document=false;
                                }
                                if cancel&&!commit||output.response.lost_focus()&&self.composing {self.composing=false;self.sync_view();}
                                else if output.response.changed()&&!self.composing&&self.buffer!=self.session.editor.document.text(){
                                    let(a,b,text)=text_change(&self.session.editor.document.text(),&self.buffer);
                                    let result=self.session.editor.replace(a,b,&text,if commit{"Input method commit"}else{"Text input"});self.fail(result);self.sync_view();
                                }else if !self.composing&&output.response.has_focus() && let Some(range)=output.cursor_range {
                                        let range=range.as_sorted_char_range();let selection=snap(&self.buffer,range.start,range.end);
                                        if selection!=self.session.editor.selection {self.session.editor.selection=selection;self.session.editor.typing_style=self.session.editor.document.style_at(selection.0);}
                                }
                                output.state.clear_undoer();output.state.store(ui.ctx(),id);
                                output.response.context_menu(|ui|{for action in [Action::Cut,Action::Copy,Action::Paste,Action::SelectAll]{self.command(ui,action.label(),action,vec2(150.,26.));}});
                                self.record("document-text",output.response.rect);
                            });
                        });
                        ui.add_space(margin);
                    });self.record("continuous-canvas",frame.response.rect);
                });
            });
        });
    }

    fn file_view(&mut self, ui: &mut Ui) {
        let full = ui.max_rect();

        let rail = Rect::from_min_max(full.min, pos2(full.left() + 180., full.bottom()));

        ui.painter()
            .rect_filled(rail, 0., Color32::from_rgb(242, 239, 247));

        ui.painter().line_segment(
            [rail.right_top(), rail.right_bottom()],
            Stroke::new(1.0_f32, chrome::LINE),
        );

        self.record("file-rail", rail);

        ui.scope_builder(
            egui::UiBuilder::new().max_rect(rail.shrink2(vec2(12., 14.))),
            |ui| {
                let (back_rect, _) = ui.allocate_exact_size(vec2(156., 32.), Sense::hover());
                let back = chrome::control(
                    ui,
                    back_rect,
                    Icon::Back,
                    "Back",
                    "Back to document",
                    true,
                    false,
                );

                self.record("file-back", back.rect);

                if back.clicked() {
                    self.backstage = false;

                    self.focus_document = true;
                }
                ui.add_space(20.);

                let home = ui.add_sized(
                    [156., 34.],
                    Button::new("Home")
                        .selected(!self.settings)
                        .frame_when_inactive(!self.settings),
                );

                self.record("file-home", home.rect);

                if home.clicked() {
                    self.settings = false;
                }
                ui.add_space(8.);

                for (icon, label, action) in [
                    (Icon::Document, "New", Action::New),
                    (Icon::Open, "Open", Action::Open),
                    (Icon::Save, "Save", Action::Save),
                    (Icon::Save, "Save As", Action::SaveAs),
                ] {
                    let (rect, _) = ui.allocate_exact_size(vec2(156., 34.), Sense::hover());

                    self.ribbon_control(ui, rect, icon, label, action);
                }
                ui.add_space(10.);

                ui.separator();

                Self::disabled(
                    ui,
                    "Print",
                    vec2(156., 34.),
                    "Printing requires pagination, planned for D3.",
                );

                ui.add_space(8.);

                let options = ui.add_sized(
                    [156., 34.],
                    Button::new("Options")
                        .selected(self.settings)
                        .frame_when_inactive(self.settings),
                );

                self.record("file-options", options.rect);

                if options.clicked() {
                    self.settings = true;
                }
            },
        );

        let body = Rect::from_min_max(full.min + vec2(210., 24.), full.max - vec2(28., 20.));

        ui.scope_builder(egui::UiBuilder::new().max_rect(body), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(RichText::new(if self.settings { "Options" } else { "Home" }).size(28.));

                ui.add_space(24.);

                if self.settings {
                    ui.label(RichText::new("Display").size(18.));

                    ui.add_space(14.);

                    ui.checkbox(&mut self.rulers, "Show canvas ruler");

                    ui.checkbox(&mut self.navigation, "Show navigation pane");

                    ui.checkbox(&mut self.ribbon_collapsed, "Collapse ribbon");

                    ui.add_space(18.);

                    ui.label("Document zoom");

                    ui.add(egui::Slider::new(&mut self.zoom, 50.0..=200.0).suffix("%"));

                    ui.add_space(20.);

                    ui.small("These display options apply to this window.");
                } else {
                    ui.label(RichText::new("New").size(18.));

                    ui.add_space(12.);

                    let (tile, _) = ui.allocate_exact_size(vec2(158., 160.), Sense::hover());

                    let response = ui.put(tile, Button::new("").frame_when_inactive(false));

                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Blank document")
                    });

                    self.record("file-blank", response.rect);

                    let sheet =
                        Rect::from_center_size(tile.center() - vec2(0., 14.), vec2(74., 96.));

                    ui.painter().rect_filled(sheet, 0., Color32::WHITE);

                    ui.painter().rect_stroke(
                        sheet,
                        0.,
                        Stroke::new(1.0_f32, Color32::from_gray(175)),
                        egui::StrokeKind::Inside,
                    );

                    ui.painter().text(
                        pos2(tile.center().x, tile.bottom() - 13.),
                        Align2::CENTER_CENTER,
                        "Blank document",
                        FontId::proportional(13.),
                        INK,
                    );

                    if response.clicked() {
                        self.execute(Action::New, ui.ctx());
                    }
                    ui.add_space(25.);

                    ui.separator();

                    ui.add_space(18.);

                    ui.label(RichText::new("Current document").size(18.));

                    ui.add_space(12.);

                    let name = self
                        .session
                        .path
                        .as_ref()
                        .and_then(|p| p.file_name())
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or("Untitled".into());

                    ui.add(egui::Label::new(RichText::new(name).strong()).truncate());

                    if let Some(path) = &self.session.path {
                        let text = path.display().to_string();

                        ui.add(egui::Label::new(&text).truncate())
                            .on_hover_text(&text);
                    }
                    let doc = &self.session.editor.document;

                    ui.label(format!(
                        "{} words · {} paragraph{} · {} characters",
                        doc.word_count(),
                        doc.paragraphs.len(),
                        if doc.paragraphs.len() == 1 { "" } else { "s" },
                        doc.character_count()
                    ));

                    ui.add_space(8.);

                    ui.label(if self.session.dirty() {
                        "Unsaved changes"
                    } else {
                        "No unsaved changes"
                    });

                    ui.add_space(18.);

                    ui.small("Local document format: BreakFree Docs (.bfdoc)");
                    ui.small("Saves use format v2. Earlier v1-only builds cannot open new saves.");

                    ui.small("Continuous layout. Pagination and printing are planned.");
                }
            });
        });
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        if let Some(mut dialog) = self.zoom_dialog.take() {
            match dialog.show(ctx, &mut self.regions) {
                ZoomOutcome::Open => self.zoom_dialog = Some(dialog),
                ZoomOutcome::Cancel => self.focus_document = true,
                ZoomOutcome::Apply(value) => {
                    self.zoom = value;
                    self.focus_document = true;
                }
            }
        }
        if let Some(mut dialog) = self.font_dialog.take() {
            match dialog.show(ctx, &self.body_font, &mut self.regions) {
                FontDialogOutcome::Open => self.font_dialog = Some(dialog),
                FontDialogOutcome::Cancel => self.sync_view(),
                FontDialogOutcome::Apply(patch) => {
                    let (start, end) = self.session.editor.selection;
                    let result = self.session.editor.set_font(start, end, patch);
                    self.fail(result);
                    self.sync_view();
                }
            }
        }
        if self.pending.is_some() {
            egui::Modal::new(Id::new("unsaved-changes")).show(ctx, |ui| {
                ui.set_width(380.);
                ui.heading("Save changes?");
                ui.label("Save changes before leaving this document?");
                ui.add_space(12.);
                ui.horizontal(|ui| {
                    let save = ui.button("Save");
                    self.record("unsaved-save", save.rect);
                    if save.clicked()
                        && self.choose_save(false)
                        && let Some(pending) = self.pending.take()
                    {
                        self.finish(pending, ctx);
                    }
                    let discard = ui.button("Discard");
                    self.record("unsaved-discard", discard.rect);
                    if discard.clicked()
                        && let Some(pending) = self.pending.take()
                    {
                        self.finish(pending, ctx);
                    }
                    let cancel = ui.button("Cancel");
                    self.record("unsaved-cancel", cancel.rect);
                    if cancel.clicked() {
                        self.pending = None;
                        self.focus_document = true;
                    }
                });
            });
        }
        if let Some(error) = self.error.clone() {
            egui::Modal::new(Id::new("document-error")).show(ctx, |ui| {
                ui.set_width(430.);
                ui.heading("BreakFree Docs");
                ui.label(error);
                ui.add_space(10.);
                if ui.button("OK").clicked() {
                    self.error = None;
                }
            });
        }
        if self.palette {
            egui::Window::new("Commands")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_TOP, [0., 42.])
                .show(ctx, |ui| {
                    let query = self.command_search.to_lowercase();
                    let actions: Vec<_> = COMMANDS
                        .iter()
                        .copied()
                        .filter(|a| a.label().to_lowercase().contains(&query))
                        .collect();
                    for action in actions.iter().take(12) {
                        if ui
                            .add_enabled(self.enabled(*action), Button::new(action.label()))
                            .clicked()
                        {
                            self.execute(*action, ctx);
                            self.palette = false;
                        }
                    }
                    if actions.is_empty() {
                        ui.label("No matching command");
                    }
                    if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter))
                        && let Some(action) = actions.into_iter().find(|a| self.enabled(*a))
                    {
                        self.execute(action, ctx);
                        self.palette = false;
                    }
                });
        }
    }

    pub fn update_ui(&mut self, ctx: &egui::Context) {
        let mut close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested && (self.font_dialog.is_some() || self.zoom_dialog.is_some()) {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            close_requested = false;
        }
        let size_ready = !close_requested || self.allow_close || self.commit_font_size_input();
        if !size_ready {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        }
        if close_requested && size_ready && self.session.dirty() && !self.allow_close {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.request(Pending::Close, ctx);
        }
        self.keytip_input(ctx);
        self.prepare_keytip_menu(ctx);
        self.shortcuts(ctx);
        self.regions.clear();
        self.ribbon_targets.clear();
        self.ribbon_menu_targets.clear();
        self.title(ctx);
        if !self.backstage {
            self.tabs(ctx);
        }
        self.ribbon(ctx);
        self.status(ctx);
        self.navigation_panel(ctx);
        self.formatting_panel(ctx);
        let panel = egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(if self.backstage {
                        Color32::WHITE
                    } else {
                        WORKSPACE
                    })
                    .inner_margin(0),
            )
            .show(ctx, |ui| {
                if self.backstage {
                    self.file_view(ui);
                } else {
                    self.document(ui);
                }
            });
        self.record("workspace", panel.response.rect);
        self.temporary_ribbon(ctx);
        self.finish_ribbon_focus(ctx);
        self.dialogs(ctx);
        self.paint_keytips(ctx);
        ctx.send_viewport_cmd(ViewportCommand::Title(format!(
            "{} — BreakFree Docs",
            self.session
                .path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or("Untitled".into())
        )));
    }
}

fn placeholder_groups(tab: usize) -> &'static [&'static str] {
    match tab {
        1 => &["Pages", "Tables", "Illustrations", "Links"],
        2 => &["Drawing Tools"],
        3 => &["Document Formatting"],
        4 => &["Page Setup", "Paragraph"],
        5 => &["Contents", "References"],
        6 => &["Mail Merge"],
        7 => &["Proofing", "Comments", "Tracking"],
        _ => &[],
    }
}

impl eframe::App for DocsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_ui(ctx);
    }
}

pub fn run(session: Session) -> Result<()> {
    let icon = image::load_from_memory(include_bytes!(
        "../docs/brand/breakfree-docs-transparent.webp"
    ))
    .map_err(|e| e.to_string())?
    .into_rgba8();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1366., 900.])
            .with_min_inner_size([640., 480.])
            .with_decorations(false)
            .with_icon(egui::IconData {
                width: icon.width(),
                height: icon.height(),
                rgba: icon.into_raw(),
            }),
        ..Default::default()
    };
    eframe::run_native(
        "BreakFree Docs",
        options,
        Box::new(move |cc| Ok(Box::new(DocsApp::new(session, &cc.egui_ctx)))),
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "gui_tests.rs"]
mod tests;
