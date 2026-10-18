//! Sequential access keys and visual hints. Dispatch uses the existing GUI actions.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Level {
    Root,
    Home,
    Tab(usize),
    Tabs,
    File,
    Paste,
    Select,
    Editing,
    Styles,
}
#[derive(Default)]
pub(super) struct KeyTips {
    pub(super) level: Option<Level>,
    pub(super) prefix: String,
    alt_down: bool,
    alt_armed: bool,
    return_focus: Option<Id>,
    consumed: Vec<Key>,
    suppress_text: bool,
}
#[derive(Clone, Copy)]
enum Destination {
    Action(Action),
    Tab(usize),
    Search,
    File,
    Options,
    QuickOptions,
    Tabs,
    Menu(&'static str, Level),
}
struct Tip {
    code: &'static str,
    anchor: String,
    destination: Destination,
}
impl Tip {
    fn action(code: &'static str, action: Action) -> Self {
        Self {
            code,
            anchor: format!("command-{action:?}"),
            destination: Destination::Action(action),
        }
    }
    fn new(code: &'static str, anchor: &str, destination: Destination) -> Self {
        Self {
            code,
            anchor: anchor.into(),
            destination,
        }
    }
}
fn plain_alt(m: Modifiers) -> bool {
    m.alt && !m.ctrl && !m.shift && !m.command && !m.mac_cmd
}

impl DocsApp {
    fn keytip_choices(&self) -> Vec<Tip> {
        use Destination::{File, Menu, Options, Search, Tab};
        let Some(level) = self.keytips.level else {
            return Vec::new();
        };
        let mut tips = match level {
            Level::Root | Level::Tabs => {
                let mut tips = vec![
                    Tip::new("F", "tab-File", File),
                    Tip::new("Q", SEARCH_ID, Search),
                ];
                for (code, action) in ["1", "2", "3", "4", "5"]
                    .into_iter()
                    .zip(self.quick_access.iter().copied())
                {
                    tips.push(Tip::new(
                        code,
                        &format!("qat-{action:?}"),
                        Destination::Action(action),
                    ));
                }
                tips.push(Tip::new("0", "qat-options", Destination::QuickOptions));
                tips.push(Tip::new("T", "more-tabs", Destination::Tabs));
                for (code, index) in [
                    ("H", 0),
                    ("N", 1),
                    ("JI", 2),
                    ("G", 3),
                    ("P", 4),
                    ("S", 5),
                    ("M", 6),
                    ("R", 7),
                    ("W", 8),
                    ("Y1", 9),
                ] {
                    tips.push(Tip::new(code, &format!("tab-{}", TABS[index]), Tab(index)));
                }
                if level == Level::Tabs {
                    tips.retain(|tip| matches!(tip.destination, Tab(_)));
                    for tip in &mut tips {
                        tip.anchor = format!("overflow-{}", tip.anchor);
                    }
                }
                tips
            }
            Level::Tab(8) => vec![
                Tip::action("K", Action::Navigation),
                Tip::action("R", Action::Rulers),
                Tip::action("Q", Action::Zoom),
                Tip::action("J", Action::Zoom100),
                Tip::action("C", Action::ZoomWidth),
            ],
            Level::Tab(_) => Vec::new(),
            Level::Home => vec![
                Tip::new(
                    "V",
                    "home-menu-Paste",
                    Menu("home-menu-Paste", Level::Paste),
                ),
                Tip::action("X", Action::Cut),
                Tip::action("C", Action::Copy),
                Tip::action("1", Action::Bold),
                Tip::action("2", Action::Italic),
                Tip::action("3", Action::Underline),
                // Docs-specific character-reset route.
                Tip::action("FC", Action::ClearFormatting),
                Tip::new("FS", "font-size", Destination::Action(Action::FontSize)),
                Tip::action("FG", Action::GrowFont),
                Tip::action("FK", Action::ShrinkFont),
                Tip::action("FN", Action::FontDialog),
                Tip::action("AL", Action::Left),
                Tip::action("AC", Action::Center),
                Tip::action("AR", Action::Right),
                Tip::action("AJ", Action::Justify),
                Tip::new(
                    "L",
                    "home-menu-Styles",
                    Menu("home-menu-Styles", Level::Styles),
                ),
                Tip::action("FD", Action::Find),
                Tip::action("R", Action::Replace),
                Tip::new(
                    "SL",
                    "home-menu-Select",
                    Menu("home-menu-Select", Level::Select),
                ),
                // A Docs-specific route for the responsive overflow, not a Word mapping claim.
                Tip::new("E", "home-overflow", Menu("home-overflow", Level::Editing)),
            ],
            Level::File => vec![
                Tip::action("N", Action::New),
                Tip::action("O", Action::Open),
                Tip::action("S", Action::Save),
                Tip::action("A", Action::SaveAs),
                Tip::new("T", "file-options", Options),
            ],
            Level::Paste => vec![Tip::action("T", Action::Paste)],
            Level::Select => vec![Tip::action("A", Action::SelectAll)],
            Level::Styles => vec![
                Tip::action("N", Action::Style(ParagraphStyle::Normal)),
                Tip::action("S", Action::Style(ParagraphStyle::NoSpacing)),
                Tip::action("H", Action::Style(ParagraphStyle::Heading1)),
                Tip::action("B", Action::CharacterStyle(Some(CharacterStyle::Strong))),
                Tip::action("I", Action::CharacterStyle(Some(CharacterStyle::Emphasis))),
                Tip::action("Z", Action::CharacterStyle(None)),
                Tip::action("M", Action::ModifyStyle),
                Tip::action("C", Action::CreateStyle),
                Tip::action("K", Action::CreateCharacterStyle),
                Tip::action("J", Action::ModifyCharacterStyle),
                Tip::action("D", Action::DeleteStyle),
            ],
            Level::Editing => vec![
                Tip::action("N", Action::Style(ParagraphStyle::Normal)),
                Tip::action("S", Action::Style(ParagraphStyle::NoSpacing)),
                Tip::action("H", Action::Style(ParagraphStyle::Heading1)),
                Tip::action("B", Action::CharacterStyle(Some(CharacterStyle::Strong))),
                Tip::action("I", Action::CharacterStyle(Some(CharacterStyle::Emphasis))),
                Tip::action("Z", Action::CharacterStyle(None)),
                Tip::action("M", Action::ModifyStyle),
                Tip::action("C", Action::CreateStyle),
                Tip::action("K", Action::CreateCharacterStyle),
                Tip::action("J", Action::ModifyCharacterStyle),
                Tip::action("D", Action::DeleteStyle),
                Tip::action("FD", Action::Find),
                Tip::action("R", Action::Replace),
                Tip::action("A", Action::SelectAll),
                Tip::action("P", Action::Formatting),
            ],
        };
        tips.retain(|tip| match tip.destination {
            Destination::Action(action) => tip.anchor.starts_with("qat-") || self.enabled(action),
            _ => true,
        });
        tips
    }

