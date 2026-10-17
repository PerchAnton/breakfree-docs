# Native BreakFree Docs document format

Extension `.bfdoc`; type `breakfree.docs.native`. Current writer: content v3.
This is not a DOCX container. Process protocol and file-content versions are independent.

```json
{
  "format": "breakfree.docs.native",
  "content_version": 3,
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
`paragraph_style` is required: `normal`, `no_spacing` or `heading1`. The fixed v3
catalog and inheritance order are specified in [ADR 0006](adr/0006-built-in-paragraph-styles.md).
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
Every explicit save writes v3, even for an unchanged older document. Use Save As
to retain the original. **Earlier builds cannot open v3 saves.** No downgrade or
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
