//! Page placement of measured rows. All dimensions are document points (1/72 inch).
use crate::{
    Result,
    model::Document,
    structure::{Block, Section},
};
#[derive(Clone, Debug)]
pub struct Measure {
    pub rows: Vec<f32>,
    pub before: f32,
    pub after: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    pub paragraph: usize,
    pub row: usize,
    pub y: f32,
    pub repeated_header: bool,
}
#[derive(Clone, Debug)]
pub struct Page {
    pub settings: Section,
    pub number: usize,
    pub items: Vec<Placement>,
}
#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub pages: Vec<Page>,
}
pub fn sections(document: &Document) -> Vec<Section> {
    let mut current = document.page.clone();
    document
        .paragraphs
        .iter()
        .map(|p| {
            if let Some(s) = &p.structure.section {
                current = s.clone();
            }
            current.clone()
        })
        .collect()
}
fn next_page(plan: &mut Plan, settings: &Section, restart: bool) -> Result<()> {
    if plan.pages.len() >= 500 {
        return Err("Document exceeds the 500-page layout limit.".into());
    }
    let number = if restart {
        settings
            .number_start
            .map(usize::from)
            .unwrap_or_else(|| plan.pages.last().map_or(1, |p| p.number + 1))
    } else {
        plan.pages.last().map_or(1, |p| p.number + 1)
    };
    plan.pages.push(Page {
        settings: settings.clone(),
        number,
        items: Vec::new(),
    });
    Ok(())
}
pub fn paginate(document: &Document, measures: &[Measure]) -> Result<Plan> {
    document.validate()?;
    if document.paragraphs.len() != measures.len() {
        return Err("Measurements do not match the document.".into());
    }
    let settings = sections(document);
    let mut plan = Plan::default();
    let mut used = 0.;
    for (index, (p, m)) in document.paragraphs.iter().zip(measures).enumerate() {
        let section = &settings[index];
        let capacity = section.text_height();
        if m.rows.is_empty()
            || m.rows
                .iter()
                .any(|h| !h.is_finite() || *h <= 0. || *h > capacity)
            || !m.before.is_finite()
            || !m.after.is_finite()
            || m.before < 0.
            || m.after < 0.
        {
            return Err(format!(
                "Paragraph {} has an object or line too tall for the page. Reduce its size or margins.",
                index + 1
            ));
        }
        let section_start = p.structure.section.is_some();
        if plan.pages.is_empty()
            || (section_start || p.structure.page_break_before)
                && !plan.pages.last().unwrap().items.is_empty()
        {
            next_page(&mut plan, section, section_start || index == 0)?;
            used = 0.;
        } else if section_start {
            let page = plan.pages.last_mut().unwrap();
            page.settings = section.clone();
            if let Some(n) = section.number_start {
                page.number = usize::from(n);
            }
        }
        let whole = m.rows.iter().sum::<f32>();
        if p.structure.keep_lines && whole > capacity {
            return Err(format!(
                "Paragraph {} cannot fit on one page with Keep lines together enabled.",
                index + 1
            ));
        }
        let mut min_next = 0.;
        if p.structure.keep_with_next {
            for (next_index, next) in measures.iter().enumerate().skip(index + 1) {
                let next_p = &document.paragraphs[next_index];
                if next_p.structure.page_break_before || next_p.structure.section.is_some() {
                    break;
                }
                min_next += next.before;
                if next_p.structure.keep_with_next || next_p.structure.keep_lines {
                    min_next += next.rows.iter().sum::<f32>() + next.after;
                } else {
                    min_next += next.rows.iter().take(2).sum::<f32>();
                }
                if !next_p.structure.keep_with_next {
                    break;
                }
            }
        }
        let required = if p.structure.keep_lines || p.structure.keep_with_next {
            whole + m.after + min_next
        } else {
            m.rows.iter().take(2).sum()
        };
        if used > 0. && required <= capacity && used + m.before + required > capacity {
            next_page(&mut plan, section, false)?;
            used = 0.;
        }
        let repeated =
            matches!(&p.structure.block,Some(Block::Table(t)) if t.repeat_header && t.rows.len()>1);
        let mut row = 0;
        while row < m.rows.len() {
            let gap = if row == 0 {
                m.before.min((capacity - m.rows[row]).max(0.))
            } else {
                0.
            };
            if used + gap + m.rows[row] > capacity + 0.001 {
                next_page(&mut plan, section, false)?;
                used = 0.;
                if repeated && row > 0 {
                    if m.rows[0] + m.rows[row] > capacity {
                        return Err(format!(
                            "Table {} header and row do not fit together on a page.",
                            index + 1
                        ));
                    }
                    plan.pages.last_mut().unwrap().items.push(Placement {
                        paragraph: index,
                        row: 0,
                        y: used,
                        repeated_header: true,
                    });
                    used += m.rows[0];
                }
            }
            if row == 0 {
                used += gap;
            }
            let mut count = 0;
            let mut h = 0.;
            for line in &m.rows[row..] {
                if used + h + line > capacity + 0.001 {
                    break;
                }
                h += line;
                count += 1;
            }
            if count == 0 {
                return Err("No layout progress; page is too small.".into());
            }
            let remaining = m.rows.len() - row;
            // Avoid a single orphan/widow where the physical page can hold two.
            if p.structure.block.is_none() && remaining > 2 && count == remaining - 1 && count >= 2
            {
                count -= 1;
            }
            if count == 1
                && row == 0
                && remaining > 1
                && used > gap
                && m.rows[0] + m.rows[1] <= capacity
            {
                next_page(&mut plan, section, false)?;
                used = 0.;
                continue;
            }
            for _ in 0..count {
                plan.pages.last_mut().unwrap().items.push(Placement {
                    paragraph: index,
                    row,
                    y: used,
                    repeated_header: false,
                });
                used += m.rows[row];
                row += 1;
            }
        }
        used = (used + m.after).min(capacity);
    }
    Ok(plan)
}
