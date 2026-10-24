//! Original vector controls for the native shell; no document or command state.
use eframe::egui::{
    self, Align2, Color32, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, pos2, vec2,
};

pub const INK: Color32 = Color32::from_rgb(45, 45, 45);
pub const ACCENT: Color32 = Color32::from_rgb(112, 71, 174);
pub const LINE: Color32 = Color32::from_rgb(210, 210, 210);
pub const CHROME: Color32 = Color32::from_rgb(245, 245, 245);
pub const SURFACE: Color32 = Color32::from_rgb(245, 245, 245);
pub const WORKSPACE: Color32 = Color32::from_rgb(235, 235, 235);
pub const FILE_RAIL: Color32 = Color32::from_rgb(235, 237, 239);
pub const HOVER: Color32 = Color32::from_rgb(226, 226, 226);
pub const PRESSED: Color32 = Color32::from_rgb(207, 207, 207);
pub const SELECTION: Color32 = Color32::from_rgb(237, 232, 245);

// Small semantic accents, independent of the brand's focus/selection color.
const BLUE: Color32 = Color32::from_rgb(48, 126, 166);
const GOLD: Color32 = Color32::from_rgb(222, 158, 55);
const PALE_GOLD: Color32 = Color32::from_rgb(255, 245, 214);
const YELLOW: Color32 = Color32::from_rgb(255, 224, 0);
const RED: Color32 = Color32::from_rgb(224, 25, 36);
const ERASER: Color32 = Color32::from_rgb(157, 81, 171);

fn tint(color: Color32, enabled: bool) -> Color32 {
    if enabled {
        color
    } else {
        // Muted color remains recognizable without suggesting availability.
        let channel = |c: u8| ((2 * u16::from(c) + 3 * 245) / 5) as u8;
        Color32::from_rgb(channel(color.r()), channel(color.g()), channel(color.b()))
    }
}

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
    CanvasWidth,
    Document,
    Open,
    Home,
    Settings,
    Print,
    Text(&'static str),
}

