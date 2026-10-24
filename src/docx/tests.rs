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
    unsupported.page.header = "Header".into();
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
