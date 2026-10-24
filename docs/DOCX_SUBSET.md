# DOCX profile — alpha.93

**DOCX is the primary/default document format.** `.bfdoc` is optional and can retain
all current model features. Primary status does not imply general Word compatibility:
many external DOCX files remain outside this restricted profile. Unsupported data
blocks opening or saving instead of being dropped. `.doc`, `.docm` and templates
are not supported.

## Supported read/write surface

| Element | Current support |
| :--- | :--- |
| Package | Stored/deflated ZIP, CRC/header checks and checked descriptors; safe internal relationship paths, no disk extraction |
| Text | Paragraphs/runs, escaped XML, preserved spaces and literal default-grid tabs; Unicode fixture |
| Character properties | Bold, italic, single underline and exact half-point size; explicit Arial declarations accepted, other families/themes rejected |
| Paragraphs | Alignment, before/after and proportional line spacing, left/right/first-line/hanging indents, keep rules and break before |
| Styles | Named paragraph/character graphs, stable exported IDs, parents and next references; unused built-ins may be absent |
| Imported defaults | Optional styles/defaults; supported non-baseline defaults become a named root style so inheritance remains editable |
| Lists | Decimal `1.` and bullets `•`, `◦`, `▪`; levels 0–8, independent streams and explicit starts/restarts |
| Tables | Fixed rectangular plain-cell profile, exact widths, empty/Unicode/multiline cells, first-row repeat flag; see strict boundary below |
| Page | One section, exact whole-point dimensions and margins |
| Relationships | Internal main document, styles and numbering; parts can use other locations; referenced external HTTP/HTTPS/mailto hyperlinks |
| Hyperlinks | Grapheme-aligned ranges, mixed runs, shared relationship targets and links in lists; no automatic activation |

Unspecified imported defaults use Arial 11 pt, single lines and zero paragraph
spacing/indents. This is an explicit import baseline, not a claim about every
application's fallback. Declared supported defaults override it. Canonical exports
retain the native default baseline in docDefaults and represent imported differences
through styles. Property order and namespace prefixes are not identities; canonical
built-in English name case is normalized. Unknown style IDs/cycles still fail.

Measurements must fit model precision/ranges exactly. Proportional lines map
between integer percent and OOXML's 240ths of a line. OOXML toggle-style combinations
that cannot preserve native absolute bold/italic inheritance remain rejected.
Every export is decoded and compared with the full model before writing.

## List boundary

Numbering stays in `word/numbering.xml` (or an internally referenced import path),
never as literal prefixes in document text. Native active levels use separate
numbering instances. Breaks, kind changes, ancestor resets and explicit restarts
create new streams with distinct abstract definitions, avoiding accidental OOXML
counter resets and Word continuation across shared definitions. Import checks
instance references, levels, starts and sequential counter behavior.

The current list profile requires unmodified 11 pt paragraph font properties and
no paragraph indent overrides. Each level declares a 24 pt inset plus 18 pt per
level and an 18 pt hanging indent; labels use decimal `%n.` or the native bullet
for that level, left justification and a tab suffix. Direct run emphasis is retained.
Other list indentation, Roman/letter/compound formats, picture/symbol-font bullets,
style-based numPr, arbitrary level overrides and lvlRestart policies are rejected.
Linked numbering continued after ordinary paragraphs or another independent
instance is rejected when the native sequential model cannot retain its behavior.
There are at most 1,024 exported instances; counters fit 1–65,535.

## Hyperlink boundary (alpha.92)

`w:hyperlink` contains ordinary supported runs and references an external hyperlink
relationship by namespace-qualified `r:id`. HTTP, HTTPS and mailto targets follow
existing model validation (at most 2,048 bytes, no spaces/control characters).
Import never fetches or activates a target. Internal package parts still require
internal relationships; external images/templates are not enabled.

Text order, per-run formatting, Unicode/grapheme ranges and distinct adjacent links
survive native/DOCX round trips. Reused relationship IDs may serve several ranges;
export assigns each range its own ID. Empty ranges, missing/duplicate IDs, unused
hyperlink relationships, nested links and grapheme-splitting boundaries fail.
Bookmarks/anchors, fields, tooltips, target frames, document locations and history
flags remain unsupported and are rejected, including explicitly written defaults.
Colors and undeclared built-in Hyperlink styles remain outside the style profile;
no automatic visual style is added during exchange.

Reproduce with `cargo run --locked --no-default-features --example docx_links`.
This writes the synthetic `examples/DocxLinks.docx` / `.bfdoc` pair and asserts
full-model equality. External links are not followed by this check.

## Table boundary (alpha.93)

