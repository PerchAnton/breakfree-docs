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

## Alpha.64: transactional position editing

Home → Paragraph spacing → Tab positions and command search open a separate
position-list draft. Add inserts in sorted order; Update replaces the selected
position; Remove and Clear all edit only the draft. Duplicates, more than 32
positions and non-half-point/out-of-range input fail without editing the list.
OK submits the full list through `set_left_tab_stops`, preserving one history
entry across all selected paragraphs. Cancel/Escape do not mutate the document.

Common lists initialize the draft; mixed lists show a notice and start an empty
replacement with OK disabled until a list action. Clear all explicitly enables
an empty replacement. Pending typed input must be added/updated or cleared before
OK. No-op acceptance preserves Redo. The independent paragraph draft survives
nested opening/cancellation; interval values stay unchanged. Native v15 and
layout semantics remain unchanged. Interactive ruler editing is a later increment.

## Alpha.65: ruler drag prototype

A toolkit-independent `tab_ruler::Draft` snapshots the document and half-open
selection. It requires identical explicit lists in the selected paragraphs and an
existing source position. Movement rounds to half points, clamps to 0.5–1440 pt,
reorders the list and changes only a preview document. A duplicate destination
invalidates the draft; release cancels rather than merging positions. Commit checks
both source document and selection, then invokes the existing list command once.
Discarding a draft and no-op acceptance preserve document/history/Redo.

The native ruler paints small left-stop L markers in existing ruler coordinates;
tab geometry uses the unindented column scale, not the indent-compression scale.
Only visible positions are painted. Indent hit targets keep priority; overlapping
tab targets are read-only. Mixed lists show first-paragraph markers as read-only.
Use Tab positions to edit mixed lists, overlaps and off-column positions.

Pointer dragging temporarily locks document input and commands. Escape, pointer
loss, window-focus loss, close request, resize/zoom or release outside the ruler
(with a 12 DIP tolerance) discards the draft. Preview includes a contextual exact
value; the canvas renders the draft and release produces one history entry.
No ruler click-to-create, drag-to-delete or tab-specific keyboard controller in this
increment; the existing keyboard-accessible dialog remains available. Indent
keyboard/source controls, native v15, palette and icons remain unchanged.

## Alpha.66: keyboard ruler tabs

Ctrl+Alt+F8 enters existing explicit tab stops (a Docs shortcut, not a verified
Word mapping). Left/Right previews 0.5 pt steps; Shift changes the step to 5 pt.
Enter applies and exits; Tab/Shift+Tab applies once and selects the next/previous
stop in sorted order, wrapping. Escape cancels only the current preview. Exact
values appear below the ruler. A no-op does not consume Undo or discard Redo.
A duplicate destination blocks acceptance until corrected or cancelled.

The controller uses the same engine draft and half-open selection as pointer
movement. Common lists are required; mixed/empty lists give an explicit message.
Keyboard access includes overlapping and off-column positions via the exact-value
preview. Pointer indent priority is unchanged. Text/paste/IME/scroll input cannot
edit the document during preview. Window focus loss, close, pointer press,
resize/zoom or hidden rulers cancel. Add/delete still use Tab positions.
Physical keyboard, VoiceOver and Windows acceptance remain unverified for this
increment. No storage, palette, icon, dependency or license change.

## Alpha.67: ruler tab creation and deletion

Click empty horizontal ruler space inside the text column to create a left tab at
the nearest half point. The click must finish on the ruler; indent/tab hit targets
keep priority. Right-click a tab and choose **Delete tab at … pt** to remove it.
Each completed action uses the same validated list command in one Undo/Redo.
Dismissing the menu changes nothing. Dragging outside still cancels movement;
this increment deliberately does not change that gesture into deletion.

In Ctrl+Alt+F8 tab mode, Delete previews removal of the original selected tab.
Enter or Tab applies once and exits (including when the last tab is removed).
Escape cancels; arrows return to movement preview. Deleting after movement removes
the original source in one transaction, without committing an intermediate move.
Exact removal text appears below the ruler. No text input reaches the document.

Creation/removal require identical lists across the half-open paragraph selection;
mixed lists use Tab positions. Creation rejects duplicates, more than 32 stops and
invalid positions. Source-document/selection checks prevent stale draft commits.
The existing dialog remains the keyboard route for creating positions, and the
route for editing overlapping/off-column markers with the pointer. Native v15,
palette, icons and ruler dimensions remain unchanged. Windows, physical OS input,
VoiceOver and matched reference acceptance are not established by widget tests.

## Alpha.68: mixed-list ruler editing

The ruler and Ctrl+Alt+F8 traversal use the sorted union of positions in the
half-open paragraph selection, including positions absent from its first paragraph.
Partial-coverage markers retain the existing gray tone; tooltips report N/M selected
paragraphs. Hover/active accents, indent hit priority, scale and icons are unchanged.

