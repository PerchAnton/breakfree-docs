use super::*;
fn sample() -> Document {
    let mut d = Document::default();
    d.paragraphs[0].text = "English & <test> Україна Русский e\u{301} 🙂\tend".into();
    d.paragraphs[0].styles = vec![CharacterFormat::default(); d.paragraphs[0].text.chars().count()];
    d.paragraphs[0].styles[0].bold = Some(true);
    d.paragraphs[0].styles[1].italic = Some(true);
    d.paragraphs[0].styles[2].underline = Some(true);
    d.paragraphs[0].styles[3].font_size = Some(FontSize::try_from(25).unwrap());
    d.paragraphs[0].spacing.first_line_half_points = Some(-24);
    d.paragraphs[0].spacing.space_before_half_points = Some(10);
    d.paragraphs[0].alignment = Alignment::Justify;
    d.paragraphs.push(Paragraph {
        text: "Second paragraph".into(),
        styles: vec![CharacterFormat::default(); 16],
        paragraph_style: ParagraphStyle::Heading1,
        ..Default::default()
    });
    d.style_catalog.custom.push(CustomStyle {
        id: 7,
        name: "Body child".into(),
        definition: StyleDefinition {
            parent: Some(ParagraphStyle::Normal),
            next_style: Some(ParagraphStyle::Heading1),
            left_indent_half_points: Some(20),
            ..Default::default()
        },
    });
    d.character_styles.push(NamedCharacterStyle {
        id: 12,
        name: "Underlined".into(),
        parent: Some(CharacterStyle::Emphasis),
        format: CharacterFormat {
            underline: Some(true),
            ..Default::default()
        },
    });
    d.paragraphs[0].paragraph_style = ParagraphStyle::Custom(7);
    d.paragraphs[0].styles[8].character_style = Some(CharacterStyle::Custom(12));
    d
}
#[test]
fn unicode_styles_geometry_and_native_roundtrip() {
    let d = sample();
    let bytes = encode(&d).unwrap();
    let reopened = decode(&bytes).unwrap();
    assert_eq!(d, reopened);
    assert_eq!(
        d,
        crate::storage::decode(&crate::storage::encode(&reopened).unwrap()).unwrap()
    );
}
#[test]
fn no_gui_file_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Example.docx");
    let d = sample();
    save(&d, &path).unwrap();
    assert_eq!(load(&path).unwrap(), d);
}
fn mutate(f: impl FnOnce(&mut Parts)) -> Vec<u8> {
    let mut p = package::read(&encode(&sample()).unwrap()).unwrap();
    f(&mut p);
    package::write(&p).unwrap()
}
#[test]
fn rejects_parts_relationships_and_unsupported_properties() {
    for (part, from, to) in [
        ("word/document.xml", "<w:body>", "<w:body><w:tbl/>"),
        (
            "word/document.xml",
            "<w:rPr>",
            "<w:rPr><w:color w:val=\"FF0000\"/>",
        ),
        ("word/document.xml", "<w:body>", "<w:body><w:ins/>"),
        (
            "word/_rels/document.xml.rels",
            "Target=\"styles.xml\"",
            "Target=\"https://example.org/\" TargetMode=\"External\"",
        ),
        (
            "[Content_Types].xml",
            MAIN_TYPE,
            "application/vnd.ms-word.document.macroEnabled.main+xml",
        ),
    ] {
        let b = mutate(|p| {
            let s = String::from_utf8(p[part].clone())
                .unwrap()
                .replacen(from, to, 1);
            p.insert(part.into(), s.into_bytes());
        });
        assert!(decode(&b).is_err(), "{part}: {to}");
    }
    let b = mutate(|p| {
        p.insert("word/comments.xml".into(), b"<comments/>".to_vec());
    });
    assert!(decode(&b).unwrap_err().contains("comments.xml"));
}
#[test]
fn rejects_zip_corruption_truncation_duplicates_and_expansion() {
    let bytes = encode(&sample()).unwrap();
    for end in [0, 4, 100, bytes.len() - 1] {
        assert!(decode(&bytes[..end]).is_err());
    }
    let mut broken = bytes.clone();
    broken[100] ^= 1;
    assert!(decode(&broken).is_err());
    let mut parts = Parts::new();
    parts.insert("../escape".into(), vec![]);
    assert!(package::write(&parts).is_err());
    parts.clear();
    parts.insert("bomb".into(), vec![0; package::PART_MAX + 1]);
    assert!(package::write(&parts).is_err());
    let mut huge = bytes;
    let pos = huge.windows(4).position(|b| b == b"PK\x01\x02").unwrap();
    huge[pos + 24..pos + 28].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(package::read(&huge).is_err());
}
#[test]
fn rejects_entities_depth_and_unknown_namespaces() {
    assert!(xml::parse(b"<x><?xml version='1.0'?></x>").is_err());
    let mut invalid = sample();
    invalid.paragraphs[0].text.push('\u{ffff}');
    invalid.paragraphs[0]
        .styles
        .push(CharacterFormat::default());
    assert!(encode(&invalid).unwrap_err().contains("XML 1.0"));
    for xml in [
        "<!DOCTYPE x [<!ENTITY a 'bad'>]><x>&a;</x>".to_owned(),
        "<x>&unknown;</x>".into(),
        "<x/><x/>".into(),
        format!("{}{}", "<x>".repeat(66), "</x>".repeat(66)),
        "<w:x/>".into(),
        "<x a='1' a='2'/>".into(),
    ] {
        assert!(xml::parse(xml.as_bytes()).is_err(), "{xml}");
    }
    let b = mutate(|p| {
        let s = String::from_utf8(p["word/document.xml"].clone())
            .unwrap()
            .replace(W, "urn:wrong");
        p.insert("word/document.xml".into(), s.into_bytes());
    });
    assert!(decode(&b).is_err());
}
#[test]
fn rejects_unknown_style_cycles_and_toggle_loss() {
    let b = mutate(|p| {
        let s = String::from_utf8(p["word/styles.xml"].clone())
            .unwrap()
            .replace(
                "<w:basedOn w:val=\"Normal\"/>",
                "<w:basedOn w:val=\"BFp7\"/>",
            );
        p.insert("word/styles.xml".into(), s.into_bytes());
    });
    assert!(decode(&b).is_err());
    let mut d = sample();
    d.style_catalog.custom[0].definition.parent = Some(ParagraphStyle::Heading1);
    d.style_catalog.custom[0].definition.character.bold = Some(false);
    assert!(encode(&d).unwrap_err().contains("toggle"));
}
#[test]
fn failed_export_and_write_preserve_previous_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("previous.docx");
    let d = sample();
    save(&d, &path).unwrap();
    let previous = std::fs::read(&path).unwrap();
    let mut unsupported = d.clone();
    unsupported.paragraphs[0].spacing.tab_interval_half_points = Some(80);
    assert!(save(&unsupported, &path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), previous);
    for stage in [
        crate::storage::SaveStage::Write,
        crate::storage::SaveStage::Sync,
        crate::storage::SaveStage::Replace,
    ] {
        assert!(
            crate::storage::write_bytes_checked(
                &path,
                &encode(&Document::default()).unwrap(),
                |s| if s == stage {
                    Err("injected".into())
                } else {
                    Ok(())
                }
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), previous);
    }
}
#[test]
fn zip_descriptor_is_checked() {
    let parts = Parts::from([("x".into(), b"data".to_vec())]);
    let mut b = package::write(&parts).unwrap();
    let cd = b.windows(4).position(|b| b == b"PK\x01\x02").unwrap();
    let crc = b[14..18].to_vec();
    let sizes = b[18..26].to_vec();
    let mut descriptor = b"PK\x07\x08".to_vec();
    descriptor.extend(crc);
    descriptor.extend(sizes);
    b.splice(cd..cd, descriptor);
    let cd = cd + 16;
    let eocd = b.len() - 22;
    b[6..8].copy_from_slice(&0x808u16.to_le_bytes());
    b[cd + 8..cd + 10].copy_from_slice(&0x808u16.to_le_bytes());
    b[eocd + 16..eocd + 20].copy_from_slice(&(cd as u32).to_le_bytes());
    assert_eq!(package::read(&b).unwrap(), parts);
    b[cd - 12] ^= 1;
    assert!(package::read(&b).is_err());
}
#[test]
fn namespace_prefix_is_not_identity() {
    let b = mutate(|p| {
        for name in ["word/document.xml", "word/styles.xml"] {
            let s = String::from_utf8(p[name].clone())
                .unwrap()
                .replace("w:", "q:")
                .replace("xmlns:w=", "xmlns:q=");
            p.insert(name.into(), s.into_bytes());
        }
    });
    assert_eq!(decode(&b).unwrap(), sample());
}

