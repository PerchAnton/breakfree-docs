# ADR 0012 — Document-owned named character styles

Date: 2026-10-18. Status: implemented prototype policy for owner review.
Extends ADR 0011 without changing the approved Rust stack or GUI palette.

## Decision

Store a separate character catalog on Document. Each custom style has a stable
positive u32 ID, a validated name and optional bold/italic/underline/font size.
A run or pending insertion format references the ID; do not copy resolved values
into direct formatting. The cascade remains paragraph → character → direct.
Explicit false wins. Alpha.29 extends inheritance with character parents; linked
paragraph/character styles remain deferred.

Character names are a separate namespace from paragraph names. Enforce 1–64 Unicode
scalars, no outer spaces/control characters/U+2028/U+2029, case-insensitive uniqueness
using Unicode lowercase, reserved built-in names and a limit of 64 custom entries.
No Unicode normalization or full case-folding claim. Allocate max ID + 1 with checked
overflow. Validate every definition/reference, including unused entries.

Create Character Style is a local draft reusing existing style-dialog components:
name, Inherit/On/Off toggles, optional size, preview, Apply checkbox, OK and Cancel.
Paragraph-specific controls are hidden. Preview uses the active paragraph baseline,
not its direct overrides; the explanatory text says direct formatting wins.
Home Styles and command search expose creation; K enters it inside Styles/Editing
KeyTips. Custom choices use current names in menus/search and remain scrollable.

A single shared editor command validates and commits definition creation plus
optional application to the grapheme-aligned selection or pending insertion style.
One undo restores the prior document, selection and pending state; redo restores
IDs/definition/references. Definition-only creation dirties the document because it
is persisted, unlike applying an existing style at an empty caret. Cancel/invalid
input preserves document, selection, dirty state and redo. Text edits, Enter and
Replace All retain the catalog. Rendering and toolbar state resolve document-owned
properties through CharacterFormat::resolve_with; paragraph definitions use only
direct properties and cannot reference character styles.

## Persistence and limits

Native v8 requires character_styles and extends run references with custom IDs.
Strict v1–v7 readers remain; imported documents receive an empty character catalog.
Opening never writes source bytes. Explicit Save writes v8; alpha.24 and earlier
reject it. Existing write/sync/replace failure retention remains tested; no new
crash-recovery promise. No Office/sibling/cloud dependency or license change.

Use engine/storage tests, actual widget/layout checks and native macOS smoke for
local evidence. These do not establish Windows DPI/input or matched Word behavior.

## Alpha.26 — Property editing

Modify Character Style drafts only properties, never identity or name. The shared
Document validator rejects built-ins, unknown IDs and nested character references
before assignment. Editor commits one snapshot preserving selection, pending
format and all run references; equal definitions do not consume undo or clear redo.
The same dialog previews inherited properties and preserves direct overrides.
Home/command search require a uniform custom selection; the style-row context
command also permits unused definitions. J enters modification in Styles/Editing
KeyTips. Cancel and invalid drafts leave the document untouched. No schema change:
existing native v8 definitions already carry these properties.

## Alpha.27 — Renaming

Modify Character Style now drafts the name as well as properties. Select the old
name initially, validate the complete candidate catalog before assignment, then
commit name/properties as one editor transaction. Names follow the creation rules;
case-only changes are allowed for the same ID. Invalid names, unknown IDs, fixed
built-ins and unchanged drafts leave history intact. IDs/order/references, selection
and pending format remain stable. Menus/search resolve the current name from the
catalog. Native v8 already stores names; no migration or schema change is needed.

## Alpha.28 — Deletion

Delete only custom definitions. Validate source and replacement before mutation;
reject missing IDs, built-ins and self-replacement. Replacement may be a surviving
custom style, Strong, Emphasis or None (Default paragraph font). Replace every run
reference and pending insertion reference, preserving direct properties, text,
paragraph styles, selection and survivor order. Commit the removed definition and
all remapped state in one shared history operation. Undo restores exact IDs and
references; redo repeats deletion. An unused definition can be removed too.

The local draft explains possible inherited appearance changes and displays the
actual count of referenced text scalars (not graphemes or words). Cancel starts
focused. Home/search and X KeyTip target a uniform custom style; context access
also handles unused definitions. Escape/Cancel do not mutate. Existing v8 storage
already represents the result; no schema change or dependency is required.

## Alpha.29 — Parent inheritance

Each custom definition has an optional parent ID, including fixed Strong/Emphasis.
None uses the current paragraph baseline. Resolve ancestors from root to leaf,
then the run's direct properties; explicit false overrides inherited true. Bound
resolution walks even for invalid external models; commands and storage reject
all missing references and direct/indirect cycles, including unused definitions.
Create/Modify drafts parent/name/properties and commits one operation. Preview
uses the same cascade. Existing property-only APIs preserve the parent.

