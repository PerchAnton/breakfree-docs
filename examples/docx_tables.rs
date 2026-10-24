//! Synthetic fixed-grid table fixture with native/DOCX equality checks.
use breakfree_docs::{
    commands::Editor,
    docx, storage,
    structure::{Block, Table},
};
fn main() -> Result<(), String> {
    let mut e = Editor::default();
    e.replace(0, 0, "DOCX table exchange", "Type")?;
    e.insert_block(Block::Table(Table {
        widths_points: vec![90, 180, 90],
        rows: vec![
            vec!["Name".into(), "Description".into(), "Value".into()],
            vec![
                "Україна".into(),
                "English and Русский\nSecond line: é".into(),
                "42".into(),
            ],
            vec!["Empty cell".into(), "".into(), "& < >".into()],
        ],
        repeat_header: true,
    }))?;
    let path = std::path::Path::new("examples/DocxTables.docx");
    docx::save(&e.document, path)?;
    assert_eq!(docx::load(path)?, e.document);
    storage::save(
        &e.document,
        std::path::Path::new("examples/DocxTables.bfdoc"),
    )?;
    println!("3×3 table, widths, repeated header and Unicode cells preserved exactly.");
    Ok(())
}
