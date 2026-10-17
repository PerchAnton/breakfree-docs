# Changelog

Only published entries are releases. This file records candidate scope; version
and publication approval follow [docs/VERSIONING.md](docs/VERSIONING.md).

## 0.1.0-alpha.18 — Unreleased; submitted for owner approval

Prepared 2026-10-17. The owner approved alpha.17 in the project chat.

- Activate Home's Normal, No Spacing and Heading 1 gallery, with responsive menus,
  selected/mixed state, keyboard traversal, KeyTips and command search. Preserve
  the palette, original icons and group gutters.
- Separate inherited character values from explicit overrides in the Rust model.
  Apply a paragraph style in one undo; keep direct formatting and alignment.
  Clear character formatting now restores each paragraph's inherited values.
  No Spacing changes actual line/paragraph spacing; Heading 1 supplies bold 18 pt.
- Preserve style identity through text editing, previewed batch replacement,
  undo/redo and native content v3. Read v1/v2 unchanged and preserve their appearance
  as explicit overrides; opening does not rewrite the source. Older builds reject
  v3 saves. Use Save As to retain an older file.
- Correct previously ignored paragraph gaps in the layout adapter and retain
  keyboard focus in overflow menus after resizing a temporary ribbon.
- Add inheritance, migration, save-failure and actual-widget/layout regressions.
  Document fixed catalog rules and the deferred style-definition editor.

No new dependency, process/settings schema or license change. Custom/character
styles, pagination and native Windows/Word comparison remain separate work.
Version approval does not authorize merge, tag or release.

## 0.1.0-alpha.17 — Unreleased; version approved 2026-10-17

Prepared 2026-10-17. The owner approved alpha.16 in the project chat.

- Activate the original Home eraser icon as Clear character formatting in full,
  compact and overflow ribbon densities. Preserve the palette, icon artwork and
  group spacing; compact layouts prioritize this working control over disabled
  Change case.
- Reset selected bold, italic, underline and size to regular 11 pt through one
  shared-engine transaction. Preserve text and paragraph alignment. At a caret,
  reset pending input only; unchanged content adds no undo and retains redo.
- Add document-only Ctrl+Space, Home KeyTips H → FC and command search. Keep
  Control distinct from Command on macOS; ignore modified chords, repeat and
  IME confirmation, and block modal background edits.
- Add range/history/Unicode/native-roundtrip and widget regression coverage;
  retain the existing keyboard and Home containment checks.

No dependency, document/protocol/settings schema or license change. Named styles,
paragraph-format reset, native Windows/IME and matched Word comparison remain
separate gates. Version approval does not authorize merge, tag or release.

## 0.1.0-alpha.16 — Unreleased; version approved 2026-10-17

Prepared 2026-10-17. The owner approved alpha.15 in the project chat. Preserve the
approved palette, original vector icons and Home spacing.

- Add Replace All from Home Replace with a modal preview of real match/change
  counts, paragraph context and replacement text. Cancel/Escape leaves content
  untouched; initial focus is Cancel. Apply commits exactly the prepared result.
- Build the validated result in the GUI-independent command engine, using original
  match ranges and each match's first-character style. Keep unaffected formatting;
  equal-text matches preserve mixed formatting and consume no history.
- Record all changes as one undo/redo operation. Reject stale previews, invalid
  ranges, unsupported control characters and oversized results before editing.
  Do not rescan inserted text or create per-match history entries.
- Block background commands/close while preview is open and guard replacement IME
  confirmation. Virtual preview rows and bounded controls adapt to window size.
- Add core and widget regressions for Unicode, mixed styles, paragraph operations,
  cancellation, keyboard Apply, stale/failed/no-op transactions and dense matches.
- Clear the completion notice after later editor actions. Validate 121 full / 51
  core-only tests, two native preview captures and macOS keyboard/Save/Open.

No dependency, document/protocol/settings schema or project-license change.
Native Windows/IME and matched Word comparison remain separate acceptance gates.
Version approval does not authorize a merge, tag or release.

## 0.1.0-alpha.15 — Unreleased; version approved 2026-10-17

Prepared 2026-10-17. The owner approved alpha.14 in the project chat. Preserve
alpha.13 colors, original vector icons and the Home group spacing.

- Add Match case and Whole words controls to Navigation, with keyboard Tab/Space
  and Escape back to the document. Options persist only in the current window.
- Share a cached literal query between counts, results, highlighting and replacement.
  Ignoring case uses Unicode simple folding; whole words use Unicode word boundaries
  including internal apostrophes. Preserve original ranges and complete graphemes.
