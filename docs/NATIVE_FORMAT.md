# Native BreakFree Docs document format

Extension `.bfdoc`; type `breakfree.docs.native`. Current writer: content v12.
This is not a DOCX container. Process protocol and file-content versions are independent.

```json
{
  "format": "breakfree.docs.native",
  "content_version": 12,
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
Every explicit save writes v12, even
for an unchanged older document. Use Save As
to retain the original. **Alpha.42 and earlier cannot open v12 saves.** No downgrade or
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
