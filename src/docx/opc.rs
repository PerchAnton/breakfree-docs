//! Resolve internal OPC relationships; inspect every referenced and unreferenced part.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
pub(super) const NUMBER_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml";
pub(super) struct Locations {
    pub document: String,
    pub styles: Option<String>,
    pub numbering: Option<String>,
    pub links: BTreeMap<String, String>,
}
fn resolve(base: &str, target: &str) -> Result<String> {
    if target.contains(['\\', ':', '%', '?', '#'])
        || target.chars().any(char::is_control)
        || target.starts_with("//")
    {
        return Err("Unsafe DOCX relationship target".into());
    }
    let mut path: Vec<&str> = if target.starts_with('/') {
        Vec::new()
    } else {
        base.rsplit_once('/')
            .map(|(p, _)| p.split('/').collect())
            .unwrap_or_default()
    };
    for c in target.trim_start_matches('/').split('/') {
        match c {
            "" => return Err("Empty relationship path component".into()),
            "." => {}
            ".." => {
                path.pop().ok_or("Relationship escapes package root")?;
            }
            _ => path.push(c),
        }
    }
    if path.is_empty() {
        return Err("Empty relationship target".into());
    }
    Ok(path.join("/"))
}
fn relationships(
    parts: &Parts,
    path: &str,
    base: &str,
    allowed: &[&str],
    links: &mut BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>> {
    let n = part(parts, path)?;
    root(&n, REL, "Relationships")?;
    n.allowed(REL, &["Relationship"], &[])?;
    let mut ids = BTreeSet::new();
    let mut result = BTreeMap::new();
    for r in &n.children {
        r.allowed(
            REL,
            &[],
            &[("", "Id"), ("", "Type"), ("", "Target"), ("", "TargetMode")],
        )?;
        let id = r.attr("", "Id").ok_or("Missing relationship ID")?;
        if id.is_empty() || !ids.insert(id) {
            return Err("Empty or duplicate relationship ID".into());
        }
        let kind = r
            .attr("", "Type")
            .and_then(|s| s.strip_prefix(OFFICE_REL))
            .filter(|s| allowed.contains(s))
            .ok_or("Unsupported DOCX relationship type")?;
        if kind == "hyperlink" {
            if r.attr("", "TargetMode") != Some("External") {
                return Err("Hyperlinks require an explicit external relationship".into());
            }
            let target = r.attr("", "Target").ok_or("Missing hyperlink target")?;
            crate::structure::validate_target(target)?;
            links.insert(id.into(), target.into());
            continue;
        }
        if r.attr("", "TargetMode").is_some_and(|v| v != "Internal") {
            return Err("Only hyperlink relationships may be external".into());
        }
        let target = resolve(
            base,
            r.attr("", "Target").ok_or("Missing relationship target")?,
        )?;
        if !parts.contains_key(&target) || result.insert(kind.into(), target).is_some() {
            return Err("Missing or duplicate DOCX relationship target".into());
        }
    }
    Ok(result)
}
pub(super) fn inspect(parts: &Parts) -> Result<Locations> {
    let mut links = BTreeMap::new();
    let main = relationships(parts, "_rels/.rels", "", &["officeDocument"], &mut links)?;
    let document = main
        .get("officeDocument")
        .ok_or("Missing main document relationship")?
        .clone();
    let (dir, file) = document.rsplit_once('/').unwrap_or(("", &document));
    let relpath = if dir.is_empty() {
        format!("_rels/{file}.rels")
    } else {
        format!("{dir}/_rels/{file}.rels")
    };
    let rels = if parts.contains_key(&relpath) {
        relationships(
            parts,
            &relpath,
            &document,
            &["styles", "numbering", "hyperlink"],
            &mut links,
        )?
    } else {
        BTreeMap::new()
    };
    let locations = Locations {
        document,
        styles: rels.get("styles").cloned(),
        numbering: rels.get("numbering").cloned(),
        links,
    };
    let mut expected = BTreeMap::from([
        (
            "_rels/.rels".to_owned(),
            "application/vnd.openxmlformats-package.relationships+xml",
        ),
        (locations.document.clone(), MAIN_TYPE),
    ]);
    if parts.contains_key(&relpath) {
        expected.insert(
            relpath,
            "application/vnd.openxmlformats-package.relationships+xml",
        );
    }
    for (name, kind) in [
        (&locations.styles, STYLE_TYPE),
        (&locations.numbering, NUMBER_TYPE),
    ] {
        if let Some(name) = name
            && expected.insert(name.clone(), kind).is_some()
        {
            return Err("One package part cannot serve multiple roles".into());
        }
    }
    let n = part(parts, "[Content_Types].xml")?;
    root(&n, CT, "Types")?;
    n.allowed(CT, &["Default", "Override"], &[])?;
    let mut defaults = BTreeMap::new();
    let mut overrides = BTreeMap::new();
    for c in &n.children {
        let content = c.attr("", "ContentType").ok_or("Missing content type")?;
        if c.name == "Default" {
            c.allowed(CT, &[], &[("", "Extension"), ("", "ContentType")])?;
            let ext = c.attr("", "Extension").ok_or("Missing extension")?;
            if !matches!(
                (ext, content),
                ("xml", "application/xml")
                    | (
                        "rels",
                        "application/vnd.openxmlformats-package.relationships+xml"
                    )
            ) || defaults.insert(ext, content).is_some()
            {
                return Err("Unsupported or duplicate default content type".into());
            }
        } else {
            c.allowed(CT, &[], &[("", "PartName"), ("", "ContentType")])?;
            let name = c
                .attr("", "PartName")
                .and_then(|s| s.strip_prefix('/'))
                .ok_or("Part name must be package-absolute")?;
            if !expected.contains_key(name) || overrides.insert(name, content).is_some() {
                return Err("Unsupported or duplicate content-type override".into());
            }
        }
    }
    for (name, kind) in &expected {
        let ty = overrides
            .get(name.as_str())
            .or_else(|| name.rsplit_once('.').and_then(|(_, ext)| defaults.get(ext)));
        if ty != Some(kind) {
            return Err(format!("Unsupported or missing content type for {name}"));
        }
    }
    for name in parts.keys() {
        if name != "[Content_Types].xml" && !expected.contains_key(name) {
            return Err(format!(
                "Unsupported DOCX part: {name}. Source remains unchanged."
            ));
        }
    }
    Ok(locations)
}
