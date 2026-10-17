//! Native text projection. One galley retains cross-paragraph selection and hit testing.
use crate::{
    commands::Editor,
    model::{Alignment, Document, Style},
    presentation,
};
use eframe::egui::{
    self, Color32, FontData, FontDefinitions, FontFamily, FontId, Galley, TextFormat,
    text::LayoutJob,
};
use std::sync::Arc;

pub fn configure_fonts(ctx: &egui::Context) -> String {
    let mut fonts = FontDefinitions::default();
    let fallback = fonts.families[&FontFamily::Proportional].clone();
    let regular = [
        "C:/Windows/Fonts/arial.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ];
    let bold = [
        "C:/Windows/Fonts/arialbd.ttf",
        "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    ];
    let mut name = "Embedded fallback".to_owned();
    for (key, paths) in [("body", regular.as_slice()), ("body-bold", bold.as_slice())] {
        let mut family = fallback.clone();
        if let Some((path, data)) = paths
            .iter()
            .find_map(|path| std::fs::read(path).ok().map(|data| (*path, data)))
        {
            fonts
                .font_data
                .insert(key.into(), FontData::from_owned(data).into());
            family.insert(0, key.into());
            if key == "body" {
                name = if path.contains("Arial") || path.contains("arial") {
                    "Arial"
                } else {
                    "DejaVu Sans"
                }
                .into();
            }
        }
        fonts.families.insert(FontFamily::Name(key.into()), family);
    }
    if let Ok(data) = std::fs::read("C:/Windows/Fonts/segoeui.ttf") {
        fonts
            .font_data
            .insert("interface".into(), FontData::from_owned(data).into());
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "interface".into());
    }
    ctx.set_fonts(fonts);
    name
}

fn format(style: Style, size: f32) -> TextFormat {
    let size = size * style.font_size.points() / Style::default().font_size.points();
    TextFormat {
        font_id: FontId::new(
            size,
            FontFamily::Name(if style.bold { "body-bold" } else { "body" }.into()),
        ),
        italics: style.italic,
        underline: if style.underline {
            egui::Stroke::new((size / 14.).max(1.), Color32::from_rgb(24, 24, 24))
        } else {
            egui::Stroke::NONE
        },
        color: Color32::from_rgb(24, 24, 24),
        line_height: Some(size * 1.25),
        ..Default::default()
    }
}

pub fn galley(
    ctx: &egui::Context,
    source: &Document,
    display: &str,
    typing_style: Style,
    width: f32,
    size: f32,
) -> Arc<Galley> {
    let mut preview = Editor::new(source.clone());
    preview.typing_style = typing_style;
    if source.text() != display {
        let (start, end, text) = presentation::text_change(&source.text(), display);
        let _ = preview.replace(start, end, &text, "Input projection");
    }
    let source = &preview.document;
    let mut whole = LayoutJob::default();
    let mut paragraphs = Vec::new();
    for (index, p) in source.paragraphs.iter().enumerate() {
        let mut job = LayoutJob::default();
        job.wrap.max_width = width;
        job.justify = p.alignment == Alignment::Justify;
        // Sections use UTF-8 byte ranges internally; editor positions remain scalar indices.
        let chars: Vec<char> = p.text.chars().collect();
        let mut start = 0;
        while start < chars.len() {
            let mut end = start + 1;
            while end < chars.len() && p.styles[end] == p.styles[start] {
                end += 1;
            }
            let text: String = chars[start..end].iter().collect();
            job.append(&text, 0., format(p.styles[start], size));
            whole.append(&text, 0., format(p.styles[start], size));
            start = end;
        }
        if chars.is_empty() {
            job.append("", 0., format(typing_style, size));
        }
        let mut paragraph = ctx.fonts_mut(|f| f.layout_job(job)).as_ref().clone();
        for placed in &mut paragraph.rows {
            let row = Arc::make_mut(&mut placed.row);
            let extra = (width - row.size.x).max(0.);
            let shift = match p.alignment {
                Alignment::Center => extra / 2.,
                Alignment::Right => extra,
                _ => 0.,
            };
            if shift > 0. {
                for glyph in &mut row.glyphs {
                    glyph.pos.x += shift;
                }
                for vertex in &mut row.visuals.mesh.vertices {
                    vertex.pos.x += shift;
                }
                row.visuals.mesh_bounds = row.visuals.mesh_bounds.translate(egui::vec2(shift, 0.));
                row.size.x += shift;
            }
        }
        paragraph.rect.max.x = width;
        if index + 1 < source.paragraphs.len() {
            paragraph.rect.max.y += size * 0.5;
            whole.append("\n", 0., format(Style::default(), size));
        }
        paragraphs.push(Arc::new(paragraph));
    }
    whole.wrap.max_width = width;
    let mut merged = Galley::concat(Arc::new(whole), &paragraphs, ctx.pixels_per_point());
    merged.rect.min.x = 0.;
    merged.rect.max.x = width;
    Arc::new(merged)
}
