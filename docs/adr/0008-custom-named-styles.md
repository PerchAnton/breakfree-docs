# ADR 0008 — custom named paragraph styles

Date: 2026-10-18. Status: implemented for review in alpha.20.
Extends [ADR 0007](0007-editable-style-definitions.md); no new dependency.

## Identity and validation

Retain the three built-in IDs and definitions. A document may additionally own up
to 64 custom paragraph styles. Each has a positive, document-local `u32` ID, a
Unicode display name and the same `StyleDefinition` as built-ins. References use
IDs, never names. Allocate the first unused ID in a candidate catalog; an undone
creation may reuse its ID only when a new command also discards the old redo branch.

Names contain 1–64 Unicode scalars, no leading/trailing whitespace, control
characters or U+2028/U+2029 separators. Reject duplicates after Rust Unicode
lowercasing, including built-in display names. Do not trim, normalize or truncate
stored names. This is not full case folding or canonical-equivalence matching;
visually equivalent composed/decomposed names can remain distinct.

Parents can be built-in, custom, or absent. Validate the entire graph, unique IDs,
name rules and all paragraph references before accepting a command/file. Reject
missing parents and direct/indirect cycles; never substitute an unrelated style.
Resolution remains root → parent → child → direct overrides, with the ADR 0007
fallback and numeric limits. Normal may inherit from a custom style only if the
graph remains acyclic. Delete and character styles are not implemented.
Alpha.21 adds renaming as described below.

## Command and interface

`Editor::create_style(name, definition, apply)` constructs and validates a cloned
candidate, then commits exactly once. With `apply`, set the style on all intersected
paragraphs, excluding a paragraph whose start is the exclusive selection end.
A caret targets its paragraph, including an empty one. Without `apply`, preserve
all paragraphs. Keep text, alignment, direct overrides, selection and pending
formatting. Undo removes the definition and restores references together; redo
restores both. Failure preserves dirty state, document and history/redo.

Home Styles/More → Create style, command search and provisional H → L → C or
H → E → C KeyTips open a local draft. Focus the name. Initially inherit from the
first selected paragraph's style with no copied direct overrides. Offer parent,
tri-state emphasis, size/spacing, character preview and an initially checked
Apply to selected paragraphs option. Empty/duplicate/invalid names disable OK;
Cancel/Escape makes no change. Reuse Modify Style's modal/IME/close guards and
layered popup Escape. Modifying a custom definition uses the existing command.

All styles appear in a bounded, keyboard-scrollable menu and command search.
A selected custom style occupies the third wide gallery card (or the sole compact
card). Long names are clipped/truncated for layout; full names remain in tooltips
and accessibility labels. The 64-style limit disables Create with an explanation.
Palette, vector icons, logo and Home group gaps stay unchanged.

The [official styles workflow](https://support.microsoft.com/en-us/word/customize-or-create-new-styles)
was inspected on 2026-10-18. This increment deliberately starts from an explicit
parent, not a snapshot of direct selection formatting. Templates, mini-toolbar,
font family/color, following style and complete Windows parity remain outside scope.

## Persistence

Content v5 adds a required `custom` array to `style_catalog`. Each entry requires
`id`, `name`, `definition`. Custom references serialize as `{"custom": 1}`;
built-in strings and run overrides keep their existing representation. Typed
strict decoding rejects missing/duplicate/unknown fields, invalid graphs, IDs and
names. Definitions without paragraph references are still saved.

Retain separate strict v1–v4 readers. V4 carries its three definitions into a
catalog with no custom entries; v3 uses frozen defaults and v1/v2 retain explicit
legacy appearance. Opening leaves source bytes unchanged; explicit save writes
v5. Alpha.19 and older reject v5; use Save As to preserve an older copy. Protocol
and preferences remain v1; checked-write guarantees do not change. This adds no
power-loss recovery, cloud dependency or external style engine.

## Alpha.21 renaming

`Editor::modify_custom_style(id, name, definition)` validates a cloned full catalog
and document before one commit. A pure name change is labelled Rename style;
combined name/parent/property changes use Modify style. Change only the target
name and requested definition, never its ID or paragraph/parent references.
Preserve selection and pending direct formatting. Reject built-in/unknown IDs and
all existing name/graph/value errors without changing document, dirty state or
history. A true no-op preserves redo; a case-only rename is a real change. The
64-style capacity limits creation, not updates to existing definitions.

Modify Style exposes an initially selected editable Name for custom IDs only.
Built-in names remain read-only. The same draft validates name, parent and fields;
OK commits all of them together, while Cancel/Escape changes nothing. Gallery and
Styles-list context menus can modify any listed style without first applying it.
The context menu is rendered independently of the source popup so that an unused
style remains a valid target when the Styles list closes. Search, tooltips,
gallery and future parent lists read current catalog names, not cached aliases.

Content v5 already separates IDs and names, so no format bump or migration is
needed. Pure rename must leave the rendered paragraph layout unchanged. Test
exact undo/redo, descendant/paragraph references, names at validation boundaries,
combined failure rollback, capacity, saved checkpoints, stale bulk previews and
save failure retention. Rename does not imply support for deletion or aliases.