    fn start_keytips(&mut self, ctx: &egui::Context) {
        self.keytips.return_focus = ctx.memory(|m| m.focused());
        self.keytips.prefix.clear();
        self.keytips.level = Some(if self.backstage {
            Level::File
        } else {
            Level::Root
        });
        if !self.backstage {
            self.focus_ribbon(ctx, false);
        } else {
            self.focus_document = false;
        }
        egui::Popup::close_all(ctx);
    }

    fn stop_keytips(&mut self, ctx: &egui::Context, restore: bool) {
        self.keytips.level = None;
        self.keytips.prefix.clear();
        if restore {
            self.leave_ribbon(ctx);
            if let Some(id) = self.keytips.return_focus {
                self.focus_document = id == Id::new(EDIT_ID);
                ctx.memory_mut(|m| m.request_focus(id));
            }
        }
    }

    fn keytip_back(&mut self, ctx: &egui::Context) {
        if !self.keytips.prefix.is_empty() {
            self.keytips.prefix.clear();
            return;
        }
        match self.keytips.level {
            Some(Level::Paste | Level::Select | Level::Editing | Level::Styles) => {
                egui::Popup::close_all(ctx);
                self.ribbon_menu_index = None;
                self.keytips.level = Some(Level::Home);
            }
            Some(Level::Home | Level::File | Level::Tab(_) | Level::Tabs) => {
                egui::Popup::close_all(ctx);
                self.ribbon_menu_index = None;
                self.backstage = false;
                self.ribbon_temporary = false;
                self.focus_ribbon(ctx, false);
                self.keytips.level = Some(Level::Root);
            }
            _ => self.stop_keytips(ctx, true),
        }
    }