- Keep replacement on the shared command/history path. Query/options changes are
  read-only; overlong queries produce an explicit error, without truncation.
- Add the small Rust regex dependency for Unicode case tables and bounded literal
  matching, with locked versions and updated dependency notices/inventory. User
  patterns are escaped; no wildcard/regular-expression search is exposed.
- Extend core and actual-widget regressions for multilingual case/word boundaries,
  literal punctuation, keyboard focus, formatting, undo/redo, persistence and bounds.
  All 110 full / 44 core-only tests pass; two native captures and macOS keyboard,
  replacement/history and system Save/Open checks passed.

No document/protocol/settings schema or project-license change. Native Windows,
IME, matched Word comparison and full release acceptance remain open. No merge,
tag or release is authorized by version approval.

## 0.1.0-alpha.14 — Unreleased; version approved 2026-10-17

Prepared 2026-10-17. The owner approved alpha.13 in the project chat. Its palette
and original vector icons are retained; approval does not authorize publication.

- Refine the existing Navigation pane with previous/next controls, an active
  result counter, highlighted excerpts and a virtualized list of all matches.
  Remove the silent 40-row limit and scroll the document to the selected result.
- Keep search focus on Enter/Down and Shift+Enter/Up; wrap at either end. Clear
  retains search focus; Escape closes the pane and returns to the document.
- Share one GUI-independent literal matcher between the count, selection and
  replacement. Reject partial combining/emoji graphemes instead of expanding a
  match into different text. Search is explicitly case sensitive, without
  normalization, wildcards or whole-word options.
- Verify two native captures and macOS keyboard search/replace, Undo/Redo and
  system Save/Open; document the OS-reserved Command+H limitation.
- Add Unicode, dense-result, excerpt, replacement/history/native-roundtrip and
  actual-widget regressions, including nine size/scale combinations and an IME
  confirmation guard. Preserve the shared editing and persistence commands.

No dependency, schema, license, merge, tag or release changes. Native Windows
input/scaling and matched Word 2024 comparison remain acceptance gates.

## 0.1.0-alpha.13 — Unreleased; version approved 2026-10-17

Prepared 2026-10-17. This focused visual update follows the owner's supplied
macOS reference captures. Alpha.12 version approval has not been received.

- Give original vector icons small semantic accents: gold Clipboard/Open,
  blue file/copy/scissors details and arrows, yellow highlight, red font-color
  indicator and a muted pink eraser. Keep text/alignment/history outlines dark.
- Add explicit enabled/disabled tint handling; unavailable commands remain muted
  and disabled with their existing explanations. Color indicators do not add
  text-color, highlighting or Format Painter functionality.
- Unify light neutral shell/ribbon/canvas/File colors, soften violet selection,
  and add original Home/Options/Print icons to the existing File actions.
- Inspect three native captures and normal macOS input for selection, Bold,
  undo/redo, File Home/Options and KeyTips. All 95 existing tests and 36 core-only
  tests pass; no test assertions or layout dimensions were relaxed.

The supplied images inform color roles only: they do not identify a Word build
or establish Windows/DPI equivalence. No source images, external icon assets,
new dependencies, schema changes, merge, tag or release.

## 0.1.0-alpha.12 — Unreleased; submitted for owner approval

Prepared 2026-10-17. The owner approved alpha.11 in the project chat; that approval
covers its version number, not a merge, tag or release.

- Add More Commands to Quick Access Toolbar customization, with an ordered list
  and Move Up / Move Down controls. OK applies and saves the draft; Cancel/Escape
  retains the previous toolbar. Unchanged OK does not write settings.
- Keep command selection and keyboard focus after moves. Support Up/Down/Home/End,
  Alt+Up/Down, Tab/Shift+Tab and Enter/Space, with disabled boundary controls.
  Numeric KeyTips follow the applied order, including unavailable history actions.
- Reuse settings conflict/error/retry handling; preserve previous bytes on failed
  writes and keep document content, selection and undo history independent.
- Add five actual-widget regressions, including 18 size/scale/list combinations,
  and two native captures. Verify native macOS cancellation, apply, renumbered
  Undo/Redo and order restoration after a normal application restart.

No dependency, license, settings-schema, document-content or process-protocol
change. Native Windows DPI/input and matched Word 2024 acceptance remain open.
No merge, tag or release.

## 0.1.0-alpha.11 — Unreleased; version approved by owner on 2026-10-17

Prepared 2026-10-17. Alpha.10 approval has not been received. This focused design
fix follows the owner's report of a Home separator overlapping Font controls.

