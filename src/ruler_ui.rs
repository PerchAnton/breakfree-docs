//! Native adapter for the shared ruler draft; no model mutations during motion.
use super::*;
use crate::ruler::{Draft, Handle, ResetIndent, indent_scale};

pub(super) struct Drag {
    pub draft: Draft,
    origin: f32,
    unit: f32,
    rect: Rect,
    zoom: f32,
    screen: Rect,
    keyboard: bool,
}
impl DocsApp {
    pub(super) fn prepare_ruler_input(&mut self, ctx: &egui::Context) {
        if self.prepare_tab_ruler_input(ctx) {
            return;
        }
        // egui derives scroll deltas before UI dispatch. Consume those as well as
        // events, and drain smoothing after dismissal so the canvas cannot jump.
        if self.ruler_sources || self.ruler_sources_wheel_guard {
            ctx.input_mut(|i| {
                let new_wheel = i
                    .events
                    .iter()
                    .any(|e| matches!(e, egui::Event::MouseWheel { .. }));
                if !self.ruler_sources && new_wheel {
                    self.ruler_sources_wheel_guard = false;
                } else {
                    self.ruler_sources_wheel_guard = self.ruler_sources
                        || i.smooth_scroll_delta != Vec2::ZERO
                        || i.raw_scroll_delta != Vec2::ZERO;
                    i.raw_scroll_delta = Vec2::ZERO;
                    i.smooth_scroll_delta = Vec2::ZERO;
                }
            });
        }
        if self.ruler_keyboard_input(ctx) {
            if self.ruler_sources {
                self.ruler_sources_wheel_guard = true;
                ctx.input_mut(|i| {
                    i.raw_scroll_delta = Vec2::ZERO;
                    i.smooth_scroll_delta = Vec2::ZERO;
                });
            }
            return;
        }
        if self.ruler_drag.is_none() {
            return;
        }
        let cancel = ctx.input(|i| {
            i.key_pressed(Key::Escape)
                || i.events
                    .iter()
                    .any(|e| matches!(e, egui::Event::PointerGone))
                || !i.focused
                || i.viewport().close_requested()
                || (!i.pointer.primary_down() && !i.pointer.primary_released())
        });
        // Do not let text, shortcuts, wheel zoom or IME alter a draft's source.
        ctx.input_mut(|i| {
            i.events.retain(|event| {
                matches!(
                    event,
                    egui::Event::PointerMoved(_)
                        | egui::Event::PointerButton { .. }
                        | egui::Event::PointerGone
                        | egui::Event::WindowFocused(_)
                )
            })
        });
        if cancel {
            self.ruler_drag = None;
            self.ruler_key_focus = None;
            self.sync_view();
        }
    }

