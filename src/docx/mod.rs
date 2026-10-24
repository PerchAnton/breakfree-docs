//! Explicit, fail-closed OOXML interchange for the primary document format.
mod links;
pub(crate) mod metadata;
mod numbering;
mod opc;
mod package;
mod pictures;
mod properties;
mod sections;
mod styles;
mod tables;
mod tabs;
mod xml;
use crate::{Result, model::*, structure::Section};
use package::Parts;
use std::{io::Read, path::Path};
use xml::{CT, Node, REL, W, escape};
const OFFICE_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/";
const MAIN_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml";
const STYLE_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const DECL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>";
fn element(name: &str, value: impl std::fmt::Display) -> String {
    format!("<w:{name} w:val=\"{}\"/>", escape(&value.to_string()))
}
fn part(parts: &Parts, name: &str) -> Result<Node> {
    xml::parse(
        parts
            .get(name)
            .ok_or_else(|| format!("Missing DOCX part: {name}"))?,
    )
}
fn root(n: &Node, ns: &str, name: &str) -> Result<()> {
    if n.ns != ns || n.name != name {
        Err(format!("Expected {name} root"))
    } else {
        Ok(())
    }
}
fn singleton_children(n: &Node) -> Result<()> {
    let mut names = std::collections::BTreeSet::new();
    for c in &n.children {
        if !names.insert((&c.ns, &c.name)) {
            return Err(format!("Duplicate DOCX property {}", c.name));
        }
    }
    Ok(())
}
/// Validate before opening a save dialog or touching an existing file.
pub fn check_export(d: &Document) -> Result<()> {
    d.validate()?;
    check_toggles(d)?;
    numbering::check_geometry(d)?;
    let names = d
        .style_catalog
        .custom
        .iter()
        .map(|s| s.name.as_str())
        .chain(d.character_styles.iter().map(|s| s.name.as_str()));
    if d.paragraphs
        .iter()
        .map(|p| p.text.as_str())
        .chain(names)
        .any(|s| s.contains(['\u{fffe}', '\u{ffff}']))
    {
        return Err(
            "Text contains a character forbidden by XML 1.0. Save as .bfdoc instead.".into(),
        );
    }
    let mut unsupported = std::collections::BTreeSet::new();
    let mut page = &d.page;
    for p in &d.paragraphs {
        if let Some(section) = &p.structure.section {
            page = section;
        }
        match &p.structure.block {
            Some(crate::structure::Block::Picture(image)) => pictures::check(p, image)?,
            Some(crate::structure::Block::Table(t)) => tables::check(page, p, t)?,
            None => {}
        }
        if p.spacing.tab_interval_half_points.is_some() {
            unsupported.insert("explicit or overridden tabulation");
        }
    }
    for id in d.style_catalog.ids() {
        let s = d.style_catalog.get(id).unwrap();
        if s.tab_interval_half_points.is_some() {
            unsupported.insert("style tabulation");
        }
    }
    if unsupported.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "DOCX export cannot preserve: {}. Save as .bfdoc instead; no output was changed.",
            unsupported.into_iter().collect::<Vec<_>>().join(", ")
        ))
    }
}
pub fn encode(d: &Document) -> Result<Vec<u8>> {
    check_export(d)?;
    let mut parts = Parts::new();
    parts.insert("[Content_Types].xml".into(),format!("{DECL}<Types xmlns=\"{CT}\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Override PartName=\"/word/document.xml\" ContentType=\"{MAIN_TYPE}\"/><Override PartName=\"/word/styles.xml\" ContentType=\"{STYLE_TYPE}\"/></Types>").into_bytes());
    parts.insert("_rels/.rels".into(),format!("{DECL}<Relationships xmlns=\"{REL}\"><Relationship Id=\"rId1\" Type=\"{OFFICE_REL}officeDocument\" Target=\"word/document.xml\"/></Relationships>").into_bytes());
    parts.insert("word/_rels/document.xml.rels".into(),format!("{DECL}<Relationships xmlns=\"{REL}\"><Relationship Id=\"rIdStyles\" Type=\"{OFFICE_REL}styles\" Target=\"styles.xml\"/></Relationships>").into_bytes());
    parts.insert("word/styles.xml".into(), styles::write(d)?.into_bytes());
    let (numbering, list_ids) = numbering::write(d)?;
    if list_ids.iter().any(Option::is_some) {
        parts.insert("word/numbering.xml".into(), numbering.into_bytes());
        let types = String::from_utf8(parts["[Content_Types].xml"].clone())
            .unwrap()
            .replace(
                "</Types>",
                &format!(
                    "<Override PartName=\"/word/numbering.xml\" ContentType=\"{}\"/></Types>",
                    opc::NUMBER_TYPE
                ),
            );
        parts.insert("[Content_Types].xml".into(), types.into_bytes());
        let rels=String::from_utf8(parts["word/_rels/document.xml.rels"].clone()).unwrap().replace("</Relationships>",&format!("<Relationship Id=\"rIdNumbering\" Type=\"{OFFICE_REL}numbering\" Target=\"numbering.xml\"/></Relationships>"));
        parts.insert("word/_rels/document.xml.rels".into(), rels.into_bytes());
    }
    let mut section_settings = vec![(0, d.page.clone())];
    for (i, p) in d.paragraphs.iter().enumerate() {
        if let Some(section) = &p.structure.section {
            if i == 0 || d.paragraphs[i - 1].structure.block.is_some() {
                return Err(
                    "Section must follow a text paragraph and cannot start at document offset zero"
                        .into(),
                );
            }
            section_settings.push((i, section.clone()));
        }
    }
    let section_xml: Vec<_> = section_settings
        .iter()
        .enumerate()
        .map(|(i, (_, s))| sections::write(s, i + 1, &mut parts))
        .collect();
    let mut link_relationships = String::new();
    let mut link_id = 0usize;
    let mut body = String::new();
    for (index, p) in d.paragraphs.iter().enumerate() {
        if let Some(crate::structure::Block::Picture(image)) = &p.structure.block {
            body += &pictures::write(image, index + 1, &mut parts);
            continue;
        }
        if let Some(crate::structure::Block::Table(t)) = &p.structure.block {
            body += &tables::write(t);
            continue;
        }
        body.push_str("<w:p><w:pPr>");
        body.push_str(&element("pStyle", styles::pid(p.paragraph_style)));
        for (name, value) in [
            ("keepNext", p.structure.keep_with_next),
            ("keepLines", p.structure.keep_lines),
            ("pageBreakBefore", p.structure.page_break_before),
        ] {
            if value {
                body.push_str(&element(name, 1));
            }
        }
        if let Some(id) = list_ids[index] {
            body.push_str(&format!(
                "<w:numPr><w:ilvl w:val=\"{}\"/><w:numId w:val=\"{id}\"/></w:numPr>",
                p.structure.list.as_ref().unwrap().level
            ));
        }
        body += &tabs::write(&p.spacing, &d.style_catalog.resolve_tabs(p.paragraph_style));
        body.push_str(&properties::write_spacing(&p.spacing)?);
        body.push_str(&element(
            "jc",
            match p.alignment {
                Alignment::Left => "left",
                Alignment::Center => "center",
                Alignment::Right => "right",
                Alignment::Justify => "both",
            },
        ));
        if let Some(next) = section_settings
            .iter()
            .position(|(start, _)| *start == index + 1)
        {
            body += &section_xml[next - 1];
        }
        body.push_str("</w:pPr>");
        let chars: Vec<_> = p.text.chars().collect();
        let mut start = 0;
        while start < chars.len() {
            let link = p
                .structure
                .links
                .iter()
                .find(|l| l.start <= start && start < l.end);
            if let Some(link) = link.filter(|l| l.start == start) {
                link_id += 1;
                link_relationships += &format!(
                    "<Relationship Id=\"rIdLink{link_id}\" Type=\"{OFFICE_REL}hyperlink\" Target=\"{}\" TargetMode=\"External\"/>",
                    escape(&link.target)
                );
                body += &format!("<w:hyperlink r:id=\"rIdLink{link_id}\">");
            }
            let boundary = link.map(|l| l.end).unwrap_or_else(|| {
                p.structure
                    .links
                    .iter()
                    .find(|l| l.start > start)
                    .map(|l| l.start)
                    .unwrap_or(chars.len())
            });
            let format = p.styles[start];
            let mut end = start + 1;
            while end < boundary && p.styles[end] == format {
                end += 1;
            }
            body.push_str("<w:r><w:rPr>");
            body.push_str(&properties::write_run(format));
            body.push_str("</w:rPr>");
            let mut text = String::new();
            for &c in &chars[start..end] {
                if c == '\t' {
                    if !text.is_empty() {
                        body.push_str(&format!(
                            "<w:t xml:space=\"preserve\">{}</w:t>",
                            escape(&text)
                        ));
                        text.clear();
                    }
                    body.push_str("<w:tab/>");
                } else {
                    text.push(c);
                }
            }
            if !text.is_empty() {
                body.push_str(&format!(
                    "<w:t xml:space=\"preserve\">{}</w:t>",
                    escape(&text)
                ));
            }
            body.push_str("</w:r>");
            if link.is_some_and(|l| l.end == end) {
                body.push_str("</w:hyperlink>");
            }
            start = end;
        }
        body.push_str("</w:p>");
    }
    let rels = String::from_utf8(parts["word/_rels/document.xml.rels"].clone())
        .unwrap()
        .replace(
            "</Relationships>",
            &(link_relationships + "</Relationships>"),
        );
    parts.insert("word/_rels/document.xml.rels".into(), rels.into_bytes());
    body += section_xml.last().unwrap();
    parts.insert(
        "word/document.xml".into(),
        format!(
            "{DECL}<w:document xmlns:w=\"{W}\" xmlns:r=\"{}\"><w:body>{body}</w:body></w:document>",
            OFFICE_REL.trim_end_matches('/')
        )
        .into_bytes(),
    );
    metadata::write(d, &mut parts)?;
    let bytes = package::write(&parts)?;
    if decode(&bytes)? != *d {
        return Err("DOCX cannot preserve this document exactly; save as .bfdoc instead.".into());
    }
    Ok(bytes)
}
pub fn decode(bytes: &[u8]) -> Result<Document> {
    let parts = package::read(bytes)?;
    let locations = opc::inspect(&parts)?;
    let mut d = Document::default();
    for (key, path) in &locations.metadata {
        d.package_properties.insert(
            key.clone(),
            String::from_utf8(parts[path].clone()).map_err(|_| "Metadata must be UTF-8")?,
        );
    }
    metadata::validate(&d.package_properties)?;
    let empty = xml::parse(format!("<w:styles xmlns:w=\"{W}\"/>").as_bytes())?;
    let style_part = locations
        .styles
        .as_deref()
        .map(|p| part(&parts, p))
        .transpose()?;
    let map = styles::read(style_part.as_ref().unwrap_or(&empty), &mut d)?;
    let mut lists = locations
        .numbering
        .as_deref()
        .map(|p| numbering::read(&part(&parts, p)?))
        .transpose()?
        .unwrap_or_default();
    let document = part(&parts, &locations.document)?;
    root(&document, W, "document")?;
    document.allowed(W, &["body"], &[])?;
    singleton_children(&document)?;
    let body = document.child("body").ok_or("Missing DOCX body")?;
    body.allowed(W, &["p", "tbl", "sectPr"], &[])?;
    d.paragraphs.clear();
    let mut section_ends: Vec<(usize, Section)> = Vec::new();
    let mut used_sections = std::collections::BTreeSet::new();
    let mut used_images = std::collections::BTreeSet::new();
    let mut used_links = std::collections::BTreeSet::new();
    for (index, n) in body.children.iter().enumerate() {
        if n.name == "sectPr" {
            if index + 1 != body.children.len() {
                return Err("Section settings must be final".into());
            }
            section_ends.push((
                d.paragraphs.len(),
                sections::read(
                    n,
                    &parts,
                    &locations,
                    &mut used_sections,
                    section_ends.last().map(|s| &s.1),
                )?,
            ));
            continue;
        }
        if n.name == "tbl" {
            lists.paragraph(None)?;
            d.paragraphs.push(tables::read(n, &map)?);
            continue;
        }
        if n.children.iter().any(|r| r.child("drawing").is_some()) {
            lists.paragraph(None)?;
            d.paragraphs
                .push(pictures::read(n, &parts, &locations, &mut used_images)?);
            continue;
        }
        n.allowed(W, &["pPr", "r", "hyperlink"], &[])?;
        if n.children.iter().filter(|n| n.name == "pPr").count() > 1 {
            return Err("Duplicate paragraph properties".into());
        }
        let mut p = Paragraph {
            paragraph_style: map.default_paragraph(),
            ..Default::default()
        };
        p.structure.list = lists.paragraph(n.child("pPr"))?;
        if let Some(pr) = n.child("pPr") {
            pr.allowed(
                W,
                &[
                    "pStyle",
                    "jc",
                    "spacing",
                    "tabs",
                    "ind",
                    "pageBreakBefore",
                    "keepNext",
                    "keepLines",
                    "numPr",
                    "sectPr",
                ],
                &[],
            )?;
            singleton_children(pr)?;
            if let Some(section) = pr.child("sectPr") {
                section_ends.push((
                    d.paragraphs.len() + 1,
                    sections::read(
                        section,
                        &parts,
                        &locations,
                        &mut used_sections,
                        section_ends.last().map(|s| &s.1),
                    )?,
                ));
            }
            if let Some(s) = pr.child("pStyle") {
                properties::leaf(s)?;
                p.paragraph_style = map.p(s.val()?)?;
            }
            if let Some(a) = pr.child("jc") {
                properties::leaf(a)?;
                p.alignment = match a.val()? {
                    "left" => Alignment::Left,
                    "center" => Alignment::Center,
                    "right" => Alignment::Right,
                    "both" => Alignment::Justify,
                    _ => return Err("Unsupported paragraph alignment".into()),
                };
            }
            p.spacing = properties::read_spacing(pr)?;
            tabs::validate_clears(
                pr,
                &d.style_catalog.resolve_tabs(p.paragraph_style),
                &p.spacing,
            )?;
            p.structure.page_break_before =
                properties::flag(pr, "pageBreakBefore")?.unwrap_or(false);
            p.structure.keep_with_next = properties::flag(pr, "keepNext")?.unwrap_or(false);
            p.structure.keep_lines = properties::flag(pr, "keepLines")?.unwrap_or(false);
        }
        for node in n.children.iter().filter(|n| n.name != "pPr") {
            if node.name == "hyperlink" {
                links::read(node, &locations.links, &mut used_links, &map, &mut p)?;
            } else {
                links::read_run(node, &map, &mut p)?;
            }
        }
        d.paragraphs.push(p);
    }
    if !section_ends.is_empty() {
        if section_ends.last().unwrap().0 != d.paragraphs.len() {
            return Err("Missing final section settings".into());
        }
        d.page = section_ends[0].1.clone();
        for pair in section_ends.windows(2) {
            let start = pair[0].0;
            if start >= pair[1].0 || start >= d.paragraphs.len() {
                return Err("Empty or duplicate section boundary".into());
            }
            d.paragraphs[start].structure.section = Some(pair[1].1.clone());
        }
    }
    if used_sections.len() != locations.sections.len() {
        return Err("Unreferenced header/footer relationship".into());
    }
    if used_images.len() != locations.images.len() {
        return Err("Unreferenced PNG relationship".into());
    }
    if used_links.len() != locations.links.len() {
        return Err("Unreferenced DOCX hyperlink relationship".into());
    }
    if d.paragraphs.is_empty() {
        return Err("DOCX body has no paragraphs".into());
    }
    d.validate()?;
    check_export(&d)?;
    Ok(d)
}
fn read_section(n: &Node) -> Result<Section> {
    n.allowed(W, &["pgSz", "pgMar"], &[])?;
    singleton_children(n)?;
    let mut s = Section::default();
    if let Some(size) = n.child("pgSz") {
        size.allowed(W, &[], &[(W, "w"), (W, "h")])?;
        s.width_points = properties::unit(size, "w", 20)?;
        s.height_points = properties::unit(size, "h", 20)?;
    }
    if let Some(m) = n.child("pgMar") {
        m.allowed(
            W,
            &[],
            &[(W, "left"), (W, "right"), (W, "top"), (W, "bottom")],
        )?;
        s.left_points = properties::unit(m, "left", 20)?;
        s.right_points = properties::unit(m, "right", 20)?;
        s.top_points = properties::unit(m, "top", 20)?;
        s.bottom_points = properties::unit(m, "bottom", 20)?;
    }
    s.validate()?;
    Ok(s)
}
pub fn load_bytes(path: &Path) -> Result<Vec<u8>> {
    extension(path)?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| format!("Cannot open DOCX: {e}"))?
        .take((package::MAX + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > package::MAX {
        return Err("DOCX exceeds 32 MiB".into());
    }
    Ok(bytes)
}
pub fn load(path: &Path) -> Result<Document> {
    decode(&load_bytes(path)?)
}
pub fn save(d: &Document, path: &Path) -> Result<()> {
    extension(path)?;
    let bytes = encode(d)?;
    crate::storage::write_bytes_checked(path, &bytes, |_| Ok(()))
}
fn extension(path: &Path) -> Result<()> {
    if path
        .extension()
        .is_some_and(|s| s.eq_ignore_ascii_case("docx"))
    {
        Ok(())
    } else {
        Err("Choose a .docx file".into())
    }
}

// Word style booleans are toggles; native style values are absolute overrides.
// Reject ambiguous compositions until the adapter can retain their semantics.
fn check_toggles(d: &Document) -> Result<()> {
    let incompatible = || {
        "DOCX style toggles cannot preserve this bold/italic inheritance. Save as .bfdoc instead."
            .to_owned()
    };
    for id in d.style_catalog.ids() {
        let s = d.style_catalog.get(id).unwrap();
        let parent = s
            .parent
            .map(|p| d.style_catalog.resolve(p).character)
            .unwrap_or_default();
        if (parent.bold && s.character.bold.is_some())
            || (parent.italic && s.character.italic.is_some())
        {
            return Err(incompatible());
        }
    }
    for s in &d.character_styles {
        let parent = CharacterFormat {
            character_style: s.parent,
            ..Default::default()
        }
        .resolve_with(Style::default(), &d.character_styles);
        if (parent.bold && s.format.bold.is_some()) || (parent.italic && s.format.italic.is_some())
        {
            return Err(incompatible());
        }
    }
    for p in &d.paragraphs {
        let base = d.style_catalog.resolve(p.paragraph_style).character;
        for f in &p.styles {
            if let Some(c) = f.character_style {
                let style = CharacterFormat {
                    character_style: Some(c),
                    ..Default::default()
                }
                .resolve_with(Style::default(), &d.character_styles);
                if (base.bold && style.bold) || (base.italic && style.italic) {
                    return Err(incompatible());
                }
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests;