    fn keytip_letter(&mut self, letter: &str, ctx: &egui::Context) {
        let prefix = format!("{}{letter}", self.keytips.prefix);
        let tips = self.keytip_choices();
        if let Some(tip) = tips.iter().find(|tip| tip.code == prefix) {
            self.keytips.prefix.clear();
            match tip.destination {
                Destination::Tab(index) => {
                    self.tab = index;
                    self.focus_ribbon(ctx, true);
                    egui::Popup::close_all(ctx);
                    self.keytips.level = Some(if index == 0 {
                        Level::Home
                    } else {
                        Level::Tab(index)
                    });
                }
                Destination::Tabs => {
                    self.keytips.level = Some(Level::Tabs);
                    self.prepare_keytip_menu(ctx);
                }
                Destination::File => {
                    self.leave_ribbon(ctx);
                    self.focus_document = false;
                    self.backstage = true;
                    self.keytips.level = Some(Level::File);
                }
                Destination::Search => {
                    self.stop_keytips(ctx, false);
                    self.leave_ribbon(ctx);
                    self.focus_document = false;
                    self.focus_search = true;
                }
                Destination::QuickOptions => {
                    self.stop_keytips(ctx, false);
                    self.open_quick_options(ctx);
                }
                Destination::Options => {
                    self.stop_keytips(ctx, false);
                    self.settings = true;
                }
                Destination::Action(action) => {
                    if !self.enabled(action) {
                        return;
                    }
                    self.stop_keytips(ctx, false);
                    self.execute(action, ctx);
                }
                Destination::Menu(name, level) => {
                    self.keytips.level = Some(level);
                    self.open_keytip_menu(name, ctx);
                }
            }
        } else if tips.iter().any(|tip| tip.code.starts_with(&prefix)) {
            self.keytips.prefix = prefix;
        }
        // Invalid keys leave the current prefix intact and never enter the document.
    }

