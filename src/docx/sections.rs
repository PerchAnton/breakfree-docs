//! Next-page sections with plain headers/footers and an optional PAGE field.
use super::*;
use std::collections::BTreeSet;
const R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
fn aux(s: &Section, header: bool) -> String {
    let tag = if header { "hdr" } else { "ftr" };
    let text = if header { &s.header } else { &s.footer };
    let mut p = format!(
        "<w:{tag} xmlns:w=\"{W}\"><w:p><w:pPr><w:tabs><w:tab w:val=\"right\" w:pos=\"{}\"/></w:tabs><w:spacing w:before=\"0\" w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/><w:ind w:left=\"0\" w:right=\"0\" w:firstLine=\"0\"/><w:jc w:val=\"left\"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/><w:b w:val=\"0\"/><w:i w:val=\"0\"/><w:sz w:val=\"18\"/><w:u w:val=\"none\"/></w:rPr><w:t xml:space=\"preserve\">{}</w:t></w:r>",
        (s.width_points - s.left_points - s.right_points) * 20,
        escape(text)
    );
    if !header && s.page_numbers {
        p += "<w:r><w:tab/></w:r><w:fldSimple w:instr=\" PAGE \" ><w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/><w:b w:val=\"0\"/><w:i w:val=\"0\"/><w:sz w:val=\"18\"/><w:u w:val=\"none\"/></w:rPr><w:t>1</w:t></w:r></w:fldSimple>";
    }
    p + &format!("</w:p></w:{tag}>")
}
pub(super) fn write(s: &Section, id: usize, parts: &mut Parts) -> String {
    let mut refs = String::new();
    for (kind, header) in [("header", true), ("footer", false)] {
        let path = format!("word/{kind}{id}.xml");
        parts.insert(path.clone(), aux(s, header).into_bytes());
        refs += &format!("<w:{kind}Reference w:type=\"default\" r:id=\"rId{kind}{id}\"/>");
        for (part, closing, item) in [
            (
                "word/_rels/document.xml.rels",
                "</Relationships>",
                format!(
                    "<Relationship Id=\"rId{kind}{id}\" Type=\"{OFFICE_REL}{kind}\" Target=\"{kind}{id}.xml\"/>"
                ),
            ),
            (
                "[Content_Types].xml",
                "</Types>",
                format!(
                    "<Override PartName=\"/{path}\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}+xml\"/>"
                ),
            ),
        ] {
            let t = String::from_utf8(parts[part].clone())
                .unwrap()
                .replace(closing, &(item + closing));
            parts.insert(part.into(), t.into_bytes());
        }
    }
    let start = s
        .number_start
        .map(|n| format!("<w:pgNumType w:start=\"{n}\"/>"))
        .unwrap_or_default();
    format!(
        "<w:sectPr>{refs}<w:type w:val=\"nextPage\"/><w:pgSz w:w=\"{}\" w:h=\"{}\"/><w:pgMar w:left=\"{}\" w:right=\"{}\" w:top=\"{}\" w:bottom=\"{}\" w:header=\"{}\" w:footer=\"{}\"/>{start}</w:sectPr>",
        s.width_points * 20,
        s.height_points * 20,
        s.left_points * 20,
        s.right_points * 20,
        s.top_points * 20,
        s.bottom_points * 20,
        s.top_points * 10,
        s.bottom_points * 10
    )
}
fn eq(a: &Node, b: &Node) -> bool {
    a.ns == b.ns
        && a.name == b.name
        && a.attrs == b.attrs
        && (a.text == b.text || (a.text.trim().is_empty() && b.text.trim().is_empty()))
        && a.children.len() == b.children.len()
        && a.children.iter().zip(&b.children).all(|(a, b)| eq(a, b))
}
pub(super) fn read(
    n: &Node,
    parts: &Parts,
    loc: &opc::Locations,
    used: &mut BTreeSet<String>,
    previous: Option<&Section>,
) -> Result<Section> {
    n.allowed(
        W,
        &[
            "headerReference",
            "footerReference",
            "type",
            "pgSz",
            "pgMar",
            "pgNumType",
        ],
        &[],
    )?;
    singleton_children(n)?;
    let mut stripped = Node {
        ns: n.ns.clone(),
        name: n.name.clone(),
        attrs: n.attrs.clone(),
        children: Vec::new(),
        text: n.text.clone(),
    };
    // Reuse geometry validation with only the original attributes.
    for c in &n.children {
        if matches!(c.name.as_str(), "pgSz" | "pgMar") {
            let mut copy=xml::parse(if c.name=="pgSz" {format!("<w:pgSz xmlns:w=\"{W}\" w:w=\"{}\" w:h=\"{}\"/>",c.attr(W,"w").ok_or("Missing page width")?,c.attr(W,"h").ok_or("Missing page height")?)} else {format!("<w:pgMar xmlns:w=\"{W}\" w:left=\"{}\" w:right=\"{}\" w:top=\"{}\" w:bottom=\"{}\"/>",c.attr(W,"left").ok_or("Missing margin")?,c.attr(W,"right").ok_or("Missing margin")?,c.attr(W,"top").ok_or("Missing margin")?,c.attr(W,"bottom").ok_or("Missing margin")?)}.as_bytes())?;
            if c.name == "pgSz" {
                c.allowed(W, &[], &[(W, "w"), (W, "h")])?;
            } else {
                c.allowed(
                    W,
                    &[],
                    &[
                        (W, "left"),
                        (W, "right"),
                        (W, "top"),
                        (W, "bottom"),
                        (W, "header"),
                        (W, "footer"),
                    ],
                )?;
            }
            copy.text = c.text.clone();
            stripped.children.push(copy);
        }
    }
    if previous.is_some() && (n.child("pgSz").is_none() || n.child("pgMar").is_none()) {
        return Err("Later sections require explicit page geometry".into());
    }
    let mut s = read_section(&stripped)?;
    if let Some(previous) = previous {
        s.header = previous.header.clone();
        s.footer = previous.footer.clone();
        s.page_numbers = previous.page_numbers;
    }
    if let Some(t) = n.child("type") {
        properties::leaf(t)?;
        if t.val()? != "nextPage" {
            return Err("Only next-page sections are supported".into());
        }
    }
    if let Some(p) = n.child("pgNumType") {
        p.allowed(W, &[], &[(W, "start")])?;
        s.number_start = Some(properties::unit(p, "start", 1)?);
    }
    if let Some(m) = n.child("pgMar") {
        for (name, expected) in [("header", s.top_points), ("footer", s.bottom_points)] {
            if m.attr(W, name).is_some() && properties::unit(m, name, 10)? != expected {
                return Err("Header/footer distance outside model profile".into());
            }
        }
    }
    for (kind, header) in [("header", true), ("footer", false)] {
        if let Some(reference) = n.child(&format!("{kind}Reference")) {
            reference.allowed(W, &[], &[(W, "type"), (R, "id")])?;
            if reference.attr(W, "type") != Some("default") {
                return Err("First/even headers are unsupported".into());
            }
            let id = reference
                .attr(R, "id")
                .ok_or("Missing header/footer reference")?;
            let (actual, path) = loc
                .sections
                .get(id)
                .ok_or("Unknown header/footer relationship")?;
            if actual != kind {
                return Err("Header/footer relationship kind mismatch".into());
            }
            let mut root = part(parts, path)?;
            if let Some(p) = root
                .children
                .iter_mut()
                .find(|c| c.ns == W && c.name == "p")
                && let Some(field) = p
                    .children
                    .iter_mut()
                    .find(|c| c.ns == W && c.name == "fldSimple")
                && let Some(run) = field
                    .children
                    .iter_mut()
                    .find(|c| c.ns == W && c.name == "r")
                && let Some(text) = run.children.iter_mut().find(|c| c.ns == W && c.name == "t")
            {
                if text.text.is_empty()
                    || !text.text.bytes().all(|b| b.is_ascii_digit())
                    || text.text.parse::<u16>().ok().filter(|v| *v > 0).is_none()
                {
                    return Err("Unsupported PAGE field cache".into());
                }
                text.text = "1".into();
            }
            let p = root.child("p").ok_or("Missing header/footer paragraph")?;
            let text = p
                .child("r")
                .and_then(|r| r.child("t"))
                .ok_or("Missing header/footer text")?
                .text
                .clone();
            if header {
                s.header = text;
            } else {
                s.footer = text;
                s.page_numbers = p.child("fldSimple").is_some();
            }
            let expected = xml::parse(aux(&s, header).as_bytes())?;
            if !eq(&root, &expected) {
                return Err("Unsupported header/footer formatting or field".into());
            }
            used.insert(id.into());
        }
    }
    s.validate()?;
    Ok(s)
}
