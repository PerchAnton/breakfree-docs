//! Original vector controls for the native shell; no document or command state.
use eframe::egui::{
    self, Align2, Color32, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, pos2, vec2,
};

pub const INK: Color32 = Color32::from_rgb(50, 49, 48);
pub const ACCENT: Color32 = Color32::from_rgb(112, 71, 174);
pub const LINE: Color32 = Color32::from_rgb(213, 213, 213);

#[derive(Clone, Copy)]
pub enum Icon {
    Back,
    Save,
    Undo,
    Redo,
    Paste,
    Cut,
    Copy,
    Brush,
    Search,
    Replace,
    Select,
    Align(u8),
    Bullets,
    Numbers,
    Multilevel,
    Sort,
    FontStep(bool),
    ClearFormatting,
    Highlight,
    FontColor,
    TextEffects,
    Indent(bool),
    Spacing,
    Border,
    Shading,
    Chevron(bool),
    Launch,
    Minimize,
    Maximize(bool),
    Close,
    Document,
    Open,
    Text(&'static str),
}

pub fn paint(ui: &Ui, icon: Icon, center: Pos2, size: f32, color: Color32) {
    let painter = ui.painter();
    let scale = size / 20.;
    let p = |x: f32, y: f32| center + vec2(x, y) * scale;
    let stroke = Stroke::new(1.15_f32, color);
    let line = |a: (f32, f32), b: (f32, f32)| {
        painter.line_segment([p(a.0, a.1), p(b.0, b.1)], stroke);
    };
    let rect = |a: (f32, f32), b: (f32, f32)| {
        painter.rect_stroke(
            Rect::from_min_max(p(a.0, a.1), p(b.0, b.1)),
            1.,
            stroke,
            egui::StrokeKind::Inside,
        );
    };
    match icon {
        Icon::Back => {
            line((-7., 0.), (7., 0.));
            line((-7., 0.), (-1., -6.));
            line((-7., 0.), (-1., 6.));
        }
        Icon::Save => {
            rect((-7., -8.), (7., 8.));
            rect((-4., -8.), (4., -1.));
            rect((-4., 2.), (4., 8.));
            line((2., -6.), (2., -3.));
        }
        Icon::Undo | Icon::Redo => {
            let d = if matches!(icon, Icon::Undo) { 1. } else { -1. };
            painter.add(egui::Shape::line(
                vec![
                    p(6. * d, 6.),
                    p(6. * d, 0.),
                    p(2. * d, -4.),
                    p(-7. * d, -4.),
                ],
                stroke,
            ));
            line((-3. * d, -8.), (-7. * d, -4.));
            line((-3. * d, 0.), (-7. * d, -4.));
        }
        Icon::Paste => {
            rect((-7., -6.), (6., 8.));
            rect((-3., -9.), (3., -4.));
            painter.rect_filled(
                Rect::from_min_max(p(-1., -1.), p(9., 10.)),
                1.,
                Color32::WHITE,
            );
            rect((-1., -1.), (9., 10.));
            line((2., 3.), (6., 3.));
            line((2., 6.), (6., 6.));
        }
        Icon::Cut => {
            painter.circle_stroke(p(-5., 5.), 3. * scale, stroke);
            painter.circle_stroke(p(5., 5.), 3. * scale, stroke);
            line((-4., 3.), (6., -8.));
            line((4., 3.), (-6., -8.));
        }
        Icon::Copy => {
            rect((-7., -8.), (3., 5.));
            painter.rect_filled(
                Rect::from_min_max(p(-2., -4.), p(8., 9.)),
                1.,
                Color32::WHITE,
            );
            rect((-2., -4.), (8., 9.));
        }
        Icon::Brush => {
            rect((-8., -8.), (8., -3.));
            line((5., -3.), (5., 0.));
            line((5., 0.), (0., 0.));
            rect((-2., 0.), (2., 9.));
        }
        Icon::Search => {
            painter.circle_stroke(p(-2., -2.), 5.5 * scale, stroke);
            line((2., 2.), (8., 8.));
        }
        Icon::Replace => {
            line((-7., -5.), (7., -5.));
            line((7., -5.), (3., -8.));
            line((7., 5.), (-7., 5.));
            line((-7., 5.), (-3., 8.));
            painter.text(
                center,
                Align2::CENTER_CENTER,
                "ab",
                FontId::proportional(8. * scale),
                color,
            );
        }
        Icon::Select => {
            painter.add(egui::Shape::closed_line(
                vec![p(-5., -8.), p(7., 1.), p(1., 2.), p(-2., 8.)],
                stroke,
            ));
        }
        Icon::Align(kind) => {
            for (i, len) in [16., 10., 16., 12.].into_iter().enumerate() {
                let len = if kind == 3 { 16. } else { len };
                let x = match kind {
                    1 => -len / 2.,
                    2 => 8. - len,
                    _ => -8.,
                };
                line((x, -6. + i as f32 * 4.), (x + len, -6. + i as f32 * 4.));
            }
        }
        Icon::Bullets | Icon::Numbers => {
            for i in 0..3 {
                let y = -6. + i as f32 * 6.;
                line((-2., y), (8., y));
                if matches!(icon, Icon::Bullets) {
                    painter.circle_filled(p(-6., y), 1.2 * scale, color);
                } else {
                    painter.text(
                        p(-6., y),
                        Align2::CENTER_CENTER,
                        (i + 1).to_string(),
                        FontId::proportional(6. * scale),
                        color,
                    );
                }
            }
        }
        Icon::Indent(increase) => {
            for y in [-7., 7.] {
                line((-8., y), (8., y));
            }
            for y in [-3., 1.] {
                line((0., y), (8., y));
            }
            let d = if increase { 1. } else { -1. };
            line((-5. - 2. * d, -3.), (-5. + 2. * d, 0.));
            line((-5. + 2. * d, 0.), (-5. - 2. * d, 3.));
        }
        Icon::Multilevel => {
            for (i, label) in ["1", "a", "i"].into_iter().enumerate() {
                let x = -7. + i as f32 * 3.;
                let y = -6. + i as f32 * 6.;
                painter.text(
                    p(x, y),
                    Align2::CENTER_CENTER,
                    label,
                    FontId::proportional(7. * scale),
                    color,
                );
                line((x + 4., y), (9., y));
            }
        }
        Icon::Sort => {
            for (label, y) in [("A", -5.), ("Z", 5.)] {
                painter.text(
                    p(-4., y),
                    Align2::CENTER_CENTER,
                    label,
                    FontId::proportional(9. * scale),
                    color,
                );
            }
            line((6., -7.), (6., 7.));
            line((3., 4.), (6., 7.));
            line((6., 7.), (9., 4.));
        }
        Icon::FontStep(grow) => {
            painter.text(
                p(-3., 1.),
                Align2::CENTER_CENTER,
                "A",
                FontId::proportional(if grow { 19. } else { 15. } * scale),
                color,
            );
            let d = if grow { -1. } else { 1. };
            painter.add(egui::Shape::convex_polygon(
                vec![
                    p(4., -5. - 2. * d),
                    p(10., -5. - 2. * d),
                    p(7., -5. + 2. * d),
                ],
                color,
                Stroke::NONE,
            ));
        }
        Icon::ClearFormatting => {
            painter.text(
                p(-3., -2.),
                Align2::CENTER_CENTER,
                "A",
                FontId::proportional(17. * scale),
                color,
            );
            painter.add(egui::Shape::convex_polygon(
                vec![p(2., 1.), p(8., 5.), p(5., 9.), p(-1., 5.)],
                Color32::WHITE,
                stroke,
            ));
            line((2., 7.), (5., 3.));
        }
        Icon::Highlight => {
            painter.add(egui::Shape::closed_line(
                vec![p(1., -7.), p(7., -3.), p(1., 5.), p(-5., 1.)],
                stroke,
            ));
            line((-5., 1.), (-7., 6.));
            line((-7., 6.), (-2., 4.));
            painter.line_segment([p(-8., 9.), p(8., 9.)], Stroke::new(2.0_f32, color));
        }
        Icon::FontColor | Icon::TextEffects => {
            painter.text(
                p(0., -1.),
                Align2::CENTER_CENTER,
                "A",
                FontId::proportional(18. * scale),
                color,
            );
            if matches!(icon, Icon::FontColor) {
                painter.line_segment([p(-8., 9.), p(8., 9.)], Stroke::new(2.0_f32, color));
            } else {
                line((-6., 8.), (-2., -6.));
            }
        }
        Icon::Spacing => {
            for y in [-6., 0., 6.] {
                line((0., y), (8., y));
            }
            line((-5., -8.), (-5., 8.));
            for d in [-1., 1.] {
                line((-8., 5. * d), (-5., 8. * d));
                line((-2., 5. * d), (-5., 8. * d));
            }
        }
        Icon::Border => {
            rect((-7., -7.), (7., 7.));
            line((-7., 0.), (7., 0.));
            line((0., -7.), (0., 7.));
        }
        Icon::Shading => {
            painter.add(egui::Shape::closed_line(
                vec![p(0., -8.), p(8., 0.), p(0., 8.), p(-8., 0.)],
                stroke,
            ));
            line((-6., 1.), (6., 1.));
        }
        Icon::Chevron(up) => {
            let d = if up { -1. } else { 1. };
            line((-4., -2. * d), (0., 2. * d));
            line((0., 2. * d), (4., -2. * d));
        }
        Icon::Launch => {
            line((-6., -5.), (5., -5.));
            line((-6., -5.), (-6., 6.));
            line((-1., -1.), (6., 6.));
            line((6., 1.), (6., 6.));
            line((1., 6.), (6., 6.));
        }
        Icon::Minimize => line((-5., 3.), (5., 3.)),
        Icon::Maximize(restore) => {
            if restore {
                rect((-3., -6.), (7., 4.));
            }
            rect((-6., -3.), (4., 7.));
        }
        Icon::Close => {
            line((-5., -5.), (5., 5.));
            line((5., -5.), (-5., 5.));
        }
        Icon::Document => {
            rect((-7., -9.), (7., 9.));
            for y in [-4., 0., 4.] {
                line((-4., y), (4., y));
            }
        }
        Icon::Open => {
            rect((-8., -4.), (8., 7.));
            line((-8., -4.), (-8., -7.));
            line((-8., -7.), (-2., -7.));
            line((-2., -7.), (1., -4.));
        }
        Icon::Text(text) => {
            let font = FontId::new(
                size,
                egui::FontFamily::Name(if text == "B" { "body-bold" } else { "body" }.into()),
            );
            let mut job = egui::text::LayoutJob::default();
            job.append(
                text,
                0.,
                egui::TextFormat {
                    font_id: font,
                    italics: text == "I",
                    color,
                    ..Default::default()
                },
            );
            let galley = ui.fonts_mut(|f| f.layout_job(job));
            painter.galley(center - galley.size() / 2., galley, color);
            if text == "U" {
                line((-5., 8.), (5., 8.));
            }
            if text == "abc" {
                line((-8., 1.), (8., 1.));
            }
        }
    }
}

/// A real keyboard-focusable button with vector contents and an accessible name.
/// Native egui owns pointer/focus behavior; selected/disabled states are explicit.
pub fn control(
    ui: &mut Ui,
    rect: Rect,
    icon: Icon,
    label: &str,
    name: &str,
    enabled: bool,
    selected: bool,
) -> Response {
    let response = button(ui, rect, name, enabled, selected);
    let color = if enabled {
        INK
    } else {
        Color32::from_gray(150)
    };
    if rect.height() > 50. && rect.width() > 40. {
        paint(
            ui,
            icon,
            pos2(rect.center().x, rect.top() + 22.),
            28.,
            color,
        );
        ui.painter().text(
            pos2(rect.center().x, rect.bottom() - 17.),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(12.),
            color,
        );
    } else if label.is_empty() {
        paint(ui, icon, rect.center(), 16., color);
    } else {
        paint(
            ui,
            icon,
            pos2(rect.left() + 12., rect.center().y),
            16.,
            color,
        );
        ui.painter()
            .with_clip_rect(rect.shrink2(vec2(2., 0.)))
            .text(
                pos2(rect.left() + 26., rect.center().y),
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(12.),
                color,
            );
    }
    response
}

/// Menu affordance with a dedicated chevron lane, including disabled commands.
pub fn dropdown(
    ui: &mut Ui,
    rect: Rect,
    icon: Icon,
    label: &str,
    name: &str,
    enabled: bool,
) -> Response {
    let response = button(ui, rect, name, enabled, false);
    let color = if enabled {
        INK
    } else {
        Color32::from_gray(150)
    };
    let body = Rect::from_min_max(rect.min, pos2(rect.right() - 10., rect.bottom()));
    paint(
        ui,
        icon,
        if label.is_empty() {
            body.center()
        } else {
            body.left_center() + vec2(12., 0.)
        },
        16.,
        color,
    );
    if !label.is_empty() {
        ui.painter()
            .with_clip_rect(body.shrink2(vec2(2., 0.)))
            .text(
                body.left_center() + vec2(26., 0.),
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(12.),
                color,
            );
    }
    paint(
        ui,
        Icon::Chevron(false),
        rect.right_center() - vec2(5., 0.),
        8.,
        color,
    );
    response
}

pub fn button(ui: &mut Ui, rect: Rect, name: &str, enabled: bool, selected: bool) -> Response {
    let response = ui
        .scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            if !enabled {
                ui.disable();
            }
            ui.spacing_mut().interact_size = rect.size();
            ui.spacing_mut().button_padding = vec2(0., 0.);
            ui.put(
                rect,
                // The visible contents and accessible name are supplied below.
                // An empty default label still reserves the platform font's line
                // height (e.g. Segoe UI at 125%), enlarging small icon controls.
                egui::Button::new(egui::RichText::new("").size(1.0))
                    .frame_when_inactive(selected)
                    .selected(selected)
                    .corner_radius(3),
            )
        })
        .inner;
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, enabled, selected, name)
    });
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect.shrink(1.),
            3.,
            Stroke::new(1.0_f32, ACCENT),
            egui::StrokeKind::Inside,
        );
    }
    response
}

pub fn field(ui: &mut Ui, rect: Rect, label: &str, explanation: &str) -> Response {
    ui.painter().rect_filled(rect, 2., Color32::WHITE);
    ui.painter().rect_stroke(
        rect,
        2.,
        Stroke::new(1.0_f32, LINE),
        egui::StrokeKind::Inside,
    );
    let painter = ui.painter().with_clip_rect(Rect::from_min_max(
        rect.min + vec2(6., 0.),
        rect.max - vec2(18., 0.),
    ));
    painter.text(
        rect.left_center() + vec2(6., 0.),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.),
        INK,
    );
    paint(
        ui,
        Icon::Chevron(false),
        rect.right_center() - vec2(9., 0.),
        12.,
        Color32::from_gray(150),
    );
    ui.allocate_rect(rect, Sense::hover())
        .on_hover_text(explanation)
}
