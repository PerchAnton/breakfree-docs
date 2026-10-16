# Native document format v2 and v1 compatibility

The only accepted file extension is `.bfdoc` (case-insensitive). Content is UTF-8
JSON with an exact schema; no scripts, HTML, URLs or embedded resources are loaded.
This is not a DOCX container. Process protocol and file-content versions are independent.

```json
{
  "format": "breakfree.docs.native",
  "content_version": 2,
  "paragraphs": [
    {
      "alignment": "left",
      "runs": [{"text": "Hello", "bold": true, "italic": false, "underline": true, "font_size_half_points": 24}]
    },
    {"alignment": "center", "runs": []}
  ]
}
```

A document contains at least one paragraph. An empty paragraph has `runs: []`.
Run text is nonempty; bold, italic and underline are required booleans.
`font_size_half_points` is a required integer from 2 through 800 (1–400 pt), with
22 representing the default 11 pt. Missing, null, fractional and out-of-range values
are rejected. There are no implicit run defaults in v2. Alignment is `left`, `center`,
`right` or `justify`. Paragraph text cannot contain LF; document paragraphs are
joined with LF for edit indices. Adjacent equal styles are compacted on write.
The model retains supported text exactly, without Unicode normalization. See
[ADR 0002](adr/0002-rust-native-ribbon.md) for Unicode boundaries and controls.

Reject unknown top-level, paragraph and run fields, duplicate JSON keys, foreign
format IDs, unsupported versions, invalid UTF-8, invalid styles and resource-limit
violations. There is no lossy fallback. Limits: 16 MiB encoded
file, 100,000 scalar characters including paragraph separators. Read failures never
write the source file or replace the current GUI session.

## Reading earlier files

The v1 schema uses the same envelope and paragraphs, but runs have exactly `text`,
`bold` and `italic`. A separate strict reader maps v1 runs to 11 pt with underline
off. Unknown v1 fields remain errors, including fields that are valid only in v2.
The original [Welcome fixture](../examples/Welcome.bfdoc) stays byte-identical.
Opening v1 performs only an in-memory conversion and does not dirty or write it.

All explicit saves write v2, including unchanged v1 documents. A successful save
to the original path replaces the older encoding; Save As keeps the older file.
**Earlier v1-only builds cannot open new v2 saves.** There is no downgrade/export
to v1. Unknown versions are rejected before constructing a document. Header and
full-schema decoding retain duplicate-key rejection. See [ADR 0003](adr/0003-character-size-and-underline.md).

Pending caret size/underline is session state until text is inserted. It does not
persist as formatting for an empty paragraph. Application version and process
protocol v1 remain independent of this file-content version.

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
