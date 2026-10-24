//! Inline PNG drawing profile, without external fetches or floating anchors.
use super::*;
use crate::structure::{Block, OBJECT, ParagraphStructure, Picture};
const WP: &str = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing";
const A: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
const PIC: &str = "http://schemas.openxmlformats.org/drawingml/2006/picture";
pub(super) fn paragraph(picture: Picture) -> Paragraph {
    Paragraph {
        text: OBJECT.into(),
        styles: vec![CharacterFormat::default()],
        structure: ParagraphStructure {
            block: Some(Block::Picture(picture)),
            ..Default::default()
        },
        ..Default::default()
    }
}
pub(super) fn check(p: &Paragraph, picture: &Picture) -> Result<()> {
    picture.validate_pixels()?;
    let mut expected = paragraph(picture.clone());
    expected.structure.section = p.structure.section.clone();
    if *p != expected {
        return Err("DOCX inline pictures cannot preserve object paragraph overrides".into());
    }
    Ok(())
}
fn drawing(p: &Picture, id: &str, name: &str, rel: &str) -> String {
    let cx = u32::from(p.width_points) * 12700;
    let cy = u32::from(p.height_points) * 12700;
    format!(
        "<w:drawing xmlns:w=\"{W}\" xmlns:r=\"{}\" xmlns:wp=\"{WP}\" xmlns:a=\"{A}\" xmlns:pic=\"{PIC}\"><wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\"><wp:extent cx=\"{cx}\" cy=\"{cy}\"/><wp:docPr id=\"{}\" name=\"{}\" descr=\"{}\"/><wp:cNvGraphicFramePr><a:graphicFrameLocks noChangeAspect=\"1\"/></wp:cNvGraphicFramePr><a:graphic><a:graphicData uri=\"{PIC}\"><pic:pic><pic:nvPicPr><pic:cNvPr id=\"0\" name=\"Picture\"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed=\"{}\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing>",
        OFFICE_REL.trim_end_matches('/'),
        escape(id),
        escape(name),
        escape(&p.alt),
        escape(rel)
    )
}
pub(super) fn write(p: &Picture, id: usize, parts: &mut Parts) -> String {
    let path = format!("word/media/image{id}.png");
    parts.insert(path.clone(), p.png.clone());
    for (part, closing, item) in [
        (
            "word/_rels/document.xml.rels",
            "</Relationships>",
            format!(
                "<Relationship Id=\"rIdImage{id}\" Type=\"{OFFICE_REL}image\" Target=\"media/image{id}.png\"/>"
            ),
        ),
        (
            "[Content_Types].xml",
            "</Types>",
            format!("<Override PartName=\"/{path}\" ContentType=\"image/png\"/>"),
        ),
    ] {
        let s = String::from_utf8(parts[part].clone())
            .unwrap()
            .replace(closing, &(item + closing));
        parts.insert(part.into(), s.into_bytes());
    }
    format!(
        "<w:p><w:r>{}</w:r></w:p>",
        drawing(p, &id.to_string(), "Picture", &format!("rIdImage{id}"))
    )
}
fn child<'a>(n: &'a Node, ns: &str, name: &str) -> Result<&'a Node> {
    let mut cs = n.children.iter().filter(|c| c.ns == ns && c.name == name);
    let c = cs.next().ok_or_else(|| format!("Missing picture {name}"))?;
    if cs.next().is_some() {
        return Err("Duplicate picture property".into());
    }
    Ok(c)
}
fn attr<'a>(n: &'a Node, name: &str) -> Result<&'a str> {
    n.attr("", name)
        .ok_or_else(|| format!("Missing picture {name}"))
}
fn equal(a: &Node, b: &Node) -> bool {
    a.ns == b.ns
        && a.name == b.name
        && a.attrs == b.attrs
        && a.text.trim().is_empty()
        && b.text.trim().is_empty()
        && a.children.len() == b.children.len()
        && a.children.iter().zip(&b.children).all(|(a, b)| equal(a, b))
}
pub(super) fn read(
    n: &Node,
    parts: &Parts,
    locations: &opc::Locations,
    used: &mut std::collections::BTreeSet<String>,
) -> Result<Paragraph> {
    n.allowed(W, &["r"], &[])?;
    singleton_children(n)?;
    let r = child(n, W, "r")?;
    r.allowed(W, &["drawing"], &[])?;
    singleton_children(r)?;
    let dr = child(r, W, "drawing")?;
    let inline = child(dr, WP, "inline")?;
    let extent = child(inline, WP, "extent")?;
    let measure = |name| -> Result<u16> {
        let v = attr(extent, name)?
            .parse::<u32>()
            .map_err(|_| "Invalid drawing extent")?;
        if v % 12700 != 0 {
            return Err("Picture requires whole-point dimensions".into());
        }
        u16::try_from(v / 12700).map_err(|_| "Picture extent out of range".into())
    };
    let dp = child(inline, WP, "docPr")?;
    let gd = child(child(inline, A, "graphic")?, A, "graphicData")?;
    let pic = child(gd, PIC, "pic")?;
    let blip = child(child(pic, PIC, "blipFill")?, A, "blip")?;
    let rel = blip
        .attr(OFFICE_REL.trim_end_matches('/'), "embed")
        .ok_or("Missing embedded PNG relationship")?;
    let path = locations
        .images
        .get(rel)
        .ok_or("Unknown picture relationship")?;
    let p = Picture {
        png: parts.get(path).ok_or("Missing PNG")?.clone(),
        width_points: measure("cx")?,
        height_points: measure("cy")?,
        alt: attr(dp, "descr")?.into(),
    };
    p.validate_pixels()?;
    let id = attr(dp, "id")?;
    id.parse::<u32>().map_err(|_| "Invalid drawing ID")?;
    let expected = xml::parse(drawing(&p, id, attr(dp, "name")?, rel).as_bytes())?;
    if !equal(dr, &expected) {
        return Err("Unsupported picture transform, crop, effect or drawing metadata".into());
    }
    used.insert(rel.into());
    Ok(paragraph(p))
}
