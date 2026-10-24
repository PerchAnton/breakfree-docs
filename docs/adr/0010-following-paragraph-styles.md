# ADR 0010 — following paragraph styles

Date: 2026-10-18. Status: implemented for review in alpha.23.
Extends ADR 0006–0009 without new dependencies or palette changes.

## Model and transaction

Each definition has `next_style`: an existing built-in/custom ID, or null/absent
for Same style. It is behavioral metadata, not inherited from the parent. Defaults
and legacy imports repeat the current style. Explicit self links and reciprocal
following links are valid: Enter follows one edge, not a recursive graph walk.
Unknown references are rejected before mutation. Inheritance remains acyclic.

Create/Modify Style exposes Following paragraph with Same style and all current
styles. Names are presentation only. The existing draft/validation/one-operation
command includes this choice along with name, parent and formatting. Cancel and
invalid drafts preserve the document and redo branch.

Enter replaces the selected grapheme-aligned range with a paragraph separator.
The left paragraph retains its style; the newly created right paragraph receives
the source paragraph's successor, at the beginning, middle or end of a paragraph.
A selection spanning paragraphs uses the first affected paragraph's rule. Keep
surviving text, direct character overrides, pending formatting and source
alignment. The insertion and style transition share one history snapshot; undo
and redo restore exact content, references, selection and pending formatting.

`insert_paragraph_break` is the explicit engine command. `replace_keyboard_text`
handles a widget batch containing typing and one or more Enter events, advancing
once per inserted separator; the complete batch retains the established single
undo boundary. Paste, replacement and IME commits use ordinary replacement and
never trigger the following-style rule. No soft-line-break model is introduced.
GUI event routing excludes batches containing paste or IME events from automatic
transitions; physical IME and unusual mixed OS batches remain acceptance gates.

Deleting a custom style remaps surviving `next_style` references to the explicitly
chosen paragraph replacement, together with paragraph and direct-parent changes.
The deletion dialog discloses this rule and counts these references. Rename keeps
IDs, so following links survive unchanged. Character styles remain outside scope.

## Native content v6

Write v6 with `next_style` in every built-in/custom definition. Null or omission
means Same style; IDs use the existing string/custom object representation.
Retain strict frozen v4/v5 definitions which reject this new field. Legacy v1–v5
imports add Same style in memory without rewriting source bytes or making the
session dirty. Save writes v6; alpha.22 and earlier reject it. Use Save As to retain
an older file. Unknown/duplicate fields, invalid references, versions and resource
limits still fail explicitly. No downgrade, recovery or new atomicity guarantee.
Process protocol and toolbar settings stay v1.

## Validation scope

Test start/middle/end/selection/empty/Unicode splits, non-inherited and reciprocal
following rules, exact history, paste isolation, missing references, deletion,
v5 migration, v6 roundtrip and failed-save retention. Exercise the actual selector,
keyboard Enter, combined typing/Enter, cancellation and dialog geometry. Native
Windows, physical IME, accessibility and paired Word screenshots remain unverified;
this document defines Docs behavior rather than claiming exact Word equivalence.