pub fn paint(ui: &Ui, icon: Icon, center: Pos2, size: f32, color: Color32, enabled: bool) {
    let painter = ui.painter();
    let scale = size / 20.;
    let p = |x: f32, y: f32| center + vec2(x, y) * scale;
    // Clipboard and file actions use a lighter, finer outline than text-format tools.
    let quiet = matches!(icon, Icon::Save | Icon::Paste | Icon::Cut | Icon::Copy);
    let color = if quiet {
        Color32::from_gray(if !enabled {
            178
        } else if matches!(icon, Icon::Save) {
            118
        } else {
            105
        })
    } else {
        color
    };
    let stroke = Stroke::new(if quiet { 1.0_f32 } else { 1.15_f32 }, color);
    let accent = |c| tint(c, enabled);
    let colored_line = |a: (f32, f32), b: (f32, f32), c| {
        painter.line_segment([p(a.0, a.1), p(b.0, b.1)], Stroke::new(1.15_f32, accent(c)));
    };
    let panel = |a: (f32, f32), b: (f32, f32), fill, edge| {
        painter.rect(
            Rect::from_min_max(p(a.0, a.1), p(b.0, b.1)),
            1.,
            accent(fill),
            Stroke::new(1.0_f32, accent(edge)),
            egui::StrokeKind::Inside,
        );
    };
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
        Icon::CanvasWidth => {
            line((-8., -7.), (-8., 7.));
            line((8., -7.), (8., 7.));
            line((-5., 0.), (5., 0.));
            line((-5., 0.), (-2., -3.));
            line((-5., 0.), (-2., 3.));
            line((5., 0.), (2., -3.));
            line((5., 0.), (2., 3.));
        }
        Icon::Back => {
            line((-7., 0.), (7., 0.));
            line((-7., 0.), (-1., -6.));
            line((-7., 0.), (-1., 6.));
        }
        Icon::Save => {
            // Clipped disk corner, recessed top slot and open label window.
            painter.add(egui::Shape::closed_line(
                vec![
                    p(-6.5, -7.),
                    p(3.5, -7.),
                    p(6.5, -4.),
                    p(6.5, 7.),
                    p(-6.5, 7.),
                ],
                stroke,
            ));
            line((-3.5, -7.), (-3.5, -2.));
            line((-3.5, -2.), (2.5, -2.));
            line((2.5, -2.), (2.5, -7.));
            line((-3.5, 7.), (-3.5, 1.));
            line((-3.5, 1.), (3.5, 1.));
            line((3.5, 1.), (3.5, 7.));
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
            // Broad board and inset paper, with a full-size sheet offset to the right.
            panel(
                (-8., -7.),
                (5., 9.),
                Color32::from_rgb(255, 251, 243),
                Color32::from_rgb(218, 164, 79),
            );
            painter.rect_filled(
                Rect::from_min_max(p(-6., -5.), p(3., 7.)),
                0.,
                Color32::WHITE,
            );
            painter.rect_filled(Rect::from_min_max(p(-5.5, -7.), p(2.5, -4.5)), 0., SURFACE);
            line((-5.5, -4.5), (-5.5, -7.));
            line((-5.5, -7.), (-3., -7.));
            painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
                [p(-3., -7.), p(-3., -10.5), p(0.5, -10.5), p(0.5, -7.)],
                false,
                Color32::TRANSPARENT,
                stroke,
            ));
            line((0.5, -7.), (2.5, -7.));
            line((2.5, -7.), (2.5, -4.5));
            line((2.5, -4.5), (-5.5, -4.5));
            painter.rect_filled(
                Rect::from_min_max(p(0., -2.5), p(8.5, 10.)),
                0.,
                Color32::WHITE,
            );
            painter.rect_stroke(
                Rect::from_min_max(p(0., -2.5), p(8.5, 10.)),
                0.5,
                Stroke::new(1.15_f32, color),
                egui::StrokeKind::Inside,
            );
        }
        Icon::Cut => {
            let handles = Stroke::new(1.0_f32, accent(Color32::from_rgb(85, 145, 173)));
            line((-2.4, 4.2), (3.5, -8.));
            line((2.4, 4.2), (-3.5, -8.));
            painter.circle_stroke(p(-3., 6.), 2.2 * scale, handles);
            painter.circle_stroke(p(3., 6.), 2.2 * scale, handles);
        }
        Icon::Copy => {
            // Offset sheets with a folded front corner, drawn as separate outlines.
            rect((-7., -8.), (2., 5.));
            painter.add(egui::Shape::convex_polygon(
                vec![p(-2., -4.), p(4., -4.), p(8., 0.), p(8., 9.), p(-2., 9.)],
                Color32::WHITE,
                stroke,
            ));
            line((4., -4.), (4., 0.));
            line((4., 0.), (8., 0.));
            line((1., 3.), (5., 3.));
            line((1., 6.), (5., 6.));
        }
        Icon::Brush => {
            painter.add(egui::Shape::convex_polygon(
                vec![p(4., -8.), p(8., -4.), p(0., 4.), p(-4., 0.)],
                accent(PALE_GOLD),
                Stroke::new(1.0_f32, accent(GOLD)),
            ));
            painter.add(egui::Shape::convex_polygon(
                vec![p(-4., 0.), p(0., 4.), p(-5., 9.), p(-9., 5.)],
                accent(PALE_GOLD),
                stroke,
            ));
            line((-6., 2.), (-2., 6.));
            colored_line((-8., 5.), (-5., 8.), GOLD);
        }
        Icon::Search => {
            painter.circle_stroke(p(-2., -2.), 5.5 * scale, stroke);
            line((2., 2.), (8., 8.));
        }
        Icon::Replace => {
            colored_line((-7., -5.), (7., -5.), BLUE);
            colored_line((7., -5.), (3., -8.), BLUE);
            colored_line((7., 5.), (-7., 5.), BLUE);
            colored_line((-7., 5.), (-3., 8.), BLUE);
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
                    painter.circle_filled(p(-6., y), 1.2 * scale, accent(BLUE));
                } else {
                    painter.text(
                        p(-6., y),
                        Align2::CENTER_CENTER,
                        (i + 1).to_string(),
                        FontId::proportional(6. * scale),
                        accent(BLUE),
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
            colored_line((-5. - 2. * d, -3.), (-5. + 2. * d, 0.), BLUE);
            colored_line((-5. + 2. * d, 0.), (-5. - 2. * d, 3.), BLUE);
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
                    accent(BLUE),
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
                    accent(BLUE),
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
                accent(BLUE),
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
                accent(ERASER),
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
            painter.line_segment(
                [p(-8., 9.), p(8., 9.)],
                Stroke::new(3.0_f32, accent(YELLOW)),
            );
        }
        Icon::FontColor | Icon::TextEffects => {
            painter.text(
                p(0., -1.),
                Align2::CENTER_CENTER,
                "A",
                FontId::proportional(18. * scale),
                if matches!(icon, Icon::TextEffects) {
                    accent(BLUE)
                } else {
                    color
                },
            );
            if matches!(icon, Icon::FontColor) {
                painter.line_segment([p(-8., 9.), p(8., 9.)], Stroke::new(3.0_f32, accent(RED)));
            } else {
                colored_line((-6., 8.), (-2., -6.), BLUE);
            }
        }
        Icon::Spacing => {
            for y in [-6., 0., 6.] {
                line((0., y), (8., y));
            }
            colored_line((-5., -8.), (-5., 8.), BLUE);
            for d in [-1., 1.] {
                colored_line((-8., 5. * d), (-5., 8. * d), BLUE);
                colored_line((-2., 5. * d), (-5., 8. * d), BLUE);
            }
        }
        Icon::Border => {
            // A dotted outer grid separates this from the window/table controls.
            for offset in [-7., -3.5, 0., 3.5, 7.] {
                for point in [p(offset, -7.), p(offset, 7.), p(-7., offset), p(7., offset)] {
                    painter.circle_filled(point, 0.65 * scale, color);
                }
            }
            line((-7., 0.), (7., 0.));
            line((0., -7.), (0., 7.));
        }
        Icon::Shading => {
            painter.add(egui::Shape::closed_line(
                vec![p(-1., -7.), p(5., -1.), p(-2., 6.), p(-8., 0.)],
                stroke,
            ));
            line((-6., 1.), (2., 1.));
            line((-2., -8.), (-2., -2.));
            painter.add(egui::Shape::convex_polygon(
                vec![p(7., -1.), p(9., 3.), p(7., 5.), p(5., 3.)],
                accent(BLUE),
                Stroke::NONE,
            ));
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
            panel((-7., -9.), (7., 9.), Color32::WHITE, BLUE);
            for y in [-4., 0., 4.] {
                line((-4., y), (4., y));
            }
        }
        Icon::Open => {
            panel((-8., -4.), (8., 7.), PALE_GOLD, GOLD);
            colored_line((-8., -4.), (-8., -7.), GOLD);
            colored_line((-8., -7.), (-2., -7.), GOLD);
            colored_line((-2., -7.), (1., -4.), GOLD);
        }
        Icon::Home => {
            painter.add(egui::Shape::closed_line(
                vec![p(-9., -1.), p(0., -9.), p(9., -1.)],
                Stroke::new(1.4_f32, accent(ACCENT)),
            ));
            panel((-6., -1.), (6., 9.), SELECTION, ACCENT);
            panel((-2., 3.), (2., 9.), Color32::WHITE, ACCENT);
        }
        Icon::Settings => {
            let points = (0..32)
                .map(|i| {
                    let angle = i as f32 * std::f32::consts::TAU / 32.;
                    let radius = if i % 4 < 2 { 8.5 } else { 6.5 };
                    p(angle.cos() * radius, angle.sin() * radius)
                })
                .collect();
            painter.add(egui::Shape::closed_line(points, stroke));
            painter.circle_stroke(center, 3. * scale, stroke);
        }
        Icon::Print => {
            rect((-5., -9.), (5., -3.));
            rect((-8., -3.), (8., 5.));
            panel((-5., 2.), (5., 9.), Color32::WHITE, color);
            painter.circle_filled(p(5., 0.), scale, accent(BLUE));
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
            pos2(
                rect.center().x,
                rect.top() + if rect.height() < 62. { 15. } else { 22. },
            ),
            if rect.height() < 62. { 22. } else { 28. },
            color,
            enabled,
        );
        ui.painter().text(
            pos2(rect.center().x, rect.bottom() - 17.),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(12.),
            color,
        );
    } else if label.is_empty() {
        paint(ui, icon, rect.center(), 16., color, enabled);
    } else {
        paint(
            ui,
            icon,
            pos2(rect.left() + 12., rect.center().y),
            16.,
            color,
            enabled,
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
        enabled,
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
        enabled,
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
        false,
    );
    ui.allocate_rect(rect, Sense::hover())
        .on_hover_text(explanation)
}
