//! Focus and transient ribbon presentation. This module never owns document text.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum RibbonFocus {
    Tab,
    QuickAccess(String),
    First,
    Control(String),
}

#[derive(Clone, Copy)]
pub(super) enum TargetKind {
    Action(Action),
    Menu,
    Text,
    Preset(FontSize),
    QuickToggle(Action),
    QuickPosition,
    QuickReset,
    QuickRetry,
    QuickOrder,
}

#[derive(Clone)]
pub(super) struct RibbonTarget {
    pub(super) name: String,
    pub(super) id: Id,
    pub(super) rect: Rect,
    pub(super) group: usize,
    pub(super) kind: TargetKind,
}

impl DocsApp {
    pub(super) fn track_ribbon(&mut self, response: &egui::Response, name: &str, kind: TargetKind) {
        if !response.enabled() || self.ribbon_group.is_none() {
            return;
        }
        let target = RibbonTarget {
            name: name.into(),
            id: response.id,
            rect: response.rect,
            group: self.ribbon_group.unwrap(),
            kind,
        };
        if self.collecting_menu {
            if self.ribbon_menu_index == Some(self.ribbon_menu_targets.len()) {
                response.scroll_to_me(Some(Align::Center));
            }
            self.ribbon_menu_targets.push(target);
        } else {
            self.ribbon_targets.push(target);
            if response.clicked() {
                self.ribbon_focus = Some(RibbonFocus::Control(name.into()));
                self.focus_document = false;
            }
        }
    }

    pub(super) fn leave_ribbon(&mut self, ctx: &egui::Context) {
        self.commit_font_size_input();
        self.ribbon_temporary = false;
        self.ribbon_focus = None;
        self.ribbon_menu_index = None;
        egui::Popup::close_all(ctx);
        self.focus_document = true;
    }

    pub(super) fn focus_ribbon(&mut self, ctx: &egui::Context, enter: bool) {
        ctx.memory_mut(|m| {
            if let Some(id) = m.focused() {
                m.surrender_focus(id);
            }
        });
        self.backstage = false;
        self.focus_document = false;
        self.focus_search = false;
        self.focus_find = false;
        self.ribbon_focus = Some(if enter {
            RibbonFocus::First
        } else {
            RibbonFocus::Tab
        });
        if enter {
            self.ribbon_temporary = self.ribbon_collapsed;
        }
    }

    fn select_target(&mut self, target: &RibbonTarget, ctx: &egui::Context) {
        if matches!(target.kind, TargetKind::Text)
            && self.ribbon_focus != Some(RibbonFocus::Control(target.name.clone()))
        {
            self.focus_font_size = true;
        }
        self.ribbon_focus = Some(RibbonFocus::Control(target.name.clone()));
        ctx.memory_mut(|m| m.request_focus(target.id));
    }

    pub(super) fn activate_target(&mut self, target: &RibbonTarget, ctx: &egui::Context) {
        match target.kind {
            TargetKind::Action(action) => self.execute(action, ctx),
            TargetKind::Preset(size) => {
                self.apply_font_size(size);
                self.leave_ribbon(ctx);
            }
            TargetKind::Menu => {
                egui::Popup::open_id(ctx, target.id.with("popup"));
                self.ribbon_menu_index = Some(0);
            }
            TargetKind::Text => self.execute(Action::FontSize, ctx),
            TargetKind::QuickToggle(action) => {
                if self.quick_access.contains(&action) {
                    self.quick_access.retain(|a| *a != action);
                } else {
                    self.quick_access.push(action);
                }
                self.finish_quick_option(ctx);
                self.save_quick_preferences();
            }
            TargetKind::QuickPosition => {
                self.quick_access_below = !self.quick_access_below;
                self.finish_quick_option(ctx);
                self.save_quick_preferences();
            }
            TargetKind::QuickReset => {
                self.finish_quick_option(ctx);
                self.quick_reset = true;
            }
            TargetKind::QuickOrder => {
                self.finish_quick_option(ctx);
                self.focus_font_size = false;
                self.focus_search = false;
                self.focus_find = false;
                ctx.memory_mut(|m| {
                    if let Some(id) = m.focused() {
                        m.surrender_focus(id);
                    }
                });
                self.quick_order = Some(QuickOrderDialog::new(&self.quick_access));
            }
            TargetKind::QuickRetry => {
                self.finish_quick_option(ctx);
                self.save_quick_preferences();
            }
        }
    }

