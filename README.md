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
Find opens navigation directly; its arrow and Select expose command menus.
Navigation shows the active result number and highlighted context for all literal
matches. Match case starts enabled; Whole words starts disabled. Tab/Space changes
these window-local search options without changing the document. Previous/Next
buttons or Enter/Down and Shift+Enter/Up in
the search field wrap through the matches without leaving that field. Clear resets
the query; Escape closes the pane and returns to the document. Selecting a result
scrolls the continuous canvas to it. Search respects complete combining/emoji
clusters. Ignoring case uses Unicode simple folding; Whole words uses Unicode
word boundaries, including internal apostrophes. Search does not normalize text,
expand ß to SS, or apply locale-specific casing. Wildcards are not supported.
Queries above 4,096 UTF-8 bytes show an error without truncation.
Replace uses the same matches and existing one-operation undo boundary. Replace
All… opens a read-only preview with match/change counts and before/after excerpts.
Cancel or Escape leaves the document unchanged; Tab from the initially focused
Cancel button to Apply, then Enter, applies one transaction. One Undo restores all
text, formatting and the prior selection. Equal-text matches are skipped without
changing their mixed formatting. Inserted text takes each match's first character
style. Over-limit results and stale previews are rejected before editing.
[Batch replacement semantics](docs/adr/0005-replace-all-preview.md).
[Navigation details and evidence](docs/UI_SPEC.md#alpha15-search-options).
The Font launcher opens a dialog for style, size and underline with a preview.
Only OK applies the draft, together in one undo operation; Cancel/Escape discards
it. Mixed properties stay unchanged unless edited. Unimplemented font-family,
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
The status bar provides zoom out/in and a percentage button that opens Zoom.
View groups real Ruler/Navigation checkboxes, Zoom, 100% and Canvas Width. Width
fit tracks the window and side panes within 50–200%; manual zoom exits fit mode.
See [current Home layout captures](docs/UI_SPEC.md#alpha11-native-captures) and
[reproducible capture commands](docs/UI_SPEC.md#actual-application-capture).

Ctrl (Command on macOS) + N/O/S, Shift+S, B/I/U, Z/Y or Shift+Z handle file, emphasis
and history actions. Ctrl/Command+Shift+P focuses and selects the size field.
Ctrl/Command+D or Ctrl/Command+Shift+F opens the Font dialog.
Ctrl/Command+Shift+> / < grows/shrinks the selected text (period/comma keys).
Ctrl/Command + L/E/R/J aligns paragraphs; F opens Find. Ctrl+H opens Replace
on Windows/Linux. On macOS use Home → Replace: Command+H is reserved by the OS.
Ctrl+F1 switches pinned/tabs-only ribbon mode; double-click a tab or use its context
menu for the same choice. A collapsed tab opens temporarily over the canvas and
closes after a command, outside click or Escape. Alt+F opens File; Alt+H enters Home;
Alt+Q focuses command search. Tap Alt (release it without another key) or F10 to
show KeyTips. Then press H for Home: 1/2/3 toggles bold/italic/underline, F then S
focuses size, F then N opens Font, and A then L/C/R/J aligns. File uses F, then
N/O/S/A/T for New/Open/Save/Save As/Options. H → E opens narrow-window More.
Top-level 1/2/3 invokes Save/Undo/Redo on the default Quick Access Toolbar (also
Alt+1/2/3). Alt, 0 opens customization: show/hide New/Open/Save/Undo/Redo, move
the toolbar above/below the ribbon, or reset. New entries append; hints 1–5 follow
visible order, and disabled actions keep their place. Command visibility, order and
above/below placement persist across launches. Visibility and placement changes
save immediately. Choose More Commands to reorder the current commands: Up/Down
selects, Alt+Up/Down or the move buttons changes order, and OK applies and saves.
Cancel/Escape discards the draft; unchanged OK does not write. Tab/Shift+Tab cycles
the list, enabled move buttons and OK/Cancel. From focused tabs, Up enters the toolbar (Down also works below the
ribbon); Left/Right skips unavailable actions, Down opens options, Tab returns to
tabs and Escape backs out. N/JI/G/P/S/M/R/W/Y1 selects the
other tabs. T opens More tabs when the row overflows. View W → K/R toggles
Navigation Pane/Canvas Rulers; W → Q opens Zoom with a validated 50–200% draft.
W → J restores 100%; W → C fits the continuous canvas width.
Unimplemented tabs explain their limits and retain Escape navigation.
Backspace removes a prefix; Escape clears it, backs up a level, then restores
previous focus. Invalid/repeated hint keys cannot enter document text.
Tab/Shift+Tab and
arrows navigate enabled Home/View controls, Ctrl+Left/Right changes groups, Down or
Alt+Down opens dropdowns, Enter/Space activates. Escape closes menu, ribbon and
focus levels in order. F6/Shift+F6 cycles document, visible panes, status and ribbon. Input events delivered in one widget update form one text undo operation;
explicit paste and formatting commands have separate boundaries. See the
[implemented KeyTips and limits](docs/UI_SPEC.md#alpha9-quick-access-and-view).
Bare Alt is implemented and event-tested; native Windows Alt/AltGr, exact Word 2024 mappings and assistive-technology behavior
remain review gates. Draw/Help/ruler/100% assignments are provisional; unsupported-tab
commands and broader toolbar customization remain roadmap work. See the
[ordering dialog](docs/UI_SPEC.md#alpha12-toolbar-ordering).

## Interface palette

The native shell uses light neutral surfaces, dark outlines and small colored
icon details: gold Clipboard/Open, blue file and editing details, and muted
indicators on unavailable formatting commands. Violet remains the BreakFree
focus/selection accent. All icons are original Rust vectors; color does not
indicate a newly implemented command. File keeps working Home/Options actions
and explained disabled Print. [Palette and captures](docs/UI_SPEC.md#alpha13-icon-palette)
describe the reference scope and remaining differences.

## Local toolbar settings

Settings apply to this user/profile across documents. Reset Toolbar asks for
confirmation and keeps the previous settings file as a backup before restoring
Save/Undo/Redo above the ribbon. Cancel/Escape changes nothing. A failed save shows
an error and a menu warning; the toolbar remains usable in the current window.
Use Retry Saving after fixing permissions or another transient cause. Corrupt or
future settings use defaults with an error and are not automatically overwritten.

Windows stores settings at `%APPDATA%\BreakFree\Docs\settings.json`, macOS at
`$HOME/Library/Application Support/BreakFree/Docs/settings.json`, and Linux under
`$XDG_CONFIG_HOME/breakfree-docs/settings.json` (default `$HOME/.config`). For an
isolated test profile, set an absolute directory before starting:

```powershell
$env:BREAKFREE_DOCS_CONFIG_DIR = Join-Path $env:TEMP "breakfree-docs-review"
.\target\release\breakfree-docs.exe --new
```

On macOS/Linux use `BREAKFREE_DOCS_CONFIG_DIR=/absolute/profile/path` before the
same executable command. Close and reopen Docs to verify persistence; there is
no extra launch flag. `--version` and `--capabilities` neither read nor write
settings. Other windows do not update live; conflicting writes are rejected.
[ADR 0004](docs/adr/0004-local-toolbar-preferences.md) defines the schema, paths,
backup/reset behavior and lock/power-loss limits. Document files and history are
independent of preferences.

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
