#![cfg(feature = "desktop")]
use breakfree_docs::{commands::Editor, page_layout::Layout, structure::*};
use eframe::egui;
#[test]
fn paged_caret_projection_preserves_unicode_and_all_rows() {
    let mut editor = Editor::default();
    editor
        .replace(
            0,
            0,
            &vec!["English Україна Русский e\u{301}"; 90].join("\n"),
            "Fixture",
        )
        .unwrap();
    editor.document.page.height_points = 300;
    let ctx = egui::Context::default();
    breakfree_docs::layout::configure_fonts(&ctx);
    let _ = ctx.run(Default::default(), |ctx| {
        let layout = Layout::build(ctx, &editor.document).unwrap();
        assert!(layout.plan.pages.len() > 2);
        let galley = layout.editable_galley(ctx, &editor.document, 1.25);
        assert_eq!(galley.job.text, editor.document.text());
        for index in [0, 7, 28, 100, editor.document.character_count()] {
            let cursor = egui::text::CCursor::new(index);
            assert!(galley.pos_from_cursor(cursor).min.is_finite());
        }
    });
}
#[test]
fn raster_pdf_contains_body_ink_and_retains_destination_on_error() {
    let mut editor = Editor::default();
    editor
        .replace(0, 0, "Visible body Україна Русский e\u{301}", "Fixture")
        .unwrap();
    let mut pages = 0;
    let bytes = breakfree_docs::pdf_output::encode(&editor.document, |_, page| {
        pages += 1;
        let ink = (140..200)
            .flat_map(|y| (140..900).map(move |x| (y * page.width + x) * 3))
            .filter(|at| page.rgb[*at] < 180)
            .count();
        assert!(ink > 100, "Body glyphs disappeared: {ink} dark pixels");
    })
    .unwrap();
    assert_eq!(pages, 1);
    assert!(bytes.starts_with(b"%PDF-1.4"));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("previous.pdf");
    std::fs::write(&path, &bytes).unwrap();
    editor.document.page.left_points = 600;
    assert!(breakfree_docs::pdf_output::export(&editor.document, &path).is_err());
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}
#[test]
fn png_payload_corruption_is_rejected_and_picture_roundtrips() {
    use image::ImageEncoder;
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(&[90, 60, 130], 1, 1, image::ExtendedColorType::Rgb8)
        .unwrap();
    let picture = Picture {
        png,
        width_points: 72,
        height_points: 72,
        alt: "Synthetic".into(),
    };
    picture.validate_pixels().unwrap();
    let mut editor = Editor::default();
    editor
        .insert_block(Block::Picture(picture.clone()))
        .unwrap();
    assert_eq!(
        breakfree_docs::storage::decode(
            &breakfree_docs::storage::encode(&editor.document).unwrap()
        )
        .unwrap(),
        editor.document
    );
    for index in [20, 40, picture.png.len() - 1] {
        let mut broken = picture.clone();
        broken.png[index] ^= 1;
        assert!(broken.validate_pixels().is_err());
        let before = editor.document.clone();
        assert!(editor.insert_block(Block::Picture(broken)).is_err());
        assert_eq!(before, editor.document);
    }
}

#[test]
fn all_report_pages_keep_headers_and_every_text_row() {
    let document =
        breakfree_docs::storage::load(std::path::Path::new("examples/StructuredReport.bfdoc"))
            .unwrap();
    let ctx = egui::Context::default();
    breakfree_docs::layout::configure_fonts(&ctx);
    ctx.set_pixels_per_point(2.);
    let mut expected = Vec::new();
    let _ = ctx.run(Default::default(), |ctx| {
        let layout = Layout::build(ctx, &document).unwrap();
        for page in &layout.plan.pages {
            let mut bands = vec![(f32::from(page.settings.top_points) / 2. - 5., 9.)];
            for item in &page.items {
                if let breakfree_docs::page_layout::Content::Text(g) =
                    &layout.content[item.paragraph]
                {
                    let row = &g.rows[item.row];
                    if row.glyphs.iter().any(|g| !g.chr.is_whitespace()) {
                        bands.push((f32::from(page.settings.top_points) + item.y, row.size.y));
                    }
                }
            }
            expected.push((page.settings.clone(), bands));
        }
    });
    breakfree_docs::pdf_output::encode(&document, |index, page| {
        let (section, bands) = &expected[index];
        for &(top, height) in bands {
            let x1 = usize::from(section.left_points) * 2;
            let x2 = usize::from(section.width_points - section.right_points) * 2;
            let mut ink = 0;
            for y in (top * 2.).floor() as usize..((top + height) * 2.).ceil() as usize {
                for x in x1..x2 {
                    if page.rgb[(y * page.width + x) * 3] < 160 {
                        ink += 1;
                    }
                }
            }
            assert!(
                ink > 100,
                "Page {}: missing row at {top}, only {ink} ink pixels",
                index + 1
            );
        }
    })
    .unwrap();
    assert_eq!(expected.len(), 6);
}
