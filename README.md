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
> **Working prototype, pending architecture review.** Create, edit, format, undo,
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

**The interface is not the engine.** The immutable document model, commands,
undo/redo, persistence, text projection, Qt layout and GUI are separate modules.
The GUI and tests use the same edit commands. The process adapter supplies the
optional Office launch interface. [ADR 0001](docs/adr/0001-reversible-desktop-prototype.md)
records Python/Qt as a reversible experiment, not a final architecture decision.

**Text and accessibility come first.** Input methods, international text, keyboard navigation, screen-reader behavior and font fallback must influence architecture selection, not wait until the final polish stage.

**A distinct member of one family.** Docs has its own approved page icon, with the shared aubergine tile and rainbow ribbon expressing freedom and inclusion.

## Roadmap and evidence

The [detailed roadmap](ROADMAP.md) defines architecture proposals, feature areas, milestone dependencies, release gates, compatibility evidence and performance investigations. The [suite development standard](https://github.com/PerchAnton/breakfree-office/blob/main/docs/DEVELOPMENT_STANDARD.md) describes how progress must be reported.

DOCX support will be measured separately for reading, editing and writing. Text preservation, page layout, unsupported elements and font substitution need separate results. Opening one sample is not proof of general compatibility.

## Run the prototype

Use Python 3.11–3.14 with pip. The evaluated host uses Python 3.14.3 and Qt 6.11.1
on macOS arm64. Windows and Linux instructions are provided for evaluation; see
[validation](docs/VALIDATION.md) for results actually available.

macOS / Linux, from this repository:

```sh
python3 -m venv .venv
.venv/bin/python -m pip install -e ".[desktop,dev]"
.venv/bin/breakfree-docs --new
.venv/bin/breakfree-docs --open "/absolute/path/My document.bfdoc"
.venv/bin/breakfree-docs --capabilities
.venv/bin/breakfree-docs --version
```

Windows PowerShell (Python 3.12 example, no activation-policy change needed):

```powershell
py -3.12 -m venv .venv
.\.venv\Scripts\python.exe -m pip install -e ".[desktop,dev]"
.\.venv\Scripts\breakfree-docs.exe --new
.\.venv\Scripts\breakfree-docs.exe --open "C:\Users\YourName\Documents\My document.bfdoc"
.\.venv\Scripts\breakfree-docs.exe --capabilities
.\.venv\Scripts\breakfree-docs.exe --version
```

Open [examples/Welcome.bfdoc](examples/Welcome.bfdoc) through the File menu for a
synthetic formatted Unicode example. `python -m breakfree_docs` is an equivalent
launcher after installation. Qt is an
optional desktop extra: `pip install .` installs the model/CLI without Qt, and
`--capabilities` / `--version` still work. A core-only installation advertises no
desktop launch actions and reports a missing-dependency error for `--new`/`--open`.
Application and dependency installation
may need package-index access; the installed editor makes no network requests.

Use the File menu or keyboard shortcuts for New, Open, Save and Save As. Select
text and use Bold/Italic; at a collapsed caret they set the next insertion style.
Use Left/Center/Right/Justify for paragraphs. Ctrl (Command on macOS) + Z undoes
one committed key/input operation; paste and formatting are separate operations.
Ctrl/Command + Shift + L/E/R/J aligns paragraphs. Clipboard paste is plain text.
Tab moves focus; use menus to reach all commands. Unsaved changes have Save,
Discard and Cancel choices. Only `.bfdoc` is supported; DOCX, PDF and printing
are intentionally absent.

## Build and check

```sh
.venv/bin/python -m unittest discover -s tests -v
.venv/bin/python -m ruff check src tests tools
.venv/bin/python -m ruff format --check src tests tools
.venv/bin/python tools/check_repository.py
.venv/bin/python -m build
.venv/bin/python tools/check_wheel.py
```

For a headless host, set `QT_QPA_PLATFORM=offscreen` before tests. PowerShell:
`$env:QT_QPA_PLATFORM = "offscreen"`; then replace `.venv/bin/python` above with
`.\.venv\Scripts\python.exe`. Offscreen tests do not prove native platform input
or accessibility. Minimal Debian/Ubuntu hosts can also need Qt runtime libraries:
`sudo apt-get install libegl1 libopengl0`. Native X11 desktop prerequisites vary;
see the [Qt Linux requirements](https://doc.qt.io/qt-6/linux-requirements.html).
Installed but broken Qt imports fail tests instead of silently skipping the GUI.
The wheel check installs a fresh environment outside the source
tree, with no Qt or Office, and needs package-index access for pinned regex.

Build output is `dist/breakfree_docs-0.1.0.dev1-py3-none-any.whl` plus a source
archive. Install that wheel with the desktop extra on a target machine:

```sh
python -m pip install "./dist/breakfree_docs-0.1.0.dev1-py3-none-any.whl[desktop]"
breakfree-docs --new
```

This is a Python distribution with a generated executable launcher, not a bundled
runtime or OS installer. The wheel includes the approved Docs icon. Build/test
steps consume only this repository and pinned package dependencies. No release or
automatic merge is configured.

## Development status

| Area | Current position |
| :--- | :--- |
| Identity and planning documents | Available |
| Implementation language | Python prototype; Rust-first long-term proposal remains open |
| UI toolkit and native persistence format | Qt 6 experiment; strict native `.bfdoc` content v1 |
| Windows, macOS and Linux | macOS local evaluation; other platform verification tracked separately |
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
