# Changelog

Only published entries are releases. This file records candidate scope; version
and publication approval follow [docs/VERSIONING.md](docs/VERSIONING.md).

## 0.1.0-alpha.7 — Unreleased; submitted for owner approval

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