    pub(super) fn ruler(&mut self, ui: &mut Ui, width: f32, zoom: f32) {
        let (rect, _) = ui.allocate_exact_size(vec2(width, 24.), Sense::hover());
        // Keep the horizontal ruler at the viewport top while the canvas scrolls.
        let rect = rect.translate(vec2(0., (ui.clip_rect().top() - rect.top()).max(0.)));
        self.record("ruler", rect);
        self.tab_type_selector(ui, rect);
        let mut menu_response = ui.interact(rect, Id::new("ruler-menu"), Sense::click());
        if let Some(drag) = &mut self.ruler_drag
            && drag.rect == Rect::NOTHING
        {
            drag.rect = rect;
        }
        if self.ruler_drag.as_ref().is_some_and(|d| {
            (!d.keyboard && d.rect != rect) || d.zoom != zoom || d.screen != ui.ctx().content_rect()
        }) {
            self.ruler_drag = None;
            self.ruler_key_focus = None;
            self.sync_view();
        }
        let (left_margin, right_margin) = if self.paged_editing {
            let p = self
                .session
                .editor
                .document
                .locate(self.session.editor.selection.0)
                .map_or(0, |v| v.0);
            let section = crate::pagination::sections(&self.session.editor.document)[p].clone();
            (
                f32::from(section.left_points) * 4. / 3. * zoom,
                f32::from(section.right_points) * 4. / 3. * zoom,
            )
        } else {
            (CANVAS_MARGIN * zoom, CANVAS_MARGIN * zoom)
        };
        let left = rect.left() + left_margin;
        let right = rect.right() - right_margin;
        paint_scale(ui, rect, left, right, zoom, false);
        let model = self
            .ruler_drag
            .as_ref()
            .map(|d| &d.draft.preview)
            .unwrap_or(&self.session.editor.document);
        let Ok((index, _)) = model.locate(self.session.editor.selection.0) else {
            return;
        };
        let Ok(summary) = crate::ruler::SelectionValues::read(model, self.session.editor.selection)
        else {
            return;
        };
        let style = model.resolved_paragraph(&model.paragraphs[index]);
        let unit = 14.6667 * zoom / 22. * indent_scale(style, 14.6667 * zoom / 22., right - left);
        let l = f32::from(style.left_indent_half_points) * unit;
        let special = f32::from(style.first_line_half_points) * unit;
        let handles = [
            (
                Handle::First,
                "ruler-first",
                "First line",
                left + l + special.max(0.),
                rect.top() + 4.,
            ),
            (
                Handle::Body,
                "ruler-body",
                "Continuation lines (hanging indent)",
                left + l + (-special).max(0.),
                rect.top() + 15.,
            ),
            (
                Handle::Left,
                "ruler-left",
                "Left indent (all lines)",
                left + l,
                rect.top() + 22.,
            ),
            (
                Handle::Right,
                "ruler-right",
                "Right indent",
                right - f32::from(style.right_indent_half_points) * unit,
                rect.top() + 15.,
            ),
        ];
        let enabled = self.enabled(Action::ParagraphSpacing) && !self.composing && !self.backstage;
        let mut indent_targets = Vec::new();
        for (handle, name, label, x, y) in handles {
            let target = Rect::from_center_size(
                pos2(x, y),
                vec2(12., if handle == Handle::Left { 4. } else { 8. }),
            );
            indent_targets.push(target);
            self.record(name, target);
            let response = ui.interact(
                target,
                Id::new(name),
                if enabled {
                    Sense::click_and_drag()
                } else {
                    Sense::focusable_noninteractive()
                },
            );
            // Assistive activation enters the same keyboard controller as Ctrl+Shift+F8.
            // A native accessibility click has no pointer-down event to start a drag.
            let accessible_entry = ui.input(|i| {
                i.has_accesskit_action_request(response.id, egui::accesskit::Action::Click)
                    || i.has_accesskit_action_request(response.id, egui::accesskit::Action::Focus)
            });
            if enabled && self.ruler_drag.is_none() && accessible_entry {
                self.leave_ribbon(ui.ctx());
                self.focus_document = false;
                self.ruler_key_focus = Some(handle);
                response.request_focus();
            }
            // Marker widgets own their clicks; forward secondary clicks to the
            // shared ruler menu without changing primary drag behavior.
            if response.secondary_clicked() {
                menu_response = menu_response.union(response.clone());
            }
            let active = self
                .ruler_drag
                .as_ref()
                .is_some_and(|d| d.draft.handle == handle);
            let fill = if active || response.hovered() || self.ruler_key_focus == Some(handle) {
                ACCENT
            } else {
                Color32::from_gray(245)
            };
            if handle == Handle::Left {
                ui.painter().rect(
                    target,
                    1.,
                    fill,
                    Stroke::new(1.0_f32, Color32::from_gray(110)),
                    egui::StrokeKind::Inside,
                );
            } else {
                let direction = if handle == Handle::First { 1. } else { -1. };
                ui.painter().add(egui::Shape::convex_polygon(
                    vec![
                        pos2(x - 5., y - 4. * direction),
                        pos2(x + 5., y - 4. * direction),
                        pos2(x, y + 4. * direction),
                    ],
                    fill,
                    Stroke::new(1.0_f32, Color32::from_gray(110)),
                ));
            }
            if summary.position(handle).mixed() {
                self.record(&format!("{name}-mixed"), target);
                // An offset dash distinguishes mixed positions without relying on color.
                ui.painter().line_segment(
                    [pos2(x + 8., y), pos2(x + 13., y)],
                    Stroke::new(2.0_f32, INK),
                );
            }
            if enabled
                && ui.input(|i| i.pointer.primary_pressed())
                && response.is_pointer_button_down_on()
                && self.ruler_drag.is_none()
                && let Some(pointer) = ui.input(|i| i.pointer.interact_pos())
            {
                match Draft::new(&self.session.editor, handle) {
                    Ok(draft) => {
                        self.ruler_drag = Some(Drag {
                            draft,
                            origin: pointer.x,
                            unit,
                            rect,
                            zoom,
                            screen: ui.ctx().content_rect(),
                            keyboard: false,
                        });
                        ui.memory_mut(|m| m.surrender_focus(Id::new(EDIT_ID)));
                        self.focus_document = false;
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            if self.ruler_key_focus == Some(handle) {
                if response.gained_focus() {
                    ui.scroll_to_rect(target.expand(4.), Some(Align::Center));
                }
                if !response.has_focus() && self.ruler_menu.is_none() {
                    response.request_focus();
                }
                ui.memory_mut(|m| {
                    m.set_focus_lock_filter(
                        response.id,
                        egui::EventFilter {
                            tab: true,
                            horizontal_arrows: true,
                            vertical_arrows: true,
                            escape: true,
                        },
                    )
                });
                ui.painter().rect_stroke(
                    target.expand(2.),
                    2.,
                    Stroke::new(1.0_f32, ACCENT),
                    egui::StrokeKind::Outside,
                );
            } else if response.gained_focus()
                && !self.focus_document
                && self.ruler_drag.is_none()
                && enabled
            {
                self.ruler_key_focus = Some(handle);
            }
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    enabled || self.ruler_key_focus == Some(handle),
                    format!(
                        "{label}. {}{}",
                        if self.ruler_sources && self.ruler_key_focus == Some(handle) {
                            summary.full_marker_details(handle)
                        } else {
                            summary.marker_details(handle)
                        },
                        if active {
                            " Preview; not yet applied."
                        } else {
                            ""
                        }
                    ),
                )
            });
            ui.ctx().accesskit_node_builder(response.id, |node| {
                if !enabled && self.ruler_key_focus != Some(handle) {
                    node.set_disabled();
                }
                node.set_description(if self.ruler_sources && self.ruler_key_focus == Some(handle) {
                    "All indent sources. Up/Down, Page Up/Page Down, Home/End scroll visually. Enter, Escape or F1 returns to the marker."
                } else {
                    "Tab changes marker. Arrows preview 0.5 pt; Shift+arrows 5 pt. Enter applies; Escape cancels. F1 reads all sources. Shift+F10 opens the context menu."
                });
            });
            response.on_hover_text(format!("{label} — drag to preview; release to apply; Esc cancels. Half-point steps. Uses the first selected paragraph; applies to selected paragraphs. Ctrl+Shift+F8 enters keyboard control; Tab changes marker, arrows nudge 0.5 pt, Shift+arrows 5 pt, Enter applies, Esc cancels, F1 shows sources, Delete previews all-indent reset; Shift+Delete resets the focused property to style indents. {}", summary.marker_details(handle)));
        }
        self.tab_markers(ui, rect, left, right, zoom, &indent_targets);
        if ui.input(|i| i.pointer.primary_pressed())
            && (menu_response.is_pointer_button_down_on() || menu_response.clicked())
            && self.enabled(Action::TabPositions)
            && !self.composing
            && !self.backstage
            && self.ruler_key_focus.is_none()
            && let Some(pointer) = menu_response.interact_pointer_pos()
            && pointer.x > left
            && pointer.x < right
            && !indent_targets.iter().any(|target| target.contains(pointer))
            && !self
                .regions
                .iter()
                .any(|r| r.name.starts_with("ruler-tab-") && r.rect.contains(pointer))
        {
            let value = ((pointer.x - left) / (14.6667 * zoom / 22.))
                .round()
                .clamp(1., 2880.) as u16;
            self.start_tab_pointer_creation(ui.ctx(), rect, pointer, zoom, value);
        }
        if let Some(drag) = &mut self.ruler_drag
            && !drag.keyboard
        {
            if let Some(pointer) = ui.input(|i| i.pointer.interact_pos()) {
                if let Err(error) = drag
                    .draft
                    .move_by((pointer.x - drag.origin) / drag.unit.max(0.001))
                {
                    self.error = Some(error);
                }
                ui.ctx().request_repaint();
            }
            if ui.input(|i| i.pointer.primary_released()) {
                let drag = self.ruler_drag.take().unwrap();
                let result = drag.draft.commit(&mut self.session.editor);
                self.fail(result);
                self.sync_view();
            }
        }
        let model = self
            .ruler_drag
            .as_ref()
            .map(|d| &d.draft.preview)
            .unwrap_or(&self.session.editor.document);
        let Ok(summary) = crate::ruler::SelectionValues::read(model, self.session.editor.selection)
        else {
            return;
        };
        // Values are contextual: they must not add permanent height above the canvas.
        if (self.ruler_key_focus.is_some() || self.ruler_drag.is_some())
            && self.ruler_menu.is_none()
        {
            let hint = self
                .ruler_key_focus
                .or_else(|| self.ruler_drag.as_ref().map(|d| d.draft.handle))
                .map(marker_name)
                .unwrap_or("Indent");
            let hint = if self.ruler_drag.as_ref().is_some_and(|d| d.draft.resetting) {
                self.ruler_drag.as_ref().unwrap().draft.reset_scope.label()
            } else {
                hint
            };
            let origin = pos2(
                rect.left().max(ui.clip_rect().left() + 4.),
                rect.bottom() + 5.,
            );
            let painter = ui
                .ctx()
                .layer_painter(egui::LayerId::new(
                    egui::Order::Tooltip,
                    Id::new("ruler-values"),
                ))
                .with_clip_rect(ui.clip_rect());
            let galley = if self.ruler_sources {
                let handle = self.ruler_key_focus.unwrap_or(Handle::Left);
                painter.layout(
                    format!(
                        "{} — value sources\n{}",
                        marker_name(handle),
                        summary.full_marker_details(handle)
                    ),
                    FontId::proportional(11.),
                    INK,
                    (ui.clip_rect().right() - origin.x - 16.).clamp(80., 440.),
                )
            } else {
                painter.layout_no_wrap(
                    format!("{hint}\n{}", summary.values()),
                    FontId::proportional(11.),
                    INK,
                )
            };
            let footer = self.ruler_sources.then(|| {
                painter.layout(
                    "Up/Down · PgUp/PgDn · Home/End: scroll\nEnter / Esc / F1: return to marker"
                        .into(),
                    FontId::proportional(11.),
                    INK,
                    (ui.clip_rect().right() - origin.x - 16.).clamp(80., 440.),
                )
            });
            let footer_height = footer.as_ref().map_or(0., |g| g.size().y + 8.);
            let height = if self.ruler_sources {
                galley
                    .size()
                    .y
                    .min((ui.clip_rect().bottom() - origin.y - footer_height - 12.).clamp(1., 320.))
            } else {
                galley.size().y
            };
            self.ruler_sources_height = height;
            let max_scroll = (galley.size().y - height).max(0.);
            self.ruler_sources_scroll = self.ruler_sources_scroll.clamp(0., max_scroll);
            let width = galley
                .size()
                .x
                .max(footer.as_ref().map_or(0., |g| g.size().x));
            let readout =
                Rect::from_min_size(origin, vec2(width + 12., height + footer_height + 8.));
            self.record("ruler-values", readout);
            if self.ruler_sources {
                self.record("ruler-sources", readout);
            }
            painter.rect(
                readout,
                2.,
                Color32::WHITE,
                Stroke::new(1.0_f32, Color32::from_gray(180)),
                egui::StrokeKind::Inside,
            );
            let content = Rect::from_min_size(origin + vec2(6., 4.), vec2(width, height));
            let offset = if self.ruler_sources {
                self.ruler_sources_scroll
            } else {
                0.
            };
            painter
                .with_clip_rect(content.intersect(ui.clip_rect()))
                .galley(content.min - vec2(0., offset), galley, INK);
            if let Some(footer) = footer {
                painter.galley(pos2(content.left(), content.bottom() + 8.), footer, INK);
                if max_scroll > 0. {
                    let track = Rect::from_min_size(
                        pos2(readout.right() - 4., content.top()),
                        vec2(2., height),
                    );
                    painter.rect_filled(track, 1., Color32::from_gray(215));
                    let thumb_height = (height * height / (height + max_scroll))
                        .max(8.)
                        .min(height);
                    let y = track.top() + (height - thumb_height) * offset / max_scroll;
                    painter.rect_filled(
                        Rect::from_min_size(pos2(track.left(), y), vec2(2., thumb_height)),
                        1.,
                        Color32::from_gray(120),
                    );
                }
            }
        }
        let menu_response = menu_response.on_hover_text("Click empty text-column ruler space or press Ctrl+Alt+Shift+F8 to add a left tab. Right-click a tab to delete it. Right-click or Shift+F10 on an indent marker for sources or reset. Ctrl+Shift+F8 enters markers; Enter applies; Esc cancels.");
        let mut popup = egui::Popup::context_menu(&menu_response);
        if self.ruler_menu.is_some() {
            let marker = self.ruler_key_focus.unwrap_or(Handle::First);
            let id = MARKERS.iter().find(|(h, _)| *h == marker).unwrap().1;
            let anchor = self
                .regions
                .iter()
                .find(|r| r.name == id)
                .map_or(rect.left_bottom(), |r| r.rect.left_bottom());
            popup = popup.at_position(anchor);
        }
        let pending = self.ruler_drag.as_ref().is_some_and(|d| d.keyboard);
        popup.show(|ui| {
            ui.label("Inspect indent sources");
            for (index, (handle, name, label)) in [
                (Handle::First, "ruler-sources-first", "First line sources"),
                (
                    Handle::Body,
                    "ruler-sources-body",
                    "Continuation line sources",
                ),
                (Handle::Left, "ruler-sources-left", "Left indent sources"),
                (Handle::Right, "ruler-sources-right", "Right indent sources"),
            ]
            .into_iter()
            .enumerate()
            {
                let available =
                    (enabled || pending) && (!pending || self.ruler_key_focus == Some(handle));
                let inspect = ui
                    .add_enabled(available, Button::new(label))
                    .on_disabled_hover_text("Apply or cancel the pending indent preview first.");
                if self.ruler_menu == Some(index) {
                    inspect.request_focus();
                }
                self.record(name, inspect.rect);
                if available
                    && (inspect.clicked()
                        || (self.ruler_menu == Some(index) && self.ruler_menu_activate))
                {
                    self.leave_ribbon(ui.ctx());
                    self.focus_document = false;
                    self.ruler_key_focus = Some(handle);
                    self.ruler_sources_scroll = 0.;
                    self.ruler_sources = true;
                    self.ruler_sources_wheel_guard = true;
                    let id = MARKERS.iter().find(|(h, _)| *h == handle).unwrap().1;
                    ui.ctx().memory_mut(|m| m.request_focus(Id::new(id)));
                    ui.close();
                }
            }
            ui.separator();
            for (index, (scope, name, label)) in [
                (
                    ResetIndent::All,
                    "ruler-reset",
                    "Reset all indents to style",
                ),
                (
                    ResetIndent::Left,
                    "ruler-reset-left",
                    "Reset left indent to style",
                ),
                (
                    ResetIndent::Right,
                    "ruler-reset-right",
                    "Reset right indent to style",
                ),
                (
                    ResetIndent::FirstLine,
                    "ruler-reset-first",
                    "Reset first-line/hanging to style",
                ),
            ]
            .into_iter()
            .enumerate()
            {
                let reset =
                    ui.add_enabled(enabled && self.ruler_drag.is_none(), Button::new(label));
                self.record(name, reset.rect);
                if self.ruler_menu == Some(index + 4) {
                    reset.request_focus();
                }
                if enabled
                    && self.ruler_drag.is_none()
                    && (reset.clicked()
                        || (self.ruler_menu == Some(index + 4) && self.ruler_menu_activate))
                {
                    self.start_ruler_reset(ui.ctx(), scope);
                    ui.close();
                }
            }
        });
        self.ruler_menu_activate = false;
        if self.ruler_menu.is_some()
            && !egui::Popup::is_id_open(ui.ctx(), Id::new("ruler-menu").with("popup"))
        {
            self.ruler_menu = None;
            if let Some(handle) = self.ruler_key_focus {
                let id = MARKERS.iter().find(|(h, _)| *h == handle).unwrap().1;
                ui.ctx().memory_mut(|m| m.request_focus(Id::new(id)));
            }
        }
    }

