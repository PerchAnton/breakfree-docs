//! Left, center and right tab fields retain one glyph/cursor position per stored scalar.
use super::*;

pub(super) fn layout(
    ctx: &egui::Context,
    job: LayoutJob,
    widths: [f32; 2],
    origins: [f32; 2],
    interval: f32,
    stops: &[(f32, f32, Option<crate::model::TabLeader>)],
) -> Galley {
    let mut measurement = job.clone();
    measurement.wrap.max_width = f32::INFINITY;
    measurement.justify = false;
    let measured = ctx.fonts_mut(|f| f.layout_job(measurement));
    let glyphs = &measured.rows[0].glyphs;
    // Measure each complete logical field once, including trailing spaces and formatting.
    // A long field is allowed to wrap; alignment never moves backwards over preceding text.
    let mut field_widths = vec![0.; glyphs.len()];
    let mut field_width = 0.;
    for (index, glyph) in glyphs.iter().enumerate().rev() {
        if glyph.chr == '\t' {
            field_widths[index] = field_width;
            field_width = 0.;
        } else {
            field_width += glyph.advance_width;
        }
    }
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
        let mut previous_stop = f32::NEG_INFINITY;
        let mut end = start;
        let mut last_break = None;
        while end < glyphs.len() {
            let g = &glyphs[end];
            let advance = if g.chr == '\t' {
                let (advance, stop) = tab_advance(
                    x,
                    origin,
                    interval,
                    width,
                    stops,
                    field_widths[end],
                    previous_stop,
                );
                previous_stop = stop;
                advance
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
        let mut previous_stop = f32::NEG_INFINITY;
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
                let (advance, stop) = tab_advance(
                    cursor,
                    origin,
                    interval,
                    width,
                    stops,
                    field_widths[start + scalar],
                    previous_stop,
                );
                previous_stop = stop;
                let delta = advance - natural;
                append(&mut row_job, section, at..at + 1, bytes[start], delta);
                row_job.sections.last_mut().unwrap().format.underline = egui::Stroke::NONE;
                let leader = stops.iter().find(|s| s.0 == stop).and_then(|s| s.2);
                tabs.push((
                    scalar,
                    cursor,
                    advance,
                    section.format.underline,
                    leader,
                    section.format.font_id.size,
                    section.format.color,
                ));
                cursor += advance;
                chunk = at + 1;
            }
            append(&mut row_job, section, chunk..high, bytes[start], 0.);
            cursor += measure(ctx, &job.text[chunk..high], &section.format);
        }
        let mut row_galley = ctx.fonts_mut(|f| f.layout_job(row_job)).as_ref().clone();
        let row = Arc::make_mut(&mut row_galley.rows[0].row);
        for (index, x, advance, underline, leader, size, color) in tabs {
            row.glyphs[index].pos.x = x;
            row.glyphs[index].advance_width = advance;
            if let Some(leader) = leader {
                let y = row.glyphs[index].logical_rect().bottom() - size * 0.15;
                paint_leader(&mut row.visuals.mesh, x, advance, y, size, color, leader);
            }
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

fn tab_advance(
    x: f32,
    origin: f32,
    interval: f32,
    width: f32,
    stops: &[(f32, f32, Option<crate::model::TabLeader>)],
    field_width: f32,
    previous_stop: f32,
) -> (f32, f32) {
    let absolute = (origin + x).max(previous_stop);
    let (position, factor, _) = stops
        .iter()
        .copied()
        .find(|(position, _, _)| *position > absolute + 0.001)
        .unwrap_or_else(|| {
            (
                (absolute / interval).floor().mul_add(interval, interval),
                0.,
                None,
            )
        });
    let target = (position - origin).min(width);
    ((target - factor * field_width - x).max(0.), position)
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

/// Decorative mesh only: no extra glyphs/scalars/cursor stops. Bounded work per tab.
fn paint_leader(
    mesh: &mut egui::epaint::Mesh,
    x: f32,
    advance: f32,
    y: f32,
    size: f32,
    color: Color32,
    leader: crate::model::TabLeader,
) {
    use crate::model::TabLeader;
    let low = x + size * 0.15;
    let high = x + advance - size * 0.15;
    let thickness = size / 16.;
    if high - low < thickness || !high.is_finite() {
        return;
    }
    if leader == TabLeader::Line {
        mesh.add_colored_rect(
            egui::Rect::from_min_max(
                egui::pos2(low, y - thickness / 2.),
                egui::pos2(high, y + thickness / 2.),
            ),
            color,
        );
        return;
    }
    let mark_width = if leader == TabLeader::Dots {
        thickness
    } else {
        size * 0.3
    };
    let pitch = size * 0.5;
    let count = (((high - low - mark_width) / pitch).floor() + 1.).clamp(0., 512.) as usize;
    for i in 0..count {
        let at = low + i as f32 * pitch;
        if leader == TabLeader::Dashes {
            mesh.add_colored_rect(
                egui::Rect::from_min_max(
                    egui::pos2(at, y - thickness / 2.),
                    egui::pos2(at + mark_width, y + thickness / 2.),
                ),
                color,
            );
        } else {
            let base = mesh.vertices.len() as u32;
            let center = egui::pos2(at + thickness / 2., y);
            mesh.colored_vertex(center, color);
            for n in 0..8 {
                let angle = n as f32 * std::f32::consts::TAU / 8.;
                mesh.colored_vertex(
                    center + egui::vec2(angle.cos(), angle.sin()) * thickness / 2.,
                    color,
                );
            }
            for n in 0..8 {
                mesh.add_triangle(base, base + 1 + n, base + 1 + (n + 1) % 8);
            }
        }
    }
}

#[cfg(test)]
mod leader_tests {
    use super::*;
    #[test]
    fn leader_mesh_stays_inside_advance_and_has_bounded_work() {
        use crate::model::TabLeader as L;
        for leader in [L::Dots, L::Dashes, L::Line] {
            for advance in [0., 0.1, 3., 30., 100000.] {
                let mut mesh = egui::epaint::Mesh::default();
                paint_leader(&mut mesh, 10., advance, 20., 11., Color32::BLACK, leader);
                assert!(mesh.vertices.len() <= 512 * 9);
                for v in &mesh.vertices {
                    assert!(v.pos.x >= 10. && v.pos.x <= 10. + advance);
                    assert!(v.pos.y >= 19. && v.pos.y <= 21.);
                }
                if advance < 3. {
                    assert!(mesh.vertices.is_empty());
                }
            }
        }
    }
}
