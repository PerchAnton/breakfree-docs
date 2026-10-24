//! Decimal/bullet numbering, kept separate from paragraph text and run formatting.
use super::*;
use crate::structure::{ListItem, ListKind};
use std::collections::BTreeMap;
#[derive(Clone)]
struct Level {
    kind: ListKind,
    start: u16,
}
struct Instance {
    levels: BTreeMap<u16, Level>,
    overrides: BTreeMap<u16, u16>,
}
#[derive(Default)]
pub(super) struct Numbering {
    instances: BTreeMap<u16, Instance>,
    counters: BTreeMap<u16, [u32; 9]>,
    native: [u32; 9],
    prior: Option<ListKind>,
    active: [Option<u16>; 9],
}
fn value(n: &Node, name: &str) -> Result<u16> {
    let c = n
        .child(name)
        .ok_or_else(|| format!("Missing numbering {name}"))?;
    properties::leaf(c)?;
    properties::unit(c, "val", 1)
}
pub(super) fn read(n: &Node) -> Result<Numbering> {
    root(n, W, "numbering")?;
    n.allowed(W, &["abstractNum", "num"], &[])?;
    if n.children.len() > 4096 {
        return Err("Too many numbering definitions".into());
    }
    let mut abstracts = BTreeMap::new();
    let mut result = Numbering::default();
    for a in n.children.iter().filter(|n| n.name == "abstractNum") {
        a.allowed(W, &["multiLevelType", "lvl"], &[(W, "abstractNumId")])?;
        let id = properties::unit(a, "abstractNumId", 1)?;
        let mut levels = BTreeMap::new();
        if let Some(t) = a.child("multiLevelType") {
            properties::leaf(t)?;
            if !["multilevel", "singleLevel", "hybridMultilevel"].contains(&t.val()?) {
                return Err("Unsupported numbering kind".into());
            }
        }
        if a.children
            .iter()
            .filter(|c| c.name == "multiLevelType")
            .count()
            > 1
        {
            return Err("Duplicate numbering type".into());
        }
        for l in a.children.iter().filter(|c| c.name == "lvl") {
            l.allowed(
                W,
                &["start", "numFmt", "lvlText", "lvlJc", "suff", "pPr"],
                &[(W, "ilvl")],
            )?;
            singleton_children(l)?;
            let level = properties::unit(l, "ilvl", 1)?;
            if level > 8 {
                return Err("Numbering level must be 0–8".into());
            }
            let start = l
                .child("start")
                .map(|_| value(l, "start"))
                .transpose()?
                .unwrap_or(1);
            if start == 0 {
                return Err("Zero-based numbering is unsupported".into());
            }
            let fmt = l.child("numFmt").ok_or("Missing numbering format")?;
            properties::leaf(fmt)?;
            let kind = match fmt.val()? {
                "decimal" => ListKind::Decimal,
                "bullet" => ListKind::Bullet,
                _ => return Err("Only decimal and bullet lists are supported".into()),
            };
            let text = l.child("lvlText").ok_or("Missing numbering text")?;
            properties::leaf(text)?;
            let expected = match kind {
                ListKind::Decimal => format!("%{}.", level + 1),
                ListKind::Bullet => ["•", "◦", "▪"][usize::from(level) % 3].into(),
            };
            if text.val()? != expected {
                return Err("Unsupported numbering label or compound numbering".into());
            }
            for (name, expected) in [("lvlJc", "left"), ("suff", "tab")] {
                if let Some(c) = l.child(name) {
                    properties::leaf(c)?;
                    if c.val()? != expected {
                        return Err(format!("Unsupported numbering {name}"));
                    }
                }
            }
            // The native renderer adds this list inset to the paragraph's text geometry.
            if let Some(pr) = l.child("pPr") {
                pr.allowed(W, &["ind"], &[])?;
                singleton_children(pr)?;
                let ind = pr.child("ind").ok_or("Missing list indentation")?;
                ind.allowed(W, &[], &[(W, "left"), (W, "hanging")])?;
                if properties::unit(ind, "left", 1)? != 480 + 360 * level
                    || properties::unit(ind, "hanging", 1)? != 360
                {
                    return Err(
                        "List indentation is outside the supported 24 pt + 18 pt/level profile"
                            .into(),
                    );
                }
            } else {
                return Err("List indentation must be explicit".into());
            }
            if levels.insert(level, Level { kind, start }).is_some() {
                return Err("Duplicate numbering level".into());
            }
        }
        if levels.is_empty() || abstracts.insert(id, levels).is_some() {
            return Err("Empty or duplicate abstract numbering".into());
        }
    }
    for n in n.children.iter().filter(|n| n.name == "num") {
        n.allowed(W, &["abstractNumId", "lvlOverride"], &[(W, "numId")])?;
        let id = properties::unit(n, "numId", 1)?;
        if id == 0 {
            return Err("Numbering instance zero is reserved".into());
        }
        if n.children
            .iter()
            .filter(|n| n.name == "abstractNumId")
            .count()
            != 1
        {
            return Err("Numbering requires one abstract reference".into());
        }
        let levels = abstracts
            .get(&value(n, "abstractNumId")?)
            .ok_or("Unknown abstract numbering reference")?
            .clone();
        let mut overrides = BTreeMap::new();
        for o in n.children.iter().filter(|n| n.name == "lvlOverride") {
            o.allowed(W, &["startOverride"], &[(W, "ilvl")])?;
            singleton_children(o)?;
            let level = properties::unit(o, "ilvl", 1)?;
            let start = value(o, "startOverride")?;
            if !levels.contains_key(&level)
                || start == 0
                || overrides.insert(level, start).is_some()
            {
                return Err("Invalid/duplicate numbering override".into());
            }
        }
        if result
            .instances
            .insert(id, Instance { levels, overrides })
            .is_some()
        {
            return Err("Duplicate numbering instance".into());
        }
    }
    Ok(result)
}
impl Numbering {
    pub fn paragraph(&mut self, pr: Option<&Node>) -> Result<Option<ListItem>> {
        let Some(n) = pr.and_then(|n| n.child("numPr")) else {
            self.prior = None;
            self.native = [0; 9];
            self.active = [None; 9];
            return Ok(None);
        };
        n.allowed(W, &["ilvl", "numId"], &[])?;
        singleton_children(n)?;
        let id = value(n, "numId")?;
        if id == 0 {
            if let Some(l) = n.child("ilvl") {
                properties::leaf(l)?;
                properties::unit(l, "val", 1)?;
            }
            self.prior = None;
            self.native = [0; 9];
            self.active = [None; 9];
            return Ok(None);
        }
        let level = n
            .child("ilvl")
            .map(|_| value(n, "ilvl"))
            .transpose()?
            .unwrap_or(0);
        let instance = self
            .instances
            .get(&id)
            .ok_or("Unknown numbering instance")?;
        let definition = instance
            .levels
            .get(&level)
            .ok_or("Unknown numbering level")?;
        let level = usize::from(level);
        if self.prior != Some(definition.kind) && self.counters.contains_key(&id) {
            return Err(
                "Linked/discontinuous numbering cannot be preserved across list breaks".into(),
            );
        }
        let counts = self.counters.entry(id).or_insert([0; 9]);
        let first = counts[level] == 0;
        let start = instance
            .overrides
            .get(&(level as u16))
            .copied()
            .unwrap_or(definition.start);
        let number = if first {
            u32::from(start)
        } else {
            counts[level] + 1
        };
        if number > u32::from(u16::MAX) {
            return Err("List counter exceeds supported range".into());
        }
        counts[level] = number;
        // OOXML defaults reset a level only when its immediately previous level is used.
        if level < 8 {
            counts[level + 1] = 0;
        }
        if self.prior != Some(definition.kind) {
            self.native = [0; 9];
            self.active = [None; 9];
        }
        self.prior = Some(definition.kind);
        let expected = self.native[level] + 1;
        if !first && (expected != number || self.active[level] != Some(id)) {
            return Err(
                "Linked/discontinuous numbering cannot be preserved by sequential list editing"
                    .into(),
            );
        }
        let explicit = first && instance.overrides.contains_key(&(level as u16));
        let restart = (explicit || number != expected).then_some(number as u16);
        self.native[level] = number;
        self.active[level] = Some(id);
        for next in level + 1..9 {
            self.native[next] = 0;
            self.active[next] = None;
        }
        Ok(Some(ListItem {
            kind: definition.kind,
            level: level as u8,
            restart,
        }))
    }
}
fn definition(id: u16, kind: ListKind) -> String {
    let mut out = String::new();

    out += &format!(
        "<w:abstractNum w:abstractNumId=\"{id}\"><w:multiLevelType w:val=\"multilevel\"/>"
    );
    for level in 0..9 {
        let label = if kind == ListKind::Decimal {
            format!("%{}.", level + 1)
        } else {
            ["•", "◦", "▪"][level % 3].into()
        };
        out += &format!(
            "<w:lvl w:ilvl=\"{level}\"><w:start w:val=\"1\"/><w:numFmt w:val=\"{}\"/><w:lvlText w:val=\"{label}\"/><w:lvlJc w:val=\"left\"/><w:suff w:val=\"tab\"/><w:pPr><w:ind w:left=\"{}\" w:hanging=\"360\"/></w:pPr></w:lvl>",
            if kind == ListKind::Decimal {
                "decimal"
            } else {
                "bullet"
            },
            480 + 360 * level
        );
    }
    out += "</w:abstractNum>";
    out
}
/// Separate numbering streams per active level avoid implicit OOXML reset rules.
/// Both abstract definition and instance are replaced on a restart/reset/break:
/// Word may continue numbering across instances sharing an abstract definition.
pub(super) fn write(d: &Document) -> Result<(String, Vec<Option<u16>>)> {
    let mut out = format!("{DECL}<w:numbering xmlns:w=\"{W}\">");
    let mut instances = String::new();
    let mut streams = [None; 9];
    let mut prior = None;
    let mut ids = Vec::new();
    let mut next = 1u16;
    for p in &d.paragraphs {
        let Some(item) = &p.structure.list else {
            streams = [None; 9];
            prior = None;
            ids.push(None);
            continue;
        };
        if prior != Some(item.kind) {
            streams = [None; 9];
        }
        prior = Some(item.kind);
        let level = usize::from(item.level);
        if item.restart.is_some() {
            streams[level] = None;
        }
        if streams[level].is_none() {
            if next > 1024 {
                return Err("DOCX list instance limit exceeded".into());
            }
            let id = next;
            next += 1;
            streams[level] = Some(id);
            out += &definition(id, item.kind);
            instances += &format!("<w:num w:numId=\"{id}\"><w:abstractNumId w:val=\"{id}\"/>");
            if let Some(start) = item.restart {
                instances += &format!(
                    "<w:lvlOverride w:ilvl=\"{level}\"><w:startOverride w:val=\"{start}\"/></w:lvlOverride>"
                );
            }
            instances += "</w:num>";
        }
        ids.push(streams[level]);
        for s in streams.iter_mut().skip(level + 1) {
            *s = None;
        }
    }
    out += &instances;
    out += "</w:numbering>";
    Ok((out, ids))
}
pub(super) fn check_geometry(d: &Document) -> Result<()> {
    for p in &d.paragraphs {
        if p.structure.list.is_some() {
            let s = d.style_catalog.resolve(p.paragraph_style);
            if s.character != Style::default()
                || s.left_indent_half_points != 0
                || s.right_indent_half_points != 0
                || s.first_line_half_points != 0
                || p.spacing.left_indent_half_points.is_some()
                || p.spacing.right_indent_half_points.is_some()
                || p.spacing.first_line_half_points.is_some()
            {
                return Err("DOCX lists currently require unmodified 11 pt paragraph font properties and default paragraph indents; save as .bfdoc to preserve this layout".into());
            }
        }
    }
    Ok(())
}
