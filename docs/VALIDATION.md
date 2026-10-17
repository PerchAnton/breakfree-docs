# Rust editor validation and remaining gates

Evaluation: **2026-10-17, Europe/Kyiv**, macOS 27.0.1 arm64, rustc/cargo 1.99.1,
eframe 0.33.0 and locked egui/epaint 0.33.3. Rust is owner-approved; toolkit adoption,
license, supported-platform and full D1 reviews remain separate gates. Historical
Python/Qt results are preserved in [history/python-validation.md](history/python-validation.md)
and must not be presented as Rust results.

## Checks actually run

Current local checkpoint: **0.1.0-alpha.18**. Earlier capture/CI evidence retains its
original scope; this iteration does not establish native Windows behavior.

| Local check | Observed result |
| :--- | :--- |
| `cargo test --locked` | 136 passed: 26 engine/storage/presentation, 6 styles/inheritance, 8 search, 7 batch replacement, 9 preferences, 4 CLI subprocess and 76 actual egui widget event tests; no skipped tests |
| `cargo test --locked --no-default-features` | 60 passed; preferences, core/styles/search/batch replacement and CLI build without GUI dependencies |
| `cargo fmt --check` / Clippy all targets with warnings denied | Passed |
| `cargo build --locked --release` | Native macOS binary built; no Office/sibling source inputs |
| Copied executable outside checkout | Alpha.18 release `--version` and `--capabilities` verified from a temporary directory without Office/source files; that copied binary also ran the native app-bundle smoke. Malformed-open and argument handling are covered by CLI subprocess tests |
| Repository references, contract and artwork | `python3 tools/check_repository.py` passed; approved logo embedded directly and archive copy byte-identical |
| Workflow YAML / whitespace | Ruby YAML parse and `git diff --check` passed |
| Native rendering | Alpha.18 gallery/spacing and minimum-width Styles overflow at 1366 × 900 / 640 × 480 DIP inspected at backing scale 2; [captures](UI_SPEC.md#alpha18-native-captures). Earlier evidence retains its version context |
| Native macOS interaction | Alpha.18: Home Heading 1, F10/H/L/N, Unicode input, italic/clear/undo/redo, system Save/Open, exact v3 readback and final-binary `--open`; gallery arrow/Enter No Spacing and undo/redo return to saved state. Isolated profile; clean close passed |
| Synthetic DPI | Actual widgets at 1/1.25/1.5 scale and 960/1366/1920 physical widths; ribbon/group/workspace/status bounds and collapse assertions passed; 27 additional width/scale combinations check individual Home control containment/nonoverlap and title/search/window bounds |

Server CI is configured for Windows, Ubuntu and macOS with the same Cargo checks
and native release build. Its observed run/commit results are recorded in
PROJECT_LOG after publication, separately from this local evidence.

The design refinement also checks actual pointer activation of italic/alignment,
selection preservation, quick Undo/Redo, disabled strikethrough/style controls leaving
the model unchanged, zoom ±/dialog/100%, File Home/Options/Back and dirty New/Cancel.
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

## Alpha.18 style inheritance validation

2026-10-17, Europe/Kyiv. Six new core tests exercise the fixed acyclic catalog,
explicit false versus inherited values, exclusive paragraph boundaries, mixed
selection, empty paragraph persistence, exact one-operation style/reset history,
resolved size stepping, split/merge, combining boundaries, previewed Replace All,
strict v3 parsing, older-format import and write/sync/replace failure preservation.
Existing v1/v2 fixtures remain unchanged. Older encoding tests still exercise real
v2 schema input; the writer's version and duplicate-key tests now target v3.

Three new actual-widget tests cover gallery pointer/menu/KeyTips at 1366/1000/640
DIP, temporary ribbon return, modal/composition guards, command search, semantic
save/reopen and resolved layout/hit testing. Empty paragraphs use their own style;
pending input affects only the active empty paragraph. Explicit coordinate checks
verify a 5.5-pt Normal gap and zero No Spacing gap. This caught a renderer defect:
Galley concatenation ignored the input rect's extra height. Placed rows and mesh
bounds now include the gap without adding fake characters. The existing resize
regression also exposed loss of ribbon focus during the first overflow-menu frame;
popup ownership now retains keyboard routing. All 136 full / 60 core tests pass,
including unchanged search, settings, CLI and failure-preservation suites.

Native normal-binary macOS test: applied Heading 1 to an empty document, typed a
heading/newline, used F10 → H → L → N for the next paragraph, pasted English,
Ukrainian, Russian and a combining accent, selected all and applied italic. Home
Clear removed overrides; undo/redo restored the intermediate state. Native Save
and Open succeeded. JSON readback confirmed exact text, first paragraph heading1,
three normal paragraphs and null direct overrides; reopening was clean with no
history. Unicode paste is evidence of preserved input, not physical IME coverage.

After the gap fix, the final release executable was copied outside the checkout
and launched through absolute `--open`. F10/H/L → Down focused No Spacing; Enter
applied it to the selection. Undo/redo/undo restored the saved mixed-style document
and clean state; close exited successfully. Two final capture-harness images were
inspected for palette, glyphs, active card/menu states, spacing and Home gutters.
The harness uses a synthetic committed v3 fixture and injected focus/menu events;
it is separate from the normal-binary native-input evidence. Release, fmt, normal
and capture Clippy, repository references/artwork, YAML and whitespace checks pass.

Not performed: native Windows/Word paired captures, 100/125/150% Windows scaling,
physical AltGr/IME and full accessibility. Custom/character-style definitions,
parent editing, automatic following styles, pagination and exact reference KeyTips
remain deferred. The fixed catalog is a reviewed increment, not completion of D2.

## Alpha.17 character-format reset validation

All 127 full tests pass (82 library, 4 CLI, 26 engine, 8 search, 7 batch
replacement); 54 pass without desktop. Three added core tests cover partial and
cross-paragraph selection, unchanged text/alignment, exact one-step undo/redo,
pending caret formatting without document changes, no-op redo preservation,
invalid scalar/grapheme ranges and native v2 save/reopen. The fixture includes
English, Ukrainian, Russian, combining marks and joined emoji; this is not proof
of support for every script.

Three added actual-widget tests exercise the eraser at full/compact/overflow
widths, F10/H/FC in a collapsed ribbon, actual Alt+Q search/Enter, Control-only
shortcuts for macOS/Windows-style modifier events, rejected AltGr/Shift/Command
variants, search focus, modal guards, pending input and IME start/commit. Existing
Home keyboard traversal now explicitly visits Clear before Bold. The existing
Home control/group/separator and KeyTip geometry matrices pass without relaxed
bounds. Fmt, normal/capture Clippy, release builds, repository/artwork/YAML and
isolated version/capabilities checks pass. No dependency or schema changed.

Native macOS captures at 1366 × 900 and 640 × 480 DIP inspect the applied clear
result and compact Font hints respectively. The original icon/palette and group
spacing are retained. In an isolated normal release app, synthetic English,
Ukrainian, Russian and combining text was pasted into two paragraphs, then made
bold/italic/underlined, centered and 24 pt. The Home eraser reset character
properties to regular 11 pt while retaining center alignment and selection.
One Command+Z restored the four properties; redo, F10/H/FC and actual Alt+Q search
also exercised the action. Native Save/Open passed. Separate JSON assertions
verified exact Unicode text, centered paragraphs, default styles and native v2;
reopen reset history and clean close passed. No user document/profile was changed.

Ctrl+Space itself has synthetic event coverage; native input-source interception
was not tested. Native Windows 100/125/150%, OS IME, physical non-ASCII typing,
full accessibility and matched Word 2024 captures remain unverified. Character
reset intentionally does not implement named-style inheritance, paragraph reset
or implicit word expansion at a caret.

## Alpha.16 Replace All validation

All 121 full tests pass (79 library, 4 CLI, 23 engine, 8 search, 7 batch
replacement); 51 pass without desktop. Seven new core tests cover read-only
preparation, exact mixed styles/selection/typing through one undo/redo, native v2
roundtrip, deletion and paragraph alignment, equal-text no-ops retaining redo,
invalid/overlapping/grapheme-splitting ranges, rejected controls/oversized output,
stale text or formatting, 50,000 matches, nonrecursive growing replacements and
combining-character output carets. Four added actual-widget tests cover initial
Cancel focus, Enter/Escape cancellation, Tab/Enter Apply, background shortcut and
close guards, stale/oversized previews, no-op disablement and replacement-field
IME confirmation. Nine physical size/scale cases check modal/button bounds and
virtual-row height. Completion notices clear after subsequent editor actions.

Two native macOS captures inspect the preview at 1366 × 900 and 640 × 480 DIP,
backing scale 2. Visual inspection found and corrected a virtual-row height
mismatch; assertions now check the actual 66-DIP row allocation. Palette, original
icons and Home spacing are unchanged. Fmt, normal/capture Clippy with warnings
denied, locked normal release builds, repository checks and isolated information
CLI checks pass. No dependency or document/protocol/settings schema changed.

A normal release app in an isolated profile pasted Ukrainian, Russian, English and
combining text in two paragraphs, applied Bold, and used keyboard Match case off /
Whole words on. Replace All preview showed three original matches and three
changes without changing the document. Initial Enter cancelled; reopening and
Tab/Enter applied all three. One QAT Undo restored the original, and Redo restored
all replacements. System Save/Open and separate JSON assertions confirmed exact
text, bold runs and native v2; reopen reset undo/redo and clean close passed. The
subsequent completion-notice cleanup is covered by the widget regression and the
rebuilt binary. No user profile or document was changed.

This is bounded plain-text context, not a full rich-text diff or per-match choice.
Native Windows 100/125/150%, physical non-ASCII typing, OS IME, full accessibility
and paired Word 2024 comparison remain unverified. Synthetic scales and CI do not
substitute for those gates.

## Alpha.15 search options validation

All 110 full tests pass (75 library, 4 CLI, 23 engine, 8 search); 44 pass without
desktop. Four added core tests cover English/Ukrainian/Russian case, Greek sigma,
Kelvin/long-s/dotted-I/simple-fold limits, literal pattern punctuation, grapheme
and word boundaries, apostrophes/digits/underscores, phrases across paragraphs,
rejected-overlap candidates, long-query errors and styled replacement/history/v2
roundtrip. The existing 50,000-result test also checks folded whole-word ranges.
Three new actual-widget tests cover Tab/Space/Escape, read-only toggles, clear,
query errors, replacement and undo/redo result refresh. Existing geometry tests
now contain both new controls at nine physical size/synthetic-scale combinations.
IME-confirmation and data-safety regressions remain passing.

Fmt, normal/capture all-target Clippy with warnings denied, locked release builds,
repository checks and isolated `--version`/`--capabilities` pass. The dependency
inventory adds regex and its resolved supporting crates; no schema/license change.

Two native macOS captures inspect options at 1366 × 900 and 640 × 480 DIP, with
the unchanged palette/icons. An isolated normal release app pasted three paragraphs
of English/Ukrainian/Russian/combining text, applied Bold, searched lowercase
`україна`, and used Tab/Space to turn Match case off and Whole words on. The count
changed from zero to two; Escape and Home Replace retained the options. Tab to the
replacement field and Enter on Replace changed only the first word. QAT Undo/Redo
restored the exact states. Native system Save/Open passed; separate JSON assertions
confirmed exact text, all bold runs and native v2. Reopen reset history and retained
window-local options; clean close passed. No user profile/document was changed.

This evidence does not establish Windows input/DPI, physical non-ASCII typing,
OS IME, full accessibility, language-specific word tailoring or paired Word 2024
pixel equivalence. Existing macOS Command+H/AX focus caveats remain applicable.

## Alpha.14 navigation validation

Four new headless tests cover English/Ukrainian/Russian, combining and joined
emoji boundaries, exact case/non-normalization, cross-paragraph matches, empty
queries, nonoverlap, 50,000 matches at the model limit, forward/backward wrapping,
bounded grapheme excerpts and replacement with formatting, undo/redo and v2
save/reopen. Four new actual-widget tests cover retained search focus through 50
successive Enter actions, Shift+Enter/Up/Down, rows beyond 40, Escape, partial-
cluster refusal, clear/replace/history refresh, IME confirmation and nine physical
size/synthetic-scale combinations. All 103 tests pass, plus 40 without desktop.

The focused-field regression also caught a pending document-focus request
stealing focus on Find entry; Find/Replace now clears that request. The existing
Escape router closes search after recognizing focus from the preceding frame,
while leaving active IME confirmation to the input widget. No earlier geometry
or data-safety assertions were weakened. Fmt, normal/capture Clippy with warnings
denied, release builds and isolated information-only CLI checks pass.

Normal-binary macOS input used an isolated profile and synthetic 58-paragraph
English/Ukrainian/Russian/combining-text content. Command+F, query paste,
Shift+Enter from first to 55th result, Enter wrapping to first, scroll/focus and
visible range highlighting passed. Command+H was intercepted by macOS; Home
Replace is the documented route there. This does not establish native Windows
keyboard/DPI, physical non-ASCII typing, OS IME or screen-reader acceptance.
Two native captures check the wide/minimum pane, result cues and preserved Home
palette. A second normal-binary run pasted a three-paragraph Unicode document,
applied Bold, opened Home Replace, traversed with Tab into the replacement field
and activated Replace with Enter. Undo/Redo restored the expected states. Native
system Save/Open passed; independent JSON decoding confirmed exact text, all bold
runs and content v2. Search/Enter/Escape after reopening and clean close passed.
Accessibility activation did not focus the replacement field in this driver;
keyboard Tab did. This is not full assistive-technology acceptance. No user profile
or existing document was changed. The final singular-result label correction was
verified by the subsequent build/tests; native behavior is otherwise unchanged.

## Alpha.13 palette validation

The change is confined to native chrome/vector painting and existing File controls.
All 95 existing tests pass, including strict Home gutter/control geometry,
keyboard/KeyTips, toolbar ordering/preferences, Unicode/history and save/reopen.
The 36 core-only tests, normal/capture-inclusive Clippy, fmt, normal release build,
repository/artwork and YAML/whitespace checks also passed. No snapshot of color
constants is presented as evidence of visual fidelity.

Three native captures were inspected at wide/minimum Home sizes and wide File.
The supplied macOS images were visually consulted for color roles; their exact
build, color profile and effective UI scale are unknown. No raw source images or
desktop content were copied into the project. Icon color availability, labels,
gutters, selection/focus and neutral backgrounds were checked in the running app.

The normal release binary ran outside the checkout with an isolated profile.
Native paste of English/Ukrainian/Russian/combining text, Select All, pointer Bold,
Undo and Redo retained the three paragraphs and selection. Cut/Copy became visibly
active while unsupported color/highlight/brush controls remained disabled. F10 F T
opened File Options; pointer Home and Back returned to the same document. F10 H
and Escape preserved ribbon navigation; closing used Discard on the synthetic
document. Native system Save/Open and settings restart were not repeated in this
visual-only iteration; existing automated roundtrips passed. Native Windows,
physical AltGr/IME and matched Word 2024 screenshots remain acceptance gates.

## Alpha.12 toolbar order validation

Five new actual-widget tests cover draft isolation, Apply/reload at both toolbar
positions, stable selected-command focus, keyboard menu entry and traversal,
background command/input/close blocking, Escape and pointer/keyboard Cancel,
failed-save byte preservation and Retry Saving, unchanged OK with a busy lock,
and empty/single/all-command lists. Eighteen geometry combinations cover those
three lists at 640/1366-DIP widths, 480-DIP height and synthetic scales 1/1.25/1.5.
Dialog bounds and every recorded control fit; boundary moves are no-ops.

A styled English/Ukrainian/Russian/combining-text document retains its selection,
history and supported model snapshot through reordering, preferences reload and
native-format save/reopen. Updated numeric KeyTips target the reordered real Undo
command. The original Home gutter and all previous keyboard/persistence tests pass.
The capture-feature all-targets Clippy check also passed with warnings denied.

Native macOS Computer Use ran the normal release executable from a temporary
app bundle outside the repository with an isolated profile. Paste of
`English e\u{301}\nУкраїнська\nРусский`, Select All and Bold prepared three paragraphs.
F10, 0 and the customization menu opened More Commands. Down then Alt+Up moved
Undo to the first row, retaining its accessible selected identity and visible
focus. Escape cancelled; opening again, Alt+Down and Enter applied Undo/Save/Redo.
F10 1 undid Bold and F10 3 restored it with the selection intact. The test document
closed through Discard. A normal restart displayed Undo/Save/Redo; independent
JSON decoding confirmed that exact order and above-ribbon placement. The empty
restarted window closed normally. User settings were not touched.

Two native captures were inspected for list/arrow alignment, focus, labels, footer
and minimum-window fit. Native system Save/Open dialogs and real OS IME were not
repeated in this iteration; automated save/reopen passed. Native Windows scales,
AltGr/accessibility and a paired Word 2024 comparison remain unperformed.

## Alpha.11 Home separator validation

The reported Clipboard/Font overlap was reproduced in the alpha.10 native render.
The new actual-widget regression failed before the fix: the Clipboard/Font gutter
was 2 DIP at 640 DIP width instead of the specified 8. The full-width child content
could also intrude into the reserved neighboring group. Allocating the parent
region once, then drawing its child without advancing the parent again, retains
the complete group width. One-DIP separators now sit at the center of each gutter.

The regression covers 54 combinations: 640/720/899/900/916/1195/1196/1366/1920 DIP,
synthetic scales 1/1.25/1.5, and pinned/temporary ribbon states. It checks exact
8-DIP group separation and that separator strokes intersect no recorded Home
control, including the overflow trigger. Existing strict control containment,
focus/KeyTips, document/history, persistence and preferences tests remain intact.

Four native captures were inspected for the font-family outline, emphasis states,
Clipboard/Font/Paragraph/Styles/Editing dividers, captions, launchers, overflow and
workspace boundary. No further overlap or caption clipping was observed in those
states. The capture fixture now fixes focus for temporary mode, matching its
existing KeyTips fixtures; this prevents launch focus loss from closing the
fixture before capture and does not change production focus handling.

The normal release executable ran outside the checkout with an isolated profile.
Computer Use pasted `English e\u{301}\nУкраїнська\nРусский`, selected all, clicked
Bold and entered 14 pt. Two undos restored 11 pt/nonbold; two redos restored
14 pt/bold with the selection intact. Ctrl+F1 and F10 H displayed temporary Home
and hints; Escape and repinning restored normal Home. Selected/hovered Bold and
the neighboring field/divider were visually inspected. The test session closed
through Discard. Save/reopen and settings restart were covered by automated tests
in this iteration, not repeated through native dialogs.

Windows native scaling and a matched Word 2024 comparison remain unperformed.
The known Welcome fixture emoji fallback remains unresolved. No broader visual,
complex-script or accessibility acceptance is inferred from these checks.

## Alpha.10 preferences validation

Nine new GUI-independent tests cover absolute profile/override paths, no startup
writes, ordered/empty toolbar roundtrip, malformed/foreign/future/duplicate/unknown
settings, bounded input, blocked parents/directories/Unix symlinks, read-only retry,
cooperating-writer lock/stale-window conflict, and failures after partial write,
before sync/replacement and before backup. Originals remain byte-identical; normal
failure cleanup allows retry. Reset retains exact original bytes before replacement.

Five added widget tests drive the actual customization/retry/reset controls and a
fresh `DocsApp` reading the saved profile. They assert order/placement/numeric hints,
unchanged Unicode document/selection/history, explicit reset cancellation and
confirmation, blocked background commands, corrupted-load warning/recovery and
reset-button containment at 640 × 480 DIP with synthetic 1/1.25/1.5 scales. The
reset-retry regression preserves the backup obligation even after further failed
customization, then saves the current draft and exact original backup. The
additional subprocess test proves information/rejected arguments neither create
missing preferences nor modify/read-error a corrupt profile. Windows CI omits the
Unix-specific symlink case; native Windows link behavior is not asserted.

Native macOS Computer Use ran the normal alpha.10 binary outside the checkout in
a temporary review app bundle with an isolated `BREAKFREE_DOCS_CONFIG_DIR`:

| Action | Observed result |
| :--- | :--- |
| F10, 0; keyboard enable Open; move below ribbon; close and restart | Save/Undo/Redo/Open restored in order beneath the ribbon; disabled history slots retained |
| Paste English/Ukrainian/Russian combining text; Command+A/B/Z/Shift+Z; Save | Three paragraphs and bold 11-pt runs retained; Saved locally |
| New; restored Alt+4; system Open | Exact saved document reopened; history empty, bold/11-pt unchanged |
| Reset via keyboard menu; Escape | Customized toolbar retained; no settings replacement |
| Reset; Tab then Enter | Defaults above ribbon; unique backup decodes to prior ordered below-ribbon toolbar |
| Make isolated settings read-only; enable New | Visible reason/path, warning and Retry Saving; original default settings unchanged |
| Restore write permission; Retry Saving | Warning clears; current toolbar saved with appended New |
| Restart with damaged settings; dismiss error; Reset, Tab, Enter | Defaults usable, original damaged bytes retained exactly in backup; settings valid again |
| Final binary: confirmed Reset with injected busy lock; dismiss error, remove test lock, Retry Saving | Original bytes preserved on failure; retry creates exact original backup before saving defaults, and menu reports Settings backup kept |

Independent decoding confirmed settings and backup contents and all document runs.
Document SHA-256:
`1e76d97a48bb7358a50a77a8b2d046523f317d6cd75e6047c6552582f401ca27`.
The Save picker initially treated an absolute filename as a literal name; the
new test fixture was moved to the intended temporary directory, then reopened
through the system picker. No user document was changed. The session was closed.

Three [native captures](UI_SPEC.md#alpha10-native-captures) were inspected; menu,
confirmation and long error path fit. The first sandboxed capture could not access
the desktop; rerunning the same helper with desktop access produced the captures.
Known fixture emoji fallback remains unresolved. The matched Word 2024 Windows
comparison, native 100/125/150%, physical Alt/AltGr, non-ASCII typing/OS IME and
screen-reader acceptance remain unverified. Settings persistence is not document
recovery; lock-crash/power-loss and external-writer limits are in ADR 0004.

## Alpha.9 toolbar and View validation

Six added actual-widget regressions cover customization/reset, visible-order
numeric hints, disabled history slots, keyboard focus/menu return, empty/below
placement, temporary-ribbon nonoverlap, direct Alt+4 with dirty New/Cancel, width
fit after window/pane changes, manual zoom/cancel behavior and exact model,
selection, history and save/reopen preservation. Eighteen geometry cases combine
640/960/1366 DIP, 1/1.25/1.5 synthetic scale and two toolbar positions with all five
commands. They check screen/workspace/title separation, target nonoverlap and
checkbox alignment. All previous 42 widget tests and 26 engine/CLI tests pass.

Native macOS verification used the normal alpha.9 release executable copied to
a temporary app bundle outside the checkout:

| Action | Observed evidence |
| :--- | :--- |
| Unicode paste, Select All, F10 H 1, Alt+2/3 | English/Ukrainian/Russian/combining text retained, Bold restored through shared history |
| F10 0, Space on New; reopen menu, Show Below | New appended as fourth command; toolbar moved below ribbon; accessible menu check state matched visibility |
| Alt+4, Cancel | Dirty-document Save/Discard/Cancel appeared; Cancel retained text and formatting |
| F10 W C, F10 W K | Canvas Width selected; displayed zoom changed from 162% to 132% when Navigation Pane reduced actual workspace |
| F10 W J, status percent, Escape | 100% exited fit; status opened Zoom with selected 100; Cancel retained scale |
| Ctrl+F1, F10 W | Temporary View and its hints stayed below the visible toolbar; no toolbar/control overlap |
| Escape, 1, system Save; New/Open | QAT Save wrote the fixture; reopening showed the same three paragraphs, four words and Saved locally |
| Reset Toolbar, F10 Up Right | Default toolbar returned above the ribbon; empty Undo/Redo were skipped and focus reached customization |

Decoded native v2 output contains exactly `English e\u{301}\nУкраїнська\nРусский`,
three left-aligned paragraphs, bold runs at 22 half-points, italic/underline false.
SHA-256: `1e76d97a48bb7358a50a77a8b2d046523f317d6cd75e6047c6552582f401ca27`.
Toolbar/view settings are absent from the document. Copied-binary information
commands report alpha.9 and protocol v1 from a temporary directory.

Four native renderer captures were inspected after correcting Show checkbox
alignment. Native Windows 100/125/150%, Word 2024 matched screenshots, physical
AltGr/non-ASCII input, real IME and full assistive-technology traversal remain
unverified. Unicode paste and synthetic event tests do not establish those gates.

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