fn list_document() -> Document {
    use crate::structure::{ListItem, ListKind};
    let mut d = Document::default();
    d.paragraphs.clear();
    for i in 0..90 {
        let text = format!("Item {i} Україна e\u{301}");
        let list = if i % 19 == 18 {
            None
        } else {
            Some(ListItem {
                kind: if i < 50 {
                    ListKind::Decimal
                } else {
                    ListKind::Bullet
                },
                level: (i % 9) as u8,
                restart: if i % 11 == 0 { Some(7) } else { None },
            })
        };
        d.paragraphs.push(Paragraph {
            text: text.clone(),
            styles: vec![CharacterFormat::default(); text.chars().count()],
            structure: crate::structure::ParagraphStructure {
                list,
                ..Default::default()
            },
            ..Default::default()
        });
    }
    d
}
#[test]
fn nested_lists_restarts_gaps_and_kinds_roundtrip_exactly() {
    let d = list_document();
    // Word for Mac continued a new outer list when abstracts were shared.
    let parts = package::read(&encode(&d).unwrap()).unwrap();
    let numbering = part(&parts, "word/numbering.xml").unwrap();
    let references: Vec<_> = numbering
        .children
        .iter()
        .filter(|n| n.name == "num")
        .map(|n| n.child("abstractNumId").unwrap().val().unwrap())
        .collect();
    assert_eq!(
        references.len(),
        references
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    );
    let reopened = decode(&encode(&d).unwrap()).unwrap();
    assert_eq!(reopened, d);
    assert_eq!(
        crate::structure::labels(&reopened),
        crate::structure::labels(&d)
    );
    let native = crate::storage::decode(&crate::storage::encode(&reopened).unwrap()).unwrap();
    assert_eq!(native, d);
}
#[test]
fn imported_lists_share_editing_and_undo() {
    let d = list_document();
    let mut e = crate::commands::Editor::new(decode(&encode(&d).unwrap()).unwrap());
    e.selection = (0, 0);
    e.restart_list(12).unwrap();
    let after = e.document.clone();
    assert_ne!(after, d);
    assert_eq!(decode(&encode(&after).unwrap()).unwrap(), after);
    e.undo();
    assert_eq!(e.document, d);
    e.redo();
    assert_eq!(e.document, after);
}