    pub(super) fn ribbon_keys(&mut self, ctx: &egui::Context) {
        if self.replace_all_dialog.is_some()
            || self.pending.is_some()
            || self.error.is_some()
            || self.zoom_dialog.is_some()
            || self.quick_reset
            || self.quick_order.is_some()
        {
            return;
        }
        // A popup owns ribbon navigation even while its first frame still reports
        // the underlying document focus (notably after resizing a temporary ribbon).
        if !self.focus_font_size
            && !egui::Popup::is_any_open(ctx)
            && ctx.memory(|m| m.has_focus(Id::new(EDIT_ID)))
        {
            self.ribbon_focus = None;
        }
        // Dismiss before drawing the canvas so the outside click reaches its target.
        if let Some(pos) = ctx.input(|i| {
            i.pointer
                .any_pressed()
                .then(|| i.pointer.interact_pos())
                .flatten()
        }) {
            let inside = self.regions.iter().any(|r| {
                (matches!(r.name.as_str(), "ribbon" | "tabs" | "quick-access-menu")
                    || r.name.starts_with("qat-"))
                    && r.rect.contains(pos)
            }) || self
                .ribbon_menu_targets
                .iter()
                .any(|r| r.rect.contains(pos));
            if !inside {
                if self.ribbon_temporary {
                    self.commit_font_size_input();
                    self.ribbon_temporary = false;
                }
                self.ribbon_focus = None;
                self.ribbon_menu_index = None;
                self.focus_document = false;
            }
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::F1)) {
            self.execute(Action::Ribbon, ctx);
            self.focus_document = true;
        }
        // Do not take focus away from a live IME preedit.
        if self.composing {
            return;
        }
        let reverse = ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::F6));
        if reverse || ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F6)) {
            let mut zones = vec![0];
            if self.navigation {
                zones.push(1);
            }
            if self.formatting {
                zones.push(4);
            }
            zones.extend([2, 3]);
            self.focus_zone = if self.ribbon_focus.is_some() {
                3
            } else if ctx.memory(|m| m.has_focus(Id::new(EDIT_ID))) {
                0
            } else if ctx.memory(|m| m.has_focus(Id::new("find-text"))) {
                1
            } else {
                ctx.memory(|m| m.focused())
                    .and_then(|id| ctx.read_response(id))
                    .and_then(|response| {
                        self.regions.iter().find_map(|region| {
                            let zone = match region.name.as_str() {
                                "navigation" => 1,
                                "formatting" => 4,
                                "status" => 2,
                                _ => return None,
                            };
                            region.rect.contains(response.rect.center()).then_some(zone)
                        })
                    })
                    .unwrap_or(self.focus_zone)
            };
            let current = zones
                .iter()
                .position(|z| *z == self.focus_zone)
                .unwrap_or(0);
            self.focus_zone =
                zones[(current + if reverse { zones.len() - 1 } else { 1 }) % zones.len()];
            self.leave_ribbon(ctx);
            self.focus_document = false;
            match self.focus_zone {
                0 => self.focus_document = true,
                1 => self.focus_find = true,
                2 => {
                    if let Some(id) = self.status_focus {
                        ctx.memory_mut(|m| m.request_focus(id));
                    }
                }
                4 => {
                    if let Some(id) = self.formatting_focus {
                        ctx.memory_mut(|m| m.request_focus(id));
                    }
                }
                _ => self.focus_ribbon(ctx, false),
            }
        }
        let Some(focus) = self.ribbon_focus.clone() else {
            return;
        };
        if egui::Popup::is_any_open(ctx) {
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
                egui::Popup::close_all(ctx);
                self.ribbon_menu_index = None;
                return;
            }
            if self.ribbon_menu_index.is_some() && !self.ribbon_menu_targets.is_empty() {
                let count = self.ribbon_menu_targets.len();
                let mut index = self.ribbon_menu_index.unwrap().min(count - 1);
                if ctx.input_mut(|i| {
                    i.consume_key(Modifiers::SHIFT, Key::Tab)
                        || i.consume_key(Modifiers::NONE, Key::ArrowUp)
                }) {
                    index = (index + count - 1) % count;
                }
                if ctx.input_mut(|i| {
                    i.consume_key(Modifiers::NONE, Key::Tab)
                        || i.consume_key(Modifiers::NONE, Key::ArrowDown)
                }) {
                    index = (index + 1) % count;
                }
                self.ribbon_menu_index = Some(index);
                if ctx.input_mut(|i| {
                    i.consume_key(Modifiers::NONE, Key::Enter)
                        || i.consume_key(Modifiers::NONE, Key::Space)
                }) {
                    let target = self.ribbon_menu_targets[index].clone();
                    self.activate_target(&target, ctx);
                }
            }
            return;
        }
        self.ribbon_menu_index = None;
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            if focus == RibbonFocus::Control("font-size".into()) {
                self.font_size_dirty = false;
                self.leave_ribbon(ctx);
            } else if focus == RibbonFocus::Tab {
                self.leave_ribbon(ctx);
            } else {
                self.ribbon_temporary = false;
                self.ribbon_focus = Some(RibbonFocus::Tab);
            }
            return;
        }
        if let RibbonFocus::QuickAccess(name) = &focus {
            let targets = self.quick_access_targets.clone();
            let index = targets.iter().position(|t| t.name == *name).unwrap_or(0);
            for (key, forward) in [(Key::ArrowLeft, false), (Key::ArrowRight, true)] {
                if !targets.is_empty() && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, key)) {
                    let next =
                        (index + if forward { 1 } else { targets.len() - 1 }) % targets.len();
                    self.ribbon_focus = Some(RibbonFocus::QuickAccess(targets[next].name.clone()));
                    return;
                }
            }
            if ctx.input_mut(|i| {
                i.consume_key(Modifiers::NONE, Key::Tab)
                    || i.consume_key(Modifiers::SHIFT, Key::Tab)
            }) {
                self.ribbon_focus = Some(RibbonFocus::Tab);
            } else if ctx.input_mut(|i| {
                i.consume_key(Modifiers::NONE, Key::ArrowDown)
                    || i.consume_key(Modifiers::ALT, Key::ArrowDown)
            }) {
                self.open_quick_options(ctx);
            } else if ctx.input_mut(|i| {
                i.consume_key(Modifiers::NONE, Key::Enter)
                    || i.consume_key(Modifiers::NONE, Key::Space)
            }) && let Some(target) = targets.get(index)
            {
                self.activate_target(target, ctx);
            }
            return;
        }
        if focus == RibbonFocus::Tab {
            if ctx.input_mut(|i| {
                i.consume_key(Modifiers::NONE, Key::ArrowUp)
                    || (self.quick_access_below && i.consume_key(Modifiers::NONE, Key::ArrowDown))
            }) {
                if let Some(target) = self.quick_access_targets.first() {
                    self.ribbon_focus = Some(RibbonFocus::QuickAccess(target.name.clone()));
                }
                return;
            }
            for (key, forward) in [(Key::ArrowLeft, false), (Key::ArrowRight, true)] {
                if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, key)) {
                    self.tab = (self.tab + if forward { 1 } else { TABS.len() - 1 }) % TABS.len();
                }
            }
            if ctx.input_mut(|i| {
                i.consume_key(Modifiers::NONE, Key::Tab)
                    || i.consume_key(Modifiers::NONE, Key::ArrowDown)
                    || i.consume_key(Modifiers::NONE, Key::Enter)
                    || i.consume_key(Modifiers::NONE, Key::Space)
            }) {
                self.focus_ribbon(ctx, true);
            }
            if ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::Tab)) {
                self.leave_ribbon(ctx);
            }
            return;
        }
        let RibbonFocus::Control(name) = focus else {
            return;
        };
        let Some(index) = self.ribbon_targets.iter().position(|r| r.name == name) else {
            return;
        };
        let current = self.ribbon_targets[index].clone();
        let reverse = ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::Tab));
        if reverse || ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Tab)) {
            if reverse && index == 0 || !reverse && index + 1 == self.ribbon_targets.len() {
                self.ribbon_focus = Some(RibbonFocus::Tab);
            } else {
                let next = self.ribbon_targets[if reverse { index - 1 } else { index + 1 }].clone();
                self.select_target(&next, ctx);
            }
            return;
        }
        for (key, forward) in [(Key::ArrowLeft, false), (Key::ArrowRight, true)] {
            if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, key)) {
                let group = (1..=5)
                    .map(|step| (current.group + if forward { step } else { 5 - step }) % 5)
                    .find(|group| self.ribbon_targets.iter().any(|r| r.group == *group));
                if let Some(next) = group
                    .and_then(|g| self.ribbon_targets.iter().find(|r| r.group == g))
                    .cloned()
                {
                    self.select_target(&next, ctx);
                }
                return;
            }
        }
        if matches!(current.kind, TargetKind::Menu | TargetKind::Text)
            && ctx.input_mut(|i| {
                i.consume_key(Modifiers::ALT, Key::ArrowDown)
                    || i.consume_key(Modifiers::NONE, Key::ArrowDown)
            })
        {
            let target = if matches!(current.kind, TargetKind::Text) {
                self.ribbon_targets
                    .iter()
                    .find(|r| r.name == "font-size-menu")
                    .cloned()
            } else {
                Some(current.clone())
            };
            if let Some(target) = target {
                self.select_target(&target, ctx);
                self.activate_target(&target, ctx);
            }
            return;
        }
        // Text fields retain native cursor movement, selection and Enter/IME handling.
        if matches!(current.kind, TargetKind::Text) {
            return;
        }
        if ctx.input_mut(|i| {
            i.consume_key(Modifiers::NONE, Key::Enter) || i.consume_key(Modifiers::NONE, Key::Space)
        }) {
            self.activate_target(&current, ctx);
            return;
        }
        for (key, direction) in [
            (Key::ArrowLeft, vec2(-1., 0.)),
            (Key::ArrowRight, vec2(1., 0.)),
            (Key::ArrowUp, vec2(0., -1.)),
            (Key::ArrowDown, vec2(0., 1.)),
        ] {
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, key)) {
                let next = self
                    .ribbon_targets
                    .iter()
                    .filter(|r| r.name != name && r.group == current.group)
                    .filter_map(|r| {
                        let delta = r.rect.center() - current.rect.center();
                        let along = delta.dot(direction);
                        (along > 1.).then_some((
                            r,
                            along + (delta.x * direction.y - delta.y * direction.x).abs() * 3.,
                        ))
                    })
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(r, _)| r.clone());
                if let Some(next) = next {
                    self.select_target(&next, ctx);
                }
                return;
            }
        }
    }

    pub(super) fn finish_ribbon_focus(&mut self, ctx: &egui::Context) {
        if self.backstage
            || self.replace_all_dialog.is_some()
            || self.font_dialog.is_some()
            || self.zoom_dialog.is_some()
            || self.quick_reset
            || self.quick_order.is_some()
            || self.pending.is_some()
            || self.error.is_some()
        {
            return;
        }
        if let Some(index) = self.ribbon_menu_index
            && let Some(target) = self.ribbon_menu_targets.get(index)
        {
            ctx.memory_mut(|m| m.request_focus(target.id));
            return;
        }
        // A resize may move the focused Editing control into More. Keep focus visible.
        let target = match &self.ribbon_focus {
            Some(RibbonFocus::QuickAccess(name)) => {
                if let Some(target) = self.quick_access_targets.iter().find(|t| t.name == *name) {
                    ctx.memory_mut(|m| m.request_focus(target.id));
                }
                return;
            }
            Some(RibbonFocus::First) => self.ribbon_targets.first(),
            Some(RibbonFocus::Control(name)) => self
                .ribbon_targets
                .iter()
                .find(|r| r.name == *name)
                .or_else(|| {
                    self.ribbon_targets
                        .iter()
                        .find(|r| r.name == "home-overflow")
                }),
            _ => None,
        }
        .cloned();
        if let Some(target) = target {
            self.select_target(&target, ctx);
        } else if !matches!(self.tab, 0 | 8) && self.ribbon_focus == Some(RibbonFocus::First) {
            self.ribbon_focus = Some(RibbonFocus::Tab);
        }
    }
}
