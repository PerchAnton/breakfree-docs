//! Read-only authored tabulation provenance. No UI or history dependencies.
use crate::{Result, formatting::PropertyInfo, model::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    Default,
    Direct,
    Style(ParagraphStyle, bool),
}
impl Source {
    pub fn label(&self, document: &Document) -> String {
        match self {
            Self::Default => "Document default".into(),
            Self::Direct => "Direct paragraph formatting".into(),
            Self::Style(id, inherited) => format!(
                "{}: {}",
                if *inherited {
                    "Inherited paragraph style"
                } else {
                    "Paragraph style"
                },
                document.style_catalog.name(*id)
            ),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub value: String,
    pub source: Source,
    pub label: String,
    pub count: usize,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Property {
    pub groups: Vec<Group>,
}
impl Property {
    fn add(&mut self, document: &Document, source: Source, value: String) {
        if let Some(group) = self
            .groups
            .iter_mut()
            .find(|g| g.source == source && g.value == value)
        {
            group.count += 1;
        } else {
            self.groups.push(Group {
                label: source.label(document),
                value,
                source,
                count: 1,
            });
        }
    }
    pub fn row(&self, name: &'static str) -> PropertyInfo {
        let first = &self.groups[0];
        PropertyInfo {
            name,
            value: if self.groups.iter().all(|g| g.value == first.value) {
                first.value.clone()
            } else {
                "Mixed".into()
            },
            source: if self.groups.iter().all(|g| g.source == first.source) {
                first.label.clone()
            } else {
                "Mixed sources".into()
            },
        }
    }
    /// Bound hover verbosity; the explicit inspector retains the complete list.
    pub fn summary(&self) -> String {
        let mut lines: Vec<_> = self
            .groups
            .iter()
            .take(3)
            .map(|g| format!("{} paragraph(s) — {}", g.count, g.label))
            .collect();
        if self.groups.len() > 3 {
            lines.push(format!(
                "{} more groups; open Sources for details.",
                self.groups.len() - 3
            ));
        }
        lines.join("\n")
    }
    pub fn description(&self) -> String {
        self.groups
            .iter()
            .map(|g| format!("{} paragraph(s) — {}: {}", g.count, g.label, g.value))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inspection {
    pub count: usize,
    pub interval: Property,
    pub positions: Property,
    pub absent: usize,
}
impl Inspection {
    pub fn description(&self) -> String {
        format!(
            "Tab interval\n{}\nTab positions\n{}{}",
            self.interval.description(),
            self.positions.description(),
            if self.absent > 0 {
                format!("\nStop absent in {} paragraph(s).", self.absent)
            } else {
                String::new()
            }
        )
    }
    pub fn rows(&self) -> Vec<PropertyInfo> {
        if self.count == 0 {
            return Vec::new();
        }
        vec![
            self.interval.row("Tab interval"),
            self.positions.row("Tab positions"),
        ]
    }
}
/// Resolve each property independently, preserving authored equal-looking overrides.
pub fn sources(document: &Document, paragraph: &Paragraph) -> [Source; 2] {
    let mut result = [Source::Default, Source::Default];
    let mut found = [
        paragraph.spacing.tab_interval_half_points.is_some(),
        paragraph.spacing.has_direct_tab_stops(),
    ];
    for i in 0..2 {
        if found[i] {
            result[i] = Source::Direct;
        }
    }
    let mut next = Some(paragraph.paragraph_style);
    let mut seen = Vec::new();
    while let Some(id) = next {
        if seen.contains(&id) {
            break;
        }
        seen.push(id);
        let Some(definition) = document.style_catalog.get(id) else {
            break;
        };
        for (i, supplied) in [
            definition.tab_interval_half_points.is_some(),
            definition.tab_stops.is_some(),
        ]
        .into_iter()
        .enumerate()
        {
            if !found[i] && supplied {
                result[i] = Source::Style(id, id != paragraph.paragraph_style);
                found[i] = true;
            }
        }
        next = definition.parent;
    }
    result
}
/// Half-open paragraph selection. A stop filter restricts counts to actual owners.
pub fn inspect(
    document: &Document,
    selection: (usize, usize),
    stop: Option<u16>,
) -> Result<Inspection> {
    let (start, end) = selection;
    if start > end {
        return Err("Invalid tabulation selection.".into());
    }
    let (first, _) = document.locate(start)?;
    let (mut last, offset) = document.locate(end)?;
    if end > start && offset == 0 && last > first {
        last -= 1;
    }
    let mut report = Inspection::default();
    for paragraph in &document.paragraphs[first..=last] {
        include(&mut report, document, paragraph, stop);
    }
    Ok(report)
}
fn include(report: &mut Inspection, document: &Document, paragraph: &Paragraph, stop: Option<u16>) {
    let tabs = document.resolved_tabs(paragraph);
    let positions = tabs.tab_positions();
    if stop.is_some_and(|s| !positions.iter().any(|p| p.0 == s)) {
        report.absent += 1;
        return;
    }
    report.count += 1;
    let [interval_source, list_source] = sources(document, paragraph);
    report.interval.add(
        document,
        interval_source,
        format!(
            "{} pt",
            f32::from(tabs.tab_interval_half_points.unwrap_or(72)) / 2.
        ),
    );
    let value = if positions.is_empty() {
        "No explicit positions (interval grid only)".into()
    } else {
        positions
            .iter()
            .filter(|p| stop.is_none_or(|s| s == p.0))
            .map(|(position, kind)| {
                format!(
                    "{} pt · {} · {}",
                    f32::from(*position) / 2.,
                    kind.label(),
                    tabs.tab_leader(*position).map_or("None", |v| v.label())
                )
            })
            .collect::<Vec<_>>()
            .join("; ")
    };
    report.positions.add(document, list_source, value);
}
pub fn paragraph(document: &Document, paragraph: &Paragraph) -> Inspection {
    let mut report = Inspection::default();
    include(&mut report, document, paragraph, None);
    report
}
/// Style drafts have no paragraph-level overrides; creation uses its provisional ID.
pub fn style(document: &Document, id: ParagraphStyle) -> Inspection {
    paragraph(
        document,
        &Paragraph {
            paragraph_style: id,
            ..Default::default()
        },
    )
}
