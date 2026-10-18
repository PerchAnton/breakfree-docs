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

Space-before, indentation, tabs, exact line-height units and pagination are
separate increments. Native Windows/reference acceptance is deferred by owner
instruction; synthetic DPI tests do not close that gate.