Deletion applies its replacement to direct child parents as well as text/pending
references. A descendant replacement that creates a cycle rejects the whole edit;
None reconnects children to the paragraph baseline. One undo restores the graph.
Native v9 stores parent references. Frozen v8 definitions reject parent fields;
v1–v8 imports give character definitions no parent. Save writes v9; alpha.28 and
earlier reject it. No parent guessing, flattening, dependency or palette change.

## Formatting provenance (alpha.30)

Open Home's Formatting pane to inspect Bold, Italic, Underline and font size.
Each row shows its resolved value and authored source, including the named parent
that supplies an inherited value. Equal-looking direct overrides remain direct.
Mixed values and mixed sources are reported independently. With no selection,
the pane describes future typing; selected paragraph separators have no character
properties. This read-only view does not change document/history or native v9.
Alpha.31 adds paragraph-property provenance below; style-management filtering remains future work.

## Paragraph provenance (alpha.31)

Formatting also shows Alignment, Line spacing (percent) and Space after (points).
Alignment is an explicit paragraph value in the current model, labeled
“Paragraph value (not inherited)”; it is not attributed to a style or guessed
from appearance. Intervals show the nearest supplying paragraph style/ancestor,
or Document default. Explicit zero is preserved. There is no space-before property
or direct paragraph-spacing override in this prototype.

The paragraph section uses the same half-open selection rule as alignment/style
commands: an exclusive end at the next paragraph start excludes that paragraph;
a caret or selected empty paragraph still has paragraph properties. Mixed values
and sources are independent. Inspection never mutates the document or undo history.
Native content stays v9; palette and icons are unchanged.

## Style filtering (alpha.32)

Open Home → More styles (or the compact Editing menu). Filter by name and cycle
All / Paragraph / Character / Custom using the scope button. Reset clears both.
The list reports no matches explicitly; management commands remain available.
F3 focuses the name field; Down enters the filtered command list, Enter applies,
and Escape closes the popup. Tab/Space operate the filter controls.

Filtering is view-only and session-local: no document/history/settings writes.
Search trims surrounding whitespace and compares Unicode lowercase substrings;
canonical normalization, locale-sensitive folding and fuzzy matching are not
claimed. Built-in/default-character entries are excluded by Custom. Existing
style application and context editing retain stable IDs. Native v9, palette and
icons are unchanged.

## Used styles (alpha.33)

Enable **Used in document** in More styles. This intersects name/category filters
and includes styles assigned to every paragraph (even empty ones), character
styles referenced by text, and all transitive parents. Direct formatting does
not remove a semantic style dependency. Unused descendants/siblings, next-style
links and pending typing alone do not count. Default paragraph font is shown for
text without a character style or an empty paragraph. Reset clears all filters.
Usage is recomputed from the current model, so edits and Undo/Redo update the list;
filtering itself never modifies content/history/settings. Native v9 is unchanged.

## Style usage labels (alpha.34)

Home style menus show Direct use, Parent dependency or Not used beneath each
name, including when the used-only filter is off. Direct assignment wins if a
style is also an ancestor. Parent dependency means inheritance-only use, not
direct formatting. Empty paragraphs and default character formatting retain the
alpha.33 rules; pending typing and next-style links do not count. Search matches
style names, not these labels. Labels are read-only, recomputed after edits and
Undo/Redo, and do not change stable IDs, ordering, storage or settings.

## Dependency consumers (alpha.35)

Style rows with assigned descendants show Required by followed by their names.
Consumers are unique directly assigned styles in document encounter order; the
walk includes every transitive parent, excludes self and unassigned intermediate
styles, and is bounded for invalid external cycles. Paragraph and character
style namespaces remain separate. Direct-use parents also show their consumers.
Names wrap inside the existing scroll area; filtering does not hide dependency
evidence. Renaming, editing and Undo/Redo are reflected from the current model.
No document, history, settings, palette, icons or native format changes.

## Dependency navigation (alpha.36)

Activate a Required by name to reveal its style row without applying it. The
menu stays open; name/category filters clear, Used in document stays unchanged,
and the destination receives keyboard focus and scrolls into view. Links use
stable paragraph/character IDs, so duplicate names across namespaces are safe.
Enter/Space activates the navigation target; a separate activation of the focused
style applies it. Escape closes the menu. Document/history/settings stay unchanged.

## Return to parent (alpha.37)

Back to parent restores the source of the first Required by jump together with
its exact query, category and used-only setting. Further jumps and filter edits
do not replace that return point. Pointer or keyboard activation keeps the menu
open, focuses and scrolls to the original style without applying it. The return
point is consumed on return and discarded when the menu closes; it is not saved
in the document, settings or undo history. The list shrinks from 125 to 100 DIP
while Back is available so the minimum-height popup remains bounded.
