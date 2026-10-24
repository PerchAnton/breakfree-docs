# DOCX exchange profile — alpha.90

This is an initial **restricted interchange profile**, not a general DOCX reader
or a Word compatibility claim. Most existing Word documents have additional
parts/properties and will currently be rejected. The profile supports files
produced by the included fixture generator and equivalent OOXML markup. Native
`.bfdoc` remains the full-fidelity editing format.

## Supported read/write surface

| Element | Initial support |
| :--- | :--- |
| Package | UTF-8 names, stored/deflated ZIP, CRC and local/central consistency, optional checked data descriptors; no extraction to disk |
| Text | Paragraphs, runs, XML escapes, preserved spaces, literal tabs using the default grid; English/Ukrainian/Russian/combining/emoji fixture |
| Character properties | Explicit bold, italic, single underline, exact half-point size; direct false overrides |
| Paragraph properties | Left/center/right/justified alignment, before/after spacing, proportional line spacing, left/right/first-line/hanging indents |
| Flow | Page break before, keep with next, keep lines |
| Styles | Named paragraph/character styles, stable exported IDs, parent and next references; graph validation; built-in names retained |
| Page | One section, exact whole-point page size and margins |
| Relationships | Internal main-document and styles relationships only |
| Defaults | Arial 11 pt, 125% lines, 5.5 pt after; explicit matching document defaults required |

Measurements outside the model's exact precision/range are rejected. In
particular, proportional line spacing must map exactly between integer percent
and OOXML's 240ths of a line. Native absolute bold/italic style overrides are not
always equivalent to OOXML style toggles: overlapping enabled inherited
bold/italic values are rejected, rather than flattened or exported incorrectly.

The package currently requires `[Content_Types].xml`, `_rels/.rels`,
`word/document.xml`, `word/styles.xml` and
`word/_rels/document.xml.rels`. Namespace prefixes and attribute order are not
identities. The documented default-property sequence is required. Strict OOXML
namespaces, relocated parts, additional document properties, theme/settings parts
and arbitrary Word-generated defaults are not accepted yet.

## Explicit rejection rather than loss

Unsupported parts, relationships, elements or attributes cause an error **before
an imported model replaces the open session**. This includes lists, tables,
images, hyperlinks, extra sections, headers/footers, page numbering, explicit or
style tab stops, revision marks, comments, fields, colors, font-family changes,
merged cells, floating drawings and macros. Extra package parts are not silently
ignored, even if their content might be harmless metadata. There is no lossy
"continue anyway" mode and no hidden native sidecar in the DOCX package.

Limits are 32 MiB compressed/total expanded, 8 MiB per part, 128 entries, XML depth
64 and 200,000 nodes per part, followed by the existing document-model limits.
Encrypted/split/ZIP64 archives, unsafe or duplicate paths, overlapping payloads,
invalid CRCs, unsupported compression, external relationships, DTDs and custom
entities are rejected. No network resource or executable content is loaded.

## Open and save behavior

- File → Open and `--open <absolute.docx>` use the same validated importer.
- An imported document has no native destination. Save/Save As creates `.bfdoc`;
  closing an unchanged import leaves its original DOCX available.
- File → Export DOCX (also command search) validates the model and writes a separate
  `.docx`. It does not change the session's save destination, dirty flag or undo.
- Failed conversion does not touch the target. Writing reuses the native checked
  temporary-file/replacement path; injected write/sync/replace failures preserve
  an existing output. No crash recovery or universal filesystem guarantee is made.
- `--capabilities` advertises `ooxml.wordprocessingml` with `docx` read/write. That
  describes available operations, not support for every OOXML feature.

## Reproduce

```sh
cargo run --locked --no-default-features --example docx_exchange
cargo run --locked -- --open "$(pwd)/examples/DocxExchange.docx"
cargo test --locked docx::
```

Windows PowerShell (build instructions remain in README; native Windows testing
is deferred):

```powershell
cargo run --locked --no-default-features --example docx_exchange
cargo run --locked -- --open (Join-Path (Get-Location) 'examples\DocxExchange.docx')
```

The generator writes the checked-in `.bfdoc`/`.docx` fixture pair and asserts exact
model equality on reopen. Run in a clean checkout if reviewing fixture changes.
See [validation](VALIDATION.md#alpha90-docx-profile-validation) for the actual
external application/build and limitations. External save-back is not validated.

## Next D4 increments and release gate

1. Expand OPC/defaults handling using independently authored input fixtures.
2. Add list numbering and hyperlinks with validated relationship targets.
3. Add rectangular tables and inline PNG drawings without losing unsupported data.
4. Add section/header/footer and tabulation mapping with semantic round-trip tests.
5. Expand independent read/write corpus, package notices and tested-target archive;
   obtain stage/publication approval before publishing an alpha package.

D4 remains **in progress**. This commit does not publish a binary, tag or release.
