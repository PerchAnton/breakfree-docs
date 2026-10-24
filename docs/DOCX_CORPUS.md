# DOCX interoperability corpus — alpha.96

This corpus separates exact native-model round trips from an external producer's
behavior. All content is original synthetic text, never user documents. Raw
external packages are checked in unchanged; no parts are stripped to make a test
pass. There is no external implementation source or new runtime dependency.

## Recorded cases

Files and SHA-256 values are in
[the manifest](../tests/fixtures/docx-corpus/manifest.json). Generation used macOS
27.0.1 (26A434), `/usr/bin/textutil`, CocoaOOXMLWriter/2775.1. The manifest pins
the executable hash and original source hashes. Generation is macOS-only; normal
Cargo fixture tests and hash verification are offline and platform-independent.
Git attributes disable line-ending conversion for checksum-pinned evidence.

| Fixture | Producer / route | Current result |
| :--- | :--- | :--- |
| independent.docx | Original literal OOXML packaged by Python zipfile; no BreakFree encoder | Accepted; Unicode, direct bold/italic and centered paragraph checked. This is independent adapter input, not external-application evidence |
| cocoa-unicode.docx | textutil from original UTF-8 text | Rejected, unchanged |
| cocoa-emphasis.docx | textutil from original RTF emphasis/alignment | Rejected, unchanged |
| cocoa-saveback.docx | textutil re-save of independent.docx | Rejected, unchanged |
| cocoa-docs-saveback.docx | textutil re-save of the existing original DocxExchange.docx | Rejected, unchanged |
| docs-saved.docx | Independent fixture → shared Session/edit → DOCX Save | Exact whole-model reopen; accepted |
| cocoa-returned.docx | docs-saved.docx → textutil DOCX re-save | Text, per-character bold/italic and paragraph alignment preserved externally; reimport still rejected, unchanged |

Five rejection cases are useful negative acceptance evidence, **not five newly
supported external documents**. All currently stop at a `customXml` relationship
in `word/_rels/document.xml.rels`. Inspection also finds a theme part, Cocoa
metadata and producer-specific character properties. Fixing the first reported
relationship alone will not establish compatibility. Unknown parts must not be
silently dropped. The diagnostic now identifies the relationship part, ID and type.

## Save-back checks

`tests/docx_corpus.rs` copies each input to a temporary destination. Accepted cases
use the GUI's shared Session/editor: edit, undo, redo, save to the same path,
reopen and compare the entire model, undo after saving, then save/reopen again.
Every rejected input retains identical bytes. The committed source fixtures are
never overwritten by the test.

The external probe checks a deliberately smaller projection: paragraph text
(including Ukrainian and a combining accent), per-character bold/italic and
alignment. It does **not** claim equality of fonts, styles, metadata, geometry,
pagination, fields, tables or pictures. Its returned file fails reimport, and the
probe confirms no output was written. See the
[recorded report](../tests/fixtures/docx-corpus/saveback-report.json).

## Reproduce

```sh
cargo test --locked --no-default-features --test docx_corpus
python3 tools/check_repository.py
# macOS only; choose a directory that does not already exist:
python3 tools/check_external_docx.py /tmp/docs-external-review-new
# Explicit fixture refresh, inspect all binary/hash changes afterward:
python3 tools/generate_docx_corpus.py
```

Cocoa ZIP timestamps can change, so regeneration is not promised byte-identical.
Hash changes require review; checked-in evidence and sources remain reproducible.
The generator invokes a developer Rust example; no command is added to the Office
integration protocol. Build/runtime behavior still has no external editor dependency.

## Remaining acceptance

A full external DOCX save-back cycle remains open. Add a carefully specified
representation/preservation policy for external themes/metadata/settings before
accepting these files, then expand external styles/lists/tables/sections coverage.
The installed Word remains activation-limited; no Word save-back or visual check
was repeated in this iteration. Windows and paper printing are deferred; VoiceOver
is last. This corpus does not establish Word 2024 compatibility or release readiness.
