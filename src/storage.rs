//! Strict .bfdoc v1 compatibility and checked temporary-file replacement.
use crate::{Result, model::*};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

pub const FORMAT_ID: &str = "breakfree.docs.native";
pub const MAX_FILE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Native {
    format: String,
    content_version: u32,
    paragraphs: Vec<NativeParagraph>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeParagraph {
    alignment: Alignment,
    runs: Vec<Run>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Run {
    text: String,
    bold: bool,
    italic: bool,
}

pub fn encode(document: &Document) -> Result<Vec<u8>> {
    document.validate()?;
    let paragraphs = document
        .paragraphs
        .iter()
        .map(|p| {
            let mut runs: Vec<Run> = Vec::new();
            for (ch, style) in p.text.chars().zip(&p.styles) {
                if let Some(last) = runs
                    .last_mut()
                    .filter(|r| r.bold == style.bold && r.italic == style.italic)
                {
                    last.text.push(ch);
                } else {
                    runs.push(Run {
                        text: ch.to_string(),
                        bold: style.bold,
                        italic: style.italic,
                    });
                }
            }
            NativeParagraph {
                alignment: p.alignment,
                runs,
            }
        })
        .collect();
    let mut bytes = serde_json::to_vec_pretty(&Native {
        format: FORMAT_ID.into(),
        content_version: 1,
        paragraphs,
    })
    .map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    if bytes.len() > MAX_FILE_BYTES {
        return Err("File exceeds 16 MiB.".into());
    }
    Ok(bytes)
}

pub fn decode(bytes: &[u8]) -> Result<Document> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err("File exceeds 16 MiB.".into());
    }
    let file: Native =
        serde_json::from_slice(bytes).map_err(|e| format!("Invalid native document: {e}"))?;
    if file.format != FORMAT_ID {
        return Err("Not a BreakFree Docs native document.".into());
    }
    if file.content_version != 1 {
        return Err("Unsupported document content version; source was not changed.".into());
    }
    let mut paragraphs = Vec::new();
    let mut total = file.paragraphs.len().saturating_sub(1);
    for p in file.paragraphs {
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
                Style {
                    bold: run.bold,
                    italic: run.italic,
                },
                count,
            ));
            text.push_str(&run.text);
        }
        paragraphs.push(Paragraph {
            text,
            styles,
            alignment: p.alignment,
        });
    }
    let result = Document { paragraphs };
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
