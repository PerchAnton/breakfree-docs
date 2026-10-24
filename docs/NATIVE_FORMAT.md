# Native BreakFree Docs document format

Extension `.bfdoc`; type `breakfree.docs.native`. Current writer: content v20.
This is not a DOCX container. Process protocol and file-content versions are independent.

```json
{
  "format": "breakfree.docs.native",
  "content_version": 18,
  "character_styles": [],
  "style_catalog": {
    "custom": [],
    "normal": {"parent": null, "character": {}, "line_height_percent": null, "space_after_half_points": null},
    "no_spacing": {"parent": "normal", "character": {}, "line_height_percent": 100, "space_after_half_points": 0},
    "heading1": {"parent": "normal", "character": {"bold": true, "font_size": 36}, "line_height_percent": null, "space_after_half_points": null}
  },
  "paragraphs": [
    {
      "alignment": "left",
      "paragraph_style": "heading1",
      "spacing": {"line_height_percent": null, "space_after_half_points": 0},
      "runs": [{"text": "Hello", "character_style": "emphasis", "direct": {"bold": null, "italic": true, "underline": null, "font_size": null}}]
    },
    {"alignment": "center", "paragraph_style": "normal", "spacing": {}, "runs": []}
  ]
}
```

A document has at least one paragraph; an empty paragraph has `runs: []`.
`paragraph_style` is required: `normal`, `no_spacing`, `heading1` or `{ "custom": 1 }`.
The required `style_catalog` holds the three built-in definitions and a required
`custom` array (empty when unused). Each built-in definition requires a `character`
object; optional/null parent, character properties and spacing inherit. Validate
all parents/cycles before accepting the file. Line height is 100–300 percent;
space after is 0–400 half points. Root fallback is regular 11 pt, 125%, 5.5 pt after.
See [ADR 0007](adr/0007-editable-style-definitions.md).

Each custom entry is `{ "id": 1, "name": "Report", "definition": { "parent":
"normal", "character": {} } }`, using the same optional properties and limits.
IDs are positive unique u32 values, names 1–64 Unicode scalars with no outer
whitespace, controls or U+2028/U+2029. Names are unique after Unicode lowercasing
against all custom and built-in display names; no normalization/full case folding.
Up to 64 custom entries are permitted, including unused definitions. Validate all
references and cycles, including built-in/custom parent chains. See
[ADR 0008](adr/0008-custom-named-styles.md).
Each nonempty run requires text and a `direct` object. Its four properties are
optional or null for inheritance; explicit booleans override bold/italic/underline.
`font_size`, when present and non-null, is an integer from 2 through 800 half points
(1–400 pt). False is distinct from inherited; 22 is an explicit 11 pt override.
The writer emits all four properties, including nulls. No resolved-style cache is saved.
Alignment is `left`, `center`, `right` or `justify`, independent of the named style.
Paragraph text cannot contain LF. Paragraphs join with LF for scalar edit indices.
Adjacent identical character references and direct overrides compact on write, without Unicode normalization.

Reject unknown fields at every level, duplicate keys, unknown style IDs, foreign
types, unsupported versions, invalid UTF-8/values and resource-limit violations.
Limits: 16 MiB encoded; 100,000 scalars including paragraph separators. There is no
lossy fallback. Read failures never modify the source or current GUI session.

## Reading earlier files

Separate strict readers retain the original v1/v2 schemas. Both have paragraphs
with alignment and runs, without paragraph_style/direct. V1 runs require exactly
text/bold/italic; missing size/underline resolve to 11 pt/off. V2 runs additionally
require underline and font_size_half_points (integer 2–800); missing/null values
are rejected. The [v1](../examples/Welcome.bfdoc) and [v2](../examples/Formatting.bfdoc)
fixtures stay byte-identical.

Import every legacy character value as an explicit override on Normal, preserving
appearance without guessing authored inheritance. Applying a named style retains
these values; Clear character formatting makes the text inherit its paragraph.
Opening performs only in-memory conversion and does not dirty/write the file.
V3 uses its frozen built-in catalog from [ADR 0006](adr/0006-built-in-paragraph-styles.md);
it rejects embedded definitions. V4 reads its original three-definition catalog
with no custom array and preserves its graph. V5 retains its custom catalog. Legacy definitions receive Same style in memory.
Every explicit save writes v13, even
for an unchanged older document. Use Save As
to retain the original. **Alpha.43 and earlier cannot open v13 saves.** No downgrade or
lossy export is provided. Header/full-schema parsing preserves duplicate rejection.

Empty paragraphs retain their named style. Pending direct formatting at a caret
is session state until text is inserted. Application version, process protocol v1
and toolbar settings v1 are independent of this content version.

