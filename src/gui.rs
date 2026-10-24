//! Custom native office shell. Document changes always go through commands::Editor.
use crate::{
    Result, VERSION,
    chrome::{self, Icon},
    layout,
    model::{Alignment, CharacterStyle, FONT_SIZE_PRESETS, FontSize, ParagraphStyle, boundaries},
    presentation::*,
    session::Session,
};
use eframe::egui::{
    self, Align, Align2, Button, Color32, FontId, Id, Key, Layout, Modifiers, Rect, RichText,
    Sense, Stroke, Ui, Vec2, ViewportCommand, pos2,
    text::{CCursor, CCursorRange},
    vec2,
};
use std::path::PathBuf;

#[path = "paragraph_dialog.rs"]
mod paragraph_dialog;
use paragraph_dialog::{ParagraphDialog, ParagraphOutcome};
#[path = "font_dialog.rs"]
mod font_dialog;
#[path = "style_dialog.rs"]
mod style_dialog;
use style_dialog::{StyleDialog, StyleOutcome};
#[path = "delete_style_dialog.rs"]
mod delete_style_dialog;
use delete_style_dialog::{DeleteStyleDialog, DeleteStyleOutcome};
#[path = "replace_all_dialog.rs"]
mod replace_all_dialog;
use font_dialog::{FontDialog, FontDialogOutcome};
use replace_all_dialog::{ReplaceAllDialog, ReplaceAllOutcome};

#[path = "ribbon_navigation.rs"]
mod ribbon_navigation;
use ribbon_navigation::{RibbonFocus, RibbonTarget, TargetKind};

#[path = "keytips.rs"]
mod keytips;
use keytips::KeyTips;

#[path = "tab_dialog.rs"]
mod tab_dialog;
#[path = "tab_positions_dialog.rs"]
mod tab_positions_dialog;
use tab_positions_dialog::{PositionsDialog, PositionsOutcome};
#[path = "tab_sources_dialog.rs"]
mod tab_sources_dialog;
use tab_sources_dialog::TabSourcesDialog;
#[path = "structure_ui.rs"]
mod structure_ui;
#[path = "zoom_dialog.rs"]
mod zoom_dialog;
use structure_ui::StructureDialog;
use tab_dialog::{TabDialog, TabOutcome};
use zoom_dialog::{ZoomDialog, ZoomOutcome};

#[path = "quick_order.rs"]
mod quick_order;
use quick_order::{OrderOutcome, QuickOrderDialog};

#[path = "navigation.rs"]
mod navigation;
#[path = "quick_access.rs"]
mod quick_access;
#[path = "ruler_ui.rs"]
mod ruler_ui;
#[path = "tab_ruler_ui.rs"]
mod tab_ruler_ui;
#[path = "view_ribbon.rs"]
mod view_ribbon;

