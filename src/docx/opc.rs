//! Resolve internal OPC relationships; inspect every referenced and unreferenced part.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
pub(super) const NUMBER_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml";
pub(super) struct Locations {
    pub document: String,
    pub metadata: BTreeMap<String, String>,
    pub styles: Option<String>,
    pub numbering: Option<String>,
    pub links: BTreeMap<String, String>,
    pub images: BTreeMap<String, String>,
    pub sections: BTreeMap<String, (String, String)>,
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
    images: &mut BTreeMap<String, String>,
    sections: &mut BTreeMap<String, (String, String)>,
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
            .and_then(|s| s.strip_prefix(OFFICE_REL).or_else(||(s=="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties").then_some("core-properties")))
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
        if matches!(kind, "header" | "footer") {
            if !parts.contains_key(&target) {
                return Err("Missing header/footer part".into());
            }
            sections.insert(id.into(), (kind.into(), target));
            continue;
        }
        if kind == "image" {
            if !parts.contains_key(&target) {
                return Err("Missing image part".into());
            }
            images.insert(id.into(), target);
            continue;
        }
        if !parts.contains_key(&target) || result.insert(kind.into(), target).is_some() {
            return Err("Missing or duplicate DOCX relationship target".into());
        }
    }
    Ok(result)
}
pub(super) fn inspect(parts: &Parts) -> Result<Locations> {
    let mut links = BTreeMap::new();
    let mut images = BTreeMap::new();
    let mut sections = BTreeMap::new();
    let main = relationships(
        parts,
        "_rels/.rels",
        "",
        &["officeDocument", "core-properties", "extended-properties"],
        &mut links,
        &mut images,
        &mut sections,
    )?;
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
            &[
                "styles",
                "numbering",
                "hyperlink",
                "image",
                "header",
                "footer",
            ],
            &mut links,
            &mut images,
            &mut sections,
        )?
    } else {
        BTreeMap::new()
    };
    let locations = Locations {
        metadata: [("core", "core-properties"), ("app", "extended-properties")]
            .into_iter()
            .filter_map(|(k, v)| main.get(v).map(|p| (k.into(), p.clone())))
            .collect(),
        document,
        styles: rels.get("styles").cloned(),
        numbering: rels.get("numbering").cloned(),
        links,
        images,
        sections,
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
    for (key, path) in &locations.metadata {
        if expected.insert(path.clone(), metadata::mime(key)).is_some() {
            return Err("Metadata part has another role".into());
        }
    }
    for (kind, path) in locations.sections.values() {
        let mime = if kind == "header" {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"
        } else {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"
        };
        if let Some(prior) = expected.insert(path.clone(), mime)
            && prior != mime
        {
            return Err("Header/footer part has another role".into());
        }
    }
    for path in locations.images.values() {
        if let Some(prior) = expected.insert(path.clone(), "image/png")
            && prior != "image/png"
        {
            return Err("Image part has another role".into());
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
                    | ("png", "image/png")
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
