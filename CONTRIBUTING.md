# Contributing to BreakFree Docs

Read the [shared contribution guide](https://github.com/PerchAnton/breakfree-office/blob/main/CONTRIBUTING.md) and [Code of Conduct](https://github.com/PerchAnton/breakfree-office/blob/main/CODE_OF_CONDUCT.md).

Use this repository for writing workflows, the document model, editing and persistence. Suite-wide identity and shared policy belong in [BreakFree Office](https://github.com/PerchAnton/breakfree-office).

Start with [ROADMAP.md](ROADMAP.md) and [the first milestone](docs/REQUIREMENTS.md). Discuss major architecture or dependencies in an issue before implementing them. Licensing must be agreed with the owner before substantive third-party code is accepted.

The owner-requested first editor is implemented as a reversible Python/Qt prototype
in [ADR 0001](docs/adr/0001-reversible-desktop-prototype.md). Architecture and major
dependency approval remain review gates; the prototype does not adopt a final stack
or a project license. No substantive third-party implementation source is vendored.

Keep pull requests focused. Explain the user outcome and checks actually run. Use
synthetic documents, never customer data, private correspondence or unlicensed assets.
Run the commands in [README.md](README.md), add meaningful engine/persistence tests
for behavior changes and keep GUI edits routed through the same model commands.
For pure-model work the desktop extra is optional. CI's offscreen Qt tests are not
evidence of native IME or assistive-technology support.

Keep project history in [docs/PROJECT_LOG.md](docs/PROJECT_LOG.md), using Europe/Kyiv
timestamps and direct event links. Read the current log and instructions before
editing, safely fetch updates and preserve other authors' work. Commit log updates
separately on the review branch in this repository. Never force push or publish
public replies/assignments without separate owner authorization. After the first
audit, check events since the last recorded audit, including replies to old issues
and PRs. Record inaccessible sources rather than claiming they contain no events.