// Independently authored markup, not produced by the DOCX exporter.
fn external_minimal() -> Parts {
    Parts::from([
        ("[Content_Types].xml".into(),format!("<Types xmlns='{CT}'><Default Extension='xml' ContentType='application/xml'/><Default Extension='rels' ContentType='application/vnd.openxmlformats-package.relationships+xml'/><Override PartName='/text/main.xml' ContentType='{MAIN_TYPE}'/><Override PartName='/numbering.xml' ContentType='{}'/></Types>",opc::NUMBER_TYPE).into_bytes()),
        ("_rels/.rels".into(),format!("<Relationships xmlns='{REL}'><Relationship Id='main' Type='{OFFICE_REL}officeDocument' Target='text/main.xml'/></Relationships>").into_bytes()),
        ("text/_rels/main.xml.rels".into(),format!("<Relationships xmlns='{REL}'><Relationship Id='numbers' Type='{OFFICE_REL}numbering' Target='../numbering.xml'/></Relationships>").into_bytes()),
        ("text/main.xml".into(),format!("<x:document xmlns:x='{W}'><x:body><x:p><x:pPr><x:numPr><x:numId x:val='7'/></x:numPr></x:pPr><x:r><x:t>First</x:t></x:r></x:p><x:p><x:pPr><x:numPr><x:ilvl x:val='0'/><x:numId x:val='7'/></x:numPr></x:pPr><x:r><x:t>Second</x:t></x:r></x:p></x:body></x:document>").into_bytes()),
        ("numbering.xml".into(),format!("<w:numbering xmlns:w='{W}'><w:abstractNum w:abstractNumId='5'><w:lvl w:ilvl='0'><w:start w:val='3'/><w:numFmt w:val='decimal'/><w:lvlText w:val='%1.'/><w:pPr><w:ind w:left='480' w:hanging='360'/></w:pPr></w:lvl></w:abstractNum><w:num w:numId='7'><w:abstractNumId w:val='5'/></w:num></w:numbering>").into_bytes()),
    ])
}
#[test]
fn external_relocated_parts_optional_styles_and_numbering() {
    let d = decode(&package::write(&external_minimal()).unwrap()).unwrap();
    assert_eq!(d.text(), "First\nSecond");
    assert_eq!(
        crate::structure::labels(&d),
        vec![Some("3.".into()), Some("4.".into())]
    );
    assert_eq!(
        d.style_catalog
            .resolve(ParagraphStyle::Normal)
            .line_height_percent,
        100
    );
    assert_eq!(
        d.style_catalog
            .resolve(ParagraphStyle::Normal)
            .space_after_half_points,
        0
    );
    assert_eq!(decode(&encode(&d).unwrap()).unwrap(), d);
}
#[test]
fn optional_styles_and_document_defaults_preserve_inheritance() {
    let mut p = package::read(&encode(&Document::default()).unwrap()).unwrap();
    p.insert("word/styles.xml".into(),format!("<w:styles xmlns:w='{W}'><w:docDefaults><w:pPrDefault><w:pPr><w:spacing w:after='40' w:before='10' w:lineRule='auto' w:line='264'/></w:pPr></w:pPrDefault><w:rPrDefault><w:rPr><w:sz w:val='24'/><w:rFonts w:hAnsi='Arial' w:ascii='Arial'/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:type='paragraph' w:styleId='Normal' w:default='true'><w:name w:val='normal'/></w:style></w:styles>").into_bytes());
    let d = decode(&package::write(&p).unwrap()).unwrap();
    let resolved = d.style_catalog.resolve(ParagraphStyle::Normal);
    assert_eq!(u16::from(resolved.character.font_size), 24);
    assert_eq!(resolved.line_height_percent, 110);
    assert_eq!(resolved.space_before_half_points, 1);
    assert_eq!(resolved.space_after_half_points, 4);
    assert!(d.style_catalog.normal.parent.is_some());
    assert_eq!(decode(&encode(&d).unwrap()).unwrap(), d);
}
#[test]
fn unsupported_numbering_and_package_targets_are_not_flattened() {
    for (from, to) in [
        ("decimal", "lowerRoman"),
        ("%1.", "%1.%2."),
        ("w:left='480'", "w:left='720'"),
        (
            "<w:start w:val='3'/>",
            "<w:start w:val='3'/><w:lvlRestart w:val='0'/>",
        ),
        ("w:abstractNumId='5'", "w:abstractNumId='9'"),
    ] {
        let mut p = external_minimal();
        let n = String::from_utf8(p["numbering.xml"].clone())
            .unwrap()
            .replacen(from, to, 1);
        p.insert("numbering.xml".into(), n.into_bytes());
        assert!(decode(&package::write(&p).unwrap()).is_err(), "{to}");
    }
    for target in [
        "../../numbering.xml",
        "https://example.org/x",
        "%2e%2e/numbering.xml",
    ] {
        let mut p = external_minimal();
        let r = String::from_utf8(p["text/_rels/main.xml.rels"].clone())
            .unwrap()
            .replace("../numbering.xml", target);
        p.insert("text/_rels/main.xml.rels".into(), r.into_bytes());
        assert!(decode(&package::write(&p).unwrap()).is_err());
    }
}
#[test]
fn disconnected_numbering_is_rejected_before_silent_conversion() {
    let mut p = external_minimal();
    let s = String::from_utf8(p["text/main.xml"].clone())
        .unwrap()
        .replacen(
            "</x:p><x:p>",
            "</x:p><x:p><x:r><x:t>Gap</x:t></x:r></x:p><x:p>",
            1,
        );
    p.insert("text/main.xml".into(), s.into_bytes());
    assert!(
        decode(&package::write(&p).unwrap())
            .unwrap_err()
            .contains("discontinuous")
    );
}
#[test]
fn docx_save_retains_path_history_and_detects_external_changes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("primary.docx");
    let d = list_document();
    save(&d, &path).unwrap();
    let mut s = crate::session::Session::open(&path).unwrap();
    assert_eq!(s.path.as_deref(), Some(path.as_path()));
    assert!(s.is_docx());
    s.editor.selection = (0, 0);
    s.editor.restart_list(9).unwrap();
    let history = s.editor.undo_stack.len();
    assert!(s.dirty());
    assert!(
        s.can_leave(crate::session::LeaveChoice::Save, None)
            .unwrap()
    );
    assert!(!s.dirty());
    assert_eq!(s.editor.undo_stack.len(), history);
    assert_eq!(load(&path).unwrap(), s.editor.document);
    s.editor.undo();
    assert!(s.dirty());
    s.editor.redo();
    assert!(!s.dirty());
    std::fs::write(&path, b"changed by another editor").unwrap();
    s.editor.restart_list(10).unwrap();
    assert!(s.save(&path).unwrap_err().contains("outside this window"));
    assert_eq!(std::fs::read(&path).unwrap(), b"changed by another editor");
    std::fs::create_dir(dir.path().join("alias")).unwrap();
    let alias = dir.path().join("alias/../primary.docx");
    assert!(s.save(&alias).unwrap_err().contains("outside this window"));
    assert!(s.dirty());
}
#[test]
fn unsupported_primary_save_keeps_destination_and_offers_native_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("primary.docx");
    save(&Document::default(), &path).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mut s = crate::session::Session::open(&path).unwrap();
    s.editor.document.paragraphs[0]
        .spacing
        .tab_interval_half_points = Some(80);
    assert!(s.save(&path).is_err());
    assert!(s.dirty());
    assert_eq!(s.path.as_deref(), Some(path.as_path()));
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let secondary = dir.path().join("optional.bfdoc");
    s.save(&secondary).unwrap();
    assert_eq!(crate::storage::load(&secondary).unwrap(), s.editor.document);
    assert!(!s.dirty());
    assert!(!s.is_docx());
}

