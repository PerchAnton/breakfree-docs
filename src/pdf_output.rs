//! Raster PDF output using the same page layout, glyph meshes and image textures.
//! No font programs are embedded. Output is visual (144 dpi), not searchable text.
use crate::{Result, model::Document};
use eframe::egui::{self, ColorImage, Pos2, TextureId};
use image::ImageEncoder;
use std::{collections::HashMap, path::Path};
pub struct RasterPage {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<u8>,
}
fn sample(texture: &ColorImage, u: f32, v: f32) -> [f32; 4] {
    let x = (u * texture.size[0] as f32 - 0.5).clamp(0., (texture.size[0] - 1) as f32);
    let y = (v * texture.size[1] as f32 - 0.5).clamp(0., (texture.size[1] - 1) as f32);
    let (x0, y0) = (x.floor() as usize, y.floor() as usize);
    let (x1, y1) = (
        (x0 + 1).min(texture.size[0] - 1),
        (y0 + 1).min(texture.size[1] - 1),
    );
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let mut out = [0.; 4];
    for (px, py, w) in [
        (x0, y0, (1. - fx) * (1. - fy)),
        (x1, y0, fx * (1. - fy)),
        (x0, y1, (1. - fx) * fy),
        (x1, y1, fx * fy),
    ] {
        for (c, b) in texture.pixels[py * texture.size[0] + px]
            .to_array()
            .into_iter()
            .enumerate()
        {
            out[c] += f32::from(b) * w / 255.;
        }
    }
    out
}
fn edge(a: Pos2, b: Pos2, p: Pos2) -> f32 {
    (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
}
fn inclusive(a: Pos2, b: Pos2) -> bool {
    b.y < a.y || (b.y == a.y && b.x > a.x)
}
fn rasterize(
    primitives: Vec<egui::ClippedPrimitive>,
    textures: &HashMap<TextureId, ColorImage>,
    width: usize,
    height: usize,
    scale: f32,
) -> Result<RasterPage> {
    let mut rgb = vec![255u8; width * height * 3];
    for primitive in primitives {
        let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive else {
            return Err("Unsupported export paint callback.".into());
        };
        let texture = textures
            .get(&mesh.texture_id)
            .ok_or("Missing export texture.")?;
        for triangle in mesh.indices.as_chunks::<3>().0 {
            let mut v = [
                mesh.vertices[triangle[0] as usize],
                mesh.vertices[triangle[1] as usize],
                mesh.vertices[triangle[2] as usize],
            ];
            for vertex in &mut v {
                vertex.pos = Pos2::new(vertex.pos.x * scale, vertex.pos.y * scale);
            }
            let mut area = edge(v[0].pos, v[1].pos, v[2].pos);
            if area < 0. {
                v.swap(1, 2);
                area = -area;
            }
            if area < 0.00001 {
                continue;
            }
            let clip = primitive.clip_rect * scale;
            let min_x = v
                .iter()
                .map(|v| v.pos.x)
                .fold(f32::INFINITY, f32::min)
                .max(clip.min.x)
                .max(0.)
                .floor() as usize;
            let max_x = v
                .iter()
                .map(|v| v.pos.x)
                .fold(f32::NEG_INFINITY, f32::max)
                .min(clip.max.x)
                .min(width as f32)
                .ceil() as usize;
            let min_y = v
                .iter()
                .map(|v| v.pos.y)
                .fold(f32::INFINITY, f32::min)
                .max(clip.min.y)
                .max(0.)
                .floor() as usize;
            let max_y = v
                .iter()
                .map(|v| v.pos.y)
                .fold(f32::NEG_INFINITY, f32::max)
                .min(clip.max.y)
                .min(height as f32)
                .ceil() as usize;
            for y in min_y..max_y.min(height) {
                for x in min_x..max_x.min(width) {
                    let p = egui::pos2(x as f32 + 0.5, y as f32 + 0.5);
                    let e = [
                        edge(v[1].pos, v[2].pos, p),
                        edge(v[2].pos, v[0].pos, p),
                        edge(v[0].pos, v[1].pos, p),
                    ];
                    if (0..3).any(|i| {
                        e[i] < 0.
                            || (e[i] == 0. && !inclusive(v[(i + 1) % 3].pos, v[(i + 2) % 3].pos))
                    }) {
                        continue;
                    }
                    let w = e.map(|e| e / area);
                    let uv = v[0].uv.to_vec2() * w[0]
                        + v[1].uv.to_vec2() * w[1]
                        + v[2].uv.to_vec2() * w[2];
                    let tex = sample(texture, uv.x, uv.y);
                    let mut color = [0.; 4];
                    for i in 0..3 {
                        for (c, b) in v[i].color.to_array().into_iter().enumerate() {
                            color[c] += f32::from(b) * w[i] / 255.;
                        }
                    }
                    let alpha = tex[3] * color[3];
                    let at = (y * width + x) * 3;
                    for c in 0..3 {
                        rgb[at + c] = ((tex[c] * color[c]
                            + f32::from(rgb[at + c]) / 255. * (1. - alpha))
                            * 255.)
                            .round()
                            .clamp(0., 255.) as u8;
                    }
                }
            }
        }
    }
    Ok(RasterPage { width, height, rgb })
}
fn update_textures(
    textures: &mut HashMap<TextureId, ColorImage>,
    delta: &egui::TexturesDelta,
) -> Result<()> {
    for (id, update) in &delta.set {
        let egui::ImageData::Color(image) = &update.image;
        if let Some([x, y]) = update.pos {
            let target = textures.get_mut(id).ok_or("Missing base texture.")?;
            for row in 0..image.size[1] {
                let start = (y + row) * target.size[0] + x;
                target.pixels[start..start + image.size[0]]
                    .copy_from_slice(&image.pixels[row * image.size[0]..(row + 1) * image.size[0]]);
            }
        } else {
            textures.insert(*id, image.as_ref().clone());
        }
    }
    Ok(())
}
fn stream(dict: &str, data: &[u8]) -> Vec<u8> {
    let mut bytes = format!("<< {dict} /Length {} >>\nstream\n", data.len()).into_bytes();
    bytes.extend(data);
    bytes.extend(b"\nendstream");
    bytes
}
fn png_data(page: &RasterPage) -> Result<Vec<u8>> {
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(
            &page.rgb,
            page.width as u32,
            page.height as u32,
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| e.to_string())?;
    let mut at = 8;
    let mut data = Vec::new();
    while at + 12 <= png.len() {
        let len = u32::from_be_bytes(png[at..at + 4].try_into().unwrap()) as usize;
        if &png[at + 4..at + 8] == b"IDAT" {
            data.extend_from_slice(&png[at + 8..at + 8 + len]);
        }
        at += len + 12;
    }
    Ok(data)
}
/// Callback permits independent rendering checks without a GUI window or printer.
pub fn encode(document: &Document, mut inspect: impl FnMut(usize, &RasterPage)) -> Result<Vec<u8>> {
    document.validate()?;
    let ctx = egui::Context::default();
    crate::layout::configure_fonts(&ctx);
    ctx.set_pixels_per_point(2.);
    let mut built = None;
    let mut error = None;
    let initial = ctx.run(
        egui::RawInput::default(),
        |ctx| match crate::page_layout::Layout::build(ctx, document) {
            Ok(layout) => built = Some(layout),
            Err(e) => error = Some(e),
        },
    );
    if let Some(e) = error {
        return Err(e);
    }
    let layout = built.ok_or("No page layout.")?;
    let _ = initial;

    let mut objects = vec![b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(), Vec::new()];
    let count = layout.plan.pages.len();
    let kids = (0..count)
        .map(|i| format!("{} 0 R", 3 + i * 3))
        .collect::<Vec<_>>()
        .join(" ");
    objects[1] = format!("<< /Type /Pages /Count {count} /Kids [{kids}] >>").into_bytes();
    for index in 0..count {
        let settings = &layout.plan.pages[index].settings;
        let (w, h) = (
            usize::from(settings.width_points) * 2,
            usize::from(settings.height_points) * 2,
        );
        // Each output page owns a font atlas. Atlas UVs and texture deltas cannot
        // be retained across independent output frames without renderer lifetime tracking.
        let render_ctx = egui::Context::default();
        crate::layout::configure_fonts(&render_ctx);
        render_ctx.set_pixels_per_point(2.);
        let mut render_error = None;
        let output = render_ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    egui::vec2(w as f32 / 2., h as f32 / 2.),
                )),
                ..Default::default()
            },
            |ctx| {
                // Refresh shaped rows in this frame: the font atlas may have been
                // rebuilt between frames. Retained UVs are not a stable font handle.
                let refreshed = match crate::page_layout::Layout::build(ctx, document) {
                    Ok(layout) => layout,
                    Err(e) => {
                        render_error = Some(e);
                        return;
                    }
                };
                if refreshed.plan.pages.len() != count {
                    render_error = Some(
                        "Page measurements changed during export; retry with stable font settings."
                            .into(),
                    );
                    return;
                }
                refreshed.paint(
                    ctx,
                    &ctx.layer_painter(egui::LayerId::background()),
                    index,
                    Pos2::ZERO,
                    1.,
                );
            },
        );
        if let Some(error) = render_error {
            return Err(error);
        }
        let mut textures = HashMap::new();
        update_textures(&mut textures, &output.textures_delta)?;
        let primitives = render_ctx.tessellate(output.shapes, output.pixels_per_point);
        let page = rasterize(primitives, &textures, w, h, output.pixels_per_point)?;
        inspect(index, &page);
        let id = 3 + index * 3;
        let commands = format!(
            "q {} 0 0 {} 0 0 cm /PageImage Do Q\n",
            settings.width_points, settings.height_points
        );
        objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Resources << /XObject << /PageImage {} 0 R >> >> /Contents {} 0 R >>",settings.width_points,settings.height_points,id+1,id+2).into_bytes());
        objects.push(stream(&format!("/Type /XObject /Subtype /Image /Width {w} /Height {h} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /DecodeParms << /Predictor 15 /Colors 3 /BitsPerComponent 8 /Columns {w} >>"),&png_data(&page)?));
        objects.push(stream("", commands.as_bytes()));
        if objects.iter().map(Vec::len).sum::<usize>() > 128 * 1024 * 1024 {
            return Err("PDF exceeds the 128 MiB export limit.".into());
        }
    }
    let mut out = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n".to_vec();
    let mut offsets = vec![0];
    for (i, object) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend(object);
        out.extend(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes());
    for offset in offsets.iter().skip(1) {
        out.extend(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            offsets.len()
        )
        .as_bytes(),
    );
    Ok(out)
}
pub fn export(document: &Document, path: &Path) -> Result<()> {
    if !path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("pdf"))
    {
        return Err("Choose a .pdf destination.".into());
    }
    let data = encode(document, |_, _| {})?;
    crate::storage::write_bytes_checked(path, &data, |_| Ok(()))
}