    fn start_ruler_reset(&mut self, ctx: &egui::Context, scope: ResetIndent) {
        let handle = self.ruler_key_focus.unwrap_or(Handle::Left);
        match Draft::new(&self.session.editor, handle).and_then(|mut draft| {
            draft.reset_property(scope)?;
            Ok(draft)
        }) {
            Ok(draft) => {
                self.ruler_drag = Some(Drag {
                    draft,
                    origin: 0.,
                    unit: 1.,
                    rect: Rect::NOTHING,
                    zoom: self.zoom / 100.,
                    screen: ctx.content_rect(),
                    keyboard: true,
                });
                self.ruler_key_focus = Some(handle);
                self.focus_document = false;
                let id = MARKERS.iter().find(|(h, _)| *h == handle).unwrap().1;
                ctx.memory_mut(|m| m.request_focus(Id::new(id)));
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn vertical_ruler(&mut self, ui: &mut Ui, canvas: Rect, zoom: f32) {
        let x = ui.clip_rect().left() + 2.;
        let rect = Rect::from_min_max(pos2(x, canvas.top()), pos2(x + 20., canvas.bottom()));
        self.record("ruler-vertical", rect);
        let section = &self.session.editor.document.page;
        let (top, bottom) = if self.paged_editing {
            (
                f32::from(section.top_points) * 4. / 3. * zoom,
                f32::from(section.bottom_points) * 4. / 3. * zoom,
            )
        } else {
            (CANVAS_MARGIN * zoom, CANVAS_MARGIN * zoom)
        };
        paint_scale(
            ui,
            rect,
            canvas.top() + top,
            canvas.bottom() - bottom,
            zoom,
            true,
        );
        ui.interact(rect.intersect(ui.clip_rect()), Id::new("ruler-vertical"), Sense::hover())
            .on_hover_text("Vertical ruler · inches from the text origin. Follows document scrolling; page margins are configured in Layout → Page setup.");
    }
}

const MARKERS: [(Handle, &str); 4] = [
    (Handle::First, "ruler-first"),
    (Handle::Body, "ruler-body"),
    (Handle::Left, "ruler-left"),
    (Handle::Right, "ruler-right"),
];
fn marker_name(handle: Handle) -> &'static str {
    match handle {
        Handle::First => "First line",
        Handle::Body => "Continuation lines",
        Handle::Left => "Left indent",
        Handle::Right => "Right indent",
    }
}
impl DocsApp {
    fn ruler_keyboard_input(&mut self, ctx: &egui::Context) -> bool {
        let enter = self.ruler_drag.is_none()
            && self.enabled(Action::ParagraphSpacing)
            && !self.composing
            && !self.backstage
            && !egui::Popup::is_any_open(ctx)
            && ctx.input_mut(|i| i.consume_key(Modifiers::CTRL | Modifiers::SHIFT, Key::F8));
        if enter {
            if !self.commit_font_size_input() {
                return true;
            }
            self.leave_ribbon(ctx);
            self.rulers = true;
            self.focus_document = false;
            self.ruler_key_focus = Some(Handle::First);
            ctx.memory_mut(|m| m.request_focus(Id::new("ruler-first")));
        }
        let Some(mut handle) = self.ruler_key_focus else {
            if self.ruler_menu.take().is_some() {
                egui::Popup::close_id(ctx, Id::new("ruler-menu").with("popup"));
                self.ruler_menu_activate = false;
            }
            self.ruler_sources = false;
            return false;
        };
        if self.ruler_drag.as_ref().is_some_and(|d| !d.keyboard) {
            return false;
        }
        let events = ctx.input(|i| i.events.clone());
        if let Some(mut index) = self.ruler_menu {
            let invalid = !self.rulers
                || self.backstage
                || ctx.input(|i| !i.focused || i.viewport().close_requested());
            if invalid {
                egui::Popup::close_id(ctx, Id::new("ruler-menu").with("popup"));
                self.ruler_menu = None;
                self.ruler_drag = None;
                self.ruler_key_focus = None;
                self.sync_view();
            } else {
                if ctx.input(|i| i.pointer.any_pressed()) {
                    self.ruler_drag = None;
                }
                for event in &events {
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } = event
                    {
                        match key {
                            Key::Escape => {
                                egui::Popup::close_id(ctx, Id::new("ruler-menu").with("popup"));
                                self.ruler_menu = None;
                                self.ruler_menu_activate = false;
                                break;
                            }
                            Key::Enter | Key::Space => self.ruler_menu_activate = true,
                            _ if self.ruler_drag.is_some() => {}
                            Key::ArrowUp => index = (index + 7) % 8,
                            Key::ArrowDown => index = (index + 1) % 8,
                            Key::Tab => index = (index + if modifiers.shift { 7 } else { 1 }) % 8,
                            Key::Home => index = 0,
                            Key::End => index = 7,
                            _ => {}
                        }
                    }
                }
                if self.ruler_menu.is_some() {
                    self.ruler_menu = Some(index);
                } else {
                    let id = MARKERS.iter().find(|(h, _)| *h == handle).unwrap().1;
                    ctx.memory_mut(|m| m.request_focus(Id::new(id)));
                }
            }
            ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
            ctx.input_mut(|i| {
                i.raw_scroll_delta = Vec2::ZERO;
                i.smooth_scroll_delta = Vec2::ZERO;
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
            });
            return true;
        }
        // The information view keeps marker focus and the uncommitted draft intact.
        // Dismissal consumes its key so Escape cannot also cancel that draft.
        if self.ruler_sources
            && self.rulers
            && !self.backstage
            && ctx
                .input(|i| i.focused && !i.viewport().close_requested() && !i.pointer.any_pressed())
        {
            let over_sources = ctx.input(|i| i.pointer.hover_pos()).is_some_and(|pos| {
                self.regions
                    .iter()
                    .any(|r| r.name == "ruler-sources" && r.rect.contains(pos))
            });
            for event in &events {
                if let egui::Event::MouseWheel {
                    unit,
                    delta,
                    modifiers,
                } = event
                    && over_sources
                    && modifiers.is_none()
                    && delta.y.is_finite()
                {
                    let scale = match unit {
                        egui::MouseWheelUnit::Point => 1.,
                        egui::MouseWheelUnit::Line => 24.,
                        egui::MouseWheelUnit::Page => (self.ruler_sources_height - 24.).max(24.),
                    };
                    self.ruler_sources_scroll -= delta.y * scale;
                }
                if let egui::Event::Key {
                    key, pressed: true, ..
                } = event
                {
                    match key {
                        Key::ArrowDown => self.ruler_sources_scroll += 24.,
                        Key::ArrowUp => self.ruler_sources_scroll -= 24.,
                        Key::PageDown => {
                            self.ruler_sources_scroll += (self.ruler_sources_height - 24.).max(24.)
                        }
                        Key::PageUp => {
                            self.ruler_sources_scroll -= (self.ruler_sources_height - 24.).max(24.)
                        }
                        Key::Home => self.ruler_sources_scroll = 0.,
                        Key::End => self.ruler_sources_scroll = f32::MAX,
                        _ => {}
                    }
                }
            }
            if events.iter().any(|e| {
                matches!(
                    e,
                    egui::Event::Key {
                        key: Key::F1 | Key::Escape | Key::Enter,
                        pressed: true,
                        ..
                    }
                )
            }) {
                self.ruler_sources = false;
            }
            ctx.memory_mut(|m| {
                m.move_focus(egui::FocusDirection::None);
                m.request_focus(Id::new(
                    MARKERS.iter().find(|(h, _)| *h == handle).unwrap().1,
                ));
            });
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
                })
            });
            return true;
        }
        let cancel = !self.rulers
            || self.backstage
            || ctx.input(|i| !i.focused || i.viewport().close_requested())
            || events.iter().any(|e| {
                matches!(
                    e,
                    egui::Event::Key {
                        key: Key::Escape,
                        pressed: true,
                        ..
                    }
                )
            });
        let pointer = ctx.input(|i| i.pointer.any_pressed());
        let lost =
            !enter && !ctx.memory(|m| MARKERS.iter().any(|(_, id)| m.has_focus(Id::new(id))));
        if cancel || pointer || lost {
            self.ruler_sources = false;
            self.ruler_drag = None;
            self.ruler_key_focus = None;
            if !pointer {
                self.sync_view();
            }
            if cancel {
                ctx.input_mut(|i| {
                    i.events.retain(|e| {
                        !matches!(
                            e,
                            egui::Event::Key { .. } | egui::Event::Text(_) | egui::Event::Paste(_)
                        )
                    })
                });
            }
            return !pointer;
        }
        ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
        // Explicit focus routing prevents Tab/arrows from reaching the editor or ribbon.
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
            })
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
                Key::F10 if modifiers == Modifiers::SHIFT => {
                    self.ruler_menu = Some(MARKERS.iter().position(|(h, _)| *h == handle).unwrap());
                    self.ruler_menu_activate = false;
                    egui::Popup::open_id(ctx, Id::new("ruler-menu").with("popup"));
                    break;
                }
                Key::F1 => {
                    self.ruler_sources_scroll = 0.;
                    self.ruler_sources = true;
                    break;
                }
                Key::Delete => self.start_ruler_reset(
                    ctx,
                    if modifiers.shift {
                        ResetIndent::for_handle(handle)
                    } else {
                        ResetIndent::All
                    },
                ),
                Key::ArrowLeft | Key::ArrowRight => {
                    if self.ruler_drag.as_ref().is_some_and(|d| d.draft.resetting) {
                        continue;
                    }
                    if self.ruler_drag.is_none() {
                        let rect = self
                            .regions
                            .iter()
                            .find(|r| r.name == "ruler")
                            .map(|r| r.rect)
                            .unwrap_or(Rect::NOTHING);
                        match Draft::new(&self.session.editor, handle) {
                            Ok(draft) => {
                                self.ruler_drag = Some(Drag {
                                    draft,
                                    origin: 0.,
                                    unit: 1.,
                                    rect,
                                    zoom: self.zoom / 100.,
                                    screen: ctx.content_rect(),
                                    keyboard: true,
                                })
                            }
                            Err(error) => {
                                self.error = Some(error);
                                continue;
                            }
                        }
                    }
                    if let Some(drag) = &mut self.ruler_drag {
                        let (index, _) = drag.draft.preview.locate(drag.draft.selection.0).unwrap();
                        let style = drag
                            .draft
                            .preview
                            .resolved_paragraph(&drag.draft.preview.paragraphs[index]);
                        let current = handle.position(style) - handle.position(drag.draft.style);
                        let step = if modifiers.shift { 10 } else { 1 };
                        let delta = current + if key == Key::ArrowLeft { -step } else { step };
                        if let Err(error) = drag.draft.move_by(delta as f32) {
                            self.error = Some(error);
                        }
                    }
                }
                Key::Enter | Key::Tab => {
                    if let Some(drag) = self.ruler_drag.take() {
                        let result = drag.draft.commit(&mut self.session.editor);
                        self.fail(result);
                    }
                    if key == Key::Tab {
                        let index = MARKERS.iter().position(|(h, _)| *h == handle).unwrap();
                        handle = MARKERS[(index + if modifiers.shift { 3 } else { 1 }) % 4].0;
                        self.ruler_key_focus = Some(handle);
                    }
                }
                Key::F6 => {
                    self.ruler_drag = None;
                    self.ruler_key_focus = None;
                    self.sync_view();
                    return true;
                }
                _ => {}
            }
        }
        if let Some((_, id)) = MARKERS.iter().find(|(h, _)| *h == handle) {
            ctx.memory_mut(|m| {
                if !m.has_focus(Id::new(id)) {
                    m.request_focus(Id::new(id));
                }
            });
        }
        true
    }
}