- Reserve each Home group's full width exactly once, keeping the 8-DIP gap even
  when its content is narrower. Center separators in the gap so they cannot cross
  Bold, the font-family outline or adjacent groups.
- Add a regression for group spacing and separator/control separation across nine
  window widths, three synthetic scales and pinned/temporary ribbon states.
- Inspect four native Home captures and verify pointer Bold, font-size entry,
  undo/redo and temporary Home through native macOS input. Stabilize focus in
  the temporary-ribbon capture fixture; production focus handling is unchanged.

No dependency, license, document-content or process-protocol change. Matched
Word 2024 Windows/native DPI acceptance remains open. No merge, tag or release.

## 0.1.0-alpha.10 — Unreleased; submitted for owner approval

Prepared 2026-10-17. The owner approved alpha.9 in the project chat; that approval
covers its version number, not a merge, tag or release.

- Persist Quick Access Toolbar command visibility, order and above/below placement
  in a versioned local user profile, restoring them at desktop startup. Keep
  settings independent of documents, history and headless information commands.
- Add confirmed Reset with a retained original-file backup, visible load/save
  errors, a menu warning and Retry Saving. Preserve corrupt/future files and
  tested failed writes; reject stale edits from another window under a shared lock.
  A retry after failed reset retains the backup requirement, including after further
  current-window customization.
- Retain the compact native toolbar geometry, ordered numeric hints and keyboard
  menu focus. Add a keyboard-accessible reset confirmation with Escape cancellation.
- Add nine settings tests, five actual-widget regressions and a CLI no-side-effects
  test. Verify native macOS restart, failure/retry/reset/backup and Unicode editing,
  undo, Save/Open; inspect three native captures. Document schema and Windows paths.

No new dependency, license, document-content or process-protocol change. Explicit
reordering controls, live cross-window refresh, crash-lock recovery and matched
Word 2024 Windows/native DPI acceptance remain open. No merge, tag or release.

## 0.1.0-alpha.9 — Unreleased; version approved by owner on 2026-10-17

Prepared 2026-10-17. The owner approved alpha.8 in the project chat; that approval
covers its version number, not a merge, tag or release.

- Add a Quick Access Toolbar menu for New/Open/Save/Undo/Redo visibility, placement
  above/below the ribbon and reset. Preferences apply to the current window;
  adding a command appends it. KeyTips follow visible order, including disabled
  history commands; Alt+0 opens customization.
- Add keyboard toolbar entry from the tab row, Left/Right navigation that skips
  unavailable actions, menu navigation and layered Escape. Keep the toolbar
  accessible when the collapsed ribbon opens temporarily.
- Rebuild View with compact Views/Show/Zoom groups, aligned checkboxes and original
  vector controls. Add 100% and a real Canvas Width fit that follows window/pane
  changes within 50–200%. Manual zoom leaves fit mode; status percentage opens Zoom.
- Preserve document selection, shared undo/save, native content v2 and protocol v1.
  Unsupported pagination/view modes remain disabled with explanations.
- Add six actual-widget regressions, including 18 toolbar/View width/scale cases,
  and four native captures. Validate the normal macOS binary through keyboard,
  system Save/Open and independently decoded Unicode/style results.

No toolbar persistence/reordering editor, pagination, Windows visual equivalence,
new dependency or license change is claimed. Alt+0 and View C are Docs-specific;
View J and exact reference mappings require Word 2024 Windows acceptance.

## 0.1.0-alpha.8 — Unreleased; version approved by owner on 2026-10-17

Prepared 2026-10-17. The owner approved alpha.7 in the project chat; that approval
covers its version number, not a merge, tag or release.

- Add Quick Access Toolbar KeyTips: fixed 1 Save, 2 Undo, 3 Redo, including direct
  Alt+number entry. Disabled history actions keep dimmed hints without renumbering.
- Give every tab an entry and retained KeyTip level, including Draw JI and Help
  Y1. Add a More tabs hint/menu at narrow widths; Escape and resizing restore the
  root level. Unimplemented commands remain disabled and Help is informational.
- Add View K Navigation Pane, R Canvas Rulers and Q Zoom routes. Group the real
  View controls, expose selected toggle states, and retain arrow/group navigation
  when leaving hints, including the temporary collapsed ribbon.
- Add a native Zoom draft dialog with percent entry, presets, validation and
  OK/Cancel. Zoom changes only the view; it cannot change document history or
  dispatch background commands while the dialog is open.
