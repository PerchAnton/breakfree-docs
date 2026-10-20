//! Pointer adapter for explicit tab stops; existing indent controls keep priority.
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
}
impl DocsApp {
    pub(super) fn prepare_tab_ruler_input(&mut self, ctx: &egui::Context) -> bool {
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
    pub(super) fn tab_markers(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        left: f32,
        right: f32,
        zoom: f32,
        indent_targets: &[Rect],
    ) {
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
        let positions = model.paragraphs[first]
            .spacing
            .left_tab_stops_half_points
            .clone();
        let mixed = model.paragraphs[first..=last]
            .iter()
            .any(|p| p.spacing.left_tab_stops_half_points != positions);
        let unit = 14.6667 * zoom / 22.;
        let enabled = self.enabled(Action::TabPositions)
            && !self.composing
            && !self.backstage
            && self.ruler_key_focus.is_none();
        for value in positions {
            let x = left + f32::from(value) * unit;
            if x < left || x > right {
                continue;
            }
            let target = Rect::from_center_size(pos2(x, rect.top() + 20.), vec2(10., 8.));
            let collision = indent_targets.iter().any(|r| r.intersects(target));
            let available = enabled && !mixed && !collision;
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
                        });
                        self.focus_document = false;
                        self.leave_ribbon(ui.ctx());
                        ui.memory_mut(|m| m.surrender_focus(Id::new(EDIT_ID)));
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            response.on_hover_text(format!("Left tab: {:.1} pt. {}",f32::from(value)/2.,if mixed {"Different lists selected; edit them through Tab positions."} else if collision {"Overlaps an indent marker; use Tab positions to edit."} else {"Drag to preview; release to apply; Escape cancels. Release outside the ruler cancels. Use Tab positions for keyboard editing."}));
        }
        if let Some(drag) = &mut self.tab_ruler_drag {
            if let Some(pointer) = ui.input(|i| i.pointer.interact_pos()) {
                drag.error = drag
                    .draft
                    .move_to(f32::from(drag.start) + (pointer.x - drag.origin) / drag.unit)
                    .err();
                ui.ctx().request_repaint();
            }
            let text = drag.error.clone().unwrap_or_else(|| {
                format!(
                    "Left tab {:.1} pt — preview; Esc cancels",
                    f32::from(drag.draft.position) / 2.
                )
            });
            egui::Area::new(Id::new("tab-ruler-preview"))
                .order(egui::Order::Tooltip)
                .fixed_pos(pos2(rect.left() + 12., rect.bottom() + 4.))
                .interactable(false)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.label(text);
                    });
                });
            if ui.input(|i| i.pointer.primary_released()) {
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
