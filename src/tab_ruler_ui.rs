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
    numeric: bool,
    input: String,
    focus_input: bool,
    accept_input: bool,
    popup: Rect,
}
impl DocsApp {
    pub(super) fn prepare_tab_ruler_input(&mut self, ctx: &egui::Context) -> bool {
        let eligible = self.tab_ruler_drag.is_none()
            && self.enabled(Action::TabPositions)
            && !self.composing
            && !self.backstage
            && self.ruler_key_focus.is_none()
            && !egui::Popup::is_any_open(ctx);
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
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F2)) {
                let drag = self.tab_ruler_drag.as_mut().unwrap();
                drag.numeric = true;
                drag.input = format!("{}", f32::from(drag.draft.position) / 2.);
                drag.focus_input = true;
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
                            | egui::Event::PointerButton { .. }
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
        let cancel = ctx.input(|i| {
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
            crate::tab_ruler::Draft::creation_preview(&self.session.editor)
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
                    numeric: false,
                    input: String::new(),
                    focus_input: false,
                    accept_input: false,
                    popup: Rect::NOTHING,
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
        let positions = crate::tab_ruler::positions(model, (start, end)).unwrap_or_default();
        let total = last - first + 1;
        let unit = 14.6667 * zoom / 22.;
        let enabled = self.enabled(Action::TabPositions)
            && !self.composing
            && !self.backstage
            && self.ruler_key_focus.is_none();
        for (value, coverage) in positions {
            let mixed = coverage != total;
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
            ui.painter().line_segment(
                [pos2(x, rect.top() + 17.), pos2(x, rect.top() + 23.)],
                Stroke::new(2.0_f32, color),
            );
            ui.painter().line_segment(
                [pos2(x, rect.top() + 23.), pos2(x + 5., rect.top() + 23.)],
                Stroke::new(2.0_f32, color),
            );
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
                            numeric: false,
                            input: String::new(),
                            focus_input: false,
                            accept_input: false,
                            popup: Rect::NOTHING,
                        });
                        self.focus_document = false;
                        self.leave_ribbon(ui.ctx());
                        ui.memory_mut(|m| m.surrender_focus(Id::new(EDIT_ID)));
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            response.context_menu(|ui| {
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
            });
            response.on_hover_text(format!("Left tab: {:.1} pt. Present in {coverage}/{total} selected paragraphs. Movement/deletion affects only paragraphs containing this stop. {}",f32::from(value)/2.,if collision {"Overlaps an indent marker; use Ctrl+Alt+F8 or Tab positions to edit."} else {"Drag to preview; release to apply; Escape cancels. Release outside the ruler cancels. Ctrl+Alt+F8 selects tabs; Ctrl+Alt+Shift+F8 creates; arrows move, Shift accelerates, Enter applies, Tab applies and selects next, Delete previews removal, Escape cancels. Right-click to delete."}));
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
                            "New tab {:.1} pt — Enter applies; Esc cancels",
                            f32::from(drag.draft.position) / 2.
                        );
                    }
                    if drag.draft.deleting {
                        return format!(
                            "Delete tab {:.1} pt — Enter applies; Esc cancels",
                            f32::from(drag.start) / 2.
                        );
                    }
                    format!(
                        "Left tab {:.1} pt — preview; {}; Esc cancels",
                        f32::from(drag.draft.position) / 2.,
                        if drag.keyboard {
                            "Enter applies"
                        } else {
                            "release applies"
                        }
                    )
                });
            let popup = egui::Area::new(Id::new("tab-ruler-preview"))
                .order(egui::Order::Tooltip)
                .fixed_pos(pos2(rect.left() + 12., rect.bottom() + 4.))
                .interactable(drag.keyboard)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_max_width((ui.ctx().content_rect().width() - 48.).max(120.));
                        if drag.numeric {
                            ui.label(if drag.draft.is_creation() {
                                "New tab"
                            } else {
                                "Left tab"
                            });
                            ui.horizontal(|ui| {
                                ui.label("Position:");
                                let mut out = egui::TextEdit::singleline(&mut drag.input)
                                    .id(Id::new("ruler-tab-position-input"))
                                    .desired_width(90.)
                                    .char_limit(16)
                                    .show(ui);
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
                            ui.small("0.5 pt steps · Enter applies · Esc cancels");
                            if let Some(error) = &drag.error {
                                ui.add(
                                    egui::Label::new(RichText::new(error).color(Color32::DARK_RED))
                                        .wrap(),
                                );
                            }
                        } else {
                            ui.add(egui::Label::new(text).wrap());
                            if drag.keyboard {
                                ui.small("F2: enter an exact position in points");
                            }
                        }
                    });
                });
            drag.popup = popup.response.rect;
            if drag.numeric && drag.accept_input && drag.error.is_none() {
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
                if inside && drag.error.is_none() {
                    let result = drag.draft.commit(&mut self.session.editor);
                    self.fail(result);
                }
                self.sync_view();
            }
        }
    }
}
