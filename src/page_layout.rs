//! One measured layout shared by page preview and PDF/print output.
use crate::{
    Result,
    model::*,
    pagination::{Measure, Plan},
    structure::Block,
};
use eframe::egui::{self, Color32, FontId, Galley, Pos2, Rect};
use std::sync::Arc;
pub enum Content {
    Text(Arc<Galley>),
    Table(Vec<crate::object_layout::TableRow>),
    Picture(crate::structure::Picture),
}
pub struct Layout {
    pub plan: Plan,
    pub content: Vec<Content>,
}
impl Layout {
    pub fn build(ctx: &egui::Context, document: &Document) -> Result<Self> {
        document.validate()?;
        let settings = crate::pagination::sections(document);
        let labels = crate::structure::labels(document);
        for section in &settings {
            for (label, text, reserve) in [
                ("Header", &section.header, 0.),
                (
                    "Footer",
                    &section.footer,
                    if section.page_numbers { 36. } else { 0. },
                ),
            ] {
                let galley = ctx.fonts_mut(|f| {
                    f.layout_no_wrap(
                        text.clone(),
                        FontId::new(9., egui::FontFamily::Name("body".into())),
                        Color32::BLACK,
                    )
                });
                if galley.size().x > section.text_width() - reserve {
                    return Err(format!(
                        "{label} is wider than the page text area. Shorten it or change the margins."
                    ));
                }
            }
        }

        let mut measures = Vec::new();
        let mut content = Vec::new();
        for (i, p) in document.paragraphs.iter().enumerate() {
            let width = settings[i].text_width();
            let resolved = document.resolved_paragraph(p);
            let (item, rows) = match &p.structure.block {
                Some(Block::Table(table)) => {
                    let rows = crate::object_layout::table_rows(ctx, table, width, 11.);
                    let heights = rows.iter().map(|r| r.height).collect();
                    (Content::Table(rows), heights)
                }
                Some(Block::Picture(picture)) => {
                    let _ = crate::object_layout::picture_texture(ctx, picture)?;
                    let size = crate::object_layout::dimensions(
                        &Block::Picture(picture.clone()),
                        width,
                        11.,
                        ctx,
                    );
                    (Content::Picture(picture.clone()), vec![size.y])
                }
                None => {
                    let mut paragraph = p.clone();
                    paragraph.spacing.space_before_half_points = Some(0);
                    paragraph.spacing.space_after_half_points = Some(0);
                    if let Some(list) = &mut paragraph.structure.list
                        && list.kind == crate::structure::ListKind::Decimal
                    {
                        list.restart = labels[i]
                            .as_ref()
                            .and_then(|l| l.trim_end_matches('.').parse().ok());
                    }
                    let single = Document {
                        paragraphs: vec![paragraph],
                        page: document.page.clone(),
                        style_catalog: document.style_catalog.clone(),
                        character_styles: document.character_styles.clone(),
                    };
                    let galley = crate::layout::galley(
                        ctx,
                        &single,
                        &p.text,
                        CharacterFormat::default(),
                        0,
                        width,
                        11.,
                    );
                    let rows = galley.rows.iter().map(|r| r.size.y).collect();
                    (Content::Text(galley), rows)
                }
            };
            measures.push(Measure {
                rows,
                before: resolved.space_before_points(),
                after: resolved.space_after_points(),
            });
            content.push(item);
        }
        let plan = crate::pagination::paginate(document, &measures)?;
        Ok(Self { plan, content })
    }
    pub fn paint(
        &self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        page: usize,
        origin: Pos2,
        scale: f32,
    ) {
        self.paint_internal(ctx, painter, page, origin, scale, true);
    }
    pub fn paint_background(
        &self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        page: usize,
        origin: Pos2,
        scale: f32,
    ) {
        self.paint_internal(ctx, painter, page, origin, scale, false);
    }
    #[allow(clippy::too_many_arguments)]
    fn paint_internal(
        &self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        page: usize,
        origin: Pos2,
        scale: f32,
        body: bool,
    ) {
        let page = &self.plan.pages[page];
        let settings = &page.settings;
        let transform = egui::emath::TSTransform::new(origin.to_vec2(), scale);
        let mut shapes = Vec::new();
        shapes.push(egui::Shape::rect_filled(
            Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(
                    f32::from(settings.width_points),
                    f32::from(settings.height_points),
                ),
            ),
            0.,
            Color32::WHITE,
        ));
        for place in page.items.iter().filter(|_| body) {
            let pos = egui::pos2(
                f32::from(settings.left_points),
                f32::from(settings.top_points) + place.y,
            );
            match &self.content[place.paragraph] {
                Content::Text(galley) => {
                    let mut fragment = galley.as_ref().clone();
                    let mut row = galley.rows[place.row].clone();
                    row.pos.y = 0.;
                    fragment.rect = Rect::from_min_size(Pos2::ZERO, row.size);
                    fragment.mesh_bounds = row.visuals.mesh_bounds.translate(row.pos.to_vec2());
                    fragment.num_vertices = row.visuals.mesh.vertices.len();
                    fragment.num_indices = row.visuals.mesh.indices.len();
                    fragment.rows = vec![row];
                    shapes.push(egui::Shape::galley(pos, Arc::new(fragment), Color32::BLACK));
                }
                Content::Table(rows) => {
                    let row = &rows[place.row];
                    let mut x = pos.x;
                    for (cell, width) in row.cells.iter().zip(&row.widths) {
                        shapes.push(egui::Shape::rect_stroke(
                            Rect::from_min_size(
                                egui::pos2(x, pos.y),
                                egui::vec2(*width, row.height),
                            ),
                            0.,
                            egui::Stroke::new(0.5_f32, Color32::from_gray(155)),
                            egui::StrokeKind::Inside,
                        ));
                        shapes.push(egui::Shape::galley(
                            egui::pos2(x + 4., pos.y + 4.),
                            cell.clone(),
                            Color32::BLACK,
                        ));
                        x += width;
                    }
                }
                Content::Picture(picture) => {
                    if let Ok(texture) = crate::object_layout::picture_texture(ctx, picture) {
                        let dimensions = crate::object_layout::dimensions(
                            &Block::Picture(picture.clone()),
                            settings.text_width(),
                            11.,
                            ctx,
                        );
                        shapes.push(egui::Shape::image(
                            texture.id(),
                            Rect::from_min_size(pos, dimensions),
                            Rect::from_min_max(Pos2::ZERO, egui::pos2(1., 1.)),
                            Color32::WHITE,
                        ));
                    }
                }
            }
        }
        for (text, y) in [
            (&settings.header, f32::from(settings.top_points) / 2. - 5.),
            (
                &settings.footer,
                f32::from(settings.height_points) - f32::from(settings.bottom_points) / 2. - 5.,
            ),
        ] {
            let galley = ctx.fonts_mut(|f| {
                f.layout_no_wrap(
                    text.clone(),
                    FontId::new(9., egui::FontFamily::Name("body".into())),
                    Color32::from_gray(55),
                )
            });
            let x = f32::from(settings.left_points);
            shapes.push(egui::Shape::galley(
                egui::pos2(x, y),
                galley,
                Color32::from_gray(55),
            ));
        }
        if settings.page_numbers {
            let text = page.number.to_string();
            let galley = ctx.fonts_mut(|f| {
                f.layout_no_wrap(text, FontId::proportional(9.), Color32::from_gray(55))
            });
            shapes.push(egui::Shape::galley(
                egui::pos2(
                    f32::from(settings.width_points - settings.right_points) - galley.size().x,
                    f32::from(settings.height_points) - f32::from(settings.bottom_points) / 2. - 5.,
                ),
                galley,
                Color32::from_gray(55),
            ));
        }
        for mut shape in shapes {
            shape.transform(transform);
            painter.add(shape);
        }
    }
}

