# ADR 0013 — Direct paragraph spacing

- Date: 2026-10-18
- Status: implemented for owner review in alpha.41
- Scope: independent Rust model/commands, layout, native v10 and Home spacing draft

## Decision

Add optional direct line-height percentage and space-after half points to each
paragraph. Resolve paragraph-style inheritance first, then these overrides. Null
inherits; explicit zero after is retained. Use the same limits as style spacing:
100–300% and 0–200 pt in half-point steps. Alignment remains independent.

A shared partial command distinguishes keep, set and inherit for each property.
Validate range, grapheme boundaries and the candidate document before committing.
Apply to intersecting paragraphs, excluding a next paragraph at an exclusive end;
a caret and an empty paragraph are valid targets. All changes share one history
entry; no-ops preserve redo. Character formatting and style catalogs are untouched.

The modal changes only checked properties. Editing checks the corresponding
property; blank means inherit. Use style spacing checks and clears both fields.
Cancel/Escape do not edit; Cancel starts focused. Background commands/KeyTips are
blocked while the draft is open. H → PS is a Docs-specific provisional KeyTip.
The original Home spacing icon becomes active without changing palette/geometry.
Formatting inspection identifies each direct spacing source separately.

## Text edit policy

Enter, pasted paragraph breaks and replacement-created paragraphs copy the first
intersected paragraph's spacing, like its alignment. Following-style selection
still changes the semantic style; explicit spacing remains. Joining paragraphs
keeps the first paragraph's spacing. Replace All carries original spacing across
untouched paragraph boundaries and copies current spacing for inserted breaks.
Snapshots restore every value on undo/redo.

## Persistence and limits

Native v10 requires a spacing object per paragraph. Strict frozen v1–v9 readers
initialize inherited values; they do not infer overrides from rendered spacing.
Unknown fields/versions/ranges fail before changing a source file. The existing
temporary-file write/sync/replace implementation and failure tests remain in use.
Alpha.40 and earlier reject v10 saves; use Save As to retain an older file.
Process/settings versions remain 1. No new dependency or project-license change.

Indentation, tabs, exact line-height units and pagination are
separate increments. Native Windows/reference acceptance is deferred by owner
instruction; synthetic DPI tests do not close that gate.

## Alpha.42 extension

Add optional space-before to paragraph style definitions and direct spacing, using
the same half-point range as space-after. Nearest ancestor supplies the value;
document default is zero. Direct zero is an override. A three-property command
preserves keep/set/inherit semantics in one validated history transaction. Dialog
reset now clears all three properties. Native v11 carries the new key; older
readers reject it and retain their strict typed validation. Continuous layout adds
before-current to after-previous, including before the first paragraph, once per
paragraph regardless of wrapping. This does not establish pagination behavior.

## Alpha.43 extension: horizontal indents

Store left/right optional half-point indents alongside paragraph geometry in the
existing spacing object and paragraph style definition. Limit each to 0–144 pt;
resolve independently through ancestors/default zero/direct override. Keep/set/
inherit patches share a validated command and one history transaction with spacing.
Existing Enter, join and Replace All geometry copying includes these fields.
Native v12 adds keys; v1–v11 reject them, even null. No dependency/license change.

The layout adapter subtracts both indents before wrapping/justification, then shifts
rendered glyphs and cursor geometry by the left indent and alignment offset. On
narrow viewports, scale indents proportionally to reserve up to 32 DIP for text,
without editing the model. First-line/hanging indents, negative outdents, ruler
handles and page-layout width constraints require separate decisions and tests.

## Alpha.44 extension: first-line and hanging indents

Add one signed optional `first_line_half_points` property to style/direct geometry,
range -288…288. Positive insets only the first row; negative insets continuation
rows, keeping the first at the left indent. Left marks the outermost edge. A blank
amount inherits both type/magnitude, zero cancels them. This representation avoids
conflicting first-line and hanging values. One validated partial command includes
the property; reset/history and editing preservation follow the existing policy.
Native v13 adds the key and v1–v12 reject it even when null.

Shape the paragraph at the first-row width to find its initial wrap boundary, keep
that row, and shape its suffix at continuation width. Keep run formats and scalar
indices; concatenate without synthesizing a newline between these portions. Use
the real first/continuation widths for alignment and justification. Rebuild first
portion metrics and retain only its real mesh counts. Scale all horizontal insets
proportionally in narrow views. This does not add page layout or ruler handles.

The type/amount UI and continuous-layout geometry are original implementations.
Matched native Windows/Word acceptance remains deferred; no equivalence claim.

## Alpha.45 correction: logical whitespace extent

Justification measures visible ink without trailing spaces. Preserve this visual
measure for alignment, then extend the logical row width through the last stored
glyph. Otherwise a cursor at a soft-wrap end maps back before its trailing space.
Do not move or remove glyphs, edit document text or change file semantics. Logical
caret/selection extent may exceed the visible justified edge through whitespace.
The regression matrix checks text reconstruction, grapheme boundaries in tested
examples, real paragraph delimiters, mesh counts, heights and cursor mapping.
