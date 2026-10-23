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

## Typed tab editing (alpha.72)

The Tab positions dialog now lists all Left/Center/Right stops and shows their type.
Selecting an entry restores its position and type. Choose Alignment, then Add or
Update to stage the edit; OK applies the entire list to the selected paragraphs in
one Undo/Redo. Pending nonempty input/type changes require Add/Update before OK.
Cancel/Escape discards the draft. Remove and Clear all affect all types. Mixed
selection compares both positions and types; its replacement policy is explicit.
Tab reaches the position field, three type controls and the existing action buttons.

The ruler shows L-shaped left stops, a bottom bar on both sides for center, and a
mirrored L for right. Partial coverage remains gray. Different types at the same
position show a gray double-bar marker and a Mixed types label. Tooltip/widget
labels include type and selected-paragraph coverage; active previews use the
existing accent. Indent hit targets retain priority.

Pointer movement/release, Ctrl+Alt+F8 traversal, arrows/Shift+arrows, F2 exact input,
Delete and context-menu deletion now support all types. Movement/deletion changes
only paragraphs owning the source position and preserves each stored type, including
mixed types. A destination conflict in any affected paragraph rejects the whole
transaction. Enter applies once; Tab applies/traverses; Escape, focus loss and
transform changes keep the previous cancellation policy. No-op acceptance preserves
Redo. Existing ruler creation still inserts left stops; choose Center/Right in the
dialog. Type selection for new ruler stops is a separate next increment.

The unified editing representation maps to the existing v16 arrays; no schema,
palette, icon or layout-policy change. Physical keyboard/IME, matched Word,
Windows and VoiceOver acceptance remain deferred.

## Typed ruler creation (alpha.73)

The 24 × 24 DIP selector immediately left of the horizontal ruler shows the type
for new stops. Click it or press Ctrl+Alt+F7 to cycle Left → Center → Right → Left;
right-click for explicit choices. This Docs-specific shortcut shows the ruler if
hidden. The choice is local to the window, starts at Left and is not serialized or
included in undo history. Existing stops and the document selection stay unchanged.
The selector is unavailable during drafts, modal editing, composition or indent
keyboard control. Type is frozen when a creation draft begins.

Press an empty ruler location to preview a stop of that type; move horizontally
while held and release inside the ruler to apply once. Escape, window focus loss,
transform changes or release outside cancel. A press/release in the same event
frame commits once. Existing indent/tab hit targets retain priority. Invalid
creation previews cannot commit; releasing inside reports their error.

Ctrl+Alt+Shift+F8 starts creation at 36 pt using the selector. Ctrl+Alt+F8 does the
same when no explicit positions exist. Arrows adjust by 0.5 pt (Shift: 5 pt), F2
opens exact point input, Enter/Tab commits once and exits, Escape/Delete cancels.
The preview identifies the selected type. Numeric-input Delete retains text-editing
behavior. Same-kind stops at the destination are preserved while missing entries
are added; a different kind at that position rejects the entire operation.
Capacity, stale-source and half-open paragraph-selection validation remain shared.

Native v16, existing layout policy, palette and icons remain unchanged. Physical
keyboard/IME, matched Word, Windows and VoiceOver acceptance remain separate.

## Interaction recovery (alpha.74)

A draft records the last attempted bounded position independently of preview
validity. Collision leaves the preview equal to the original document and commit
invalid, while the next arrow advances from the attempted position. This avoids
trapping keyboard movement before adjacent occupied stops. Invalid numeric syntax
still invalidates the preview without inventing a coordinate. Delete targets the
original source, even after a rejected move; a valid later move cancels deletion.

Pointer drafts now cancel when rulers are hidden or File view becomes active,
matching the keyboard controller. Cancellation does not change history or Redo.
No storage, layout policy, palette, icon or dependency changes are introduced.

## Tab leader storage and layout (alpha.75)

Explicit Left/Center/Right positions may own a dot, dash or line leader. Absence
means none; automatic grid stops never acquire leaders. Native v17 stores sorted
`spacing.tab_leaders` objects with `position_half_points` and `leader` (`dots`,
`dashes`, `line`). Every entry must reference an existing explicit stop; duplicates,
unsorted/orphan positions, null and unknown keys/types are rejected. v1–v16 remain
readable but reject this field even when empty/null. Alpha.74 and older reject v17.

The shared `Editor::set_tab_leader` sets/clears a chosen position across a half-open
paragraph selection in one Undo/Redo. A missing stop in any selected paragraph
rejects the transaction. Ruler movement carries each owner's leader, deletion
removes it, and cancel/invalid previews preserve the source. Whole-list replacement
retains leaders at surviving coordinates and removes those at removed coordinates.
Changing alignment at the same coordinate retains its leader. Enter copies the
paragraph geometry under the existing policy; no style inheritance is added.

Layout paints original geometric dots, dashes or a line inside the actual positive
tab advance, with 0.15-em side clearance and the tab's resolved color/font size.
Dot/dash pitch is 0.5 em; at most 512 marks are drawn per tab (a longer gap leaves
its tail unfilled). Line rendering uses one rectangle. Very short/zero advances
have no fill. Center/right fields keep their existing measured alignment; fills
never extend into the following field. Wrapped rows, indentation, zoom and paragraph
alignment use the existing tab geometry. Underline remains independent.

Leaders add no stored characters, glyph entries or cursor stops. No layout-policy,
palette/icon or dependency changes. This is a prototype rendering policy, not
verified Word equivalence. UI selection of leader kind follows separately; open
[TabLeaders.bfdoc](../../examples/TabLeaders.bfdoc) to inspect the current renderer.