fn linked_document() -> Document {
    use crate::structure::Link;
    let mut d = sample();
    d.paragraphs[0].structure.links = vec![
        Link {
            start: 0,
            end: 7,
            target: "https://example.org/?a=1&b=%D1%82#part".into(),
        },
        Link {
            start: 7,
            end: 9,
            target: "mailto:test@example.org?subject=Hello%20world".into(),
        },
        Link {
            start: 10,
            end: d.paragraphs[0].styles.len(),
            target: "http://example.org/".into(),
        },
    ];
    d
}
#[test]
fn hyperlinks_preserve_ranges_runs_unicode_and_native_roundtrip() {
    let d = linked_document();
    let bytes = encode(&d).unwrap();
    assert_eq!(decode(&bytes).unwrap(), d);
    assert_eq!(
        crate::storage::decode(&crate::storage::encode(&d).unwrap()).unwrap(),
        d
    );
    let parts = package::read(&bytes).unwrap();
    let xml = String::from_utf8(parts["word/document.xml"].clone()).unwrap();
    assert_eq!(xml.matches("<w:hyperlink ").count(), 3);
    assert_eq!(xml.matches("</w:hyperlink>").count(), 3);
    assert!(
        String::from_utf8(parts["word/_rels/document.xml.rels"].clone())
            .unwrap()
            .contains("&amp;b=%D1%82#part")
    );
}
fn external_links() -> Parts {
    let mut p = external_minimal();
    let rel = String::from_utf8(p["text/_rels/main.xml.rels"].clone()).unwrap().replace("</Relationships>", &format!("<Relationship Id='web' Type='{OFFICE_REL}hyperlink' Target='https://example.org/?a=1&amp;b=2' TargetMode='External'/></Relationships>"));
    p.insert("text/_rels/main.xml.rels".into(), rel.into_bytes());
    p.insert("text/main.xml".into(),format!("<x:document xmlns:x='{W}' xmlns:q='{}'><x:body><x:p><x:pPr><x:numPr><x:numId x:val='7'/></x:numPr></x:pPr><x:r><x:t>Go </x:t></x:r><x:hyperlink q:id='web'><x:r><x:rPr><x:b/></x:rPr><x:t>Україна </x:t></x:r><x:r><x:t>é🙂</x:t></x:r></x:hyperlink><x:r><x:t> / </x:t></x:r><x:hyperlink q:id='web'><x:r><x:t>again</x:t></x:r></x:hyperlink></x:p></x:body></x:document>",OFFICE_REL.trim_end_matches('/')).into_bytes());
    p
}
#[test]
fn external_links_with_shared_relationship_and_list_import_without_activation() {
    let d = decode(&package::write(&external_links()).unwrap()).unwrap();
    let p = &d.paragraphs[0];
    assert_eq!(p.text, "Go Україна é🙂 / again");
    assert_eq!(
        (p.structure.links[0].start, p.structure.links[0].end),
        (3, 14)
    );
    assert_eq!(p.structure.links[0].target, "https://example.org/?a=1&b=2");
    assert_eq!(p.structure.links[1].target, p.structure.links[0].target);
    assert_eq!(p.styles[3].bold, Some(true));
    assert_eq!(crate::structure::labels(&d), vec![Some("3.".into())]);
    assert_eq!(decode(&encode(&d).unwrap()).unwrap(), d);
}
#[test]
fn hyperlink_relationships_and_unrepresented_properties_fail_closed() {
    for (path, from, to) in [
        (
            "text/_rels/main.xml.rels",
            "https://example.org/?a=1&amp;b=2",
            "file:///tmp/private",
        ),
        (
            "text/_rels/main.xml.rels",
            "https://example.org/?a=1&amp;b=2",
            "javascript:alert(1)",
        ),
        (
            "text/_rels/main.xml.rels",
            "TargetMode='External'",
            "TargetMode='Internal'",
        ),
        ("text/_rels/main.xml.rels", "TargetMode='External'", ""),
        ("text/_rels/main.xml.rels", "Id='web'", "Id='numbers'"),
        ("text/main.xml", "q:id='web'", "q:id='absent'"),
        ("text/main.xml", "q:id='web'", "x:anchor='bookmark'"),
        ("text/main.xml", "q:id='web'", "q:id='web' x:tooltip='tip'"),
        (
            "text/main.xml",
            "q:id='web'",
            "q:id='web' x:history='false'",
        ),
        (
            "text/main.xml",
            "<x:hyperlink q:id='web'>",
            "<x:hyperlink q:id='web'><x:hyperlink/>",
        ),
    ] {
        let mut p = external_links();
        let original = String::from_utf8(p[path].clone()).unwrap();
        assert!(original.contains(from));
        p.insert(path.into(), original.replacen(from, to, 1).into_bytes());
        assert!(decode(&package::write(&p).unwrap()).is_err(), "{path} {to}");
    }
    let mut p = external_links();
    p.insert(
        "text/main.xml".into(),
        format!("<w:document xmlns:w='{W}'><w:body><w:p/></w:body></w:document>").into_bytes(),
    );
    assert!(
        decode(&package::write(&p).unwrap())
            .unwrap_err()
            .contains("Unreferenced")
    );
}
#[test]
fn hyperlink_grapheme_boundaries_and_empty_ranges_are_rejected() {
    for text in [
        "<x:hyperlink q:id='web'><x:r><x:t>e</x:t></x:r></x:hyperlink><x:r><x:t>́</x:t></x:r>",
        "<x:hyperlink q:id='web'><x:r><x:t/></x:r></x:hyperlink>",
    ] {
        let mut p = external_links();
        p.insert("text/main.xml".into(),format!("<x:document xmlns:x='{W}' xmlns:q='{}'><x:body><x:p>{text}</x:p></x:body></x:document>",OFFICE_REL.trim_end_matches('/')).into_bytes());
        assert!(decode(&package::write(&p).unwrap()).is_err());
    }
}
#[test]
fn imported_hyperlink_edit_undo_redo_and_primary_save() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("links.docx");
    let d = linked_document();
    save(&d, &path).unwrap();
    let mut session = crate::session::Session::open(&path).unwrap();
    session.editor.selection = (0, 7);
    session
        .editor
        .set_link(Some("https://example.org/revised"))
        .unwrap();
    let after = session.editor.document.clone();
    assert_ne!(after, d);
    session.save(&path).unwrap();
    assert_eq!(load(&path).unwrap(), after);
    session.editor.undo();
    assert_eq!(session.editor.document, d);
    session.editor.redo();
    assert_eq!(session.editor.document, after);
    session.editor.set_link(None).unwrap();
    assert_eq!(
        session.editor.document.paragraphs[0].structure.links.len(),
        2
    );
    session.editor.undo();
    assert_eq!(session.editor.document, after);
}