## Save transaction and exact guarantees

1. Validate extension and serialize the model completely in memory.
2. Reject symbolic-link destinations. Create a uniquely named temporary file in
   the destination directory; never open/truncate the old destination for writing.
3. Write all bytes with Rust `write_all`, flush and `sync_all` the temporary file.
4. Preserve the existing destination's permissions where available, then call
   `NamedTempFile::persist` to replace the destination. Only after success update the session's path and
   clean checkpoint.
5. On pre-replacement failure, report the error and remove the temporary file when
   possible. The old destination and in-memory edited document remain available.

Rust tests inject a failure after a partial temporary write and before file sync
and replacement; they check previous bytes and dirty checkpoints. Real replacement
failure (destination directory) and missing parent directories are also tested.
Permission-setting errors propagate; Rust has no injected permission-failure case. This implements write-then-replace, **not a claim
of power-loss recovery or universal filesystem atomicity**. There is no directory
fsync, backup, journal, concurrent-writer protection, locking or autosave. A killed
process can leave an orphan temporary file; cleanup of those is not implemented.
Network filesystems and abrupt hardware failures have not been tested. Existing
extended attributes/ACLs are not copied; binary distribution and OS integration need
a separate review. New files use the temporary-file restrictive default mode.

Layout note: alpha.18 also fixes a previously ignored paragraph gap. Legacy
character values and alignment are preserved; corrected spacing can change the
vertical positions compared with an earlier renderer. This is a layout bug fix,
not a stored-text conversion.

## Following paragraph styles (v6)

Each definition accepts `next_style`: null/absent means Same style, otherwise an
existing style ID. This is not inherited. Self and reciprocal following links are
valid; missing IDs are rejected. The writer emits this field, while frozen v4/v5
readers reject it. See [ADR 0010](adr/0010-following-paragraph-styles.md).

## Character styles (v7)

Each run additionally has `character_style`: `"strong"`, `"emphasis"`, or null
(default/missing means no character style). Strong overrides bold to true;
Emphasis overrides italic to true. Other properties inherit the resolved paragraph
style. Direct properties are applied last, including explicit false. A run has
only one named character style. Definitions are fixed in v7; there is no mutable
character-style catalog in this iteration.

The writer emits the reference separately from `direct`. Paragraph definitions
and the `direct` object reject this field; v1–v6 schemas reject v7 run fields.
Legacy imports receive no character reference, without guessing from appearance.
Distinct references are never merged just because their current appearance matches.
Pending caret style is session state, not persisted before insertion. See
[ADR 0011](adr/0011-character-style-precedence.md).

## Named character definitions (v8)

V8 requires a top-level `character_styles` array (empty if unused), separate from
`style_catalog`. Each entry is `{ "id": 1, "name": "Annotation", "format": {
"italic": true, "font_size": 28 } }`. A run references it using
`"character_style": { "custom": 1 }`; built-in strings/null retain v7 meaning.

IDs are positive unique u32 values. The writer allocates max ID + 1, rejects
exhaustion, and never renumbers surviving entries. Names contain 1–64 Unicode
scalars without outer spaces/control characters/U+2028/U+2029, unique after Unicode
lowercasing against character styles, Strong, Emphasis and Default paragraph font.
No normalization/full case folding. Paragraph names are a separate namespace.
Up to 64 custom character definitions are allowed, including unused entries.

The format object accepts only the four optional direct properties, with the same
boolean/font-size validation as runs. It cannot reference another style. Unset
properties inherit the resolved paragraph; run direct overrides apply last.
All references and unused definitions are validated before accepting the document.
Unknown/duplicate fields and IDs are rejected, without lossy fallback. Strict v7
rejects custom references/catalog fields; older imports receive an empty catalog.
Creation/history/typing/replacement preserve semantic IDs and definitions. See
[ADR 0012](adr/0012-named-character-styles.md).

Alpha.26 property editing preserves character definition IDs/names and every run reference; only optional properties change. No new schema fields or version increment are required.

## Character parents (v9)

V9 character definitions add optional/null `parent`: `strong`, `emphasis` or
`{ "custom": 1 }`. None inherits the paragraph baseline. All parent references
must exist and form an acyclic graph, including unused entries. Resolve ancestors,
then leaf properties, then run direct overrides. Deletion remaps direct child
parents to its replacement and validates the resulting graph before committing.
Frozen v8 rejects parent fields; v1–v8 imports have no character parents. No
inference from appearance. The alpha.29 writer emitted v9; alpha.41 now writes v10.


## Direct paragraph spacing (v10)