## Leader editing controls (alpha.76)

Five connected controls complete the next leader-editing increment:

1. The tab positions dialog offers None/Dots/Dashes/Line. Select a row to restore
   its type, position and leader. Add/Update stages all three; OK commits once.
   Moving a position via Update carries the selected leader. Remove/Clear all also
   remove the corresponding leaders. Pending edits require Add/Update.
2. An explicitly illustrative Sample/Text preview uses the real native layout
   adapter at a fixed sample position, without changing the document or history.
3. Mixed detection includes leaders. Even identical positions/types with different
   leaders require an explicit replacement or Clear all; Cancel preserves each
   paragraph. OK replaces positions, types and leaders together across the half-open
   selection. Unchanged common lists preserve Redo.
4. The ruler tab context menu offers leader set/reset. It changes only selected
   paragraphs owning that stop, retaining nonowners, types and positions. One shared
   command produces one Undo/Redo; None clears the leader without deleting the stop.
5. Tab/Shift+Tab reaches the leader choices between Alignment and Add; Enter/Space
   activates focused controls and Escape cancels. Full keyboard-only creation,
   staging and OK are covered by synthetic widget tests. Physical OS input and
   VoiceOver acceptance remain separate.

Native v17, layout policy, palette and icons remain unchanged. Windows is deferred.
The list height is bounded for compact windows; no pagination or Word equivalence
is claimed. Next: leader provenance/coverage labels for mixed ruler selections.

## Leader inspection and keyboard previews (alpha.77)

Ruler tooltip/widget labels report None/Dots/Dashes/Line counts among paragraphs
owning the stop and a separate no-stop count. A half-open selection excludes the
next paragraph when its start is the exclusive end. The context menu repeats the
counts, selects a choice only when all owners agree, and retains owner-only edits.
Mixed values do not falsely select None; an absent stop is not a None leader.

In Ctrl+Alt+F8 (existing) or Ctrl+Alt+Shift+F8 (creation) keyboard preview, F1 toggles
read-only leader details. F3 cycles None → Dots → Dashes → Line → None; a mixed
selection starts at None. F3 is available outside numeric input and only for valid
nondeletion previews. Choose a leader before F2 to combine it with exact position
input. Invalid previews show details as unavailable, rather than another stop's
values. Delete and Escape keep their established semantics.

The chosen override survives arrows, temporary collision errors and later valid
numeric input. Existing-marker movement changes only owners; creation changes the
newly added stops while preserving preexisting same-kind occurrences. Enter commits
position and leader together once; cancellation leaves history/Redo untouched.
No file schema, layout geometry, palette/icon or dependency change. Windows,
physical OS input and VoiceOver remain deferred. Next: direct keyboard leader
selection without cycling, preserving this preview/history boundary.

### Direct ruler leader selection (alpha.78)

In a keyboard tab preview, press 0/1/2/3 for None/Dots/Dashes/Line. Shift+F3
cycles backward (mixed starts at Line); F3 still cycles forward. Backspace
restores each original owner's leader, keeping the current position. Creation
restores no fill on newly added stops and preserves preexisting owners.
F2 enters numeric input; digits then edit the position. F4 returns to marker
controls only when the position is valid, without committing. Enter applies once;
Escape cancels all draft changes. The chosen leader is displayed separately from
coverage, because preserved creation owners may retain different leaders.
These are BreakFree preview shortcuts; no reference-app shortcut equivalence is
claimed. Native v17, palette and icons remain unchanged.

### Shared ruler preview controls (alpha.79)

Right-click a tab marker and choose Edit tab preview to enter the same draft used
by Ctrl+Alt+F8. None/Dots/Dashes/Line and Restore leaders change the preview only.
Exact position opens numeric input; Back to marker requires valid input and does
not commit. Apply commits the entire draft once; Cancel/Escape discards it.
Invalid positions disable leader changes and Apply, but leave Cancel available.
Outside clicks still cancel; pointer dragging still uses its original release
policy. Mixed owners and preexisting creation stops retain the established rules.
Keyboard commands remain available; Tab retains its apply-and-next behavior.
Focus traversal for these new buttons is a separate next step. Windows and
VoiceOver acceptance remain deferred. Native v17, palette and icons are unchanged.

### Preview control navigation (alpha.80)

F6 enters the preview buttons at the first enabled control. Tab/Shift+Tab wraps
through enabled leader choices, Restore, Position, Apply and Cancel. Enter/Space
activates the focused button; repeated key events do not reactivate it. A border
using the existing accent marks focus independently of the selected leader.
Clicking a button makes it the starting point for subsequent keyboard traversal.
F6 returns to the marker or numeric input without committing; clicking numeric
input returns to text editing. Outside controls, Tab keeps apply-and-next behavior.
If the focused control becomes disabled, focus moves to Cancel. Invalid numeric
input exposes only Cancel to control traversal; F6 returns to input for correction.
Escape still cancels the entire draft. Native v17 and palette/icons are retained.
These are local synthetic/native-render checks, not Windows or VoiceOver acceptance.

### Preview descriptions (alpha.81)

The focused preview button shows a read-only explanation and contextual shortcuts.
The same help is available on pointer hover and in button accessibility labels.
Leader help distinguishes existing owners from newly created stops; position help
switches between F2 entry and F4 return. Numeric-mode digits remain position input.
Disabled controls explain invalid positions, deletion or already-retained original
leaders. Selection and disabled metadata remain separate from descriptive labels.
No help inspection changes document/history. Native v17, palette/icons and existing
preview commands are retained. AccessKit tree checks do not establish VoiceOver
speech; physical input, Windows and spoken-output acceptance remain deferred.