fn table_document() -> Document {
    use crate::structure::{Block, Table};
    let mut e = crate::commands::Editor::default();
    e.replace(0, 0, "Before table", "Type").unwrap();
    e.insert_block(Block::Table(Table {
        widths_points: vec![72, 144, 96],
        rows: vec![
            vec!["Name".into(), "Description".into(), "Value".into()],
            vec![
                "Україна".into(),
                "Русский é 🙂\nSecond line\tvalue".into(),
                "".into(),
            ],
            vec!["& < >".into(), " trailing \n".into(), "42".into()],
        ],
        repeat_header: true,
    }))
    .unwrap();
    e.document
}
#[test]
fn tables_preserve_cells_widths_header_and_native_roundtrip() {
    for header in [false, true] {
        let mut d = table_document();
        if let Some(crate::structure::Block::Table(t)) = &mut d.paragraphs[1].structure.block {
            t.repeat_header = header;
        }
        let b = encode(&d).unwrap();
        assert_eq!(decode(&b).unwrap(), d);
        assert_eq!(
            crate::storage::decode(&crate::storage::encode(&d).unwrap()).unwrap(),
            d
        );
        let p = package::read(&b).unwrap();
        let xml = String::from_utf8(p["word/document.xml"].clone()).unwrap();
        assert!(xml.contains("<w:tbl>"));
        assert!(!xml.contains(crate::structure::OBJECT));
        assert_eq!(xml.contains("<w:tblHeader/>"), header);
    }
}
// Independent fixed-grid markup, not generated by the writer.
fn external_table() -> Parts {
    let mut p = external_minimal();
    let mut body = String::from(
        "<x:tbl><x:tblPr><x:tblW x:type='dxa' x:w='2880'/><x:jc x:val='left'/><x:tblLayout x:type='fixed'/><x:tblBorders>",
    );
    for n in ["top", "left", "bottom", "right", "insideH", "insideV"] {
        body += &format!("<x:{n} x:val='single' x:sz='4' x:color='9b9b9b'/>");
    }
    body += "</x:tblBorders><x:tblCellMar>";
    for n in ["top", "left", "bottom", "right"] {
        body += &format!("<x:{n} x:w='80' x:type='dxa'/>");
    }
    body += "</x:tblCellMar></x:tblPr><x:tblGrid><x:gridCol x:w='1440'/><x:gridCol x:w='1440'/></x:tblGrid><x:tr><x:trPr><x:cantSplit x:val='true'/></x:trPr>";
    for text in ["External &amp; cell", "Українська"] {
        body += &format!(
            "<x:tc><x:tcPr><x:tcW x:w='1440' x:type='dxa'/></x:tcPr><x:p><x:pPr><x:jc x:val='left'/><x:ind x:left='0' x:right='0' x:firstLine='0'/><x:spacing x:before='0' x:after='0' x:line='240' x:lineRule='auto'/></x:pPr><x:r><x:rPr><x:sz x:val='22'/><x:b x:val='false'/><x:i x:val='false'/><x:u x:val='none'/><x:rFonts x:hAnsi='Arial' x:ascii='Arial'/></x:rPr><x:t>{text}</x:t></x:r></x:p></x:tc>"
        );
    }
    body += "</x:tr></x:tbl>";
    p.insert(
        "text/main.xml".into(),
        format!("<x:document xmlns:x='{W}'><x:body>{body}</x:body></x:document>").into_bytes(),
    );
    p
}
#[test]
fn independently_authored_table_only_document_imports() {
    let d = decode(&package::write(&external_table()).unwrap()).unwrap();
    assert_eq!(d.paragraphs.len(), 1);
    let Some(crate::structure::Block::Table(t)) = &d.paragraphs[0].structure.block else {
        panic!("missing table")
    };
    assert_eq!(t.widths_points, vec![72, 72]);
    assert_eq!(t.rows, vec![vec!["External & cell", "Українська"]]);
    assert!(!t.repeat_header);
    assert_eq!(decode(&encode(&d).unwrap()).unwrap(), d);
}
#[test]
fn table_structure_and_formatting_are_not_silently_flattened() {
    for (from, to) in [
        ("x:type='fixed'", "x:type='autofit'"),
        ("<x:tcW", "<x:gridSpan x:val='2'/><x:tcW"),
        ("<x:tcW", "<x:vMerge/><x:tcW"),
        ("<x:tblGrid>", "<x:tblGrid><x:gridCol x:w='1440'/>"),
        ("x:w='2880'", "x:w='2881'"),
        ("x:w='2880'", "x:w='3000'"),
        ("x:color='9b9b9b'", "x:color='FF0000'"),
        ("x:w='80'", "x:w='100'"),
        ("x:val='false'", "x:val='true'"),
        ("<x:tcPr>", "<x:tbl/><x:tcPr>"),
        ("</x:p></x:tc>", "</x:p><x:p/></x:tc>"),
        ("<x:trPr>", "<x:trPr><x:trHeight x:val='200'/>"),
        ("<x:cantSplit x:val='true'/>", ""),
        ("<x:tblPr>", "<x:tblPr><x:tblStyle x:val='Custom'/>"),
    ] {
        let mut p = external_table();
        let s = String::from_utf8(p["text/main.xml"].clone()).unwrap();
        assert!(s.contains(from));
        p.insert("text/main.xml".into(), s.replacen(from, to, 1).into_bytes());
        assert!(decode(&package::write(&p).unwrap()).is_err(), "{to}");
    }
}
#[test]
fn table_edit_undo_redo_save_and_failed_save_preserve_previous_file() {
    use crate::structure::Block;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("table.docx");
    let original = table_document();
    save(&original, &path).unwrap();
    let mut s = crate::session::Session::open(&path).unwrap();
    let Some(Block::Table(mut table)) = s.editor.document.paragraphs[1].structure.block.clone()
    else {
        panic!()
    };
    table.rows[1][0] = "Changed é".into();
    table.widths_points[1] = 120;
    s.editor.update_block(1, Block::Table(table)).unwrap();
    let after = s.editor.document.clone();
    s.save(&path).unwrap();
    assert_eq!(load(&path).unwrap(), after);
    s.editor.undo();
    assert_eq!(s.editor.document, original);
    s.editor.redo();
    assert_eq!(s.editor.document, after);
    let bytes = std::fs::read(&path).unwrap();
    s.editor.document.paragraphs[1].structure.keep_with_next = true;
    assert!(s.save(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert!(s.dirty());
}
#[test]
fn tables_between_numbered_lists_reset_the_native_stream() {
    use crate::structure::{ListItem, ListKind};
    let mut d = table_document();
    for i in [0, 2] {
        d.paragraphs[i].structure.list = Some(ListItem {
            kind: ListKind::Decimal,
            level: 0,
            restart: None,
        });
    }
    assert_eq!(decode(&encode(&d).unwrap()).unwrap(), d);
    assert_eq!(
        crate::structure::labels(&d),
        vec![Some("1.".into()), None, Some("1.".into())]
    );
}

#[test]
fn table_limits_and_unrepresentable_object_properties_fail_before_writing() {
    let d = table_document();
    for mode in 0..5 {
        let mut bad = d.clone();
        let p = &mut bad.paragraphs[1];
        match mode {
            0 => p.alignment = Alignment::Right,
            1 => p.styles[0].bold = Some(true),
            2 => p.spacing.space_before_half_points = Some(10),
            3 => {
                if let Some(crate::structure::Block::Table(t)) = &mut p.structure.block {
                    t.widths_points = vec![200, 200, 200];
                }
            }
            _ => {
                if let Some(crate::structure::Block::Table(t)) = &mut p.structure.block {
                    t.rows = vec![t.rows[0].clone(); 101];
                }
            }
        }
        assert!(encode(&bad).is_err());
    }
    let mut p = external_table();
    let s = String::from_utf8(p["text/main.xml"].clone()).unwrap();
    let begin = s.find("<x:tr>").unwrap();
    let end = s.find("</x:tr>").unwrap() + 7;
    let row = &s[begin..end];
    p.insert(
        "text/main.xml".into(),
        s.replacen(row, &row.repeat(101), 1).into_bytes(),
    );
    assert!(decode(&package::write(&p).unwrap()).is_err());
}

fn extended() -> Document {
    use crate::structure::{Picture, Section};
    use image::ImageEncoder;
    let mut d = sample();
    d.page.header = "D4 · Україна".into();
    d.page.footer = "Synthetic fixture".into();
    d.page.page_numbers = true;
    d.paragraphs[1].structure.section = Some(Section {
        width_points: 842,
        height_points: 595,
        number_start: Some(7),
        header: "Section two".into(),
        page_numbers: true,
        ..Default::default()
    });
    d.style_catalog.normal.character.family = Some(TextFamily::TimesNewRoman);
    d.paragraphs[0].styles[5].family = Some(TextFamily::CourierNew);
    d.style_catalog.normal.tab_stops = Some(vec![
        StyleTabStop {
            position_half_points: 144,
            kind: TabKind::Left,
            leader: Some(TabLeader::Dots),
        },
        StyleTabStop {
            position_half_points: 360,
            kind: TabKind::Center,
            leader: Some(TabLeader::Dashes),
        },
    ]);
    d.paragraphs[0]
        .spacing
        .set_tab_positions(&[(144, TabKind::Right), (500, TabKind::Center)])
        .unwrap();
    d.paragraphs[0].spacing.tab_leaders = vec![TabLeaderStop {
        position_half_points: 500,
        leader: TabLeader::Line,
    }];
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(&[100, 70, 180, 255], 1, 1, image::ExtendedColorType::Rgba8)
        .unwrap();
    d.paragraphs.push(pictures::paragraph(Picture {
        png,
        width_points: 72,
        height_points: 36,
        alt: "Original <test> & Ї".into(),
    }));
    d.package_properties.insert("core".into(), format!("<cp:coreProperties xmlns:cp=\"{}\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\"><dc:title>Test &amp; Україна</dc:title></cp:coreProperties>", metadata::CORE));
    d.package_properties.insert("app".into(), format!("<Properties xmlns=\"{}\"><Application>Fixture</Application><Words>9</Words></Properties>", metadata::APP));
    d
}
fn change_part(parts: &mut Parts, path: &str, old: &str, new: &str) {
    let text = String::from_utf8(parts[path].clone()).unwrap();
    assert!(text.contains(old), "{path}: {old}");
    parts.insert(path.into(), text.replace(old, new).into_bytes());
}
#[test]
fn combined_sections_png_fonts_tabs_metadata_exact_roundtrip_and_history() {
    let d = extended();
    assert_eq!(decode(&encode(&d).unwrap()).unwrap(), d);
    assert_eq!(
        crate::storage::decode(&crate::storage::encode(&d).unwrap()).unwrap(),
        d
    );
    let mut e = crate::commands::Editor::new(d.clone());
    e.replace(0, 0, "Їe\u{301}", "Type").unwrap();
    let edited = e.document.clone();
    assert_eq!(edited.package_properties, d.package_properties);
    e.undo();
    assert_eq!(e.document, d);
    e.redo();
    assert_eq!(e.document, edited);
    assert_eq!(decode(&encode(&edited).unwrap()).unwrap(), edited);
}
#[test]
fn explicit_empty_and_inherited_tab_lists_survive() {
    let mut d = extended();
    d.paragraphs[0].spacing.set_tab_positions(&[]).unwrap();
    assert_eq!(decode(&encode(&d).unwrap()).unwrap(), d);
    d.paragraphs[0].spacing.clear_tab_stops = false;
    assert_eq!(decode(&encode(&d).unwrap()).unwrap(), d);
}
#[test]
fn external_document_default_tabs_are_retained_as_parent() {
    let bytes = mutate(|p| {
        change_part(
            p,
            "word/styles.xml",
            "<w:pPrDefault><w:pPr>",
            "<w:pPrDefault><w:pPr><w:tabs><w:tab w:val=\"right\" w:pos=\"1440\" w:leader=\"dot\"/></w:tabs>",
        )
    });
    let d = decode(&bytes).unwrap();
    assert_eq!(
        d.style_catalog
            .resolve_tabs(d.paragraphs[0].paragraph_style)
            .tab_positions(),
        vec![(144, TabKind::Right)]
    );
    assert_eq!(decode(&encode(&d).unwrap()).unwrap(), d);
}
#[test]
fn invalid_picture_geometry_data_and_external_relationship_fail() {
    let source = package::read(&encode(&extended()).unwrap()).unwrap();
    for (path, old, new) in [
        ("word/document.xml", "cx=\"914400\"", "cx=\"914401\""),
        (
            "word/document.xml",
            "<a:stretch>",
            "<a:srcRect l=\"10\"/><a:stretch>",
        ),
        (
            "word/_rels/document.xml.rels",
            "Target=\"media/image3.png\"",
            "Target=\"https://example.com/image.png\" TargetMode=\"External\"",
        ),
    ] {
        let mut parts = source.clone();
        change_part(&mut parts, path, old, new);
        assert!(decode(&package::write(&parts).unwrap()).is_err());
    }
    let mut parts = source;
    parts.insert("word/media/image3.png".into(), b"not PNG".to_vec());
    assert!(decode(&package::write(&parts).unwrap()).is_err());
}
#[test]
fn unsupported_sections_fields_fonts_and_tab_inheritance_fail() {
    let source = package::read(&encode(&extended()).unwrap()).unwrap();
    for (path, old, new) in [
        (
            "word/document.xml",
            "w:val=\"nextPage\"",
            "w:val=\"continuous\"",
        ),
        ("word/footer1.xml", " PAGE ", " DATE "),
        ("word/styles.xml", "Times New Roman", "Unknown Font"),
        (
            "word/document.xml",
            "<w:tab w:val=\"clear\" w:pos=\"3600\"/>",
            "",
        ),
        (
            "word/document.xml",
            "w:val=\"center\" w:pos=\"5000\"",
            "w:val=\"decimal\" w:pos=\"5000\"",
        ),
    ] {
        let mut parts = source.clone();
        change_part(&mut parts, path, old, new);
        assert!(decode(&package::write(&parts).unwrap()).is_err(), "{old}");
    }
}
#[test]
fn absent_later_header_reference_inherits_without_losing_text() {
    let mut d = extended();
    d.paragraphs[1].structure.section.as_mut().unwrap().header = d.page.header.clone();
    let mut parts = package::read(&encode(&d).unwrap()).unwrap();
    change_part(
        &mut parts,
        "word/document.xml",
        "<w:headerReference w:type=\"default\" r:id=\"rIdheader2\"/>",
        "",
    );
    change_part(
        &mut parts,
        "word/_rels/document.xml.rels",
        &format!(
            "<Relationship Id=\"rIdheader2\" Type=\"{OFFICE_REL}header\" Target=\"header2.xml\"/>"
        ),
        "",
    );
    change_part(
        &mut parts,
        "[Content_Types].xml",
        "<Override PartName=\"/word/header2.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml\"/>",
        "",
    );
    parts.remove("word/header2.xml");
    assert_eq!(decode(&package::write(&parts).unwrap()).unwrap(), d);
}
#[test]
fn native_v20_rejects_old_version_extensions_and_invalid_metadata() {
    let d = extended();
    let mut value: serde_json::Value =
        serde_json::from_slice(&crate::storage::encode(&d).unwrap()).unwrap();
    value["content_version"] = 19.into();
    assert!(crate::storage::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut invalid = d;
    invalid
        .package_properties
        .insert("core".into(), "<bad/>".into());
    assert!(crate::storage::encode(&invalid).is_err());
    assert!(encode(&invalid).is_err());
}
#[test]
fn extended_failed_save_keeps_previous_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("D4.docx");
    let d = extended();
    save(&d, &path).unwrap();
    let before = std::fs::read(&path).unwrap();
    let mut bad = d.clone();
    bad.paragraphs[0].spacing.tab_interval_half_points = Some(80);
    assert!(save(&bad, &path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(load(&path).unwrap(), d);
}
