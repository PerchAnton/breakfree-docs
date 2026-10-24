//! Explicit tab stops retain replace-list semantics through OOXML clear entries.
use super::*;
fn leader(l: TabLeader) -> &'static str {
    match l {
        TabLeader::Dots => "dot",
        TabLeader::Dashes => "hyphen",
        TabLeader::Line => "underscore",
    }
}
pub(super) fn write(p: &ParagraphSpacing, parent: &ParagraphSpacing) -> String {
    if !p.has_direct_tab_stops() {
        return String::new();
    }
    let stops = p.style_tab_stops();
    let mut entries = std::collections::BTreeMap::new();
    for (pos, _) in parent.tab_positions() {
        if !stops.iter().any(|s| s.position_half_points == pos) {
            entries.insert(
                pos,
                format!("<w:tab w:val=\"clear\" w:pos=\"{}\"/>", pos * 10),
            );
        }
    }
    for s in stops {
        let kind = match s.kind {
            TabKind::Left => "left",
            TabKind::Center => "center",
            TabKind::Right => "right",
        };
        let fill = s
            .leader
            .map(|l| format!(" w:leader=\"{}\"", leader(l)))
            .unwrap_or_default();
        entries.insert(
            s.position_half_points,
            format!(
                "<w:tab w:val=\"{kind}\" w:pos=\"{}\"{fill}/>",
                s.position_half_points * 10
            ),
        );
    }
    format!(
        "<w:tabs>{}</w:tabs>",
        entries.values().cloned().collect::<String>()
    )
}
pub(super) fn read(n: &Node, p: &mut ParagraphSpacing) -> Result<()> {
    let Some(t) = n.child("tabs") else {
        return Ok(());
    };
    t.allowed(W, &["tab"], &[])?;
    if t.children.len() > 64 {
        return Err("Too many tab records".into());
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut stops = Vec::new();
    let mut leaders = Vec::new();
    for s in &t.children {
        s.allowed(W, &[], &[(W, "val"), (W, "pos"), (W, "leader")])?;
        let pos = properties::unit(s, "pos", 10)?;
        if !seen.insert(pos) {
            return Err("Duplicate tab position".into());
        }
        let kind = match s.val()? {
            "left" => TabKind::Left,
            "center" => TabKind::Center,
            "right" => TabKind::Right,
            "clear" => {
                if s.attr(W, "leader").is_some() {
                    return Err("Leader on cleared tab".into());
                }
                continue;
            }
            _ => return Err("Unsupported tab alignment".into()),
        };
        stops.push((pos, kind));
        if let Some(fill) = s.attr(W, "leader") {
            let l = match fill {
                "none" => continue,
                "dot" => TabLeader::Dots,
                "hyphen" => TabLeader::Dashes,
                "underscore" => TabLeader::Line,
                _ => return Err("Unsupported tab leader".into()),
            };
            leaders.push(TabLeaderStop {
                position_half_points: pos,
                leader: l,
            });
        }
    }
    stops.sort_by_key(|s| s.0);
    leaders.sort_by_key(|s| s.position_half_points);
    p.set_tab_positions(&stops)?;
    p.tab_leaders = leaders;
    p.validate()
}
pub(super) fn validate_clears(
    n: &Node,
    parent: &ParagraphSpacing,
    p: &ParagraphSpacing,
) -> Result<()> {
    let Some(t) = n.child("tabs") else {
        return Ok(());
    };
    let actual = t
        .children
        .iter()
        .filter(|s| s.attr(W, "val") == Some("clear"))
        .map(|s| properties::unit(s, "pos", 10))
        .collect::<Result<std::collections::BTreeSet<_>>>()?;
    let own = p.tab_positions();
    let expected = parent
        .tab_positions()
        .iter()
        .filter(|(pos, _)| !own.iter().any(|(x, _)| x == pos))
        .map(|(p, _)| *p)
        .collect();
    if actual != expected {
        return Err(
            "DOCX additive/partial tab inheritance cannot preserve replace-list semantics".into(),
        );
    }
    Ok(())
}
