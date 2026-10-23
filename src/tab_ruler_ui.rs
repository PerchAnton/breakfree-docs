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
    focus_input: bool,
    accept_input: bool,
    popup: Rect,
    control_focus: Option<usize>,
    activate_control: bool,
}
impl TabDrag {
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

    fn control(&mut self, ui: &Ui, index: usize, response: &egui::Response) -> bool {
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
                    focus_input: false,
                    accept_input: false,
                    popup: Rect::NOTHING,
                    control_focus: None,
                    activate_control: false,
                });
                self.focus_document = false;
                self.leave_ribbon(ctx);
                ctx.memory_mut(|m| m.surrender_focus(Id::new(EDIT_ID)));
                ctx.request_repaint();
            }
            Err(error) => self.error = Some(error),
        }
    }
    pub(super) fn prepare_tab_ruler_input(&mut self, ctx: &egui::Context) -> bool {
        let eligible = self.tab_ruler_drag.is_none()
            && self.enabled(Action::TabPositions)
            && !self.composing
            && !self.backstage
            && self.ruler_key_focus.is_none()
            && !egui::Popup::is_any_open(ctx);
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
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F6)) {
                let drag = self.tab_ruler_drag.as_mut().unwrap();
                drag.control_focus = if drag.control_focus.is_some() {
                    None
                } else {
                    drag.enabled_controls().first().copied()
                };
                if drag.control_focus.is_none() && drag.numeric {
                    drag.focus_input = true;
                }
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
                drag.numeric = true;
                drag.control_focus = None;
                drag.input = format!("{}", f32::from(drag.draft.position) / 2.);
                drag.focus_input = true;
            }
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F4)) {
                let drag = self.tab_ruler_drag.as_mut().unwrap();
                if drag.numeric {
                    drag.error = drag.draft.move_to_text(&drag.input).err();
                    if drag.error.is_none() {
                        drag.numeric = false;
                        drag.control_focus = None;
                        ctx.memory_mut(|m| m.surrender_focus(Id::new("ruler-tab-position-input")));
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
                            drag.show_leaders = true;
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
                            drag.show_leaders = true;
                        }
                    }
                    Key::Backspace => {
                        let drag = self.tab_ruler_drag.as_mut().unwrap();
                        if drag.error.is_none() && !drag.draft.deleting {
                            drag.error = drag.draft.restore_preview_leaders().err();
                            drag.show_leaders = true;
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
                    focus_input: false,
                    accept_input: false,
                    popup: Rect::NOTHING,
                    control_focus: None,
                    activate_control: false,
                });
                self.focus_document = false;
                self.leave_ribbon(ctx);
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
                &self.session.editor.document,
                (start, end),
                value,
            )
            .unwrap();
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
                        "{kind_label} tab, {:.1} points, {coverage}/{total} paragraphs. {leader_details}",
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
                            focus_input: false,
                            accept_input: false,
                            popup: Rect::NOTHING,
                            control_focus: None,
                            activate_control: false,
                        });
                        self.focus_document = false;
                        self.leave_ribbon(ui.ctx());
                        ui.memory_mut(|m| m.surrender_focus(Id::new(EDIT_ID)));
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            response.context_menu(|ui| {
                let edit = ui.add_enabled(available, Button::new("Edit tab preview…"));
                self.record("ruler-tab-preview", edit.rect);
                if edit.clicked() {
                    ui.close();
                    self.start_tab_keyboard(ui.ctx(), value);
                    if let Some(drag) = &mut self.tab_ruler_drag {
                        drag.show_leaders = true;
                    }
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
            response.on_hover_text(format!("{kind_label} tab: {:.1} pt. Present in {coverage}/{total} selected paragraphs. {leader_details} Movement/deletion affects only paragraphs containing this stop and preserves each type. {}",f32::from(value)/2.,if collision {"Overlaps an indent marker; use Ctrl+Alt+F8 or Tab positions to edit."} else {"Drag to preview; release to apply; Escape cancels. Release outside the ruler cancels. Ctrl+Alt+F8 selects tabs; Ctrl+Alt+Shift+F8 creates; arrows move, Shift accelerates, Enter applies, Tab applies and selects next, Delete previews removal, Escape cancels. Right-click to delete."}));
        }
        if let Some(drag) = &mut self.tab_ruler_drag {
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
            let text = drag
                .error
                .clone()
                .map(|error| {
                    if drag.keyboard {
                        format!("{error} Arrows adjust; Esc cancels.")
                    } else {
                        error
                    }
                })
                .unwrap_or_else(|| {
                    if drag.draft.is_creation() {
                        return format!(
                            "New {kind_label} tab {:.1} pt — {}; Esc cancels",
                            f32::from(drag.draft.position) / 2.,
                            if drag.keyboard {
                                "Enter applies"
                            } else {
                                "release applies"
                            }
                        );
                    }
                    if drag.draft.deleting {
                        return format!(
                            "Delete tab {:.1} pt — Enter applies; Esc cancels",
                            f32::from(drag.start) / 2.
                        );
                    }
                    format!(
                        "{kind_label} tab {:.1} pt — preview; {}; Esc cancels",
                        f32::from(drag.draft.position) / 2.,
                        if drag.keyboard {
                            "Enter applies"
                        } else {
                            "release applies"
                        }
                    )
                });
            let text = if drag.control_focus.is_some() {
                text.replace("Enter applies", "Apply confirms")
            } else {
                text
            };
            let mut apply = false;
            let mut cancel = false;
            let popup = egui::Area::new(Id::new("tab-ruler-preview"))
                .order(egui::Order::Tooltip)
                .fixed_pos(pos2(rect.left() + 12., rect.bottom() + 4.))
                .interactable(drag.keyboard)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_max_width((ui.ctx().content_rect().width() - 48.).clamp(120., 520.));
                        if drag.show_leaders {
                            if drag.error.is_none() && !drag.draft.deleting {
                                if let Ok(summary) = crate::tab_ruler::leader_summary(&drag.draft.preview, self.session.editor.selection, drag.draft.position) {
                                    let r = ui.add(egui::Label::new(summary.description()).wrap());
                                    self.regions.push(Region { name: "tab-leader-details".into(), rect: r.rect });
                                }
                            } else { ui.label("Leader details unavailable for an invalid or deletion preview."); }
                        }
                        if drag.show_leaders && drag.error.is_none() && !drag.draft.deleting {
                            let choice = match drag.draft.preview_leader_choice() {
                                Some(value) => format!("Chosen leader: {}", value.map_or("None", |v| v.label())),
                                None => "Original per-paragraph leaders retained".into(),
                            };
                            ui.add(egui::Label::new(choice).wrap());
                        }
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
                                if out.response.clicked() { drag.control_focus = None; }
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
                                if drag.focus_input {
                                    out.response.request_focus();
                                    out.state.cursor.set_char_range(Some(
                                        egui::text::CCursorRange::two(
                                            egui::text::CCursor::new(0),
                                            egui::text::CCursor::new(drag.input.chars().count()),
                                        ),
                                    ));
                                    out.state.store(ui.ctx(), out.response.id);
                                    drag.focus_input = false;
                                }
                                ui.label("pt");
                            });
                            drag.error = drag.draft.move_to_text(&drag.input).err();
                            ui.add(egui::Label::new(if drag.control_focus.is_some() { "F6 returns to input · Apply confirms · Esc cancels" } else { "0.5 pt steps · F4 returns to marker · Enter applies · Esc cancels" }).wrap());
                            if let Some(error) = &drag.error {
                                ui.add(
                                    egui::Label::new(RichText::new(error).color(Color32::DARK_RED))
                                        .wrap(),
                                );
                            }
                        } else {
                            ui.add(egui::Label::new(text).wrap());
                            if drag.keyboard && drag.control_focus.is_none() {
                                ui.add(egui::Label::new("0 None · 1 Dots · 2 Dashes · 3 Line · Backspace restores original leaders").wrap());
                                ui.add(egui::Label::new("F1 details · F2 position · F3 next / Shift+F3 previous").wrap());
                            }
                        }
                        if drag.keyboard {
                            ui.add(egui::Label::new(if drag.control_focus.is_some() { "Controls: Tab / Shift+Tab · Enter / Space activates · F6 returns" } else { "F6: focus preview controls" }).wrap());
                            ui.separator();
                            let valid = drag.error.is_none() && !drag.draft.deleting;
                            ui.horizontal_wrapped(|ui| {
                                for (index, leader) in [None, Some(crate::model::TabLeader::Dots), Some(crate::model::TabLeader::Dashes), Some(crate::model::TabLeader::Line)].into_iter().enumerate() {
                                    let label = leader.map_or("None", |v| v.label());
                                    let common = crate::tab_ruler::leader_summary(&drag.draft.preview, self.session.editor.selection, drag.draft.position).ok().and_then(|s| s.common());
                                    let r = ui.add_enabled(valid, Button::new(label).selected(valid && drag.draft.preview_leader_choice().or(common) == Some(leader))).on_disabled_hover_text("Choose a valid position; leaders cannot be changed during deletion.");
                                    self.regions.push(Region { name: format!("tab-preview-leader-{}", label.to_lowercase()), rect: r.rect });
                                    if drag.control(ui, index, &r) { drag.error = drag.draft.set_preview_leader(leader).err(); drag.show_leaders = true; }
                                }
                            });
                            ui.horizontal_wrapped(|ui| {
                                let r = ui.add_enabled(valid && drag.draft.preview_leader_choice().is_some(), Button::new("Restore leaders")).on_hover_text("Restore each original leader, keeping the current position.");
                                self.regions.push(Region { name: "tab-preview-restore".into(), rect: r.rect });
                                if drag.control(ui, 4, &r) { drag.error = drag.draft.restore_preview_leaders().err(); drag.show_leaders = true; }
                                let r = ui.add_enabled(!drag.numeric || drag.error.is_none(), Button::new(if drag.numeric { "Back to marker" } else { "Exact position…" }));
                                self.regions.push(Region { name: "tab-preview-position".into(), rect: r.rect });
                                if drag.control(ui, 5, &r) {
                                    drag.control_focus = None;
                                    drag.numeric = !drag.numeric;
                                    ui.ctx().request_repaint();
                                    if drag.numeric { drag.input = format!("{}", f32::from(drag.draft.position) / 2.); drag.focus_input = true; }
                                    else { ui.memory_mut(|m| m.surrender_focus(Id::new("ruler-tab-position-input"))); }
                                }
                                let r = ui.add_enabled(drag.error.is_none(), Button::new("Apply"));
                                self.regions.push(Region { name: "tab-preview-apply".into(), rect: r.rect });
                                apply = drag.control(ui, 6, &r);
                                let r = ui.button("Cancel");
                                self.regions.push(Region { name: "tab-preview-cancel".into(), rect: r.rect });
                                cancel = drag.control(ui, 7, &r);
                            });
                        }
                    });
                });
            drag.popup = popup.response.rect;
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
