//! Strict .bfdoc v1–v19 reading, v19 writing and checked temporary-file replacement.
use crate::{Result, model::*};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

pub const FORMAT_ID: &str = "breakfree.docs.native";
pub const CONTENT_VERSION: u32 = 19;
pub const MAX_FILE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Native<R> {
    format: String,
    content_version: u32,
    paragraphs: Vec<NativeParagraph<R>>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeParagraph<R> {
    alignment: Alignment,
    runs: Vec<R>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Run {
    text: String,
    bold: bool,
    italic: bool,
    underline: bool,
    font_size_half_points: FontSize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunV1 {
    text: String,
    bold: bool,
    italic: bool,
}
impl From<RunV1> for Run {
    fn from(run: RunV1) -> Self {
        Self {
            text: run.text,
            bold: run.bold,
            italic: run.italic,
            underline: false,
            font_size_half_points: FontSize::default(),
        }
    }
}

// Inspect only the header first, then deserialize the entire versioned schema.
// Do not round-trip through Value: that would discard duplicate JSON keys.
#[derive(Deserialize)]
struct Header {
    format: String,
    content_version: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeV3 {
    format: String,
    content_version: u32,
    paragraphs: Vec<ParagraphV3>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeV4 {
    format: String,
    content_version: u32,
    style_catalog: CatalogV4,
    paragraphs: Vec<ParagraphV3>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogV4 {
    normal: LegacyDefinition,
    no_spacing: LegacyDefinition,
    heading1: LegacyDefinition,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeV6 {
    format: String,
    content_version: u32,
    style_catalog: StyleCatalog,
    paragraphs: Vec<ParagraphV3>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParagraphV3 {
    alignment: Alignment,
    paragraph_style: ParagraphStyle,
    runs: Vec<RunV3>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RunV3 {
    text: String,
    direct: CharacterFormat,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeV7 {
    format: String,
    content_version: u32,
    style_catalog: StyleCatalog,
    paragraphs: Vec<ParagraphV7>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParagraphV7 {
    alignment: Alignment,
    paragraph_style: ParagraphStyle,
    runs: Vec<RunV7>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RunV7 {
    text: String,
    character_style: Option<CharacterStyle>,
    direct: CharacterFormat,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeV9 {
    format: String,
    content_version: u32,
    style_catalog: StyleCatalog,
    character_styles: Vec<NamedCharacterStyle>,
    paragraphs: Vec<ParagraphV7>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParagraphV10 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    structure: Option<crate::structure::ParagraphStructure>,
    alignment: Alignment,
    paragraph_style: ParagraphStyle,
    spacing: ParagraphSpacing,
    runs: Vec<RunV7>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeSpacing {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    page: Option<crate::structure::Section>,
    format: String,
    content_version: u32,
    style_catalog: StyleCatalog,
    character_styles: Vec<NamedCharacterStyle>,
    paragraphs: Vec<ParagraphV10>,
}

// Frozen v8 rejects parent fields instead of silently importing newer semantics.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CharacterStyleV8 {
    id: u32,
    name: String,
    format: CharacterFormat,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeV8 {
    format: String,
    content_version: u32,
    style_catalog: StyleCatalog,
    character_styles: Vec<CharacterStyleV8>,
    paragraphs: Vec<ParagraphV7>,
}

// Frozen v4/v5 definitions reject v6 fields instead of silently accepting them.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyDefinition {
    parent: Option<ParagraphStyle>,
    character: CharacterFormat,
    line_height_percent: Option<u16>,
    space_after_half_points: Option<u16>,
}
impl From<LegacyDefinition> for StyleDefinition {
    fn from(d: LegacyDefinition) -> Self {
        Self {
            tab_interval_half_points: None,
            tab_stops: None,
            next_style: None,
            parent: d.parent,
            character: d.character,
            line_height_percent: d.line_height_percent,
            space_after_half_points: d.space_after_half_points,
            space_before_half_points: None,
            left_indent_half_points: None,
            right_indent_half_points: None,
            first_line_half_points: None,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyCustom {
    id: u32,
    name: String,
    definition: LegacyDefinition,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogV5 {
    normal: LegacyDefinition,
    no_spacing: LegacyDefinition,
    heading1: LegacyDefinition,
    custom: Vec<LegacyCustom>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeV5 {
    format: String,
    content_version: u32,
    style_catalog: CatalogV5,
    paragraphs: Vec<ParagraphV3>,
}

pub fn encode(document: &Document) -> Result<Vec<u8>> {
    document.validate()?;
    let paragraphs = document
        .paragraphs
        .iter()
        .map(|p| {
            let mut runs: Vec<RunV7> = Vec::new();
            for (ch, &direct) in p.text.chars().zip(&p.styles) {
                if let Some(last) = runs.last_mut().filter(|r| r.direct == direct) {
                    last.text.push(ch);
                } else {
                    runs.push(RunV7 {
                        text: ch.to_string(),
                        character_style: direct.character_style,
                        direct,
                    });
                }
            }
            ParagraphV10 {
                structure: Some(p.structure.clone()),
                spacing: p.spacing.clone(),
                alignment: p.alignment,
                paragraph_style: p.paragraph_style,
                runs,
            }
        })
        .collect();
    let mut bytes = serde_json::to_vec_pretty(&NativeSpacing {
        page: Some(document.page.clone()),
        format: FORMAT_ID.into(),
        content_version: CONTENT_VERSION,
        style_catalog: document.style_catalog.clone(),
        character_styles: document.character_styles.clone(),
        paragraphs,
    })
    .map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    if bytes.len() > MAX_FILE_BYTES {
        return Err("File exceeds 16 MiB.".into());
    }
    Ok(bytes)
}

fn decode_styled(paragraphs: Vec<ParagraphV3>, style_catalog: StyleCatalog) -> Result<Document> {
    decode_catalog(paragraphs, style_catalog, Vec::new())
}
fn decode_catalog(
    paragraphs: Vec<ParagraphV3>,
    style_catalog: StyleCatalog,
    character_styles: Vec<NamedCharacterStyle>,
) -> Result<Document> {
    let result = decode_catalog_raw(paragraphs, style_catalog, character_styles)?;
    result.validate()?;
    Ok(result)
}
fn decode_catalog_raw(
    paragraphs: Vec<ParagraphV3>,
    style_catalog: StyleCatalog,
    character_styles: Vec<NamedCharacterStyle>,
) -> Result<Document> {
    style_catalog.validate()?;
    let native_paragraphs = paragraphs;
    let mut paragraphs = Vec::new();
    let mut total = native_paragraphs.len().saturating_sub(1);
    for p in native_paragraphs {
        let mut paragraph = Paragraph {
            alignment: p.alignment,
            paragraph_style: p.paragraph_style,
            ..Default::default()
        };
        for run in p.runs {
            if run.text.is_empty() {
                return Err("Native runs must be nonempty.".into());
            }
            let count = run.text.chars().count();
            total += count;
            if total > MAX_CHARACTERS {
                return Err("Document exceeds 100,000 characters.".into());
            }
            paragraph
                .styles
                .extend(std::iter::repeat_n(run.direct, count));
            paragraph.text.push_str(&run.text);
        }
        paragraphs.push(paragraph);
    }
    let result = Document {
        paragraphs,
        style_catalog,
        character_styles,
        ..Default::default()
    };
    Ok(result)
}

fn decode_character_paragraphs(
    paragraphs: Vec<ParagraphV7>,
    catalog: StyleCatalog,
    character_styles: Vec<NamedCharacterStyle>,
) -> Result<Document> {
    let paragraphs = paragraphs
        .into_iter()
        .map(|p| ParagraphV3 {
            alignment: p.alignment,
            paragraph_style: p.paragraph_style,
            runs: p
                .runs
                .into_iter()
                .map(|r| RunV3 {
                    text: r.text,
                    direct: CharacterFormat {
                        character_style: r.character_style,
                        ..r.direct
                    },
                })
                .collect(),
        })
        .collect();
    decode_catalog_raw(paragraphs, catalog, character_styles)
}

pub fn decode(bytes: &[u8]) -> Result<Document> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err("File exceeds 16 MiB.".into());
    }
    let file: Header =
        serde_json::from_slice(bytes).map_err(|e| format!("Invalid native document: {e}"))?;
    if file.format != FORMAT_ID {
        return Err("Not a BreakFree Docs native document.".into());
    }
    if file.content_version < 19 {
        let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if value.get("page").is_some()
            || value
                .get("paragraphs")
                .and_then(|p| p.as_array())
                .is_some_and(|p| p.iter().any(|v| v.get("structure").is_some()))
        {
            return Err("Document structure and page settings require native content v19.".into());
        }
    }
    // Legacy catalogs/spacing share model types. Explicitly freeze the new v11
    // property for every older schema, even when null. This is only a gate: full
    // typed deserialization below still rejects duplicates and unknown fields.
    if file.content_version < 11 {
        fn has_before(value: &serde_json::Value) -> bool {
            match value {
                serde_json::Value::Object(map) => {
                    map.contains_key("space_before_half_points") || map.values().any(has_before)
                }
                serde_json::Value::Array(items) => items.iter().any(has_before),
                _ => false,
            }
        }
        let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if has_before(&value) {
            return Err("Space before requires native content v11.".into());
        }
    }
    if file.content_version < 12 {
        fn has_indent(value: &serde_json::Value) -> bool {
            match value {
                serde_json::Value::Object(map) => {
                    map.contains_key("left_indent_half_points")
                        || map.contains_key("right_indent_half_points")
                        || map.values().any(has_indent)
                }
                serde_json::Value::Array(items) => items.iter().any(has_indent),
                _ => false,
            }
        }
        let value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if has_indent(&value) {
            return Err("Paragraph indents require native content v12.".into());
        }
    }
    if file.content_version < 13 {
        fn has_special(value: &serde_json::Value) -> bool {
            match value {
                serde_json::Value::Object(map) => {
                    map.contains_key("first_line_half_points") || map.values().any(has_special)
                }
                serde_json::Value::Array(items) => items.iter().any(has_special),
                _ => false,
            }
        }
        let value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if has_special(&value) {
            return Err("Special indent requires native content v13.".into());
        }
    }
    if file.content_version < 14 {
        fn has_tabs(value: &serde_json::Value) -> bool {
            match value {
                serde_json::Value::Object(map) => {
                    map.contains_key("tab_interval_half_points") || map.values().any(has_tabs)
                }
                serde_json::Value::Array(items) => items.iter().any(has_tabs),
                _ => false,
            }
        }
        let value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if has_tabs(&value) {
            return Err("Tab intervals require native content v14.".into());
        }
    }
    if file.content_version < 15 {
        fn has_stops(value: &serde_json::Value) -> bool {
            match value {
                serde_json::Value::Object(map) => {
                    map.contains_key("left_tab_stops_half_points") || map.values().any(has_stops)
                }
                serde_json::Value::Array(items) => items.iter().any(has_stops),
                _ => false,
            }
        }
        let value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if has_stops(&value) {
            return Err("Explicit left tab stops require native content v15.".into());
        }
    }
    if file.content_version < 17 {
        fn has_leaders(value: &serde_json::Value) -> bool {
            match value {
                serde_json::Value::Object(map) => {
                    map.contains_key("tab_leaders") || map.values().any(has_leaders)
                }
                serde_json::Value::Array(items) => items.iter().any(has_leaders),
                _ => false,
            }
        }
        let value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if has_leaders(&value) {
            return Err("Tab leaders require native content v17.".into());
        }
    }
    if file.content_version < 16 {
        fn has_aligned_stops(value: &serde_json::Value) -> bool {
            match value {
                serde_json::Value::Object(map) => {
                    map.contains_key("aligned_tab_stops") || map.values().any(has_aligned_stops)
                }
                serde_json::Value::Array(items) => items.iter().any(has_aligned_stops),
                _ => false,
            }
        }
        let value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if has_aligned_stops(&value) {
            return Err("Center/right tab stops require native content v16.".into());
        }
    }
    if file.content_version < 18 {
        let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        fn has_new_keys(value: &serde_json::Value) -> bool {
            match value {
                serde_json::Value::Object(map) => {
                    map.contains_key("tab_stops")
                        || map.contains_key("clear_tab_stops")
                        || map.values().any(has_new_keys)
                }
                serde_json::Value::Array(values) => values.iter().any(has_new_keys),
                _ => false,
            }
        }
        fn has_style_interval(value: &serde_json::Value) -> bool {
            match value {
                serde_json::Value::Object(map) => {
                    map.contains_key("tab_interval_half_points")
                        || map.values().any(has_style_interval)
                }
                serde_json::Value::Array(values) => values.iter().any(has_style_interval),
                _ => false,
            }
        }
        if has_new_keys(&value) || value.get("style_catalog").is_some_and(has_style_interval) {
            return Err("Style tabulation requires native content v18.".into());
        }
    }
    let native_paragraphs = match file.content_version {
        1 => serde_json::from_slice::<Native<RunV1>>(bytes)
            .map_err(|e| format!("Invalid native v1 document: {e}"))?
            .paragraphs
            .into_iter()
            .map(|p| NativeParagraph {
                alignment: p.alignment,
                runs: p.runs.into_iter().map(Run::from).collect(),
            })
            .collect(),
        3 => {
            let file: NativeV3 = serde_json::from_slice(bytes)
                .map_err(|e| format!("Invalid native v3 document: {e}"))?;
            return decode_styled(file.paragraphs, StyleCatalog::default());
        }
        4 => {
            let file: NativeV4 = serde_json::from_slice(bytes)
                .map_err(|e| format!("Invalid native v4 document: {e}"))?;
            return decode_styled(
                file.paragraphs,
                StyleCatalog {
                    normal: file.style_catalog.normal.into(),
                    no_spacing: file.style_catalog.no_spacing.into(),
                    heading1: file.style_catalog.heading1.into(),
                    custom: Vec::new(),
                },
            );
        }
        5 => {
            let file: NativeV5 = serde_json::from_slice(bytes)
                .map_err(|e| format!("Invalid native v5 document: {e}"))?;
            let _ = (&file.format, file.content_version);
            let c = file.style_catalog;
            return decode_styled(
                file.paragraphs,
                StyleCatalog {
                    normal: c.normal.into(),
                    no_spacing: c.no_spacing.into(),
                    heading1: c.heading1.into(),
                    custom: c
                        .custom
                        .into_iter()
                        .map(|s| CustomStyle {
                            id: s.id,
                            name: s.name,
                            definition: s.definition.into(),
                        })
                        .collect(),
                },
            );
        }
        6 => {
            let file: NativeV6 = serde_json::from_slice(bytes)
                .map_err(|e| format!("Invalid native v6 document: {e}"))?;
            return decode_styled(file.paragraphs, file.style_catalog);
        }
        10..=18 | CONTENT_VERSION => {
            let file: NativeSpacing = serde_json::from_slice(bytes)
                .map_err(|e| format!("Invalid native spacing document: {e}"))?;
            if file.content_version == CONTENT_VERSION
                && (file.page.is_none() || file.paragraphs.iter().any(|p| p.structure.is_none()))
            {
                return Err("Native v19 requires page settings and paragraph structure.".into());
            }
            let page = file.page.unwrap_or_default();
            let (paragraphs, spacing): (Vec<_>, Vec<_>) = file
                .paragraphs
                .into_iter()
                .map(|p| {
                    (
                        ParagraphV7 {
                            alignment: p.alignment,
                            paragraph_style: p.paragraph_style,
                            runs: p.runs,
                        },
                        (p.spacing, p.structure.unwrap_or_default()),
                    )
                })
                .unzip();
            let mut document =
                decode_character_paragraphs(paragraphs, file.style_catalog, file.character_styles)?;
            document.page = page;
            for (p, (spacing, structure)) in document.paragraphs.iter_mut().zip(spacing) {
                p.spacing = spacing;
                p.structure = structure;
            }
            document.validate()?;
            for paragraph in &document.paragraphs {
                if let Some(crate::structure::Block::Picture(picture)) = &paragraph.structure.block
                {
                    picture.validate_pixels()?;
                }
            }
            return Ok(document);
        }
        7..=9 => {
            let (paragraphs, catalog, character_styles) = if file.content_version == 7 {
                let file: NativeV7 = serde_json::from_slice(bytes)
                    .map_err(|e| format!("Invalid native v7 document: {e}"))?;
                (file.paragraphs, file.style_catalog, Vec::new())
            } else if file.content_version == 8 {
                let file: NativeV8 = serde_json::from_slice(bytes)
                    .map_err(|e| format!("Invalid native v8 document: {e}"))?;
                let _ = (&file.format, file.content_version);
                let styles = file
                    .character_styles
                    .into_iter()
                    .map(|s| NamedCharacterStyle {
                        id: s.id,
                        name: s.name,
                        format: s.format,
                        parent: None,
                    })
                    .collect();
                (file.paragraphs, file.style_catalog, styles)
            } else {
                let file: NativeV9 = serde_json::from_slice(bytes)
                    .map_err(|e| format!("Invalid native v9 document: {e}"))?;
                (file.paragraphs, file.style_catalog, file.character_styles)
            };
            let document = decode_character_paragraphs(paragraphs, catalog, character_styles)?;
            document.validate()?;
            return Ok(document);
        }
        2 => {
            serde_json::from_slice::<Native<Run>>(bytes)
                .map_err(|e| format!("Invalid native v2 document: {e}"))?
                .paragraphs
        }
        _ => return Err("Unsupported document content version; source was not changed.".into()),
    };
    let mut paragraphs = Vec::new();
    let mut total = native_paragraphs.len().saturating_sub(1);
    for p in native_paragraphs {
        let mut text = String::new();
        let mut styles = Vec::new();
        for run in p.runs {
            if run.text.is_empty() {
                return Err("Native runs must be nonempty.".into());
            }
            let count = run.text.chars().count();
            total += count;
            if total > MAX_CHARACTERS {
                return Err("Document exceeds 100,000 characters.".into());
            }
            styles.extend(std::iter::repeat_n(
                CharacterFormat::explicit(Style {
                    bold: run.bold,
                    italic: run.italic,
                    underline: run.underline,
                    font_size: run.font_size_half_points,
                }),
                count,
            ));
            text.push_str(&run.text);
        }
        paragraphs.push(Paragraph {
            text,
            styles,
            alignment: p.alignment,
            paragraph_style: ParagraphStyle::Normal,
            spacing: ParagraphSpacing::default(),
            ..Default::default()
        });
    }
    let result = Document {
        paragraphs,
        style_catalog: StyleCatalog::default(),
        character_styles: Vec::new(),
        ..Default::default()
    };
    result.validate()?;
    Ok(result)
}

fn native_path(path: &Path) -> Result<()> {
    if !path
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x.eq_ignore_ascii_case("bfdoc"))
    {
        return Err("Only .bfdoc native files are supported.".into());
    }
    Ok(())
}
pub fn load(path: &Path) -> Result<Document> {
    native_path(path)?;
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| format!("Cannot open {}: {e}", path.display()))?
        .take((MAX_FILE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Cannot read file: {e}"))?;
    decode(&bytes)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveStage {
    Write,
    Sync,
    Replace,
}

pub fn save(document: &Document, path: &Path) -> Result<()> {
    save_checked(document, path, |_| Ok(()))
}

/// Fault hook exercises the real transaction. No original file is opened for writing.
pub fn save_checked(
    document: &Document,
    path: &Path,
    check: impl FnMut(SaveStage) -> Result<()>,
) -> Result<()> {
    native_path(path)?;
    let data = encode(document)?;
    write_bytes_checked(path, &data, check)
}

/// Shared checked replacement for native documents and generated output.
pub fn write_bytes_checked(
    path: &Path,
    data: &[u8],
    mut check: impl FnMut(SaveStage) -> Result<()>,
) -> Result<()> {
    if fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err("Choose a destination that is not a symbolic link.".into());
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temp = tempfile::Builder::new()
        .prefix(".breakfree-docs-")
        .suffix(".tmp")
        .tempfile_in(parent)
        .map_err(|e| format!("Cannot prepare save: {e}"))?;
    // A partial temporary write is deliberate so a failed-write test reaches disk.
    let half = data.len() / 2;
    temp.write_all(&data[..half])
        .map_err(|e| format!("Cannot write file: {e}"))?;
    check(SaveStage::Write)?;
    temp.write_all(&data[half..])
        .and_then(|_| temp.flush())
        .map_err(|e| format!("Cannot write file: {e}"))?;
    check(SaveStage::Sync)?;
    temp.as_file()
        .sync_all()
        .map_err(|e| format!("Cannot flush file: {e}"))?;
    if let Ok(metadata) = fs::metadata(path) {
        temp.as_file()
            .set_permissions(metadata.permissions())
            .map_err(|e| format!("Cannot preserve permissions: {e}"))?;
    }
    check(SaveStage::Replace)?;
    temp.persist(path)
        .map_err(|e| format!("Cannot replace destination: {}", e.error))?;
    Ok(())
}
