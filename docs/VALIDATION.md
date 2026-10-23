# Rust editor validation and remaining gates

Evaluation: **2026-10-20, Europe/Kyiv**, macOS 27.0.1 arm64, rustc/cargo 1.99.1,
eframe 0.33.0 and locked egui/epaint 0.33.3. Rust is owner-approved; toolkit adoption,
license, supported-platform and full D1 reviews remain separate gates. Historical
Python/Qt results are preserved in [history/python-validation.md](history/python-validation.md)
and must not be presented as Rust results.

## Checks actually run

Current local checkpoint: **0.1.0-alpha.57**. Historical evidence retains its scope.

| Local check | Observed result |
| --- | --- |
| Full / no-default-features tests | 306 / 149 passed; Shift+F10, all-marker navigation/activation, preview preservation and focus/resize cleanup checked |
| fmt / Clippy | Passed; normal/capture all-target checks with warnings denied |
| Release builds | Native binary and capture harness passed without Office/sibling inputs |
| CLI | Version/capabilities report alpha.57; existing CLI subprocess tests pass |
| Repository/artwork/YAML/whitespace | Passed |
| Native rendering | Keyboard ruler sources inspected at 1366 × 900 / 640 × 480 DIP |
| Native input | Not repeated this iteration; previous automation lost its window. CPU widget events are separate evidence |
| Existing regressions | Home geometry, minimum-width 64-style keyboard scrolling, history, storage and synthetic scaling checks retained |

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

## Alpha.19 editable-style validation

Five additional core tests cover parent/child resolution, explicit false/direct
precedence, exact definition/selection/pending-input undo/redo, no-op redo retention,
self/indirect cycles, non-default roots, v4 save/reopen, unknown/duplicate/missing
catalog fields, numeric limits, unchanged v3 source bytes and all three save-failure
stages. Text edits and batch replacements preserve the catalog; changed definitions
invalidate stale previews. Existing English/Ukrainian/Russian/combining-boundary
checks remain active. This is scoped evidence, not full writing-system support.

Four actual-widget regressions cover draft cancellation, invalid parents, one
history operation, modal keyboard/paste/close guards, minimum 640 × 480 DIP bounds
at synthetic 1/1.25/1.5 scales, F10/H/E/M, inherited layout/selection reporting,
parent-menu selection and Escape layering. A native check found dropdowns stayed
open after choosing a parent. Selection now closes them; Escape closes only the
open dropdown before cancelling the modal on a later press. Both behaviors have
regressions. Definition resolution is shared and computed once per paragraph for
range operations rather than allocated for each selected scalar.

Build, tests and captures use Rust 1.99.1 on macOS 27.0.1 arm64. No new dependencies,
Office services or sibling repositories were used. Protocol/preferences stay v1;
content advances to v4. Earlier builds reject new saves. Windows/reference/DPI,
physical IME, assistive-technology and long-document performance gates remain open.

Final normal-binary native check used an isolated temporary app bundle/profile,
with the executable copied outside the checkout. `--open` restored the saved v4
Normal definition (14 pt, 150%, 8.5 pt after). F10/H/L/H then H/L/M opened Heading 1;
clearing its size and choosing No Spacing closed the dropdown. Enter applied both
changes; one Command+Z restored them and Command+Shift+Z reapplied them. Save and
the system Open dialog succeeded. Readback asserted the Heading 1 parent, null
inherited size, all original direct overrides and exact `English café`, `Україна`,
`Русский` paragraphs. Reopened Normal showed its three saved numeric properties;
Cancel and clean close exited successfully. `--version`/`--capabilities` from the
same copied binary reported alpha.19 and unchanged protocol v1 with clean stderr.
The earlier native cycle/Cancel/context-menu check was repeated in widget tests;
Windows and full accessibility are not established by this scoped macOS exercise.

## Alpha.20 custom style validation

Seven new core tests cover definition-only and apply creation, empty paragraphs,
exclusive selection ends, direct overrides, exact undo/redo/selection/pending input,
custom descendants and reparenting, invalid IDs/names/parents/cycles, Unicode and
64-style/name limits. Reject invalid commands without losing redo. V5 roundtrip,
missing/duplicate/foreign/unknown data, write/sync/replace failure retention, strict
v4 migration without source rewrites, bulk edit preservation/stale previews and
the checked-in custom-style fixture pass. Existing v1–v3 tests remain passing.

Three new widget tests cover F10/H/E/C, focused Unicode name input, empty/duplicate
name rejection, invalid Enter focus, Cancel/redo preservation, optional application,
custom parent choice, command search, gallery/Modify targeting, 64 long names and
keyboard scrolling. Create modal/control containment passes at 640 × 480 logical
pixels at scales 1/1.25/1.5; long selected names/parents do not expand Modify.
The existing overflow test was updated for the real additional Create command.
Full result: 155 tests; 72 without GUI. Both Clippy configurations, fmt, release and
capture builds, repository/artwork checks, YAML and whitespace checks passed.

Native macOS smoke used the final release executable copied outside the checkout
(SHA256 `def7312523e8c6237d9ff6c5e2317a931fac36c8a16b1ccdfd7b312948f4cafd`).
`--new`, F10/H/L/C, a Heading 1 parent and `Звіт café` created/applied a style to
three paragraphs. One Command+Z restored Normal/11 pt; one Shift+Command+Z restored
bold/18 pt and the named style. A second style `Дочірній` inherited the custom
parent. Duplicate creation showed an error, disabled OK and retained name focus;
Escape left the saved document clean. System Save wrote v5. Independent JSON
readback matched `English café\nУкраїна\nРусский`, both exact names/IDs/parents,
and all three paragraph references. `--open` in a fresh process restored the child,
bold/18 pt, three paragraphs and Saved locally; clean close passed. Information
commands report alpha.20 and protocol v1 with clean stderr outside the checkout.
The automation's key-based `typeText` did not transmit Cyrillic/combining text;
the verified Unicode scenario used native clipboard paste. This is not a physical
IME/keyboard-layout certification. Windows/native DPI, paired Word reference,
AltGr/IME, accessibility and complex-script gates remain open.

## Alpha.21 rename validation

Four new core tests cover unchanged IDs, paragraphs, descendant definitions and
resolved formatting; exact history/selection/pending input; case-only names;
no-op/invalid-name/combined-invalid-graph redo preservation; updates at the 64-style
limit; combined rename/property edits in one transaction; dirty save checkpoints;
stale Replace All previews; v5 roundtrip; injected write/sync/replace failures
retaining the previous file. No content-version bump is needed.

Four new widget tests cover selected Name focus at minimum size and three synthetic
scales; duplicate/Cancel/no-op handling; combined parent/property/name undo;
built-in read-only names; real command-search focus; updated gallery/search labels;
identical name-only galley size/row placement; and context-menu modification of an
unused style without applying it. The latter exposed popup lifetime coupling:
rendering the context popup independently now preserves the target after the
source Styles menu closes. Existing regressions remain passing. Total: 163 full,
76 core-only; normal/capture Clippy, format, release builds, repository references,
artwork, workflow YAML and diff whitespace passed.

Native macOS check used the final ordinary release copied outside the repository,
SHA-256 `af46a940bd4249a8b20aca5286d9bf65876e0022c27d6ea89e2c605d53c23064`.
F10/H/L/M opened the selected custom style with its name selected. Unicode clipboard
paste renamed it to `Дочірній café`; one Command+Z restored the original name and
saved state, one Command+Shift+Z restored the rename. The Styles-list context menu
opened its unselected parent without applying it to text. `Normal` plus Enter
showed the duplicate error and retained name focus; replacing it with
`Заголовок café Русский` and saving succeeded. Independent JSON comparison proved
the document differed only in those two names: IDs, child parent, all paragraphs,
English/Ukrainian/Russian text with combining accent, and formatting were exact.

A new process opened the saved file through `--open`: the child gallery name and
renamed parent in Modify Style were correct, Undo was initially disabled and the
file was clean. Editing a draft and Escape left it clean. Both test processes
closed normally. Isolated `--version`/`--capabilities` reported alpha.21/protocol v1
with clean stderr. Native Save As was not repeated in this iteration; existing
Save As evidence retains its earlier scope.

Two final native renders were inspected separately (see UI_SPEC). Unicode native
input used clipboard paste; physical IME/AltGr, Windows 100/125/150%, screen-reader
acceptance and matched Word 2024 Windows comparison were unavailable. No broad
script-support, exact reference-parity or power-loss guarantees are inferred.

## Alpha.22 deletion validation

Five new core regressions cover independent paragraph and child-parent replacements;
indirect descendants; built-in children and No style; replacing paragraphs with a
surviving descendant; text/Unicode/direct overrides/alignment/selection/pending
input; unknown/self/cyclic references preserving history/redo; unused-style removal
at capacity and ID reuse; dirty checkpoints; stale Replace All plans; exact v5
roundtrip and injected write/sync/replace failures preserving the old file.

Four new widget regressions drive both actual selectors and Delete; check one
undo/redo, disabled invalid drafts, modal edit/file/history/close guards, layered
Escape and existing redo; cover long-name containment at minimum size and synthetic
1/1.25/1.5 scales; verify initial Cancel focus, keyboard commit, Home KeyTips,
unused-style context targeting, command search and composition/pending guards.
All 172 full / 81 core-only tests pass. fmt, normal/capture Clippy with warnings
denied, release/capture builds, repository/artwork/YAML and whitespace checks pass.

Native macOS verification used the final ordinary release copied outside the
checkout, SHA-256 `9ecc3d36a4df8e329dc57c1829bc039ed527047d1bae2a5c053b027c41885e93`.
F10/H/L/D opened the selected style's draft and showed two paragraphs/one child;
Enter activated the initially focused Cancel without mutation. The Styles-list
context menu then targeted Report Heading without applying it to text, showing
one paragraph/one direct child. Choosing its indirect descendant Report Note as
the new parent visibly rejected a cycle and disabled Delete. Choosing No Spacing
for paragraphs and Heading 1 for children, then Delete and Command+S, succeeded.

