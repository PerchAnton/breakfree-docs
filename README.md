<p align="center">
  <a href="https://github.com/PerchAnton/breakfree-office"><img src="https://raw.githubusercontent.com/PerchAnton/breakfree-office/main/docs/brand/breakfree-icon.webp" alt="BreakFree Office" width="160" height="160"></a>
</p>

<h1 align="center">BreakFree Docs</h1>

<p align="center">
  <strong>Your words. Your freedom.</strong><br>
  An independent document editor in the making.<br>
  A place for letters, reports and ideas — part of BreakFree Office.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-planning-8055C8?style=flat-square" alt="Status: planning">
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
> **Planning stage — not a working editor yet.** This repository contains project documentation and the development plan. There are no installers, build commands or verified DOCX compatibility claims. Everything described below is proposed unless explicitly marked as completed.

## Write without getting in your own way

BreakFree Docs aims to make everyday writing clear and dependable: start a document, give it structure, save it locally and return to it with confidence.

We are beginning with a small editor that works reliably, not a promise to reproduce every feature of Microsoft Word at once.

## The first useful workflow

> **Create → Write → Format → Save → Reopen → Continue.**

The first milestone will focus on plain text, paragraphs, basic formatting, undo/redo and safe local persistence. Saving and reopening must preserve all supported content. The exact native file format and implementation stack still need to be chosen.

| Proposed first | Investigate next | Outside the initial scope |
| :--- | :--- | :--- |
| Text and paragraph editing | Styles, lists and tables | Full Word feature parity |
| Basic emphasis and alignment | Images and pagination | VBA and macro execution |
| Undo, redo and unsaved-change prompts | Print and PDF export | Cloud collaboration |
| Local save and reopen | A documented DOCX subset | Enterprise administration |

See the [acceptance criteria](docs/REQUIREMENTS.md) and [roadmap](ROADMAP.md) for the proposed sequence.

## Design direction

**Keep the document in the user's hands.** Prioritize local files and useful offline work. Saving should be explicit and recoverable; unsupported content should never disappear silently.

**Make the basics accessible.** Keyboard operation, readable controls and international text belong in the foundation, not in a final cosmetic pass.

**Share a family resemblance.** Docs will use the BreakFree identity while remaining a separate application. The approved office-document icon and its restrained rainbow accent express freedom and inclusion.

These are implementation goals, not completed features.

## File compatibility

DOCX import and export are research and development goals. Compatibility will need fixture-based tests for text, formatting, tables, images and layout. Supporting one sample is not proof that an arbitrary Word document will round-trip correctly.

A future compatibility matrix must list supported features, limitations and behavior for unsupported elements. No macro support is planned for the first milestone.

## Development status

| Area | Current position |
| :--- | :--- |
| Product direction | Documented |
| Implementation language and UI toolkit | Not selected |
| Native persistence format | Not selected |
| Windows, macOS and Linux | Candidate targets; no supported builds yet |
| Application code and automated tests | Not present yet |
| Software license | Owner decision pending |

There is nothing to install yet. Build instructions will be added when they can be run and verified. This is the first development priority within the suite.

## The BreakFree family

| Project | Purpose |
| :--- | :--- |
| [**BreakFree Office**](https://github.com/PerchAnton/breakfree-office) | Suite hub, identity and shared direction |
| **BreakFree Docs** | Documents — you are here |
| [**BreakFree Sheets**](https://github.com/PerchAnton/breakfree-sheets) | Spreadsheets and calculations |
| [**BreakFree Slides**](https://github.com/PerchAnton/breakfree-slides) | Presentations and visual storytelling |

## Get involved

Suggest a writing workflow, discuss accessibility or help evaluate the first technical foundation in [Issues](https://github.com/PerchAnton/breakfree-docs/issues). Use synthetic examples rather than confidential files. Read [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md) before preparing changes.

Everyone is welcome, including LGBTQ+ contributors and users. Follow the [shared Code of Conduct](https://github.com/PerchAnton/breakfree-office/blob/main/CODE_OF_CONDUCT.md).

## License and independence

No software license has been selected; public visibility alone is not a reuse license. Licensing will be resolved before substantive third-party implementation contributions are accepted.

BreakFree Docs is independent and is not affiliated with Microsoft. Microsoft Word and DOCX references describe intended workflows and interoperability goals, not endorsement or a completed replacement.

---

<p align="center"><strong>BreakFree Office · Your work. Your freedom.</strong><br><sub>Created by <a href="https://github.com/PerchAnton">Anton Perchenko</a></sub></p>
