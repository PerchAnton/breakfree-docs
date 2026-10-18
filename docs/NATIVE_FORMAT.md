# Native BreakFree Docs document format

Extension `.bfdoc`; type `breakfree.docs.native`. Current writer: content v6.
This is not a DOCX container. Process protocol and file-content versions are independent.

```json
{
  "format": "breakfree.docs.native",
  "content_version": 6,
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
      "runs": [{"text": "Hello", "direct": {"bold": null, "italic": true, "underline": null, "font_size": null}}]
    },
    {"alignment": "center", "paragraph_style": "normal", "runs": []}
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
Adjacent identical direct overrides compact on write, without Unicode normalization.

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
Every explicit save writes v6, even
for an unchanged older document. Use Save As
to retain the original. **Alpha.22 and earlier cannot open v6 saves.** No downgrade or
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
