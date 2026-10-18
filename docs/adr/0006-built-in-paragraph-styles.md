# ADR 0006 — Built-in paragraph styles and direct overrides

- Date: 2026-10-17
- Status: implemented for owner review in alpha.18; extensible catalog editing remains deferred
- Scope: shared Rust model, commands, native content v3 and Home gallery; no new dependencies

Alpha.19 extends this historical fixed-catalog decision through
[ADR 0007](0007-editable-style-definitions.md): editable document definitions and
content v4. The frozen v3 defaults remain the migration baseline.

## Decision

Separate resolved `Style` values from stored `CharacterFormat` overrides. Each
scalar stores optional bold, italic, underline and size. `None` inherits the
paragraph style; `Some(false)` explicitly turns a property off. Resolution happens
in the engine model and is shared by selection reporting and the layout adapter.
Do not flatten inherited properties into the document when applying a style.

Each paragraph, including an empty one, references a typed built-in style:

| Style | Parent | Character overrides | Line height | Space after |
| :--- | :--- | :--- | :--- | :--- |
| Normal | Root | Regular 11 pt, no underline | 1.25 × | 5.5 pt |
| No Spacing | Normal | Inherit | 1 × | 0 pt |
| Heading 1 | Normal | Bold, 18 pt | Inherit | Inherit |

These are BreakFree prototype definitions, not claims about a Word template's
exact defaults. Parent definitions resolve first, then the child, then direct
character overrides. Paragraph alignment remains an explicit independent value.
The fixed enum catalog cannot contain unknown parents or cycles. Custom/character
styles, editable definitions, themes, outline navigation, keep-with-next and
following-paragraph rules require a later decision and versioned schema.

## Commands and input

Apply a style to paragraphs intersecting the half-open selection, or the caret's
paragraph. An exclusive end at the next paragraph start excludes that paragraph.
Preserve text, alignment, direct overrides, selection and pending direct input;
record one snapshot operation. Reapplying the same style adds no history.
Clear character formatting removes direct overrides; each run resolves against its
own paragraph style. At a caret this changes only pending input. Grow/Shrink and
Font dialog read resolved values and write explicit overrides. Even a visually
unchanged explicit override is a real semantic change and can consume undo.

Typed/split/newline paragraphs inherit the first affected paragraph's style and
alignment, including Heading 1; there is no automatic return to Normal on Enter.
Merged paragraphs likewise use the first paragraph's properties. Unaffected
paragraphs keep theirs. Replace All preserves style references on untouched
separators; inserted separators inherit the current paragraph, and replacement
characters use the first matched character's direct overrides. Preview remains
read-only and stale semantic changes invalidate it. History retains semantic data.
Empty paragraphs persist their named style, but pending direct input stays local.

## Persistence and migration

Write content v3 with explicit paragraph IDs and a `direct` object per run. Read
v1/v2 through their unchanged strict schemas and import their resolved values as
explicit overrides on Normal. Earlier files cannot reveal authored inheritance;
do not guess it. Opening changes no source bytes and does not mark the session
dirty. Applying a style to imported text preserves those explicit values; Clear
character formatting makes the text inherit the new style. Explicit save writes
v3, which older builds reject. Save As preserves the old source. See
[NATIVE_FORMAT](../NATIVE_FORMAT.md). Process protocol and preferences stay v1.

## Interface and review limits

Home's existing gallery becomes functional without changing the palette/icons or
8-DIP group gaps. Full width shows three cards; compact width shows the current
uniform style (Normal when mixed) and an all-styles menu. Mixed selections have no
selected card. Minimum width uses More. Menus, keyboard traversal, sequential
KeyTips and command search use the same actions. No live hover preview is used.
Modify style is explicitly disabled. Empty-paragraph rendering uses its style and
only the active caret's pending overrides.

Official reference: [Home Styles gallery](https://support.microsoft.com/en-us/word/add-and-remove-styles-from-the-quick-styles-gallery).
This validates the gallery concept, not exact Windows geometry or shortcut parity.
Native Windows/Word paired captures, 100/125/150% DPI, IME and assistive technology
remain acceptance gates. No license, branding or Office contract change.

Layout note: alpha.18 also fixes a previously ignored paragraph gap. Legacy
character values and alignment are preserved; corrected spacing can change the
vertical positions compared with an earlier renderer. This is a layout bug fix,
not a stored-text conversion.
