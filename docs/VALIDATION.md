# Rust editor validation and remaining gates

Evaluation: **2026-10-16, Europe/Kyiv**, macOS 27.0.1 arm64, rustc/cargo 1.99.1,
eframe 0.33.0 and locked egui/epaint 0.33.3. Rust is owner-approved; toolkit adoption,
license, supported-platform and full D1 reviews remain separate gates. Historical
Python/Qt results are preserved in [history/python-validation.md](history/python-validation.md)
and must not be presented as Rust results.

## Checks actually run

Current local checkpoint: **0.1.0-alpha.4**. Earlier capture/CI evidence retains its
original scope; this iteration does not establish native Windows behavior.

| Local check | Observed result |
| :--- | :--- |
| `cargo test --locked` | 44 passed: 21 engine/storage/presentation, 3 CLI subprocess and 20 actual egui widget event tests; no skipped tests |
| `cargo test --locked --no-default-features` | 24 passed; core and CLI build without GUI dependencies |
| `cargo fmt --check` / Clippy all targets with warnings denied | Passed |
| `cargo build --locked --release` | Native macOS binary built; no Office/sibling source inputs |
| Copied executable outside checkout | Alpha.4 release `--version` and `--capabilities` verified from a temporary directory without Office/source files; that copied binary also ran the native app-bundle smoke. Malformed-open and argument handling are covered by CLI subprocess tests |
| Repository references, contract and artwork | `python3 tools/check_repository.py` passed; approved logo embedded directly and archive copy byte-identical |
| Workflow YAML / whitespace | Ruby YAML parse and `git diff --check` passed |
| Native rendering | Alpha.4 Formatting fixture and Home inspected at [1366 × 900 DIP](screenshots/rust-home-alpha4-macos.png), [960 × 700](screenshots/rust-home-alpha4-compact-macos.png) and [640 × 700](screenshots/rust-home-alpha4-narrow-macos.png), backing scale 2. Earlier captures retain their version context |
| Native macOS interaction | ASCII typing, exact Unicode paste, selection, B/U, size field, Grow shortcut, undo/redo, system Save/Open, dirty New/Cancel/Discard passed through Computer Use; saved v2 text/styles inspected. Native non-ASCII typing/IME is not certified; see the scoped result below |
| Synthetic DPI | Actual widgets at 1/1.25/1.5 scale and 960/1366/1920 physical widths; ribbon/group/workspace/status bounds and collapse assertions passed; 27 additional width/scale combinations check individual Home control containment/nonoverlap and title/search/window bounds |

Server CI is configured for Windows, Ubuntu and macOS with the same Cargo checks
and native release build. Its observed run/commit results are recorded in
PROJECT_LOG after publication, separately from this local evidence.

The design refinement also checks actual pointer activation of italic/alignment,
selection preservation, quick Undo/Redo, disabled strikethrough/style controls leaving
the model unchanged, zoom ±/reset, File Home/Options/Back and dirty New/Cancel.
The reference theme image was inspected, but no controlled Word comparison was run.
The first design CI on Windows exposed Segoe UI line-height expansion of an empty
15 × 16 DIP icon button at 125%. Icon controls now reserve a minimal empty label
and paint their contents separately. A font-metric regression with a 36-DIP button
font fails on the previous implementation and passes with the fix at 1/1.25/1.5.
The existing strict geometry assertions are retained; server rerun results are
recorded separately in the journal.

Alpha.3 adds real field/preset/underline events, Ctrl/Command+U and Shift+P, selection
preservation, mixed values, Enter/Tab/Escape, invalid input, styled typing/IME,
save/reopen and size input pending during Save/close. The Escape test exposed the
toolkit releasing focus before the UI pass; cancellation now recognizes previous
focus and leaves history/model unchanged. Sized/underlined galley tests check actual
geometry and scalar hit positions at 50/100/200% zoom. Core tests retain exact
English/Ukrainian/Russian/combining text, pending style snapshots, strict v1/v2
schemas and failed writes. V1 opening preserves source bytes; explicit v2 save
preserves all supported styles. Missing/null/out-of-range/fractional sizes and
duplicate/unknown run fields are rejected.

Alpha.4 adds preset stepping across mixed runs and every valid half-point size,
no-op bounds/history, native roundtrip and grapheme rejection. Pointer/keyboard
widget tests cover wide/minimum-width Grow/Shrink, composition disablement,
Paste/Find/Select menus, Escape cancellation and working overflow. Strict geometry
checks now include menu triggers and the More button. The initial 640-DIP budget
failure was resolved by narrowing the overflow Font group, keeping the assertions.

## Native macOS alpha.4 smoke

The current release binary was copied to a temporary app bundle and verified with
`--version`/`--capabilities` outside the checkout. Computer Use successfully opened
it in this iteration. This supersedes the earlier launch/permission limitation only
for the specific checks below; the bundle is not a supported installer.