Independent JSON comparison checked the complete semantic document after Delete,
one Command+Z plus Save, and one Command+Shift+Z plus Save. Only the requested
custom definition and direct references changed; all text, English/Ukrainian/Russian
with combining accent, direct run overrides, alignment, surviving IDs and indirect
parent links were preserved. Comparison treats absent/null optional inherited
properties as equivalent; native serialization writes explicit null values.
Undo restored the complete original catalog and paragraphs, with Undo disabled.

A fresh process opened the result through `--open` with a clean state and no undo
history. Modify Style showed the surviving child based on Heading 1; moving to
the first paragraph showed No Spacing, regular 11 pt. Both processes closed
normally. The identical copied binary reported alpha.22/protocol v1 with clean
stderr for isolated `--version`/`--capabilities`. Native Save As and new text input
were not repeated; earlier evidence retains its scope. Two final native captures
were inspected separately; native Windows DPI, physical IME/AltGr, full assistive
technology and controlled Word 2024 Windows comparison remain unverified.

## Alpha.23 following-style validation

All 180 full tests and 86 without GUI pass. Five new core cases cover start/middle/
end/selected/empty/cross-paragraph Enter, reciprocal successors without inheritance,
missing references preserving redo, combining-boundary rejection, paste isolation,
deletion remapping, strict v5-to-v6 migration, dirty checkpoints and injected
write/sync/replace failures. Three actual-widget cases cover the selector,
combined typing/Enter, cancel/invalid drafts and Enter next to an existing separator.
The last regression also replaces a selected separator without changing plain
text: the semantic style still changes and one Undo restores exact selection.

A native check exposed an ambiguous text diff around equal adjacent newlines.
`text_change_from_selection` now uses the actual original edit range when the
unchanged prefix/suffix match, rather than moving an Enter to a neighboring
paragraph. The final native run and widget regression verify this correction.

fmt, normal/capture Clippy with warnings denied, locked full/core tests, normal/
capture release builds, repository/artwork checks, YAML parsing and whitespace
checks passed. Two final captures at 1366 × 900 and 640 × 480 DIP were inspected;
existing synthetic 1/1.25/1.5 dialog/ribbon regressions also pass.

### Native macOS final-binary check

Copied ordinary release SHA-256:
`d187198af52cfbb43b060d7b60c399a247b7daa892bc8187a4996a138e5487ab`.
The app-bundle executable matched the release bytes. From a temporary working
directory it reported alpha.23 and clean protocol-v1 capabilities without Office.

On a temporary copy of FollowingStyles.bfdoc, moved to Heading 1 and opened Modify
Style via F10/H/L/M. Changed Following paragraph from Normal to No Spacing through
the actual selector, accepted and saved. Enter at the heading end before an
existing paragraph separator created an empty No Spacing paragraph. Saved, then
performed one Undo/Save and one Redo/Save. Independent complete JSON comparison
confirmed exactly the configured successor and expected inserted paragraph, with
all other catalog/paragraph/text/Unicode/direct-format data unchanged. Comparison
normalizes absent versus null optional properties, not document semantics.

Fresh-process --open restored six paragraphs with clean history; reopening Modify
Style showed No Spacing as the saved successor. Escape left the file clean. No
physical IME/AltGr or new Unicode typing certification follows from fixture
preservation. Native Windows 100/125/150%, matched Word build, full accessibility,
soft line breaks and unusual mixed paste/IME/keyboard batches remain unverified.
Content v6 changes compatibility: alpha.22 and earlier reject new saves; use Save
As to retain older documents. No new save atomicity/recovery claim is made.

## Alpha.24 character-style validation

Final local checks: **189 full / 92 without GUI**, including six character-style
engine/storage regressions and three new real-widget regressions. Existing CLI,
Unicode, save-failure, style deletion/following, search/Replace All and synthetic
DPI checks remain passing. fmt, normal/capture all-target Clippy, release/capture
builds, repository/artwork checks, YAML and whitespace checks passed.

New checks cover paragraph → character → direct precedence and explicit false;
selection apply/remove/clear, caret history without dirtying, Enter, mixed/invalid
Unicode ranges, no-op redo preservation, parent modification, Replace All, v7
roundtrip, frozen v6 schema, unknown references and injected write/sync/replace
failures preserving previous bytes. Widget tests exercise wide/narrow KeyTips,
undo/redo, pending input, IME guard and scrolling to hidden rows. KeyTips no longer
anchor to clipped menu rows and overlap unrelated controls.

Native macOS smoke used the final release binary copied into a temporary app
bundle and a disposable CharacterStyles fixture. Select All → F10/H/L/B → Save
set every run reference to Strong while preserving direct false, paragraph styles,
catalog and Unicode. One Cmd+Z/Save restored the original supported document;
Cmd+Shift+Z/Save restored the Strong result. A fresh process opened that saved v7
file; F10/H/L/Z/Save removed references while preserving all direct properties.
Independent full JSON comparisons verified each checkpoint (only absent/null
optional fields normalized for the original hand-authored fixture). No user file
was used. The copied binary reports alpha.24 and protocol-v1 capabilities outside
the checkout, without Office or sibling repositories.

Two final native captures were inspected: character formatting is visibly resolved
and the narrow popup fits the window with clipped rows scrollable by keyboard.
The fixture's emoji renders a missing-glyph box on this machine; its Unicode scalar
remains intact in storage. Custom character definitions, native Windows input and
100/125/150% scaling, matched Word screenshots, physical IME/AltGr and full
accessibility remain unverified. The supplied macOS references establish palette
intent only. No release/merge or complete D2 acceptance follows from these checks.

## Alpha.25 named character-style validation

Final local checks: **198 full / 97 without GUI**, with five new engine/storage
cases and four new actual-widget/layout cases. fmt, normal/capture all-target
Clippy, release/capture builds, repository/artwork, YAML and whitespace passed.

Coverage: create/apply in one exact transaction; definition-only and caret creation;
Unicode names/text and explicit direct overrides; invalid/reserved/duplicate names,
IDs, nested references, capacity and overflow; invalid-input redo preservation;
font stepping, clear, Enter, Replace All and exact v8 save/reopen; strict v7 imports
and malformed custom references; injected write/sync/replace failure retention.
Widget checks exercise creation KeyTips, name input/OK, invalid/Cancel/IME/modal
guards, 640 × 480 at synthetic 1/1.25/1.5 scale and actual custom font layout.

The final copied release binary reported alpha.25 and protocol-v1 capabilities
from /private/tmp without Office/sibling sources. Native macOS smoke opened a
copy of the v7 fixture, selected all text, used F10/H/L/K, pasted the Unicode name
Примітка é, selected Italic On, entered 16 pt with Tab navigation, confirmed and
saved. One Cmd+Z/Save restored the original document with an empty v8 catalog;
Cmd+Shift+Z/Save restored the definition and every reference. Full JSON comparisons
verified each state, normalizing only absent/null optional fields in the authored
fixture. A fresh process reopened the file: 16 pt/Italic and the exact name remained;
arrow navigation reached the custom menu row and Enter returned to the document.

Two native dialog captures were inspected at 1366 × 900 and 640 × 480 DIP. The
palette/icons are unchanged. Native AX field clicks did not transfer text focus in
this automation session; Tab navigation worked. This is not full accessibility
acceptance. Windows/Word comparison, native DPI/IME/AltGr and emoji fallback remain
open. Custom character modification/rename/delete and parent graphs are deferred.

## Alpha.26 character-property validation

202 full / 99 core-only tests pass. Two engine regressions cover exact definition
history, stable identity/order/run references, selection/pending preservation,
direct overrides, Unicode roundtrip, unused definitions, invalid IDs/built-ins/
nested references and no-op redo retention. Two widget regressions cover draft
validation/Cancel/OK, actual rendered italic/size, one Undo/Redo, modal guards,
read-only names and minimum-window bounds at synthetic 1/1.25/1.5 scales.

The release executable copied outside the checkout answered version/capabilities.
Native macOS: select Annotation text in a disposable NamedCharacterStyles fixture,
F10/H/L/J, replace selected size 14 with 18, Enter, Save. Full parsed JSON comparison
(normalizing absent versus null optional fields) confirms only definition size
changed: catalog IDs/names/order, runs, direct formatting, text and paragraph
styles remain equal. One Undo/Save equals the original fixture; Redo/Save equals
the changed document. A fresh process reopens size 18; a 30-pt draft cancelled with
Escape leaves size 18 and no dirty state. Two native captures were inspected.

fmt, normal/capture Clippy, full/core tests, release/capture builds, repository
links/artwork, workflow YAML and whitespace checks pass. Capture harness was rebuilt
and linted after its new mode was added. Existing malformed-file, unsupported
version and failure-retention regressions pass. Native format remains v8.
Windows/Word/DPI/physical IME/AltGr/full accessibility remain unverified; no new
platform acceptance is claimed. Renaming/deleting character styles and parents
remain deferred. Earlier entries above retain their historical scope.

## Alpha.27 character-style renaming validation

205 full / 101 core tests pass. New engine tests verify stable IDs/run references,
selection/pending format, rename-only and combined property history, Unicode v8
roundtrip, unused definitions, case-only names, reserved/duplicate/invalid names,
fixed built-ins, missing IDs and unchanged redo. Widget regression verifies the
editable name, invalid OK, Cancel, combined name/size commit, updated command label
and exact one-step history. Existing minimum-window/synthetic DPI tests pass.

Native macOS: select Annotation text in a disposable fixture, F10/H/L/J, paste
`Примітка é` into the initially selected name, Enter and Save. Full JSON comparison
(normalizing absent/null optional fields) confirms only the name changed. One
Undo/Save exactly restores the fixture; Redo/Save exactly restores the renamed
file. A fresh process shows the saved Unicode name. Two native captures inspected.
Copied release version/capabilities, fmt, normal/capture Clippy, full/core suites,
release/capture builds, repository/artwork/YAML/whitespace checks pass.
Windows/Word/DPI/physical IME and complete accessibility remain unverified. Native
format v8, protocol/settings v1, dependencies, palette and icons remain unchanged.

