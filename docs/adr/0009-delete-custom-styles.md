# ADR 0009 — deleting custom styles and replacing references

Date: 2026-10-18. Status: implemented for review in alpha.22.
Extends [ADR 0008](0008-custom-named-styles.md); no dependency or schema change.

## Reference policy

Only custom styles can be removed. Require an existing replacement style for all
paragraphs directly using the target, and independently select an existing new
parent or No style for every definition whose parent is the target. The latter
includes built-in definitions: they may inherit from custom styles, although they
cannot themselves be deleted. Indirect descendants retain their direct parent
IDs. The same replacement parent applies to all immediate children in this step;
individual child-by-child remapping is outside scope.

Exclude the target from both lists. Reject missing references and validate the
entire surviving graph after reparenting, including self/indirect cycles. A
paragraph replacement may be a surviving descendant if the resulting graph is
valid. Default both replacements to the target's current parent; a root defaults
to Normal for paragraphs and No style for children. Defaults are visible in the
draft and require explicit Delete activation. Validate choices even when their
current reference count is zero.

Preserve text, direct character overrides, paragraph alignment, selection and
pending input. Do not flatten inherited values or copy the removed definition to
survivors. Inherited appearance and spacing may change according to the chosen
replacements; disclose this before applying. Stable IDs/names/overrides of all
surviving definitions remain unchanged except for requested direct parent links.
Deleting an unused definition still changes the document and creates one undo.

## Command and draft

`StyleCatalog::without_custom_style` validates replacement choices and returns a
candidate catalog. The GUI uses it for draft validation. The shared
`Editor::delete_custom_style` remaps paragraph references, validates the resulting
document and commits once with the label Delete style. Invalid operations leave
all document/history/dirty state intact. Undo restores the removed definition and
all references together; redo restores the exact deletion result. The normal
snapshot history makes later ID reuse safe; branching discards obsolete redo.

Home Styles/More, command search and provisional H → L → D or narrow H → E → D
KeyTips target the first selected paragraph's style. Built-in targets disable the
command with an explanation. Gallery and Styles-list context menus target a
specific custom style without applying it first, including unused definitions.

Delete Style is a modal draft with two replacement selectors and actual counts
of directly affected paragraphs and child definitions. It has no document mutation
until Delete. Cancel is initially focused; Enter activates the focused control
rather than unconditionally deleting. Escape closes an open selector before the
draft. Invalid choices show an error and disable Delete. Background edits, history,
file actions and window close are blocked; composition prevents opening the draft.
Existing palette, original icons, continuous canvas and Home spacing stay intact.
This explicit replacement workflow is a scoped Docs design, not a claim of exact
Word deletion behavior or Windows visual equivalence.

## Persistence and checks

Continue writing native v5 and strictly reading v1–v5. No tombstones, aliases or
new fields are needed; remaining references are valid v5 IDs. Save/reopen retains
the exact result, including unused survivors. Prior-file retention under injected
write/sync/replace failures remains required. A deletion changes the source
snapshot and invalidates an old Replace All preview.

Cover distinct paragraph/parent replacements, built-in children, No style,
indirect descendants, cycle rejection, unknown IDs, capacity/ID reuse, Unicode and
direct overrides, one exact undo/redo, dirty checkpoints, failed saves and native
roundtrip. Exercise actual controls, unused-style context targeting, keyboard,
Cancel/modal/IME guards and minimum-window geometry. Native Windows/DPI/IME and
matched reference acceptance remain separate gates.

Alpha.23 extension: [ADR 0010](0010-following-paragraph-styles.md) adds v6
following links; deletion remaps them to the paragraph replacement in the same
transaction, with counts and disclosure in the dialog. Earlier v5 text is historical.
