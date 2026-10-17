# ADR 0002 — Rust editor and native office ribbon

- Date: 2026-10-16
- Status: **Rust language and Word 2024 Windows visual target approved by the owner;
  eframe/egui implementation is a replaceable prototype pending toolkit review**
- Supersedes: ADR 0001's Python/Qt implementation choice
- License: unchanged; project and brand licenses remain undecided

Alpha.3 extension: [ADR 0003](0003-character-size-and-underline.md) adds character
size/underline and native v2 writing with strict v1 reading. The v1-only and fixed
size statements below describe the initial Rust checkpoint.

## Decision and scope

Implement every runtime layer in Rust. The owner's explicit instruction settles
the language; no further language approval is needed. Keep the historical Python
experiment under `prototypes/python/`, outside the Cargo build and CI application
path. Do not ship JS/TS, Electron or an HTML/WebView editor.

Use native eframe 0.33.0 with egui, OpenGL and native file/clipboard adapters to
prove this iteration. The toolkit remains replaceable: native IME, accessibility,
complex shaping and Windows DPI are adoption gates. A standard toolkit demo does
not meet the interface requirement. Custom Rust window chrome, ribbon groups,
backstage, navigation, formatting pane, rulers and status bar follow
[UI_SPEC](../UI_SPEC.md). No third-party application source or artwork is reused.

## Boundaries

| Module | Responsibility | GUI dependency |
| :--- | :--- | :--- |
| model | Paragraphs, per-scalar styles, validation, Unicode grapheme boundaries | None |
| commands | Replacement, emphasis, alignment, selection and bounded undo/redo | None |
| storage | Strict native v1 serialization, bounded parsing, temporary-file replacement | None |
| session | Path, saved checkpoint and Save/Discard/Cancel decisions | None |
| presentation | Text-diff/selection conversion, ribbon sizing policy | None |
| layout | Font loading, rich galley projection and hit testing | egui text layout only; no widgets |
| chrome | Original vector icons and accessible shell controls; no document state | egui only |
| gui | Native input surface, custom ribbon, dialogs and panes | eframe/egui |
| gui::ribbon_navigation | Temporary ribbon, focus traversal and menu routing; no document ownership | egui only |
| gui::keytips | Sequential access-key state and anchored hints, dispatching existing GUI actions; no document ownership | egui only |
| integration | OS arguments and protocol JSON; main dispatches this same GUI | None |

The model is authoritative. TextEdit is an input/projection surface: native undo
is cleared, committed diffs call the shared command engine, and saves serialize
the model. IME preedit remains transient; a commit is one model transaction.
No document exists only in a widget. The engine also builds without `desktop`.

## Preserved semantics and compatibility

Native `.bfdoc` content v1 and process protocol v1 are unchanged and independent.
The historical Welcome fixture is tested by the Rust reader. Unicode scalar
indices, half-open grapheme-aligned edits, first-affected-paragraph alignment
inheritance, 100-operation snapshot history and pending caret emphasis are retained.
Input events delivered in one widget update form one text undo operation; explicit
paste, deletion and formatting commands have separate boundaries. Time-based typing
coalescing is not implemented. No Unicode normalization occurs.
Model limits remain 100,000 scalars and 16 MiB encoded files.

Storage uses a same-directory tempfile, checked writes, flush, file sync,
permission preservation and `tempfile::NamedTempFile::persist`. Fault tests reach
partial temporary bytes and failures before sync/replacement. See
[NATIVE_FORMAT](../NATIVE_FORMAT.md) for the exact guarantees and omissions.

## Consequences and review gates

Cargo.lock pins dependencies and rust-toolchain.toml pins the tested compiler.
No build reads another repository's live branch. The local Office baseline stays
provisional until published contracts can be pinned and reconciled. No suite engine
is extracted and no network service is introduced.

