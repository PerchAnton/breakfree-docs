# ADR 0007 — document-owned style definitions

Date: 2026-10-18. Status: implemented for review in alpha.19.
Extends [ADR 0006](0006-built-in-paragraph-styles.md); no new dependency.

## Decision

Keep the three stable paragraph IDs (Normal, No Spacing, Heading 1), but move their
parent and formatting definitions into each document's `StyleCatalog`. Names and
IDs remain fixed. Creating, renaming or deleting styles and character styles are
separate increments. No template/global default is changed by Modify Style.

Each definition contains an optional parent, character overrides, line-height
percentage and space-after half points. Missing/null optional properties inherit.
Resolution is root → parent → child → direct character overrides. A root without
a value uses the frozen baseline: regular 11 pt, no underline, 125% line height,
5.5 pt after. Initial definitions retain alpha.18 appearance. All styles may have
no parent. Normal may acquire a parent only when the entire graph stays acyclic.

Reject self/indirect cycles, unknown IDs, line height outside 100–300%, space after
outside 0–200 pt (half points) and invalid font sizes. The shared command validates
the complete candidate before committing. The resolver is also bounded for invalid
in-memory input; validation is still mandatory before accepting/editing/saving it.

`Editor::modify_style` replaces the definition in one snapshot operation, including
the parent and all properties. Preserve text, paragraph references/alignment,
direct overrides, selection and pending input. Descendants resolve dynamically;
no character values are flattened. A semantic no-op preserves undo/redo. Text and
batch replacements carry the catalog forward; a definition change invalidates an
older Replace All preview. Session dirty comparison includes definitions.

## Interface

Home Styles → Modify style edits the first selected paragraph's style (also the
caret's style); right-click a gallery card to modify that definition without
applying it. Command search and provisional KeyTips H → L → M (wide/compact) or
H → E → M (minimum width) use the same action. The dialog names its fixed target.

The local draft exposes Based on, tri-state bold/italic/underline (Inherit/On/Off),
size, line spacing and space after. Blank numeric fields inherit. Invalid values
or cycles show an error and disable OK; a character preview reflects the validated
draft. The preview does not simulate paragraph spacing. OK applies once; Cancel
or Escape discards. Open combos handle their own keys first. Background edits,
history, file actions and native window close are blocked while the draft is open.
IME guards prevent opening it during composition. Gallery previews reflect the
resolved font, bounded to 12–20 DIP so large document sizes cannot expand cards.
Palette, vector icons and Home gutters remain unchanged.

The [official Modify Style workflow](https://support.microsoft.com/en-us/word/customize-or-create-new-styles)
informs this design. This prototype exposes only supported properties, applies to
the current document, and makes no exact Windows visual/keyboard parity claim.
Alignment still belongs to paragraphs; font family, colors, following-paragraph
style, templates and auto-update are not implemented by this dialog.

## Persistence

Content v4 requires a `style_catalog` object with exactly the three known entries.
Each entry requires `character`; parent and numeric overrides may be absent/null.
The full graph is validated before a document is accepted. Typed strict decoding
rejects duplicate/unknown fields and missing catalog entries. The run/paragraph
schema stays as v3. The writer emits every definition, not resolved run caches.

Read v3 with its frozen default catalog; retain strict v1/v2 readers and explicit
legacy character overrides. Opening older files never rewrites them. Explicit save
writes v4; alpha.18 and earlier reject it. Save As retains the original source.
Process contract and preferences remain v1. Existing checked-write guarantees and
limits remain unchanged. No claim of crash recovery or universal atomicity.