- Add regression coverage for quick-access history/save, all tabs, overflow,
  View state, modal focus and control bounds at synthetic scales. Extend the
  capture fixture with tab-overflow, View and Zoom states.

Native content v2 and process protocol v1 stay unchanged. Draw/Help/ruler mappings
are provisional pending a named Word 2024 Windows comparison. Quick-access order
is fixed in this build; toolbar customization is not implemented. Native Windows
Alt/AltGr/IME/DPI and exact reference matching remain open. No merge, release,
dependency or license change is included.

## 0.1.0-alpha.7 — Unreleased; version approved by owner on 2026-10-17

Prepared 2026-10-17. The owner approved alpha.6 in the project chat; that approval
covers its version number, not a merge, tag or release.

- Add bare-Alt release and F10 entry into sequential KeyTips. Home/File/search
  access keys dispatch existing commands; hints follow real control positions.
- Add Home emphasis, size/Font dialog, alignment, clipboard and Editing routes,
  including a narrow-window More level. Preserve collapsed mode and selection.
- Filter multi-letter prefixes, support Backspace and layered Escape, restore
  previous focus, and hand arrows/Tab back to ribbon navigation. Consume paired
  text and repeated access keys so they cannot modify the document accidentally.
- Guard composition, dialogs, lost window focus and modifier chords. Treat AltGr
  Ctrl+Alt key events separately from command shortcuts while retaining text input.
- Add seven widget regressions, nine badge width/scale cases and four native
  render captures. Verify native macOS F10/Alt+H sequences, size input, menus,
  Font dialog and Save/Open with independently decoded Unicode/styles.

Native content v2 and process protocol v1 stay unchanged. Bare Alt is covered by
synthetic modifier transitions; the macOS input tool cannot send a modifier-only
press. Native Windows Alt/AltGr/IME/DPI and matched Word 2024 checks remain open.
Non-Home command KeyTips, Draw/Help and quick-access hints remain incomplete.
No merge, release, dependency or license change is included.

## 0.1.0-alpha.6 — Unreleased; version approved by owner on 2026-10-17

Prepared 2026-10-17. The owner approved alpha.5 in the project chat; that approval
covers its version number, not a merge, tag or release.

- Open a collapsed ribbon temporarily from a tab or Alt+H, over the canvas without
  reflow. Commands, outside clicks and Escape dismiss it while retaining tabs-only
  mode. Ctrl+F1, double-click and the tab context menu switch the pinned mode.
- Add F10, Tab/Shift+Tab, directional navigation, Ctrl+Left/Right between Home
  groups, Enter/Space and keyboard dropdowns. Disabled commands are skipped;
  menus close one level at a time. Resize moves hidden Editing focus into More.
- Cycle visible document/panes/status/ribbon regions with F6/Shift+F6. Preserve
  native size-field editing, document selection, undo and Unicode save/reopen.
- Add six real-widget regressions and nine temporary-ribbon width/scale cases.
  Inspect three native macOS captures and exercise native keyboard/menu/history,
  F6, pinning and system Save/Open. Update measured UI behavior and remaining gates.

Native content v2 and process protocol v1 stay unchanged. Bare-Alt activation and
full sequential KeyTips remain pending, as do native Windows/Word/DPI/IME reviews.
No merge, release, dependency or license change is included.

## 0.1.0-alpha.5 — Unreleased; version approved by owner on 2026-10-17

Prepared 2026-10-16. The owner approved alpha.4 in the project chat; that approval
covers its version number, not a merge, tag or release.

- Give Home's Font launcher its own native Font dialog, also available through
  Ctrl/Command+D and Ctrl/Command+Shift+F. Keep the Paragraph launcher separate.
- Add compact Font / Font style / Size columns, preset sizes, mixed underline,
  a clipped preview, and OK/Cancel. Unsupported font-family choice stays disabled.
- Stage changes without editing the document; OK applies changed properties in
  one shared undo transaction, preserving untouched mixed runs and Unicode ranges.
  Escape/Cancel keeps the original; invalid sizes keep the dialog open. Document
  shortcuts and window close cannot bypass the modal draft.
- Split Find into a direct navigation action and a separate options arrow; retain
  Select and overflow menus and the 104-DIP Home layout.
- Extend engine and real-widget tests for dialog keyboard/pointer behavior,
  mixed formatting, history, save/reopen and minimum-window bounds at synthetic DPI.
- Verify native macOS Font cancellation/validation/undo and Unicode Save/Open;
  inspect Home/Font at 1366×900 and 640×480 DIP. Capture helper waits for modal layout.

