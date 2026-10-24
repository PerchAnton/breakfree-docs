//! Pointer and keyboard adapter for explicit tab stops; existing indent controls keep priority.
use super::*;

pub(super) struct TabDrag {
    pub draft: crate::tab_ruler::Draft,
    rect: Rect,
    screen: Rect,
    zoom: f32,
    origin: f32,
    start: u16,
    unit: f32,
    error: Option<String>,
    keyboard: bool,
    show_leaders: bool,
    numeric: bool,
    input: String,
    select_input: bool,
    accept_input: bool,
    popup: Rect,
    control_focus: Option<usize>,
    activate_control: bool,
}
impl TabDrag {
    fn focus_numeric(&mut self, ctx: &egui::Context) {
        // Re-entering the existing field must retain an incomplete draft and caret.
        if !self.numeric {
            self.input = format!("{}", f32::from(self.draft.position) / 2.);
            self.select_input = true;
        }
        self.numeric = true;
        self.control_focus = None;
        ctx.memory_mut(|m| m.request_focus(Id::new("ruler-tab-position-input")));
    }

    fn focus_marker(&mut self, ctx: &egui::Context) {
        self.numeric = false;
        self.select_input = false;
        self.control_focus = None;
        // A button can own native focus even when the position field is visible.
        ctx.memory_mut(|m| {
            if let Some(id) = m.focused() {
                m.surrender_focus(id);
            }
        });
    }

    // Leader choices 0..4, restore 4, position 5, apply 6, cancel 7.
    fn enabled_controls(&self) -> Vec<usize> {
        let valid = self.error.is_none() && !self.draft.deleting;
        let mut controls = Vec::new();
        if valid {
            controls.extend(0..4);
        }
        if valid && self.draft.preview_leader_choice().is_some() {
            controls.push(4);
        }
        if !self.numeric || self.error.is_none() {
            controls.push(5);
        }
        if self.error.is_none() {
            controls.push(6);
        }
        controls.push(7);
        controls
    }

    pub(super) fn control_help(&self, index: usize) -> String {
        let action = match index {
            0 => "None: preview this stop without a fill.",
            1 => "Dots: preview a dotted fill for this stop.",
            2 => "Dashes: preview a dashed fill for this stop.",
            3 => "Line: preview a continuous fill for this stop.",
            4 => "Restore leaders: recover each original leader, keeping the draft position.",
            5 if self.numeric => {
                "Back to marker: keep the valid position and return without applying."
            }
            5 => "Exact position: edit the position in points without applying.",
            6 if self.draft.deleting => {
                "Apply: delete this stop from its selected owners in one Undo/Redo."
            }
            6 => "Apply: confirm the draft position and leaders in one Undo/Redo.",
            _ => "Cancel: discard the entire preview; document and history stay unchanged.",
        };
        let scope = if index < 4 {
            if self.draft.is_creation() {
                " Existing stops keep their leaders during creation."
            } else {
                " Only selected paragraphs owning this stop change."
            }
        } else {
            ""
        };
        let shortcut = match index {
            0..=3 => format!("Marker mode: {index}. Numeric mode: digits edit the position."),
            4 => "Marker mode: Backspace.".into(),
            5 if self.numeric => "F4: return after valid input.".into(),
            5 => "F2: exact position.".into(),
            6 => "Marker/input mode: Enter applies.".into(),
            _ => "Escape: cancel from any preview mode.".into(),
        };
        format!(
            "{action}{scope}\n{shortcut} Controls: Enter/Space activates; F6 returns to editing."
        )
    }

    pub(super) fn control_disabled_reason(&self, index: usize) -> Option<String> {
        if self.enabled_controls().contains(&index) {
            return None;
        }
        Some(if let Some(error) = &self.error {
            format!(
                "Unavailable: {error} Return to editing with F6 to correct the position, or cancel."
            )
        } else if self.draft.deleting {
            "Leader edits are unavailable during deletion. Return to marker mode to adjust, or cancel.".into()
        } else {
            "Original per-paragraph leaders are already retained; there is no leader override to restore.".into()
        })
    }

