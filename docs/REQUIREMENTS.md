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
