//! Reproduce the DOCX numbering fixture and compare it with native-model labels.
use breakfree_docs::{
    docx,
    model::*,
    storage,
    structure::{ListItem, ListKind, ParagraphStructure, labels},
};
fn main() -> Result<(), String> {
    let mut d = Document::default();
    d.paragraphs.clear();
    for (text, list) in [
        ("DOCX list exchange", None),
        ("First numbered item", Some((ListKind::Decimal, 0, Some(3)))),
        ("Nested item", Some((ListKind::Decimal, 1, None))),
        ("Second nested item", Some((ListKind::Decimal, 1, None))),
        ("Deeper restart", Some((ListKind::Decimal, 2, Some(8)))),
        ("Back to outer level", Some((ListKind::Decimal, 0, None))),
        ("Reset nested stream", Some((ListKind::Decimal, 1, None))),
        ("A paragraph between lists", None),
        ("Bullet: Україна", Some((ListKind::Bullet, 0, None))),
        ("Nested bullet: Русский", Some((ListKind::Bullet, 1, None))),
        (
            "Deeper bullet: e\u{301} 🙂",
            Some((ListKind::Bullet, 2, None)),
        ),
        ("New numbered list", Some((ListKind::Decimal, 0, None))),
    ] {
        d.paragraphs.push(Paragraph {
            text: text.into(),
            styles: vec![CharacterFormat::default(); text.chars().count()],
            structure: ParagraphStructure {
                list: list.map(|(kind, level, restart)| ListItem {
                    kind,
                    level,
                    restart,
                }),
                ..Default::default()
            },
            ..Default::default()
        });
    }
    let path = std::path::Path::new("examples/DocxLists.docx");
    docx::save(&d, path)?;
    let reopened = docx::load(path)?;
    assert_eq!(d, reopened);
    assert_eq!(labels(&d), labels(&reopened));
    storage::save(&d, std::path::Path::new("examples/DocxLists.bfdoc"))?;
    for (p, label) in d.paragraphs.iter().zip(labels(&d)) {
        println!("{} {}", label.unwrap_or_default(), p.text);
    }
    Ok(())
}
