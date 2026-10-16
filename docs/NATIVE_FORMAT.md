# Native document format v1

The only accepted file extension is `.bfdoc` (case-insensitive). Content is UTF-8
JSON with an exact schema; no scripts, HTML, URLs or embedded resources are loaded.
This is not a DOCX container. Process protocol and file-content versions are independent.

```json
{
  "format": "breakfree.docs.native",
  "content_version": 1,
  "paragraphs": [
    {
      "alignment": "left",
      "runs": [{"text": "Hello", "bold": true, "italic": false}]
    },
    {"alignment": "center", "runs": []}
  ]
}
```

A document contains at least one paragraph. An empty paragraph has `runs: []`.
Run text is nonempty; emphasis fields are booleans. Alignment is `left`, `center`,
`right` or `justify`. Paragraph text cannot contain LF; document paragraphs are
joined with LF for edit indices. Adjacent equal styles are compacted on write.
The model retains supported text exactly, without Unicode normalization. See
[ADR 0002](adr/0002-rust-native-ribbon.md) for Unicode boundaries and controls.

Reject unknown top-level, paragraph and run fields, duplicate JSON keys, foreign
format IDs, unsupported versions, invalid UTF-8, invalid styles and resource-limit
violations. There is no implicit migration or lossy fallback. Limits: 16 MiB encoded
file, 100,000 scalar characters including paragraph separators. Read failures never
write the source file or replace the current GUI session.

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
