//! Left tab grids retain one glyph/cursor position per stored scalar.
use super::*;

pub(super) fn layout(
    ctx: &egui::Context,
    job: LayoutJob,
    widths: [f32; 2],
    origins: [f32; 2],
    interval: f32,
    stops: &[f32],
) -> Galley {
    let mut measurement = job.clone();
    measurement.wrap.max_width = f32::INFINITY;
    measurement.justify = false;
    let measured = ctx.fonts_mut(|f| f.layout_job(measurement));
    let glyphs = &measured.rows[0].glyphs;
    let bytes: Vec<_> = job
        .text
        .char_indices()
        .map(|(i, _)| i)
        .chain([job.text.len()])
        .collect();
    let mut boundary = vec![false; glyphs.len() + 1];
    for index in crate::model::boundaries(&job.text) {
        boundary[index] = true;
    }
    let mut rows = Vec::new();
    let mut start = 0;
    while start < glyphs.len() {
        let which = usize::from(!rows.is_empty());
        let width = widths[which];
        let origin = origins[which];
        let mut x = 0.;
        let mut end = start;
        let mut last_break = None;
        while end < glyphs.len() {
            let g = &glyphs[end];
            let advance = if g.chr == '\t' {
                tab_advance(x, origin, interval, width, stops)
            } else {
                g.advance_width
            };
            if end > start && boundary[end] && x + advance > width + 0.01 {
                end = last_break.unwrap_or(end);
                break;
            }
            x += advance;
            end += 1;
            if g.chr.is_whitespace() && boundary[end] {
                last_break = Some(end);
            }
        }
        // Preserve combining clusters even when a cluster exceeds the column.
        while end < glyphs.len() && !boundary[end] {
            end += 1;
        }
        let mut row_job = LayoutJob {
            text: job.text[bytes[start]..bytes[end]].to_owned(),
            ..Default::default()
        };
        row_job.wrap.max_width = f32::INFINITY;
        // Tab rows use fixed grid spacing, not full justification.
        row_job.justify = false;
        row_job.sections.clear();
        let mut cursor = 0.;
        let mut tabs = Vec::new();
        let section_start = job
            .sections
            .partition_point(|s| s.byte_range.end <= bytes[start]);
        for section in &job.sections[section_start..] {
            if section.byte_range.start >= bytes[end] {
                break;
            }
            let low = section.byte_range.start.max(bytes[start]);
            let high = section.byte_range.end.min(bytes[end]);
            if low >= high {
                continue;
            }
            let mut chunk = low;
            for (offset, ch) in job.text[low..high].char_indices() {
                if ch != '\t' {
                    continue;
                }
                let at = low + offset;
                append(&mut row_job, section, chunk..at, bytes[start], 0.);
                cursor += measure(ctx, &job.text[chunk..at], &section.format);
                let scalar = bytes.binary_search(&at).unwrap() - start;
                let natural = glyphs[start + scalar].advance_width;
                let advance = tab_advance(cursor, origin, interval, width, stops);
                let delta = advance - natural;
                append(&mut row_job, section, at..at + 1, bytes[start], delta);
                row_job.sections.last_mut().unwrap().format.underline = egui::Stroke::NONE;
                tabs.push((scalar, cursor, advance, section.format.underline));
                cursor += advance;
                chunk = at + 1;
            }
            append(&mut row_job, section, chunk..high, bytes[start], 0.);
            cursor += measure(ctx, &job.text[chunk..high], &section.format);
        }
        let mut row_galley = ctx.fonts_mut(|f| f.layout_job(row_job)).as_ref().clone();
        let row = Arc::make_mut(&mut row_galley.rows[0].row);
        for (index, x, advance, underline) in tabs {
            row.glyphs[index].pos.x = x;
            row.glyphs[index].advance_width = advance;
            if underline != egui::Stroke::NONE && advance > 0. {
                let y = row.glyphs[index].logical_rect().bottom();
                row.visuals.mesh.add_colored_rect(
                    egui::Rect::from_min_max(
                        egui::pos2(x, y - underline.width / 2.),
                        egui::pos2(x + advance, y + underline.width / 2.),
                    ),
                    underline.color,
                );
            }
        }
        row.visuals.mesh_bounds = row.visuals.mesh.calc_bounds();
        row_galley.mesh_bounds = row.visuals.mesh_bounds;
        row_galley.num_vertices = row.visuals.mesh.vertices.len();
        row_galley.num_indices = row.visuals.mesh.indices.len();
        rows.push(Arc::new(row_galley));
        start = end;
    }
    let mut result = Galley::concat(Arc::new(job), &rows, ctx.pixels_per_point());
    for row in &mut result.rows {
        Arc::make_mut(&mut row.row).ends_with_newline = false;
    }
    result
}

fn tab_advance(x: f32, origin: f32, interval: f32, width: f32, stops: &[f32]) -> f32 {
    let absolute = origin + x;
    let next = stops
        .iter()
        .copied()
        .find(|stop| *stop > absolute + 0.001)
        .unwrap_or_else(|| (absolute / interval).floor().mul_add(interval, interval))
        - origin;
    (next.min(width) - x).max(0.)
}

fn append(
    job: &mut LayoutJob,
    source: &egui::text::LayoutSection,
    range: std::ops::Range<usize>,
    base: usize,
    leading: f32,
) {
    if range.is_empty() {
        return;
    }
    let mut section = source.clone();
    section.byte_range = range.start - base..range.end - base;
    section.leading_space = leading;
    job.sections.push(section);
}

fn measure(ctx: &egui::Context, text: &str, format: &TextFormat) -> f32 {
    if text.is_empty() {
        return 0.;
    }
    let mut job = LayoutJob::default();
    job.append(text, 0., format.clone());
    ctx.fonts_mut(|f| f.layout_job(job)).rows[0].size.x
}
