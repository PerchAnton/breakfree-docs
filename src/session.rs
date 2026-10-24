use crate::{Result, commands::Editor, model::Document, storage};
use std::path::{Path, PathBuf};

pub struct Session {
    pub editor: Editor,
    pub path: Option<PathBuf>,
    saved: Document,
    pub imported_docx: bool,
}
impl Default for Session {
    fn default() -> Self {
        Self::new(Document::default(), None)
    }
}
impl Session {
    /// DOCX imports have no native save destination; Save must choose a .bfdoc.
    pub fn open(path: &Path) -> Result<Self> {
        if path
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("docx"))
        {
            let mut session = Self::new(crate::docx::load(path)?, None);
            session.imported_docx = true;
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
            imported_docx: false,
        }
    }
    pub fn dirty(&self) -> bool {
        self.saved != self.editor.document
    }
    pub fn save(&mut self, path: &Path) -> Result<()> {
        storage::save(&self.editor.document, path)?;
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