## Alpha.28 character-style deletion validation

209 full / 103 core-only tests pass. Engine regressions cover custom/built-in/None
replacements, all text references, pending input, selection, direct properties,
exact one-operation undo/redo, unused definitions, self/missing replacement and
fixed/missing targets. Invalid commands preserve redo; native v8 roundtrip passes.
Widget regressions cover cancellation, replacement, shared history, modal/IME
guards, unused-style targeting and 640 × 480 bounds at synthetic 1/1.25/1.5 scale.

Native macOS: select Annotation text, F10/H/L/X, choose Regular note, Tab from
Cancel to Delete, Enter and Save. Complete parsed JSON comparison (absent/null
optional fields normalized) verifies only removal of definition 1 and remapping
its run references to 2. One Undo/Save equals the original fixture; Redo/Save equals
the expected deleted document. Fresh-process reopen shows Regular note on the remapped selection; Enter on the initially focused Cancel leaves the file unchanged. Two native captures inspected without overlap.
Copied executable version/capabilities and all fmt/Clippy/build/test/repository/
artwork/YAML/whitespace gates pass. Existing invalid-file/failed-save tests pass.
Native Windows/Word/DPI/physical IME and full accessibility remain unverified.
No schema, dependency, license, palette or icon change.

## Alpha.29 character-parent validation

2026-10-18, Europe/Kyiv. Three new core tests cover multi-level inheritance,
explicit false, direct overrides, stable references, pending input, one-operation
history, self/indirect cycles, missing parents, deletion reparenting and rejection
of descendant replacements that would create cycles. V8 imports gain no parent;
v8 rejects parent fields, v9 roundtrips and rejects malformed graphs, and invalid
saves preserve previous bytes. One new widget test checks parent validation,
preview, rendered inherited formatting and exact undo/redo. Existing dialog and
Home geometry/scaling regressions remain enabled.

Normal release native macOS smoke: select Annotation, F10/H/L/J, choose Strong,
OK and Save. Bold changes from off to on; one Command+Z/Save restores the entire
serialized document except the intentionally migrated v9 header; Redo/Save exactly
restores the saved document. Fresh-process reopen displays Strong in the parent
selector with inherited Bold. Wide and minimum-window captures were inspected.
Windows/Word comparison, native DPI/IME and complete accessibility remain unverified.

## Alpha.30 formatting provenance validation

Three core regressions distinguish authored overrides from identical inherited
values, explicit false, character and paragraph parents, independent mixed
values/sources, pending typing, Unicode scalar selection, separator-only selection,
and source changes through Undo/Redo. A widget regression opens the actual pane
at three scales and verifies no document mutation. Adjacent identical runs reuse
the previous inspection. Provenance is read-only and uses the renderer's resolved
values; names/sources come from authored definitions. Legacy explicit imports
remain labeled direct rather than guessed as inherited.

Native macOS keyboard smoke and two captures passed as listed above. This stage
does not re-establish native Save/Open evidence; storage roundtrip/history tests
passed. Windows/Word side-by-side, native 100/125/150% scaling, IME and full
accessibility remain unverified. Paragraph spacing/alignment provenance is deferred.

## Alpha.31 paragraph provenance validation

Core tests verify root defaults, named/inherited paragraph intervals, explicit zero,
equal values with different sources, mixed alignments, caret/empty paragraphs,
Unicode scalar and exclusive-end boundaries, Undo/Redo and v9 save/reopen.
Read-only inspection preserves serialized bytes and does not consume history.
The GUI test opens the actual pane at three scales and verifies document/history
behavior. Normal-release native input checks No Spacing, keyboard centering and
separate Undo/Redo of both commands. Two native render captures were inspected.
The previous temporary app bundle failed to launch; a fresh disposable bundle
opened an untitled document, used for the input test. No native file-open claim
is based on that launch. Storage roundtrip remains covered by core tests.

Alignment has no inherited source in v9; the display reports the stored paragraph
value without inventing authorship. Intervals have no direct paragraph override
or space-before field. Windows/Word comparison, native Windows DPI/IME and full
accessibility remain unverified.

## Alpha.32 style-filter validation

Core coverage checks scope/name intersection, trimmed lowercase English/Ukrainian/
Russian substrings, combining sequences and the explicit absence of canonical
normalization. Widget tests cover narrow-menu F3/text/Down/Enter/Undo, empty scope,
Reset, pointer category switching and unchanged document state. These exposed
KeyTips consuming F3, focus restoration stealing text input and default popup
click closure; routing now hands F3 to the filter and filter clicks keep the menu
open. Existing action dispatch still closes it when applying a style.

Native normal-release smoke independently checked wide-menu focus/search/application,
Undo, Reset and all categories, including empty Custom. Initial batched automation
sent paste before the popup was ready; it was undone, then each observed focus/state
was checked before proceeding. Two native captures were inspected. Windows/Word,
native Windows DPI/IME and full accessibility remain unverified. No schema change;
this iteration does not claim a new native Save/Open check.

## Alpha.33 used-style validation

227 full / 115 core-only tests pass. New core cases cover paragraph and character
parent chains, unused siblings, next-style exclusion, direct overrides, empty
paragraphs, pending typing, undo/redo and v9 save/reopen. The actual-widget test
checks the used checkbox, inherited Normal, excluded No Spacing, Reset and
unchanged document/history. Normal/capture Clippy, fmt, release builds and
repository/artwork/YAML/whitespace checks pass.

Two native capture-harness images were inspected at 1366 × 900 and 640 × 480 DIP.
The ordinary macOS binary launched and applied Heading 1; subsequent menu actions
did not produce the expected popup and automation reported noWindowsAvailable.
Native used-filter keyboard acceptance is therefore incomplete; the harness and
CPU widget checks are not a substitute. No new native Save/Open claim is made.
Windows 100%/125%/150%, matched Word comparison, physical IME and full accessibility
remain unverified. Protocol/settings v1 and document writer v9 are unchanged.

## Alpha.34 usage-caption validation

Core tests distinguish direct, parent-only and unused paragraph/character styles,
including an ancestor encountered before a later direct reference, Undo/Redo and
v9 roundtrip. The existing minimum-window widget regression now asserts both
Direct use and Parent dependency captions without changing document/history.
Two context-menu tests filter their target into view because secondary captions
make list entries taller; their rename/delete assertions remain. The 64-style
keyboard scrolling regression remains unchanged.

No native keyboard acceptance or matched Word/Windows DPI/IME/accessibility claim
is made. This changes only derived usage reporting and presentation; writer v9,
protocol/settings v1, palette, icons and dependencies are unchanged.

## Alpha.35 dependency-consumer validation

The new core fixture covers transitive/shared paragraph and character ancestors,
direct-use parents, self exclusion, deduplication, unassigned intermediates, Unicode
names, v9 roundtrip and bounded invalid cycles. History assertions verify consumer
changes through Undo/Redo. The minimum-window widget test keeps Required by visible
when a name filter hides the consuming style, without document/history changes.
Existing 64-style keyboard scrolling, context commands and synthetic scale tests
remain. Native keyboard input is not repeated this iteration; harness captures
are rendering evidence only. Windows/Word/DPI/IME/accessibility gates remain open.

## Alpha.36 dependency-navigation validation

A new minimum-window actual-widget test activates Required by with pointer and
Enter, verifies name/category reset, used-only preservation, popup persistence,
destination focus/visibility, unchanged document/undo depth and Escape closure.
The first keyboard test read targets before the popup's next layout frame; it
now waits one UI frame before locating the link. Existing style/filter/history
and 64-style scrolling regressions remain. Native input was not repeated; the
capture harness provides rendering evidence only. Windows/Word/DPI/IME and full
accessibility acceptance remain open. No storage or dependency changes.

## Alpha.37 return-navigation validation

The actual-widget pointer/keyboard navigation regression now changes all filters
after jumping, then uses pointer/Space on Back to parent and verifies exact
restoration, original-row focus/visibility, menu persistence and unchanged
document/undo depth. A separate transient-state test checks first-origin retention
through chained character-style targets and disposal when no popup is open.

A test exposed target-index drift when Back first appeared; resolving a jump on
the next menu layout prevents that drift. Visibility assertions wait 12 frames
for the existing scroll animation to settle. Full existing regressions remain.
Native keyboard input is not repeated; captures are rendering evidence only.
Windows/Word/DPI/IME/accessibility acceptance remains open.

## Alpha.38 named return validation

The existing pointer/Space return test now asserts Back to: Normal and bounded
button width. A new actual-widget case uses a long Ukrainian/combining character
style name, checks the 220-DIP menu bound and window edge, then renames the same
ID and verifies the displayed name updates. The toolkit expands the requested
210-DIP minimum button to the menu's 220-DIP width; assertions use that actual
layout contract. Existing return/filter/history and scrolling checks remain.
Ordinary native input was not repeated; harness captures only establish rendering.
Windows/Word/DPI/IME/accessibility gates remain open.


## Alpha.39 dependency navigation validation

Final gate: 234 full / 117 core tests, fmt, normal/capture all-target Clippy with
warnings denied, release/capture builds, repository/artwork/YAML/whitespace and
CLI version/capabilities passed. Two new widget tests cover renamed/deleted/no
longer used destinations with exact filter restoration and filter focus; a real
32-style reverse-order character inheritance chain with two directly assigned
styles, repeated Enter navigation, scrolling and return. Document and undo depth
remain unchanged by navigation. Reset cannot revive the stale destination.

Two native harness captures at 1366 × 900 and 640 × 480 DIP were inspected; the
palette, icons and existing return menu remain intact. This harness injects
widget events, not physical input. Ordinary native input was not repeated;
Windows 100/125/150%, matched Word comparison, physical IME and accessibility
remain unverified. No content/protocol/settings/dependency/license changes.


## Alpha.40 Windows acceptance preparation

