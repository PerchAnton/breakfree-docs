use crate::{Result, commands::Editor, model::Document, storage};
use std::path::{Path, PathBuf};

pub struct Session {
    pub editor: Editor,
    pub path: Option<PathBuf>,
    saved: Document,
    docx_source: Option<(PathBuf, Vec<u8>)>,
}
impl Default for Session {
    fn default() -> Self {
        Self::new(Document::default(), None)
    }
}
impl Session {
    /// Open the primary DOCX format or the optional full-model native format.
    pub fn open(path: &Path) -> Result<Self> {
        Self::check_format(path)?;
        if path
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("docx"))
        {
            let bytes = crate::docx::load_bytes(path)?;
            let mut session = Self::new(crate::docx::decode(&bytes)?, Some(path.into()));
            session.docx_source = Some((path.into(), bytes));
            Ok(session)
        } else {
            Ok(Self::new(storage::load(path)?, Some(path.to_path_buf())))
        }
    }

    pub fn new(document: Document, path: Option<PathBuf>) -> Self {
        Self {
            saved: document.clone(),
            editor: Editor::new(document),
            path,
            docx_source: None,
        }
    }
    pub fn dirty(&self) -> bool {
        self.saved != self.editor.document
    }
    pub fn is_docx(&self) -> bool {
        self.path.as_deref().is_some_and(Self::docx_path)
    }
    fn docx_path(path: &Path) -> bool {
        path.extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("docx"))
    }
    fn check_format(path: &Path) -> Result<()> {
        if Self::docx_path(path)
            || path
                .extension()
                .is_some_and(|s| s.eq_ignore_ascii_case("bfdoc"))
        {
            Ok(())
        } else {
            Err("Choose .docx (primary) or .bfdoc (optional). Binary .doc and macro-enabled .docm are unsupported.".into())
        }
    }
    pub fn save(&mut self, path: &Path) -> Result<()> {
        Self::check_format(path)?;
        if Self::docx_path(path) {
            let bytes = crate::docx::encode(&self.editor.document)?;
            if let Some((source, original)) = &self.docx_source
                && (source == path
                    || std::fs::canonicalize(source)
                        .ok()
                        .zip(std::fs::canonicalize(path).ok())
                        .is_some_and(|(a, b)| a == b))
                && crate::docx::load_bytes(path)? != *original
            {
                return Err(
                    "The DOCX changed outside this window. Use Save As to keep both versions."
                        .into(),
                );
            }
            storage::write_bytes_checked(path, &bytes, |_| Ok(()))?;
            self.docx_source = Some((path.into(), bytes));
        } else {
            storage::save(&self.editor.document, path)?;
            self.docx_source = None;
        }
        self.path = Some(path.into());
        self.saved = self.editor.document.clone();
        Ok(())
    }
    pub fn can_leave(&mut self, choice: LeaveChoice, save_path: Option<&Path>) -> Result<bool> {
        if !self.dirty() {
            return Ok(true);
        }
        match choice {
            LeaveChoice::Cancel => Ok(false),
            LeaveChoice::Discard => Ok(true),
            LeaveChoice::Save => {
                let path = save_path
                    .map(Path::to_path_buf)
                    .or_else(|| self.path.clone())
                    .ok_or("Choose a save location.")?;
                self.save(&path)?;
                Ok(true)
            }
        }
    }
}
#[derive(Clone, Copy)]
pub enum LeaveChoice {
    Save,
    Discard,
    Cancel,
}
