//! Scoped save-back probe using the same session and edit command as the GUI.
use breakfree_docs::{docx, session::Session};
use std::path::PathBuf;
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("Usage: docx_saveback <source.docx> <new-output.docx>".into());
    }
    let source = PathBuf::from(&args[0]);
    let output = PathBuf::from(&args[1]);
    if output.exists() {
        return Err("Probe output must not exist".into());
    }
    let original_bytes = std::fs::read(&source).map_err(|e| e.to_string())?;
    let mut session = Session::open(&source)?;
    session.editor.replace(0, 0, "Edited · ", "Corpus edit")?;
    session.save(&output)?;
    if docx::load(&output)? != session.editor.document {
        return Err("Save-back model mismatch".into());
    }
    if std::fs::read(&source).map_err(|e| e.to_string())? != original_bytes {
        return Err("Source changed".into());
    }
    println!("Exact model save/reopen and unchanged source verified.");
    Ok(())
}
