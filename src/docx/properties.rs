use super::{styles::Map, *};
pub(super) fn leaf(n: &Node) -> Result<()> {
    n.allowed(W, &[], &[(W, "val")])
}
pub(super) fn flag(n: &Node, name: &str) -> Result<Option<bool>> {
    n.child(name)
        .map(|v| {
            leaf(v)?;
            match v.attr(W, "val").unwrap_or("1") {
                "1" | "true" | "on" => Ok(true),
                "0" | "false" | "off" => Ok(false),
                _ => Err(format!("Invalid {name} boolean")),
            }
        })
        .transpose()
}
pub(super) fn unit(n: &Node, key: &str, divisor: u16) -> Result<u16> {
    let v = n
        .attr(W, key)
        .ok_or_else(|| format!("Missing {} / {key}", n.name))?
        .parse::<u32>()
        .map_err(|_| "Invalid OOXML numeric value")?;
    if v % u32::from(divisor) != 0 {
        return Err("DOCX measurement is outside the supported precision".into());
    }
    u16::try_from(v / u32::from(divisor)).map_err(|_| "DOCX measurement out of range".into())
}
pub(super) fn write_run(f: CharacterFormat) -> String {
    let mut s = String::new();
    if let Some(id) = f.character_style {
        s += &element("rStyle", styles::cid(id));
    }
    for (n, v) in [("b", f.bold), ("i", f.italic)] {
        if let Some(v) = v {
            s += &element(n, u8::from(v));
        }
    }
    if let Some(v) = f.underline {
        s += &element("u", if v { "single" } else { "none" });
    }
    if let Some(v) = f.font_size {
        s += &element("sz", u16::from(v));
    }
    s
}
pub(super) fn read_run(n: &Node, map: &Map) -> Result<CharacterFormat> {
    n.allowed(W, &["b", "i", "u", "sz", "rStyle"], &[])?;
    singleton_children(n)?;
    let mut f = CharacterFormat {
        bold: flag(n, "b")?,
        italic: flag(n, "i")?,
        ..Default::default()
    };
    if let Some(u) = n.child("u") {
        leaf(u)?;
        f.underline = Some(match u.attr(W, "val").unwrap_or("single") {
            "single" => true,
            "none" => false,
            _ => return Err("Only single underline is supported".into()),
        });
    }
    if let Some(sz) = n.child("sz") {
        leaf(sz)?;
        f.font_size = Some(unit(sz, "val", 1)?.try_into()?);
    }
    if let Some(id) = n.child("rStyle") {
        leaf(id)?;
        f.character_style = Some(map.c(id.val()?)?);
    }
    Ok(f)
}
pub(super) fn write_spacing(p: &ParagraphSpacing) -> Result<String> {
    let mut s = String::new();
    let mut spacing = String::new();
    for (n, v) in [
        ("before", p.space_before_half_points),
        ("after", p.space_after_half_points),
    ] {
        if let Some(v) = v {
            spacing += &format!(" w:{n}=\"{}\"", u32::from(v) * 10);
        }
    }
    if let Some(v) = p.line_height_percent {
        if u32::from(v) * 240 % 100 != 0 {
            return Err("DOCX line spacing cannot represent this percentage exactly".into());
        }
        spacing += &format!(
            " w:line=\"{}\" w:lineRule=\"auto\"",
            u32::from(v) * 240 / 100
        );
    }
    if !spacing.is_empty() {
        s += &format!("<w:spacing{spacing}/>");
    }
    let mut indent = String::new();
    for (n, v) in [
        ("left", p.left_indent_half_points),
        ("right", p.right_indent_half_points),
    ] {
        if let Some(v) = v {
            indent += &format!(" w:{n}=\"{}\"", u32::from(v) * 10);
        }
    }
    if let Some(v) = p.first_line_half_points {
        indent += &format!(
            " w:{}=\"{}\"",
            if v < 0 { "hanging" } else { "firstLine" },
            u32::from(v.unsigned_abs()) * 10
        );
    }
    if !indent.is_empty() {
        s += &format!("<w:ind{indent}/>");
    }
    Ok(s)
}
pub(super) fn read_spacing(n: &Node) -> Result<ParagraphSpacing> {
    let mut p = ParagraphSpacing::default();
    if let Some(s) = n.child("spacing") {
        s.allowed(
            W,
            &[],
            &[(W, "before"), (W, "after"), (W, "line"), (W, "lineRule")],
        )?;
        if s.attr(W, "before").is_some() {
            p.space_before_half_points = Some(unit(s, "before", 10)?);
        }
        if s.attr(W, "after").is_some() {
            p.space_after_half_points = Some(unit(s, "after", 10)?);
        }
        if s.attr(W, "line").is_some() {
            if s.attr(W, "lineRule").unwrap_or("auto") != "auto" {
                return Err("Only proportional line spacing is supported".into());
            }
            let line = u32::from(unit(s, "line", 1)?);
            if line * 100 % 240 != 0 {
                return Err("Line spacing cannot be represented exactly".into());
            }
            p.line_height_percent = Some(
                (line * 100 / 240)
                    .try_into()
                    .map_err(|_| "Line spacing out of range")?,
            );
        } else if s.attr(W, "lineRule").is_some() {
            return Err("Line rule without line spacing".into());
        }
    }
    if let Some(i) = n.child("ind") {
        i.allowed(
            W,
            &[],
            &[(W, "left"), (W, "right"), (W, "firstLine"), (W, "hanging")],
        )?;
        if i.attr(W, "left").is_some() {
            p.left_indent_half_points = Some(unit(i, "left", 10)?);
        }
        if i.attr(W, "right").is_some() {
            p.right_indent_half_points = Some(unit(i, "right", 10)?);
        }
        for (key, sign) in [("firstLine", 1), ("hanging", -1)] {
            if i.attr(W, key).is_some() {
                if p.first_line_half_points.is_some() {
                    return Err("Conflicting first-line and hanging indentation".into());
                }
                let v = i16::try_from(unit(i, key, 10)?).map_err(|_| "Indent out of range")?;
                p.first_line_half_points = Some(v * sign);
            }
        }
    }
    p.validate()?;
    Ok(p)
}
pub(super) fn spacing(s: &StyleDefinition) -> ParagraphSpacing {
    ParagraphSpacing {
        line_height_percent: s.line_height_percent,
        space_before_half_points: s.space_before_half_points,
        space_after_half_points: s.space_after_half_points,
        left_indent_half_points: s.left_indent_half_points,
        right_indent_half_points: s.right_indent_half_points,
        first_line_half_points: s.first_line_half_points,
        ..Default::default()
    }
}
