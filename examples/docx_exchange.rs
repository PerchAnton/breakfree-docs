//! Reproducible supported-subset interchange fixture; no external application needed.
use breakfree_docs::{docx, model::*, storage};
fn main() -> Result<(), String> {
    let mut d = Document::default();
    d.paragraphs.clear();
    for (text, style) in [
        ("BreakFree Docs exchange", ParagraphStyle::Heading1),
        (
            "English: saved text, bold, italic and underline.",
            ParagraphStyle::Normal,
        ),
        (
            "Українська: збереження документа. Русский: проверка текста.",
            ParagraphStyle::Normal,
        ),
        (
            "Combining: e\u{301}. Emoji: 🙂. XML: <text> & values.\tTab.",
            ParagraphStyle::Normal,
        ),
    ] {
        d.paragraphs.push(Paragraph {
            text: text.into(),
            styles: vec![CharacterFormat::default(); text.chars().count()],
            paragraph_style: style,
            ..Default::default()
        });
    }
    d.paragraphs[1].styles[21..25]
        .iter_mut()
        .for_each(|s| s.bold = Some(true));
    d.paragraphs[1].styles[27..33]
        .iter_mut()
        .for_each(|s| s.italic = Some(true));
    d.paragraphs[1].styles[38..47]
        .iter_mut()
        .for_each(|s| s.underline = Some(true));
    d.paragraphs[2].spacing.left_indent_half_points = Some(36);
    d.paragraphs[2].spacing.first_line_half_points = Some(-18);
    d.validate()?;
    storage::save(&d, std::path::Path::new("examples/DocxExchange.bfdoc"))?;
    docx::save(&d, std::path::Path::new("examples/DocxExchange.docx"))?;
    assert_eq!(
        docx::load(std::path::Path::new("examples/DocxExchange.docx"))?,
        d
    );
    Ok(())
}