/// Both axes use the same physical scale. Only visible ticks are visited, including
/// for long documents. The text origin is zero; margins share its physical scale.
fn paint_scale(ui: &Ui, rect: Rect, start: f32, end: f32, zoom: f32, vertical: bool) {
    let painter = ui.painter();
    painter.rect_filled(rect, 0., Color32::from_gray(224));
    let body = if vertical {
        Rect::from_min_max(pos2(rect.left(), start), pos2(rect.right(), end))
    } else {
        Rect::from_min_max(pos2(start, rect.top()), pos2(end, rect.bottom()))
    };
    painter.rect_filled(body, 0., Color32::from_gray(252));
    painter.rect_stroke(
        rect,
        0.,
        Stroke::new(1.0_f32, Color32::from_gray(195)),
        egui::StrokeKind::Inside,
    );
    let visible = rect.intersect(ui.clip_rect());
    if !visible.is_positive() {
        return;
    }
    let (low, high) = if vertical {
        (visible.top(), visible.bottom())
    } else {
        (visible.left(), visible.right())
    };
    let step = 12. * zoom;
    let first = ((low - start) / step).ceil() as i32;
    let last = ((high - start) / step).floor() as i32;
    for tick in first..=last {
        let position = start + tick as f32 * step;
        let major = tick % 8 == 0;
        let middle = tick % 4 == 0;
        let length = if middle { 6. } else { 3. };
        let color = Color32::from_gray(100);
        if major && tick != 0 && position >= low + 7. && position <= high - 7. {
            let center = if vertical {
                pos2(rect.center().x, position)
            } else {
                pos2(position, rect.center().y)
            };
            painter.text(
                center,
                Align2::CENTER_CENTER,
                (tick.abs() / 8).to_string(),
                FontId::proportional(10.),
                color,
            );
        } else if !major {
            let points = if vertical {
                [
                    pos2(rect.center().x - length / 2., position),
                    pos2(rect.center().x + length / 2., position),
                ]
            } else {
                [
                    pos2(position, rect.center().y - length / 2.),
                    pos2(position, rect.center().y + length / 2.),
                ]
            };
            painter.line_segment(points, Stroke::new(1.0_f32, color));
        }
    }
}
