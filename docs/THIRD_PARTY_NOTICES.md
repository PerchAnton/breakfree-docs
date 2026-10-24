# Dependency and asset notices

Project and brand licenses remain owner decisions. This file does not grant a
project license. Application code/tests are original. No other application's
implementation or branding is vendored. Cargo.lock records exact dependency
versions, including target-specific packages. Each crate retains its upstream
copyright and license files; do not strip them when distributing binaries.

| Direct runtime dependency | Purpose | Upstream license |
| :--- | :--- | :--- |
| serde / serde_json | Native v1/v2 schemas and JSON | MIT OR Apache-2.0 |
| unicode-segmentation | Extended grapheme boundaries and word statistics | MIT OR Apache-2.0 |
| regex | Unicode simple case folding for escaped literal search | MIT OR Apache-2.0 |
| tempfile | Same-directory checked replacement | MIT OR Apache-2.0 |
| eframe / egui / epaint | Native desktop, widgets and text projection | MIT OR Apache-2.0 |
| rfd | Native file picker / Linux XDG portal | MIT |
| arboard | Plain-text OS clipboard | MIT OR Apache-2.0 |
| quick-xml | Bounded namespace-aware DOCX XML parsing | MIT |
| flate2 / crc32fast | DOCX deflate and ZIP checksums | MIT OR Apache-2.0 |
| image | Approved WebP logo decode and PNG capture | MIT OR Apache-2.0 |

Exact resolved macOS versions, repository URLs and declared SPDX expressions are
in [dependencies-macos.json](dependencies-macos.json), generated from locked Cargo
metadata for aarch64-apple-darwin. This is an inventory, not a completed legal or
security audit. Windows/Linux transitive inventories and a full binary-distribution
notice bundle remain a release gate; this PR publishes no binary release/installer.
Source dependency terms are supplied in each downloaded crate.

Egui's default-font dependency embeds Ubuntu, Hack, Noto Emoji and emoji-icon-font.
Their original notices are preserved locally: [Ubuntu Font Licence](licenses/UFL.txt),
[SIL Open Font License](licenses/OFL.txt), [Hack notice](licenses/Hack-Regular.txt)
and [emoji-icon-font MIT](licenses/emoji-icon-font-mit-license.txt). These are real
embedded dependencies, unlike the optional host Arial/Segoe UI/DejaVu fonts, which
are read from the user's system and are not redistributed. Host font availability,
font fallback, color emoji and shaping vary; no universal rendering claim is made.

The runtime embeds the unchanged approved `docs/brand/breakfree-docs.webp` directly.
The historical Python archive retains its identical packaged copy and original
regex/PySide/Qt/build-tool metadata. Those are not dependencies of the Rust editor.
Historical Qt/PySide distributions retain their LGPL/GPL and component notices;
see [Qt for Python licensing](https://doc.qt.io/qtforpython-6/licenses.html).
