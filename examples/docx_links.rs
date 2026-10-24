//! Synthetic hyperlink fixture: no link is activated by this generator.
use breakfree_docs::{
    docx,
    model::*,
    storage,
    structure::{Link, ListItem, ListKind},
};
fn main() -> Result<(), String> {
    let mut d = Document::default();
    let p = &mut d.paragraphs[0];
    p.text = "DOCX links: Україна é / Русский / email".into();
    p.styles = vec![CharacterFormat::default(); p.text.chars().count()];
    for (text, target) in [
        ("Україна é", "https://example.org/?a=1&b=2#part"),
        ("Русский", "http://example.org/"),
        ("email", "mailto:test@example.org"),
    ] {
        let byte = p.text.find(text).unwrap();
        let start = p.text[..byte].chars().count();
        let end = start + text.chars().count();
        p.structure.links.push(Link {
            start,
            end,
            target: target.into(),
        });
        for f in &mut p.styles[start..end] {
            f.underline = Some(true);
        }
    }
    p.styles[12].bold = Some(true);
    p.structure.list = Some(ListItem {
        kind: ListKind::Decimal,
        level: 0,
        restart: Some(3),
    });
    docx::save(&d, std::path::Path::new("examples/DocxLinks.docx"))?;
    assert_eq!(
        docx::load(std::path::Path::new("examples/DocxLinks.docx"))?,
        d
    );
    storage::save(&d, std::path::Path::new("examples/DocxLinks.bfdoc"))?;
    println!("Three links, mixed runs and numbering preserved exactly.");
    Ok(())
}
