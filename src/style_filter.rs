//! Transient style-list filtering; never part of the document or undo history.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scope {
    #[default]
    All,
    Paragraph,
    Character,
    Custom,
}
impl Scope {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All styles",
            Self::Paragraph => "Paragraph styles",
            Self::Character => "Character styles",
            Self::Custom => "Custom styles",
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Paragraph,
            Self::Paragraph => Self::Character,
            Self::Character => Self::Custom,
            Self::Custom => Self::All,
        }
    }
    /// Unicode lowercase substring matching, without normalization or full case folding.
    pub fn matches(self, query: &str, name: &str, character: bool, custom: bool) -> bool {
        let kind = match self {
            Self::All => true,
            Self::Paragraph => !character,
            Self::Character => character,
            Self::Custom => custom,
        };
        kind && name.to_lowercase().contains(&query.trim().to_lowercase())
    }
}