Creation adds a position to each selected paragraph where it is missing, leaving
existing occurrences and all other positions untouched. If all paragraphs already
have that position, show an explicit message without modifying history. Move and
remove affect only paragraphs containing the original source position. A destination
in a paragraph without that source is not a collision. A destination conflict in
any affected paragraph, or a 32-stop capacity failure when adding, rejects the whole
operation. Failed preview returns to the original document, never a partial result.

The shared command commits distinct per-paragraph lists as one Undo/Redo, retaining
other geometry, styles, text and exclusive-boundary paragraphs. Stale document or
selection rejects acceptance. Keyboard traversal follows the resulting sorted union;
delete still exits after acceptance. The Tab positions dialog intentionally retains
its explicit whole-list replacement policy. Native v15 remains unchanged.

Pointer/keyboard preview, Escape/focus/transform cancellation and outside-release
behavior remain as before. Physical input, VoiceOver, Windows DPI and matched Word
acceptance remain deferred; this is an explicit Docs policy, not claimed equivalence.

## Alpha.69: keyboard ruler tab creation

Ctrl+Alt+Shift+F8 starts a new tab preview, including when existing positions are
present. Ctrl+Alt+F8 keeps existing-marker traversal, but starts creation if the
selected union is empty. These are Docs shortcuts, not verified Word mappings.
Creation begins at 36 pt independently of the paragraph's automatic grid interval.
Left/Right adjusts by 0.5 pt; Shift uses 5 pt, bounded to 0.5–1440 pt.

The exact readout says **New tab**; the canvas and accent marker show the preview.
Enter or Tab applies once and exits. Escape or Delete cancels only the new draft.
Focus loss, pointer press, resize/zoom and hidden-ruler cancellation retain the same
policy as marker editing. Input is isolated; a second creation shortcut during an
active preview cannot silently commit or replace it.

A duplicate in every selected paragraph or a capacity error blocks acceptance,
leaves the real document unchanged and keeps arrow correction/Escape available.
The draft is invalid until a position validates. Mixed selections add only missing
occurrences, preserving every unrelated position. Commit uses the existing shared
validated command and one Undo/Redo; cancellation preserves Redo. Native v15,
ruler geometry, palette, icons and pointer behavior remain unchanged. The numeric
Tab positions dialog is still available. Physical keyboard/IME, matched Word,
Windows and VoiceOver acceptance remain deferred.

## Exact ruler tab input (alpha.70)

Enter a keyboard tab preview with Ctrl+Alt+F8 (existing stops) or
Ctrl+Alt+Shift+F8 (new stop), then press F2. The focused Position field selects
its current value. Type or paste points using a decimal dot, in 0.5 pt steps
from 0.5 to 1440. Enter or Tab applies once and exits; Escape cancels the entire
preview. Text-editing keys operate the field; Delete edits text in this mode.
Invalid input resets the visual preview and blocks acceptance, including after
a previously valid value. No rounding/clamping of typed values is performed.
Mixed-list and stale-source checks use the same validated draft transaction.
This is a Docs-specific shortcut; native v15, palette and icons are unchanged.

## Center and right tab storage/layout (alpha.71)

Native v16 adds `spacing.aligned_tab_stops`, an optional sorted array of
`{position_half_points, alignment}` objects. Alignment is `center` or `right`.
Existing `left_tab_stops_half_points` is retained. Across both lists, at most 32
positions are allowed, each distinct and within 1–2880 half points (0.5–1440 pt).
Null, unknown fields/types/alignment, unsorted or duplicate positions are rejected.
Missing arrays mean no aligned stops. v1–v15 reject the new key, even empty/null;
v15 without it still opens. Alpha.70 and older reject v16 saves.

All explicit types share one ordered sequence from the unindented text-column
origin. A tab selects the next position strictly beyond the cursor and any stop
already consumed on that row. After the final explicit stop, the original left
interval grid resumes. Center/right alignment measures the complete field through
the next tab or paragraph end, including trailing spaces and resolved formatting.
Center uses half its measured width; Right uses all of it. Empty fields have zero
width. The visible anchor is bounded by the available row edge, as for left tabs.

Advances never become negative: oversized fields begin at the current cursor and
wrap normally without overlapping preceding text. Continuation rows do not repeat
the field alignment. This is a defined continuous-layout policy, not pagination
or verified Word equivalence. Paragraph alignment and tab-row justification policy
remain unchanged. Stops do not inherit from named styles; Enter copies geometry.

`Editor::set_aligned_tab_stops` replaces only center/right entries in the selected
paragraphs with one validated Undo/Redo; exclusive next-paragraph boundaries are
excluded. Left-stop commands preserve aligned entries and reject conflicts.
Current dialogs and ruler edit left stops only; type selection and center/right
markers are a later increment. Palette and icons remain unchanged.

Open [AlignedTabs.bfdoc](../../examples/AlignedTabs.bfdoc) to inspect the working layout.