    pub(super) fn keytip_input(&mut self, ctx: &egui::Context) {
        let (modifiers, focused, events) =
            ctx.input(|i| (i.modifiers, i.focused, i.events.clone()));
        let blocked = !focused
            || self.composing
            || self.replace_all_dialog.is_some()
            || self.font_dialog.is_some()
            || self.style_dialog.is_some()
            || self.delete_style_dialog.is_some()
            || self.zoom_dialog.is_some()
            || self.quick_reset
            || self.quick_order.is_some()
            || self.pending.is_some()
            || self.error.is_some()
            || events.iter().any(|e| matches!(e, egui::Event::Ime(_)));
        if blocked {
            if self.keytips.level.is_some() {
                self.stop_keytips(ctx, focused);
            }
            if !focused {
                self.ribbon_temporary = false;
                self.ribbon_focus = None;
                self.ribbon_menu_index = None;
                egui::Popup::close_all(ctx);
            }
            self.keytips.alt_armed = false;
            self.keytips.alt_down = modifiers.alt;
            self.keytips.consumed.clear();
            self.keytips.suppress_text = false;
            return;
        }
        if modifiers.alt && !self.keytips.alt_down {
            self.keytips.alt_armed = plain_alt(modifiers);
        }
        if modifiers.ctrl || modifiers.shift || modifiers.command || modifiers.mac_cmd {
            self.keytips.alt_armed = false;
        }
        let mut keep = Vec::new();
        for event in events {
            match &event {
                egui::Event::Key {
                    key,
                    pressed,
                    repeat,
                    modifiers,
                    ..
                } => {
                    // AltGr is commonly delivered as Ctrl+Alt. egui's logical shortcut
                    // matching allows extra Alt, so do not forward this key as Ctrl+H etc.
                    // The accompanying committed Text event remains normal editor input.
                    if modifiers.alt && modifiers.ctrl && !modifiers.mac_cmd {
                        self.keytips.alt_armed = false;
                        self.keytips.suppress_text = false;
                        if self.keytips.level.is_some() {
                            self.stop_keytips(ctx, true);
                        }
                        continue;
                    }
                    if !pressed {
                        if let Some(index) = self.keytips.consumed.iter().position(|k| k == key) {
                            self.keytips.consumed.remove(index);
                            self.keytips.suppress_text = false;
                            continue;
                        }
                        keep.push(event);
                        continue;
                    }
                    self.keytips.alt_armed = false;
                    self.keytips.suppress_text = false;
                    if self.keytips.consumed.contains(key) {
                        self.keytips.suppress_text = true;
                        continue;
                    }
                    let mut consumed = false;
                    if *key == Key::F10 && modifiers.is_none() {
                        if !repeat {
                            if self.keytips.level.is_some() {
                                self.stop_keytips(ctx, true);
                            } else {
                                self.start_keytips(ctx);
                            }
                        }
                        consumed = true;
                    } else if plain_alt(*modifiers)
                        && self.keytips.level.is_none()
                        && matches!(
                            key,
                            Key::H
                                | Key::F
                                | Key::Q
                                | Key::N
                                | Key::J
                                | Key::G
                                | Key::P
                                | Key::S
                                | Key::M
                                | Key::R
                                | Key::W
                                | Key::Y
                                | Key::T
                                | Key::Num0
                                | Key::Num4
                                | Key::Num5
                                | Key::Num1
                                | Key::Num2
                                | Key::Num3
                        )
                    {
                        if !repeat {
                            if self.keytips.level.is_none() {
                                self.start_keytips(ctx);
                            }
                            self.keytips.level = Some(Level::Root);
                            self.keytips.prefix.clear();
                            self.keytip_letter(key.name(), ctx);
                        }
                        consumed = true;
                    } else if self.keytips.level.is_some() {
                        if matches!(
                            key,
                            Key::Tab
                                | Key::ArrowLeft
                                | Key::ArrowRight
                                | Key::ArrowUp
                                | Key::ArrowDown
                                | Key::Enter
                                | Key::Space
                                | Key::F6
                        ) {
                            self.stop_keytips(ctx, false);
                        } else if modifiers.ctrl || modifiers.command || modifiers.mac_cmd {
                            // Hand modified shortcuts back to the regular router, including AltGr.
                            self.stop_keytips(ctx, true);
                        } else {
                            if !repeat {
                                match key {
                                    Key::Escape => self.keytip_back(ctx),
                                    Key::Backspace => {
                                        self.keytips.prefix.pop();
                                    }
                                    _ => {
                                        let name = key.name().to_ascii_uppercase();
                                        if name.len() == 1
                                            && name.as_bytes()[0].is_ascii_alphanumeric()
                                        {
                                            self.keytip_letter(&name, ctx);
                                        }
                                    }
                                }
                            }
                            consumed = true;
                        }
                    }
                    if consumed {
                        self.keytips.consumed.push(*key);
                        self.keytips.suppress_text = true;
                    } else {
                        keep.push(event);
                    }
                }
                egui::Event::Text(_) | egui::Event::Paste(_) => {
                    self.keytips.alt_armed = false;
                    if !self.keytips.suppress_text && self.keytips.level.is_none() {
                        keep.push(event);
                    }
                }
                egui::Event::PointerButton { pressed: true, .. } => {
                    self.keytips.alt_armed = false;
                    self.stop_keytips(ctx, false);
                    keep.push(event);
                }
                _ => keep.push(event),
            }
        }
        if self.keytips.alt_down && !modifiers.alt && self.keytips.alt_armed {
            if self.keytips.level.is_some() {
                self.stop_keytips(ctx, true);
            } else {
                self.start_keytips(ctx);
            }
            self.keytips.alt_armed = false;
        }
        self.keytips.alt_down = modifiers.alt;
        ctx.input_mut(|i| i.events = keep);
    }

