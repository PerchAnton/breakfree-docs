# Rust editor validation and remaining gates

Evaluation: **2026-10-17, Europe/Kyiv**, macOS 27.0.1 arm64, rustc/cargo 1.99.1,
eframe 0.33.0 and locked egui/epaint 0.33.3. Rust is owner-approved; toolkit adoption,
license, supported-platform and full D1 reviews remain separate gates. Historical
Python/Qt results are preserved in [history/python-validation.md](history/python-validation.md)
and must not be presented as Rust results.

## Checks actually run

Current local checkpoint: **0.1.0-alpha.8**. Earlier capture/CI evidence retains its
original scope; this iteration does not establish native Windows behavior.

| Local check | Observed result |
| :--- | :--- |
| `cargo test --locked` | 68 passed: 23 engine/storage/presentation, 3 CLI subprocess and 42 actual egui widget event tests; no skipped tests |
| `cargo test --locked --no-default-features` | 26 passed; core and CLI build without GUI dependencies |
| `cargo fmt --check` / Clippy all targets with warnings denied | Passed |
| `cargo build --locked --release` | Native macOS binary built; no Office/sibling source inputs |
| Copied executable outside checkout | Alpha.8 release `--version` and `--capabilities` verified from a temporary directory without Office/source files; that copied binary also ran the native app-bundle smoke. Malformed-open and argument handling are covered by CLI subprocess tests |
| Repository references, contract and artwork | `python3 tools/check_repository.py` passed; approved logo embedded directly and archive copy byte-identical |
| Workflow YAML / whitespace | Ruby YAML parse and `git diff --check` passed |
| Native rendering | Alpha.8 root/QAT and View at 1366 × 900, More tabs and Zoom at 640 × 480 DIP inspected, backing scale 2; [captures](UI_SPEC.md#alpha8-native-captures) use deterministic fixture events. Earlier evidence retains its version context |
| Native macOS interaction | Alpha.8 Home emphasis, Alt+2/3 history, Alt+W,Q Zoom validation/application, collapsed View navigation, Draw/Help, QAT Save and system New/Open passed through Computer Use; exact Unicode and 11-pt bold runs decoded independently. Native Windows/AltGr/IME remains unverified |
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

## Alpha.8 quick-access, tabs and View validation

Seven new actual-widget tests cover fixed/disabled QAT numbering, shared history
and save/reopen, every tab scope, held-Alt multi-key sequences, hidden tab routes,
More tabs Escape/resize, View toggles, group navigation in a collapsed ribbon,
Zoom validation/cancel/apply/background-shortcut/close guards, document selection
and history preservation. Twenty-seven additional root/View/overflow width/scale
cases use 640/900/1366 DIP × synthetic 1/1.25/1.5; three minimum-window scale cases
check Zoom controls inside the modal. These are not native Windows DPI tests.

The tests exposed an initial focus fallback while the temporary View area was
still measuring. View now waits for enabled targets; placeholder tabs still focus
the tab. The overflow Escape path closes the popup explicitly. Capture timing now
waits at least 0.4 seconds as well as eight frames, avoiding partially faded menus.
All previous 35 widget and 26 engine/CLI cases remain passing.

Native macOS Computer Use exercised the normal release binary copied into a
throwaway app bundle outside the checkout:

| Action | Observed evidence |
| :--- | :--- |
| Unicode paste, Select All, F10 → H → 1 | Exact English, Ukrainian, Russian and combining-accent text remained selected; Bold became active |
| Alt+2 / Alt+3 | Bold became inactive / was restored; selection remained intact |
| Alt+W → Q | Percent received focus and selected the current value; 201 was rejected, replacement 125 plus Enter applied 125% and returned the document selection |
| Ctrl+F1, F10 → W → K | Navigation Pane appeared; ribbon returned to tabs-only mode |
| F10 → J → I; Escape → Y → 1 | Draw showed disabled tools and an Escape explanation; Help reported 0.1.0-alpha.8 without modifying text |
| Escape → 1, system Save; New/Open | QAT Save created the test .bfdoc; reopening restored all three paragraphs and bold state. Zoom remained view state, outside the file schema |

The saved native file was independently decoded: content v2, three left-aligned
paragraphs, exact `English é\nУкраїнська\nРусский`, every run bold, 22 half-points,
no italic/underline. SHA-256:
`1e76d97a48bb7358a50a77a8b2d046523f317d6cd75e6047c6552582f401ca27`.
Copied-executable information commands report alpha.8 and protocol v1 in a fresh
working directory without Office/source inputs. Four native render captures were
inspected separately. No controlled Word 2024, Windows DPI, native bare-Alt/AltGr,
physical non-ASCII typing or OS IME comparison was possible; earlier limits remain.

## Alpha.7 KeyTips validation

Seven new actual-widget tests cover bare Alt down/release/toggle, modifier chords,
AltGr with Windows command flags, focus loss, same-frame IME entry, prefix filtering,
invalid keys, Backspace/layered Escape, size entry, Font modal, menu/overflow routes,
pointer dismissal and restoration of search/document focus. Multiple access keys
and paired Text events in one frame cannot leak into the document; repeated keys
cannot create extra history entries. Unicode selection, formatting, undo/redo and
File-key save/reopen share the existing command engine. Nine combinations of
640/900/1366 DIP × synthetic 1/1.25/1.5 check badge containment and non-overlap.
Previous 28 widget and 26 engine/CLI tests remain passing.

Native macOS Computer Use exercised the release binary outside the checkout:

| Action | Observed evidence |
| :--- | :--- |
| Paste English/Ukrainian/Russian with combining accent, Command+A, Ctrl+F1, F10 | Exact three paragraphs; top-level hints and Home focus, collapsed preference retained |
| H then 1; undo then redo | Bold action returns to selected document; one undo/redo operation, no access-key characters inserted |
| Alt+H, F, S; type 19.5, Enter | Size field selected; 19.5 pt applied and temporary ribbon dismissed |
| F10, H, F, N | Font dialog shows 19.5 pt; Escape cancels |
| F10, H, S, L, Escape | Real Select menu, focused Select all; Escape closes menu only; further Escapes return through Home/root/document |
| F10, F, S; system Save, New, system Open | Local v2 file reopened with exact original Unicode text, bold 19.5 pt and empty history; Saved locally |

Independent JSON assertions checked all three text runs, bold true, italic/underline
false, `font_size_half_points: 39`. File SHA-256:
`f36aff0bf20062dd404d8e2550115836002a1f59053e6d2f8c42014b819714e8`.
The saved native test session was closed.

Bare Alt has synthetic event evidence only: the Computer Use tool rejects
modifier-only key presses. The egui adapter exposes modifier snapshots, so a very
short press/release coalesced between frames can be missed. Native Windows Alt,
AltGr, layout switching, OS IME, assistive technology and 100/125/150% are not
certified. Four actual-renderer captures are inspected, but no matched Word 2024
Windows image is available. Non-Home command hints and quick-access/Draw/Help tips
remain incomplete. No percentage of visual similarity is asserted.

## Alpha.6 ribbon validation

Six additional actual-widget tests cover temporary expansion with no workspace
reflow, outside dismissal, command focus return, Unicode selection/history/native
roundtrip, F10/Alt+H, Tab/Shift+Tab, spatial/group arrows, dropdown activation,
layered Escape, visible-region F6, size cancellation/commit and resize-to-More focus.
Temporary geometry also passes nine width/DPI combinations (640/900/1366 DIP ×
1/1.25/1.5). Existing 27 density cases, modal tests, IME projection and engine/storage
regressions remain passing. Initial focus regressions and first-frame overlay
measurement were corrected without weakening existing assertions.

Native macOS Computer Use on the alpha.6 release binary confirmed:

| Action | Observed evidence |
| :--- | :--- |
| Unicode paste, Command+A, Ctrl+F1, F10 | Three exact paragraphs retained; only tabs remain; Home receives focus |
| Tab, Ctrl+Right, four Tab steps, Space | Font group reached; Bold changes the selected text; temporary ribbon closes |
| Command+Z / Command+Shift+Z with separate observations | Bold off / on; original Unicode text retained |
| Alt+H, Font group, Down, Escape | Preset menu focuses 8; Escape closes only the menu and returns focus to its arrow |
| Down, Down, Enter | 9-pt preset applied to original selection; temporary ribbon closes |
| F6, F6, Shift+F6 | Focus visits 100% status, Home, then 100%; hidden navigation stays hidden |
| System Save, New, Open | Saved locally; reopened exact English/Ukrainian/Russian combining text, all bold 9 pt; empty history |
| Double-click Home | Pins expanded ribbon; subsequent Escape leaves it visible. Final build also returns document focus when switching to tabs-only mode |
| Click status zoom +, then F6 | Final build focuses Home from the status region, including after pointer input |

The saved file was independently decoded: exactly three expected text runs, all
bold with `font_size_half_points: 18`, no italic/underline. SHA-256:
`9daf859ddb8692b57a88778c45d92a2aedb303f8e024ca86ccdc8ce83561f9fa`.
The saved test session was closed. Native Windows DPI, Word matching, non-ASCII
character typing, OS IME and assistive technology remain unverified. F10/Alt+H
was the supported alpha.6 keyboard entry; alpha.7 evidence is recorded above.

## Alpha.5 Font and Editing validation

Shared-command tests cover partial Font patches, untouched mixed values, exact
selection/pending style/history snapshots, invalid grapheme boundaries, no-op and
caret-only commands, and save/reopen. Actual widgets cover launchers and Command/Ctrl+D,
Cancel/Escape, invalid size, Enter/OK, blocked background edits and window close,
composition disablement, direct Find/options and unchanged overflow behavior.
Dialog bounds and every recorded control fit 640 × 480 DIP at synthetic 1/1.25/1.5
scales, including a 400-pt clipped preview. These are not native Windows DPI results.
The initial dialog inherited horizontal child layout and exceeded the minimum width;
explicit vertical columns fixed it without weakening geometry assertions.

Native macOS Computer Use on the release binary independently verified:

| Action | Observed evidence |
| :--- | :--- |
| Paste English / Ukrainian / Russian combining text; Command+A/D | Exact three paragraphs; Font dialog opens, size field selected |
| Draft 19.5 pt + Bold Italic + Underline; Escape | Draft discarded; reopening shows original 11 pt, regular, no underline |
| Type 401; Enter | Visible validation error; dialog stays open with invalid size selected |
| Replace with 19.5, choose Bold Italic/Underline, Enter | All selected text changes together; original Unicode selection retained |
| Command+Z; Command+Shift+Z | One undo restores 11 pt/regular/no underline; one redo restores all four properties |
| System Save, New, system Open | Reopened exact three paragraphs, 19.5 pt + bold/italic/underline, Saved locally, empty history |
| Find primary click; separate options arrow | First click opens/focuses navigation; arrow opens Find/Navigation menu |

The saved v2 document was independently decoded and all text/run properties asserted.
SHA-256: `11ba3dafe02908d251a113325e717effc0e8afddf921134fb4009f45a7ad4df9`.
The test session was closed after saving. Non-ASCII character typing was not retested;
alpha.4's unresolved result below still applies. Native OS IME, assistive technology,
Windows scaling and a controlled Word 2024 comparison remain unverified.

## Native macOS alpha.4 smoke

The alpha.4 release binary was copied to a temporary app bundle and verified with
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