This delivers short-document editing on a continuous white canvas, not pagination.
Font family/size editing, styles, lists, contextual object tabs, print, PDF and DOCX
remain disabled/planned. OS-provided fonts are loaded locally; egui's embedded
fallback retains its upstream notices. Full script shaping, color emoji, screen
readers, native Windows IME and 100/125/150% scaling require explicit evidence.
If the toolkit fails those gates, replace the GUI/layout while keeping engine,
native fixtures and protocol tests. Do not reopen the approved Rust decision.

## Alpha.8 view and access-key extension

Quick-access and tab/View KeyTips extend the existing GUI-only routing module.
Dedicated QAT anchors prevent collisions with File actions. The separate Rust
`zoom_dialog` module owns only a temporary view-scale draft; acceptance does not
change the document model or command history. View controls register with the
same ribbon focus policy as Home. No dependency or engine-format change is needed.

## Alpha.9 toolbar preferences and View layout

The Rust `quick_access` module owns only window-local visibility/order/placement
and delegates actions to the existing router. Its focus targets share the ribbon
navigation machinery; dynamic numeric hints reflect the visible order. No settings
file, new dependency or document schema is introduced. Persistence and an arbitrary
reordering editor are deferred until their settings lifecycle is designed.

The separate `view_ribbon` module supplies compact native controls. Width fitting
is a GUI view mode calculated from the actual workspace; manual zoom exits it.
It is excluded from document history and native serialization. Continuous-canvas
wording remains explicit until a real pagination layer is implemented.

## Alpha.14 navigation refinement

Extract literal matching into GUI-independent `search`, returning scalar ranges
that start and end at complete grapheme boundaries. Use the same results for the
count, result rows, previous/next selection and the existing Replace action. Keep
case sensitivity, nonoverlapping matches and exact input bytes; do not normalize
or silently expand partial matches. This narrows the old GUI substring search,
which counted partial clusters and then expanded their selection.

`gui::navigation` owns only widgets and view focus/scroll state. The document and
history remain in the shared engine. Finding is read-only; selecting a result sets
its range and insertion style, while Replace uses the existing validated command
and one undo operation. Bounded context excerpts are presentation only. Virtual
rows expose every match without retaining the old first-40 cap. No schema or
GUI dependency is added to the engine. Case folding, word boundaries, replace-all
and richer search options remain separately scoped work.

## Alpha.15 search options

Extend `search::Query` with window-local Match case (default on) and Whole words
(default off). Cache the prepared query until input/options change. `regex` supplies
[Unicode simple case folding](https://docs.rs/regex/1.13.1/regex/struct.RegexBuilder.html#method.unicode)
for escaped literal input; existing `unicode-segmentation` supplies word/grapheme
boundaries. Keep original byte/scalar positions, including matches whose UTF-8
length differs from the query. Do not normalize, expand multi-character folds
(such as ß/SS), use locale-specific casing, or expose a pattern language.

A whole-word match starts at a Unicode word start and ends at a word end; phrases
may span words/paragraphs, but punctuation-only input has no whole-word matches.
Internal apostrophes, digits and underscores remain part of their Unicode words.
Rejected partial matches must not consume a later overlapping valid match.
Accepted matches remain nonoverlapping. Queries over 4,096 UTF-8 bytes or failed
preparation show an error and disable match actions; input is retained verbatim.

This small library addition does not change the GUI toolkit or document engine
architecture. Cargo.lock pins the dependency graph; notices and the macOS metadata
inventory record upstream licenses. No implementation source is vendored and no
project license is selected. Full distribution review remains open. Search options
are neither document content nor saved preferences; replace-all, locale tailoring,
normalization and cross-launch search settings remain separate work.

## Alpha.16 batch replacement

[ADR 0005](0005-replace-all-preview.md) adds an immutable prepared replacement
transaction to `commands`. `gui::replace_all_dialog` only displays context and
collects confirmation. Document state, validation and the single history boundary
remain in the same engine used by tests. Palette/icons and schemas are unchanged.
