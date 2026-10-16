# Working in BreakFree Docs

Read README.md, ROADMAP.md, docs/REQUIREMENTS.md and CONTRIBUTING.md before making changes. This repository is currently documentation-only; a language, UI toolkit, license and native format have not been selected.

## Scope

Build a reliable writing and local-persistence loop before advanced document-format compatibility. Keep the application independent; suite-wide branding lives in `PerchAnton/breakfree-office`.

## Rules

- Preserve the approved BreakFree name and shared office-document icon.
- Keep public documentation in English and distinguish planned work from working features.
- Discuss architecture, license changes and major dependencies with the owner.
- Do not copy proprietary code or add unlicensed fonts or document fixtures.
- Treat imported files as untrusted. Do not execute macros/scripts or silently upload documents.
- Do not silently discard unsupported content or overwrite a good file on a failed save.
- Test supported Unicode, editing, undo/redo and save/reopen behavior as implementation grows.
- Never invent build commands, platform support, passing tests or DOCX compatibility.
- Update status documentation when a feature is actually implemented and verified.

## Validation

For now, check Markdown links, image paths and YAML syntax. There is no application test command yet. Report exactly what was checked and what was not run. Add reproducible build/test instructions with the first implementation.
