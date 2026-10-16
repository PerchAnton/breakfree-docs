<p align="center">
  <img src="docs/brand/breakfree-docs-transparent.webp" alt="BreakFree Docs — document pages with a rainbow ribbon" width="160" height="160">
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
> **Planning stage — not a working editor yet.** This repository contains original project documentation and approved branding. There are no installers or verified DOCX compatibility results. The roadmap describes proposed work, not shipped functionality.

## Write without getting in your own way

BreakFree Docs is being planned for everyday writing and, over time, demanding document workflows. The aim is a dependable editor for letters, reports and long-form documents, with local files, clear controls and an extensible foundation.

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

All items are planned. Advanced capabilities are separate milestones, not promises for the first release. Full compatibility with every Word feature is not an initial release requirement.

## Built around the document

**Your files stay under your control.** Offline essentials, recoverable saves and explicit handling of unsupported content are design requirements. Imported macros and remote resources must not run or load silently.

**The interface is not the engine.** The proposed architecture separates the document model, editing operations, layout, storage and application shell. Desktop UI, command-line tools and future opt-in automation will use the same validated command interface.

**Text and accessibility come first.** Input methods, international text, keyboard navigation, screen-reader behavior and font fallback must influence architecture selection, not wait until the final polish stage.

**A distinct member of one family.** Docs has its own approved page icon, with the shared aubergine tile and rainbow ribbon expressing freedom and inclusion.

## Roadmap and evidence

The [detailed roadmap](ROADMAP.md) defines architecture proposals, feature areas, milestone dependencies, release gates, compatibility evidence and performance investigations. The [suite development standard](https://github.com/PerchAnton/breakfree-office/blob/main/docs/DEVELOPMENT_STANDARD.md) describes how progress must be reported.

DOCX support will be measured separately for reading, editing and writing. Text preservation, page layout, unsupported elements and font substitution need separate results. Opening one sample is not proof of general compatibility.

## Development status

| Area | Current position |
| :--- | :--- |
| Identity and planning documents | Available |
| Implementation language | Rust-first proposal for review; not adopted yet |
| UI toolkit and native persistence format | Selection and prototypes pending |
| Windows, macOS and Linux | Intended evaluation targets; no supported builds |
| Application code, test results and releases | Not available |
| License | Owner decision pending |

There is nothing to install yet. Reproducible build instructions will be published with a working implementation. Docs remains the first development priority within the suite.

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
