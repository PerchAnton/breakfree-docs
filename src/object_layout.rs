//! Native rendering for anchored, document-owned table and picture blocks.
use crate::{
    Result,
    structure::{Block, Picture, Table},
};
use eframe::egui::{self, Color32, FontId, Galley, Pos2, Rect, Vec2};
use std::sync::Arc;
#[derive(Clone)]
pub struct TableRow {
    pub cells: Vec<Arc<Galley>>,
    pub widths: Vec<f32>,
    pub height: f32,
}
pub fn table_rows(ctx: &egui::Context, table: &Table, width: f32, size: f32) -> Vec<TableRow> {
    let total = table
        .widths_points
        .iter()
        .map(|v| f32::from(*v))
        .sum::<f32>();
    let scale = (width / total).min(size / 11.);
    let widths: Vec<f32> = table
        .widths_points
        .iter()
        .map(|v| f32::from(*v) * scale)
        .collect();
    table
        .rows
        .iter()
        .enumerate()
        .map(|(row, cells)| {
            let cells: Vec<_> = cells
                .iter()
                .zip(&widths)
                .map(|(text, w)| {
                    ctx.fonts_mut(|f| {
                        f.layout(
                            text.clone(),
                            FontId::new(
                                size,
                                egui::FontFamily::Name(
                                    if row == 0 && table.repeat_header {
                                        "body-bold"
                                    } else {
                                        "body"
                                    }
                                    .into(),
                                ),
                            ),
                            Color32::from_gray(24),
                            (w - 8. * size / 11.).max(1.),
                        )
                    })
                })
                .collect();
            let height = cells.iter().map(|g| g.size().y).fold(size, f32::max) + 8. * size / 11.;
            TableRow {
                cells,
                widths: widths.clone(),
                height,
            }
        })
        .collect()
}
pub fn row_mesh(row: &TableRow, origin: Pos2, scale: f32) -> egui::epaint::Mesh {
    let mut mesh = egui::epaint::Mesh::default();
    let mut x = origin.x;
    for (cell, width) in row.cells.iter().zip(&row.widths) {
        let rect = Rect::from_min_size(egui::pos2(x, origin.y), egui::vec2(*width, row.height));
        let stroke = Color32::from_gray(155);
        mesh.add_colored_rect(
            Rect::from_min_size(rect.min, egui::vec2(*width, 0.5 * scale)),
            stroke,
        );
        mesh.add_colored_rect(
            Rect::from_min_size(
                rect.left_bottom() - egui::vec2(0., 0.5 * scale),
                egui::vec2(*width, 0.5 * scale),
            ),
            stroke,
        );
        mesh.add_colored_rect(
            Rect::from_min_size(rect.min, egui::vec2(0.5 * scale, row.height)),
            stroke,
        );
        mesh.add_colored_rect(
            Rect::from_min_size(
                rect.right_top() - egui::vec2(0.5 * scale, 0.),
                egui::vec2(0.5 * scale, row.height),
            ),
            stroke,
        );
        for placed in &cell.rows {
            let mut part = placed.visuals.mesh.clone();
            part.translate(
                egui::vec2(x + 4. * scale, origin.y + 4. * scale) + placed.pos.to_vec2(),
            );
            mesh.append(part);
        }
        x += width;
    }
    mesh
}
pub fn dimensions(block: &Block, width: f32, size: f32, ctx: &egui::Context) -> Vec2 {
    match block {
        Block::Table(t) => {
            let rows = table_rows(ctx, t, width, size);
            egui::vec2(
                rows.first().map_or(width, |r| r.widths.iter().sum()),
                rows.iter().map(|r| r.height).sum(),
            )
        }
        Block::Picture(p) => {
            let scale = (width / f32::from(p.width_points)).min(size / 11.);
            egui::vec2(
                f32::from(p.width_points) * scale,
                f32::from(p.height_points) * scale,
            )
        }
    }
}
pub fn picture_texture(ctx: &egui::Context, picture: &Picture) -> Result<egui::TextureHandle> {
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    picture.png.hash(&mut hash);
    let id = egui::Id::new(("picture", hash.finish()));
    if let Some(texture) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        return Ok(texture);
    }
    picture.validate()?;
    let image = image::load_from_memory_with_format(&picture.png, image::ImageFormat::Png)
        .map_err(|e| format!("Cannot decode picture: {e}"))?
        .to_rgba8();
    let color = egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image.as_raw(),
    );
    let texture = ctx.load_texture(
        format!("picture-{}", hash.finish()),
        color,
        egui::TextureOptions::LINEAR,
    );
    ctx.data_mut(|d| d.insert_temp(id, texture.clone()));
    Ok(texture)
}
pub fn apply_block(ctx: &egui::Context, galley: &mut Galley, block: &Block, width: f32, size: f32) {
    let dimensions = dimensions(block, width, size, ctx);
    if let Some(placed) = galley.rows.first_mut() {
        let row = Arc::make_mut(&mut placed.row);
        row.size = dimensions;
        row.visuals.mesh = egui::epaint::Mesh::default();
        if let Block::Table(table) = block {
            let mut y = 0.;
            for r in table_rows(ctx, table, width, size) {
                row.visuals
                    .mesh
                    .append(row_mesh(&r, egui::pos2(0., y), size / 11.));
                y += r.height;
            }
        }
        row.visuals.mesh_bounds = Rect::from_min_size(Pos2::ZERO, dimensions);
        if let Some(glyph) = row.glyphs.first_mut() {
            glyph.pos = Pos2::ZERO;
            glyph.advance_width = dimensions.x;
        }
    }
    galley.rect = Rect::from_min_size(Pos2::ZERO, dimensions);
    galley.mesh_bounds = galley.rect;
}
