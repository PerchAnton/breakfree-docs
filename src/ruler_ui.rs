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
}
impl DocsApp {
    pub(super) fn prepare_ruler_input(&mut self, ctx: &egui::Context) {
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
            self.sync_view();
        }
    }

    pub(super) fn ruler(&mut self, ui: &mut Ui, width: f32, zoom: f32) {
        let (rect, _) = ui.allocate_exact_size(vec2(width, 36.), Sense::hover());
        self.record("ruler", rect);
        if self.ruler_drag.as_ref().is_some_and(|d| {
            d.rect != rect || d.zoom != zoom || d.screen != ui.ctx().content_rect()
        }) {
            self.ruler_drag = None;
            self.sync_view();
        }
        let margin = CANVAS_MARGIN * zoom;
        let left = rect.left() + margin;
        let right = rect.right() - margin;
        ui.painter().rect_filled(rect, 0., Color32::from_gray(214));
        ui.painter().rect_filled(
            Rect::from_min_max(pos2(left, rect.top()), pos2(right, rect.bottom())),
            0.,
            Color32::from_gray(250),
        );
        for tick in 0..=((right - left) / (12. * zoom)) as usize {
            let x = left + tick as f32 * 12. * zoom;
            let large = tick % 8 == 0;
            ui.painter().line_segment(
                [
                    pos2(x, rect.top() + 14.),
                    pos2(x, rect.top() + if large { 23. } else { 19. }),
                ],
                Stroke::new(1.0_f32, Color32::from_gray(115)),
            );
            if large {
                ui.painter().text(
                    pos2(x, rect.top() + 7.),
                    Align2::CENTER_CENTER,
                    (tick / 8).to_string(),
                    FontId::proportional(10.),
                    Color32::from_gray(90),
                );
            }
        }
        let model = self
            .ruler_drag
            .as_ref()
            .map(|d| &d.draft.preview)
            .unwrap_or(&self.session.editor.document);
        let Ok((index, _)) = model.locate(self.session.editor.selection.0) else {
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
                rect.top() + 5.,
            ),
            (
                Handle::Body,
                "ruler-body",
                "Continuation lines (hanging indent)",
                left + l + (-special).max(0.),
                rect.top() + 19.,
            ),
            (
                Handle::Left,
                "ruler-left",
                "Left indent (all lines)",
                left + l,
                rect.top() + 31.,
            ),
            (
                Handle::Right,
                "ruler-right",
                "Right indent",
                right - f32::from(style.right_indent_half_points) * unit,
                rect.top() + 19.,
            ),
        ];
        let enabled = self.enabled(Action::ParagraphSpacing) && !self.composing && !self.backstage;
        for (handle, name, label, x, y) in handles {
            let target = Rect::from_center_size(pos2(x, y), vec2(12., 10.));
            self.record(name, target);
            let response = ui.interact(
                target,
                Id::new(name),
                if enabled {
                    Sense::click_and_drag()
                } else {
                    Sense::hover()
                },
            );
            let active = self
                .ruler_drag
                .as_ref()
                .is_some_and(|d| d.draft.handle == handle);
            let fill = if active || response.hovered() {
                ACCENT
            } else {
                Color32::from_gray(245)
            };
            if handle == Handle::Left {
                ui.painter().rect(
                    target.shrink(1.),
                    1.,
                    fill,
                    Stroke::new(1.0_f32, INK),
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
                    Stroke::new(1.0_f32, INK),
                ));
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
                        });
                        ui.memory_mut(|m| m.surrender_focus(Id::new(EDIT_ID)));
                        self.focus_document = false;
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            response.on_hover_text(format!("{label} — drag to preview; release to apply; Esc cancels. Half-point steps. Uses the first selected paragraph; applies to selected paragraphs. Keyboard alternative: Home → Paragraph spacing."));
        }
        if let Some(drag) = &mut self.ruler_drag {
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
    }
}