Native content v2 and protocol v1 stay unchanged. No release, merge, dependency or
license change is included. Native Windows/Word comparison remains unverified.

## 0.1.0-alpha.4 — Unreleased; version approved by owner on 2026-10-16

Prepared 2026-10-16. The owner approved alpha.3 in the project chat; that approval
covers its version number, not a merge, tag or release.

- Tighten Home to 104 DIP: compact Clipboard with split Paste, closer Font/Paragraph
  rows, joined size field, original dropdown/vector icons and inset group launchers.
- Replace separate AaBbCc tiles with a single gallery of inline style-name previews;
  styles remain disabled and no applied style is fabricated.
- Add working Grow/Shrink Font buttons and Ctrl/Command+Shift+>/< shortcuts. Each
  selected run steps independently through presets in one shared undo transaction;
  pending caret style, Unicode boundaries and unrelated formatting are preserved.
- Add real Find/Select menus; keep commands reachable via overflow at 640 DIP.
  Extend control/menu geometry and pointer/keyboard regressions.
- Inspect native macOS captures at 1366, 960 and 640 DIP. Verify a native macOS
  keyboard/Unicode-paste → format → undo/redo → system Save/Open loop plus dirty
  New/Cancel/Discard. Automated non-ASCII character typing remains unresolved;
  native IME and Windows/Word comparison are not certified.

Native content v2 and protocol v1 stay unchanged. No release, merge, dependency or
license change is included.

## 0.1.0-alpha.3 — Unreleased; version approved by owner on 2026-10-16

Prepared 2026-10-16. No release or tag is published by this entry.

- Add real Home font-size input/presets (1–400 pt, half-point steps), underline,
  Ctrl/Command+U and Ctrl/Command+Shift+P; show uniform/mixed selection values.
- Keep selection, pending insertion style and exact undo/redo in shared Rust
  commands. Enter/blur applies size, Escape cancels, invalid input preserves the
  document; Save/close resolves pending size before acting on the model.
- Render actual run sizes and underline in the hit-testable text projection;
  document zoom does not modify stored sizes.
- Write native content v2, with strict read-only v1 conversion to 11-pt/no-underline
  defaults. **Older v1-only builds cannot open v2 saves.** No lossy downgrade;
  process protocol remains v1. Add a formatted Unicode v2 fixture and ADR 0003.
- Extend engine/storage/widget regression coverage for Unicode, formatting,
  history, malformed v2 files and failed writes preserving previous bytes.

Native Windows keyboard/DPI/IME/accessibility and a matched Word 2024 comparison
remain unverified. Font-family choice, named styles and pagination remain out of scope.

## 0.1.0-alpha.2 — Unreleased; submitted for owner approval

Prepared 2026-10-16. No release or tag is published by this entry.

- Require a version increment and dated changelog entry for every delivered change
  set, including documentation and journal work, under the updated owner policy.
- Define one coherent work iteration as the increment unit; its validation and
  separate journal commit share that version. Later changes increment it again.
- Advance Cargo package/lock metadata to `0.1.0-alpha.2`; CLI, capabilities and GUI
  Help obtain the version from the same build metadata. Editor behavior, native
  content v1 and process protocol v1 are unchanged.
- Preserve separate owner approval for stage transitions, merge, tags and releases.

## 0.1.0-alpha.1 — Unreleased; submitted for owner approval

Prepared 2026-10-16. No release or tag is published by this entry.

- Standalone Rust document model, shared edit commands, undo/redo and native GUI.
- Custom Home ribbon, File, navigation/formatting panes and real document statistics.
- Refined light desktop shell: original vector controls, compact Font/Paragraph
  rows, style previews, File Home/Options, status zoom buttons and responsive
  bounds tests; native wide/narrow/File captures.
- Versioned `.bfdoc` content v1, checked temporary-file replacement, file errors and
  Save/Discard/Cancel; process integration v1 and independent executable.
- English/Ukrainian/Russian and grapheme regression tests, CPU widget event tests,
  reproducible builds and platform evidence tracked in the validation report.
- Adopt the owner's pre-0.1.0 alpha/beta/RC cycle; supersede the earlier unreleased
  `0.2.0-dev.1` build label. No document-format or process-protocol version change.

Native Windows input/DPI, matched Word 2024 comparison, full shaping/accessibility,
pagination, installers and license adoption remain open. DOCX/PDF/printing and
cloud features are not implemented. See [validation](docs/VALIDATION.md) and
[roadmap](ROADMAP.md) for detailed evidence and gates.
