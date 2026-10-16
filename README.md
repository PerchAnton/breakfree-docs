<p align="center">
  <img src="docs/brand/breakfree-docs-transparent.webp" alt="BreakFree Docs — document pages with a rainbow ribbon" width="160" height="160">
</p>

<h1 align="center">BreakFree Docs</h1>

<p align="center">
  <strong>Your words. Your freedom.</strong><br>
  An independent local document editor prototype.<br>
  A place for letters, reports and ideas — part of BreakFree Office.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-working%20prototype-8055C8?style=flat-square" alt="Status: working prototype">
  <img src="https://img.shields.io/badge/focus-documents-8055C8?style=flat-square" alt="Focus: documents">
  <img src="https://img.shields.io/badge/suite-BreakFree%20Office-241337?style=flat-square" alt="Part of BreakFree Office">
</p>

<p align="center">
  <a href="#write-without-getting-in-your-own-way">Vision</a> ·
  <a href="ROADMAP.md">Roadmap</a> ·
  <a href="docs/REQUIREMENTS.md">First milestone</a> ·
  <a href="CONTRIBUTING.md">Contribute</a> ·
  <a href="https://github.com/PerchAnton/breakfree-office">Office hub</a>
</p>

---

> [!IMPORTANT]
> **Working Rust editor, native ribbon prototype.** Create, edit, format, undo,
> save and reopen local `.bfdoc` documents in a standalone desktop window. No Office
> installation, cloud service or neighboring repository is required. There are no
> installers, releases or DOCX compatibility claims. The detailed roadmap remains
> the longer-term plan; the [validation report](docs/VALIDATION.md) separates actual
> checks from pending work.

## Write without getting in your own way

BreakFree Docs starts with everyday writing and can grow toward demanding document workflows. The aim is a dependable editor for letters, reports and long-form documents, with local files, clear controls and an extensible foundation.

Start with a small workflow that earns trust. Expand it through demonstrated results, not a list of buttons that do not work.

## The first useful workflow

> **Create → Write → Format → Save → Reopen → Continue.**

The first implementation milestone covers paragraphs, selection, basic emphasis and alignment, undo/redo, safe local persistence and unsaved-change handling. The [first-milestone requirements](docs/REQUIREMENTS.md) remain the acceptance contract for this initial editor.

## Where we are heading

| Foundation | Everyday documents | Advanced workflows |
| :--- | :--- | :--- |
| Independent document model and command system | Styles, lists, tables and images | Comments and tracked changes |
| Unicode input and keyboard access | Pagination, sections and headers | Footnotes, references and equations |
| Undo, recovery and versioned persistence | Search, navigation, print and PDF | Templates, mail merge and automation |
| Tests that run without the interface | A tested DOCX subset | Optional assistant integrations |

The basic D1 writing loop is implemented as a prototype. Advanced capabilities are
planned separate milestones, not promises for the first release. See the roadmap
for the distinction between local evidence and completed milestone approval.

## Built around the document

**Your files stay under your control.** Save writes a temporary file beside the
destination, flushes it, then replaces the destination. Tested failures before
replacement preserve the previous file. There is no autosave, crash recovery or
power-loss guarantee; see [file semantics](docs/NATIVE_FORMAT.md). Unsupported
formats and content versions are rejected explicitly.

**The interface is not the engine.** The value-based document model, commands,
undo/redo, persistence, text projection, native layout and GUI are separate Rust modules.
The GUI and tests use the same edit commands. The process adapter supplies the
optional Office launch interface. Rust is approved by the owner;
[ADR 0002](docs/adr/0002-rust-native-ribbon.md) records the replaceable native GUI
prototype. [UI_SPEC](docs/UI_SPEC.md) defines the Word 2024 Windows reference,
measurable layout and pending comparison gates.

**Text and accessibility come first.** Input methods, international text, keyboard navigation, screen-reader behavior and font fallback must influence architecture selection, not wait until the final polish stage.

**A distinct member of one family.** Docs has its own approved page icon, with the shared aubergine tile and rainbow ribbon expressing freedom and inclusion.

## Roadmap and evidence

