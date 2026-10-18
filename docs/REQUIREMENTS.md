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
