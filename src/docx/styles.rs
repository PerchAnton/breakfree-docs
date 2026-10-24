use super::*;
use std::collections::BTreeMap;
#[derive(Default)]
pub(super) struct Map {
    p: BTreeMap<String, ParagraphStyle>,
    c: BTreeMap<String, CharacterStyle>,
}
impl Map {
    pub fn p(&self, id: &str) -> Result<ParagraphStyle> {
        self.p
            .get(id)
            .copied()
            .ok_or_else(|| format!("Unknown paragraph style {id}"))
    }
    pub fn c(&self, id: &str) -> Result<CharacterStyle> {
        self.c
            .get(id)
            .copied()
            .ok_or_else(|| format!("Unknown character style {id}"))
    }
}
pub(super) fn pid(id: ParagraphStyle) -> String {
    match id {
        ParagraphStyle::Normal => "Normal".into(),
        ParagraphStyle::NoSpacing => "NoSpacing".into(),
        ParagraphStyle::Heading1 => "Heading1".into(),
        ParagraphStyle::Custom(n) => format!("BFp{n}"),
    }
}
pub(super) fn cid(id: CharacterStyle) -> String {
    match id {
        CharacterStyle::Strong => "Strong".into(),
        CharacterStyle::Emphasis => "Emphasis".into(),
        CharacterStyle::Custom(n) => format!("BFc{n}"),
    }
}
const DEFAULTS: &str = "<w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/><w:sz w:val=\"22\"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:line=\"300\" w:lineRule=\"auto\" w:before=\"0\" w:after=\"110\"/></w:pPr></w:pPrDefault></w:docDefaults>";
pub(super) fn write(d: &Document) -> Result<String> {
    let mut s = format!("{DECL}<w:styles xmlns:w=\"{W}\">{DEFAULTS}");
    for id in d.style_catalog.ids() {
        let def = d.style_catalog.get(id).unwrap();
        let name = match id {
            ParagraphStyle::Custom(n) => d
                .style_catalog
                .custom
                .iter()
                .find(|s| s.id == n)
                .unwrap()
                .name
                .as_str(),
            _ => id.name(),
        };
        s += &format!(
            "<w:style w:type=\"paragraph\" w:styleId=\"{}\"{}>",
            pid(id),
            if id == ParagraphStyle::Normal {
                " w:default=\"1\""
            } else {
                ""
            }
        );
        s += &element("name", name);
        if let Some(p) = def.parent {
            s += &element("basedOn", pid(p));
        }
        if let Some(p) = def.next_style {
            s += &element("next", pid(p));
        }
        s += "<w:pPr>";
        s += &properties::write_spacing(&properties::spacing(def))?;
        s += "</w:pPr><w:rPr>";
        s += &properties::write_run(def.character);
        s += "</w:rPr></w:style>";
    }
    for (id, name, parent, format) in [
        (
            CharacterStyle::Strong,
            "Strong",
            None,
            CharacterFormat {
                bold: Some(true),
                ..Default::default()
            },
        ),
        (
            CharacterStyle::Emphasis,
            "Emphasis",
            None,
            CharacterFormat {
                italic: Some(true),
                ..Default::default()
            },
        ),
    ]
    .into_iter()
    .chain(d.character_styles.iter().map(|s| {
        (
            CharacterStyle::Custom(s.id),
            s.name.as_str(),
            s.parent,
            s.format,
        )
    })) {
        s += &format!("<w:style w:type=\"character\" w:styleId=\"{}\">", cid(id));
        s += &element("name", name);
        if let Some(p) = parent {
            s += &element("basedOn", cid(p));
        }
        s += "<w:rPr>";
        s += &properties::write_run(format);
        s += "</w:rPr></w:style>";
    }
    s += "</w:styles>";
    Ok(s)
}
// Default-property equality is semantic (namespace/attribute order independent).
fn equivalent(a: &Node, b: &Node) -> bool {
    a.ns == b.ns
        && a.name == b.name
        && a.attrs == b.attrs
        && a.text.trim() == b.text.trim()
        && a.children.len() == b.children.len()
        && a.children
            .iter()
            .zip(&b.children)
            .all(|(a, b)| equivalent(a, b))
}
pub(super) fn read(n: &Node, d: &mut Document) -> Result<Map> {
    root(n, W, "styles")?;
    n.allowed(W, &["docDefaults", "style"], &[])?;
    let defaults =
        xml::parse(format!("<w:styles xmlns:w=\"{W}\">{DEFAULTS}</w:styles>").as_bytes())?;
    if n.children
        .iter()
        .filter(|n| n.name == "docDefaults")
        .count()
        != 1
        || !equivalent(
            n.child("docDefaults").ok_or("Missing document defaults")?,
            defaults.child("docDefaults").unwrap(),
        )
    {
        return Err(
            "DOCX document defaults differ from the supported profile (Arial 11 pt, 125% spacing)."
                .into(),
        );
    }
    let mut map = Map::default();
    let mut countp = 0;
    let mut countc = 0;
    for s in n.children.iter().filter(|n| n.name == "style") {
        s.allowed(
            W,
            &["name", "basedOn", "next", "rPr", "pPr"],
            &[(W, "type"), (W, "styleId"), (W, "default")],
        )?;
        singleton_children(s)?;
        let id = s.attr(W, "styleId").ok_or("Style has no identifier")?;
        let name = s.child("name").ok_or("Style has no name")?;
        properties::leaf(name)?;
        match s.attr(W, "type") {
            Some("paragraph") => {
                let p = match id {
                    "Normal" => ParagraphStyle::Normal,
                    "NoSpacing" => ParagraphStyle::NoSpacing,
                    "Heading1" => ParagraphStyle::Heading1,
                    _ => {
                        countp += 1;
                        let value = id
                            .strip_prefix("BFp")
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(1000 + countp);
                        ParagraphStyle::Custom(value)
                    }
                };
                if map.p.values().any(|v| *v == p) || map.p.insert(id.into(), p).is_some() {
                    return Err("Duplicate paragraph style identifier".into());
                }
                if s.attr(W, "default")
                    .is_some_and(|v| id != "Normal" || v != "1")
                {
                    return Err("Unsupported default style".into());
                }
                if !matches!(p, ParagraphStyle::Custom(_)) && name.val()? != p.name() {
                    return Err("Unsupported built-in style name".into());
                }
            }
            Some("character") => {
                let c = match id {
                    "Strong" => CharacterStyle::Strong,
                    "Emphasis" => CharacterStyle::Emphasis,
                    _ => {
                        countc += 1;
                        CharacterStyle::Custom(
                            id.strip_prefix("BFc")
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(1000 + countc),
                        )
                    }
                };
                if map.c.values().any(|v| *v == c) || map.c.insert(id.into(), c).is_some() {
                    return Err("Duplicate character style identifier".into());
                }
                if s.attr(W, "default").is_some() {
                    return Err("Default character style unsupported".into());
                }
            }
            _ => return Err("Unsupported style kind".into()),
        }
    }
    if countp > 64 || countc > 64 {
        return Err("Style count exceeds limit".into());
    }
    for id in ["Normal", "NoSpacing", "Heading1"] {
        map.p(id)?;
    }
    for id in ["Strong", "Emphasis"] {
        map.c(id)?;
    }
    for s in n.children.iter().filter(|n| n.name == "style") {
        let id = s.attr(W, "styleId").unwrap();
        let name = s.child("name").unwrap().val()?.to_owned();
        let parent = s
            .child("basedOn")
            .map(|n| {
                properties::leaf(n)?;
                n.val()
            })
            .transpose()?;
        let format = s
            .child("rPr")
            .map(|r| properties::read_run(r, &map))
            .transpose()?
            .unwrap_or_default();
        if format.character_style.is_some() {
            return Err("Style definitions cannot reference rStyle".into());
        }
        if s.attr(W, "type") == Some("paragraph") {
            let next = s
                .child("next")
                .map(|n| {
                    properties::leaf(n)?;
                    map.p(n.val()?)
                })
                .transpose()?;
            let spacing = if let Some(pr) = s.child("pPr") {
                pr.allowed(W, &["spacing", "ind"], &[])?;
                singleton_children(pr)?;
                properties::read_spacing(pr)?
            } else {
                ParagraphSpacing::default()
            };
            let def = StyleDefinition {
                parent: parent.map(|p| map.p(p)).transpose()?,
                next_style: next,
                character: format,
                line_height_percent: spacing.line_height_percent,
                space_before_half_points: spacing.space_before_half_points,
                space_after_half_points: spacing.space_after_half_points,
                left_indent_half_points: spacing.left_indent_half_points,
                right_indent_half_points: spacing.right_indent_half_points,
                first_line_half_points: spacing.first_line_half_points,
                ..Default::default()
            };
            match map.p(id)? {
                ParagraphStyle::Custom(id) => d.style_catalog.custom.push(CustomStyle {
                    id,
                    name,
                    definition: def,
                }),
                p => *d.style_catalog.get_mut(p).unwrap() = def,
            }
        } else {
            if s.child("next").is_some() || s.child("pPr").is_some() {
                return Err("Paragraph properties in character style".into());
            }
            match map.c(id)? {
                CharacterStyle::Custom(id) => d.character_styles.push(NamedCharacterStyle {
                    id,
                    name,
                    parent: parent.map(|p| map.c(p)).transpose()?,
                    format,
                }),
                builtin => {
                    let expected = CharacterFormat {
                        bold: (builtin == CharacterStyle::Strong).then_some(true),
                        italic: (builtin == CharacterStyle::Emphasis).then_some(true),
                        ..Default::default()
                    };
                    if format != expected || parent.is_some() || name != builtin.name() {
                        return Err("Modified built-in character style unsupported".into());
                    }
                }
            }
        }
    }
    d.validate()?;
    Ok(map)
}
