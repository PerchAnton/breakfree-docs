//! Synthetic D2/D3 corpus generator; no customer content or reference artwork.
use breakfree_docs::{commands::Editor, model::*, storage, structure::*};
fn main() -> Result<(), String> {
    let mut e = Editor::default();
    e.replace(0,0,"BreakFree Docs report\nEnglish · Українська · Русский · e\u{301}\nFirst item\nSecond item\nSource link\nTable follows\nFigures and sections", "Fixture")?;
    e.document.paragraphs[0].paragraph_style = ParagraphStyle::Heading1;
    e.document.paragraphs[0].structure.keep_with_next = true;
    for i in [2, 3] {
        e.document.paragraphs[i].structure.list = Some(ListItem {
            kind: ListKind::Decimal,
            level: 0,
            restart: None,
        });
    }
    e.document.paragraphs[4].structure.links = vec![Link {
        start: 0,
        end: 11,
        target: "https://example.com".into(),
    }];
    e.document.page = Section {
        width_points: 595,
        height_points: 842,
        page_numbers: true,
        header: "BreakFree Docs · synthetic report".into(),
        footer: "Local document · validation fixture".into(),
        ..Default::default()
    };
    let mut table = Table::new(22, 3)?;
    table.widths_points = vec![90, 180, 90];
    table.repeat_header = true;
    table.rows[0] = vec!["Item".into(), "Description".into(), "Value".into()];
    for i in 1..table.rows.len() {
        table.rows[i] = vec![
            format!("{i}"),
            format!("Україна · Русский · item {i}"),
            format!("{}", i * 7),
        ];
    }
    e.selection = (
        e.document.paragraphs[..6]
            .iter()
            .map(|p| p.styles.len() + 1)
            .sum(),
        e.document.paragraphs[..6]
            .iter()
            .map(|p| p.styles.len() + 1)
            .sum(),
    );
    e.insert_block(Block::Table(table))?;
    let mut png = Vec::new();
    let mut pixels = vec![0u8; 160 * 90 * 3];
    for y in 0..90 {
        for x in 0..160 {
            let at = (y * 160 + x) * 3;
            pixels[at] = (95 + x / 3) as u8;
            pixels[at + 1] = (70 + y) as u8;
            pixels[at + 2] = 190;
        }
    }
    use image::ImageEncoder;
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(&pixels, 160, 90, image::ExtendedColorType::Rgb8)
        .map_err(|e| e.to_string())?;
    let end = e.document.character_count();
    e.selection = (end, end);
    e.insert_block(Block::Picture(Picture {
        png,
        width_points: 216,
        height_points: 122,
        alt: "Original purple gradient test image".into(),
    }))?;
    let end = e.document.character_count();
    e.selection = (end, end);
    e.replace(end,end,"Landscape section\nThis section uses its own page size and numbering.\nLong paragraphs below test line splitting and page transitions.","Fixture")?;
    let n = e.document.paragraphs.len();
    e.document.paragraphs[n - 3].structure.section = Some(Section {
        width_points: 842,
        height_points: 595,
        number_start: Some(10),
        ..e.document.page.clone()
    });
    for i in 0..18 {
        let text = format!(
            "\nParagraph {}. {}",
            i + 1,
            "English text, український текст, русский текст and combining e\u{301}. ".repeat(10)
        );
        let end = e.document.character_count();
        e.replace(end, end, &text, "Fixture")?;
    }
    e.document.validate()?;
    let args: Vec<_> = std::env::args().collect();
    let path = std::path::Path::new(
        args.get(1)
            .map_or("/tmp/StructuredReport.bfdoc", String::as_str),
    );
    storage::save(&e.document, path)?;
    if let Some(pdf) = args.get(2) {
        breakfree_docs::pdf_output::export(&e.document, std::path::Path::new(pdf))?;
    }
    println!("Saved {}", path.display());
    Ok(())
}
