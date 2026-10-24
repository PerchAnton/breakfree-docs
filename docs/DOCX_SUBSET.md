# DOCX profile — alpha.95

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
| Character properties | Bold, italic, single underline and exact half-point size; Arial, Times New Roman and Courier New family overrides; other families/themes rejected |
| Paragraphs | Alignment, before/after and proportional line spacing, left/right/first-line/hanging indents, keep rules and break before |
| Styles | Named paragraph/character graphs, stable exported IDs, parents and next references; unused built-ins may be absent |
| Imported defaults | Optional styles/defaults; supported non-baseline defaults become a named root style so inheritance remains editable |
| Lists | Decimal `1.` and bullets `•`, `◦`, `▪`; levels 0–8, independent streams and explicit starts/restarts |
| Tables | Fixed rectangular plain-cell profile, exact widths, empty/Unicode/multiline cells, first-row repeat flag; see strict boundary below |
| Page | Next-page sections, exact whole-point dimensions/margins, plain headers/footers and PAGE restarts |
| PNG | Inline atomic images, exact pixels/whole-point size/alternative text; no transforms |
| Tabulation | Explicit left/center/right positions, dot/dash/line leaders and style/direct replacement lists |
| Metadata | Original bounded core/app/custom properties retained verbatim; counters may be stale after edits |
| Relationships | Internal main document, styles, numbering, images, headers/footers and core/app properties; parts can use other locations; referenced external HTTP/HTTPS/mailto hyperlinks |
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
disappear. Native v20 retains the prior table model; optional `.bfdoc` retains those model
properties. Fixed native and Word renderers can differ in line metrics/pagination;
no pixel-identical table layout is claimed.

Reproduce: `cargo run --locked --no-default-features --example docx_tables`, then
open `examples/DocxTables.docx`. The generator writes the native counterpart and
asserts exact full-model reopen. On Windows use
`cargo run --locked -- --open (Join-Path (Get-Location) 'examples\DocxTables.docx')`.
Native Windows checks remain deferred.

## Explicit rejection rather than loss

Revision marks, comments, non-PAGE fields, colors, unsupported fonts/themes,
rendering settings remain errors. Non-default periodic
tab intervals, additive/partial tab inheritance, first/even headers, continuous
sections, rich headers/footers, floating images, crop/effects and merged/rich cells
are not silently flattened. No lossy bypass or hidden native sidecar is included.

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

## Combined profile boundaries (alpha.94)

PNG must be an isolated paragraph object with no authored paragraph overrides.
The package uses the canonical inline DrawingML structure, no crop/transform,
whole-point display dimensions and checked pixels (4 MiB, 16 MP, 8192 pixels/side).
Names/IDs are not model identities; alternative text is retained. Unrepresented
drawing properties fail even when another application writes equivalent defaults.

Sections begin after a text paragraph, never at document offset zero. A section
immediately after an object is not representable in this profile; insert a text
paragraph explicitly. Header/footer text is one line, up to 200 characters; the
canonical auxiliary paragraph is Arial 9 pt with a right tab for PAGE. Distances
are half the corresponding page margin. First/even/linked rich variants and
noncanonical auxiliary formatting remain outside the model. Imported missing
later references inherit previous text; exported empty text gets its own part.
PAGE caches are canonical placeholders; the viewer computes actual numbering.

Tab positions use exact half points and the model limits. Default periodic stops
remain 36 pt. Style/direct list overrides emit explicit clear records for removed
parent positions, retaining empty versus inherited lists. Unsupported periodic
overrides must use optional .bfdoc.

Font family names are retained through paragraph/character/direct inheritance and
can be selected in the Font dialog. Installed fonts are used where found; local
fallback can change metrics. No host fonts are bundled, no exact Word pagination
is promised, and font provenance inspection remains limited to existing properties.

Core/app/custom XML is retained unchanged, bounded to 128 KiB each and namespace-validated.
Its imported page/word counters may be stale; the editor's displayed counts come
from the current model. Metadata is never interpreted as instructions or fetched.

Reproduce: `cargo run --locked --no-default-features --example docx_extended`.
The combined pair exercises Unicode, fonts, tabs/leaders, lists, tables, PNG,
portrait/landscape sections and number restart. See [validation](VALIDATION.md).
Local archive preparation is documented in [REVIEW_PACKAGE](REVIEW_PACKAGE.md).

## Remaining D4 acceptance

External save-back remains unavailable in the installed read-only Word. Broader
external tables/themes/settings/fonts need additional model design and fixtures,
not a silent compatibility switch. A local unsigned review archive is not a
published alpha: license review, tested-target acceptance and owner publication
approval remain gates. D4 remains in progress.


## Alpha.95 property and field extension

Custom properties are an internally referenced root-package part with the standard
custom-properties relationship/content type. Preserve its original XML verbatim,
with a validated namespace/root and existing bounded parser; do not interpret
values, fetch linked content, expose a property editor or execute instructions.
Duplicate roles, unsafe paths, external metadata relationships, malformed XML and
oversized parts remain errors. Native v21 retains this passive data.

A canonical PAGE field may contain an updated decimal cache from 1 through 65535.
The cache is derived display data; section numbering remains the authored model
value. Export writes a placeholder for the viewer to recompute. Arbitrary field
instructions and nondecimal/unsupported caches remain rejected.

Reproduce the property pair with:
cargo run --locked --no-default-features --example docx_properties.
No external save-back acceptance is implied by the adapter regression.


## Alpha.96 external corpus

The [corpus report](DOCX_CORPUS.md) records actual raw external inputs and save-back
outcomes. Cocoa preserves the tested text/emphasis/alignment projection but its
DOCX output still fails import on unsupported package relationships. No profile
restriction was relaxed to accommodate those samples. Diagnostics include the
relationship part, ID and full type; no relationship target is fetched.