| Action | Observed evidence |
| :--- | :--- |
| ASCII typing and Unicode clipboard paste | Three paragraphs: `English`, `Українська`, `Русский é`; toolkit accessibility text retained the combining accent |
| Command+A/B/U, Shift+P, Enter | Selection retained; Bold/Underline on; 18.5-pt field applied |
| Command+Shift+period, Z, Shift+Z | Grow to 20 pt, undo to 18.5, redo to 20, with selection retained |
| Command+S, native save dialog | Saved local v2 document; status became Saved locally |
| Command+N/O, native open dialog | Reopened exact three paragraphs; field 20 pt and Bold/Underline on, empty session history |
| Unsaved replacement → New → Cancel | Temporary text retained and document stayed dirty |
| New → Discard | New empty document; saved file still contained original Unicode and 20-pt bold/underlined runs |

The saved file's decoded text/run properties were independently asserted after
Discard. Its SHA-256 is `c980222ae7823c9140b047536ac7d1a32afee2aede4e32fd0f06d72a564ead32`.
The first automated `typeText` attempt omitted Cyrillic characters and the combining
accent; clipboard paste worked. The cause was not isolated between input injection,
keyboard layout and application handling. Therefore non-ASCII character-by-character
native typing and OS IME remain unresolved and need a real keyboard/layout check.
No native screen-reader certification, Windows/DPI comparison, or complete native
Save/Discard/Cancel matrix is implied; the modal Save branch retains widget coverage.

## D01–D09 evidence map

| ID | Rust evidence | Remaining limits |
| :--- | :--- | :--- |
| D01 | Core insertion/split/merge/replacement/deletion; actual text-widget paste/type/select/replace and ribbon pointer click | Short documents; drag/drop not implemented |
| D02 | Exact English/Ukrainian/Russian, combining accents, non-BMP, ZWJ, flag boundaries; Shift+Arrow grapheme selection/deletion; Windows clipboard CRLF normalization; IME preedit/commit/cancel | Synthetic widget IME events are not native OS IME evidence. CJK/RTL/complex shaping and all writing systems not certified; missing emoji glyph visible in capture |
| D03 | Exact bold/italic/underline/size/alignment snapshots and native roundtrips; mixed-size galley geometry and scalar hit tests; actual Home controls | Font-family choice, named styles and lists unsupported; no pagination/fidelity claim |
| D04 | Exact text/format undo/redo, branch redo clearing, pending insertion style, selection restoration, widget shortcut history and one-commit IME transaction | 100 snapshot operations, no persistent history or time-based coalescing |
| D05 | Complete v2 roundtrip, every alignment, read-only Welcome v1 conversion, GUI save/reopen through shared session | No downgrade to v1, autosave or recovery |
| D06 | Core Save/Discard/Cancel and failure; pointer events activate all three real modal buttons; failed save keeps dirty session; close approval flag | macOS Save/Open and Cancel/Discard smoke recorded below; cancelled picker, modal Save and native window-manager close need further platform review |
| D07 | Real partial temporary write, injected errors before sync/replace, real replacement failure and missing parent; previous file bytes and dirty model preserved | No power-loss, directory fsync, concurrent-writer, ACL/extended-attribute or network-filesystem guarantee |
| D08 | Corrupt JSON/UTF-8, foreign type, unknown version/fields, duplicate keys, invalid styles/text and unsupported extension rejected; source unchanged | Bounded regression tests, not completed fuzzing |
| D09 | Actual egui keyboard and pointer event loop covers edit/selection, emphasis, undo/redo and save; model compared with projection | Limited macOS native keyboard/dialog smoke above; non-ASCII typing, IME, full focus/accessibility and Windows remain pending |

The native executable and fixed-size capture harness were launched and captured on
macOS. The earlier unbundled application was not exposed to Computer Use. During
the design refinement, a temporary local app bundle was tried; Computer Use reported
that permissions were not granted. No native keyboard/picker result is claimed
for those historical attempts; alpha.4 results are recorded above. The temporary bundle is not a distribution artifact. Word for macOS is not the approved Windows reference and was not used as
a substitute. In alpha.3, Computer Use inventory was available, but opening the
temporary review bundle timed out and reset its runtime; no manual keyboard pass
is claimed for this attempt. Windows itself and Word 2024 for Windows are unavailable here.

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
matrix in [UI_SPEC](UI_SPEC.md). Review the GUI toolkit against those results.
Keep exact Rust native-format/history regressions intact for further owner-approved work.
No DOCX/PDF/print/cloud/collaboration/macros/AI, installer, signing, merge or release
is included. Limits remain 100,000 scalars / 16 MiB and unmeasured long-document costs.
