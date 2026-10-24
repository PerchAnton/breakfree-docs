//! Original combined D4 corpus; both formats must retain the exact model.
use breakfree_docs::{commands::Editor, docx, model::*, storage, structure::*};
use image::ImageEncoder;
fn main() -> Result<(), String> {
    let mut e = Editor::default();
    e.replace(0,0,"DOCX exchange review\nEnglish · Українська · Русский · e\u{301}\nLeft\tCenter\tRight\nFirst item\nSecond item\nLandscape section\nHeader, footer and page number restart.", "Fixture")?;
    e.document.paragraphs[0].paragraph_style = ParagraphStyle::Heading1;
    e.document.paragraphs[1]
        .styles
        .iter_mut()
        .for_each(|f| f.family = Some(TextFamily::TimesNewRoman));
    e.document.paragraphs[2]
        .styles
        .iter_mut()
        .for_each(|f| f.family = Some(TextFamily::CourierNew));
    e.document.paragraphs[2]
        .spacing
        .set_tab_positions(&[(288, TabKind::Center), (650, TabKind::Right)])?;
    e.document.paragraphs[2].spacing.tab_leaders = vec![
        TabLeaderStop {
            position_half_points: 288,
            leader: TabLeader::Dots,
        },
        TabLeaderStop {
            position_half_points: 650,
            leader: TabLeader::Dashes,
        },
    ];
    for i in [3, 4] {
        e.document.paragraphs[i].structure.list = Some(ListItem {
            kind: ListKind::Decimal,
            level: 0,
            restart: None,
        });
    }
    e.document.page.header = "BreakFree Docs · D4 review".into();
    e.document.page.footer = "Original synthetic document".into();
    e.document.page.page_numbers = true;
    e.document.paragraphs[5].structure.section = Some(Section {
        width_points: 842,
        height_points: 595,
        header: "Landscape section".into(),
        footer: "Numbering starts at 7".into(),
        page_numbers: true,
        number_start: Some(7),
        ..Default::default()
    });
    let end = e.document.character_count();
    e.selection = (end, end);
    let mut t = Table::new(3, 3)?;
    t.widths_points = vec![100, 180, 100];
    t.repeat_header = true;
    t.rows = vec![
        vec!["Name".into(), "Description".into(), "Value".into()],
        vec!["Україна".into(), "English · Русский".into(), "42".into()],
        vec!["Empty".into(), "".into(), "e\u{301}".into()],
    ];
    e.insert_block(Block::Table(t))?;
    let mut png = Vec::new();
    let pixels: Vec<u8> = (0..160 * 60)
        .flat_map(|i| [100 + (i % 160 / 4) as u8, 80 + (i / 160) as u8, 180])
        .collect();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(&pixels, 160, 60, image::ExtendedColorType::Rgb8)
        .map_err(|e| e.to_string())?;
    let end = e.document.character_count();
    e.selection = (end, end);
    e.insert_block(Block::Picture(Picture {
        png,
        width_points: 240,
        height_points: 90,
        alt: "Original gradient · PNG test".into(),
    }))?;
    e.document.package_properties.insert("core".into(), "<cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\"><dc:title>D4 synthetic exchange</dc:title></cp:coreProperties>".into());
    let path = std::path::Path::new("examples/DocxExtended.docx");
    docx::save(&e.document, path)?;
    assert_eq!(docx::load(path)?, e.document);
    storage::save(
        &e.document,
        std::path::Path::new("examples/DocxExtended.bfdoc"),
    )?;
    assert_eq!(
        storage::load(std::path::Path::new("examples/DocxExtended.bfdoc"))?,
        e.document
    );
    println!("Combined DOCX/native exact-model reopen passed.");
    Ok(())
}
