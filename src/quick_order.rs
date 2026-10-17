//! Toolbar-order draft. Only OK changes preferences; document state is never owned here.
use super::*;

pub(super) enum OrderOutcome {
    Open,
    Cancel,
    Apply(Vec<Action>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    List,
    Up,
    Down,
    Ok,
    Cancel,
}

pub(super) struct QuickOrderDialog {
    commands: Vec<Action>,
    selected: usize,
    focus: Focus,
}

impl QuickOrderDialog {
    pub fn new(commands: &[Action]) -> Self {
        Self {
            commands: commands.to_vec(),
            selected: 0,
            focus: if commands.is_empty() {
                Focus::Ok
            } else {
                Focus::List
            },
        }
    }

    fn can_move(&self, up: bool) -> bool {
        !self.commands.is_empty()
            && if up {
                self.selected > 0
            } else {
                self.selected + 1 < self.commands.len()
            }
    }

    fn move_selected(&mut self, up: bool) {
        if self.can_move(up) {
            let next = if up {
                self.selected - 1
            } else {
                self.selected + 1
            };
            self.commands.swap(self.selected, next);
            self.selected = next;
        }
        // Keep the same command selected and focus visible at either boundary.
        if (self.focus == Focus::Up && !self.can_move(true))
            || (self.focus == Focus::Down && !self.can_move(false))
        {
            self.focus = Focus::List;
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, regions: &mut Vec<Region>) -> OrderOutcome {
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            return OrderOutcome::Cancel;
        }
        for (key, up) in [(Key::ArrowUp, true), (Key::ArrowDown, false)] {
            if ctx.input_mut(|i| i.consume_key(Modifiers::ALT, key)) {
                self.move_selected(up);
            }
        }
        if self.focus == Focus::List && !self.commands.is_empty() {
            for (key, next) in [
                (Key::ArrowUp, self.selected.saturating_sub(1)),
                (
                    Key::ArrowDown,
                    (self.selected + 1).min(self.commands.len() - 1),
                ),
                (Key::Home, 0),
                (Key::End, self.commands.len() - 1),
            ] {
                if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, key)) {
                    self.selected = next;
                }
            }
        }
        for (modifiers, backwards) in [(Modifiers::SHIFT, true), (Modifiers::NONE, false)] {
            if ctx.input_mut(|i| i.consume_key(modifiers, Key::Tab)) {
                let stops: Vec<_> = [
                    Focus::List,
                    Focus::Up,
                    Focus::Down,
                    Focus::Ok,
                    Focus::Cancel,
                ]
                .into_iter()
                .filter(|f| match f {
                    Focus::List => !self.commands.is_empty(),
                    Focus::Up => self.can_move(true),
                    Focus::Down => self.can_move(false),
                    _ => true,
                })
                .collect();
                let index = stops.iter().position(|f| *f == self.focus).unwrap_or(0);
                self.focus =
                    stops[(index + if backwards { stops.len() - 1 } else { 1 }) % stops.len()];
            }
        }
        let mut accept = false;
        let mut cancel = false;
        let enter = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
        let space = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Space));
        if enter || space {
            match self.focus {
                Focus::Up => self.move_selected(true),
                Focus::Down => self.move_selected(false),
                Focus::Cancel => cancel = true,
                Focus::Ok => accept = true,
                Focus::List => accept = enter,
            }
        }
        let mut targets = Vec::new();
        let mut record = |name: &str, rect| {
            regions.push(Region {
                name: name.into(),
                rect,
            })
        };
        let id = Id::new("qat-order-dialog");
        let modal = egui::Modal::new(id)
            .area(egui::Modal::default_area(id).fade_in(false))
            .show(ctx, |ui| {
                ui.set_width(420.);
                ui.heading("Quick Access Toolbar");
                ui.separator();
                ui.label("Change the order of commands on the toolbar.");
                ui.small("Use the toolbar menu to add or remove commands.");
                ui.add_space(8.);
                ui.label("Commands on the toolbar:");
                ui.horizontal_top(|ui| {
                    egui::Frame::new()
                        .fill(Color32::WHITE)
                        .stroke(Stroke::new(1.0_f32, chrome::LINE))
                        .inner_margin(3)
                        .show(ui, |ui| {
                            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                                ui.set_width(368.);
                                ui.set_min_height(142.);
                                if self.commands.is_empty() {
                                    ui.weak("The toolbar is empty.");
                                }
                                for (index, action) in self.commands.clone().into_iter().enumerate()
                                {
                                    let (rect, _) =
                                        ui.allocate_exact_size(vec2(368., 26.), Sense::hover());
                                    // Command identity survives reordering; positional auto IDs do not.
                                    let response = ui.interact(
                                        rect,
                                        Id::new(("qat-order-row", action.label())),
                                        Sense::click(),
                                    );
                                    let selected = self.selected == index;
                                    response.widget_info(|| {
                                        egui::WidgetInfo::selected(
                                            egui::WidgetType::SelectableLabel,
                                            true,
                                            selected,
                                            format!("{}: {}", index + 1, action.label()),
                                        )
                                    });
                                    let visuals =
                                        ui.style().interact_selectable(&response, selected);
                                    if selected
                                        || response.hovered()
                                        || response.is_pointer_button_down_on()
                                    {
                                        ui.painter().rect_filled(rect, 0., visuals.weak_bg_fill);
                                    }
                                    if response.has_focus() {
                                        ui.painter().rect_stroke(
                                            rect.shrink(1.),
                                            0.,
                                            Stroke::new(1.0_f32, ACCENT),
                                            egui::StrokeKind::Inside,
                                        );
                                    }
                                    ui.painter().text(
                                        rect.left_center() + vec2(8., 0.),
                                        Align2::LEFT_CENTER,
                                        (index + 1).to_string(),
                                        FontId::proportional(13.),
                                        visuals.text_color(),
                                    );
                                    ui.painter().text(
                                        rect.left_center() + vec2(36., 0.),
                                        Align2::LEFT_CENTER,
                                        action.label(),
                                        FontId::proportional(13.),
                                        visuals.text_color(),
                                    );
                                    record(&format!("qat-order-{action:?}"), response.rect);
                                    if response.clicked() {
                                        self.selected = index;
                                        self.focus = Focus::List;
                                    }
                                    targets.push((Focus::List, Some(index), response.id));
                                }
                            });
                        });
                    ui.vertical(|ui| {
                        ui.set_width(30.);
                        for (up, focus, name) in [
                            (true, Focus::Up, "Move Up"),
                            (false, Focus::Down, "Move Down"),
                        ] {
                            let (rect, _) = ui.allocate_exact_size(vec2(30., 28.), Sense::hover());
                            let response = chrome::control(
                                ui,
                                rect,
                                Icon::Chevron(up),
                                "",
                                name,
                                self.can_move(up),
                                false,
                            )
                            .on_hover_text(format!(
                                "{name} (Alt+{})",
                                if up { "Up" } else { "Down" }
                            ))
                            .on_disabled_hover_text(
                                "Select a command that can move in this direction.",
                            );
                            ui.painter().rect_stroke(
                                rect,
                                2.,
                                Stroke::new(1.0_f32, chrome::LINE),
                                egui::StrokeKind::Inside,
                            );
                            record(
                                if up { "qat-order-up" } else { "qat-order-down" },
                                response.rect,
                            );
                            if response.clicked() {
                                self.focus = focus;
                                self.move_selected(up);
                            }
                            targets.push((focus, None, response.id));
                        }
                    });
                });
                ui.add_space(6.);
                ui.small("Up / Down select · Alt+Up / Alt+Down move");
                ui.small("Changes apply to all documents when you choose OK.");
                ui.add_space(10.);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    for (focus, name) in [(Focus::Cancel, "Cancel"), (Focus::Ok, "OK")] {
                        let response = ui.add_sized([82., 27.], Button::new(name));
                        record(
                            if focus == Focus::Ok {
                                "qat-order-ok"
                            } else {
                                "qat-order-cancel"
                            },
                            response.rect,
                        );
                        if response.clicked() {
                            self.focus = focus;
                            accept |= focus == Focus::Ok;
                            cancel |= focus == Focus::Cancel;
                        }
                        targets.push((focus, None, response.id));
                    }
                });
            });
        record("qat-order-dialog", modal.response.rect);
        if cancel {
            return OrderOutcome::Cancel;
        }
        if accept {
            return OrderOutcome::Apply(self.commands.clone());
        }
        if let Some((_, _, id)) = targets.iter().find(|(focus, index, _)| {
            *focus == self.focus && index.is_none_or(|i| i == self.selected)
        }) {
            ctx.memory_mut(|m| m.request_focus(*id));
        }
        OrderOutcome::Open
    }
}
