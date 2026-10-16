use crate::{Result, commands::Editor, model::Document, storage};
use std::path::{Path, PathBuf};

pub struct Session {
    pub editor: Editor,
    pub path: Option<PathBuf>,
    saved: Document,
}
impl Default for Session {
    fn default() -> Self {
        Self::new(Document::default(), None)
    }
}
impl Session {
    pub fn new(document: Document, path: Option<PathBuf>) -> Self {
        Self {
            saved: document.clone(),
            editor: Editor::new(document),
            path,
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