Every paragraph requires a `spacing` object. Its optional/null
`line_height_percent` (integer 100–300) and `space_after_half_points` (integer
0–400) override the resolved style; absent/null fields inherit and zero after
remains explicit. Unknown fields and invalid ranges are rejected. Older v1–v9
readers are frozen; their imports initialize both properties to null without
changing source bytes. Relabeling v10 as v9 is rejected instead of dropping data.
See [ADR 0013](adr/0013-direct-paragraph-spacing.md).

## Space before (v11)

Paragraph `spacing` and paragraph style definitions accept optional
`space_before_half_points`: null/absent inherits, integers 0–400 are explicit.
The cascade uses the nearest paragraph style/ancestor, then document default 0,
then the direct paragraph override. Explicit zero is preserved. Readers v1–v10
reject this key (including null) anywhere before their strict typed validation;
legacy files import with inherited zero before. Writer v11 omits unset before
values. Protocol/settings versions stay 1. Continuous layout adds before-current
to after-previous and applies first-paragraph before; wrapped lines do not repeat it.

## Left/right indents (v12)

Paragraph style definitions and the existing paragraph `spacing` object accept
optional `left_indent_half_points` and `right_indent_half_points`: absent/null
inherits, integer 0–288 supplies an explicit value (0–144 pt). Each resolves
independently through the nearest ancestor, default zero, then direct override.
The spacing object now groups paragraph geometry; existing field names remain
stable. Writer v12 omits unset indent fields. Every v1–v11 reader rejects the new
keys even when null, before strict typed validation. Imports therefore inherit zero
without changing their source bytes. Protocol/settings remain v1. There is no
first-line or hanging indent field in v12.

## Special indent (v13)

Paragraph style definitions and paragraph `spacing` accept optional signed integer
`first_line_half_points`, range -288…288. Absent/null inherits; positive means
first-line inset, negative means hanging continuation inset, zero explicitly
disables either. Resolve the nearest style/ancestor, default zero, then direct
override. Type and magnitude are one property and one history update.

The left indent remains the outermost edge: positive adds to the first row only,
negative adds its magnitude to continuation rows only. This is the project's
continuous-layout policy, not a claim of document-format equivalence. Writer v13
omits unset values. Every v1–v12 reader rejects the key even when null before
strict typed validation; old imports inherit zero. Process/settings stay v1.

## Content v14: left tab intervals

Paragraph spacing accepts optional `tab_interval_half_points`: integer 1–288,
null/absent = default 72 (36 pt). Literal tabs remain U+0009 within text runs.
This is a direct paragraph property, not named-style inheritance. Readers v1–v13
reject the new key even when null; v14 validates bounds and rejects unknown fields.
Older applications reject v14 saves: retain an older copy when required.
[Storage/layout policy](adr/0014-tab-grid.md).

## Content v15: explicit left tab positions

Paragraph `spacing.left_tab_stops_half_points` is an optional array of up to 32
strictly increasing unique integers from 1 to 2880 (0.5–1440 pt). Omitted/empty
means no explicit positions; null, duplicates, descending order and invalid types
or bounds are rejected. Example: `[97, 192]` stores 48.5 pt and 96 pt. Stops are
measured from the unindented text-column origin. The next explicit position takes
precedence; after the final position, resume the paragraph's interval grid from
that origin. Positions beyond the current column are retained, not clamped in
storage. See ADR 0014 for line-edge/indent/layout policy and remaining limitations.

Native v1–v14 files without this field import an empty list; the new key in those
versions is rejected even when empty/null. Alpha.62 and older reject v15 saves.
Keep a separate earlier copy if an older build must still read the document.
The tested temporary-file replacement and failure-preservation behavior is unchanged.

## Center and right tab storage/layout (alpha.71)

Native v16 adds `spacing.aligned_tab_stops`, an optional sorted array of
`{position_half_points, alignment}` objects. Alignment is `center` or `right`.
Existing `left_tab_stops_half_points` is retained. Across both lists, at most 32
positions are allowed, each distinct and within 1–2880 half points (0.5–1440 pt).
Null, unknown fields/types/alignment, unsorted or duplicate positions are rejected.
Missing arrays mean no aligned stops. v1–v15 reject the new key, even empty/null;
v15 without it still opens. Alpha.70 and older reject v16 saves.

All explicit types share one ordered sequence from the unindented text-column
origin. A tab selects the next position strictly beyond the cursor and any stop
already consumed on that row. After the final explicit stop, the original left
interval grid resumes. Center/right alignment measures the complete field through
the next tab or paragraph end, including trailing spaces and resolved formatting.
Center uses half its measured width; Right uses all of it. Empty fields have zero
width. The visible anchor is bounded by the available row edge, as for left tabs.

