# First milestone — a reliable writing loop

**Acceptance requirements for the first prototype.** Implementation and local test
evidence now exist in [VALIDATION.md](VALIDATION.md); complete milestone approval
and target-platform reviews remain open. The scenario IDs below are preserved.

## User story

A person creates a short document, types several paragraphs, applies basic formatting, saves it locally, reopens it and continues editing without losing supported content.

## In scope

A single document window, paragraphs, text selection, basic emphasis, paragraph
alignment, undo/redo, local persistence and unsaved-change handling. Rust is the approved runtime language. The native ribbon prototype and preserved
v1 reader are documented in [ADR 0002](adr/0002-rust-native-ribbon.md).
[ADR 0003](adr/0003-character-size-and-underline.md) adds size/underline and content
v2 writing while preserving the D01–D09 scenarios.
[ADR 0006](adr/0006-built-in-paragraph-styles.md) extends this with fixed paragraph
styles, direct overrides and v3 writing while retaining strict v1/v2 readers.
[ADR 0007](adr/0007-editable-style-definitions.md) adds editable definitions, parent
validation and v4 writing with strict v1/v2/v3 readers.
[ADR 0008](adr/0008-custom-named-styles.md) adds named custom paragraph styles,
validated IDs/parents, one-operation creation/application and v5 writing with
strict v1–v4 readers. Alpha.21 extends the same command boundary to renaming,
with stable IDs/references and atomic name/definition undo.
[ADR 0009](adr/0009-delete-custom-styles.md) adds deletion with explicit paragraph
and child-parent replacements in one transaction, retaining native v5. Follow the
Word 2024 Windows reference in [UI_SPEC](UI_SPEC.md). GUI-library adoption, licensing
and supported-platform decisions remain open.

## Acceptance scenarios

| ID | Scenario | Observable result |
| :--- | :--- | :--- |
| D01 | Create, type, select, replace and delete text | The document model and displayed text agree. |
| D02 | Enter English, Ukrainian and mixed Unicode examples | Supported text is preserved exactly; unsupported rendering is identified rather than silently changed. |
| D03 | Apply supported emphasis and paragraph alignment | Save/reopen preserves the chosen formatting. |
| D04 | Undo and redo a sequence of edits | Content and formatting return to the recorded prior and next states. |
| D05 | Save, close, reopen and continue | Supported document structure and content are preserved. |
| D06 | Close with unsaved changes | Save, discard and cancel have distinct, tested outcomes. |
| D07 | Simulate a failed or interrupted save | A previously valid destination is not silently destroyed; the error is visible and the edited document remains recoverable where designed. |
| D08 | Open malformed or unsupported input | A clear error is shown; the input file is not modified. |
| D09 | Complete the basic workflow using the keyboard | Focus is visible and controls are reachable in a sensible order. |

## Required engineering decisions

Specify the document model, file-format versioning, encoding, save transaction, undo boundaries and supported text-rendering behavior. Record the decision, alternatives and tradeoffs rather than choosing a framework in a README badge.

Do not execute macros or embedded scripts and do not upload files to remote services as part of this milestone.

## Not included yet

DOCX round-trip compatibility, tables, images, pagination, PDF export, printing, cloud accounts, collaboration and macros. These need later, separately testable milestones.

## Completion evidence

A reproducible build and test command, automated persistence and undo tests, synthetic fixtures, manual keyboard checks on the declared operating system and a list of known limitations. Only actual checks may be reported as passed.

[Back to roadmap](../ROADMAP.md)

Alpha.23 extends style behavior with [ADR 0010](adr/0010-following-paragraph-styles.md):
following styles, transactional Enter and strict v6 migration. D01–D09 remain.

Alpha.24 adds built-in character styles with explicit precedence and one-operation
history, specified in [ADR 0011](adr/0011-character-style-precedence.md). Native v7
stores references independently of direct properties; earlier readers stay strict.

Alpha.25 adds named character-style creation, optional one-step application and
strict v8 storage under [ADR 0012](adr/0012-named-character-styles.md).

Alpha.26 adds transactional custom character property editing with stable references. Native v8 remains unchanged.

Alpha.27 adds validated character-style renaming and combined property edits in one Undo/Redo, preserving IDs/references and native v8.

Alpha.28 adds custom character-style deletion with explicit text/pending replacement and one Undo/Redo, retaining native v8.

Alpha.29 adds character parent inheritance, graph validation, child-parent remapping on deletion and transactional drafts; native v9 retains v1–v8 readers.

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
or Document default. Explicit zero is preserved. Space before and direct interval overrides are supported from alpha.42; the inspector identifies their supplying style or direct source.

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

## Named return destination (alpha.38)

The return button now says Back to: followed by the original parent style name.
The name is resolved from the stored stable ID on each frame, including custom
paragraph and character names. Its 23-DIP-high button fits the 220-DIP menu width and truncates long labels;
the tooltip retains the full name and restoration explanation. Return behavior,
palette, icons, document/history and storage are unchanged.


## Dependency navigation resilience (alpha.39)

