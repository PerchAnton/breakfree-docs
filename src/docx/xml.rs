//! Namespace-aware, UTF-8 XML with bounded depth/nodes and no DTD or entities.
use crate::Result;
use quick_xml::{NsReader, events::Event, name::ResolveResult};
use std::collections::BTreeMap;
pub const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
pub const REL: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
pub const CT: &str = "http://schemas.openxmlformats.org/package/2006/content-types";
#[derive(Debug)]
pub struct Node {
    pub ns: String,
    pub name: String,
    pub attrs: BTreeMap<(String, String), String>,
    pub children: Vec<Node>,
    pub text: String,
}
impl Node {
    pub fn child(&self, name: &str) -> Option<&Self> {
        self.children.iter().find(|n| n.ns == W && n.name == name)
    }
    pub fn val(&self) -> Result<&str> {
        self.attr(W, "val")
            .ok_or_else(|| format!("Missing value on {}", self.name))
    }
    pub fn attr(&self, ns: &str, name: &str) -> Option<&str> {
        self.attrs
            .get(&(ns.into(), name.into()))
            .map(String::as_str)
    }
    pub fn allowed(&self, ns: &str, names: &[&str], attrs: &[(&str, &str)]) -> Result<()> {
        for n in &self.children {
            if n.ns != ns || !names.contains(&n.name.as_str()) {
                return Err(format!(
                    "Unsupported DOCX element: {} / {}. The source was not changed.",
                    self.name, n.name
                ));
            }
        }
        for (n, k) in self.attrs.keys() {
            if !attrs.contains(&(n.as_str(), k.as_str())) {
                return Err(format!("Unsupported DOCX attribute: {} / {k}.", self.name));
            }
        }
        if !self.text.trim().is_empty() {
            return Err(format!("Unexpected text in {}", self.name));
        }
        Ok(())
    }
}
fn ns(r: ResolveResult<'_>) -> Result<String> {
    match r {
        ResolveResult::Bound(n) => {
            String::from_utf8(n.as_ref().to_vec()).map_err(|e| e.to_string())
        }
        ResolveResult::Unbound => Ok(String::new()),
        ResolveResult::Unknown(_) => Err("Unbound XML prefix".into()),
    }
}
pub fn parse(bytes: &[u8]) -> Result<Node> {
    let input = std::str::from_utf8(bytes).map_err(|_| "DOCX XML must use UTF-8")?;
    if bytes.len() > super::package::PART_MAX {
        return Err("XML part exceeds limit".into());
    }
    let mut reader = NsReader::from_str(input);
    let mut stack: Vec<Node> = Vec::new();
    let mut root = None;
    let mut declared = false;
    let mut count = 0;
    loop {
        let event = reader
            .read_event()
            .map_err(|e| format!("Invalid DOCX XML: {e}"))?;
        let empty = matches!(&event, Event::Empty(_));
        match event {
            Event::Start(e) | Event::Empty(e) => {
                count += 1;
                if count > 200_000 || stack.len() >= 64 {
                    return Err("XML exceeds node/depth limits".into());
                }
                let (namespace, local) = reader.resolver().resolve_element(e.name());
                let mut node = Node {
                    ns: ns(namespace)?,
                    name: String::from_utf8(local.as_ref().to_vec()).map_err(|e| e.to_string())?,
                    attrs: BTreeMap::new(),
                    children: Vec::new(),
                    text: String::new(),
                };
                for a in e.attributes() {
                    let a = a.map_err(|e| e.to_string())?;
                    if a.key.as_ref() == b"xmlns" || a.key.as_ref().starts_with(b"xmlns:") {
                        continue;
                    }
                    let (n, k) = reader.resolver().resolve_attribute(a.key);
                    let key = (
                        ns(n)?,
                        String::from_utf8(k.as_ref().to_vec()).map_err(|e| e.to_string())?,
                    );
                    if node
                        .attrs
                        .insert(
                            key,
                            a.decoded_and_normalized_value(
                                quick_xml::XmlVersion::Implicit1_0,
                                reader.decoder(),
                            )
                            .map_err(|e| e.to_string())?
                            .into_owned(),
                        )
                        .is_some()
                    {
                        return Err("Duplicate XML attribute".into());
                    }
                }
                if empty {
                    attach(node, &mut stack, &mut root)?;
                } else {
                    stack.push(node);
                }
            }
            Event::End(_) => {
                let n = stack.pop().ok_or("Unexpected XML closing tag")?;
                attach(n, &mut stack, &mut root)?;
            }
            Event::Text(e) => {
                let t = e
                    .xml_content(quick_xml::XmlVersion::Implicit1_0)
                    .map_err(|e| e.to_string())?;
                if let Some(n) = stack.last_mut() {
                    n.text.push_str(&t);
                } else if !t.trim().is_empty() {
                    return Err("Text outside XML root".into());
                }
            }
            Event::GeneralRef(e) => {
                let s = e.decode().map_err(|e| e.to_string())?;
                let text = quick_xml::escape::unescape(&format!("&{s};"))
                    .map_err(|_| "Unsupported XML entity")?
                    .into_owned();
                stack
                    .last_mut()
                    .ok_or("Entity outside root")?
                    .text
                    .push_str(&text);
            }
            Event::Decl(e) => {
                if declared
                    || root.is_some()
                    || !stack.is_empty()
                    || e.version().map_err(|e| e.to_string())?.as_ref() != b"1.0"
                {
                    return Err("Invalid XML declaration position or version".into());
                }
                declared = true;
                if e.encoding()
                    .transpose()
                    .map_err(|e| e.to_string())?
                    .is_some_and(|v| !v.eq_ignore_ascii_case(b"utf-8"))
                {
                    return Err("Only UTF-8 XML is supported".into());
                }
            }
            Event::Comment(_) => {}
            Event::Eof => break,
            _ => {
                return Err(
                    "DTD, processing instructions and CDATA are unsupported in DOCX XML".into(),
                );
            }
        }
    }
    if !stack.is_empty() {
        return Err("Unclosed XML elements".into());
    }
    root.ok_or("Missing XML root".into())
}
fn attach(n: Node, stack: &mut [Node], root: &mut Option<Node>) -> Result<()> {
    if let Some(p) = stack.last_mut() {
        p.children.push(n);
    } else if root.replace(n).is_some() {
        return Err("Multiple XML roots".into());
    }
    Ok(())
}
pub fn escape(text: &str) -> String {
    quick_xml::escape::escape(text).into_owned()
}
