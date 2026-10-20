# ADR 0014 — Stored left tab grid and native text layout

- Date: 2026-10-20
- Status: implemented prototype; advanced tab types remain future work

Store literal U+0009 in text runs, preserving character formatting and scalar
indices. Paragraph spacing optionally stores `tab_interval_half_points` (1–288);
null/absence selects 72 half points (36 pt). This direct property does not inherit
from named styles in this iteration. The shared `set_tab_interval` command applies
one undoable half-open paragraph selection change; replacement/Enter copies the
existing geometry policy. Native v14 writes it; v1–v13 import with the default and
reject the new key even when null. Existing readers reject v14 explicitly.

Layout uses left stops on an interval grid measured from the paragraph's
unindented text-column origin. First-line/hanging offsets determine the starting
position, so a Tab advances to the next strictly greater stop. Zoom scales the
grid with text. At the column edge a Tab consumes the remaining line width;
following text wraps and uses that row's grid. Repeated/trailing tabs retain one
stored scalar and one cursor advance each. Oversized grapheme clusters remain
whole, consistent with the existing continuous canvas policy.

The tab adapter shapes original text fragments; it never stores spaces in place
of tabs. It preserves mixed-format glyphs and synthesizes the tab underline over
its actual advance. Rows with tabs keep fixed spacing: Justify currently uses
left placement for those paragraphs; Center/Right align the resulting row as a
block. Paragraphs without tabs keep the existing layout path. This provisional
policy is not Word layout equivalence. Custom tab positions, right/center/decimal
stops, leaders, style inheritance and interactive ruler editing are not supported.

Plain Tab in the focused editor inserts a literal Tab through the existing shared
text-edit transaction. Modified Tab is reserved and does not delete selections;
F6/Alt routes remain available for leaving the editor. Alpha.62 exposes the validated interval command through a native draft dialog.
Each acceptance applies to the half-open paragraph selection in one history entry;
Cancel/Escape and invalid input do not mutate it. The parent paragraph draft stays
open and unchanged while the interval dialog runs as a separate transaction.
Absent overrides use 36 pt.
No ribbon/palette/icon change, dependency or license change is required.

## Alpha.63: explicit left positions

Paragraph spacing now owns `left_tab_stops_half_points`, a sorted unique vector
with at most 32 entries, each 1–2880 (0.5–1440 pt). Empty means interval grid only.
These bounded direct paragraph settings are independent of named styles; retain
positions outside the current visible column when resizing. The shared
`set_left_tab_stops` command rejects invalid order/duplicates/ranges, uses half-open
paragraph selection and commits one undoable snapshot. No-op preserves Redo.
Enter, joins, paste and Replace All use the existing paragraph-geometry policy.

A Tab uses the first explicit stop strictly greater than the current coordinate,
measured from the unindented text-column origin. Explicit stops suppress the
regular interval grid before the last stop. After the last stop, resume the next
regular grid multiple from the same origin (not last-stop plus interval). At or
beyond the line edge, preserve the existing edge/wrap policy without altering stored
positions. First-line/hanging indents, continuation rows and zoom use the same
coordinates. This is a prototype policy, not a claim of Word layout equivalence.

Native v15 stores the list in paragraph spacing. Absence imports an empty list;
null is invalid. v1–v14 import without positions and reject this new key even when
empty/null. Readers from alpha.62 and earlier reject v15. The application and
process protocol versions remain independent. No dependency/license change.

The interval dialog remains available and changes only the interval, preserving
explicit stops. Editing positions through the GUI/ruler is the next increment;
right/center/decimal stops, leaders and style inheritance remain future work.
