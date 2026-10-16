# ADR 0003 — Character size, underline and native content v2

- Date: 2026-10-16
- Status: implemented for owner-requested alpha.3 review
- Scope: two direct-formatting properties; no GUI-toolkit or license change
- Extends ADR 0002; replaces its fixed-size/native-v1-only limitation

## Decision

Keep underline and font size in the GUI-independent per-scalar `Style`. Use a
validated `FontSize` value in integer half-points: 1–400 pt, default 11 pt. This
prototype limit is not a claim about another editor's accepted range. The field
accepts whole or half points using a decimal dot. No float enters persisted state.

Shared range commands validate grapheme boundaries, preserve unrelated style
properties and retain selections. A selected-range change is one history operation;
no-op commands add none. A collapsed caret changes pending insertion style without
dirtying the document or consuming history. Inserting/replacing text uses that
style; undo/redo snapshots preserve the pending style as well as document/selection.
Empty-paragraph character formatting remains pending input state, not a persisted
paragraph default. Named styles, font-family selection and other effects stay out
of scope. Mixed selected sizes display an empty field instead of a fabricated size.

The existing layout adapter projects actual run sizes and underline strokes into
the same galley used for selection and hit testing. Document zoom scales rendering
only. Home's editable size field and presets, Ctrl/Command+U, Ctrl/Command+Shift+P,
command search and the formatting pane use shared editor commands. Enter or blur
commits valid size input; Escape cancels. Save dispatch commits pending size input
first. Invalid sizes show an error and leave the document/history intact.

## Persistence and compatibility

Native content v1 has a strict schema with only bold/italic runs. Adding fields
under that version would misrepresent compatibility. Write content v2 with required
`underline` and `font_size_half_points` fields; retain a separate strict v1 reader
that supplies false/11-pt defaults. Reading never changes the source. Only an
explicit successful save writes v2, including when the source was v1. File view
and documentation disclose that older v1-only builds cannot open new saves.

Deserialize the header to select the schema, then the full typed v1/v2 structure.
Do not deserialize through a generic JSON value that would erase duplicate keys.
Reject unsupported versions, unknown/missing/duplicate fields and invalid sizes.
There is no v2-to-v1 export or lossy fallback. Keep write-then-replace and failure
preservation unchanged. The process protocol, app ID and `.bfdoc` extension stay v1,
`breakfree.docs` and `.bfdoc` respectively; file and application versions are separate.

## Verification and limits

Engine/storage tests cover exact style/history snapshots, Unicode range rejection,
mixed/pending styles, v1 read-only conversion, strict v2 parsing and failed saves.
Actual widget-event tests cover size entry/cancel/errors/presets, underline shortcuts,
selection retention, styled IME commits, save/reopen and rich-layout hit testing.
See [validation](../VALIDATION.md) for results actually obtained. Native Windows
input/DPI, accessibility and a controlled Word 2024 comparison remain open.

## Alpha.4 Home extension

The owner's next Home refinement adds Grow/Shrink Font using the same validated
range transaction. Each selected run steps independently to the next/previous
preset (8, 9, 10, 11, 12, 14, 16, 18, 20, 22, 24, 26, 28, 36, 48, 72 pt).
Outside that preset interval, move one point and clamp at 1/400 pt. Preserve half
points outside the interval. This is an explicit Docs policy, not a claim about
all reference-editor edge cases. The entire selection changes in one undo entry;
no-op limits add none. Caret-only changes remain pending and preserve other style
properties. Buttons, command search and Ctrl/Command+Shift+>/< call this engine
command. No model schema, content version or integration protocol change is needed.