The [detailed roadmap](ROADMAP.md) defines architecture proposals, feature areas, milestone dependencies, release gates, compatibility evidence and performance investigations. The [suite development standard](https://github.com/PerchAnton/breakfree-office/blob/main/docs/DEVELOPMENT_STANDARD.md) describes how progress must be reported.

DOCX support will be measured separately for reading, editing and writing. Text preservation, page layout, unsupported elements and font substitution need separate results. Opening one sample is not proof of general compatibility.

## Versions and release approval

The first cycle is `0.1.0-alpha.N` → `0.1.0-beta.N` → `0.1.0-rc.N` → `0.1.0`.
After publishing the first stable release, development moves to `0.2.0`. Read
[versioning and approval](docs/VERSIONING.md) and [CHANGELOG](CHANGELOG.md).
`Cargo.toml` defines the current application version; `--version`, capabilities
and GUI Help show that same build version. Every delivered change set increments
the prerelease counter and updates CHANGELOG, including documentation-only work.
Every work report ends with the current version for owner approval. No release
or tag is automatically published.

## Build and run

Install [Rust](https://rust-lang.org/tools/install/); `rust-toolchain.toml` selects
1.99.1 and `Cargo.lock` pins dependencies. All runtime layers are Rust. No Office,
Python, Node, neighboring repository or cloud account is required. Dependency
installation needs registry access; the installed editor makes no network requests.
The earlier Python experiment is retained only in `prototypes/python/`.

On macOS install Xcode Command Line Tools. On Debian/Ubuntu install native build
libraries (`pkg-config libxkbcommon-dev libwayland-dev libgl1-mesa-dev libegl1-mesa-dev`)
and use a working desktop/OpenGL session. Linux file dialogs use an XDG portal.
Then, from this checkout:

```sh
cargo build --locked --release
./target/release/breakfree-docs --new
./target/release/breakfree-docs --open "/absolute/path/My document.bfdoc"
./target/release/breakfree-docs --capabilities
./target/release/breakfree-docs --version
```

On Windows install Rust's MSVC toolchain and Visual Studio Build Tools with
**Desktop development with C++** (MSVC and Windows SDK). In PowerShell:

```powershell
cargo build --locked --release
.\target\release\breakfree-docs.exe --new
.\target\release\breakfree-docs.exe --open "C:\Users\YourName\Documents\My document.bfdoc"
.\target\release\breakfree-docs.exe --capabilities
.\target\release\breakfree-docs.exe --version
```

The native executable is `breakfree-docs` (`.exe` on Windows); there is no installer,
file association or signing yet. Build/CI success is distinct from native-platform
input validation; see [VALIDATION](docs/VALIDATION.md). `cargo run --locked -- --new`
is convenient during development. `cargo build --locked --no-default-features`
builds a model/CLI binary that answers information commands without a GUI toolkit;
it advertises no desktop launch actions and rejects launch attempts clearly.

## Use the editor

Open [examples/Welcome.bfdoc](examples/Welcome.bfdoc) for the original v1 fixture or
[examples/Formatting.bfdoc](examples/Formatting.bfdoc) for sized/underlined Unicode text.
Home has real clipboard, bold/italic/underline, font size, alignment, Find/Replace
and Select All commands. At a collapsed caret, character formatting sets the next
insertion style. The size field accepts 1–400 pt in half-point steps (for example,
`12.5`); Enter or leaving the field applies it and Escape cancels. Mixed selected
sizes show an empty field. Grow/Shrink Font steps each selected run independently
through common presets, with one undo operation. At a caret it changes pending
insertion size. Native paste is plain text; the Paste menu explains this scope.
Find and Select menus expose their working commands. Unimplemented font-family,
style and list commands are disabled with explanations.

This build reads native content v1 and v2 and **saves v2**. Opening a v1 file does
not change it; an explicit save converts it to v2. Earlier v1-only builds cannot
open new saves. Use Save As to keep an older copy. See [format details](docs/NATIVE_FORMAT.md).
File provides New/Open/Save/Save As, unsaved Save/Discard/Cancel and view settings;
Print is disabled. Navigation and the formatting pane operate on real model ranges.
The white canvas is continuous; pagination, page numbers and print layout are absent.
Status reports real word/paragraph counts and zoom. The ribbon uses compact vector controls, two-row Font/Paragraph groups and
disabled style previews; it compacts and moves commands into overflow menus as
the window narrows. File has Home/Options navigation and real document properties.
The status bar provides zoom out/in and a click-to-reset percentage.
See [current native screenshots and capture commands](docs/UI_SPEC.md#actual-application-capture).

Ctrl (Command on macOS) + N/O/S, Shift+S, B/I/U, Z/Y or Shift+Z handle file, emphasis
and history actions. Ctrl/Command+Shift+P focuses and selects the size field.
Ctrl/Command+Shift+> / < grows/shrinks the selected text (period/comma keys).
Ctrl/Command + L/E/R/J aligns paragraphs; F/H opens Find/Replace.
Ctrl+F1 collapses the ribbon; Alt+F/H opens File/Home; Alt+Q focuses command search;
F6 cycles document/search/navigation focus. Tab traverses controls. Input events delivered in one widget update form one text undo operation;
explicit paste and formatting commands have separate boundaries. Full
Word KeyTips and assistive-technology behavior are pending review.

## Check

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --locked --no-default-features
python3 tools/check_repository.py
```

Python is optional for the documentation/brand check only. GUI event tests run the
actual egui widgets without creating OS windows; they do not prove native IME or
Windows scaling. To capture this application's native rendering on a desktop:

```sh
mkdir -p artifacts
EFRAME_SCREENSHOT_TO="$PWD/artifacts/editor.png" cargo run --locked --features capture -- --new
```

PowerShell: create `artifacts`, set `$env:EFRAME_SCREENSHOT_TO` to an absolute PNG
path, then run the same Cargo command. The capture-only feature exits after a frame.
See [UI_SPEC](docs/UI_SPEC.md) for comparison conditions and remaining visual gates.

## Development status

| Area | Current position |
| :--- | :--- |
| Identity and planning documents | Available |
| Implementation language | Rust approved; language decision complete |
| UI toolkit and native persistence format | Native eframe/egui ribbon experiment; strict `.bfdoc` v1/v2 reading, v2 writing |
| Windows, macOS and Linux | macOS local build/render; native Windows/Linux evaluation pending |
| Application code and tests | Available; [evidence and limits](docs/VALIDATION.md) |
| Releases and installers | None |
| License | Owner decision pending |

Docs remains the first development priority within the suite. The
[local process contract v1](contracts/v1/BASELINE.md) is provisional: Office has
not published a contract at the inspected source commit, so upstream conformance
and launching from an actual Office shell remain unverified.

## The BreakFree family

| Project | Purpose |
| :--- | :--- |
| [**BreakFree Office**](https://github.com/PerchAnton/breakfree-office) | Suite hub, identity and shared direction |
| **BreakFree Docs** | Documents — you are here |
| [**BreakFree Sheets**](https://github.com/PerchAnton/breakfree-sheets) | Spreadsheets and calculations |
| [**BreakFree Slides**](https://github.com/PerchAnton/breakfree-slides) | Presentations and visual storytelling |

## Get involved

Propose a writing workflow, accessibility improvement or technical experiment in [Issues](https://github.com/PerchAnton/breakfree-docs/issues). Read [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md). Use synthetic documents, not customer data or private correspondence.

Everyone is welcome, including LGBTQ+ contributors and users. Follow the [shared Code of Conduct](https://github.com/PerchAnton/breakfree-office/blob/main/CODE_OF_CONDUCT.md).

## License and independence

The software and brand-asset licenses remain owner decisions. Public visibility is not a grant of reuse rights. Third-party dependencies and assets must retain any required notices.

BreakFree Docs is an independent project, not affiliated with Microsoft. References to Word and DOCX describe intended workflows and interoperability goals, not endorsement or verified equivalence.

---

<p align="center"><strong>BreakFree Office · Your work. Your freedom.</strong><br><sub>Created by <a href="https://github.com/PerchAnton">Anton Perchenko</a></sub></p>
