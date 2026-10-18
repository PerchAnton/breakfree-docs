//! Native adapter for the shared ruler draft; no model mutations during motion.
use super::*;
use crate::ruler::{Draft, Handle, indent_scale};

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
        if self.ruler_keyboard_input(ctx) {
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
        let menu_response = ui.interact(rect, Id::new("ruler-menu"), Sense::click());
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
        let margin = CANVAS_MARGIN * zoom;
        let left = rect.left() + margin;
        let right = rect.right() - margin;
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
        for (handle, name, label, x, y) in handles {
            let target = Rect::from_center_size(
                pos2(x, y),
                vec2(12., if handle == Handle::Left { 4. } else { 8. }),
            );
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
                if !response.has_focus() {
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
                    format!("{label}. {}", summary.marker_details(handle)),
                )
            });
            response.on_hover_text(format!("{label} — drag to preview; release to apply; Esc cancels. Half-point steps. Uses the first selected paragraph; applies to selected paragraphs. Ctrl+Shift+F8 enters keyboard control; Tab changes marker, arrows nudge 0.5 pt, Shift+arrows 5 pt, Enter applies, Esc cancels, Delete previews reset to style indents. {}", summary.marker_details(handle)));
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
        if self.ruler_key_focus.is_some() || self.ruler_drag.is_some() {
            let hint = self
                .ruler_key_focus
                .or_else(|| self.ruler_drag.as_ref().map(|d| d.draft.handle))
                .map(marker_name)
                .unwrap_or("Indent");
            let hint = if self.ruler_drag.as_ref().is_some_and(|d| d.draft.resetting) {
                "Reset to style"
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
            let galley = painter.layout_no_wrap(
                format!("{}\n{hint} · Enter applies · Esc cancels", summary.values()),
                FontId::proportional(11.),
                INK,
            );
            let readout = Rect::from_min_size(origin, galley.size() + vec2(12., 8.));
            self.record("ruler-values", readout);
            painter.rect(
                readout,
                2.,
                Color32::from_rgb(255, 255, 245),
                Stroke::new(1.0_f32, Color32::from_gray(180)),
                egui::StrokeKind::Inside,
            );
            painter.galley(origin + vec2(6., 4.), galley, INK);
        }
        menu_response.on_hover_text("Right-click for Reset indents to style. Ctrl+Shift+F8 enters markers; Delete previews reset; Enter applies; Esc cancels.")
            .context_menu(|ui| {
                let reset = ui.add_enabled(enabled && self.ruler_drag.is_none(), Button::new("Reset indents to style"));
                self.record("ruler-reset", reset.rect);
                if reset.clicked() {
                    self.start_ruler_reset(ui.ctx());
                    ui.close();
                }
            });
    }

    fn start_ruler_reset(&mut self, ctx: &egui::Context) {
        let handle = self.ruler_key_focus.unwrap_or(Handle::Left);
        match Draft::new(&self.session.editor, handle).and_then(|mut draft| {
            draft.reset_indents()?;
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
        paint_scale(
            ui,
            rect,
            canvas.top() + CANVAS_MARGIN * zoom,
            canvas.bottom() - CANVAS_MARGIN * zoom,
            zoom,
            true,
        );
        ui.interact(rect.intersect(ui.clip_rect()), Id::new("ruler-vertical"), Sense::hover())
            .on_hover_text("Vertical ruler · inches from the continuous canvas text origin. Follows document scrolling. Page margins and pagination are not implemented.");
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
            return false;
        };
        if self.ruler_drag.as_ref().is_some_and(|d| !d.keyboard) {
            return false;
        }
        let events = ctx.input(|i| i.events.clone());
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
                Key::Delete => self.start_ruler_reset(ctx),
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
