//! Hyperlink ranges are model metadata; importing never activates a target.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn read(
    n: &Node,
    targets: &BTreeMap<String, String>,
    used: &mut BTreeSet<String>,
    map: &styles::Map,
    p: &mut Paragraph,
) -> Result<()> {
    let r = OFFICE_REL.trim_end_matches('/');
    n.allowed(W, &["r"], &[(r, "id")])?;
    let id = n.attr(r, "id").ok_or("Hyperlink has no relationship ID")?;
    let target = targets.get(id).ok_or("Missing hyperlink relationship")?;
    let start = p.styles.len();
    for run in &n.children {
        read_run(run, map, p)?;
    }
    let end = p.styles.len();
    if start == end {
        return Err("Empty hyperlink cannot be preserved".into());
    }
    p.structure.links.push(crate::structure::Link {
        start,
        end,
        target: target.clone(),
    });
    used.insert(id.into());
    Ok(())
}
pub(super) fn read_run(run: &Node, map: &styles::Map, p: &mut Paragraph) -> Result<()> {
    run.allowed(W, &["rPr", "t", "tab"], &[])?;
    if run.children.iter().filter(|n| n.name == "rPr").count() > 1 {
        return Err("Duplicate run properties".into());
    }
    let format = run
        .child("rPr")
        .map(|r| properties::read_run(r, map))
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
    Ok(())
}