Tables map to model-owned atomic table blocks, not tab-separated paragraph text.
The initial profile requires 1–100 rows, 1–12 columns, widths of 18–720 whole points,
no more than 50,000 cell characters and total width within the page text area.
Grid, total and individual cell widths must agree exactly. A table can be the only
body block; tables reset the native sequential list stream.

The model has no arbitrary cell formatting storage. Therefore import requires an
explicit fixed plain-cell presentation: left-aligned fixed layout, six single
0.5 pt `9B9B9B` grid borders, 4 pt cell margins, non-splitting rows, and at most the
first row marked as a repeated header. Cells contain one paragraph with explicit
zero spacing/indents, single lines, left alignment and explicit Arial 11 pt runs,
no italic/underline, bold only for the repeated header. Text, empty cells, preserved
spaces, literal tabs and line breaks survive. Multiple identical-format runs are
accepted; other formatting is not flattened. This deliberately excludes most
arbitrarily styled external tables until the model can represent their properties.

Merged cells, row spans, nested tables, multiple cell paragraphs, rich text, links,
images, table styles, custom border/margin/alignment, auto-fit and row heights fail.
Table-object paragraph style/format/flow overrides also block export rather than
disappear. Native v19 remains unchanged; optional `.bfdoc` retains those model
properties. Fixed native and Word renderers can differ in line metrics/pagination;
no pixel-identical table layout is claimed.

Reproduce: `cargo run --locked --no-default-features --example docx_tables`, then
open `examples/DocxTables.docx`. The generator writes the native counterpart and
asserts exact full-model reopen. On Windows use
`cargo run --locked -- --open (Join-Path (Get-Location) 'examples\DocxTables.docx')`.
Native Windows checks remain deferred.

## Explicit rejection rather than loss

Images, multiple sections, headers/footers, page numbering,
explicit/style tab stops, revision marks, comments, fields, colors and unsupported
fonts/themes are still rejected. Additional unaccounted parts, including document
properties/theme/settings parts, are not silently stripped. No lossy bypass or
hidden native sidecar is included in DOCX.

Limits: 32 MiB compressed/total expanded, 8 MiB per part, 128 ZIP entries, XML depth
64 and 200,000 nodes per part, plus existing model limits. ZIP64/encrypted/split
archives, unsafe/duplicate paths, overlapping payloads, CRC failures, external
relationships other than supported hyperlinks, DTDs and custom entities fail. Relationship traversal cannot escape
the package root; percent-encoded/external targets are rejected. Nothing is fetched
or executed. A styles part and document relationships part are optional when unused;
any declared relationship must resolve with the proper content type.

## Open and save behavior

- New Save proposes `Untitled.docx`; Open/Save As list DOCX first, `.bfdoc` second.
- File → Open and `--open <absolute.docx>` share validation and retain the path.
- Save updates the selected file only after successful conversion. Existing
  `.bfdoc` files retain that explicit format; Save As changes it deliberately.
- Unsupported DOCX data leaves destination, dirty checkpoint and history unchanged.
  Choose optional `.bfdoc` explicitly to preserve features outside this profile.
- A loaded DOCX keeps a byte snapshot. Saving to that source detects external
  changes, including canonical-path aliases, and asks for Save As. This is a
  best-effort pre-write check, not an atomic multi-process lock or crash recovery.
- File → Export DOCX remains an explicit copy operation, preserving the current
  save destination and history. Saves reuse checked temporary-file replacement.
- `--capabilities` lists DOCX first and native second using the existing v1 schema;
  read/write flags describe available operations, not complete OOXML support.

## Reproduce

```sh
cargo run --locked --no-default-features --example docx_lists
cargo run --locked -- --open "$(pwd)/examples/DocxLists.docx"
cargo test --locked docx::
```

Windows PowerShell commands (native Windows validation remains deferred):

```powershell
cargo run --locked --no-default-features --example docx_lists
cargo run --locked -- --open (Join-Path (Get-Location) 'examples\DocxLists.docx')
```

The generator writes the checked-in DOCX/native fixture pair and asserts exact
model/label equality on reopen. Tests also assemble independently authored XML
with relocated parts, absent styles and start-at-3 numbering; they do not rely only
on self-produced files. See [validation](VALIDATION.md#alpha91-primary-docx-and-list-validation)
for actual external/native checks. External save-back remains unverified.

## Next D4 increments

Expand ordinary external defaults/fonts and metadata preservation, then
broader tables/PNG, sections/headers/footers and tabulation. Extend independent fixtures and
save-back checks before preparing tested-target archives and reviewing distribution
notices. D4 remains in progress; no tag, merge or public alpha package is authorized.
