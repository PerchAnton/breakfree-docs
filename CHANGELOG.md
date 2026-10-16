# Changelog

Only published entries are releases. This file records candidate scope; version
and publication approval follow [docs/VERSIONING.md](docs/VERSIONING.md).

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
