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