    fn open_keytip_menu(&mut self, name: &str, ctx: &egui::Context) {
        let target = self
            .ribbon_targets
            .iter()
            .find(|t| t.name == name)
            .or_else(|| {
                (name == "home-menu-Select")
                    .then(|| {
                        self.ribbon_targets
                            .iter()
                            .find(|t| t.name == "home-overflow")
                    })
                    .flatten()
            })
            .cloned();
        if let Some(target) = target {
            self.activate_target(&target, ctx);
        }
        ctx.request_repaint();
    }

    pub(super) fn prepare_keytip_menu(&mut self, ctx: &egui::Context) {
        if egui::Popup::is_any_open(ctx) {
            return;
        }
        if self.keytips.level == Some(Level::Tabs) {
            if let Some(id) = self.more_tabs_id {
                egui::Popup::open_id(ctx, id.with("popup"));
            }
            return;
        }
        let name = match self.keytips.level {
            Some(Level::Paste) => "home-menu-Paste",
            Some(Level::Select) => "home-menu-Select",
            Some(Level::Editing) => "home-overflow",
            _ => return,
        };
        self.open_keytip_menu(name, ctx);
    }

    pub(super) fn paint_keytips(&mut self, ctx: &egui::Context) {
        if self.replace_all_dialog.is_some()
            || self.font_dialog.is_some()
            || self.style_dialog.is_some()
            || self.delete_style_dialog.is_some()
            || self.zoom_dialog.is_some()
            || self.quick_reset
            || self.quick_order.is_some()
            || self.pending.is_some()
            || self.error.is_some()
        {
            return;
        }
        let painter =
            ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, Id::new("keytips")));
        for tip in self.keytip_choices() {
            if !tip.code.starts_with(&self.keytips.prefix) {
                continue;
            }
            let Some(anchor) = self
                .regions
                .iter()
                .rev()
                .find(|r| r.name == tip.anchor)
                .map(|r| r.rect)
            else {
                continue;
            };
            let menu = matches!(
                self.keytips.level,
                Some(
                    Level::Paste
                        | Level::Select
                        | Level::Editing
                        | Level::Styles
                        | Level::File
                        | Level::Tabs
                )
            );
            let center = if menu {
                pos2(anchor.right() - 16., anchor.center().y)
            } else {
                anchor.center_bottom() + vec2(0., 2.)
            };
            let size = vec2(10. + 7. * tip.code.len() as f32, 17.);
            let rect = Rect::from_center_size(center, size);
            let enabled = match tip.destination {
                Destination::Action(action) => self.enabled(action),
                _ => true,
            };
            painter.rect_filled(
                rect,
                2.,
                if enabled {
                    Color32::from_rgb(55, 55, 55)
                } else {
                    Color32::from_gray(224)
                },
            );
            painter.text(
                center,
                Align2::CENTER_CENTER,
                tip.code,
                FontId::proportional(12.),
                if enabled {
                    Color32::WHITE
                } else {
                    Color32::from_gray(125)
                },
            );
            self.record(&format!("keytip-{}", tip.code), rect);
        }
    }
}
