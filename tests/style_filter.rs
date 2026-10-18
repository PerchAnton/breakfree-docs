use breakfree_docs::style_filter::Scope;
#[test]
fn scope_and_unicode_name_matching_are_independent() {
    assert!(Scope::All.matches("  НОТАТКА ", "Нотатка Україна", false, true));
    assert!(Scope::Custom.matches("рус", "Русский стиль", true, true));
    assert!(Scope::Character.matches("STRONG", "Strong", true, false));
    assert!(!Scope::Paragraph.matches("", "Strong", true, false));
    assert!(!Scope::Custom.matches("", "Normal", false, false));
    assert!(Scope::Paragraph.matches("e\u{301}", "cafe\u{301}", false, false));
    assert!(!Scope::All.matches("é", "cafe\u{301}", false, false));
    assert!(Scope::All.matches("", "Normal", false, false));
    assert!(!Scope::All.matches("absent", "Normal", false, false));
    let mut scope = Scope::All;
    for _ in 0..4 {
        scope = scope.next();
    }
    assert_eq!(scope, Scope::All);
}

#[test]
fn used_styles_include_parents_not_siblings_or_following_links() {
    use breakfree_docs::{model::*, style_filter::UsedStyles};
    let mut doc = Document::default();
    doc.style_catalog.heading1.next_style = Some(ParagraphStyle::NoSpacing);
    let paragraph_parent = doc
        .style_catalog
        .add(
            "Paragraph parent",
            StyleDefinition {
                parent: Some(ParagraphStyle::Heading1),
                ..Default::default()
            },
        )
        .unwrap();
    let paragraph_child = doc
        .style_catalog
        .add(
            "Paragraph child",
            StyleDefinition {
                parent: Some(paragraph_parent),
                ..Default::default()
            },
        )
        .unwrap();
    let sibling = doc
        .style_catalog
        .add(
            "Sibling",
            StyleDefinition {
                parent: Some(paragraph_parent),
                ..Default::default()
            },
        )
        .unwrap();
    doc.paragraphs[0].paragraph_style = paragraph_child;
    let parent = doc
        .add_character_style("Parent", CharacterFormat::default())
        .unwrap();
    doc.set_character_parent(parent, Some(CharacterStyle::Strong))
        .unwrap();
    let child = doc
        .add_character_style("Child", CharacterFormat::default())
        .unwrap();
    doc.set_character_parent(child, Some(parent)).unwrap();
    let unused = doc
        .add_character_style("Unused", CharacterFormat::default())
        .unwrap();
    doc.set_character_parent(unused, Some(parent)).unwrap();
    doc.paragraphs[0].text = "é".into();
    doc.paragraphs[0].styles = vec![CharacterFormat {
        character_style: Some(child),
        bold: Some(false),
        ..Default::default()
    }];
    let before = doc.clone();
    let used = UsedStyles::collect(&doc);
    assert_eq!(
        used.paragraphs,
        vec![
            paragraph_child,
            paragraph_parent,
            ParagraphStyle::Heading1,
            ParagraphStyle::Normal
        ]
    );
    assert_eq!(used.characters, vec![child, parent, CharacterStyle::Strong]);
    assert!(!used.paragraphs.contains(&sibling));
    assert!(!used.character(Some(unused)));
    assert!(!used.character(None));
    assert_eq!(doc, before);
    let decoded =
        breakfree_docs::storage::decode(&breakfree_docs::storage::encode(&doc).unwrap()).unwrap();
    assert_eq!(UsedStyles::collect(&decoded), used);
}

#[test]
fn empty_paragraphs_pending_typing_and_undo_have_explicit_usage_rules() {
    use breakfree_docs::{commands::Editor, model::*, style_filter::UsedStyles};
    let mut editor = Editor::new(Document::default());
    editor.typing_style.character_style = Some(CharacterStyle::Emphasis);
    let before = UsedStyles::collect(&editor.document);
    assert_eq!(before.paragraphs, vec![ParagraphStyle::Normal]);
    assert!(before.character(None));
    assert!(!before.character(Some(CharacterStyle::Emphasis)));
    editor
        .apply_paragraph_style(0, 0, ParagraphStyle::NoSpacing)
        .unwrap();
    assert!(
        UsedStyles::collect(&editor.document)
            .paragraphs
            .contains(&ParagraphStyle::NoSpacing)
    );
    editor.undo();
    assert_eq!(UsedStyles::collect(&editor.document), before);
    editor.redo();
    assert!(
        UsedStyles::collect(&editor.document)
            .paragraphs
            .contains(&ParagraphStyle::NoSpacing)
    );
}
