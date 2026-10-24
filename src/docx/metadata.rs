//! Preserve bounded non-rendering package properties verbatim, including unknown fields.
use super::*;
use std::collections::BTreeMap;
pub(super) const CORE: &str =
    "http://schemas.openxmlformats.org/package/2006/metadata/core-properties";
pub(super) const APP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/extended-properties";
pub(super) fn mime(key: &str) -> &'static str {
    if key == "core" {
        "application/vnd.openxmlformats-package.core-properties+xml"
    } else {
        "application/vnd.openxmlformats-officedocument.extended-properties+xml"
    }
}
pub(crate) fn validate(properties: &BTreeMap<String, String>) -> Result<()> {
    for (key, text) in properties {
        if text.len() > 128 * 1024 {
            return Err("Metadata exceeds 128 KiB".into());
        }
        let n = xml::parse(text.as_bytes())?;
        match key.as_str() {
            "core" => root(&n, CORE, "coreProperties")?,
            "app" => root(&n, APP, "Properties")?,
            _ => return Err("Unknown metadata part".into()),
        };
    }
    Ok(())
}
pub(super) fn write(d: &Document, parts: &mut Parts) -> Result<()> {
    validate(&d.package_properties)?;
    for (key, text) in &d.package_properties {
        let path = format!("docProps/{key}.xml");
        parts.insert(path.clone(), text.as_bytes().to_vec());
        let kind = if key == "core" {
            "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties"
                .into()
        } else {
            format!("{OFFICE_REL}extended-properties")
        };
        for (part, end, item) in [
            (
                "_rels/.rels",
                "</Relationships>",
                format!("<Relationship Id=\"rId{key}\" Type=\"{kind}\" Target=\"{path}\"/>"),
            ),
            (
                "[Content_Types].xml",
                "</Types>",
                format!(
                    "<Override PartName=\"/{path}\" ContentType=\"{}\"/>",
                    mime(key)
                ),
            ),
        ] {
            let s = String::from_utf8(parts[part].clone())
                .unwrap()
                .replace(end, &(item + end));
            parts.insert(part.into(), s.into_bytes());
        }
    }
    Ok(())
}
