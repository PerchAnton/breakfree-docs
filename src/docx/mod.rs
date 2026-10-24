//! Explicit, fail-closed OOXML interchange. Never modifies an imported source.
mod package;
mod properties;
mod styles;
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
    if !d.page.header.is_empty()
        || !d.page.footer.is_empty()
        || d.page.page_numbers
        || d.page.number_start.is_some()
    {
        unsupported.insert("headers, footers or page numbering");
    }
    for p in &d.paragraphs {
        if p.structure.block.is_some() {
            unsupported.insert("tables or pictures");
        }
        if p.structure.list.is_some() {
            unsupported.insert("lists");
        }
        if !p.structure.links.is_empty() {
            unsupported.insert("hyperlinks");
        }
        if p.structure.section.is_some() {
            unsupported.insert("multiple sections");
        }
        if p.spacing.has_direct_tab_stops() || p.spacing.tab_interval_half_points.is_some() {
            unsupported.insert("explicit or overridden tabulation");
        }
    }
    for id in d.style_catalog.ids() {
        let s = d.style_catalog.get(id).unwrap();
        if s.tab_stops.is_some() || s.tab_interval_half_points.is_some() {
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
    let mut body = String::new();
    for p in &d.paragraphs {
        body.push_str("<w:p><w:pPr>");
        body.push_str(&element("pStyle", styles::pid(p.paragraph_style)));
        body.push_str(&element(
            "jc",
            match p.alignment {
                Alignment::Left => "left",
                Alignment::Center => "center",
                Alignment::Right => "right",
                Alignment::Justify => "both",
            },
        ));
        body.push_str(&properties::write_spacing(&p.spacing)?);
        for (name, value) in [
            ("pageBreakBefore", p.structure.page_break_before),
            ("keepNext", p.structure.keep_with_next),
            ("keepLines", p.structure.keep_lines),
        ] {
            if value {
                body.push_str(&element(name, 1));
            }
        }
        body.push_str("</w:pPr>");
        let chars: Vec<_> = p.text.chars().collect();
        let mut start = 0;
        while start < chars.len() {
            let format = p.styles[start];
            let mut end = start + 1;
            while end < chars.len() && p.styles[end] == format {
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
            start = end;
        }
        body.push_str("</w:p>");
    }
    let p = &d.page;
    body.push_str(&format!("<w:sectPr><w:pgSz w:w=\"{}\" w:h=\"{}\"/><w:pgMar w:left=\"{}\" w:right=\"{}\" w:top=\"{}\" w:bottom=\"{}\"/></w:sectPr>",p.width_points*20,p.height_points*20,p.left_points*20,p.right_points*20,p.top_points*20,p.bottom_points*20));
    parts.insert(
        "word/document.xml".into(),
        format!("{DECL}<w:document xmlns:w=\"{W}\"><w:body>{body}</w:body></w:document>")
            .into_bytes(),
    );
    package::write(&parts)
}
pub fn decode(bytes: &[u8]) -> Result<Document> {
    let parts = package::read(bytes)?;
    // Every package part is accounted for. Comments, signatures, macros and
    // external relationships are rejected, never silently stripped.
    let expected = [
        "[Content_Types].xml",
        "_rels/.rels",
        "word/document.xml",
        "word/styles.xml",
        "word/_rels/document.xml.rels",
    ];
    for name in parts.keys() {
        if !expected.contains(&name.as_str()) {
            return Err(format!(
                "Unsupported DOCX part: {name}. Source remains unchanged."
            ));
        }
    }
    let types = part(&parts, "[Content_Types].xml")?;
    root(&types, CT, "Types")?;
    types.allowed(CT, &["Default", "Override"], &[])?;
    let mut seen = std::collections::BTreeSet::new();
    for n in &types.children {
        let (key, value) = if n.name == "Default" {
            n.allowed(CT, &[], &[("", "Extension"), ("", "ContentType")])?;
            (
                n.attr("", "Extension").ok_or("Missing extension")?,
                "application/vnd.openxmlformats-package.relationships+xml",
            )
        } else {
            n.allowed(CT, &[], &[("", "PartName"), ("", "ContentType")])?;
            (
                n.attr("", "PartName").ok_or("Missing part name")?,
                if n.attr("", "PartName") == Some("/word/document.xml") {
                    MAIN_TYPE
                } else {
                    STYLE_TYPE
                },
            )
        };
        if !["rels", "/word/document.xml", "/word/styles.xml"].contains(&key)
            || n.attr("", "ContentType") != Some(value)
            || !seen.insert(key)
        {
            return Err("Unsupported, duplicate or macro-enabled content type".into());
        }
    }
    if seen.len() != 3 {
        return Err("Missing content types".into());
    }
    for (path, kind, target) in [
        ("_rels/.rels", "officeDocument", "word/document.xml"),
        ("word/_rels/document.xml.rels", "styles", "styles.xml"),
    ] {
        let rel = part(&parts, path)?;
        root(&rel, REL, "Relationships")?;
        rel.allowed(REL, &["Relationship"], &[])?;
        if rel.children.len() != 1 {
            return Err("Unsupported package relationships".into());
        }
        let n = &rel.children[0];
        n.allowed(REL, &[], &[("", "Id"), ("", "Type"), ("", "Target")])?;
        if n.attr("", "Type") != Some(format!("{OFFICE_REL}{kind}").as_str())
            || n.attr("", "Target") != Some(target)
            || n.attr("", "Id").is_none_or(str::is_empty)
        {
            return Err("Unsupported or external DOCX relationship".into());
        }
    }
    let mut d = Document::default();
    let map = styles::read(&part(&parts, "word/styles.xml")?, &mut d)?;
    let document = part(&parts, "word/document.xml")?;
    root(&document, W, "document")?;
    document.allowed(W, &["body"], &[])?;
    singleton_children(&document)?;
    let body = document.child("body").ok_or("Missing DOCX body")?;
    body.allowed(W, &["p", "sectPr"], &[])?;
    d.paragraphs.clear();
    for (index, n) in body.children.iter().enumerate() {
        if n.name == "sectPr" {
            if index + 1 != body.children.len() {
                return Err("Section settings must be final".into());
            }
            d.page = read_section(n)?;
            continue;
        }
        n.allowed(W, &["pPr", "r"], &[])?;
        if n.children.iter().filter(|n| n.name == "pPr").count() > 1 {
            return Err("Duplicate paragraph properties".into());
        }
        let mut p = Paragraph::default();
        if let Some(pr) = n.child("pPr") {
            pr.allowed(
                W,
                &[
                    "pStyle",
                    "jc",
                    "spacing",
                    "ind",
                    "pageBreakBefore",
                    "keepNext",
                    "keepLines",
                ],
                &[],
            )?;
            singleton_children(pr)?;
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
            p.structure.page_break_before =
                properties::flag(pr, "pageBreakBefore")?.unwrap_or(false);
            p.structure.keep_with_next = properties::flag(pr, "keepNext")?.unwrap_or(false);
            p.structure.keep_lines = properties::flag(pr, "keepLines")?.unwrap_or(false);
        }
        for run in n.children.iter().filter(|n| n.name == "r") {
            run.allowed(W, &["rPr", "t", "tab"], &[])?;
            if run.children.iter().filter(|n| n.name == "rPr").count() > 1 {
                return Err("Duplicate run properties".into());
            }
            let format = run
                .child("rPr")
                .map(|r| properties::read_run(r, &map))
                .transpose()?
                .unwrap_or_default();
            for node in run.children.iter().filter(|n| n.name != "rPr") {
                let text = if node.name == "tab" {
                    node.allowed(W, &[], &[])?;
                    "\t"
                } else {
                    if !node.children.is_empty()
                        || node.attrs.iter().any(|((ns, k), v)| {
                            ns != "http://www.w3.org/XML/1998/namespace"
                                || k != "space"
                                || !["preserve", "default"].contains(&v.as_str())
                        })
                    {
                        return Err("Unsupported text properties".into());
                    }
                    &node.text
                };
                p.text.push_str(text);
                p.styles
                    .extend(std::iter::repeat_n(format, text.chars().count()));
            }
        }
        d.paragraphs.push(p);
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
pub fn load(path: &Path) -> Result<Document> {
    extension(path)?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| format!("Cannot open DOCX: {e}"))?
        .take((package::MAX + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    decode(&bytes)
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
