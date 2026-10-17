# ADR 0005 — Previewed replacement transaction

- Date: 2026-10-17
- Status: implemented prototype, submitted for review; broader R1/D2 gates remain open
- Scope: replace every current literal-search match with one undo boundary

## Decision

`Editor::preview_replace_all` accepts ordered, disjoint, nonempty original scalar
ranges and replacement text. It validates source, grapheme boundaries, controls
and the final 100,000-scalar limit before returning an immutable `ReplacementPlan`.
The plan stores source/result snapshots, match/change counts and the final caret
and typing style. It changes no live document, selection, dirty state or history.

Build the result in a forward pass over original scalar/style units. Do not invoke
one live command per match or repeatedly copy the whole document. Do not rescan
inserted text. Each changed match inherits the first matched scalar's character
style (a paragraph separator uses its preceding paragraph's last style, or the
default for an empty paragraph). Text outside matches retains its original styles.
Replacement-created or joined paragraphs retain the first affected output
paragraph's alignment; an untouched original paragraph separator restores the
following original paragraph's alignment. This follows the existing split/merge
policy. Snap the final caret to a complete resulting grapheme boundary.

An equal-text replacement is a true no-op, including mixed formatting. Match and
actual-change counts are separate. An all-no-op or empty plan preserves selection,
typing style, undo and redo. `apply_replace_all` compares the entire current document
with the source snapshot, including formatting. A stale plan fails without changes.
Otherwise the result is committed once through the existing bounded snapshot
history. Undo restores prior content, formatting, selection and typing state;
redo restores the complete result. This is an in-memory edit, not a file save.

## GUI projection

Home Replace uses the same cached search policy (Match case/Whole words) to prepare
the plan. The modal shows original paragraph/context excerpts and replacement text,
with ellipses for bounded display; the full replacement is applied without display
truncation. Virtual rows expose every match. Cancel is initially focused; Enter
there cancels. Tab to Apply then Enter, or pointer Apply, commits the plan. Escape
cancels. Unchanged-only previews disable Apply. Background edits, commands and
window-close are blocked until the preview is resolved. The input composition
guard includes the replacement field. Cancel returns to search without syncing
or editing the document; Apply refreshes search from the committed result.

## Limits and alternatives

Do not promise locale-specific search, normalization, regex input or per-match
checkbox selection. The existing literal Unicode matching policy is unchanged.
Whole-document snapshots are consistent with the current short-document model;
larger-document performance remains a separate milestone. There is no new schema,
dependency, project license, cloud integration or file-recovery guarantee. Native
Windows input/DPI, IME and exact reference-dialog fidelity still need acceptance.
