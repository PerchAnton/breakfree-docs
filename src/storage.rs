//! Strict .bfdoc v1/v2/v3/v4 reading, v4 writing and checked temporary-file replacement.
use crate::{Result, model::*};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

pub const FORMAT_ID: &str = "breakfree.docs.native";
pub const CONTENT_VERSION: u32 = 4;
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

pub fn encode(document: &Document) -> Result<Vec<u8>> {
    document.validate()?;
    let paragraphs = document
        .paragraphs
        .iter()
        .map(|p| {
            let mut runs: Vec<RunV3> = Vec::new();
            for (ch, &direct) in p.text.chars().zip(&p.styles) {
                if let Some(last) = runs.last_mut().filter(|r| r.direct == direct) {
                    last.text.push(ch);
                } else {
                    runs.push(RunV3 {
                        text: ch.to_string(),
                        direct,
                    });
                }
            }
            ParagraphV3 {
                alignment: p.alignment,
                paragraph_style: p.paragraph_style,
                runs,
            }
        })
        .collect();
    let mut bytes = serde_json::to_vec_pretty(&NativeV4 {
        format: FORMAT_ID.into(),
        content_version: CONTENT_VERSION,
        style_catalog: document.style_catalog.clone(),
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
    };
    result.validate()?;
    Ok(result)
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
        CONTENT_VERSION => {
            let file: NativeV4 = serde_json::from_slice(bytes)
                .map_err(|e| format!("Invalid native v4 document: {e}"))?;
            return decode_styled(file.paragraphs, file.style_catalog);
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
        });
    }
    let result = Document {
        paragraphs,
        style_catalog: StyleCatalog::default(),
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
    mut check: impl FnMut(SaveStage) -> Result<()>,
) -> Result<()> {
    native_path(path)?;
    let data = encode(document)?;
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