Advances never become negative: oversized fields begin at the current cursor and
wrap normally without overlapping preceding text. Continuation rows do not repeat
the field alignment. This is a defined continuous-layout policy, not pagination
or verified Word equivalence. Paragraph alignment and tab-row justification policy
remain unchanged. Stops do not inherit from named styles; Enter copies geometry.

`Editor::set_aligned_tab_stops` replaces only center/right entries in the selected
paragraphs with one validated Undo/Redo; exclusive next-paragraph boundaries are
excluded. Left-stop commands preserve aligned entries and reject conflicts.
Current dialogs and ruler edit left stops only; type selection and center/right
markers are a later increment. Palette and icons remain unchanged.

Open [AlignedTabs.bfdoc](../examples/AlignedTabs.bfdoc) to inspect the working layout.

## Tab leaders (v17)

Explicit Left/Center/Right positions may own a dot, dash or line leader. Absence
means none; automatic grid stops never acquire leaders. Native v17 stores sorted
`spacing.tab_leaders` objects with `position_half_points` and `leader` (`dots`,
`dashes`, `line`). Every entry must reference an existing explicit stop; duplicates,
unsorted/orphan positions, null and unknown keys/types are rejected. v1–v16 remain
readable but reject this field even when empty/null. Alpha.74 and older reject v17.

## Inherited tabulation (v18)

Style definitions accept optional `tab_interval_half_points` and `tab_stops`.
An example list is `[{"position_half_points": 240, "kind": "right", "leader":
"dots"}]`. Missing/null inherits the nearest ancestor; `[]` explicitly selects
no positions. Interval and complete list resolve independently. Stops are unique,
increasing, limited to 32 and 1–2880 half points; interval is 1–288.

Paragraph nonempty direct arrays retain precedence. With empty arrays, the new
optional boolean `clear_tab_stops` distinguishes explicit suppression (`true`)
from inheritance (`false` or absent). Null is rejected. Leaders belong to the
complete stop list. See [ADR 0015](adr/0015-style-tabulation.md).

V1–v17 readers reject these new keys even when null/empty, including style
intervals; the writer emits v18. Legacy files have no style tabulation, so their
existing effective layout remains unchanged on import. Default/empty settings
can inherit a later explicit style edit; authored direct settings are preserved.
Alpha.85 and older reject v18; Save As preserves an older source. Protocol stays v1.

## Structure and page settings (v19)

The v19 writer retains paragraph runs, catalogs and direct spacing. It requires a
root `page` object and a `structure` object on every paragraph. Older v1–v18 files
import default Letter pages with 72 pt margins and empty structure; declaring an
older version while including these fields is rejected. Alpha.88 and older reject
v19; use Save As to preserve an older source.

`page` and optional `structure.section` contain integer point dimensions/margins,
single-line header/footer strings, `page_numbers` and nullable `number_start`.
Sections start a new page. Dimensions are 144–1440 pt; margins at least 18 pt and
leave at least 72 pt of content in each dimension.

`structure` contains nullable `list` (`kind`: bullet/decimal, `level`: 0–8,
nullable positive `restart`), sorted nonoverlapping `links` (`start`, `end`,
`target`, scalar offsets on grapheme boundaries), nullable `block`, boolean
`page_break_before`, `keep_with_next`, `keep_lines`, and nullable `section`.
Block paragraphs contain exactly U+FFFC and one format entry. A table block has
`kind: table`, `widths_points` (18–720), rectangular string `rows` (1–100 × 1–12)
and `repeat_header`. A picture block has `kind: picture`, PNG byte array `png`,
`width_points`/`height_points` (12–1440) and `alt` (up to 1024 characters).
PNG limits: 4 MiB, 16 megapixels, 8192 pixels per dimension, valid chunk framing,
checksums and pixel data. The total native file limit remains 16 MiB; oversized
serialized documents fail before replacing a prior file. Cell text contributes to
the overall 100,000-character limit and has its own 50,000-character table limit.

All structures reject unknown fields and invalid values. The schema does not
store platform print queues, PDF bytes or UI state. See [ADR 0016](adr/0016-document-structure-and-pages.md).


## V20: named fonts and retained package properties

V20 adds optional `family` in direct/style character properties: `Arial`,
`TimesNewRoman`, or `CourierNew`. Absent values inherit; the default remains
Arial. Optional top-level `package_properties` holds original `core`/`app`
metadata XML, each bounded to 128 KiB with validated namespace/root and no DTD.
Metadata survives native/DOCX conversion and history; imported counters are not
updated after edits. Host font files are never embedded. Earlier v1–v19 inputs
reject these new fields even when null. Page/structure requirements from v19 stay.
See [ADR 0019](adr/0019-docx-sections-pictures-tabs-fonts.md).