    fn control(&mut self, ui: &Ui, index: usize, response: &egui::Response) -> bool {
        let help = self.control_help(index);
        let reason = self.control_disabled_reason(index);
        response
            .clone()
            .on_hover_text(&help)
            .on_disabled_hover_text(reason.as_deref().unwrap_or(&help));
        let label = reason
            .as_ref()
            .map_or_else(|| help.clone(), |reason| format!("{help}\n{reason}"));
        response.widget_info(|| {
            if index < 4 {
                let leader = [
                    None,
                    Some(crate::model::TabLeader::Dots),
                    Some(crate::model::TabLeader::Dashes),
                    Some(crate::model::TabLeader::Line),
                ][index];
                let common = crate::tab_ruler::leader_summary(
                    &self.draft.preview,
                    self.draft.selection(),
                    self.draft.position,
                )
                .ok()
                .and_then(|s| s.common());
                egui::WidgetInfo::selected(
                    egui::WidgetType::Button,
                    response.enabled(),
                    response.enabled()
                        && self.draft.preview_leader_choice().or(common) == Some(leader),
                    &label,
                )
            } else {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, response.enabled(), &label)
            }
        });
        if !response.enabled() {
            return false;
        }
        if response.clicked() {
            self.control_focus = Some(index);
        }
        let focused = self.control_focus == Some(index);
        if focused {
            response.request_focus();
            ui.painter().rect_stroke(
                response.rect.expand(1.),
                3.,
                Stroke::new(1.5_f32, ACCENT),
                egui::StrokeKind::Inside,
            );
        }
        let activate = focused && self.activate_control;
        if activate {
            self.activate_control = false;
        }
        response.clicked() || activate
    }
}
impl DocsApp {
    fn cycle_tab_creation_kind(&mut self) {
        use crate::model::TabKind;
        self.tab_creation_kind = match self.tab_creation_kind {
            TabKind::Left => TabKind::Center,
            TabKind::Center => TabKind::Right,
            TabKind::Right => TabKind::Left,
        };
    }
    pub(super) fn tab_type_selector(&mut self, ui: &mut Ui, ruler: Rect) {
        let rect = Rect::from_min_size(pos2(ruler.left() - 28., ruler.top()), vec2(24., 24.));
        let enabled = self.enabled(Action::TabPositions)
            && !self.composing
            && !self.backstage
            && self.ruler_key_focus.is_none();
        let response = ui.interact(
            rect,
            Id::new("tab-type-selector"),
            if enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        let visuals = ui.style().interact(&response);
        ui.painter().rect(
            rect,
            visuals.corner_radius,
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
        self.record("tab-type-selector", response.rect);
        if enabled && response.clicked() {
            self.cycle_tab_creation_kind();
        }
        let kind = self.tab_creation_kind;
        let label = format!(
            "New tab type: {}. Click or Ctrl+Alt+F7 cycles Left, Center, Right. Right-click to choose. Applies to new tabs only.",
            kind.label()
        );
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, &label));
        let x = rect.center().x;
        let y = rect.center().y;
        let (low, high) = match kind {
            crate::model::TabKind::Left => (0., 5.),
            crate::model::TabKind::Center => (-5., 5.),
            crate::model::TabKind::Right => (-5., 0.),
        };
        let color = if enabled {
            INK
        } else {
            Color32::from_gray(150)
        };
        ui.painter().line_segment(
            [pos2(x, y - 4.), pos2(x, y + 4.)],
            Stroke::new(2.0_f32, color),
        );
        ui.painter().line_segment(
            [pos2(x + low, y + 4.), pos2(x + high, y + 4.)],
            Stroke::new(2.0_f32, color),
        );
        if enabled {
            response.context_menu(|ui| {
                for kind in [
                    crate::model::TabKind::Left,
                    crate::model::TabKind::Center,
                    crate::model::TabKind::Right,
                ] {
                    let r = ui.selectable_label(self.tab_creation_kind == kind, kind.label());
                    self.record(
                        &format!("tab-create-kind-{}", kind.label().to_lowercase()),
                        r.rect,
                    );
                    if r.clicked() {
                        self.tab_creation_kind = kind;
                        ui.close();
                    }
                }
            });
        }
        response.on_hover_text(label);
    }
    pub(super) fn start_tab_pointer_creation(
        &mut self,
        ctx: &egui::Context,
        rect: Rect,
        pointer: egui::Pos2,
        zoom: f32,
        value: u16,
    ) {
        match crate::tab_ruler::Draft::creation_preview_typed(
            &self.session.editor,
            self.tab_creation_kind,
        ) {
            Ok(mut draft) => {
                let error = draft.move_to(f32::from(value)).err();
                // A complete click may arrive within one frame; it still commits exactly once.
                if ctx.input(|i| i.pointer.primary_released()) {
                    if let Some(error) = error {
                        self.error = Some(error);
                    } else {
                        let result = draft.commit(&mut self.session.editor);
                        self.fail(result);
                    }
                    self.sync_view();
                    return;
                }
                self.tab_ruler_drag = Some(TabDrag {
                    draft,
                    rect,
                    screen: ctx.content_rect(),
                    zoom,
                    origin: pointer.x,
                    start: value,
                    unit: 14.6667 * zoom / 22.,
                    error,
                    keyboard: false,
                    show_leaders: false,
                    numeric: false,
                    input: String::new(),
                    select_input: false,
                    accept_input: false,
                    popup: Rect::NOTHING,
                    control_focus: None,
                    activate_control: false,
                });
                self.leave_ribbon(ctx);
                self.focus_document = false;
                ctx.memory_mut(|m| m.surrender_focus(Id::new(EDIT_ID)));
                ctx.request_repaint();
            }
            Err(error) => self.error = Some(error),
        }
    }
    pub(super) fn prepare_tab_ruler_input(&mut self, ctx: &egui::Context) -> bool {
        if self.tab_sources_dialog.is_some() {
            return true;
        }
        if self.tab_ruler_drag.as_ref().is_some_and(|d| d.keyboard)
            && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F9))
        {
            let stop = self
                .tab_ruler_drag
                .as_ref()
                .filter(|d| !d.draft.is_creation())
                .map(|d| d.start);
            self.open_tab_sources(ctx, stop);
            return true;
        }
        let eligible = self.tab_ruler_drag.is_none()
            && self.enabled(Action::TabPositions)
            && !self.composing
            && !self.backstage
            && self.ruler_key_focus.is_none()
            && !egui::Popup::is_any_open(ctx);
        if eligible && ctx.input_mut(|i| i.consume_key(Modifiers::CTRL | Modifiers::ALT, Key::F9)) {
            self.open_tab_sources(ctx, None);
            return true;
        }
        if eligible && ctx.input_mut(|i| i.consume_key(Modifiers::CTRL | Modifiers::ALT, Key::F7)) {
            self.rulers = true;
            self.cycle_tab_creation_kind();
            return true;
        }
        let create = eligible
            && ctx.input_mut(|i| {
                i.consume_key(Modifiers::CTRL | Modifiers::ALT | Modifiers::SHIFT, Key::F8)
            });
        let enter = create
            || (eligible
                && ctx.input_mut(|i| i.consume_key(Modifiers::CTRL | Modifiers::ALT, Key::F8)));
        if enter {
            if !self.commit_font_size_input() {
                return true;
            }
            self.rulers = true;
            if create {
                self.start_tab_keyboard_mode(ctx, 72, true);
            } else if let Some((value, _)) = crate::tab_ruler::positions(
                &self.session.editor.document,
                self.session.editor.selection,
            )
            .unwrap_or_default()
            .first()
            .copied()
            {
                self.start_tab_keyboard(ctx, value);
            } else {
                self.start_tab_keyboard_mode(ctx, 72, true);
            }
            return true;
        }
        if self.tab_ruler_drag.as_ref().is_some_and(|d| d.keyboard) {
            let events = ctx.input(|i| i.events.clone());
            let lost_window = events
                .iter()
                .any(|e| matches!(e, egui::Event::WindowFocused(false)));
            let transform_changed = self
                .tab_ruler_drag
                .as_ref()
                .is_some_and(|d| d.zoom != self.zoom / 100. || d.screen != ctx.content_rect());
            let cancel = transform_changed
                || lost_window
                || !self.rulers
                || self.backstage
                || ctx.input(|i| {
                    !i.focused
                        || i.viewport().close_requested()
                        || (i.pointer.any_pressed()
                            && !i.pointer.interact_pos().is_some_and(|p| {
                                self.tab_ruler_drag.as_ref().unwrap().popup.contains(p)
                            }))
                        || i.key_pressed(Key::Escape)
                });
            if cancel {
                self.tab_ruler_drag = None;
                ctx.input_mut(|i| {
                    // egui already derived pointer state from the raw events. Consume
                    // the dismissing press too, so ribbon routing cannot steal focus
                    // and its later release cannot activate a background command.
                    if i.pointer.any_pressed() {
                        i.pointer = Default::default();
                    }
                    i.events.retain(|e| {
                        !matches!(
                            e,
                            egui::Event::Key { .. }
                                | egui::Event::Text(_)
                                | egui::Event::Paste(_)
                                | egui::Event::Ime(_)
                                | egui::Event::PointerButton { .. }
                        )
                    });
                });
                self.sync_view();
                return true;
            }
            // Consume repeats too, but change visibility only on a fresh press.
            if events.iter().any(|e| {
                matches!(e, egui::Event::Key {
                key: Key::F7, pressed: true, repeat: false, modifiers, ..
            } if modifiers.is_none())
            }) {
                self.tab_preview_help = !self.tab_preview_help;
            }
            ctx.input_mut(|i| {
                i.consume_key(Modifiers::NONE, Key::F7);
            });
            let fresh_mode_key = |key| {
                events.iter().any(|e| {
                    matches!(e,
                egui::Event::Key { key: k, pressed: true, repeat: false, modifiers, .. }
                if *k == key && modifiers.is_none())
                })
            };
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F6)) && fresh_mode_key(Key::F6)
            {
                let drag = self.tab_ruler_drag.as_mut().unwrap();
                drag.control_focus = if drag.control_focus.is_some() {
                    None
                } else {
                    drag.enabled_controls().first().copied()
                };
                ctx.memory_mut(|m| {
                    if let Some(id) = m.focused() {
                        m.surrender_focus(id);
                    }
                });
            }
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F1)) {
                let drag = self.tab_ruler_drag.as_mut().unwrap();
                drag.show_leaders = !drag.show_leaders;
            }
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F2)) {
                let drag = self.tab_ruler_drag.as_mut().unwrap();
                drag.focus_numeric(ctx);
            }
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F4)) {
                let drag = self.tab_ruler_drag.as_mut().unwrap();
                if drag.numeric {
                    drag.error = drag.draft.move_to_text(&drag.input).err();
                    if drag.error.is_none() {
                        drag.focus_marker(ctx);
                    }
                }
            }
            if self
                .tab_ruler_drag
                .as_ref()
                .unwrap()
                .control_focus
                .is_some()
            {
                let drag = self.tab_ruler_drag.as_mut().unwrap();
                let enabled = drag.enabled_controls();
                if !enabled.contains(&drag.control_focus.unwrap()) {
                    drag.control_focus = Some(7);
                }
                for event in &events {
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        repeat,
                        modifiers,
                        ..
                    } = event
                    {
                        if modifiers.ctrl || modifiers.alt || modifiers.command {
                            continue;
                        }
                        match key {
                            Key::Tab => {
                                let at = enabled
                                    .iter()
                                    .position(|c| Some(*c) == drag.control_focus)
                                    .unwrap();
                                let next = if modifiers.shift {
                                    (at + enabled.len() - 1) % enabled.len()
                                } else {
                                    (at + 1) % enabled.len()
                                };
                                drag.control_focus = Some(enabled[next]);
                            }
                            Key::Enter | Key::Space if !repeat => drag.activate_control = true,
                            _ => {}
                        }
                    }
                }
                ctx.input_mut(|i| {
                    i.events.retain(|e| {
                        !matches!(
                            e,
                            egui::Event::Key { .. }
                                | egui::Event::Text(_)
                                | egui::Event::Paste(_)
                                | egui::Event::Ime(_)
                                | egui::Event::MouseWheel { .. }
                        )
                    });
                    i.raw_scroll_delta = Vec2::ZERO;
                    i.smooth_scroll_delta = Vec2::ZERO;
                });
                ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
                return true;
            }
            if self.tab_ruler_drag.as_ref().unwrap().numeric {
                ctx.memory_mut(|m| {
                    m.move_focus(egui::FocusDirection::None);
                    m.request_focus(Id::new("ruler-tab-position-input"));
                });
                let accept = ctx.input_mut(|i| {
                    i.raw_scroll_delta = Vec2::ZERO;
                    i.smooth_scroll_delta = Vec2::ZERO;
                    i.events
                        .retain(|e| !matches!(e, egui::Event::MouseWheel { .. }));
                    let enter = i.consume_key(Modifiers::NONE, Key::Enter);
                    let tab = i.consume_key(Modifiers::NONE, Key::Tab);
                    enter || tab
                });
                self.tab_ruler_drag.as_mut().unwrap().accept_input = accept;
                return true;
            }
            ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
            ctx.input_mut(|i| {
                i.events.retain(|e| {
                    !matches!(
                        e,
                        egui::Event::Key { .. }
                            | egui::Event::Text(_)
                            | egui::Event::Paste(_)
                            | egui::Event::Ime(_)
                            | egui::Event::MouseWheel { .. }
                    )
                });
                i.raw_scroll_delta = Vec2::ZERO;
                i.smooth_scroll_delta = Vec2::ZERO;
            });
            for event in events {
                let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                else {
                    continue;
                };
                if modifiers.ctrl || modifiers.alt || modifiers.command {
                    continue;
                }
                match key {
                    Key::ArrowLeft | Key::ArrowRight => {
                        let drag = self.tab_ruler_drag.as_mut().unwrap();
                        let step = if modifiers.shift { 10. } else { 1. };
                        let delta = if key == Key::ArrowLeft { -step } else { step };
                        drag.error = drag
                            .draft
                            .move_to(f32::from(drag.draft.position) + delta)
                            .err();
                    }
                    Key::F3 => {
                        let drag = self.tab_ruler_drag.as_mut().unwrap();
                        if drag.error.is_none() && !drag.draft.deleting {
                            use crate::model::TabLeader as L;
                            let common = crate::tab_ruler::leader_summary(
                                &drag.draft.preview,
                                self.session.editor.selection,
                                drag.draft.position,
                            )
                            .ok()
                            .and_then(|s| s.common());
                            let current = drag.draft.preview_leader_choice().or(common);
                            let next = if modifiers.shift {
                                match current {
                                    Some(Some(L::Dots)) => None,
                                    Some(Some(L::Dashes)) => Some(L::Dots),
                                    Some(Some(L::Line)) => Some(L::Dashes),
                                    _ => Some(L::Line),
                                }
                            } else {
                                match current {
                                    Some(None) => Some(L::Dots),
                                    Some(Some(L::Dots)) => Some(L::Dashes),
                                    Some(Some(L::Dashes)) => Some(L::Line),
                                    _ => None,
                                }
                            };
                            drag.error = drag.draft.set_preview_leader(next).err();
                        }
                    }
                    Key::Num0 | Key::Num1 | Key::Num2 | Key::Num3 if !modifiers.shift => {
                        let drag = self.tab_ruler_drag.as_mut().unwrap();
                        if drag.error.is_none() && !drag.draft.deleting {
                            use crate::model::TabLeader as L;
                            let leader = match key {
                                Key::Num1 => Some(L::Dots),
                                Key::Num2 => Some(L::Dashes),
                                Key::Num3 => Some(L::Line),
                                _ => None,
                            };
                            drag.error = drag.draft.set_preview_leader(leader).err();
                        }
                    }
                    Key::Backspace => {
                        let drag = self.tab_ruler_drag.as_mut().unwrap();
                        if drag.error.is_none() && !drag.draft.deleting {
                            drag.error = drag.draft.restore_preview_leaders().err();
                        }
                    }
                    Key::Delete => {
                        if self.tab_ruler_drag.as_ref().unwrap().draft.is_creation() {
                            self.tab_ruler_drag = None;
                            self.sync_view();
                            break;
                        }
                        let drag = self.tab_ruler_drag.as_mut().unwrap();
                        drag.error = drag.draft.remove().err();
                    }
                    Key::Enter | Key::Tab => {
                        if self.tab_ruler_drag.as_ref().unwrap().error.is_some() {
                            continue;
                        }
                        let drag = self.tab_ruler_drag.take().unwrap();
                        let position = drag.draft.position;
                        let deleting = drag.draft.deleting;
                        let creating = drag.draft.is_creation();
                        let result = drag.draft.commit(&mut self.session.editor);
                        let success = result.is_ok();
                        self.fail(result);
                        self.sync_view();
                        if success && key == Key::Tab && !deleting && !creating {
                            let stops = crate::tab_ruler::positions(
                                &self.session.editor.document,
                                self.session.editor.selection,
                            )
                            .unwrap();
                            let current = stops.iter().position(|(p, _)| *p == position).unwrap();
                            let next = stops[(current
                                + if modifiers.shift { stops.len() - 1 } else { 1 })
                                % stops.len()]
                            .0;
                            self.start_tab_keyboard(ctx, next);
                        }
                        break;
                    }
                    _ => {}
                }
            }
            return true;
        }
        if self.tab_ruler_drag.is_none() {
            return false;
        }
        let cancel = !self.rulers
            || self.backstage
            || ctx.input(|i| {
                i.key_pressed(Key::Escape)
                    || !i.focused
                    || i.viewport().close_requested()
                    || i.events.iter().any(|e| {
                        matches!(
                            e,
                            egui::Event::PointerGone | egui::Event::WindowFocused(false)
                        )
                    })
                    || (!i.pointer.primary_down() && !i.pointer.primary_released())
            });
        ctx.input_mut(|i| {
            i.events.retain(|e| {
                matches!(
                    e,
                    egui::Event::PointerMoved(_)
                        | egui::Event::PointerButton { .. }
                        | egui::Event::PointerGone
                        | egui::Event::WindowFocused(_)
                )
            });
            i.raw_scroll_delta = Vec2::ZERO;
            i.smooth_scroll_delta = Vec2::ZERO;
        });
        if cancel {
            self.tab_ruler_drag = None;
            self.sync_view();
        }
        true
    }
    fn start_tab_keyboard(&mut self, ctx: &egui::Context, value: u16) {
        self.start_tab_keyboard_mode(ctx, value, false);
    }
    fn start_tab_keyboard_mode(&mut self, ctx: &egui::Context, value: u16, creating: bool) {
        let result = if creating {
            crate::tab_ruler::Draft::creation_preview_typed(
                &self.session.editor,
                self.tab_creation_kind,
            )
        } else {
            crate::tab_ruler::Draft::new(&self.session.editor, value)
        };
        match result {
            Ok(mut draft) => {
                let error = if creating {
                    draft.move_to(f32::from(value)).err()
                } else {
                    None
                };
                self.tab_ruler_drag = Some(TabDrag {
                    draft,
                    rect: Rect::NOTHING,
                    screen: ctx.content_rect(),
                    zoom: self.zoom / 100.,
                    origin: 0.,
                    start: value,
                    unit: 1.,
                    error,
                    keyboard: true,
                    show_leaders: false,
                    numeric: false,
                    input: String::new(),
                    select_input: false,
                    accept_input: false,
                    popup: Rect::NOTHING,
                    control_focus: None,
                    activate_control: false,
                });
                self.leave_ribbon(ctx);
                self.focus_document = false;
                ctx.memory_mut(|m| m.surrender_focus(Id::new(EDIT_ID)));
            }
            Err(error) => self.error = Some(error),
        }
    }
    pub(super) fn tab_markers(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        left: f32,
        right: f32,
        zoom: f32,
        indent_targets: &[Rect],
    ) {
        if let Some(drag) = &mut self.tab_ruler_drag
            && drag.rect == Rect::NOTHING
        {
            drag.rect = rect;
        }
        if self.tab_ruler_drag.as_ref().is_some_and(|d| {
            d.rect != rect || d.zoom != zoom || d.screen != ui.ctx().content_rect()
        }) {
            self.tab_ruler_drag = None;
            self.sync_view();
        }
        let (start, end) = self.session.editor.selection;
        let model = self
            .tab_ruler_drag
            .as_ref()
            .map(|d| &d.draft.preview)
            .unwrap_or(&self.session.editor.document);
        let Ok((first, _)) = model.locate(start) else {
            return;
        };
        let Ok((mut last, offset)) = model.locate(end) else {
            return;
        };
        if end > start && offset == 0 && last > first {
            last -= 1;
        }
        let positions: Vec<_> = crate::tab_ruler::positions(model, (start, end))
            .unwrap_or_default()
            .into_iter()
            .map(|(position, coverage)| {
                (
                    position,
                    coverage,
                    crate::tab_ruler::common_kind(model, (start, end), position),
                )
            })
            .collect();
        let total = last - first + 1;
        let unit = 14.6667 * zoom / 22.;
        let enabled = self.enabled(Action::TabPositions)
            && !self.composing
            && !self.backstage
            && self.ruler_key_focus.is_none();
        for (value, coverage, kind) in positions {
            let leaders = crate::tab_ruler::leader_summary(
                self.tab_ruler_drag
                    .as_ref()
                    .map(|d| &d.draft.preview)
                    .unwrap_or(&self.session.editor.document),
                (start, end),
                value,
            )
            .unwrap();
            let active_draft = self
                .tab_ruler_drag
                .as_ref()
                .filter(|d| d.draft.position == value);
            let sources = if active_draft.is_some_and(|d| d.draft.is_creation()) {
                "New stop preview; no authored source yet.".into()
            } else {
                let original_position = active_draft.map_or(value, |d| d.start);
                crate::tab_sources::inspect(
                    &self.session.editor.document,
                    (start, end),
                    Some(original_position),
                )
                .unwrap_or_default()
                .positions
                .summary()
            };
            let leader_details = leaders.description();
            let kind_label = kind.map_or("Mixed types", crate::model::TabKind::label);
            let mixed = coverage != total || kind.is_none();
            let x = left + f32::from(value) * unit;
            if x < left || x > right {
                continue;
            }
            let target = Rect::from_center_size(pos2(x, rect.top() + 20.), vec2(10., 8.));
            let collision = indent_targets.iter().any(|r| r.intersects(target));
            let available = enabled && !collision;
            self.record(&format!("ruler-tab-{value}"), target);
            let response = ui.interact(
                target,
                Id::new(("ruler-tab", value)),
                if available {
                    Sense::click_and_drag()
                } else {
                    Sense::hover()
                },
            );
            let active = self
                .tab_ruler_drag
                .as_ref()
                .is_some_and(|d| d.draft.position == value);
            let color = if active || (available && response.hovered()) {
                ACCENT
            } else if mixed || collision {
                Color32::from_gray(150)
            } else {
                INK
            };
            let foot = match kind {
                Some(crate::model::TabKind::Left) => (0., 5.),
                Some(crate::model::TabKind::Right) => (-5., 0.),
                Some(crate::model::TabKind::Center) => (-5., 5.),
                None => (-4., 4.),
            };
            ui.painter().line_segment(
                [pos2(x, rect.top() + 17.), pos2(x, rect.top() + 23.)],
                Stroke::new(2.0_f32, color),
            );
            ui.painter().line_segment(
                [
                    pos2(x + foot.0, rect.top() + 23.),
                    pos2(x + foot.1, rect.top() + 23.),
                ],
                Stroke::new(2.0_f32, color),
            );
            if kind.is_none() {
                ui.painter().line_segment(
                    [
                        pos2(x - 3., rect.top() + 17.),
                        pos2(x + 3., rect.top() + 17.),
                    ],
                    Stroke::new(1.0_f32, color),
                );
            }
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    available,
                    format!(
                        "{kind_label} tab, {:.1} points, {coverage}/{total} paragraphs. {leader_details} Sources: {sources}",
                        f32::from(value) / 2.
                    ),
                )
            });
            if available
                && ui.input(|i| i.pointer.primary_pressed())
                && response.is_pointer_button_down_on()
                && let Some(pointer) = ui.input(|i| i.pointer.interact_pos())
            {
                match crate::tab_ruler::Draft::new(&self.session.editor, value) {
                    Ok(draft) => {
                        self.tab_ruler_drag = Some(TabDrag {
                            draft,
                            rect,
                            screen: ui.ctx().content_rect(),
                            zoom,
                            origin: pointer.x,
                            start: value,
                            unit,
                            error: None,
                            keyboard: false,
                            show_leaders: false,
                            numeric: false,
                            input: String::new(),
                            select_input: false,
                            accept_input: false,
                            popup: Rect::NOTHING,
                            control_focus: None,
                            activate_control: false,
                        });
                        self.leave_ribbon(ui.ctx());
                        self.focus_document = false;
                        ui.memory_mut(|m| m.surrender_focus(Id::new(EDIT_ID)));
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            response.context_menu(|ui| {
                let source = ui.button("Tabulation sources…");
                self.record("ruler-tab-sources", source.rect);
                if source.clicked() {
                    ui.close();
                    self.open_tab_sources(ui.ctx(), Some(value));
                    if let Some(dialog) = &mut self.tab_sources_dialog {
                        dialog.return_focus = Some(Id::new(EDIT_ID));
                    }
                }
                let edit = ui.add_enabled(available, Button::new("Edit tab preview…"));
                self.record("ruler-tab-preview", edit.rect);
                if edit.clicked() {
                    ui.close();
                    self.start_tab_keyboard(ui.ctx(), value);
                }
                let remove = ui.add_enabled(
                    available,
                    Button::new(format!("Delete tab at {:.1} pt", f32::from(value) / 2.)),
                );
                self.record("ruler-tab-delete", remove.rect);
                if remove.clicked() {
                    let result = crate::tab_ruler::Draft::new(&self.session.editor, value)
                        .and_then(|mut draft| {
                            draft.remove()?;
                            draft.commit(&mut self.session.editor)
                        });
                    self.fail(result);
                    self.sync_view();
                    ui.close();
                }
                ui.separator();
                ui.label("Leader — paragraphs owning this stop");
                ui.add(egui::Label::new(&leader_details).wrap());
                for leader in [
                    None,
                    Some(crate::model::TabLeader::Dots),
                    Some(crate::model::TabLeader::Dashes),
                    Some(crate::model::TabLeader::Line),
                ] {
                    let label = leader.map_or("None", |v| v.label());
                    let index = match leader {
                        None => 0,
                        Some(crate::model::TabLeader::Dots) => 1,
                        Some(crate::model::TabLeader::Dashes) => 2,
                        Some(crate::model::TabLeader::Line) => 3,
                    };
                    let r = ui.add_enabled(
                        available,
                        Button::new(format!(
                            "{label} ({}/{})",
                            leaders.counts[index],
                            leaders.owners()
                        ))
                        .selected(leaders.common() == Some(leader)),
                    );
                    self.record(&format!("ruler-leader-{}", label.to_lowercase()), r.rect);
                    if r.clicked() {
                        let result = self
                            .session
                            .editor
                            .set_owned_tab_leader(start, end, value, leader);
                        self.fail(result);
                        self.sync_view();
                        ui.close();
                    }
                }
            });
            response.on_hover_text(format!("{kind_label} tab: {:.1} pt. Present in {coverage}/{total} selected paragraphs. {leader_details} Sources: {sources}. Movement/deletion affects only paragraphs containing this stop and preserves each type. {}",f32::from(value)/2.,if collision {"Overlaps an indent marker; use Ctrl+Alt+F8 or Tab positions to edit."} else {"Drag to preview; release to apply; Escape cancels. Release outside the ruler cancels. Ctrl+Alt+F8 selects tabs; Ctrl+Alt+Shift+F8 creates; arrows move, Shift accelerates, Enter applies, Tab applies and selects next, Delete previews removal, Escape cancels. Right-click to delete."}));
        }
        if self.tab_sources_dialog.is_none()
            && let Some(drag) = &mut self.tab_ruler_drag
        {
            if !drag.keyboard
                && let Some(pointer) = ui.input(|i| i.pointer.interact_pos())
            {
                drag.error = drag
                    .draft
                    .move_to(f32::from(drag.start) + (pointer.x - drag.origin) / drag.unit)
                    .err();
                ui.ctx().request_repaint();
            }
            let kind_label = drag
                .draft
                .kind()
                .map_or("Mixed types", crate::model::TabKind::label);
            let text = drag.error.clone().unwrap_or_else(|| {
                let prefix = if drag.draft.is_creation() { "New " } else { "" };
                if drag.draft.deleting {
                    format!("Delete tab at {:.1} pt", f32::from(drag.start) / 2.)
                } else {
                    format!(
                        "{prefix}{kind_label} tab · {:.1} pt",
                        f32::from(drag.draft.position) / 2.
                    )
                }
            });
            let mut apply = false;
            let mut cancel = false;
            let mut inspect_sources = false;
            let popup = egui::Area::new(Id::new("tab-ruler-preview"))
                .order(egui::Order::Tooltip)
                .fixed_pos(pos2(rect.left() + 12., rect.bottom() + 4.))
                .interactable(drag.keyboard)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_max_width((ui.ctx().content_rect().width() - 48.).clamp(120., 420.));
                        if drag.numeric {
                            ui.label(if drag.draft.is_creation() {
                                format!("New {kind_label} tab")
                            } else {
                                kind_label.to_owned()
                            });
                            ui.horizontal(|ui| {
                                ui.label("Position:");
                                let mut out = egui::TextEdit::singleline(&mut drag.input)
                                    .id(Id::new("ruler-tab-position-input"))
                                    .desired_width(90.)
                                    .char_limit(16)
                                    .show(ui);
                                self.regions.push(Region { name: "tab-preview-position-input".into(), rect: out.response.rect });
                                if out.response.clicked() || out.response.is_pointer_button_down_on() { drag.control_focus = None; }
                                if out.response.changed() {
                                    ui.ctx().request_repaint();
                                }
                                out.response.widget_info(|| {
                                    egui::WidgetInfo::labeled(
                                        egui::WidgetType::TextEdit,
                                        true,
                                        "Tab position in points, 0.5 to 1440",
                                    )
                                });
                                if drag.select_input {
                                    out.response.request_focus();
                                    out.state.cursor.set_char_range(Some(
                                        egui::text::CCursorRange::two(
                                            egui::text::CCursor::new(0),
                                            egui::text::CCursor::new(drag.input.chars().count()),
                                        ),
                                    ));
                                    out.state.store(ui.ctx(), out.response.id);
                                    drag.select_input = false;
                                }
                                ui.label("pt");
                            });
                            drag.error = drag.draft.move_to_text(&drag.input).err();
                            if let Some(error) = &drag.error {
                                ui.add(
                                    egui::Label::new(RichText::new(error).color(Color32::DARK_RED))
                                        .wrap(),
                                );
                            }
                        } else {
                            ui.add(egui::Label::new(RichText::new(text).color(if drag.error.is_some() { Color32::DARK_RED } else { INK })).wrap())
                                .on_hover_text(if drag.keyboard { "Preview only. Enter applies; Escape cancels. F7 shows keyboard help." } else { "Drag to preview. Release on the ruler to apply; Escape cancels." });
                        }
                        if drag.keyboard {
                            if drag.error.is_none() && !drag.draft.deleting
                                && let Ok(summary) = crate::tab_ruler::leader_summary(&drag.draft.preview, self.session.editor.selection, drag.draft.position) {
                                let leader = summary.common().map_or("Mixed", |v| v.map_or("None", |v| v.label()));
                                let r = ui.small(format!("Leader: {leader} · {} of {} paragraphs", summary.owners(), summary.selected))
                                    .on_hover_text(format!("{}\nF1: show or hide details. Only selected paragraphs owning this stop change.", summary.description()));
                                self.regions.push(Region { name: "tab-leader-summary".into(), rect: r.rect });
                                if drag.show_leaders {
                                    let r = ui.add(egui::Label::new(summary.description()).wrap());
                                    self.regions.push(Region { name: "tab-leader-details".into(), rect: r.rect });
                                }
                            }
                            ui.separator();
                            let valid = drag.error.is_none() && !drag.draft.deleting;
                            ui.horizontal_wrapped(|ui| {
                                for (index, leader) in [None, Some(crate::model::TabLeader::Dots), Some(crate::model::TabLeader::Dashes), Some(crate::model::TabLeader::Line)].into_iter().enumerate() {
                                    let label = leader.map_or("None", |v| v.label());
                                    let common = crate::tab_ruler::leader_summary(&drag.draft.preview, self.session.editor.selection, drag.draft.position).ok().and_then(|s| s.common());
                                    let r = ui.add_enabled(valid, Button::new(label).selected(valid && drag.draft.preview_leader_choice().or(common) == Some(leader)));
                                    self.regions.push(Region { name: format!("tab-preview-leader-{}", label.to_lowercase()), rect: r.rect });
                                    if drag.control(ui, index, &r) { drag.error = drag.draft.set_preview_leader(leader).err(); }
                                }
                            });
                            ui.horizontal_wrapped(|ui| {
                                let r = ui.add_enabled(valid && drag.draft.preview_leader_choice().is_some(), Button::new("Restore leaders"));
                                self.regions.push(Region { name: "tab-preview-restore".into(), rect: r.rect });
                                if drag.control(ui, 4, &r) { drag.error = drag.draft.restore_preview_leaders().err(); }
                                let r = ui.add_enabled(!drag.numeric || drag.error.is_none(), Button::new(if drag.numeric { "Back to marker" } else { "Exact position…" }));
                                self.regions.push(Region { name: "tab-preview-position".into(), rect: r.rect });
                                if drag.control(ui, 5, &r) {
                                    if drag.numeric { drag.focus_marker(ui.ctx()); }
                                    else { drag.focus_numeric(ui.ctx()); }
                                    ui.ctx().request_repaint();
                                }
                                let r = ui.add_enabled(drag.error.is_none(), Button::new("Apply"));
                                self.regions.push(Region { name: "tab-preview-apply".into(), rect: r.rect });
                                apply = drag.control(ui, 6, &r);
                                let r = ui.button("Cancel");
                                self.regions.push(Region { name: "tab-preview-cancel".into(), rect: r.rect });
                                cancel = drag.control(ui, 7, &r);
                                // This auxiliary action must not take the editing/control focus.
                                // Restoring only the widget ID leaves the numeric caret/selection intact.
                                let previous_focus = if drag.numeric && drag.control_focus.is_none() {
                                    Some(Id::new("ruler-tab-position-input"))
                                } else {
                                    ui.memory(|m| m.focused())
                                };
                                let source = ui.button("Sources…").on_hover_text("F9: inspect original tabulation sources without changing this preview.");
                                self.regions.push(Region { name: "tab-preview-sources".into(), rect: source.rect });
                                inspect_sources = source.clicked();
                                if source.clicked() || source.is_pointer_button_down_on() {
                                    ui.memory_mut(|m| { m.surrender_focus(source.id); if let Some(id) = previous_focus { m.request_focus(id); } });
                                }
                                let r = ui.add(Button::new("Help").selected(self.tab_preview_help));
                                self.regions.push(Region { name: "tab-preview-help-toggle".into(), rect: r.rect });
                                if r.clicked() {
                                    self.tab_preview_help = !self.tab_preview_help;
                                    ui.ctx().request_repaint();
                                }
                                if r.clicked() || r.is_pointer_button_down_on() {
                                    ui.memory_mut(|m| {
                                        m.surrender_focus(r.id);
                                        if let Some(id) = previous_focus { m.request_focus(id); }
                                    });
                                }
                                let help = format!("Preview help {}. F7 toggles descriptions without leaving the current control or position input. Hover descriptions remain available. Remembered for this editor window only.", if self.tab_preview_help { "shown" } else { "hidden" });
                                r.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, self.tab_preview_help, &help));
                                r.on_hover_text(help);
                            });
                            if self.tab_preview_help {
                                let help = if let Some(index) = drag.control_focus {
                                    drag.control_help(index)
                                } else if drag.numeric {
                                    "Position in 0.5 pt steps. F4: return to marker; Enter: apply; Esc: cancel. F6: preview controls; F1: leader details; F7: hide help. Digits edit the position.".into()
                                } else {
                                    "Arrows: move; Shift+arrows: larger step. 0–3: None, Dots, Dashes, Line; F3 / Shift+F3: cycle; Backspace: restore leaders. F2: exact position; F6: controls; F1: details; F7: hide help. Enter: apply; Esc: cancel.".into()
                                };
                                let r = ui.add(egui::Label::new(help).wrap());
                                self.regions.push(Region { name: "tab-preview-help".into(), rect: r.rect });
                            }

                        }
                    });
                });
            drag.popup = popup.response.rect;
            if inspect_sources {
                let stop = (!drag.draft.is_creation()).then_some(drag.start);
                self.open_tab_sources(ui.ctx(), stop);
                return;
            }
            if cancel {
                self.tab_ruler_drag = None;
                ui.memory_mut(|m| m.surrender_focus(Id::new("ruler-tab-position-input")));
                self.sync_view();
                return;
            }
            if (apply || (drag.numeric && drag.accept_input)) && drag.error.is_none() {
                let drag = self.tab_ruler_drag.take().unwrap();
                let result = drag.draft.commit(&mut self.session.editor);
                self.fail(result);
                ui.memory_mut(|m| m.surrender_focus(Id::new("ruler-tab-position-input")));
                self.sync_view();
                return;
            }
            if !drag.keyboard && ui.input(|i| i.pointer.primary_released()) {
                let drag = self.tab_ruler_drag.take().unwrap();
                let inside = ui.input(|i| {
                    i.pointer
                        .interact_pos()
                        .is_some_and(|p| rect.expand(12.).contains(p))
                });
                if inside {
                    if drag.error.is_none() {
                        let result = drag.draft.commit(&mut self.session.editor);
                        self.fail(result);
                    } else if drag.draft.is_creation() {
                        self.error = drag.error;
                    }
                }
                self.sync_view();
            }
        }
    }
}
