# ADR 0001 — Reversible standalone desktop prototype

- Status: **implemented experiment, proposed for owner review; not an adopted final stack**
- Date: 2026-10-16
- Scope: D0 prototype and the D1 writing loop
- License: unchanged; software and brand licensing remain owner decisions

## Context and inputs

Docs at `f2353b093e2fe7390c54f9d4e96a03fb1d882aa2` contains documentation and the
approved Docs artwork, with a Rust-first proposal but no adopted language, toolkit,
native format, code or ADR. The owner's current instruction authorizes a working,
reversible prototype. It does not settle the long-term architecture or license.

Office at `cfaaf419e84d9981acd421d130113ca2265061fd` has a planning README and
roadmap, no ADR, `contracts/v1/` or `docs/DEVELOPMENT_STANDARD.md`. Its roadmap asks
for independent applications and small language/toolkit experiments. The inspected
revision and provisional specification are [recorded locally](../../contracts/v1/BASELINE.md).
Nothing is loaded from another repository during building or running.

## Experiment

Use Python 3.11+ for an original immutable model and commands, with Qt 6 widgets
through the optional `PySide6-Essentials==6.11.1` dependency. Use `regex==2026.9.29`
extended grapheme segmentation to validate editing boundaries. The initial
evaluation host is macOS arm64; Windows and Linux remain evaluation targets until
their builds, input and dialogs are independently checked.

Python/Qt is available on the evaluation host and allows a real desktop writing
loop and native text input to be exercised now. Rust was proposed but no Rust
toolchain is present here. This experiment is not evidence that Rust is unsuitable.
A Rust implementation and another toolkit remain alternatives for owner review.
PySide/Qt licensing and distribution obligations need review before binary releases;
no third-party implementation source is vendored. See [dependency notices](../THIRD_PARTY_NOTICES.md).

## Boundaries

| Module | Responsibility / dependencies |
| :--- | :--- |
| `model` | Immutable paragraphs, scalar offsets and style values; Python + regex |
| `commands` | Replace, emphasis, alignment, bounded undo/redo; model only |
| `storage` | Strict native v1 parser and write-then-replace; model + standard library |
| `session` | Clean checkpoint, file path and unsaved decisions; commands + storage |
| `presentation` | Spans and UTF-16 conversion; model, no GUI |
| `layout` | Qt text layout projection, line wrapping and formatting; no widgets |
| `gui` | Native input/navigation, controls, dialogs; shared engine/session/layout |
| `integration` | Validated process arguments and capabilities; imports GUI only to launch |

The immutable model is authoritative. QTextEdit is a projection and an input
surface. Its native undo and rich-text paste are disabled. Committed text changes
are translated to the same replacement command used by headless tests, then
formatting is projected from the model. Clipboard insertion, deletion, formatting
and history call that engine directly. Saving serializes the model, never widget HTML.

## Semantics and deliberate limits

- Core indices are Unicode scalar offsets into paragraphs joined with LF. Ranges
  are half-open and must align to regex `\X` grapheme boundaries. Qt positions use
  UTF-16 units through an explicit conversion layer. No Unicode normalization.
- Every key input commit, IME commit, paste, cut, delete or formatting command is
  one undo step; there is no time-based typing coalescing. Store up to 100 immutable
  before/after operations. New edits clear redo. Caret and selection restore with history.
- Character emphasis at a collapsed caret sets the next insertion style, not a
  document operation. Empty paragraphs do not persist a pending insertion style.
- Replacing/splitting/merging paragraphs gives new paragraphs the first affected
  paragraph's alignment. Unaffected paragraphs keep theirs. An exclusive selection
  end at the next paragraph start does not align that paragraph.
- Native v1 is UTF-8 JSON, type `breakfree.docs.native`, `.bfdoc`, independently
  versioned with `content_version: 1`. Unknown fields/types/versions are rejected.
- Same-directory temporary write, flush, file fsync, close and replace keep the
  original target intact on tested pre-replacement failures. There is no autosave,
  crash recovery, concurrent-writer detection or power-loss durability guarantee.
- Limit documents to 100,000 scalar characters and files to 16 MiB. Full document
  projection/snapshot history favors correctness for short documents; performance
  for long documents is unmeasured. No pagination or general shaping claim.
- CRLF/CR clipboard input becomes LF. NUL, most C0 controls, surrogate code points
  and U+2028/U+2029 are explicitly rejected; NBSP is preserved. Tabs can be pasted;
  Tab navigates controls. Modifier+Delete/Backspace currently deletes one grapheme.

## Review and reversal criteria

Keep D0 architecture, license and platform approval open. Review IME composition,
assistive technology, font fallback, long-document costs and Windows behavior before
adoption. Preserve native fixtures, command semantics and black-box tests if replacing
Python or Qt. A future engine migration needs its own decision; do not extract a
suite-wide document engine from this prototype.