No suitable Windows desktop was available; the owner confirmed no access could
be provided. Native Windows keyboard and OS display scaling were **not tested**.
The macOS host's installed remote-desktop client is not Windows GUI evidence.

Local gate: 236 full / 118 core tests, fmt, normal/capture all-target Clippy,
release/capture builds, repository/artwork/YAML/whitespace and CLI checks passed.
The new matrix exercises 18 combinations: three DIP sizes × synthetic scales
1.0/1.25/1.5 × expanded/collapsed Home. F10 or Alt+H, F3, Down, Tab/Shift+Tab,
Enter, Up, Space and Escape use widget events without manually assigning a menu
index. Destination focus, Back bounds, exact filters and unchanged document/undo
are asserted. It does not test bare Alt routing, AltGr, native DPI or OS IME.
The fixture test validates 32 character styles, transitive consumers, combining
Unicode text and exact native save/reopen.

The prior alpha.39 Windows job's test and Clippy steps were observed successful;
its release build was still running when inspected:
[Windows job](https://github.com/PerchAnton/breakfree-docs/actions/runs/39131923187/job/117688671545).
This is evidence for that commit, not alpha.40 or native acceptance.

The updated workflow will upload an EXE, fixture, checklist and SHA/version
metadata after successful Windows checks. Artifact publication and alpha.40 CI
completion are separate from these local results. Follow
[WINDOWS_ACCEPTANCE](WINDOWS_ACCEPTANCE.md); all native result cells remain Not run.
No new screenshots, runtime, dependency, schema, palette or icon changes.


## Alpha.41 direct paragraph spacing validation

244 full / 123 core tests passed. Added five core tests for transactional changes,
inheritance reset, mixed/partial/empty/exclusive-end selection, invalid range and
redo preservation, Enter/join/style-change/Replace All behavior, provenance,
v9 import, strict v10 save/reopen and injected write/sync/replace failures.
Existing frozen-reader tests retain legacy field rejection. Three widget tests
exercise real field input, OK/reset/Cancel, invalid decimal values, modal command
guards, one Undo/Redo, effective layout row heights and nine size/scale cases via
F10 → H → PS. No direct menu-index injection is used by these dialog checks.

fmt, normal/capture all-target Clippy, release/capture builds,
repository/artwork/YAML/whitespace and CLI version/capabilities passed. Native
macOS capture harness images at 1366 × 900 and 640 × 480 DIP were inspected;
this is rendering evidence, not physical keyboard/IME acceptance. Ordinary native
input was not repeated. Windows/reference/native DPI acceptance is deferred by
owner instruction, not completed. Palette/icons/dependencies/license unchanged.

Writer v10/readers v1–v10; process/settings v1. Alpha.40 and earlier reject v10
saves. Space-before, exact line-height units, indentation and pagination remain
unimplemented. Alpha.40 code/journal CI succeeded; alpha.41 CI is reported in the
project log separately from local results.

## Alpha42 space-before validation

2026-10-18, macOS arm64, Rust 1.99.1: **249 full tests / 127 core-only tests**
passed. fmt, all-target Clippy (normal and capture, warnings denied), release and
capture builds, repository/artwork checks, workflow YAML and whitespace passed.
Coverage includes inherited/direct zero, partial/reset transactions, invalid/no-op
redo preservation, Unicode selections, Enter/join/Replace All, v11 roundtrip, v10
migration, older-schema key rejection and injected write/sync/replace failures.
Actual-widget input sets before with the other intervals in one undo; reset/redo
restore exact documents. Layout tests cover wrapped lines, empty paragraphs, first
paragraph and additive intervals. Nine synthetic scale/size modal checks pass.
Two native capture-harness images inspected (UI_SPEC); ordinary native input not
repeated. Native Windows and matched Word reference checks remain deferred.
Server CI is reported separately in the project journal; no merge or release.

## Alpha43 paragraph-indent validation

2026-10-18, macOS arm64 / Rust 1.99.1: **254 full tests / 130 core-only tests**
passed. fmt, normal/capture all-target Clippy with warnings denied, release/capture
builds, repository/artwork/YAML/whitespace checks passed. CLI reports alpha.43 and
protocol v1. No new runtime dependency or sibling repository requirement.

Coverage includes independent inherited/direct left/right values, explicit zero,
exclusive selection boundaries, reset, parent modification, one-step undo/redo,
no-op redo preservation, invalid ranges/values, Enter/join preservation, v12
roundtrip, legacy rejection/migration and write/sync/replace failure preservation.
Widget tests type indents with spacing in one operation; invalid input and Cancel
leave the model intact. Style dialog changes remain a single history transaction.
Layout tests cover wrapped/empty paragraphs, all four alignment modes, Unicode
scalar hit testing at grapheme boundaries and narrow viewport scaling.

The extended style dialog initially exceeded the 640 × 480 DIP window; compact
grid spacing and preview height corrected it. Existing create/modify/rename
keyboard and synthetic size/scale tests now pass. Native capture-harness images
are recorded in UI_SPEC. Ordinary native keyboard/IME input was not repeated;
Windows 100/125/150% and matched Word reference checks remain deferred. Server CI
is recorded separately in the journal. No merge/tag/release.

## Alpha44 special-indent validation

2026-10-18, macOS arm64 / Rust 1.99.1: **258 full tests / 133 core-only tests**
passed. fmt, normal/capture all-target Clippy with warnings denied, release/capture
builds, repository/artwork/YAML/whitespace checks passed. CLI reports alpha.44 and
protocol v1. No new dependency or sibling checkout requirement.

Coverage includes inherited first-line/hanging values, direct zero, reset, parent
modification, mixed/empty selection, exclusive range boundaries, one Undo/Redo,
no-op redo preservation, out-of-range rejection, Enter/join/Replace All, v13
roundtrip, v12 migration, old-key rejection and injected save-failure preservation.
Widget input combines special and other geometry fields into one operation; type
selection/reset and style modification retain history semantics.

Layout tests cover both signs, zero/extreme values, mixed character formats, empty
paragraphs and all four alignment modes. Row character totals and paragraph-break
counts remain exact; selected scalar positions hit-test back correctly. Tested
English/Ukrainian/Russian/combining examples retain grapheme boundaries at wraps.
This is not a claim of complete script/shaping support. Distinct first/suffix
layout retains real row metrics and mesh counts without a synthetic newline.

Three native capture-harness images inspected (UI_SPEC). Existing minimum-window
and synthetic 100/125/150% scale checks pass. Ordinary native keyboard/IME input
not repeated; Windows/reference acceptance remains deferred. CI is recorded
separately in the journal; no merge/tag/release.

## Alpha45 expanded indent validation

2026-10-18, macOS arm64 / Rust 1.99.1: **262 full tests / 133 core-only tests**
passed. fmt, normal/capture all-target Clippy with warnings denied, release/capture
builds, repository/artwork/YAML/whitespace checks passed. CLI reports alpha.45,
protocol v1; native content stays v13. No new dependency/license change.

The 108-case layout matrix combines 80/145/500 DIP widths, 100/125/150% synthetic
scales, -144/0/+144 pt special indents and four alignments. It checks actual scale,
text reconstruction, real paragraph delimiters, tested grapheme wrap boundaries,
mesh counts, positive/nonoverlapping row heights, model immutability and cursor
hit testing. Fixtures include long mixed-format English/Ukrainian/Russian text,
combining marks, leading/trailing spaces, empty paragraphs and unbroken words.

A failing matrix case reproduced a justified soft-wrap-end hit at scalar 351
returning 350. Logical row extent now includes the final whitespace glyph after
visual alignment. A focused regression also checks transient input projection.
Actual-widget selection survives 50/125/200/100% zoom, paste, one Undo/Redo and
save/reopen without losing indents. Existing visible-column assertions now exclude
trailing whitespace; its logical caret extent may exceed the visible text edge.

Alpha.44 [Windows CI](https://github.com/PerchAnton/breakfree-docs/actions/runs/39136311155)
reported Create Style bounds y=-2…482 in a 480 DIP window. A local taller-font
regression also failed before the compact-layout fix and passes afterwards at
three scales. Server confirmation of the revised code is recorded separately in
the journal; no native Windows acceptance is inferred.

Three native capture-harness images inspected (UI_SPEC); ordinary native keyboard,
IME, accessibility and matched Windows/reference comparison remain unverified or
deferred. Interactive ruler handles are not implemented. No merge/tag/release.

## Alpha.46 interactive ruler validation

2026-10-18, macOS arm64, Rust 1.99.1: 269 full tests / 137 without GUI passed.
The seven new cases cover draft isolation, one transaction/history/save/reopen,
no-op and cancellation preserving inheritance/redo, all four handles and bounds
with first/hanging/zero indents, half-open paragraph ranges, stale-source rejection,
and actual-widget pointer press/move/release/Escape. Widget cases exercise zoom
50/100/200% × synthetic DPI 100/125/150%, focus loss, resize, PointerGone, release
outside the ruler and blocked input. A regression prevents a held button from
restarting a cancelled drag. Existing Unicode/layout/format/storage suites pass.

fmt, normal/capture all-target Clippy with warnings denied, release/capture builds,
repository/artwork/contract checks, workflow YAML and diff whitespace passed.
Isolated executable `--version` and JSON `--capabilities` report alpha.46, protocol
1 and breakfree.docs without opening GUI or relying on a neighboring repository.
Two native capture-harness images were inspected (UI_SPEC). Ordinary native
pointer/keyboard/IME input was not repeated. Windows/manual display-scale and
matched-reference acceptance remain explicitly deferred. Alpha.45 code/journal
CI succeeded, including Windows jobs; alpha.46 server CI is separate from these
local results. No merge, tag or release.

## Alpha.47 ruler keyboard validation

2026-10-18, macOS arm64, Rust 1.99.1: 275 full / 138 core-only tests passed.
Six new tests cover exact half-point values and origin semantics; keyboard entry,
Tab/reverse traversal, fine/coarse nudges, Enter/Tab transaction boundaries,
Escape cancellation, history and redo-preserving no-op at nine zoom/synthetic-DPI
combinations; immediate reversal at bounds; blocked input/modal entry; focus and
resize cancellation; hidden-ruler entry and pointer takeover; compact maximum-value
text width; high-zoom focus scrolling without losing the keyboard preview.
The existing pointer, Unicode, layout, storage and invalid-file tests remain.

fmt, normal/capture all-target Clippy with warnings denied, release/capture builds,
repository/artwork/contract checks, workflow YAML and diff whitespace passed.
Isolated CLI version/capabilities report alpha.47 and protocol 1 without a GUI or
neighboring repository. Two native macOS capture-harness images show active
keyboard preview and exact values; they do not prove physical keyboard/IME input.
The native automation app selector could not address the standalone executable by path or name; ordinary OS keyboard input was not verified. Windows/manual DPI and matched-reference
acceptance remain deferred. Alpha.46 code/journal CI succeeded; alpha.47 server
results are recorded separately in the project log. No merge, tag or release.

## Alpha.48 mixed-selection ruler validation

2026-10-18, macOS arm64, Rust 1.99.1: 280 full / 141 core-only tests passed.
Five new cases cover resolved equality across inherited/direct sources, per-property
ranges, logical marker origins versus special amounts, caret/exclusive-end/empty
paragraphs and combining text, invalid ranges, preview/cancel/commit/history/reopen,
actual-widget selection changes and keyboard preview at three synthetic scales,
and compact readout bounds with no document/history mutations. Existing pointer,
keyboard, Unicode, layout, persistence and failure suites remain in the full gate.

fmt, normal/capture all-target Clippy with warnings denied, release/capture builds,
repository/artwork/contract checks, workflow YAML and diff whitespace passed.
Isolated version/capabilities report alpha.48, protocol 1 and breakfree.docs without
GUI or sibling dependencies. Two native macOS capture fixtures were inspected;
these do not verify ordinary OS keyboard/IME input. Windows/manual DPI and matched
reference acceptance remain deferred. Alpha.47 code/journal CI succeeded; alpha.48
CI results are separate. No merge, tag or release.


## Alpha.49 compact two-axis ruler validation

2026-10-18, macOS arm64 / Rust 1.99.1. Full suite: **282 passed**; engine-only:
**141 passed**. fmt, normal/capture all-target Clippy with warnings denied, native
release/capture builds, repository/artwork/YAML and whitespace checks passed.

Two added widget cases cover both axes at nine zoom/DPI combinations (50/100/200%
zoom × 1/1.25/1.5 synthetic scale), alignment with actual canvas bounds, shared
visibility without edits, and long-document scrolling with the horizontal ruler
remaining visible. Updated contextual-readout checks retain compact maximum/mixed
value bounds. Existing pointer/keyboard preview/cancel/Undo/Redo, mixed selection,
storage, Unicode, styles and failed-save regressions pass unchanged.

Native capture fixtures at 1366 × 900 and 640 × 480 DIP are recorded in UI_SPEC.
They are actual application rendering, not ordinary OS keyboard automation.
Windows display scaling/input and matched Word 2024 acceptance remain deferred.
The vertical axis is a read-only continuous-canvas scale, not page layout. Format
v13 and protocol/settings v1 are unchanged. Alpha.48 code/journal push and PR CI
succeeded; alpha.49 server CI is separate from local results. No merge/tag/release.


## Alpha.50 ruler reset validation

2026-10-18, macOS arm64 / Rust 1.99.1: **286 full tests / 143 without GUI** passed.
fmt, normal/capture all-target Clippy with warnings denied, release/capture builds,
repository/artwork/YAML/whitespace checks and copied-binary isolated CLI identity
and capabilities passed. No Office or sibling repository is needed at runtime.

Four new tests cover distinct paragraph styles and parent inheritance, removal of
only indent overrides, preservation of spacing and half-open selection boundaries,
preview isolation, cancel/no-op redo preservation, stale selection rejection,
one-entry Undo/Redo and save/reopen. Actual-widget tests exercise context-menu
reset/cancel and keyboard reset/accept/history. Existing ruler input, zoom/DPI,
scrolling, Unicode and persistence regressions remain passing.

Two native capture fixtures were inspected at 1366 × 900 and 640 × 480 DIP.
These establish rendering only; ordinary OS input, native Windows scaling and
matched Word reference validation were not performed. Native v13 and process/settings
v1 are unchanged. Server CI is reported separately in PROJECT_LOG. No merge/tag/release.


## Alpha.51 individual ruler reset validation

2026-10-18, macOS arm64 / Rust 1.99.1: **289 full / 145 core-only tests** passed.
fmt, normal/capture all-target Clippy with warnings denied, release/capture builds,
repository/artwork/YAML/whitespace checks and isolated copied-binary version/capabilities
passed. Three added tests cover individual-property reset preservation, half-open
selection, single-entry undo/redo, save/reopen, no-op redo and focused-marker mapping.
The existing context-menu test now exercises all four choices; keyboard coverage
exercises all four markers with Shift+Delete, cancellation and history.

Two native capture fixtures at 1366 × 900 and 640 × 480 DIP were inspected. The
special-property reset preserves displayed left/right values and the scoped hint
fits. These use injected widget events; ordinary OS keyboard, Windows display
scaling and matched Word reference checks were not performed. Native v13 and
process/settings v1 are unchanged. Alpha.50 code/journal CI succeeded; alpha.51 CI
is reported separately. No merge/tag/release.


## Alpha.52 ruler provenance validation

2026-10-18, macOS arm64 / Rust 1.99.1: **293 full / 148 core-only tests** passed.
fmt, normal/capture all-target Clippy with warnings denied, release/capture builds,
repository/artwork/YAML/whitespace and isolated executable CLI identity/capabilities
passed. Four new cases cover equal values with different sources, direct/inherited/
default attribution, half-open selection and counts, custom style names, reset
preview/commit/undo/redo and real widget tooltip text without document/history edits.
The capture fixture's formatting was corrected and capture Clippy repeated.

Two native hover captures at 1366 × 900 / 640 × 480 DIP were inspected: mixed
source groups and counts wrap within the window. Ordinary OS keyboard, Windows
scaling and matched Word reference checks remain unperformed/deferred. The source
inspector is shared with the Formatting pane; native v13 and process/settings v1
are unchanged. Server CI is reported separately. No merge/tag/release.


## Alpha.53 keyboard ruler source validation

2026-10-18, macOS arm64 / Rust 1.99.1: **296 full / 148 core-only tests** passed.
fmt, normal/capture all-target Clippy with warnings denied, release/capture builds,
repository/artwork/YAML/whitespace and isolated executable CLI identity/capabilities
passed. Three new widget cases cover all four marker entries, blocked edit/navigation
keys, Enter/Escape/F1 dismissal, preserved pending preview and separate acceptance,
compact bounds at three synthetic scales and focus-loss cleanup. The first run
exposed Escape cancelling the pending draft; explicit focus restoration and consumed
dismissal events fix that regression. The final full suite includes the fix.

Two native captures at 1366 × 900 / 640 × 480 DIP were inspected. These are
widget-injected keyboard fixtures, not ordinary OS input. Windows/Word matching,
native display scaling and full accessibility acceptance remain deferred. Source
summaries retain the three-group limit. Native v13 and process/settings v1 unchanged.
Alpha.52 code/journal CI succeeded; alpha.53 CI is recorded separately. No merge/tag/release.


## Alpha.54 complete source-list validation

2026-10-20: full/core suites pass with 298/149 tests on macOS arm64, Rust 1.99.1.
The source-list regression checks every group and count beyond the hover limit,
including the independent special-indent default. The widget fixture uses 32
styles with long names at 640 × 480 DIP and synthetic scales 1/1.25/1.5.
It verifies Page Up/Down, End clamping, Home/Up clamping, viewport bounds, reopening
at zero and unchanged document/history. Existing pending-preview tests now also
scroll before each dismissal and retain the separate one-operation acceptance.

Native capture fixtures at 1366 × 900 and 640 × 480 DIP inject F1 then End.
Both show the final source and special-indent default above the fixed footer.
Initial inspection found missing arrow glyphs in the footer; use the explicit
Up/Down key names instead. Captures are widget-injected rendering evidence, not
ordinary OS keyboard, physical DPI, IME, accessibility or Windows/Word acceptance.
Windows remains deferred. No format/dependency/license change; native v13 and
protocol/settings v1 remain unchanged. Alpha.53 CI succeeded; current CI is
recorded separately in the journal.


## Alpha.55 source-wheel validation

2026-10-20, macOS arm64 / Rust 1.99.1: 300 full and 149 core tests.
The actual-widget matrix at synthetic scales 1/1.25/1.5 checks point/line/page
units, direction, upper/lower bounds, hover outside the panel, missing pointer,
horizontal and Ctrl-wheel input. The canvas rectangle stays fixed throughout
inspection and 90 frames after dismissal; a fresh document wheel event works.
A separate reset-preview case verifies scroll without mutation, pointer takeover
cancellation and keyboard dismissal followed by a single commit/Undo/Redo.

Native capture fixtures at 1366 × 900 and 640 × 480 DIP open F1 and inject a
wheel event over the panel to reach the last source. They establish rendering
only, not ordinary OS mouse/trackpad behavior or Windows/Word equivalence.
Windows remains deferred. Native v13, protocol/settings v1, dependencies and
license are unchanged. Alpha.54 CI succeeded; current CI is recorded separately.


## Alpha.56 context-source validation

2026-10-20, macOS arm64 / Rust 1.99.1: 303 full and 149 core tests.
The actual-widget matrix opens the real ruler context menu and selects each of
four source commands at synthetic scales 1/1.25/1.5 in 640 × 480 DIP. It checks
item bounds, popup closure, chosen marker, zero initial scroll, wheel access and
focus after Escape, with unchanged document, selection and undo history.
An additional regression exposed marker widgets intercepting secondary clicks;
forwarding them to the shared menu now passes on all four markers.
A separate right-click takeover regression cancels a pending ruler edit before
inspection without accepting it. Existing context reset, F1 preview-preserving
inspection, wheel and keyboard cases remain passing.

Native macOS capture fixtures show the open menu and a source panel selected
through it. Events are injected into widgets; ordinary OS input, physical DPI,
accessibility and Windows/Word acceptance are unverified. No native-format,
protocol/settings, dependency or license change. Alpha.55 CI was still running
at the audit; current CI is recorded separately in the journal.


## Alpha.57 keyboard-context validation

2026-10-20, macOS arm64 / Rust 1.99.1: 306 full and 149 core tests.
The actual-widget matrix enters the ruler, tabs to all four markers, opens with
Shift+F10 and activates the initially selected source command using Enter, at
640 × 480 DIP and synthetic scales 1/1.25/1.5. It verifies bounds, popup closure,
marker focus and unchanged document/history. Separate tests exercise Space,
Home/End, arrows and Tab/Shift+Tab wrap, blocked text, Escape preserving a draft,
source inspection and one-operation acceptance/Undo/Redo, then reset acceptance.
Focus loss and resize-cancelled previews close stale menus without editing.
Existing pointer source/reset and wheel regressions remain passing.

Two native macOS fixtures use injected keyboard events to open the menu at
1366 × 900 and 640 × 480 DIP. They verify rendering, not physical Shift+F10,
Windows DPI, IME or assistive-technology acceptance. The dedicated Menu key is
not represented in the current toolkit key enum. Native v13 and protocol/settings
v1 remain unchanged. At audit, alpha.56 journal push/PR CI succeeded and its code
runs were still running; current CI is recorded separately in the journal.

Initial compact capture exposed the value readout covering the first menu item.
Suppress it during keyboard-menu display; the widget matrix asserts its absence.

## Alpha.58 ruler accessibility validation

2026-10-20, macOS 27.0.1 arm64, Rust 1.99.1. Full suite: 309 tests; core-only:
149 tests. fmt, normal/capture Clippy with warnings denied, normal/capture release
builds, repository/artwork checks, workflow YAML and diff whitespace checks pass.
An isolated copy of the executable answers version/capabilities without Office.

Three AccessKit-output regressions exercise all four markers with Focus/Click,
full source summaries, preview naming, all eight menu entries, keyboard focus
and Escape return, native-style accessibility menu activation, and disabled
commands during a draft. These complement existing three-scale keyboard/widget
coverage; they are not Windows or speech tests. Inspection preserves the document
and history; the existing apply/cancel/undo/redo regression suite passes.

A disposable app bundle of the normal release executable opened on macOS. Native
AX inspection exposed all four marker names/help. Ctrl+Shift+F8 then Shift+F10
focused First line sources; Down/Enter opened continuation sources and restored
marker focus. Escape, Right and Escape exposed a 0.5 pt hanging preview with
“Preview; not yet applied”, then restored zero and document focus without edits.
The native screenshot retained the approved design/palette/icons.

VoiceOver acceptance is **not passed**: Command+F5 did not confirm activation.
System Settings showed VoiceOver off; clicking its switch briefly showed on, but
a later read showed off again. Native AX/keyboard checks continued, but sustained
VoiceOver execution and spoken output could not be established. Final observed
switch state was off, matching the initial state. The native tool subsequently
reported procNotFound for the temporary app and System Settings. No speech,
VoiceOver cursor navigation, braille or complete accessibility claim is made.

Remaining acceptance: a sustained VoiceOver session must read all sources, follow
menu focus, announce unavailable commands and preview changes, return safely, and
verify speech verbosity for long mixed selections. Windows/Narrator/NVDA and
100/125/150% native display checks remain deferred by owner instruction.

## Alpha.59 VoiceOver acceptance deferral

2026-10-20: a direct VoiceOver launch attempt timed out. System Settings then
briefly reported VoiceOver on and later off; VoiceOver Utility opened and showed
the caption panel already enabled. The owner clarified in chat that they were
manually switching VoiceOver off and requested postponement until the final
review. The earlier observed deactivation therefore does **not** establish a
VoiceOver crash, OS failure or editor defect. No further activation was attempted.

Speech, VoiceOver cursor navigation, menu announcements and long-source verbosity
remain **not verified**, now deferred by owner request. Alpha.58 native AX and
ordinary keyboard evidence remains valid within its documented scope. The final
accessibility review must establish sustained speech/caption output, traverse all
four markers and all eight menu commands, inspect mixed-source content, distinguish
preview/disabled states, cancel without changes and return to document focus.

This increment changes version and documentation only; runtime behavior, design,
palette/icons and native v13 are unchanged. The standard local Cargo checks and
repository validation were rerun for alpha.59; 309 full / 149 core tests pass.
Windows acceptance remains separately deferred. No native VoiceOver acceptance
claim is based on these automated checks.

## Alpha.60 indent command validation

2026-10-20, macOS arm64, Rust 1.99.1: 313 full / 151 core tests pass.
fmt, normal/capture Clippy, release/capture builds, repository/artwork/YAML and
diff checks pass. Isolated version/capabilities identify alpha.60 without Office.
Two core regressions cover inherited/mixed/clamped indents, Unicode selection,
exclusive-end exclusion, unchanged right/special indents, one Undo/Redo, native
v13 encoding/decoding, no-op inheritance/redo and invalid ranges. Two widget tests
activate the actual Home controls and collapsed-ribbon KeyTips; IME guards hold.

A native capture at 1366 × 900 DIP was inspected: the two existing indent icons
are enabled, with unchanged group geometry and palette. The known missing emoji
glyph in the Welcome fixture remains. Capture uses the development harness, not
physical OS input. Native keyboard/input was not repeated this iteration.
VoiceOver final review, Windows and matched Word comparison remain deferred.

## Alpha.61 tab storage and layout validation

2026-10-20, macOS arm64, Rust 1.99.1: 318 full / 153 core tests pass.
fmt, normal/capture all-target Clippy, normal/capture release builds,
repository/artwork/YAML/whitespace and isolated CLI checks pass.

Two core additions cover literal tabs with English/Ukrainian/Russian/combining
text, native v14 save/reopen, interval bounds, strict v13 field rejection,
selection boundaries, Enter propagation and Undo/Redo. Three widget/layout
additions cover native-widget Tab insertion/history, modified-Tab selection
safety, cursor hit testing, paragraph separators, default/custom grid positions,
indents, repeated/trailing tabs, narrow wrapping and combining-cluster boundaries
at synthetic 1/1.25/1.5 scale and 50/100/200% zoom. Existing persistence fault,
formatting and undo tests remain passing. Synthetic scales are not Windows tests.

Native render captures at 1366 × 900 and 640 × 480 DIP were inspected with a
five-paragraph tab fixture. Names of different widths reach different grid stops;
this is a repeating grid, not a table. Chrome/palette/icons retain their design.
Capture uses a development fixture; physical Tab/IME was not retested.

Limits: no GUI interval editor, custom ruler stops, style inheritance, leaders or
right/center/decimal tab types. Justify uses left placement for tab paragraphs;
Center/Right align the fixed-spacing row as a block. No Word fidelity claim.
VoiceOver final review and Windows remain deferred. Content v14 is rejected by
older readers; retain older files separately when using alpha.60 or earlier.

## Alpha.62 tab interval dialog validation

Validated on macOS arm64 with Rust 1.99.1. The full suite passes **322 tests**;
`--no-default-features` passes **153 tests**. New real-widget cases cover mixed
paragraph selection and exclusive end, interval apply/reset in one Undo/Redo,
invalid input and Escape without mutation, nested cancellation and retained
paragraph dialog, no-op acceptance preserving Redo, keyboard activation and modal
bounds at synthetic 100/125/150% scales. Exact half-point parsing rejects fractional
rounding, overflow and non-finite/exponential inputs. Existing v14 save/reopen,
legacy rejection and failed-write preservation tests still pass.

Formatting, both normal/capture all-target Clippy, release/capture builds,
repository references/contract/artwork, workflow YAML and whitespace checks pass.
Two native captures were inspected (see UI_SPEC); physical keyboard/IME were not
repeated. Isolated executable version/capabilities checks pass without Office.
VoiceOver final acceptance, Windows scaling/input and matched reference comparison
remain deferred. Local results do not establish server CI; see PROJECT_LOG.

## Alpha.63 explicit left tab validation

macOS arm64, Rust 1.99.1: **327 full tests / 156 without GUI** pass. New cases
cover v15 save/reopen, mixed selection/exclusive paragraph end, set/clear/no-op
history, invalid/duplicate/descending/oversized/out-of-range lists, malformed JSON
types and pre-v15 rejection of the key even when empty/null. Enter, join and
Replace All preserve the relevant positions; changing the interval leaves them
intact. Existing file failure-preservation and formatting suites still pass.

Layout tests exercise next explicit stops, exact-stop progression, fallback to the
original grid origin, zoom and synthetic DPI, cursor roundtrip, first/hanging
indents, narrow wrapping, Unicode combining boundaries and mixed formatting.
Text glyph coordinates use the renderer's one-pixel rounding tolerance; tab
positions retain fractional geometry. This is not full writing-system acceptance.

fmt, normal/capture all-target Clippy, release/capture builds, repository/artwork,
workflow YAML, whitespace and isolated executable version/capabilities checks pass.
After the final precedence hint and interval-preservation assertion, focused dialog
and storage/history tests and capture Clippy were repeated successfully. Two native
screenshots inspected (UI_SPEC). Physical keyboard/IME, matched reference, Windows
and VoiceOver acceptance were not performed. Position editing UI, other stop types,
leaders and named-style inheritance remain later work; CI is reported separately.

## Alpha.64 tab positions dialog validation

macOS arm64 / Rust 1.99.1: **331 full tests / 156 without GUI** pass. Real-widget
cases cover mixed replacement, exclusive paragraph end, Add/Update/Remove/Clear,
uncommitted input and validation, Cancel/Escape, nested paragraph cancellation,
no-op Redo preservation, one-operation undo/redo and disk save/reopen. Draft tests
check exact half-point parsing, duplicate/limit rejection and updating a full list.
Keyboard traversal/activation and modal bounds are tested at synthetic 100/125/150%
with 32 positions. These tests do not establish physical Windows input behavior.

fmt, both normal/capture all-target Clippy, release/capture builds, repository
references/contract/artwork, workflow YAML, whitespace and isolated executable
version/capabilities checks pass. Two native macOS captures inspected at 1366 × 900
and 640 × 480 DIP (UI_SPEC). Native v15 and existing failure-preservation tests
remain unchanged. Physical keyboard/IME, matched Word comparison, Windows and
VoiceOver were not tested this iteration. Interactive ruler tabs, other tab types,
leaders and named-style inheritance remain planned. CI is reported separately.

## Alpha.65 tab ruler validation

macOS arm64 / Rust 1.99.1: **337 full tests / 159 without GUI** pass. Core draft
tests cover isolated preview, crossing/sorted positions, half-open selection,
clamping/rounding, duplicate/non-finite rejection, mixed lists, stale selection and
document protection, no-op Redo and one-step history. The moved snapshot roundtrips
through native v15. Existing save-failure and tab-dialog suites still pass.

Real-widget pointer tests exercise 50/100/200% zoom and synthetic 100/125/150% DPI,
preview without model/history mutation, release as one operation, Undo/Redo,
Escape, pointer/focus loss, resize/zoom, outside release, duplicate release and
suppression of text input during a drag. Mixed lists cannot start a tab drag;
overlapping indent markers keep their existing interaction. No physical input or
screen-reader acceptance is inferred from these tests.

fmt, normal/capture all-target Clippy, release/capture builds, repository/artwork,
workflow YAML, whitespace and isolated version/capabilities pass. After lowering
markers below scale numerals and adding hover feedback, focused widget/core tests,
capture Clippy and release builds were repeated. Two native screenshots inspected
(UI_SPEC). Native v15, palette and icons unchanged. Physical keyboard/IME, matched
Word comparison, Windows and VoiceOver remain untested/deferred. Ruler tab keyboard
control, add/delete and mixed-list dragging are not implemented. CI is separate.

## Alpha.66 keyboard ruler validation

2026-10-20, macOS arm64, Rust 1.99.1. Alpha.65 approved; alpha.66 awaits approval.

- Full suite: 340 tests; core-only: 159 tests. Fine/coarse movement, one Undo/Redo,
  Tab/reverse traversal, duplicate rejection, cancellation, empty/mixed lists,
  overlapping/off-column access and no-op Redo preservation are covered.
- Widget input tests run at synthetic 100/125/150% scale. They exposed a missing
  explicit window-focus-loss cancellation, now covered and fixed.
- Native capture exposed overbroad event filtering; the controller now preserves
  screenshot/service events while suppressing editing input. Two captures at
  1366 × 900 and 640 × 480 DIP were inspected: existing palette/icons and compact
  ribbon remain; active L marker and exact 77.0 pt preview are visible and fit.
- fmt, normal/capture all-target Clippy, full/core tests, release/capture builds,
  repository/artwork/YAML/whitespace and isolated CLI checks pass.
- Native v15 and pointer-drag/storage/layout regression tests remain unchanged.
  No physical OS keyboard/IME or matched Word comparison was performed in this
  increment. VoiceOver remains deferred to final review; Windows is deferred.

Reproduce native captures with `cargo build --release --features capture --example
capture_ui`, then `BREAKFREE_CAPTURE_TO=/tmp/tab-keyboard.png
./target/release/examples/capture_ui 1366 900 tab-keyboard` (environment assignment
and command on one shell line). Injected harness keys are not physical input evidence.

## Alpha.67 ruler tab creation and deletion validation

2026-10-20, macOS arm64, Rust 1.99.1. Alpha.66 approved; alpha.67 awaits approval.

- 346 full tests / 162 core-only tests pass. New engine coverage checks creation,
  removal after movement, one-step history, v15 roundtrip, half-open selection,
  duplicate/full/mixed/stale rejection, empty lists and last-stop deletion.
- Widget tests exercise empty-ruler click, context deletion/menu dismissal,
  keyboard removal preview/cancel/apply, exact Undo/Redo at synthetic 100/125/150%,
  duplicate/mixed-list rejection and outside-release cancellation. Existing ruler,
  keyboard, layout, Unicode, persistence and failed-write tests remain passing.
- fmt, normal/capture all-target Clippy, release/capture builds, documentation,
  artwork, YAML and whitespace checks pass. Isolated executable --version and
  --capabilities report alpha.67 without Office or neighboring repositories.
- Native macOS captures at 1366 × 900 and 640 × 480 DIP were inspected: the exact
  deletion notice fits below the ruler, the preview removes the chosen marker and
  relayouts only the selected paragraph; palette/icons and compact Home remain.
- No physical OS keyboard/IME, matched Word comparison or screen-reader speech
  acceptance this increment. VoiceOver remains final-stage work; Windows deferred.

Reproduce captures with the existing capture example and mode `tab-delete`.
Injected events verify rendered state, not physical input or Windows DPI behavior.

## Alpha.68 mixed-list ruler editing validation

2026-10-20, macOS arm64, Rust 1.99.1. Alpha.67 approved; alpha.68 awaits approval.

- 350 full / 165 core-only tests pass. Mixed-list union, first-paragraph-empty
  selection, pointer entry and later-paragraph keyboard traversal/editing are covered.
- Create adds missing occurrences; move/delete preserve nonowners and unrelated
  positions. Tests verify one history operation, exact Undo/Redo, half-open range,
  v15 roundtrip, stale selection, capacity failure and destination conflict without
  partial preview or commit. A destination only in a nonowner is allowed.
- Updated earlier mixed-list rejection tests to the new explicit policy; homogeneous
  duplicate/invalid/no-op, cancellation, pointer priority and dialog tests remain.
  Synthetic widget scales 100/125/150% do not establish Windows acceptance.
- fmt, normal/capture all-target Clippy, release/capture builds, documentation,
  artwork, YAML and whitespace pass. Isolated executable CLI reports alpha.68.
- Two native captures (1366 × 900, 640 × 480 DIP) inspected: four union positions
  visible, partial markers gray, active marker accented; exact preview fits and the
  first paragraph keeps its separate grid. Palette/icons and ribbon unchanged.
- Capture mode `tab-mixed` explicitly selects all fixture paragraphs before entering
  keyboard preview. No physical OS keyboard/IME, Word comparison or VoiceOver speech
  acceptance this iteration. VoiceOver final review and Windows remain deferred.

## Alpha.69 keyboard ruler tab creation validation

2026-10-20, macOS arm64, Rust 1.99.1. Alpha.68 approved; alpha.69 awaits approval.

- 354 full / 166 core-only tests pass. New creation draft cannot commit before
  validation; duplicate recovery and stale-source rejection are covered.
- Widget checks cover direct creation shortcut, empty-list entry, fine/coarse steps,
  isolated text input, one-step Undo/Redo, invalid acceptance blocking, mixed-list
  missing-only insertion, Enter/Tab exit, Escape/Delete/focus/resize cancellation
  and retained Redo. Synthetic 100/125/150% scales are not Windows acceptance.
- Existing mixed-list conflict/capacity, range, persistence and pointer checks pass.
  fmt, normal/capture all-target Clippy, release/capture builds, repository/artwork,
  YAML and whitespace checks pass. Isolated CLI reports alpha.69 without Office.
- Two native captures at 1366 × 900 and 640 × 480 DIP inspected: new 41.0 pt marker
  and exact new-tab readout fit; prior markers, palette/icons and ribbon preserved.
  Mode `tab-create` injects the creation shortcut and one coarse arrow step.
- No physical OS keyboard/IME, matched Word reference or spoken screen-reader test
  this iteration. VoiceOver remains final review; Windows remains deferred.

## Alpha.70 exact ruler tab input validation

2026-10-21, macOS arm64, Rust 1.99.1. Alpha.69 approved; alpha.70 awaits approval.

- 357 full / 167 core-only tests pass. Exact numeric parser tests cover boundaries,
  unsupported fractional steps, overflow, nonnumeric input and invalidation after
  a valid preview; duplicate recovery and one-operation history are verified.
- Widget tests cover F2, selected initial text, typing/paste, same-frame text and
  Enter/Tab, creation/movement, conflict blocking, Escape and retained Redo.
  Synthetic 100/125/150% scales include a compact window; these are not Windows tests.
- fmt, normal/capture all-target Clippy, release/capture builds, repository/artwork,
  YAML and whitespace checks pass. Copied isolated binary reports alpha.70 and
  correct capabilities without Office or adjacent repositories.
- Two native captures at 1366 × 900 and 640 × 480 DIP inspected: the focused
  48.5 pt field, New tab label and help fit; ruler, ribbon, palette/icons retained.
  Capture mode `tab-numeric` injects keyboard entry/F2/text. Physical OS keyboard,
  IME and matched Word comparison were not checked. VoiceOver final review and
  Windows acceptance remain deferred.

## Alpha.71 center/right tab storage and layout validation

2026-10-21, macOS arm64, Rust 1.99.1. Alpha.70 approved; alpha.71 awaits approval.

- 363 full / 170 core-only tests pass. New coverage checks one-operation history,
  half-open paragraph ranges, save/reopen, Enter geometry copying, left-control
  preservation/conflicts, combined capacity, invalid types and strict v15 migration.
- Native renderer tests verify center/right anchors with mixed formatting,
  English/Ukrainian/Russian/combining text, 50/100/200% zoom, empty fields, trailing
  spaces, left/hanging/first-line indents, narrow wraps, forward cursor geometry,
  ASCII cursor hit testing and subsequent grid fallback. Fixed repeated selection
  of a consumed right stop caused by font measurement rounding.
- fmt, normal/capture all-target Clippy, full/core tests, release/capture builds,
  repository/artwork/YAML and whitespace checks pass. Isolated executable version
  and capabilities identify alpha.71 without Office or adjacent repositories.
- AlignedTabs.bfdoc v16 is tested and rendered in native captures at 1366 × 900 and
  640 × 480 DIP. Centered fields share a center and right fields share a right edge;
  palette/icons/ribbon retained. This does not establish physical keyboard/IME,
  matched Word or Windows DPI acceptance. VoiceOver final review is deferred.
- Type-selection controls and typed ruler markers are not implemented this increment.

## Alpha.72 typed tab dialog and ruler validation

2026-10-21, macOS arm64, Rust 1.99.1. Alpha.70 explicitly approved; alpha.71 has
no explicit approval recorded. Alpha.72 is one three-step candidate for review.

- 370 full / 173 core-only tests pass. New coverage checks unified typed-list
  history/roundtrip, half-open ranges, duplicate/order/capacity rejection, mixed
  types at one position, owner-only movement/deletion, conflicts, stale drafts and
  no-op Redo preservation. Existing strict v16 and layout tests pass.
- Widget tests cover staged kind changes requiring Update, one final Undo/Redo,
  compact controls at synthetic 100/125/150% scale, keyboard traversal of typed
  stops, F2 movement, deletion preview/cancel and pointer movement preserving type.
  Updated the explicit Tab path to include Left/Center/Right controls. Native OS
  keyboard/IME and Windows DPI acceptance are separate and were not performed.
- fmt, normal/capture all-target Clippy, full/core tests, release/capture builds,
  repository/artwork/YAML/whitespace checks pass. The isolated copied executable
  reports alpha.72 and correct capabilities without Office or sibling repositories.
- Three native captures inspected: marker view at 1366 × 900, dialog at 1366 × 900
  and 640 × 480 DIP. Type captions/controls fit, center/right shapes are visible,
  and palette/icons remain unchanged. No matched Word comparison or spoken reader
  acceptance is claimed. Windows deferred; VoiceOver remains final review.

## Alpha.73 typed tab creation validation

2026-10-21, macOS arm64, Rust 1.99.1. Alpha.70 explicitly approved; alpha.71–72
have no explicit approval recorded. Alpha.73 combines three related creation steps.

- 377 full / 175 core-only tests pass. Typed creation checks missing-only updates,
  same/different-kind destinations, capacity, invalid positions, cancellation,
  Redo preservation, save/reopen and one-operation Undo/Redo.
- Widget tests cover selector click/context menu/Ctrl+Alt+F7, unchanged selection,
  frozen draft type, keyboard F2 exact positions, pointer preview/apply/Escape/
  focus-loss/outside cancellation and a complete click within one event frame.
  Compact selector geometry is checked at synthetic 100/125/150% scale.
- The full suite caught selector allocation changing the canvas bounds and scroll
  layout. The selector now uses a non-allocating interaction region; existing
  horizontal/vertical ruler alignment, zoom and long-document scroll tests pass.
- Windows and physical OS keyboard/IME acceptance remain deferred; synthetic
  inputs do not replace them. VoiceOver remains final review. No matched Word
  comparison or visual equivalence is claimed. Native format v16 is unchanged.
- fmt, normal/capture all-target Clippy, release/capture builds, repository/artwork,
  YAML and whitespace checks pass. The copied isolated executable reports alpha.73
  and correct v1 capabilities without Office or sibling repositories.
- Three native captures inspected: Center/Right creation at 1366 × 900 and Right
  creation at 640 × 480 DIP. Selector, type-specific preview and markers fit;
  palette/icons are retained. Screenshot hashes are recorded in UI_SPEC.md.

## Alpha.74 tab interaction recovery validation

2026-10-21, macOS arm64, Rust 1.99.1. Owner approved alpha.73; alpha.74 awaits review.

- Reproduced two failing regressions before fixes: repeated arrows could not pass
  a conflicting tab position; hiding rulers left a pointer preview active.
- 382 full / 177 core-only tests cover adjacent conflict recovery in creation and
  movement, mixed-owner atomicity, numeric-error/deletion recovery, stale selection,
  no-op Redo, typed save/reopen and one-operation Undo/Redo. Unicode paragraph
  fixtures include English, Ukrainian, Russian and a combining character.
- Widget scenarios cover zoom/compact resize/hidden-ruler/pointer cancellation,
  subsequent keyboard re-entry and preserved Redo for all three creation types.
  These are synthetic event checks, not physical OS keyboard or Windows acceptance.
- Native v16, layout policy, palette/icons and geometry remain unchanged. Leaders
  are not implemented. Windows and matched reference review remain deferred;
  VoiceOver is reserved for final review.
- fmt, normal/capture all-target Clippy, release/capture builds, repository/artwork,
  YAML, whitespace and isolated CLI version/capabilities checks pass.
- Fresh native 1366 × 900 Center and 640 × 480 Right creation PNGs are byte-identical
  to the corresponding alpha.73 captures linked in UI_SPEC.md. This establishes
  unchanged rendering for those fixtures, not all interaction states.

## Alpha.75 tab leader validation

2026-10-21, macOS arm64, Rust 1.99.1. Owner approved alpha.74; alpha.75 awaits review.

- 389 full / 182 core-only tests: per-type leader set/clear with one Undo/Redo,
  exclusive paragraph boundaries, save/reopen, Enter, mixed-owner ruler movement,
  collision/deletion, whole-list replacement and no-op Redo preservation.
- Native v17 tests reject null/unknown/duplicate/unsorted/orphan leaders, leader
  fields in v1–v16 (including empty/null), and unknown versions. Plain v16 opens.
  An invalid leader save preserves the previous file bytes; the complete existing
  write-failure/Unicode/storage suite also runs.
- Renderer matrix compares leader/no-leader text, row count, every glyph position
  and advance, and grapheme-boundary cursor rectangles across Left/Center/Right,
  dots/dashes/line, English/Ukrainian/Russian/combining text, 50/100/200% zoom and
  narrow/wide columns. Mesh tests verify bounded work and containment within the
  tab advance. These examples do not establish all-script shaping support.
- Leader controls are not exposed yet. Windows, physical OS keyboard/IME, matched
  reference comparison and VoiceOver acceptance remain deferred.
- fmt, normal/capture all-target Clippy, release/capture builds, repository/artwork,
  YAML, whitespace and isolated CLI checks pass. Version/capabilities report alpha.75
  independently of Office. Protocol/settings remain v1; document writer is v17.
- Two native captures inspected at 1366 × 900 and 640 × 480 DIP using the openable
  TabLeaders fixture. All leader kinds are visible with intact field alignment;
  the palette/icons/ribbon remain unchanged. Hashes are recorded in UI_SPEC.md.

## Alpha.76 leader editing validation

2026-10-21, macOS arm64, Rust 1.99.1. Alpha.74 approved; alpha.75 has no explicit
approval recorded. Alpha.76 is one five-step review candidate.

- 396 full / 184 core-only tests cover shared whole-list positions/types/leaders
  replacement, invalid-leader atomic rejection, half-open ranges, save/reopen,
  owner-only changes/reset, one history entry and no-op Redo.
- Dialog tests cover mixed leaders with identical positions/types, move/conflict
  retention, pending changes requiring Update, clear/cancel, and actual keyboard-
  only input → Dots → Add → OK. Mixed replacement and dialog bounds pass at
  synthetic 100/125/150% scale in 640 × 480 DIP; the existing keyboard traversal
  check now includes the four leader controls. Context-menu edits preserve nonowners.
- The live sample uses the existing tested renderer. Physical OS keyboard/IME,
  matched reference comparison, Windows and VoiceOver acceptance remain deferred.
- fmt, normal/capture all-target Clippy, release/capture builds, repository/artwork,
  YAML, whitespace and isolated executable version/capabilities checks pass.
- Two native selected-leader dialog captures inspected at 1366 × 900 and 640 × 480
  DIP: sample, selected controls and action buttons fit. Hashes are in UI_SPEC.md.
  Capture interaction is synthetic and does not establish physical OS acceptance.

## Alpha.77 leader coverage validation

2026-10-23, macOS arm64, Rust 1.99.1. Alpha.76 approved; alpha.77 awaits review.

- 401 full / 186 core-only tests pass. Distribution checks distinguish missing
  stops from no fill, honor half-open selection and preserve the document.
- Preview checks cover mixed owners, conflict recovery, numeric position, retained
  leader choice, cancellation, save/reopen and one-operation Undo/Redo. Creation
  preserves preexisting owners and continues cycling when their leaders differ.
- Synthetic widget checks cover F1 read-only toggling, F3 cycling, arrows/F2,
  apply/cancel and detail bounds at 100/125/150% in a 640 × 480 DIP window.
- fmt, normal/capture all-target Clippy, release/capture builds, repository/artwork,
  YAML, whitespace and isolated CLI checks pass. Native v17 remains unchanged.
- Three native captures inspected: wide/compact details and wide mixed context
  menu. Physical OS keyboard/IME, matched reference comparison, Windows and
  VoiceOver acceptance remain deferred. CI is recorded separately in the journal.

## Alpha.78 direct leader validation

2026-10-23, macOS arm64, Rust 1.99.1. Alpha.77 approved; alpha.78 awaits review.

- 404 full / 187 core-only tests pass. New scenarios cover all four direct keys,
  reverse cycling, mixed-original restoration at a changed position, numeric-mode
  return, invalid numeric input, cancel and one-operation Undo/Redo.
- Existing synthetic compact 100/125/150% checks and all storage/layout checks run.
  Numeric digits remain text input; shortcut selection is limited to marker mode.
- fmt, normal/capture Clippy, release/capture builds, repository/artwork/YAML,
  whitespace and isolated CLI checks pass. An initial build hit disk exhaustion;
  only generated incremental cache was removed before repeating validation.
- Two native wide/compact direct-choice captures inspected. Physical OS keyboard,
  Windows, matched reference comparison and VoiceOver remain unverified/deferred.
  Native v17 and the palette/icons are retained; CI is recorded separately.
