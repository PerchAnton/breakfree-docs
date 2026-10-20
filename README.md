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
Clear character formatting (the Home eraser, Ctrl+Space in the document, or
KeyTips H → FC) removes direct bold/italic/underline and size overrides in one
Undo operation, restoring the paragraph style. Text and paragraph alignment remain unchanged. At a caret it resets
only future input; it does not expand to the surrounding word. This is the
supported character-only reset; paragraph style and alignment remain intact.
Control remains Control on macOS; system input-source shortcuts may intercept it.
The ribbon and command search provide the same action without that chord.
[Navigation details and evidence](docs/UI_SPEC.md#alpha15-search-options).
The Font launcher opens a dialog for style, size and underline with a preview.
Only OK applies the draft, together in one undo operation; Cancel/Escape discards
it. Mixed properties stay unchanged unless edited. Unimplemented font-family and list commands are disabled with explanations.

Home Styles applies Normal, No Spacing or Heading 1 to selected paragraphs in one
undo. Direct formatting and alignment remain unchanged; Clear character formatting
returns the selected text to its paragraph style. No Spacing changes real spacing.
Older files retain their explicit formatting until cleared. New paragraphs repeat
the current style unless a following style is explicitly selected. Compact layouts offer all styles through
More styles or More. [Inheritance rules](docs/adr/0006-built-in-paragraph-styles.md).

Modify an existing definition through Home → More styles → Modify style, or
right-click its gallery card. Change its parent, emphasis, size and paragraph
spacing in a local draft. Blank fields inherit; On/Off explicitly override a
parent. Invalid cycles disable OK. Cancel/Escape changes nothing; OK updates
inheriting paragraphs and descendants in one Undo, preserving direct formatting.
These settings belong only to the current document. [Definition rules](docs/adr/0007-editable-style-definitions.md).

Home Styles → Create style opens a named-style draft. Choose a built-in/custom
parent or No style, enter a unique name and optionally adjust supported properties.
Apply to selected paragraphs is checked initially; creation and application use
one Undo. Uncheck it to create the definition only. Cancel/Escape changes nothing.
The current document supports up to 64 custom styles; names are 1–64 Unicode
characters, without outer whitespace/control characters, unique after Unicode
lowercasing. Styles remain available in More styles and command search after reopen.
KeyTips H → L → C (wide/compact) or H → E → C (minimum width) opens creation.
Modify Style also renames custom styles: the current name is selected on entry.
Confirm name, parent and property changes together in one undo; IDs and all
paragraph/descendant references stay intact. Built-in names remain fixed.
Right-click a gallery card or any Styles-list row → Modify style to edit it
without applying it to the document. Cancel/unchanged OK preserves redo history.
Delete style opens a draft with separate replacements for paragraphs and direct
child styles (including a No style parent). Counts reflect actual references;
inherited appearance may change, while direct formatting and text remain intact.
Delete commits once; one Undo restores the definition and all references.
Cancel/Escape changes nothing; built-ins cannot be deleted and cycles are rejected.
Use the style's context menu to delete an unused definition without applying it.
KeyTips H → L → D or narrow H → E → D opens the selected style's draft.
Following paragraph in Create/Modify Style selects Same style or a built-in/custom
style. Enter applies it to the new right paragraph in one Undo/Redo; paste does not.
This choice is independent of the parent. Deletion remaps following references to
the paragraph replacement. [Following-style rules](docs/adr/0010-following-paragraph-styles.md).
Home → Styles also offers **Strong**, **Emphasis** and **Default paragraph font**.
The cascade is paragraph style → character style → direct formatting: direct
`false` wins too. Each run has at most one character style. Applying/removing it
preserves direct overrides; Clear character formatting removes only direct
properties. Select text to change it in one Undo/Redo, or use a caret to set an
undoable insertion style without dirtying the document until text is entered.
After F10/Alt → H → L (narrow: H → E), B/I/Z applies Strong/Emphasis/removes the
character style. Arrow navigation scrolls the Styles list; command search also
finds these actions. Built-ins remain fixed.

Home Styles → Create character style opens a local draft with a unique name,
optional Bold/Italic/Underline (Inherit/On/Off), font size and a preview based on
the active paragraph. Blank size inherits. Apply to selection / future typing is
checked initially; definition creation and application share one Undo/Redo.
Uncheck it to create an unused definition. Cancel/Escape changes nothing.
Names are 1–64 Unicode scalars, unique ignoring case within character styles;
up to 64 custom character styles are stored per document. They appear in the
scrollable Character styles list and command search after reopening. KeyTips
H → L → K (narrow: H → E → K) opens creation. Paragraph and character names are
separate namespaces. Direct formatting is preserved and may mask the new style.
Select text carrying one custom character style, then Home Styles → Modify
character style (KeyTips H → L → J; narrow H → E → J). Change Inherit/On/Off
properties or size, preview, then OK to commit one Undo/Redo. Cancel changes
nothing. Right-click a custom style in the list to modify even an unused definition.
IDs, references and direct overrides stay unchanged; direct overrides can
mask definition changes. Built-ins and mixed selections disable the main command.
The name is selected on entry: type a new unique name to rename the style.
Name and property changes commit together in one Undo/Redo. Cancel/Escape changes
nothing; invalid or reserved names keep OK disabled. Home and command search show
the new name after confirmation and reopen. Unchanged drafts preserve redo.
Home Styles → Delete character style opens a replacement draft for a uniform
custom style (F10/Alt → H → L → X; narrow H → E → X). Its context menu also deletes
unused definitions. Choose another custom style, Strong/Emphasis or Default
paragraph font to remove references. All text and future-typing references are
remapped together; direct overrides and paragraph styles stay. Appearance may
change. Delete commits one Undo/Redo; Cancel/Escape changes nothing. Cancel starts
focused, so Enter alone does not delete. Built-ins cannot be deleted.
Create/Modify Character Style includes Style based on: a custom style, Strong,
Emphasis or (Paragraph style). Unset properties follow the parent chain; direct
formatting wins. Invalid parent cycles disable OK. Name, parent and properties
commit together in one Undo/Redo. Deleting a parent also remaps its direct children
to the replacement and rejects a descendant replacement that would create a cycle.
[Named character styles](docs/adr/0012-named-character-styles.md).
[Character-style rules](docs/adr/0011-character-style-precedence.md).
[Style rules](docs/adr/0008-custom-named-styles.md) · [Deletion rules](docs/adr/0009-delete-custom-styles.md).

This build reads native content v1/v2/v3/v4/v5/v6/v7/v8/v9/v10/v11/v12/v13 and **saves v13**. Opening an older file
does not change it; an explicit save converts it to v13. Earlier builds cannot
open new v13 saves. Use Save As to keep an older copy. See [format details](docs/NATIVE_FORMAT.md).
File provides New/Open/Save/Save As, unsaved Save/Discard/Cancel and view settings;
Print is disabled. Navigation and the formatting pane operate on real model ranges.
The white canvas is continuous; pagination, page numbers and print layout are absent.
Status reports real word/paragraph counts and zoom. The ribbon uses compact vector controls, two-row Font/Paragraph groups and
an active paragraph-style gallery; it compacts and moves commands into overflow menus as
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
| UI toolkit and native persistence format | Native eframe/egui ribbon experiment; strict `.bfdoc` v1/v2/v3 reading, v3 writing |
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

## Formatting provenance (alpha.30)

Open Home's Formatting pane to inspect Bold, Italic, Underline and font size.
Each row shows its resolved value and authored source, including the named parent
that supplies an inherited value. Equal-looking direct overrides remain direct.
Mixed values and mixed sources are reported independently. With no selection,
the pane describes future typing; selected paragraph separators have no character
properties. This read-only view does not change document/history or native v9.
Alpha.31 adds paragraph-property provenance below; style-management filtering remains future work.

## Paragraph provenance (alpha.31)

Formatting also shows Alignment, Line spacing (percent) and Space after (points).
Alignment is an explicit paragraph value in the current model, labeled
“Paragraph value (not inherited)”; it is not attributed to a style or guessed
from appearance. Intervals show the nearest supplying paragraph style/ancestor,
or Document default. Explicit zero is preserved. Space before and direct interval overrides are supported from alpha.42; the inspector identifies their supplying style or direct source.

The paragraph section uses the same half-open selection rule as alignment/style
commands: an exclusive end at the next paragraph start excludes that paragraph;
a caret or selected empty paragraph still has paragraph properties. Mixed values
and sources are independent. Inspection never mutates the document or undo history.
Native content stays v9; palette and icons are unchanged.

## Style filtering (alpha.32)

Open Home → More styles (or the compact Editing menu). Filter by name and cycle
All / Paragraph / Character / Custom using the scope button. Reset clears both.
The list reports no matches explicitly; management commands remain available.
F3 focuses the name field; Down enters the filtered command list, Enter applies,
and Escape closes the popup. Tab/Space operate the filter controls.

Filtering is view-only and session-local: no document/history/settings writes.
Search trims surrounding whitespace and compares Unicode lowercase substrings;
canonical normalization, locale-sensitive folding and fuzzy matching are not
claimed. Built-in/default-character entries are excluded by Custom. Existing
style application and context editing retain stable IDs. Native v9, palette and
icons are unchanged.

## Used styles (alpha.33)

Enable **Used in document** in More styles. This intersects name/category filters
and includes styles assigned to every paragraph (even empty ones), character
styles referenced by text, and all transitive parents. Direct formatting does
not remove a semantic style dependency. Unused descendants/siblings, next-style
links and pending typing alone do not count. Default paragraph font is shown for
text without a character style or an empty paragraph. Reset clears all filters.
Usage is recomputed from the current model, so edits and Undo/Redo update the list;
filtering itself never modifies content/history/settings. Native v9 is unchanged.

## Style usage labels (alpha.34)

Home style menus show Direct use, Parent dependency or Not used beneath each
name, including when the used-only filter is off. Direct assignment wins if a
style is also an ancestor. Parent dependency means inheritance-only use, not
direct formatting. Empty paragraphs and default character formatting retain the
alpha.33 rules; pending typing and next-style links do not count. Search matches
style names, not these labels. Labels are read-only, recomputed after edits and
Undo/Redo, and do not change stable IDs, ordering, storage or settings.

## Dependency consumers (alpha.35)

Style rows with assigned descendants show Required by followed by their names.
Consumers are unique directly assigned styles in document encounter order; the
walk includes every transitive parent, excludes self and unassigned intermediate
styles, and is bounded for invalid external cycles. Paragraph and character
style namespaces remain separate. Direct-use parents also show their consumers.
Names wrap inside the existing scroll area; filtering does not hide dependency
evidence. Renaming, editing and Undo/Redo are reflected from the current model.
No document, history, settings, palette, icons or native format changes.

## Dependency navigation (alpha.36)

Activate a Required by name to reveal its style row without applying it. The
menu stays open; name/category filters clear, Used in document stays unchanged,
and the destination receives keyboard focus and scrolls into view. Links use
stable paragraph/character IDs, so duplicate names across namespaces are safe.
Enter/Space activates the navigation target; a separate activation of the focused
style applies it. Escape closes the menu. Document/history/settings stay unchanged.

## Return to parent (alpha.37)

Back to parent restores the source of the first Required by jump together with
its exact query, category and used-only setting. Further jumps and filter edits
do not replace that return point. Pointer or keyboard activation keeps the menu
open, focuses and scrolls to the original style without applying it. The return
point is consumed on return and discarded when the menu closes; it is not saved
in the document, settings or undo history. The list shrinks from 125 to 100 DIP
while Back is available so the minimum-height popup remains bounded.

## Named return destination (alpha.38)

The return button now says Back to: followed by the original parent style name.
The name is resolved from the stored stable ID on each frame, including custom
paragraph and character names. Its 23-DIP-high button fits the 220-DIP menu width and truncates long labels;
the tooltip retains the full name and restoration explanation. Return behavior,
palette, icons, document/history and storage are unchanged.


## Dependency navigation resilience (alpha.39)

Pending jumps are checked against the current document and filters. If a source
was removed, renamed outside the saved query or excluded by changed usage,
Return restores the exact filters, explains why the row cannot be shown and
focuses the filter. It does not apply a replacement style. Explicit filter edits
cancel pending navigation and clear the notice. Navigation does not change
content or undo history. A 32-style reverse-catalog chain and repeated keyboard
jumps are covered by widget tests; native Windows acceptance remains open.


## Windows navigation acceptance preparation (alpha.40)

An 18-case CPU widget matrix exercises dependency navigation at synthetic
100/125/150% scale, three window sizes and expanded/collapsed Home.
The [Windows acceptance checklist](docs/WINDOWS_ACCEPTANCE.md) defines the separate native gate.
A Unicode 32-style fixture and a Windows CI review artifact with source/version/hash
metadata make the procedure reproducible. No suitable Windows desktop was
available; native keyboard, display scaling and Word comparison remain unverified.
The review artifact is not a release. Runtime behavior, palette and icons are unchanged.


## Direct paragraph spacing (alpha.41)

Home's existing spacing icon or F10 → H → PS opens Paragraph spacing; command
search also finds it. Enter a whole line-height percentage (100–300) or space
after in points (0–200, in 0.5 pt steps). Editing checks that property; unchecked
fields keep their existing per-paragraph values, including mixed selections.
A checked blank field inherits its style; Use style spacing resets all three. OK is
one undo operation; Cancel/Escape changes nothing. The selection uses the same
half-open paragraph range as alignment. Direct spacing survives style changes,
Enter, joins and Replace All. Formatting inspection identifies direct overrides.

Alignment stays independent; first-line/hanging indents,
tabs and pagination remain deferred. [ADR 0013](docs/adr/0013-direct-paragraph-spacing.md)
defines native v10 and editing policy. Windows acceptance is deferred by the owner.

## Space before paragraph (alpha.42)

Home → Paragraph spacing supports Space before in points (0–200, steps of 0.5).
Check the field to change it; leave it unchecked to preserve each selected value.
A checked blank field inherits from the paragraph style. Use style spacing clears
all three direct interval overrides in one Undo/Redo. Paragraph style creation and
modification also support space before; a blank style value inherits its parent,
with zero as the document default. Explicit zero overrides a nonzero ancestor.
Formatting inspection reports the source. Spacing applies once before each
paragraph, including the first, and adds to the previous paragraph's space after.
This continuous layout policy is not pagination. Native v11 preserves the property;
v1–v10 imports resolve to zero before. Windows acceptance remains deferred.

## Paragraph indents (alpha.43)

Home → Paragraph spacing now includes Left indent and Right indent, in points
(0–144, half-point steps). Each checkbox selects a property to change; unchecked
fields keep existing values. A checked blank restores style inheritance. Use style
spacing and indents resets all five properties in one Undo/Redo. Paragraph style
creation/modification exposes the same fields; blank inherits the nearest ancestor,
then document default zero. Direct zero explicitly cancels an inherited indent.
Formatting inspection identifies each supplying style or direct override.

Wrap and alignment use the indented column. On narrow views, horizontal indents
shrink proportionally to retain up to 32 DIP of text width; document values remain
unchanged. This is continuous layout, not pagination. Enter/join/Replace All follow
the existing paragraph geometry copying policy. Native v12 preserves both indents;
v1–v11 imports use zero. First-line/hanging indents, negative outdents and interactive
ruler keyboard nudging remains a separate increment. Windows acceptance remains deferred.

## First-line and hanging indents (alpha.44)

Home → Paragraph spacing → Special indent changes only the checked property.
Choose First line or Hanging and enter 0–144 pt in 0.5 pt steps. First line moves
the first row right from the left indent; Hanging leaves the first row at the left
indent and moves continuation rows right by the amount. The left indent denotes
the outermost edge in this continuous-layout policy. Right indent is unchanged.

A checked blank inherits the nearest paragraph style/ancestor; explicit zero
disables the inherited special indent. Use style spacing and indents resets all
six properties together. Style creation/modification has the same type and amount.
A blank style amount inherits both type and magnitude regardless of the displayed
type choice. Document default is zero. Apply/reset/parent changes each use one
Undo/Redo. Formatting inspection reports type, amount and source.

Wrapping uses distinct first-row/continuation widths without inserting characters
or paragraph breaks. Narrow views proportionally reduce visual left/right/special
indents to retain up to 32 DIP of text width; saved values do not change. Enter,
joins and Replace All retain the established geometry copying policy. Native v13
preserves the signed property; older imports inherit zero. Negative outer-edge outdents and pagination remain future work. Windows is deferred.

## Indent layout hardening (alpha.45)

Justified trailing spaces retain their logical cursor positions after visible ink.
The visual text column is unchanged; a row-end caret can extend through trailing
whitespace beyond the visible justified edge. This avoids moving the caret before
a stored space at a soft wrap. Tests cover long mixed-format Unicode paragraphs,
empty/trailing paragraphs, narrow views, synthetic scale and zoom/edit/history.
Compact style creation reserves extra room for differing interface-font metrics.
Native content stays v13; native Windows acceptance remains pending.

## Interactive ruler prototype (alpha.46)

With View → Ruler enabled, drag the upper triangle for the first line, the lower
triangle for continuation lines, the rectangle for the overall left indent, or
the right triangle for the right indent. The first selected paragraph supplies
starting values; changed properties apply to selected paragraphs, excluding an
exclusive end at the next paragraph start. Values snap to 0.5 pt within existing
0–144 pt indent / ±144 pt special-indent limits. Other properties stay inherited.

Motion previews without editing the document. Release commits one “Ruler indent”
Undo/Redo. Escape, focus/pointer loss or a changed canvas/window transform cancels;
returning to the original value preserves inheritance and redo. Text/shortcuts
are blocked during dragging. Home → Paragraph spacing remains the keyboard
alternative. This is a continuous-canvas prototype: no margins, tab stops,
pagination yet. Alpha.49 adds a read-only vertical canvas ruler.

## Ruler keyboard control and values (alpha.47)

Ctrl+Shift+F8 reveals the ruler and focuses its first-line marker (a provisional
Docs shortcut). Tab/Shift+Tab cycles first line, continuation lines, overall left
and right. Left/Right previews 0.5 pt movement; Shift+Left/Right moves 5 pt. Enter
commits one Undo/Redo and keeps focus. Tab commits the current marker before moving
to another. Escape or F6 cancels the pending preview and returns to the document.
Focus loss, resize or pointer takeover also cancels uncommitted changes. At a bound,
reverse direction responds immediately. Text and editing shortcuts stay blocked
while controlling markers; return to the document to use Undo/Redo.

The readout shows exact resolved left/right and first-line/hanging amounts in
points for the first selected paragraph, including the current preview. Zero is
shown as 0.0 pt; zoom never changes stored/reported values. Left remains the
outermost edge, while First/Hanging are additional distances. The first-line and
continuation handles preserve the opposite origin. No native-format change.

## Mixed paragraph indents (alpha.48)

The ruler readout shows Mixed independently for Left, Right, First and Hanging
when selected paragraphs have different resolved amounts. Equal values from
different styles or direct overrides remain numeric. Hover a marker
for exact minimum/maximum values. A caret includes its paragraph; an exclusive end
at the next paragraph start does not include that next paragraph.

A small dash beside a marker means its logical origin differs across the selection.
First/continuation origins include the left indent, so a common marker origin can
coexist with mixed individual amounts. Markers stay anchored to the first selected
paragraph; tooltips identify this baseline. Editing still applies changed properties
to selected paragraphs with one transaction. Indicators follow the live preview,
cancel and Undo/Redo. Inspection never creates overrides or history entries.


## Compact two-axis rulers (alpha.49)

View → Ruler shows both axes. The horizontal ruler stays above the document when
scrolling; the vertical ruler follows the continuous canvas text origin. Both use
inches (eight subdivisions per inch) and follow document zoom. The 24-DIP horizontal
and 20-DIP vertical scales retain neutral margins and compact indent markers.
Exact values appear in marker tooltips and during keyboard/drag editing without a
permanent readout above the document. Keyboard shortcuts and one-step Undo/Redo are
unchanged. Vertical ruler margins are read-only; tab stops, editable page margins
and pagination remain future work. Windows/reference matching remains unverified.


## Reset ruler indents (alpha.50)

Right-click the horizontal ruler and choose **Reset indents to style**. Alternatively,
press Ctrl+Shift+F8 to enter the markers, then Delete. The preview removes direct
left/right and first-line/hanging overrides from the selected paragraphs; each
paragraph inherits its own style. Enter applies in one Undo/Redo step; Escape or F6
cancels. Tab applies before changing markers. Arrow nudging is paused during reset
preview. Spacing and alignment remain unchanged. A reset with no overrides creates
no history entry and preserves redo. The vertical ruler remains read-only.


## Individual ruler resets (alpha.51)

The horizontal ruler context menu also resets **Left**, **Right**, or
**First-line/hanging** independently. Enter marker control with Ctrl+Shift+F8,
select a marker with Tab, then press **Shift+Delete** to preview its property reset.
First-line and continuation markers refer to the same signed property; resetting
it preserves the direct left indent. Plain Delete still resets all three properties.
Enter/Tab accepts one Undo/Redo operation, Escape/F6 cancels. Other direct indents,
spacing, alignment and content are preserved, including when the chosen property
already inherits its value (no-op reset preserves redo).


## Ruler value sources (alpha.52)

Hover an indent marker to see whether its value comes from direct paragraph
formatting, its paragraph style, an ancestor style (named), or the document default.
Source groups include paragraph counts; equal values can still have Mixed sources.
First-line/continuation markers show both left and special-indent provenance.
The inspection follows the current reset/drag preview and undo/redo. It creates no
overrides or history. Long source lists show three groups and a remaining count.


## Keyboard ruler sources (alpha.53)

Press Ctrl+Shift+F8 to enter the ruler, Tab to choose a marker, then **F1** to
inspect its values and sources. **Enter, Escape or F1** closes the details and
returns to the same marker. Pending indentation previews remain untouched; apply
or cancel them separately after closing details. Other editing/navigation keys
are ignored while inspecting. Existing focus-loss/pointer-takeover cancellation
still applies. This is a Docs-specific shortcut, not verified Word equivalence.

Ruler source inspection: focus a marker with Ctrl+Shift+F8, then press F1.
Use Up/Down, Page Up/Down or Home/End to read all source groups; Enter, Escape
or F1 returns to the marker without accepting or cancelling an indent preview.

Mouse wheel source inspection (alpha.55): hover the open F1 panel to scroll its
list. The wheel stays isolated from the document while inspecting, including at
list edges. Horizontal/modified wheel input is ignored; click takeover retains
the existing draft-cancellation policy. Keyboard scrolling remains available.

Right-click the horizontal ruler and choose First line sources, Continuation line
sources, Left indent sources or Right indent sources (alpha.56). Enter/Escape/F1
returns to the chosen marker. Right-click cancels any uncommitted ruler preview;
use F1 on its focused marker to inspect that preview without cancelling it.

With a ruler marker focused, press **Shift+F10** to open its context menu
(alpha.57). Use arrows, Tab/Shift+Tab or Home/End; Enter/Space runs the selected
command and Escape returns to the marker. Opening/closing with the keyboard
preserves a pending preview; until it is applied/cancelled, only that marker's
source command is available. The dedicated Menu key is not exposed by the current
GUI key abstraction; use Shift+F10. Windows acceptance remains deferred.

Alpha.58 exposes marker keyboard instructions and uncommitted-preview status to
assistive technology. F1 exposes the full source list through the focused marker's
accessible name; it does not depend on visually scrolling every row. Accessibility
Focus/Click enters marker control without editing. Native speech acceptance is
still open; see [validation](docs/VALIDATION.md#alpha58-ruler-accessibility-validation).

VoiceOver speech/cursor acceptance is postponed to the final accessibility review
by owner request (alpha.59). Existing keyboard/AX checks do not substitute for it.

Home **Increase indent / Decrease indent** changes the selected paragraphs' left
indents by 36 pt within 0–144 pt, preserving right and special indents. One Undo
restores the selection's prior formatting. Each paragraph steps from its own
resolved value. Command search and Home KeyTips II/DI offer the same actions.

## Alpha.61 tab grid

Plain Tab in the document inserts a stored tab, advancing to a left grid stop
every 36 pt. Existing tabs survive save/reopen in native v14. Modified Tab is
reserved; keyboard routes to the ribbon remain available. The engine can store
a custom paragraph interval; a GUI interval editor is not included yet.
Tab paragraphs use fixed spacing: Justify displays left placement, while
Center/Right align the row as a block. Arbitrary ruler stops, leaders and decimal
tabs remain future work. The ribbon, palette and icons are unchanged.

## Alpha.62 tab interval dialog

Open Home → Paragraph spacing → **Tab interval…**, or search commands for
**Tab interval**. Enter 0.5–144 pt in half-point steps, or select **Use default
interval (36 pt)**. OK applies to the selected paragraphs as one Undo/Redo step;
Cancel/Escape changes nothing. Mixed selections are labeled and require an explicit
value or default choice. The paragraph dialog retains its independent draft.
The interval is direct formatting in native v14; custom stops and style inheritance
remain planned. VoiceOver final review and Windows validation remain deferred.

## Alpha.63 explicit left tab positions

The engine and native v15 files support up to 32 explicit left tab positions in
half-point units (0.5–1440 pt). Tabs advance to the next position and resume the
regular interval grid after the final one. The shared command preserves one-step
Undo/Redo and paragraph editing; empty lists restore interval-only behavior.
This increment adds storage and layout; GUI/ruler position editing follows next.
The existing interval dialog preserves explicit positions when changing the grid.
Alpha.62 and older cannot read v15 saves. See [format details](docs/NATIVE_FORMAT.md).

## Alpha.64 tab position editing

Open **Home → Paragraph spacing → Tab positions…**, or search for **Tab positions**.
Enter a position in points and use **Add**; select an existing position, change its
value and use **Update**. **Remove** deletes the selected stop; **Clear all** restores
interval-only layout. The scrollable list supports up to 32 left positions.
**OK** applies the complete draft to the selected paragraphs in one Undo/Redo;
**Cancel/Escape** changes nothing. For mixed lists, explicitly build a replacement
or choose Clear all. Native v15, palette and icons are unchanged.

## Alpha.65 ruler tab markers

Existing explicit left positions appear as L markers on the horizontal ruler.
Drag a marker to preview its position; release applies one Undo/Redo operation.
**Escape** or release outside the ruler cancels. Focus loss, resizing and zoom
changes also cancel; a duplicate destination cannot overwrite another stop.
Selected paragraphs must have identical lists. Mixed lists, overlaps with indent
markers and off-column positions remain editable through **Tab positions**.
The prototype moves existing stops; add/remove and keyboard editing use the dialog.
Native v15, design, palette and icons remain unchanged.
