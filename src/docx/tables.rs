//! Bounded fixed-grid plain-cell tables; unsupported presentation is never discarded.
use super::*;
use crate::structure::{Block, OBJECT, ParagraphStructure, Table};
const EDGES: [&str; 6] = ["top", "left", "bottom", "right", "insideH", "insideV"];
const SIDES: [&str; 4] = ["top", "left", "bottom", "right"];
pub(super) fn paragraph(table: Table) -> Paragraph {
    Paragraph {
        text: OBJECT.into(),
        styles: vec![CharacterFormat::default()],
        structure: ParagraphStructure {
            block: Some(Block::Table(table)),
            ..Default::default()
        },
        ..Default::default()
    }
}
pub(super) fn check(page: &Section, p: &Paragraph, t: &Table) -> Result<()> {
    let mut expected = paragraph(t.clone());
    expected.structure.section = p.structure.section.clone();
    if *p != expected {
        return Err("DOCX tables cannot preserve formatting or flow overrides on the object paragraph; save as .bfdoc instead".into());
    }
    let width: u32 = t.widths_points.iter().map(|v| u32::from(*v)).sum();
    if width > u32::from(page.width_points - page.left_points - page.right_points) {
        return Err(
            "DOCX table exceeds the page text width; adjust columns or save as .bfdoc".into(),
        );
    }
    Ok(())
}
fn cell_pr() -> &'static str {
    "<w:pPr><w:spacing w:before=\"0\" w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/><w:ind w:left=\"0\" w:right=\"0\" w:firstLine=\"0\"/><w:jc w:val=\"left\"/></w:pPr>"
}
pub(super) fn write(t: &Table) -> String {
    let width: u32 = t.widths_points.iter().map(|v| u32::from(*v) * 20).sum();
    let mut s = format!(
        "<w:tbl><w:tblPr><w:tblW w:w=\"{width}\" w:type=\"dxa\"/><w:jc w:val=\"left\"/><w:tblBorders>"
    );
    for side in EDGES {
        s += &format!("<w:{side} w:val=\"single\" w:sz=\"4\" w:color=\"9B9B9B\"/>");
    }
    s += "</w:tblBorders><w:tblLayout w:type=\"fixed\"/><w:tblCellMar>";
    for side in SIDES {
        s += &format!("<w:{side} w:w=\"80\" w:type=\"dxa\"/>");
    }
    s += "</w:tblCellMar></w:tblPr><w:tblGrid>";
    for width in &t.widths_points {
        s += &format!("<w:gridCol w:w=\"{}\"/>", u32::from(*width) * 20);
    }
    s += "</w:tblGrid>";
    for (row, cells) in t.rows.iter().enumerate() {
        let header = row == 0 && t.repeat_header;
        s += "<w:tr><w:trPr><w:cantSplit/>";
        if header {
            s += "<w:tblHeader/>";
        }
        s += "</w:trPr>";
        for (text, width) in cells.iter().zip(&t.widths_points) {
            s += &format!(
                "<w:tc><w:tcPr><w:tcW w:w=\"{}\" w:type=\"dxa\"/></w:tcPr><w:p>{}<w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/><w:b w:val=\"{}\"/><w:i w:val=\"0\"/><w:u w:val=\"none\"/><w:sz w:val=\"22\"/></w:rPr>",
                u32::from(*width) * 20,
                cell_pr(),
                u8::from(header)
            );
            let mut segment = String::new();
            for c in text.chars() {
                if matches!(c, '\n' | '\t') {
                    s += &format!("<w:t xml:space=\"preserve\">{}</w:t>", escape(&segment));
                    segment.clear();
                    s += if c == '\n' { "<w:br/>" } else { "<w:tab/>" };
                } else {
                    segment.push(c);
                }
            }
            s += &format!(
                "<w:t xml:space=\"preserve\">{}</w:t></w:r></w:p></w:tc>",
                escape(&segment)
            );
        }
        s += "</w:tr>";
    }
    s + "</w:tbl>"
}
fn required<'a>(n: &'a Node, name: &str) -> Result<&'a Node> {
    n.child(name)
        .ok_or_else(|| format!("Plain DOCX table requires {} / {name}", n.name))
}
fn width(n: &Node) -> Result<u16> {
    n.allowed(W, &[], &[(W, "w"), (W, "type")])?;
    if n.attr(W, "type") != Some("dxa") {
        return Err("Table widths require exact point measurements".into());
    }
    properties::unit(n, "w", 20)
}
pub(super) fn read(n: &Node, map: &styles::Map) -> Result<Paragraph> {
    n.allowed(W, &["tblPr", "tblGrid", "tr"], &[])?;
    for name in ["tblPr", "tblGrid"] {
        if n.children.iter().filter(|c| c.name == name).count() != 1 {
            return Err("Missing or duplicate table properties/grid".into());
        }
    }
    let pr = required(n, "tblPr")?;
    pr.allowed(
        W,
        &["tblW", "jc", "tblBorders", "tblLayout", "tblCellMar"],
        &[],
    )?;
    singleton_children(pr)?;
    let layout = required(pr, "tblLayout")?;
    layout.allowed(W, &[], &[(W, "type")])?;
    if layout.attr(W, "type") != Some("fixed") {
        return Err("Only fixed-grid DOCX tables are supported".into());
    }
    let jc = required(pr, "jc")?;
    properties::leaf(jc)?;
    if jc.val()? != "left" {
        return Err("Only left-aligned DOCX tables are supported".into());
    }
    let borders = required(pr, "tblBorders")?;
    borders.allowed(W, &EDGES, &[])?;
    singleton_children(borders)?;
    for name in EDGES {
        let b = required(borders, name)?;
        b.allowed(W, &[], &[(W, "val"), (W, "sz"), (W, "color")])?;
        if b.val()? != "single"
            || properties::unit(b, "sz", 1)? != 4
            || !b
                .attr(W, "color")
                .is_some_and(|v| v.eq_ignore_ascii_case("9B9B9B"))
        {
            return Err("Table border is outside the plain-cell profile".into());
        }
    }
    let margins = required(pr, "tblCellMar")?;
    margins.allowed(W, &SIDES, &[])?;
    singleton_children(margins)?;
    for name in SIDES {
        if width(required(margins, name)?)? != 4 {
            return Err("Table cell margins must be 4 pt".into());
        }
    }
    let grid = required(n, "tblGrid")?;
    grid.allowed(W, &["gridCol"], &[])?;
    if !(1..=12).contains(&grid.children.len()) {
        return Err("Table requires 1–12 columns".into());
    }
    let widths = grid
        .children
        .iter()
        .map(|c| {
            c.allowed(W, &[], &[(W, "w")])?;
            properties::unit(c, "w", 20)
        })
        .collect::<Result<Vec<_>>>()?;
    if u32::from(width(required(pr, "tblW")?)?) != widths.iter().map(|v| u32::from(*v)).sum::<u32>()
    {
        return Err("Table width disagrees with grid".into());
    }
    let mut t = Table {
        widths_points: widths,
        rows: Vec::new(),
        repeat_header: false,
    };
    for row in n.children.iter().filter(|c| c.name == "tr") {
        if t.rows.len() == 100 {
            return Err("Table exceeds 100 rows".into());
        }
        row.allowed(W, &["trPr", "tc"], &[])?;
        if row.children.iter().filter(|c| c.name == "trPr").count() != 1 {
            return Err("Missing or duplicate table row properties".into());
        }
        let rp = required(row, "trPr")?;
        rp.allowed(W, &["cantSplit", "tblHeader"], &[])?;
        singleton_children(rp)?;
        if properties::flag(rp, "cantSplit")? != Some(true) {
            return Err("Table rows must not split across pages".into());
        }
        let header = properties::flag(rp, "tblHeader")?.unwrap_or(false);
        if header && !t.rows.is_empty() {
            return Err("Only the first table row can repeat as header".into());
        }
        if t.rows.is_empty() {
            t.repeat_header = header;
        }
        let cells: Vec<_> = row.children.iter().filter(|c| c.name == "tc").collect();
        if cells.len() != t.widths_points.len() {
            return Err("Table rows must match the column grid".into());
        }
        let mut values = Vec::new();
        for (cell, w) in cells.iter().zip(&t.widths_points) {
            cell.allowed(W, &["tcPr", "p"], &[])?;
            singleton_children(cell)?;
            let cp = required(cell, "tcPr")?;
            cp.allowed(W, &["tcW"], &[])?;
            singleton_children(cp)?;
            if width(required(cp, "tcW")?)? != *w {
                return Err("Cell width disagrees with grid".into());
            }
            values.push(read_cell(required(cell, "p")?, header, map)?);
        }
        t.rows.push(values);
    }
    t.validate()?;
    Ok(paragraph(t))
}
fn read_cell(n: &Node, header: bool, map: &styles::Map) -> Result<String> {
    n.allowed(W, &["pPr", "r"], &[])?;
    if n.children.iter().filter(|c| c.name == "pPr").count() != 1 {
        return Err("Cell requires explicit plain paragraph properties".into());
    }
    let pr = required(n, "pPr")?;
    pr.allowed(W, &["spacing", "ind", "jc"], &[])?;
    singleton_children(pr)?;
    let sp = properties::read_spacing(pr)?;
    if sp
        != (ParagraphSpacing {
            line_height_percent: Some(100),
            space_before_half_points: Some(0),
            space_after_half_points: Some(0),
            left_indent_half_points: Some(0),
            right_indent_half_points: Some(0),
            first_line_half_points: Some(0),
            ..Default::default()
        })
    {
        return Err("Cell spacing/indents are outside the plain profile".into());
    }
    let jc = required(pr, "jc")?;
    properties::leaf(jc)?;
    if jc.val()? != "left" {
        return Err("Cell must be left aligned".into());
    }
    let mut text = String::new();
    for run in n.children.iter().filter(|c| c.name == "r") {
        run.allowed(W, &["rPr", "t", "tab", "br"], &[])?;
        if run.children.iter().filter(|c| c.name == "rPr").count() != 1 {
            return Err("Cell requires explicit plain run properties".into());
        }
        let rp = required(run, "rPr")?;
        let f = properties::read_run(rp, map)?;
        if f != (CharacterFormat {
            bold: Some(header),
            family: Some(TextFamily::Arial),
            italic: Some(false),
            underline: Some(false),
            font_size: Some(FontSize::try_from(22)?),
            character_style: None,
        }) {
            return Err("Rich cell formatting cannot be preserved".into());
        }
        let font = required(rp, "rFonts")?;
        if font.attr(W, "ascii") != Some("Arial") || font.attr(W, "hAnsi") != Some("Arial") {
            return Err("Cell font must be explicit Arial".into());
        }
        for node in run.children.iter().filter(|c| c.name != "rPr") {
            if node.name == "br" {
                node.allowed(W, &[], &[])?;
                text.push('\n');
            } else if node.name == "tab" {
                node.allowed(W, &[], &[])?;
                text.push('\t');
            } else {
                if !node.children.is_empty()
                    || node.attrs.iter().any(|((ns, k), v)| {
                        ns != "http://www.w3.org/XML/1998/namespace"
                            || k != "space"
                            || !["preserve", "default"].contains(&v.as_str())
                    })
                {
                    return Err("Unsupported cell text properties".into());
                }
                text += &node.text;
            }
        }
    }
    Ok(text)
}
