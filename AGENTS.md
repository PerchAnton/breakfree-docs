# Working in BreakFree Docs

Read README.md, ROADMAP.md, docs/REQUIREMENTS.md and CONTRIBUTING.md before making changes. The repository is documentation-only. The roadmap proposes a Rust-first architecture for review; language, toolkit, license and native format are not yet adopted.

## Scope

Build a reliable writing and local-persistence loop before advanced document compatibility. Follow D0–D8 and preserve the initial D01–D09 acceptance scenarios. Use the shared development standard in `PerchAnton/breakfree-office/docs/DEVELOPMENT_STANDARD.md` for evidence and reporting.

## Rules

- Preserve BreakFree Docs and its approved app-specific icon at `docs/brand/breakfree-docs.webp`. Do not replace it with the suite icon or a musical symbol.
- Keep public documentation in English and distinguish proposed, implemented and verified work.
- Obtain owner review for architecture, license changes and major dependencies.
- Write original project code and documentation. Do not copy another application's source or branding. Retain all legally required dependency and asset notices.
- Keep the document model, command engine and layout independent of the UI toolkit.
- Do not execute imported macros/scripts or silently upload documents.
- Do not discard unsupported content silently or overwrite a valid file after a failed save.
- Test Unicode, editing, undo/redo, layout and persistence separately.
- Add CLI and optional automation through the same validated command interface, with explicit file and consent boundaries.
- Never invent build commands, platform support, test results, dates or DOCX compatibility percentages.
- Update milestone status only when evidence exists; do not check implementation tasks off for documentation changes.

## Validation

For documentation changes, check links, image references and any changed YAML. No application test command exists yet. Add reproducible build/test instructions with the first implementation. Report actual checks and limitations separately.