impl Layout {
    pub fn page_origins(&self) -> Vec<f32> {
        let mut y = 0.;
        self.plan
            .pages
            .iter()
            .map(|p| {
                let top = y;
                y += f32::from(p.settings.height_points) + 16.;
                top
            })
            .collect()
    }
    pub fn extent(&self) -> egui::Vec2 {
        egui::vec2(
            self.plan
                .pages
                .iter()
                .map(|p| f32::from(p.settings.width_points))
                .fold(0., f32::max),
            self.plan
                .pages
                .iter()
                .map(|p| f32::from(p.settings.height_points) + 16.)
                .sum::<f32>()
                - 16.,
        )
    }
    /// Retain the logical text/glyph sequence while placing rows on actual pages.
    /// Table cells remain an atomic object here and are edited by the table editor.
    pub fn editable_galley(
        &self,
        ctx: &egui::Context,
        document: &Document,
        scale: f32,
    ) -> Arc<Galley> {
        let settings = crate::pagination::sections(document);
        let mut job = egui::text::LayoutJob::default();
        job.append(&document.text(), 0., egui::TextFormat::default());
        let galleys: Vec<_> = self
            .content
            .iter()
            .enumerate()
            .map(|(p, content)| match content {
                Content::Text(g) => g.clone(),
                _ => {
                    let single = Document {
                        paragraphs: vec![document.paragraphs[p].clone()],
                        page: document.page.clone(),
                        style_catalog: document.style_catalog.clone(),
                        character_styles: document.character_styles.clone(),
                    };
                    crate::layout::galley(
                        ctx,
                        &single,
                        &single.text(),
                        CharacterFormat::default(),
                        0,
                        settings[p].text_width(),
                        11.,
                    )
                }
            })
            .collect();
        let mut galley = Galley::concat(Arc::new(job), &galleys, ctx.pixels_per_point());
        let origins = self.page_origins();
        let extent = self.extent();
        let mut placements = vec![Vec::new(); document.paragraphs.len()];
        for (page_index, page) in self.plan.pages.iter().enumerate() {
            let x = (extent.x - f32::from(page.settings.width_points)) / 2.
                + f32::from(page.settings.left_points);
            for p in &page.items {
                placements[p.paragraph].push((
                    p.row,
                    p.repeated_header,
                    egui::pos2(
                        x,
                        origins[page_index] + f32::from(page.settings.top_points) + p.y,
                    ),
                ));
            }
        }
        let mut index = 0;
        for (p, source) in galleys.iter().enumerate() {
            match &self.content[p] {
                Content::Text(_) => {
                    for (row, _repeat, pos) in &placements[p] {
                        galley.rows[index + row].pos =
                            *pos + egui::vec2(source.rows[*row].pos.x, 0.);
                    }
                }
                Content::Table(rows) => {
                    let first = placements[p][0].2;
                    let placed = &mut galley.rows[index];
                    placed.pos = first;
                    let row = Arc::make_mut(&mut placed.row);
                    row.visuals.mesh = egui::epaint::Mesh::default();
                    let mut bottom = first.y;
                    for (r, _, pos) in &placements[p] {
                        row.visuals.mesh.append(crate::object_layout::row_mesh(
                            &rows[*r],
                            Pos2::ZERO + (*pos - first),
                            1.,
                        ));
                        bottom = bottom.max(pos.y + rows[*r].height);
                    }
                    row.size.y = bottom - first.y;
                    row.visuals.mesh_bounds = row.visuals.mesh.calc_bounds();
                }
                Content::Picture(_) => galley.rows[index].pos = placements[p][0].2,
            }
            index += source.rows.len();
        }
        galley.mesh_bounds = Rect::NOTHING;
        galley.num_vertices = 0;
        galley.num_indices = 0;
        for placed in &mut galley.rows {
            placed.pos = Pos2::ZERO + placed.pos.to_vec2() * scale;
            let row = Arc::make_mut(&mut placed.row);
            row.size *= scale;
            for glyph in &mut row.glyphs {
                glyph.pos = Pos2::ZERO + glyph.pos.to_vec2() * scale;
                glyph.advance_width *= scale;
                glyph.line_height *= scale;
                glyph.font_ascent *= scale;
                glyph.font_height *= scale;
                glyph.font_impl_ascent *= scale;
                glyph.font_impl_height *= scale;
            }
            for vertex in &mut row.visuals.mesh.vertices {
                vertex.pos = Pos2::ZERO + vertex.pos.to_vec2() * scale;
            }
            row.visuals.mesh_bounds = row.visuals.mesh.calc_bounds();
            galley.mesh_bounds |= row.visuals.mesh_bounds.translate(placed.pos.to_vec2());
            galley.num_vertices += row.visuals.mesh.vertices.len();
            galley.num_indices += row.visuals.mesh.indices.len();
        }
        galley.rect = Rect::from_min_size(Pos2::ZERO, extent * scale);
        Arc::new(galley)
    }
}
