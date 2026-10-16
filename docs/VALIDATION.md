# Rust editor validation and remaining gates

Evaluation: **2026-10-16, Europe/Kyiv**, macOS 27.0.1 arm64, rustc/cargo 1.99.1,
eframe 0.33.0 and locked egui/epaint 0.33.3. Rust is owner-approved; toolkit adoption,
license, supported-platform and full D1 reviews remain separate gates. Historical
Python/Qt results are preserved in [history/python-validation.md](history/python-validation.md)
and must not be presented as Rust results.

## Checks actually run

| Local check | Observed result |
| :--- | :--- |
| `cargo test --locked` | 27 passed: 14 engine/storage/presentation, 3 CLI subprocess and 10 actual egui widget event tests; no skipped tests |
| `cargo test --locked --no-default-features` | 17 passed; core and CLI build without GUI dependencies |
| `cargo fmt --check` / Clippy all targets with warnings denied | Passed |
| `cargo build --locked --release` | Native macOS binary built; no Office/sibling source inputs |
| Copied executable outside checkout | Capabilities/version and malformed-open checks without Office or source files; CLI tests also run from a temporary working directory |
| Repository references, contract and artwork | `python3 tools/check_repository.py` passed; approved logo embedded directly and archive copy byte-identical |
| Workflow YAML / whitespace | Ruby YAML parse and `git diff --check` passed |
| Native rendering | Real native `--new` window launched; `--open` rendered Welcome fixture to the committed [capture](screenshots/rust-home-macos.png) through opt-in eframe capture |
| Synthetic DPI | Actual widgets at 1/1.25/1.5 scale and 960/1366/1920 physical widths; ribbon/group/workspace/status bounds and collapse assertions passed |

Server CI is configured for Windows, Ubuntu and macOS with the same Cargo checks
and native release build. Its observed run/commit results are recorded in
PROJECT_LOG after publication, separately from this local evidence.

## D01–D09 evidence map

| ID | Rust evidence | Remaining limits |
| :--- | :--- | :--- |
| D01 | Core insertion/split/merge/replacement/deletion; actual text-widget paste/type/select/replace and ribbon pointer click | Short documents; drag/drop not implemented |
| D02 | Exact English/Ukrainian/Russian, combining accents, non-BMP, ZWJ, flag boundaries; Shift+Arrow grapheme selection/deletion; Windows clipboard CRLF normalization; IME preedit/commit/cancel | Synthetic widget IME events are not native OS IME evidence. CJK/RTL/complex shaping and all writing systems not certified; missing emoji glyph visible in capture |
| D03 | Exact emphasis/alignment snapshots and native roundtrips; styled galley alignment and scalar cursor hit tests; actual Bold ribbon click | Fonts/styles/lists unsupported; no pagination/fidelity claim |
| D04 | Exact text/format undo/redo, branch redo clearing, pending insertion style, selection restoration, widget shortcut history and one-commit IME transaction | 100 snapshot operations, no persistent history or time-based coalescing |
| D05 | Complete native roundtrip, every alignment, earlier Welcome v1 fixture, GUI save/reopen through shared session | No migrations, autosave or recovery |
| D06 | Core Save/Discard/Cancel and failure; pointer events activate all three real modal buttons; failed save keeps dirty session; close approval flag | Actual OS pickers, cancelled picker and native window-manager closing still need manual platform review |
| D07 | Real partial temporary write, injected errors before sync/replace, real replacement failure and missing parent; previous file bytes and dirty model preserved | No power-loss, directory fsync, concurrent-writer, ACL/extended-attribute or network-filesystem guarantee |
| D08 | Corrupt JSON/UTF-8, foreign type, unknown version/fields, duplicate keys, invalid styles/text and unsupported extension rejected; source unchanged | Bounded regression tests, not completed fuzzing |
| D09 | Actual egui keyboard and pointer event loop covers edit/selection, emphasis, undo/redo and save; model compared with projection | Native OS keyboard/dialog/IME and full focus/accessibility review pending |

The native executable was launched and captured on macOS. Computer Use inventory
was accessible, but it did not expose the unbundled executable; selecting its path
returned `Invalid app`. Therefore no manual OS keyboard or picker interaction is
claimed. Word for macOS is not the approved Windows reference and was not used as
a substitute. Windows itself and Word 2024 for Windows are unavailable here.

## Integration and independence

`tests/cli.rs` verifies exclusive argument handling, absolute paths, invalid-file
failure before a window, clean stdout/stderr, exact v1 capability IDs/formats/actions
and build version. Information commands do not create a graphical context and work
outside the checkout with display variables absent. Desktop-disabled builds advertise
no launch actions. Core tests compile without eframe, GUI adapters or Office.

The [pinned owner baseline](../contracts/v1/BASELINE.md) still applies: a fresh Office
main tree read returned cfaaf419e84d9981acd421d130113ca2265061fd with no contracts/ADR.
Upstream contract reconciliation and an actual Office launcher remain unverified.
No build downloads another application's main branch or starts a service.

## Next small gate

Run this native binary on Windows, capture the named Word 2024 reference at matched
conditions and complete the keyboard/IME/dialog, clipping, fallback and native DPI
matrix in [UI_SPEC](UI_SPEC.md). Review the GUI toolkit against those results before
adding document features. Keep exact Rust native-format/history regressions intact.
No DOCX/PDF/print/cloud/collaboration/macros/AI, installer, signing, merge or release
is included. Limits remain 100,000 scalars / 16 MiB and unmeasured long-document costs.
