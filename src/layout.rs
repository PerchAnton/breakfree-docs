//! Native text projection. One galley retains cross-paragraph selection and hit testing.
use crate::{
    commands::Editor,
    model::{Alignment, CharacterFormat, Document, Style},
    presentation,
};
use eframe::egui::{
    self, Color32, FontData, FontDefinitions, FontFamily, FontId, Galley, TextFormat,
    text::LayoutJob,
};
use std::sync::Arc;
#[path = "tab_layout.rs"]
mod tab_layout;

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

fn format(style: Style, size: f32, line_height: f32) -> TextFormat {
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
        line_height: Some(size * line_height),
        ..Default::default()
    }
}

pub fn galley(
    ctx: &egui::Context,
    source: &Document,
    display: &str,
    typing_style: CharacterFormat,
    typing_position: usize,
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
    let active_paragraph = source.locate(typing_position).ok().map(|(p, _)| p);
    let mut whole = LayoutJob::default();
    let mut paragraphs = Vec::new();
    for (index, p) in source.paragraphs.iter().enumerate() {
        let resolved = source.resolved_paragraph(p);
        let mut job = LayoutJob::default();
        let mut left = f32::from(resolved.left_indent_half_points) * size / 22.;
        let mut right = f32::from(resolved.right_indent_half_points) * size / 22.;
        let mut special = f32::from(resolved.first_line_half_points) * size / 22.;
        let scale = crate::ruler::indent_scale(resolved, size / 22., width);
        left *= scale;
        right *= scale;
        special *= scale;
        let first_left = left + special.max(0.);
        let body_left = left + (-special).max(0.);
        let first_width = (width - first_left - right).max(1.);
        let text_width = (width - body_left - right).max(1.);
        job.wrap.max_width = text_width;
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
            job.append(
                &text,
                0.,
                format(
                    p.styles[start].resolve_with(resolved.character, &source.character_styles),
                    size,
                    resolved.line_height(),
                ),
            );
            whole.append(
                &text,
                0.,
                format(
                    p.styles[start].resolve_with(resolved.character, &source.character_styles),
                    size,
                    resolved.line_height(),
                ),
            );
            start = end;
        }
        if chars.is_empty() {
            let style = if active_paragraph == Some(index) {
                typing_style.resolve_with(resolved.character, &source.character_styles)
            } else {
                resolved.character
            };
            job.append("", 0., format(style, size, resolved.line_height()));
        }
        let mut paragraph = if p.text.contains('\t') {
            tab_layout::layout(
                ctx,
                job,
                [first_width, text_width],
                [first_left, body_left],
                f32::from(p.spacing.tab_interval_half_points.unwrap_or(72)) * size / 22.,
                &p.spacing
                    .left_tab_stops_half_points
                    .iter()
                    .map(|p| f32::from(*p) * size / 22.)
                    .collect::<Vec<_>>(),
            )
        } else {
            layout_first_line(ctx, job, first_width, text_width)
        };
        for (row_index, placed) in paragraph.rows.iter_mut().enumerate() {
            let row = Arc::make_mut(&mut placed.row);
            let row_width = if row_index == 0 {
                first_width
            } else {
                text_width
            };
            let extra = (row_width - row.size.x).max(0.);
            let shift = if row_index == 0 {
                first_left
            } else {
                body_left
            } + match p.alignment {
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
        // Justification measures visible ink and excludes trailing whitespace.
        // Cursor-at-row-end must still advance past every stored whitespace glyph.
        for placed in &mut paragraph.rows {
            let row = Arc::make_mut(&mut placed.row);
            if let Some(last) = row.glyphs.last() {
                row.size.x = row.size.x.max(last.max_x());
            }
        }
        paragraph.rect.max.x = width;
        if index + 1 < source.paragraphs.len() {
            whole.append(
                "\n",
                0.,
                format(resolved.character, size, resolved.line_height()),
            );
        }
        paragraphs.push(Arc::new(paragraph));
    }
    whole.wrap.max_width = width;
    let mut merged = Galley::concat(Arc::new(whole), &paragraphs, ctx.pixels_per_point());
    // Galley::concat derives height from rows, ignoring each input galley's rect.
    // Offset placed rows after concatenation so paragraph gaps affect both painting
    // and cursor hit testing, without creating fictitious text rows or characters.
    let mut spacing = size
        * source
            .resolved_paragraph(&source.paragraphs[0])
            .space_before_points()
        / 11.;
    let mut paragraph_index = 0;
    merged.mesh_bounds = egui::Rect::NOTHING;
    for row in &mut merged.rows {
        row.pos.y += spacing;
        merged.mesh_bounds |= row.visuals.mesh_bounds.translate(row.pos.to_vec2());
        if row.ends_with_newline {
            spacing += size
                * source
                    .resolved_paragraph(&source.paragraphs[paragraph_index])
                    .space_after_points()
                / 11.;
            paragraph_index += 1;
            if let Some(next) = source.paragraphs.get(paragraph_index) {
                spacing += size * source.resolved_paragraph(next).space_before_points() / 11.;
            }
        }
    }
    merged.rect.max.y += spacing;
    merged.rect.min.x = 0.;
    merged.rect.max.x = width;
    Arc::new(merged)
}

/// Shape the first row at its own width, then shape the untouched suffix at the
/// continuation width. Concatenation must not introduce a paragraph delimiter.
fn layout_first_line(
    ctx: &egui::Context,
    job: LayoutJob,
    first_width: f32,
    body_width: f32,
) -> Galley {
    if (first_width - body_width).abs() < f32::EPSILON {
        return ctx.fonts_mut(|f| f.layout_job(job)).as_ref().clone();
    }
    let mut first_job = job.clone();
    first_job.wrap.max_width = first_width;
    let mut first = ctx.fonts_mut(|f| f.layout_job(first_job)).as_ref().clone();
    if first.rows.len() <= 1 {
        return first;
    }
    let count = first.rows[0].char_count_excluding_newline();
    let cut = job
        .text
        .char_indices()
        .nth(count)
        .map_or(job.text.len(), |(i, _)| i);
    let mut rest_job = job.clone();
    rest_job.text = job.text[cut..].to_owned();
    rest_job.wrap.max_width = body_width;
    rest_job.sections = job
        .sections
        .iter()
        .filter_map(|section| {
            if section.byte_range.end <= cut {
                return None;
            }
            let mut section = section.clone();
            section.byte_range =
                section.byte_range.start.saturating_sub(cut)..section.byte_range.end - cut;
            section.leading_space = 0.;
            Some(section)
        })
        .collect();
    let rest = ctx.fonts_mut(|f| f.layout_job(rest_job));
    let first_row = first.rows[0].clone();
    let mut prefix_job = job.clone();
    prefix_job.text = job.text[..cut].to_owned();
    prefix_job.wrap.max_width = first_width;
    prefix_job.sections = job
        .sections
        .iter()
        .filter_map(|section| {
            if section.byte_range.start >= cut {
                return None;
            }
            let mut section = section.clone();
            section.byte_range.end = section.byte_range.end.min(cut);
            Some(section)
        })
        .collect();
    first = ctx.fonts_mut(|f| f.layout_job(prefix_job)).as_ref().clone();
    first.rows = vec![first_row];
    first.rect = first.rows[0].rect();
    first.mesh_bounds = first.rows[0].visuals.mesh_bounds;
    first.num_vertices = first.rows[0].visuals.mesh.vertices.len();
    first.num_indices = first.rows[0].visuals.mesh.indices.len();
    let mut merged = Galley::concat(
        Arc::new(job),
        &[Arc::new(first), rest],
        ctx.pixels_per_point(),
    );
    Arc::make_mut(&mut merged.rows[0].row).ends_with_newline = false;
    merged
}