Pending jumps are checked against the current document and filters. If a source
was removed, renamed outside the saved query or excluded by changed usage,
Return restores the exact filters, explains why the row cannot be shown and
focuses the filter. It does not apply a replacement style. Explicit filter edits
cancel pending navigation and clear the notice. Navigation does not change
content or undo history. A 32-style reverse-catalog chain and repeated keyboard
jumps are covered by widget tests; native Windows acceptance remains open.


## Windows navigation acceptance preparation (alpha.40)

An 18-case CPU widget matrix exercises dependency navigation at synthetic
100/125/150% scale, three window sizes and expanded/collapsed Home.
The [Windows acceptance checklist](WINDOWS_ACCEPTANCE.md) defines the separate native gate.
A Unicode 32-style fixture and a Windows CI review artifact with source/version/hash
metadata make the procedure reproducible. No suitable Windows desktop was
available; native keyboard, display scaling and Word comparison remain unverified.
The review artifact is not a release. Runtime behavior, palette and icons are unchanged.


## Direct paragraph spacing (alpha.41)

Paragraphs may override line height (100–300 percent) and space after (0–400 half
points); null inherits the paragraph style. Partial updates leave unchecked
properties untouched, including mixed selections. Explicit zero is an override.
The shared command validates before commit and changes intersecting paragraphs
with one undo/redo. GUI, layout and source inspection use the same model.
Enter and inserted replacement paragraphs copy the starting paragraph's direct
spacing; joins keep the first paragraph's spacing. Unaffected paragraphs retain
theirs. Native v10 persists the overrides; frozen v1–v9 imports inherit.
No palette/icon change. Native Windows acceptance is deferred by owner request.

## Space before paragraph (alpha.42)

Home → Paragraph spacing supports Space before in points (0–200, steps of 0.5).
Check the field to change it; leave it unchecked to preserve each selected value.
A checked blank field inherits from the paragraph style. Use style spacing clears
all three direct interval overrides in one Undo/Redo. Paragraph style creation and
modification also support space before; a blank style value inherits its parent,
with zero as the document default. Explicit zero overrides a nonzero ancestor.
Formatting inspection reports the source. Spacing applies once before each
paragraph, including the first, and adds to the previous paragraph's space after.
This continuous layout policy is not pagination. Native v11 preserves the property;
v1–v10 imports resolve to zero before. Windows acceptance remains deferred.

## Paragraph indents (alpha.43)

Home → Paragraph spacing now includes Left indent and Right indent, in points
(0–144, half-point steps). Each checkbox selects a property to change; unchecked
fields keep existing values. A checked blank restores style inheritance. Use style
spacing and indents resets all five properties in one Undo/Redo. Paragraph style
creation/modification exposes the same fields; blank inherits the nearest ancestor,
then document default zero. Direct zero explicitly cancels an inherited indent.
Formatting inspection identifies each supplying style or direct override.

Wrap and alignment use the indented column. On narrow views, horizontal indents
shrink proportionally to retain up to 32 DIP of text width; document values remain
unchanged. This is continuous layout, not pagination. Enter/join/Replace All follow
the existing paragraph geometry copying policy. Native v12 preserves both indents;
v1–v11 imports use zero. First-line/hanging indents, negative outdents and interactive
ruler handles remain separate increments. Windows acceptance remains deferred.

## First-line and hanging indents (alpha.44)

Home → Paragraph spacing → Special indent changes only the checked property.
Choose First line or Hanging and enter 0–144 pt in 0.5 pt steps. First line moves
the first row right from the left indent; Hanging leaves the first row at the left
indent and moves continuation rows right by the amount. The left indent denotes
the outermost edge in this continuous-layout policy. Right indent is unchanged.

A checked blank inherits the nearest paragraph style/ancestor; explicit zero
disables the inherited special indent. Use style spacing and indents resets all
six properties together. Style creation/modification has the same type and amount.
A blank style amount inherits both type and magnitude regardless of the displayed
type choice. Document default is zero. Apply/reset/parent changes each use one
Undo/Redo. Formatting inspection reports type, amount and source.

Wrapping uses distinct first-row/continuation widths without inserting characters
or paragraph breaks. Narrow views proportionally reduce visual left/right/special
indents to retain up to 32 DIP of text width; saved values do not change. Enter,
joins and Replace All retain the established geometry copying policy. Native v13
preserves the signed property; older imports inherit zero. Interactive ruler handles,
negative outer-edge outdents and pagination remain future work. Windows is deferred.

## Indent layout hardening (alpha.45)

Justified trailing spaces retain their logical cursor positions after visible ink.
The visual text column is unchanged; a row-end caret can extend through trailing
whitespace beyond the visible justified edge. This avoids moving the caret before
a stored space at a soft wrap. Tests cover long mixed-format Unicode paragraphs,
empty/trailing paragraphs, narrow views, synthetic scale and zoom/edit/history.
Compact style creation reserves extra room for differing interface-font metrics.
Native content stays v13; interactive ruler handles and native Windows acceptance
remain pending.

Alpha.86 adds paragraph-style tab interval and complete stop/leader inheritance
under [ADR 0015](adr/0015-style-tabulation.md), native v18 and transactional nested
style drafts. Direct overrides win; explicit empty lists differ from inheritance.
D01–D09 remain required; Windows and VoiceOver acceptance remain separate.
