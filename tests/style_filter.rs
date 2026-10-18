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

#[test]
fn usage_classification_handles_direct_parents_and_history() {
    use breakfree_docs::{
        commands::Editor,
        model::*,
        style_filter::{Usage, UsedStyles},
    };
    let mut doc = Document::default();
    let child = doc
        .add_character_style("Child", CharacterFormat::default())
        .unwrap();
    doc.set_character_parent(child, Some(CharacterStyle::Strong))
        .unwrap();
    doc.paragraphs[0].paragraph_style = ParagraphStyle::Heading1;
    doc.paragraphs[0].text = "ab".into();
    doc.paragraphs[0].styles = vec![
        CharacterFormat {
            character_style: Some(child),
            ..Default::default()
        };
        2
    ];
    let mut editor = Editor::new(doc);
    let initial = UsedStyles::collect(&editor.document);
    assert_eq!(
        initial.paragraph_usage(ParagraphStyle::Heading1),
        Usage::Direct
    );
    assert_eq!(
        initial.paragraph_usage(ParagraphStyle::Normal),
        Usage::Parent
    );
    assert_eq!(
        initial.paragraph_usage(ParagraphStyle::NoSpacing),
        Usage::Unused
    );
    assert_eq!(initial.character_usage(Some(child)), Usage::Direct);
    assert_eq!(
        initial.character_usage(Some(CharacterStyle::Strong)),
        Usage::Parent
    );
    assert_eq!(initial.character_usage(None), Usage::Unused);
    editor
        .apply_paragraph_style(0, 0, ParagraphStyle::Normal)
        .unwrap();
    assert_eq!(
        UsedStyles::collect(&editor.document).paragraph_usage(ParagraphStyle::Normal),
        Usage::Direct
    );
    editor.undo();
    assert_eq!(UsedStyles::collect(&editor.document), initial);
    assert_eq!(
        UsedStyles::collect(&editor.document)
            .paragraph_consumers(&editor.document, ParagraphStyle::Normal),
        vec![ParagraphStyle::Heading1]
    );
    editor.redo();
    assert_eq!(
        UsedStyles::collect(&editor.document).paragraph_usage(ParagraphStyle::Heading1),
        Usage::Unused
    );
    assert!(
        UsedStyles::collect(&editor.document)
            .paragraph_consumers(&editor.document, ParagraphStyle::Normal)
            .is_empty()
    );
    // Parent was encountered through the child first; a later direct reference still wins.
    editor.document.paragraphs[0].styles[1].character_style = Some(CharacterStyle::Strong);
    assert_eq!(
        UsedStyles::collect(&editor.document).character_usage(Some(CharacterStyle::Strong)),
        Usage::Direct
    );
    let reopened = breakfree_docs::storage::decode(
        &breakfree_docs::storage::encode(&editor.document).unwrap(),
    )
    .unwrap();
    assert_eq!(
        UsedStyles::collect(&reopened),
        UsedStyles::collect(&editor.document)
    );
}

#[test]
fn consumers_include_transitive_direct_users_only_and_survive_roundtrip() {
    use breakfree_docs::{model::*, style_filter::UsedStyles};
    let mut doc = Document::default();
    let middle = doc
        .style_catalog
        .add(
            "Middle",
            StyleDefinition {
                parent: Some(ParagraphStyle::Heading1),
                ..Default::default()
            },
        )
        .unwrap();
    let leaf = doc
        .style_catalog
        .add(
            "Український e\u{301}",
            StyleDefinition {
                parent: Some(middle),
                ..Default::default()
            },
        )
        .unwrap();
    doc.paragraphs[0].paragraph_style = leaf;
    let mut second = doc.paragraphs[0].clone();
    second.paragraph_style = ParagraphStyle::Heading1;
    doc.paragraphs.push(second);
    doc.paragraphs.push(doc.paragraphs[0].clone());
    let cm = doc
        .add_character_style("Middle", CharacterFormat::default())
        .unwrap();
    doc.set_character_parent(cm, Some(CharacterStyle::Strong))
        .unwrap();
    let cl = doc
        .add_character_style("Leaf", CharacterFormat::default())
        .unwrap();
    doc.set_character_parent(cl, Some(cm)).unwrap();
    doc.paragraphs[0].text = "abc".into();
    doc.paragraphs[0].styles = vec![
        CharacterFormat {
            character_style: Some(cl),
            ..Default::default()
        },
        CharacterFormat {
            character_style: Some(CharacterStyle::Strong),
            ..Default::default()
        },
        CharacterFormat {
            character_style: Some(cl),
            ..Default::default()
        },
    ];
    let used = UsedStyles::collect(&doc);
    assert_eq!(
        used.paragraph_consumers(&doc, ParagraphStyle::Normal),
        vec![leaf, ParagraphStyle::Heading1]
    );
    assert_eq!(
        used.paragraph_consumers(&doc, ParagraphStyle::Heading1),
        vec![leaf]
    );
    assert_eq!(used.paragraph_consumers(&doc, middle), vec![leaf]);
    assert!(used.paragraph_consumers(&doc, leaf).is_empty());
    assert!(
        used.paragraph_consumers(&doc, ParagraphStyle::NoSpacing)
            .is_empty()
    );
    assert_eq!(
        used.character_consumers(&doc, CharacterStyle::Strong),
        vec![cl]
    );
    assert_eq!(used.character_consumers(&doc, cm), vec![cl]);
    assert!(used.character_consumers(&doc, cl).is_empty());
    let reopened =
        breakfree_docs::storage::decode(&breakfree_docs::storage::encode(&doc).unwrap()).unwrap();
    assert_eq!(
        UsedStyles::collect(&reopened).paragraph_consumers(&reopened, ParagraphStyle::Normal),
        vec![leaf, ParagraphStyle::Heading1]
    );
    // An invalid external cycle is bounded even when the requested target is absent.
    doc.set_character_parent(cm, None).unwrap();
    assert!(
        used.character_consumers(&doc, CharacterStyle::Strong)
            .is_empty()
    );
    doc.style_catalog.heading1.parent = Some(middle);
    assert!(
        used.paragraph_consumers(&doc, ParagraphStyle::NoSpacing)
            .is_empty()
    );
}

#[test]
fn windows_acceptance_fixture_has_transitive_consumers_and_roundtrips() {
    use breakfree_docs::{model::CharacterStyle, storage, style_filter::UsedStyles};
    let doc = storage::decode(include_bytes!("../examples/StyleNavigation.bfdoc")).unwrap();
    let used = UsedStyles::collect(&doc);
    assert_eq!(doc.character_styles.len(), 32);
    assert_eq!(
        used.character_consumers(&doc, CharacterStyle::Custom(32)),
        vec![CharacterStyle::Custom(1), CharacterStyle::Custom(17)]
    );
    assert!(doc.text().contains("Україна Русский e\u{301}"));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("navigation.bfdoc");
    storage::save(&doc, &path).unwrap();
    assert_eq!(storage::load(&path).unwrap(), doc);
}
