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

/// Semantic usage in persisted document content, including parent dependencies.
/// Pending typing and following-style links are not actual use in the document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UsedStyles {
    pub paragraphs: Vec<crate::model::ParagraphStyle>,
    pub characters: Vec<crate::model::CharacterStyle>,
    pub default_character: bool,
    pub direct_paragraphs: Vec<crate::model::ParagraphStyle>,
    pub direct_characters: Vec<crate::model::CharacterStyle>,
}
impl UsedStyles {
    pub fn collect(document: &crate::model::Document) -> Self {
        use crate::model::CharacterStyle;
        let mut used = Self::default();
        for paragraph in &document.paragraphs {
            if !used.direct_paragraphs.contains(&paragraph.paragraph_style) {
                used.direct_paragraphs.push(paragraph.paragraph_style);
            }
            let mut next = Some(paragraph.paragraph_style);
            while let Some(id) = next {
                if used.paragraphs.contains(&id) {
                    break;
                }
                used.paragraphs.push(id);
                next = document.style_catalog.get(id).and_then(|s| s.parent);
            }
            if paragraph.styles.is_empty() {
                used.default_character = true;
            }
            for format in &paragraph.styles {
                if format.character_style.is_none() {
                    used.default_character = true;
                }
                if let Some(id) = format.character_style
                    && !used.direct_characters.contains(&id)
                {
                    used.direct_characters.push(id);
                }
                let mut next = format.character_style;
                while let Some(id) = next {
                    if used.characters.contains(&id) {
                        break;
                    }
                    used.characters.push(id);
                    next = match id {
                        CharacterStyle::Custom(key) => document
                            .character_styles
                            .iter()
                            .find(|s| s.id == key)
                            .and_then(|s| s.parent),
                        _ => None,
                    };
                }
            }
        }
        used
    }
    pub fn character(&self, id: Option<crate::model::CharacterStyle>) -> bool {
        id.map_or(self.default_character, |id| self.characters.contains(&id))
    }
}

/// Direct assignment takes precedence when a style is also another style's parent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Usage {
    Direct,
    Parent,
    Unused,
}
impl Usage {
    pub fn label(self) -> &'static str {
        match self {
            Self::Direct => "Direct use",
            Self::Parent => "Parent dependency",
            Self::Unused => "Not used",
        }
    }
}
impl UsedStyles {
    pub fn paragraph_usage(&self, id: crate::model::ParagraphStyle) -> Usage {
        if self.direct_paragraphs.contains(&id) {
            Usage::Direct
        } else if self.paragraphs.contains(&id) {
            Usage::Parent
        } else {
            Usage::Unused
        }
    }
    pub fn character_usage(&self, id: Option<crate::model::CharacterStyle>) -> Usage {
        if id.map_or(self.default_character, |id| {
            self.direct_characters.contains(&id)
        }) {
            Usage::Direct
        } else if self.character(id) {
            Usage::Parent
        } else {
            Usage::Unused
        }
    }
}

// Walk each directly assigned style independently: shared ancestors must retain
// every consumer, and invalid external cycles must not hang the inspector.
fn depends_on<T: Copy + PartialEq>(source: T, target: T, parent: impl Fn(T) -> Option<T>) -> bool {
    if source == target {
        return false;
    }
    let mut seen = vec![source];
    let mut next = parent(source);
    while let Some(id) = next {
        if seen.contains(&id) {
            break;
        }
        if id == target {
            return true;
        }
        seen.push(id);
        next = parent(id);
    }
    false
}
impl UsedStyles {
    /// Unique directly assigned consumers, in document encounter order.
    pub fn paragraph_consumers(
        &self,
        document: &crate::model::Document,
        parent: crate::model::ParagraphStyle,
    ) -> Vec<crate::model::ParagraphStyle> {
        self.direct_paragraphs
            .iter()
            .copied()
            .filter(|&id| {
                depends_on(id, parent, |id| {
                    document.style_catalog.get(id).and_then(|s| s.parent)
                })
            })
            .collect()
    }
    pub fn character_consumers(
        &self,
        document: &crate::model::Document,
        parent: crate::model::CharacterStyle,
    ) -> Vec<crate::model::CharacterStyle> {
        self.direct_characters
            .iter()
            .copied()
            .filter(|&id| {
                depends_on(id, parent, |id| match id {
                    crate::model::CharacterStyle::Custom(key) => document
                        .character_styles
                        .iter()
                        .find(|s| s.id == key)
                        .and_then(|s| s.parent),
                    _ => None,
                })
            })
            .collect()
    }
}
