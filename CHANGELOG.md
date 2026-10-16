# Changelog

Only published entries are releases. This file records candidate scope; version
and publication approval follow [docs/VERSIONING.md](docs/VERSIONING.md).

## 0.1.0-alpha.4 — Unreleased; submitted for owner approval

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
