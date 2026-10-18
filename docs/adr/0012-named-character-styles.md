# ADR 0012 — Document-owned named character styles

Date: 2026-10-18. Status: implemented prototype policy for owner review.
Extends ADR 0011 without changing the approved Rust stack or GUI palette.

## Decision

Store a separate character catalog on Document. Each custom style has a stable
positive u32 ID, a validated name and optional bold/italic/underline/font size.
A run or pending insertion format references the ID; do not copy resolved values
into direct formatting. The cascade remains paragraph → character → direct.
Explicit false wins, unset properties inherit the paragraph. Character parents,
linked paragraph/character styles and modification/rename/delete are deferred.

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