const INK: Color32 = chrome::INK;
const ACCENT: Color32 = chrome::ACCENT;
const CHROME: Color32 = chrome::CHROME;
const WORKSPACE: Color32 = chrome::WORKSPACE;
const EDIT_ID: &str = "document-text";
const SEARCH_ID: &str = "command-search";
const FONT_SIZE_ID: &str = "font-size-input";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Bullets,
    Numbering,
    ListSettings,
    Link,
    Table,
    Picture,
    PageSetup,
    PageFlow,
    PageBreak,
    PrintLayout,
    PrintPreview,
    ExportPdf,
    ExportDocx,
    Print,

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
    ClearFormatting,
    Style(ParagraphStyle),
    CharacterStyle(Option<CharacterStyle>),
    FontSize,
    FontDialog,
    ParagraphSpacing,
    TabInterval,
    TabPositions,
    IncreaseIndent,
    DecreaseIndent,
    ModifyStyle,
    CreateStyle,
    CreateCharacterStyle,
    ModifyCharacterStyle,
    DeleteStyle,
    DeleteCharacterStyle,
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
    Zoom100,
    ZoomWidth,
    Formatting,
}
impl Action {
    fn label(self) -> &'static str {
        match self {
            Self::Bullets => "Bullets",
            Self::Numbering => "Numbering",
            Self::ListSettings => "List settings",
            Self::Link => "Hyperlink",
            Self::Table => "Table",
            Self::Picture => "Picture",
            Self::PageSetup => "Page setup",
            Self::PageFlow => "Paragraph pagination",
            Self::PageBreak => "Page break before",
            Self::PrintLayout => "Print Layout",
            Self::PrintPreview => "Print preview",
            Self::ExportPdf => "Export PDF",
            Self::ExportDocx => "Export DOCX (limited subset)",
            Self::Print => "Print",
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
            Self::ClearFormatting => "Clear character formatting",
            Self::Style(style) => style.name(),
            Self::CharacterStyle(style) => {
                style.map_or("Default paragraph font", CharacterStyle::name)
            }
            Self::FontSize => "Font size",
            Self::FontDialog => "Font dialog",
            Self::IncreaseIndent => "Increase indent",
            Self::DecreaseIndent => "Decrease indent",
            Self::ParagraphSpacing => "Paragraph spacing",
            Self::TabInterval => "Tab interval",
            Self::TabPositions => "Tab positions",
            Self::ModifyStyle => "Modify style…",
            Self::CreateStyle => "Create style…",
            Self::ModifyCharacterStyle => "Modify character style…",
            Self::CreateCharacterStyle => "Create character style…",
            Self::DeleteCharacterStyle => "Delete character style…",
            Self::DeleteStyle => "Delete style…",
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
            Self::Zoom100 => "100%",
            Self::ZoomWidth => "Canvas Width",
            Self::Formatting => "Formatting pane",
        }
    }
}
const COMMANDS: &[Action] = &[
    Action::Bullets,
    Action::Numbering,
    Action::ListSettings,
    Action::Link,
    Action::Table,
    Action::Picture,
    Action::PageSetup,
    Action::PageFlow,
    Action::PageBreak,
    Action::PrintLayout,
    Action::PrintPreview,
    Action::ExportPdf,
    Action::ExportDocx,
    Action::Print,
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
    Action::ClearFormatting,
    Action::Style(ParagraphStyle::Normal),
    Action::Style(ParagraphStyle::NoSpacing),
    Action::Style(ParagraphStyle::Heading1),
    Action::CharacterStyle(Some(CharacterStyle::Strong)),
    Action::CharacterStyle(Some(CharacterStyle::Emphasis)),
    Action::CharacterStyle(None),
    Action::FontSize,
    Action::FontDialog,
    Action::ParagraphSpacing,
    Action::TabInterval,
    Action::TabPositions,
    Action::IncreaseIndent,
    Action::DecreaseIndent,
    Action::ModifyStyle,
    Action::CreateStyle,
    Action::CreateCharacterStyle,
    Action::ModifyCharacterStyle,
    Action::DeleteStyle,
    Action::DeleteCharacterStyle,
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
    Action::Zoom100,
    Action::ZoomWidth,
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
    tab_dialog: Option<TabDialog>,
    tab_positions_dialog: Option<PositionsDialog>,
    tab_sources_dialog: Option<TabSourcesDialog>,
    structure_dialog: Option<StructureDialog>,
    page_preview: bool,
    paged_editing: bool,
    page_count: Option<usize>,
    print_requested: bool,
    printer_queue: String,
    output_notice: Option<String>,
    quick_access: Vec<Action>,
    quick_access_below: bool,
    quick_access_targets: Vec<RibbonTarget>,
    zoom_width: bool,
    preferences: Option<crate::preferences::Store>,
    preferences_warning: Option<String>,
    preferences_backup: Option<PathBuf>,
    preferences_reset_pending: bool,
    quick_reset: bool,
    quick_order: Option<QuickOrderDialog>,
    ribbon_targets: Vec<RibbonTarget>,
    ribbon_menu_targets: Vec<RibbonTarget>,
    ribbon_menu_index: Option<usize>,
    ribbon_group: Option<usize>,
    collecting_menu: bool,
    status_focus: Option<Id>,
    formatting_focus: Option<Id>,
    pub navigation: bool,
    pub rulers: bool,
    ruler_drag: Option<ruler_ui::Drag>,
    tab_ruler_drag: Option<tab_ruler_ui::TabDrag>,
    tab_creation_kind: crate::model::TabKind,
    tab_preview_help: bool,
    ruler_key_focus: Option<crate::ruler::Handle>,
    ruler_menu: Option<usize>,
    ruler_menu_activate: bool,
    ruler_sources: bool,
    ruler_sources_scroll: f32,
    ruler_sources_height: f32,
    ruler_sources_wheel_guard: bool,
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
    paragraph_dialog: Option<ParagraphDialog>,
    style_dialog: Option<StyleDialog>,
    delete_style_dialog: Option<DeleteStyleDialog>,
    style_used_only: bool,
    style_jump: Option<Action>,
    style_jump_ready: bool,
    style_navigation_notice: Option<String>,
    style_return: Option<(Action, String, crate::style_filter::Scope, bool)>,
    style_filter: String,
    style_scope: crate::style_filter::Scope,
    style_context: Option<(ParagraphStyle, egui::Pos2, egui::LayerId)>,
    pending: Option<Pending>,
    error: Option<String>,
    backstage: bool,
    settings: bool,
    formatting: bool,
    command_search: String,
    palette: bool,
    find: String,
    find_options: crate::search::Options,
    find_option_ids: [Option<Id>; 2],
    find_query: crate::search::Query,
    replacement: String,
    find_scroll: Option<usize>,
    find_composing: bool,
    scroll_to_selection: bool,
    replace_mode: bool,
    replace_composing: bool,
    replace_all_dialog: Option<ReplaceAllDialog>,
    replace_all_notice: Option<String>,
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
            s.visuals.widgets.inactive.bg_fill = Color32::WHITE;
            s.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
            s.visuals.window_fill = Color32::WHITE;
            s.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_gray(218));
            s.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_gray(185));
            s.visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
            s.visuals.widgets.inactive.corner_radius = 3.into();
            s.visuals.widgets.hovered.corner_radius = 3.into();
            s.visuals.widgets.active.corner_radius = 3.into();
            s.visuals.override_text_color = Some(INK);
            s.visuals.selection.bg_fill = chrome::SELECTION;
            s.visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);
            s.visuals.widgets.hovered.bg_fill = chrome::HOVER;
            s.visuals.widgets.hovered.weak_bg_fill = chrome::HOVER;
            s.visuals.widgets.active.bg_fill = chrome::PRESSED;
            s.visuals.widgets.active.weak_bg_fill = chrome::PRESSED;
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
        let output_notice = session.imported_docx.then(|| {
            "DOCX imported. Save creates a native .bfdoc; the source DOCX is unchanged.".into()
        });
        let buffer = session.editor.document.text();
        let initial_cursor = CCursorRange::two(
            CCursor::new(session.editor.selection.0),
            CCursor::new(session.editor.selection.1),
        );
        Self {
            session,
            tab: 0,
            ribbon_collapsed: false,
            ribbon_temporary: false,
            ribbon_focus: None,
            keytips: KeyTips::default(),
            more_tabs_id: None,
            zoom_dialog: None,
            tab_dialog: None,
            tab_positions_dialog: None,
            tab_sources_dialog: None,
            structure_dialog: None,
            page_preview: false,
            paged_editing: false,
            page_count: None,
            print_requested: false,
            printer_queue: String::new(),
            output_notice,
            quick_access: quick_access::DEFAULT.to_vec(),
            quick_access_below: false,
            quick_access_targets: Vec::new(),
            zoom_width: false,
            preferences: None,
            preferences_warning: None,
            preferences_backup: None,
            preferences_reset_pending: false,
            quick_reset: false,
            quick_order: None,
            ribbon_targets: Vec::new(),
            ribbon_menu_targets: Vec::new(),
            ribbon_menu_index: None,
            ribbon_group: None,
            collecting_menu: false,
            status_focus: None,
            formatting_focus: None,
            navigation: false,
            rulers: true,
            ruler_drag: None,
            tab_ruler_drag: None,
            tab_creation_kind: crate::model::TabKind::Left,
            tab_preview_help: false,
            ruler_key_focus: None,
            ruler_menu: None,
            ruler_menu_activate: false,
            ruler_sources: false,
            ruler_sources_scroll: 0.,
            ruler_sources_height: 0.,
            ruler_sources_wheel_guard: false,
            zoom: 100.,
            regions: Vec::new(),
            buffer,
            composing: false,
            cursor_override: Some(initial_cursor),
            focus_document: true,
            focus_search: false,
            focus_find: false,
            focus_font_size: false,
            font_size_text: String::new(),
            font_size_dirty: false,
            font_dialog: None,
            paragraph_dialog: None,
            style_dialog: None,
            delete_style_dialog: None,
            style_used_only: false,
            style_jump: None,
            style_jump_ready: false,
            style_navigation_notice: None,
            style_return: None,
            style_filter: String::new(),
            style_scope: Default::default(),
            style_context: None,
            pending: None,
            error: None,
            backstage: false,
            settings: false,
            formatting: false,
            command_search: String::new(),
            palette: false,
            find: String::new(),
            find_options: crate::search::Options::default(),
            find_option_ids: [None, None],
            find_query: crate::search::Query::new("", crate::search::Options::default()),
            replacement: String::new(),
            find_scroll: None,
            find_composing: false,
            scroll_to_selection: false,
            replace_mode: false,
            replace_composing: false,
            replace_all_dialog: None,
            replace_all_notice: None,
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
        if self.ruler_drag.is_some()
            || self.tab_ruler_drag.is_some()
            || self.replace_all_dialog.is_some()
            || self.font_dialog.is_some()
            || self.paragraph_dialog.is_some()
            || self.style_dialog.is_some()
            || self.delete_style_dialog.is_some()
            || self.zoom_dialog.is_some()
            || self.tab_dialog.is_some()
            || self.tab_positions_dialog.is_some()
            || self.tab_sources_dialog.is_some()
            || self.structure_dialog.is_some()
            || self.quick_reset
            || self.quick_order.is_some()
        {
            return false;
        }
        if matches!(
            action,
            Action::ParagraphSpacing
                | Action::FontDialog
                | Action::ModifyStyle
                | Action::CreateStyle
                | Action::ModifyCharacterStyle
                | Action::CreateCharacterStyle
                | Action::DeleteCharacterStyle
                | Action::DeleteStyle
                | Action::Zoom
                | Action::TabInterval
                | Action::TabPositions
        ) && (self.pending.is_some() || self.error.is_some())
        {
            return false;
        }
        if self.composing {
            return matches!(action, Action::Copy | Action::Ribbon | Action::Rulers);
        }
        match action {
            Action::ModifyCharacterStyle | Action::DeleteCharacterStyle => matches!(
                self.session.editor.selected_character_style(),
                Some(Some(CharacterStyle::Custom(_)))
            ),
            Action::DeleteStyle => matches!(self.target_style(), ParagraphStyle::Custom(_)),
            Action::CreateCharacterStyle => {
                self.session.editor.document.character_styles.len()
                    < crate::model::MAX_CUSTOM_STYLES
            }
            Action::CreateStyle => {
                self.session.editor.document.style_catalog.custom.len()
                    < crate::model::MAX_CUSTOM_STYLES
            }
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
            Action::Style(style) => self.session.editor.selected_paragraph_style() == Some(style),
            Action::CharacterStyle(style) => {
                self.session.editor.selected_character_style() == Some(style)
            }
            Action::Bullets | Action::Numbering => {
                let kind = if action == Action::Bullets {
                    crate::structure::ListKind::Bullet
                } else {
                    crate::structure::ListKind::Decimal
                };
                crate::structure::paragraph_indices(
                    &self.session.editor.document,
                    self.session.editor.selection,
                )
                .is_ok_and(|indices| {
                    indices.into_iter().all(|i| {
                        self.session.editor.document.paragraphs[i]
                            .structure
                            .list
                            .as_ref()
                            .is_some_and(|l| l.kind == kind)
                    })
                })
            }
            Action::PrintLayout => self.paged_editing,
            Action::PrintPreview => self.page_preview,
            Action::Navigation => self.navigation,
            Action::Rulers => self.rulers,
            Action::ZoomWidth => self.zoom_width,
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
        self.replace_all_notice = None;
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
            Pending::Open(path) => match Session::open(&path) {
                Ok(session) => self.session = session,
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
        self.output_notice = self.session.imported_docx.then(|| {
            "DOCX imported. Save creates a native .bfdoc; the source DOCX is unchanged.".into()
        });
        self.backstage = false;
        self.font_size_dirty = false;
        self.sync_view();
    }
    pub fn execute(&mut self, action: Action, ctx: &egui::Context) {
        if !self.enabled(action) {
            return;
        }
        if matches!(
            action,
            Action::ClearFormatting
                | Action::Style(_)
                | Action::CharacterStyle(_)
                | Action::ModifyStyle
                | Action::CreateStyle
                | Action::ModifyCharacterStyle
                | Action::CreateCharacterStyle
                | Action::DeleteCharacterStyle
                | Action::DeleteStyle
        ) && (self.pending.is_some()
            || self.error.is_some()
            || ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Ime(_)))))
        {
            return;
        }
        // A keyboard Save may leave the size field focused. Commit it before
        // dispatch, so Save/New/Undo cannot silently operate on an older model.
        if action != Action::FontSize && !self.commit_font_size_input() {
            return;
        }
        let (start, end) = self.session.editor.selection;
        match action {
            Action::Bullets
            | Action::Numbering
            | Action::ListSettings
            | Action::Link
            | Action::Table
            | Action::Picture
            | Action::PageSetup
            | Action::PageFlow
            | Action::PageBreak
            | Action::PrintLayout
            | Action::PrintPreview
            | Action::ExportPdf
            | Action::ExportDocx
            | Action::Print => self.structure_action(action, ctx),
            Action::File => self.backstage = true,
            Action::New => self.request(Pending::New, ctx),
            Action::Open => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Open document (DOCX subset is restricted)")
                    .add_filter("Documents", &["bfdoc", "docx"])
                    .pick_file()
                {
                    match Session::open(&path) {
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
            Action::Style(style) => {
                let result = self.session.editor.apply_paragraph_style(start, end, style);
                self.fail(result);
                self.sync_view();
            }
            Action::CharacterStyle(style) => {
                let result = self.session.editor.apply_character_style(start, end, style);
                self.fail(result);
                self.sync_view();
            }
            Action::ClearFormatting => {
                let result = self.session.editor.clear_character_formatting(start, end);
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
            Action::ModifyCharacterStyle => {
                if let Some(Some(id)) = self.session.editor.selected_character_style() {
                    self.open_character_style_dialog(id, ctx);
                }
            }
            Action::CreateCharacterStyle => {
                let id = self.target_style();
                self.open_style_dialog(id, ctx);
                self.style_dialog = Some(StyleDialog::create_character(
                    &self.session.editor.document,
                    id,
                ));
            }
            Action::DeleteCharacterStyle => {
                if let Some(Some(id)) = self.session.editor.selected_character_style() {
                    self.open_delete_character_style(id, ctx);
                }
            }
            Action::DeleteStyle => self.open_delete_style_dialog(self.target_style(), ctx),
            Action::ModifyStyle | Action::CreateStyle => {
                let id = self
                    .session
                    .editor
                    .document
                    .locate(start)
                    .map(|(p, _)| self.session.editor.document.paragraphs[p].paragraph_style)
                    .unwrap_or_default();
                self.open_style_dialog(id, ctx);
                if action == Action::CreateStyle {
                    self.style_dialog = Some(StyleDialog::create(
                        id,
                        &self.session.editor.document.style_catalog,
                    ));
                }
            }
            Action::IncreaseIndent | Action::DecreaseIndent => {
                let indices = crate::structure::paragraph_indices(
                    &self.session.editor.document,
                    (start, end),
                );
                let all_lists = indices.is_ok_and(|r| {
                    r.into_iter().all(|i| {
                        self.session.editor.document.paragraphs[i]
                            .structure
                            .list
                            .is_some()
                    })
                });
                let result = if all_lists {
                    self.session
                        .editor
                        .change_list_level(action == Action::IncreaseIndent)
                } else {
                    self.session.editor.step_paragraph_indent(
                        start,
                        end,
                        action == Action::IncreaseIndent,
                    )
                };
                self.fail(result);
                self.sync_view();
            }
            Action::TabPositions => {
                self.tab_positions_dialog = Some(PositionsDialog::new(&self.session.editor));
                self.focus_document = false;
                self.focus_font_size = false;
                self.palette = false;
                ctx.memory_mut(|m| m.surrender_focus(Id::new(EDIT_ID)));
            }
            Action::TabInterval => {
                self.tab_dialog = Some(TabDialog::new(&self.session.editor));
                self.focus_document = false;
                self.focus_font_size = false;
                self.palette = false;
                ctx.memory_mut(|m| m.surrender_focus(Id::new(EDIT_ID)));
            }
            Action::ParagraphSpacing => {
                self.paragraph_dialog = Some(ParagraphDialog::default());
                self.focus_document = false;
                self.focus_font_size = false;
                self.palette = false;
                ctx.memory_mut(|m| m.surrender_focus(Id::new(EDIT_ID)));
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
                self.focus_document = false;
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
            Action::Zoom100 => {
                self.zoom = 100.;
                self.zoom_width = false;
            }
            Action::ZoomWidth => self.zoom_width = true,
            Action::Navigation => self.navigation = !self.navigation,
            Action::Rulers => self.rulers = !self.rulers,
            Action::Formatting => self.formatting = !self.formatting,
        }
        if self.structure_dialog.is_some() {
            self.focus_document = false;
            return;
        }
        if action != Action::FontSize && action != Action::Ribbon {
            self.ribbon_temporary = false;
            self.ribbon_focus = None;
            self.ribbon_menu_index = None;
            egui::Popup::close_all(ctx);
            if !matches!(
                action,
                Action::Find
                    | Action::Replace
                    | Action::ParagraphSpacing
                    | Action::FontDialog
                    | Action::ModifyStyle
                    | Action::CreateStyle
                    | Action::ModifyCharacterStyle
                    | Action::CreateCharacterStyle
                    | Action::DeleteCharacterStyle
                    | Action::DeleteStyle
                    | Action::Zoom
                    | Action::TabInterval
                    | Action::TabPositions
                    | Action::File
            ) {
                self.focus_document = true;
            }
        }
    }

    fn command_name(&self, action: Action) -> &str {
        match action {
            Action::Style(id) => self.session.editor.document.style_catalog.name(id),
            Action::CharacterStyle(Some(id)) => {
                self.session.editor.document.character_style_name(id)
            }
            _ => action.label(),
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
        } else if (action == Action::Bold && label == "B")
            || action == Action::CharacterStyle(Some(CharacterStyle::Strong))
        {
            RichText::new(label).strong()
        } else if (action == Action::Italic && label == "I")
            || action == Action::CharacterStyle(Some(CharacterStyle::Emphasis))
        {
            RichText::new(label).italics()
        } else if action == Action::Underline {
            RichText::new(label).underline()
        } else {
            RichText::new(label)
        };
        let mut button = Button::new(text).truncate().selected(self.checked(action));
        if self.collecting_menu {
            // A trailing flexible lane aligns menu labels with the leading edge.
            button = button.right_text("");
        }
        let response = ui
            .add_enabled_ui(enabled, |ui| ui.add_sized(size, button))
            .inner;
        // Hidden scroll-list rows must not anchor KeyTips over unrelated controls.
        // Keep every row in keyboard targets so arrow navigation can scroll to it.
        if !self.collecting_menu || ui.clip_rect().contains_rect(response.rect) {
            self.record(&format!("command-{action:?}"), response.rect);
        }
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
        if let Action::Style(id) = action {
            self.style_context_menu(&response, id);
        }
        if let Action::CharacterStyle(Some(id @ CharacterStyle::Custom(_))) = action {
            response.context_menu(|ui| {
                if ui
                    .add_enabled(enabled, Button::new("Modify character style…"))
                    .clicked()
                {
                    self.open_character_style_dialog(id, ui.ctx());
                    ui.close();
                }
                if ui
                    .add_enabled(enabled, Button::new("Delete character style…"))
                    .clicked()
                {
                    self.open_delete_character_style(id, ui.ctx());
                    ui.close();
                }
            });
        }
        let tip = if self.composing && !enabled {
            "Finish text composition first."
        } else if matches!(
            action,
            Action::ModifyCharacterStyle | Action::DeleteCharacterStyle
        ) && !enabled
        {
            "Choose one custom character style; built-ins cannot be modified."
        } else if action == Action::CreateCharacterStyle && !enabled {
            "Limit: 64 custom character styles per document."
        } else if action == Action::CharacterStyle(None) {
            "Remove the named character style; retain direct formatting."
        } else if action == Action::ClearFormatting {
            "Clear direct font overrides; retain paragraph and character styles."
        } else if action == Action::DeleteStyle
            && !matches!(self.target_style(), ParagraphStyle::Custom(_))
        {
            "Built-in styles cannot be deleted. Choose a custom style."
        } else {
            self.command_name(action)
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
                        Rect::from_min_size(rect.min + vec2(10., 6.), vec2(24., 24.)),
                        egui::Image::new(icon).fit_to_exact_size(vec2(24., 24.)),
                    );
                }
                let title_left = if self.quick_access_below || self.backstage {
                    43.
                } else {
                    self.quick_access_bar(ui, rect.min + vec2(43., 4.));
                    43. + (self.quick_access.len() + 1) as f32 * 30. + 8.
                };
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
                    rect.min + vec2(title_left, 4.),
                    pos2(search_x - 16., rect.top() + 32.),
                );
                ui.put(title_rect, egui::Label::new(&title).truncate())
                    .on_hover_text(&title);
                self.record("document-title", title_rect);
                let search_rect = Rect::from_min_size(
                    rect.min + vec2(search_x, 4.),
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
                    true,
                );
                let response = ui.put(
                    Rect::from_min_max(
                        search_rect.min + vec2(28., 4.),
                        search_rect.max - vec2(5., 2.),
                    ),
                    egui::TextEdit::singleline(&mut self.command_search)
                        .id(Id::new(SEARCH_ID))
                        .frame(false)
                        .hint_text("Search"),
                );
                let response = response.on_hover_text("Search commands (Alt+Q).");
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::TextEdit,
                        true,
                        "Search commands (Alt+Q)",
                    )
                });
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
            Action::ClearFormatting => "Ctrl+Space",
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
        let shortcut = if cfg!(target_os = "macos") && action != Action::ClearFormatting {
            shortcut.replace("Ctrl", "Command")
        } else {
            shortcut.to_owned()
        };
        let mut tip = format!(
            "{}{}{}",
            action.label(),
            if shortcut.is_empty() { "" } else { " · " },
            shortcut
        );
        if matches!(action, Action::IncreaseIndent | Action::DecreaseIndent) {
            tip.push_str("\nChange each selected paragraph's left indent by 36 pt, within 0–144 pt. Preserve right and first-line/hanging indents. One Undo restores the change.");
        }
        if action == Action::ClearFormatting {
            tip.push_str("\nRemove direct bold, italic, underline and size overrides; inherit the paragraph style. Paragraph alignment stays unchanged. At a caret, reset the next text you type.");
        }
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
        // The parent already reserved this group's full width. A scope would
        // advance it again using only the child content, shrinking the gutter.
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(bounds));
        self.home_contents(&mut child, index, width, bounds);
        self.ribbon_group = None;
        if index < 4 {
            let x = bounds.right() + 4.;
            self.record(
                &format!("home-separator-{}", HOME_GROUPS[index]),
                Rect::from_min_max(
                    pos2(x - 0.5, bounds.top() + 3.),
                    pos2(x + 0.5, bounds.bottom() - 3.),
                ),
            );
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
        let mut font = FontId::proportional(13.);
        let row_height = ui.fonts_mut(|f| f.row_height(&font));
        // Installed interface fonts have different line metrics. Keep the real
        // input hit target inside its row instead of clipping an oversized widget.
        font.size *= ((edit_rect.height() - 2.) / row_height).min(1.);
        let response = ui
            .add_enabled_ui(!self.composing, |ui| {
                ui.put(
                    edit_rect,
                    egui::TextEdit::singleline(&mut self.font_size_text)
                        .id(id)
                        .font(font)
                        .desired_width(0.)
                        .frame(false)
                        .vertical_align(Align::Center)
                        .margin(vec2(4., 1.))
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
        let top = Rect::from_min_size(rect.min, vec2(rect.width(), rect.height() * 0.54));
        let response = chrome::button(ui, top, Action::Paste.label(), enabled, false);
        self.record("command-Paste", response.rect);
        self.track_ribbon(
            &response,
            "command-Paste",
            TargetKind::Action(Action::Paste),
        );
        chrome::paint(ui, Icon::Paste, top.center(), 26., color, enabled);
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
            enabled,
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
        // Keep the established horizontal hit targets while tightening row pitch.
        let row_scale = (RIBBON_HEIGHT - 16.) / 88.;
        let r = |x: f32, y: f32, w: f32, h: f32| {
            Rect::from_min_size(origin + vec2(x, y * row_scale), vec2(w, h * row_scale))
        };
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
                let full = width >= 280.;
                let family = if full { 92. } else { width - 182. };
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
                if full {
                    self.unavailable_menu(
                        ui,
                        r(x + 56., 5., 32., 25.),
                        Icon::Text("Aa"),
                        "Change case",
                        "Changing letter case is planned.",
                    );
                }
                self.ribbon_control(
                    ui,
                    r(x + if full { 92. } else { 56. }, 5., 25., 25.),
                    Icon::ClearFormatting,
                    "",
                    Action::ClearFormatting,
                );
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
                    [
                        origin + vec2(170., 36. * row_scale),
                        origin + vec2(170., 57. * row_scale),
                    ],
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
                for (i, (icon, action)) in [
                    (Icon::Bullets, Action::Bullets),
                    (Icon::Numbers, Action::Numbering),
                    (Icon::Multilevel, Action::ListSettings),
                ]
                .into_iter()
                .enumerate()
                {
                    self.ribbon_control(ui, r(i as f32 * 31., 5., 30., 25.), icon, "", action);
                }
                ui.painter().line_segment(
                    [
                        origin + vec2(95., 7. * row_scale),
                        origin + vec2(95., 28. * row_scale),
                    ],
                    Stroke::new(1.0_f32, chrome::LINE),
                );
                for (i, action) in [Action::DecreaseIndent, Action::IncreaseIndent]
                    .into_iter()
                    .enumerate()
                {
                    self.ribbon_control(
                        ui,
                        r(99. + i as f32 * 26., 5., 25., 25.),
                        Icon::Indent(i == 1),
                        "",
                        action,
                    );
                }
                for (i, (icon, name)) in [(Icon::Sort, "Sort"), (Icon::Text("¶"), "Show marks")]
                    .into_iter()
                    .enumerate()
                {
                    self.unavailable(
                        ui,
                        r(151. + i as f32 * 26., 5., 25., 25.),
                        icon,
                        "",
                        name,
                        "Sorting and formatting marks are planned.",
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
                self.ribbon_control(
                    ui,
                    r(101., 34., 31., 25.),
                    Icon::Spacing,
                    "",
                    Action::ParagraphSpacing,
                );
                ui.painter().line_segment(
                    [
                        origin + vec2(135., 36. * row_scale),
                        origin + vec2(135., 57. * row_scale),
                    ],
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
                let gallery = r(0., 3., width - 1., 63.);
                ui.painter().rect_stroke(
                    gallery,
                    2.,
                    Stroke::new(1.0_f32, chrome::LINE),
                    egui::StrokeKind::Inside,
                );
                let count = if width >= 280. { 3 } else { 1 };
                let card = (width - 19.) / count as f32;
                let selected = self.session.editor.selected_paragraph_style();
                let styles = if count == 1 {
                    vec![selected.unwrap_or_default()]
                } else {
                    let mut styles = ParagraphStyle::ALL.to_vec();
                    if let Some(id @ ParagraphStyle::Custom(_)) = selected {
                        styles[2] = id;
                    }
                    styles
                };
                for (i, style) in styles.into_iter().enumerate() {
                    let name = self
                        .session
                        .editor
                        .document
                        .style_catalog
                        .name(style)
                        .to_owned();
                    let action = Action::Style(style);
                    let rect = r(2. + i as f32 * card, 5., card - 3., 59.);
                    ui.painter().rect_filled(rect, 2., Color32::WHITE);
                    let response = chrome::button(
                        ui,
                        rect,
                        &name,
                        self.enabled(action),
                        selected == Some(style),
                    );
                    self.record(&format!("command-{action:?}"), response.rect);
                    self.track_ribbon(
                        &response,
                        &format!("command-{action:?}"),
                        TargetKind::Action(action),
                    );
                    let resolved = self
                        .session
                        .editor
                        .document
                        .style_catalog
                        .resolve(style)
                        .character;
                    let mut preview = RichText::new("AaBbCcDd").font(FontId::new(
                        resolved.font_size.points().clamp(14., 18.),
                        egui::FontFamily::Name(
                            if resolved.bold { "body-bold" } else { "body" }.into(),
                        ),
                    ));
                    if resolved.italic {
                        preview = preview.italics();
                    }
                    if resolved.underline {
                        preview = preview.underline();
                    }
                    let galley = egui::WidgetText::from(preview).into_galley(
                        ui,
                        Some(egui::TextWrapMode::Truncate),
                        (rect.width() - 8.).max(1.),
                        egui::TextStyle::Body,
                    );
                    if selected == Some(style) {
                        ui.painter().rect_stroke(
                            rect,
                            2.,
                            Stroke::new(1.2_f32, ACCENT),
                            egui::StrokeKind::Inside,
                        );
                    }
                    let caption = Rect::from_min_max(
                        pos2(rect.left() + 2., rect.bottom() - 17.),
                        rect.max - vec2(2., 2.),
                    );
                    let sample = Rect::from_min_max(
                        rect.min + vec2(3., 3.),
                        pos2(rect.right() - 3., caption.top() - 2.),
                    );
                    ui.painter()
                        .rect_filled(caption, 1., Color32::from_gray(234));
                    let color = if self.enabled(action) {
                        INK
                    } else {
                        Color32::from_gray(135)
                    };
                    ui.painter().with_clip_rect(sample).galley(
                        sample.center() - galley.size() / 2.,
                        galley,
                        color,
                    );
                    let label = ui.painter().layout(
                        name.clone(),
                        FontId::proportional(10.),
                        color,
                        f32::INFINITY,
                    );
                    let label = if label.size().x > caption.width() - 4. {
                        egui::WidgetText::from(RichText::new(&name).size(10.)).into_galley(
                            ui,
                            Some(egui::TextWrapMode::Truncate),
                            (caption.width() - 4.).max(1.),
                            egui::TextStyle::Small,
                        )
                    } else {
                        label
                    };
                    ui.painter().with_clip_rect(caption).galley(
                        caption.center() - label.size() / 2.,
                        label,
                        color,
                    );
                    self.record(&format!("gallery-sample-{style:?}"), sample);
                    self.record(&format!("gallery-caption-{style:?}"), caption);
                    self.style_context_menu(&response, style);
                    if response.on_hover_text(format!("Apply {} to the selected paragraphs. Direct font formatting and alignment stay unchanged.", name))
                        .on_disabled_hover_text("Finish the current dialog or text composition first.").clicked() {
                        self.execute(action, ui.ctx());
                    }
                }
                let response = chrome::control(
                    ui,
                    r(width - 17., 4., 15., 61.),
                    Icon::Chevron(false),
                    "",
                    "More styles",
                    self.enabled(Action::Style(ParagraphStyle::Normal)),
                    false,
                );
                self.record("home-menu-Styles", response.rect);
                self.track_ribbon(&response, "home-menu-Styles", TargetKind::Menu);
                egui::Popup::menu(&response)
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                    .show(|ui| {
                        self.collecting_menu = true;
                        self.styles_menu(ui, false);
                        self.collecting_menu = false;
                    });
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
            pos2(origin.x + width / 2., origin.y + 78. * row_scale),
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
    fn style_context_menu(&mut self, response: &egui::Response, id: ParagraphStyle) {
        if response.secondary_clicked() && self.enabled(Action::ModifyStyle) {
            self.style_context = Some((
                id,
                response
                    .interact_pointer_pos()
                    .unwrap_or(response.rect.center()),
                response.layer_id,
            ));
            egui::Popup::open_id(&response.ctx, Id::new("style-context"));
        }
    }

    fn show_style_context_menu(&mut self, ctx: &egui::Context) {
        let Some((id, position, layer)) = self.style_context else {
            return;
        };
        // Render independently: opening a context menu replaces its source Styles popup.
        egui::Popup::new(Id::new("style-context"), ctx.clone(), position, layer)
            .kind(egui::PopupKind::Menu)
            .layout(Layout::top_down_justified(Align::Min))
            .open_memory(None)
            .show(|ui| {
                let modify = ui.add_enabled(
                    self.enabled(Action::ModifyStyle),
                    Button::new("Modify style…"),
                );
                self.record(&format!("style-modify-{id:?}"), modify.rect);
                if modify.clicked() {
                    self.open_style_dialog(id, ui.ctx());
                    egui::Popup::close_all(ui.ctx());
                }
                let delete = ui
                    .add_enabled(
                        self.enabled(Action::ModifyStyle)
                            && matches!(id, ParagraphStyle::Custom(_)),
                        Button::new("Delete style…"),
                    )
                    .on_disabled_hover_text("Built-in styles cannot be deleted.");
                self.record(&format!("style-delete-{id:?}"), delete.rect);
                if delete.clicked() {
                    self.open_delete_style_dialog(id, ui.ctx());
                    egui::Popup::close_all(ui.ctx());
                }
            });
        if !egui::Popup::is_id_open(ctx, Id::new("style-context")) {
            self.style_context = None;
        }
    }

    fn target_style(&self) -> ParagraphStyle {
        self.session
            .editor
            .document
            .locate(self.session.editor.selection.0)
            .map(|(p, _)| self.session.editor.document.paragraphs[p].paragraph_style)
            .unwrap_or_default()
    }

    fn open_delete_character_style(&mut self, id: CharacterStyle, ctx: &egui::Context) {
        if !self.enabled(Action::ModifyStyle)
            || ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Ime(_))))
        {
            return;
        }
        if let Some(dialog) = DeleteStyleDialog::character(id, &self.session.editor.document) {
            self.delete_style_dialog = Some(dialog);
            self.focus_document = false;
            self.focus_font_size = false;
            self.palette = false;
            ctx.memory_mut(|m| m.surrender_focus(Id::new(EDIT_ID)));
        }
    }
    fn open_delete_style_dialog(&mut self, id: ParagraphStyle, ctx: &egui::Context) {
        if !self.enabled(Action::ModifyStyle)
            || !matches!(id, ParagraphStyle::Custom(_))
            || ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Ime(_))))
        {
            return;
        }
        if let Some(dialog) =
            DeleteStyleDialog::new(id, &self.session.editor.document.style_catalog)
        {
            self.delete_style_dialog = Some(dialog);
            self.focus_document = false;
            self.focus_font_size = false;
            self.palette = false;
            ctx.memory_mut(|m| m.surrender_focus(Id::new(EDIT_ID)));
        }
    }

    fn open_style_dialog(&mut self, id: ParagraphStyle, ctx: &egui::Context) {
        self.style_dialog = Some(StyleDialog::new(
            id,
            &self.session.editor.document.style_catalog,
        ));
        self.focus_document = false;
        self.focus_font_size = false;
        self.palette = false;
        ctx.memory_mut(|m| m.surrender_focus(Id::new(EDIT_ID)));
    }

    fn open_character_style_dialog(&mut self, id: CharacterStyle, ctx: &egui::Context) {
        let paragraph = self.target_style();
        if let Some(dialog) =
            StyleDialog::modify_character(&self.session.editor.document, paragraph, id)
        {
            self.open_style_dialog(paragraph, ctx);
            self.style_dialog = Some(dialog);
        }
    }
    fn jump_to_style(&mut self, source: Action, action: Action) {
        self.style_navigation_notice = None;
        if self.style_return.is_none() {
            self.style_return = Some((
                source,
                self.style_filter.clone(),
                self.style_scope,
                self.style_used_only,
            ));
        }
        self.style_filter.clear();
        self.style_scope = Default::default();
        self.ribbon_menu_index = None;
        self.style_jump = Some(action);
        self.style_jump_ready = false;
        self.focus_document = false;
    }

    fn return_to_style(&mut self) {
        if let Some((source, query, scope, used)) = self.style_return.take() {
            self.style_filter = query;
            self.style_scope = scope;
            self.style_used_only = used;
            self.ribbon_menu_index = None;
            self.style_jump = Some(source);
            self.style_jump_ready = false;
            self.focus_document = false;
        }
    }

    fn style_menu_command(
        &mut self,
        ui: &mut Ui,
        name: &str,
        action: Action,
        usage: crate::style_filter::Usage,
        used: &crate::style_filter::UsedStyles,
    ) {
        if self.style_jump_ready && self.style_jump == Some(action) {
            self.ribbon_menu_index = Some(self.ribbon_menu_targets.len());
            self.style_jump = None;
        }
        self.command(ui, name, action, vec2(210., 27.));
        let label = ui.small(usage.label()).on_hover_text(match usage {
            crate::style_filter::Usage::Direct => "Assigned to document content. May also be a parent of another used style.",
            crate::style_filter::Usage::Parent => "Used only through inheritance by a style assigned to document content.",
            crate::style_filter::Usage::Unused => "Neither assigned to document content nor required as a parent. Future typing and next-style links do not count.",
        });
        self.record(
            &format!("style-usage-{action:?}-{}", usage.label()),
            label.rect,
        );
        let document = &self.session.editor.document;
        let consumers: Vec<(Action, String)> = match action {
            Action::Style(id) => used
                .paragraph_consumers(document, id)
                .into_iter()
                .map(|id| {
                    (
                        Action::Style(id),
                        document.style_catalog.name(id).to_owned(),
                    )
                })
                .collect(),
            Action::CharacterStyle(Some(id)) => used
                .character_consumers(document, id)
                .into_iter()
                .map(|id| {
                    (
                        Action::CharacterStyle(Some(id)),
                        document.character_style_name(id).to_owned(),
                    )
                })
                .collect(),
            _ => Vec::new(),
        };
        if !consumers.is_empty() {
            let text = format!(
                "Required by: {}",
                consumers
                    .iter()
                    .map(|(_, name)| name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            let heading = ui.small("Required by:");
            self.record(&format!("style-consumers-{action:?}-{text}"), heading.rect);
            for (destination, name) in consumers {
                let response = ui.add(
                    egui::Button::new(RichText::new(&name).small())
                        .wrap()
                        .frame(false),
                );
                let key = format!("style-jump-{action:?}-{destination:?}");
                self.record(&key, response.rect);
                self.track_ribbon(&response, &key, TargetKind::StyleJump(action, destination));
                if response.on_hover_text("Go to this style without applying it. Clears name/category filters; keeps Used in document.").clicked() {
                    self.jump_to_style(action, destination);
                    ui.ctx().request_repaint();
                }
            }
        }
    }

    fn style_destination_visible(&self, action: Action) -> bool {
        let document = &self.session.editor.document;
        let used = crate::style_filter::UsedStyles::collect(document);
        let (exists, character, custom, in_use) = match action {
            Action::Style(id) => (
                document.style_catalog.get(id).is_some(),
                false,
                matches!(id, ParagraphStyle::Custom(_)),
                used.paragraphs.contains(&id),
            ),
            Action::CharacterStyle(id) => (
                id.is_none_or(|id| document.character_style_exists(id)),
                true,
                matches!(id, Some(CharacterStyle::Custom(_))),
                used.character(id),
            ),
            _ => return false,
        };
        exists
            && self.style_scope.matches(
                &self.style_filter,
                self.command_name(action),
                character,
                custom,
            )
            && (!self.style_used_only || in_use)
    }

    fn styles_menu(&mut self, ui: &mut Ui, overflow: bool) {
        // Resolve against the current document and filters, never leave an unreachable
        // jump pending: an undo, rename or usage change can invalidate the bookmark.
        let unavailable = self
            .style_jump
            .filter(|action| !self.style_destination_visible(*action));
        if let Some(action) = unavailable {
            self.style_navigation_notice = Some(format!(
                "Cannot show {}. The style was removed or is hidden by the restored filters.",
                self.command_name(action)
            ));
            self.style_jump = None;
            self.ribbon_menu_index = None;
            ui.ctx()
                .memory_mut(|m| m.request_focus(Id::new("style-filter-input")));
        }
        self.style_jump_ready = self.style_jump.is_some();
        ui.set_width(220.);
        let filter = ui.add(
            egui::TextEdit::singleline(&mut self.style_filter)
                .id(Id::new("style-filter-input"))
                .hint_text("Filter styles")
                .char_limit(64)
                .desired_width(210.),
        );
        let filter = filter.on_hover_text("Filter styles by name (F3).");
        self.record("style-filter-input", filter.rect);
        if filter.clicked() || filter.changed() {
            self.style_jump = None;
            self.style_navigation_notice = None;
            self.ribbon_menu_index = None;
        }
        ui.horizontal(|ui| {
            let scope = ui.button(self.style_scope.label());
            self.record("style-filter-scope", scope.rect);
            if scope
                .on_hover_text("Cycle: All, Paragraph, Character, Custom. Tab/Space also work.")
                .clicked()
            {
                self.style_jump = None;
                self.style_navigation_notice = None;
                self.style_scope = self.style_scope.next();
                self.ribbon_menu_index = None;
            }
            let clear = ui.button("Reset");
            self.record("style-filter-reset", clear.rect);
            if clear.clicked() {
                self.style_jump = None;
                self.style_navigation_notice = None;
                self.style_filter.clear();
                self.style_scope = Default::default();
                self.style_used_only = false;
                self.ribbon_menu_index = None;
            }
        });
        let used_toggle = ui.checkbox(&mut self.style_used_only, "Used in document");
        self.record("style-filter-used", used_toggle.rect);
        if used_toggle.changed() {
            self.style_jump = None;
            self.style_navigation_notice = None;
            self.ribbon_menu_index = None;
        }
        used_toggle.on_hover_text("Document text and empty paragraphs, plus parent styles. Excludes future typing and next-style links.");
        if let Some((source, _, _, _)) = &self.style_return {
            let label = format!("Back to: {}", self.command_name(*source));
            let back = ui.add_sized(vec2(210., 23.), Button::new(&label).truncate())
                .on_hover_text(format!("{label}\nRestore the original name, category and used-only filters. Does not apply formatting."));
            self.record(&format!("style-return-label-{label}"), back.rect);
            self.record("style-return", back.rect);
            self.track_ribbon(&back, "style-return", TargetKind::StyleReturn);
            if back.clicked() {
                self.return_to_style();
            }
        }
        let used = crate::style_filter::UsedStyles::collect(&self.session.editor.document);
        let mut matches = 0;
        egui::ScrollArea::vertical()
            .id_salt("style-list")
            .max_height(if overflow {
                // The combined menu must also leave room for Editing commands.
                if self.style_return.is_some() {
                    100.
                } else {
                    125.
                }
            } else {
                (ui.ctx().content_rect().bottom() - ui.cursor().top() - 118.).clamp(65., 300.)
            })
            .show(ui, |ui| {
                if let Some(notice) = &self.style_navigation_notice {
                    let response = ui.label(notice);
                    self.record("style-navigation-notice", response.rect);
                }
                for style in self.session.editor.document.style_catalog.ids() {
                    let name = self
                        .session
                        .editor
                        .document
                        .style_catalog
                        .name(style)
                        .to_owned();
                    if self.style_scope.matches(
                        &self.style_filter,
                        &name,
                        false,
                        matches!(style, ParagraphStyle::Custom(_)),
                    ) && (!self.style_used_only || used.paragraphs.contains(&style))
                    {
                        matches += 1;
                        self.style_menu_command(
                            ui,
                            &name,
                            Action::Style(style),
                            used.paragraph_usage(style),
                            &used,
                        );
                    }
                }
                let mut character_heading = false;
                for style in [
                    None,
                    Some(CharacterStyle::Strong),
                    Some(CharacterStyle::Emphasis),
                ] {
                    let action = Action::CharacterStyle(style);
                    if self
                        .style_scope
                        .matches(&self.style_filter, action.label(), true, false)
                        && (!self.style_used_only || used.character(style))
                    {
                        if !character_heading {
                            ui.separator();
                            ui.small("Character styles");
                            character_heading = true;
                        }
                        matches += 1;
                        self.style_menu_command(
                            ui,
                            action.label(),
                            action,
                            used.character_usage(style),
                            &used,
                        );
                    }
                }
                let styles = self.session.editor.document.character_styles.clone();
                for style in styles {
                    if !self
                        .style_scope
                        .matches(&self.style_filter, &style.name, true, true)
                        || (self.style_used_only
                            && !used.character(Some(CharacterStyle::Custom(style.id))))
                    {
                        continue;
                    }
                    if !character_heading {
                        ui.separator();
                        ui.small("Character styles");
                        character_heading = true;
                    }
                    matches += 1;
                    self.style_menu_command(
                        ui,
                        &style.name,
                        Action::CharacterStyle(Some(CharacterStyle::Custom(style.id))),
                        used.character_usage(Some(CharacterStyle::Custom(style.id))),
                        &used,
                    );
                }
                if matches == 0 {
                    ui.label("No matching styles");
                }
                self.command(
                    ui,
                    "Delete character style…",
                    Action::DeleteCharacterStyle,
                    vec2(210., 27.),
                );
                self.command(
                    ui,
                    "Modify character style…",
                    Action::ModifyCharacterStyle,
                    vec2(210., 27.),
                );
                self.command(
                    ui,
                    "Create character style…",
                    Action::CreateCharacterStyle,
                    vec2(210., 27.),
                );
            });
        ui.separator();
        self.command(ui, "Modify style…", Action::ModifyStyle, vec2(210., 27.));
        self.command(ui, "Create style…", Action::CreateStyle, vec2(210., 27.));
        self.command(ui, "Delete style…", Action::DeleteStyle, vec2(210., 27.));
        if self.session.editor.document.style_catalog.custom.len()
            >= crate::model::MAX_CUSTOM_STYLES
        {
            ui.small("Limit: 64 custom styles per document.");
        }
    }

    fn ribbon_contents(&mut self, ui: &mut Ui) {
        let shell = ui.max_rect();
        ui.painter().rect_filled(shell, 2., chrome::SURFACE);
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
                    let (menu_response, _) = egui::containers::menu::MenuButton::new("More")
                        .config(
                            egui::containers::menu::MenuConfig::new()
                                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
                        )
                        .ui(ui, |ui| {
                            self.collecting_menu = true;
                            ui.label("Styles");
                            self.styles_menu(ui, true);
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
                    self.record("home-overflow", menu_response.rect);
                    self.track_ribbon(&menu_response, "home-overflow", TargetKind::Menu);
                    self.ribbon_group = None;
                }
            });
        } else if self.tab == 1 || self.tab == 4 {
            self.structure_ribbon(ui);
        } else if self.tab == 8 {
            self.view_ribbon(ui);
        } else if self.tab == 9 {
            ui.label(format!("BreakFree Docs {VERSION}"));
            ui.label("Local .bfdoc documents · pages, preview and PDF");
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
            .fixed_pos(pos2(
                ctx.content_rect().left(),
                TITLE_HEIGHT + TABS_HEIGHT + if self.quick_access_below { 32. } else { 0. },
            ))
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
        if self.replace_all_dialog.is_some()
            || self.font_dialog.is_some()
            || self.paragraph_dialog.is_some()
            || self.style_dialog.is_some()
            || self.delete_style_dialog.is_some()
            || self.zoom_dialog.is_some()
            || self.tab_dialog.is_some()
            || self.tab_positions_dialog.is_some()
            || self.tab_sources_dialog.is_some()
            || self.structure_dialog.is_some()
            || self.quick_reset
            || self.quick_order.is_some()
        {
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
                (Key::K, false, Action::Link),
                (Key::Enter, false, Action::PageBreak),
                (Key::P, true, Action::FontSize),
                (Key::P, false, Action::PrintPreview),
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
                // Ctrl+Space is a physical Control chord on every platform. Do not
                // take Command+Space, AltGr, shifted chords or IME confirmation.
                let clear = self.pending.is_none()
                    && self.error.is_none()
                    && !self.backstage
                    && !self.palette
                    && ctx.input_mut(|i| {
                        if i.events.iter().any(|e| matches!(e, egui::Event::Ime(_))) {
                            return false;
                        }
                        let mut activate = false;
                        i.events.retain(|event| {
                            if let egui::Event::Key {
                                key: Key::Space,
                                pressed: true,
                                repeat,
                                modifiers,
                                ..
                            } = event
                                && modifiers.ctrl
                                && !modifiers.alt
                                && !modifiers.shift
                                && !modifiers.mac_cmd
                            {
                                activate |= !repeat;
                                false
                            } else {
                                true
                            }
                        });
                        activate
                    });
                if clear {
                    self.execute(Action::ClearFormatting, ctx);
                }
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
        let find_focus = self.navigation
            && ctx.memory(|m| {
                m.has_focus(Id::new("find-text"))
                    || m.had_focus_last_frame(Id::new("find-text"))
                    || self
                        .find_option_ids
                        .iter()
                        .flatten()
                        .any(|id| m.has_focus(*id) || m.had_focus_last_frame(*id))
            });
        let find_ime = find_focus
            && (self.find_composing
                || ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Ime(_)))));
        if !find_ime && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            if find_focus {
                self.navigation = false;
                self.focus_document = true;
            }
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
                        self.record("zoom-status",percent.rect);self.status_focus=Some(percent.id);if percent.clicked(){percent.request_focus();}
                        if percent.on_hover_text("Open Zoom dialog").clicked(){self.execute(Action::Zoom,ctx);}
                        let plus=ui.add_sized([22.,20.],Button::new("+").frame_when_inactive(false));self.record("zoom-in",plus.rect);if plus.clicked(){plus.request_focus();}
                        if plus.on_hover_text("Zoom in").clicked(){self.zoom=(self.zoom+10.).min(200.);self.zoom_width=false;}
                        ui.spacing_mut().slider_width=110.;
                        let slider=ui.add(egui::Slider::new(&mut self.zoom,50.0..=200.0).show_value(false).clamping(egui::SliderClamping::Always));self.record("zoom-slider",slider.rect);if slider.changed(){self.zoom_width=false;}
                        let minus=ui.add_sized([22.,20.],Button::new("−").frame_when_inactive(false));self.record("zoom-out",minus.rect);if minus.clicked(){minus.request_focus();}
                        if minus.on_hover_text("Zoom out").clicked(){self.zoom=(self.zoom-10.).max(50.);self.zoom_width=false;}
                        if ui.available_width()>195. {ui.separator();ui.small(if self.paged_editing {self.page_count.map_or("Print Layout".into(),|n|format!("Print Layout · {n} pages"))} else {"Continuous layout".into()}).on_hover_text("Use View → Print Layout for computed pages; Print preview and PDF use the same layout.");}
                    });
                });
            });
        self.record("status", panel.response.rect);
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
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let (a, b) = self.session.editor.selection;
                    ui.label(format!("{} selected characters", b - a));
                    ui.label(if a == b {
                        "Future typing"
                    } else {
                        "Selected text"
                    });
                    ui.vertical(|ui| {
                        let rows = crate::formatting::selection(&self.session.editor);
                        if rows.is_empty() {
                            ui.label("No text properties (paragraph separators only).");
                        }
                        for row in rows {
                            ui.label(
                                egui::RichText::new(format!("{}: {}", row.name, row.value))
                                    .strong(),
                            );
                            ui.add(egui::Label::new(row.source).wrap());
                        }
                    });
                    ui.separator();
                    ui.label(
                        egui::RichText::new(if a == b {
                            "Current paragraph"
                        } else {
                            "Selected paragraphs"
                        })
                        .strong(),
                    );
                    for row in crate::formatting::paragraph_selection(&self.session.editor) {
                        let response = ui.label(
                            egui::RichText::new(format!("{}: {}", row.name, row.value)).strong(),
                        );
                        self.record(&format!("provenance-{}", row.name), response.rect);
                        ui.add(egui::Label::new(row.source).wrap());
                    }
                    let sources = ui
                        .button("Tabulation sources…")
                        .on_hover_text("Inspect interval and position sources. Ctrl+Alt+F9.");
                    self.record("formatting-tab-sources", sources.rect);
                    if sources.clicked() {
                        sources.request_focus();
                        self.open_tab_sources(ctx, None);
                    }
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
            });
        self.record("formatting", panel.response.rect);
    }

    fn document(&mut self, ui: &mut Ui) {
        if self.page_preview {
            self.page_view(ui);
            return;
        }
        if self.paged_editing {
            self.paged_document(ui);
            return;
        }
        if self.zoom_width {
            let fit = ((ui.available_width() - 48.) / CANVAS_WIDTH * 100.).clamp(50., 200.);
            if (self.zoom - fit).abs() > 0.01 {
                ui.ctx().request_repaint();
            }
            self.zoom = fit;
        }
        let zoom = self.zoom / 100.;
        let width = CANVAS_WIDTH * zoom;
        let margin = CANVAS_MARGIN * zoom;
        let text_width = width - 2. * margin;
        let font_size = 14.6667 * zoom;
        let model = self
            .tab_ruler_drag
            .as_ref()
            .map(|drag| &drag.draft.preview)
            .or_else(|| self.ruler_drag.as_ref().map(|drag| &drag.draft.preview))
            .unwrap_or(&self.session.editor.document)
            .clone();
        let typing = self.session.editor.typing_style;
        let typing_position = self.session.editor.selection.0;
        let natural = layout::galley(
            ui.ctx(),
            &model,
            &self.buffer,
            typing,
            typing_position,
            text_width,
            font_size,
        )
        .size()
        .y;
        let min_height = (ui.available_height() - 45.)
            .max(natural + 2. * margin)
            .max(300.);
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let space = ((ui.available_width() - width - 32.) / 2.).max(if self.rulers {
                    32.
                } else {
                    12.
                });
                ui.add_space(10.);
                ui.horizontal_top(|ui| {
                    ui.add_space(space);
                    ui.vertical(|ui| {
                        if self.rulers {
                            self.ruler(ui, width, zoom);
                            ui.add_space(5.);
                        }
                        let original_clip = ui.clip_rect();
                        let mut canvas_clip = original_clip;
                        if self.rulers {
                            canvas_clip.min.x += 26.;
                            if let Some(r) = self.regions.iter().find(|r| r.name == "ruler") {
                                canvas_clip.min.y = r.rect.bottom() + 5.;
                            }
                        }
                        ui.set_clip_rect(canvas_clip);
                        let frame = egui::Frame::new()
                            .fill(Color32::WHITE)
                            .stroke(Stroke::new(1.0_f32, Color32::from_gray(197)))
                            .shadow(egui::epaint::Shadow {
                                offset: [0, 2],
                                blur: 5,
                                spread: 0,
                                color: Color32::from_black_alpha(24),
                            })
                            .inner_margin(0)
                            .show(ui, |ui| {
                                ui.set_min_size(vec2(width, min_height));
                                ui.add_space(margin);
                                ui.horizontal_top(|ui| {
                                    ui.add_space(margin);
                                    ui.allocate_ui_with_layout(
                                        vec2(text_width, natural.max(40.)),
                                        Layout::top_down(Align::Min),
                                        |ui| {
                                            if self.ruler_drag.is_some()
                                                || self.tab_ruler_drag.is_some()
                                            {
                                                ui.disable();
                                            }
                                            self.text_surface(
                                                ui,
                                                &model,
                                                typing,
                                                typing_position,
                                                text_width,
                                                font_size,
                                                zoom,
                                                false,
                                            );
                                        },
                                    );
                                });
                                ui.add_space(margin);
                            });
                        self.record("continuous-canvas", frame.response.rect);
                        if self.rulers {
                            ui.set_clip_rect(Rect::from_min_max(
                                pos2(original_clip.left(), canvas_clip.top()),
                                original_clip.max,
                            ));
                            self.vertical_ruler(ui, frame.response.rect, zoom);
                        }
                        ui.set_clip_rect(original_clip);
                    });
                });
            });
    }

    #[allow(clippy::too_many_arguments)]
    fn text_surface(
        &mut self,
        ui: &mut Ui,
        model: &crate::model::Document,
        typing: crate::model::CharacterFormat,
        typing_position: usize,
        text_width: f32,
        font_size: f32,
        zoom: f32,
        paged: bool,
    ) {
        let id = Id::new(EDIT_ID);
        let mut state = egui::TextEdit::load_state(ui.ctx(), id).unwrap_or_default();
        state.clear_undoer();
        if let Some(range) = self.cursor_override.take() {
            state.cursor.set_char_range(Some(range));
        }
        state.store(ui.ctx(), id);
        if ui.memory(|m| m.has_focus(id)) {
            ui.input_mut(|i| {
                for event in &mut i.events {
                    if let egui::Event::Paste(text) = event {
                        *text = text.replace("\r\n", "\n").replace('\r', "\n");
                    }
                }
            });
        }
        // Modified Tab is reserved; never let Shift+Tab delete selected text.
        if ui.memory(|m| m.has_focus(id)) {
            ui.input_mut(|i|i.events.retain(|e| !matches!(e, egui::Event::Key { key: Key::Tab, modifiers, .. } if *modifiers != Modifiers::NONE)));
        }
        let events = ui.input(|i| i.events.clone());
        let focused = ui.memory(|m| m.has_focus(id));
        let mut commit = false;
        let mut cancel = false;
        if focused {
            for event in &events {
                if let egui::Event::Ime(ime) = event {
                    match ime {
                        egui::ImeEvent::Enabled | egui::ImeEvent::Preedit(_) => {
                            self.composing = true
                        }
                        egui::ImeEvent::Commit(text) => {
                            self.composing = false;
                            if text.is_empty() || text == "\n" || text == "\r" {
                                cancel = true;
                            } else {
                                commit = true;
                            }
                        }
                        egui::ImeEvent::Disabled => {
                            if self.composing {
                                cancel = true;
                            }
                            self.composing = false;
                        }
                    }
                }
            }
        }
        let search_range = if self.navigation
            && !self.composing
            && !self.find_composing
            && self
                .find_query
                .search(&self.buffer)
                .active(self.session.editor.selection)
                .is_some()
        {
            let (a, b) = self.session.editor.selection;
            Some(CCursorRange::two(CCursor::new(b), CCursor::new(a)))
        } else {
            None
        };
        let mut layouter = |ui: &Ui, text: &dyn egui::TextBuffer, wrap: f32| {
            let mut galley = if paged {
                let mut draft = crate::commands::Editor::new(model.clone());
                draft.typing_style = typing;
                if draft.document.text() != text.as_str() {
                    let (a, b, t) = text_change(&draft.document.text(), text.as_str());
                    let _ = draft.replace(a, b, &t, "Input projection");
                }
                crate::page_layout::Layout::build(ui.ctx(), &draft.document)
                    .map(|l| l.editable_galley(ui.ctx(), &draft.document, font_size / 11.))
                    .unwrap_or_else(|_| {
                        layout::galley(
                            ui.ctx(),
                            model,
                            text.as_str(),
                            typing,
                            typing_position,
                            wrap,
                            font_size,
                        )
                    })
            } else {
                layout::galley(
                    ui.ctx(),
                    model,
                    text.as_str(),
                    typing,
                    typing_position,
                    wrap,
                    font_size,
                )
            };
            if !ui.memory(|m| m.has_focus(id))
                && let Some(range) = search_range
            {
                egui::text_selection::visuals::paint_text_selection(
                    &mut galley,
                    ui.visuals(),
                    &range,
                    None,
                );
            }
            galley
        };
        let mut output = egui::TextEdit::multiline(&mut self.buffer)
            .lock_focus(true)
            .id(id)
            .frame(false)
            .margin(0)
            .desired_width(text_width)
            .desired_rows(1)
            .font(FontId::new(
                font_size,
                egui::FontFamily::Name("body".into()),
            ))
            .layouter(&mut layouter)
            .show(ui);
        if self.focus_document && ui.input(|i| i.focused) {
            if !output.response.has_focus() {
                output.response.request_focus();
            }
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    id,
                    egui::EventFilter {
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        ..Default::default()
                    },
                )
            });
            self.focus_document = false;
        }
        let keyboard_enter = !commit
            && events.iter().any(|e| {
                matches!(
                    e,
                    egui::Event::Key {
                        key: Key::Enter,
                        pressed: true,
                        ..
                    }
                )
            })
            && !events
                .iter()
                .any(|e| matches!(e, egui::Event::Paste(_) | egui::Event::Ime(_)));
        let selected_break = keyboard_enter && {
            let (a, b) = self.session.editor.selection;
            b == a + 1 && self.session.editor.document.text().chars().nth(a) == Some('\n')
        };
        if cancel && !commit || output.response.lost_focus() && self.composing {
            self.composing = false;
            self.sync_view();
        } else if output.response.changed()
            && !self.composing
            && (self.buffer != self.session.editor.document.text() || selected_break)
        {
            // Preserve the user's replacement range when equal prefixes/suffixes
            // would narrow a minimal diff and therefore the undo selection.
            let (a, b, text) = text_change_from_selection(
                &self.session.editor.document.text(),
                &self.buffer,
                self.session.editor.selection,
            );
            let paragraph_enter = keyboard_enter && text.contains('\n');
            let result = if paragraph_enter {
                self.session.editor.replace_keyboard_text(a, b, &text)
            } else {
                self.session.editor.replace(
                    a,
                    b,
                    &text,
                    if commit {
                        "Input method commit"
                    } else {
                        "Text input"
                    },
                )
            };
            self.fail(result);
            self.sync_view();
        } else if !self.composing
            && output.response.has_focus()
            && let Some(range) = output.cursor_range
        {
            let range = range.as_sorted_char_range();
            let selection = snap(&self.buffer, range.start, range.end);
            if selection != self.session.editor.selection {
                self.session.editor.selection = selection;
                self.session.editor.typing_style =
                    self.session.editor.document.style_at(selection.0);
            }
        }
        output.state.clear_undoer();
        output.state.store(ui.ctx(), id);
        output.response.context_menu(|ui| {
            for action in [Action::Cut, Action::Copy, Action::Paste, Action::SelectAll] {
                self.command(ui, action.label(), action, vec2(150., 26.));
            }
        });
        if self.scroll_to_selection {
            let caret = output
                .galley
                .pos_from_cursor(CCursor::new(self.session.editor.selection.0))
                .translate(output.galley_pos.to_vec2());
            ui.scroll_to_rect(caret, Some(Align::Center));
            self.scroll_to_selection = false;
        }
        self.object_overlays(ui, &output.galley, output.galley_pos, zoom);
        self.record("document-text", output.response.rect);
    }

    fn file_view(&mut self, ui: &mut Ui) {
        let full = ui.max_rect();

        let rail = Rect::from_min_max(full.min, pos2(full.left() + 180., full.bottom()));

        ui.painter().rect_filled(rail, 0., chrome::FILE_RAIL);

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

                let (rect, _) = ui.allocate_exact_size(vec2(156., 34.), Sense::hover());
                let home =
                    chrome::control(ui, rect, Icon::Home, "Home", "Home", true, !self.settings);

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

                let (rect, _) = ui.allocate_exact_size(vec2(156., 34.), Sense::hover());
                self.ribbon_control(ui, rect, Icon::Print, "Print", Action::Print);
                self.command(ui, "Export PDF", Action::ExportPdf, vec2(156., 30.));
                self.command(ui, "Export DOCX", Action::ExportDocx, vec2(156., 30.));

                ui.add_space(8.);

                let (rect, _) = ui.allocate_exact_size(vec2(156., 34.), Sense::hover());
                let options = chrome::control(
                    ui,
                    rect,
                    Icon::Settings,
                    "Options",
                    "Options",
                    true,
                    self.settings,
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

                    ui.checkbox(&mut self.rulers, "Show horizontal and vertical rulers");

                    ui.checkbox(&mut self.navigation, "Show navigation pane");

                    ui.checkbox(&mut self.ribbon_collapsed, "Collapse ribbon");

                    ui.add_space(18.);

                    ui.label("Document zoom");

                    if ui
                        .add(egui::Slider::new(&mut self.zoom, 50.0..=200.0).suffix("%"))
                        .changed()
                    {
                        self.zoom_width = false;
                    }

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
                    if let Some(notice) = &self.output_notice {
                        ui.label(notice);
                    }

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
                    ui.small(format!(
                        "Saves use format v{}. Earlier builds cannot open new saves.",
                        crate::storage::CONTENT_VERSION
                    ));

                    ui.small("View → Print Layout edits computed pages. Print preview and PDF share that layout.");
                }
            });
        });
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        if self.structure_dialogs(ctx) {
            return;
        }
        if let Some(mut dialog) = self.tab_sources_dialog.take() {
            if dialog.show(ctx, &mut self.regions) {
                dialog.restore_focus(ctx);
                if self.tab_ruler_drag.is_none() && dialog.return_focus.is_none() {
                    self.focus_document = true;
                }
            } else {
                self.tab_sources_dialog = Some(dialog);
            }
            return;
        }
        self.show_style_context_menu(ctx);
        self.quick_reset_dialog(ctx);
        if let Some(mut dialog) = self.quick_order.take() {
            match dialog.show(ctx, &mut self.regions) {
                OrderOutcome::Open => self.quick_order = Some(dialog),
                OrderOutcome::Cancel => self.finish_quick_option(ctx),
                OrderOutcome::Apply(commands) => {
                    self.finish_quick_option(ctx);
                    if self.quick_access != commands {
                        self.quick_access = commands;
                        self.save_quick_preferences();
                    }
                }
            }
        }
        if let Some(mut dialog) = self.zoom_dialog.take() {
            match dialog.show(ctx, &mut self.regions) {
                ZoomOutcome::Open => self.zoom_dialog = Some(dialog),
                ZoomOutcome::Cancel => self.focus_document = true,
                ZoomOutcome::Apply(value) => {
                    self.zoom = value;
                    self.zoom_width = false;
                    self.focus_document = true;
                }
            }
        }
        if let Some(mut dialog) = self.delete_style_dialog.take() {
            match dialog.show(ctx, &self.session.editor.document, &mut self.regions) {
                DeleteStyleOutcome::Open => self.delete_style_dialog = Some(dialog),
                DeleteStyleOutcome::Cancel => self.sync_view(),
                DeleteStyleOutcome::Apply => {
                    let result = if let Some(id) = dialog.character_id {
                        self.session
                            .editor
                            .delete_character_style(id, dialog.character_replacement)
                    } else {
                        self.session.editor.delete_custom_style(
                            dialog.id,
                            dialog.paragraph_replacement,
                            dialog.parent_replacement,
                        )
                    };
                    self.fail(result);
                    self.sync_view();
                }
            }
        }
        if let Some(mut dialog) = self.style_dialog.take() {
            match dialog.show(ctx, &mut self.regions) {
                StyleOutcome::Open => self.style_dialog = Some(dialog),
                StyleOutcome::Cancel => self.sync_view(),
                StyleOutcome::Apply(definition) => {
                    let result = if let Some(id) = dialog.character_id {
                        self.session.editor.update_character_style_with_parent(
                            id,
                            &dialog.name,
                            definition.character,
                            dialog.character_parent,
                        )
                    } else if dialog.character {
                        self.session
                            .editor
                            .create_character_style_with_parent(
                                &dialog.name,
                                definition.character,
                                dialog.character_parent,
                                dialog.apply,
                            )
                            .map(|_| ())
                    } else if dialog.creating {
                        self.session
                            .editor
                            .create_style(&dialog.name, definition, dialog.apply)
                            .map(|_| ())
                    } else if matches!(dialog.id, ParagraphStyle::Custom(_)) {
                        self.session
                            .editor
                            .modify_custom_style(dialog.id, &dialog.name, definition)
                    } else {
                        self.session.editor.modify_style(dialog.id, definition)
                    };
                    self.fail(result);
                    self.sync_view();
                }
            }
        }
        if self.tab_dialog.is_none()
            && self.tab_positions_dialog.is_none()
            && let Some(mut dialog) = self.paragraph_dialog.take()
        {
            match dialog.show(ctx, &mut self.regions) {
                ParagraphOutcome::Open => self.paragraph_dialog = Some(dialog),
                ParagraphOutcome::Cancel => self.sync_view(),
                ParagraphOutcome::Positions => {
                    self.tab_positions_dialog = Some(PositionsDialog::new(&self.session.editor));
                    dialog.restore_focus();
                    self.paragraph_dialog = Some(dialog);
                }
                ParagraphOutcome::Tabs => {
                    self.tab_dialog = Some(TabDialog::new(&self.session.editor));
                    dialog.restore_focus();
                    self.paragraph_dialog = Some(dialog);
                }
                ParagraphOutcome::Apply(patch) => {
                    let (start, end) = self.session.editor.selection;
                    let result = self
                        .session
                        .editor
                        .set_paragraph_geometry(start, end, patch);
                    self.fail(result);
                    self.sync_view();
                }
            }
        }
        if let Some(mut dialog) = self.tab_dialog.take() {
            match dialog.show(ctx, &mut self.regions) {
                TabOutcome::Open => self.tab_dialog = Some(dialog),
                TabOutcome::Cancel => self.focus_document = self.paragraph_dialog.is_none(),
                TabOutcome::Apply(value) => {
                    let (start, end) = self.session.editor.selection;
                    let result = self.session.editor.set_tab_interval(start, end, value);
                    self.fail(result);
                    self.sync_view();
                    self.focus_document = self.paragraph_dialog.is_none();
                }
            }
        }
        if let Some(mut dialog) = self.tab_positions_dialog.take() {
            match dialog.show(ctx, &mut self.regions) {
                PositionsOutcome::Open => self.tab_positions_dialog = Some(dialog),
                PositionsOutcome::Cancel => self.focus_document = self.paragraph_dialog.is_none(),
                PositionsOutcome::Inherit => {
                    let (start, end) = self.session.editor.selection;
                    let result = self.session.editor.inherit_tab_stops(start, end);
                    self.fail(result);
                    self.sync_view();
                    self.focus_document = self.paragraph_dialog.is_none();
                }
                PositionsOutcome::Apply { positions, leaders } => {
                    let (start, end) = self.session.editor.selection;
                    let result = self
                        .session
                        .editor
                        .set_tab_settings(start, end, positions, leaders);
                    self.fail(result);
                    self.sync_view();
                    self.focus_document = self.paragraph_dialog.is_none();
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
        if let Some(mut dialog) = self.replace_all_dialog.take() {
            match dialog.show(ctx, &mut self.regions) {
                ReplaceAllOutcome::Open => self.replace_all_dialog = Some(dialog),
                ReplaceAllOutcome::Cancel => {
                    self.focus_document = false;
                    self.focus_find = true;
                }
                ReplaceAllOutcome::Apply => {
                    let result = self.session.editor.apply_replace_all(dialog.plan);
                    self.sync_view();
                    match result {
                        Ok(count) => {
                            self.replace_all_notice = Some(format!(
                                "Replaced {count} match{}.",
                                if count == 1 { "" } else { "es" }
                            ))
                        }
                        Err(error) => self.error = Some(error),
                    }
                    self.scroll_to_selection = true;
                    self.focus_document = false;
                    self.focus_find = true;
                    self.find_scroll = None;
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
                let ok = ui.button("OK");
                self.record("error-ok", ok.rect);
                if ok.clicked() {
                    self.error = None;
                    self.focus_document = true;
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
                        .chain(
                            self.session
                                .editor
                                .document
                                .style_catalog
                                .custom
                                .iter()
                                .map(|s| Action::Style(ParagraphStyle::Custom(s.id))),
                        )
                        .chain(
                            self.session
                                .editor
                                .document
                                .character_styles
                                .iter()
                                .map(|s| {
                                    Action::CharacterStyle(Some(CharacterStyle::Custom(s.id)))
                                }),
                        )
                        .filter(|a| self.command_name(*a).to_lowercase().contains(&query))
                        .collect();
                    for action in actions.iter().take(12) {
                        if ui
                            .add_enabled(
                                self.enabled(*action),
                                Button::new(self.command_name(*action)).truncate(),
                            )
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
        if ctx.input(|i| i.focused)
            && let Some(id) = ctx.data_mut(|d| {
                let key = Id::new("tab-sources-return-focus");
                let value = d.get_temp::<Id>(key);
                d.remove::<Id>(key);
                value
            })
        {
            if id == Id::new(EDIT_ID) {
                self.focus_document = true;
            } else {
                ctx.memory_mut(|m| m.request_focus(id));
            }
        }
        self.prepare_ruler_input(ctx);
        let mut close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested
            && (self.replace_all_dialog.is_some()
                || self.font_dialog.is_some()
                || self.paragraph_dialog.is_some()
                || self.style_dialog.is_some()
                || self.delete_style_dialog.is_some()
                || self.zoom_dialog.is_some()
                || self.tab_dialog.is_some()
                || self.tab_positions_dialog.is_some()
                || self.tab_sources_dialog.is_some()
                || self.structure_dialog.is_some()
                || self.quick_reset
                || self.quick_order.is_some())
        {
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
        if !egui::Popup::is_any_open(ctx) {
            self.style_return = None;
            self.style_jump = None;
            self.style_navigation_notice = None;
        }
        self.keytip_input(ctx);
        self.prepare_keytip_menu(ctx);
        self.shortcuts(ctx);
        self.regions.clear();
        self.ribbon_targets.clear();
        self.quick_access_targets.clear();
        self.ribbon_menu_targets.clear();
        self.title(ctx);
        if !self.backstage {
            self.tabs(ctx);
        }
        self.ribbon(ctx);
        self.quick_access_panel(ctx);
        self.status(ctx);
        self.navigation_panel(ctx);
        self.formatting_panel(ctx);
        let panel = egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(if self.backstage {
                        chrome::SURFACE
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
        Box::new(move |cc| {
            Ok(Box::new(DocsApp::with_preferences(
                session,
                &cc.egui_ctx,
                crate::preferences::default_path(),
            )))
        }),
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "gui_tests.rs"]
mod tests;
