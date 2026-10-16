# Contributing to BreakFree Docs

Read the [shared contribution guide](https://github.com/PerchAnton/breakfree-office/blob/main/CONTRIBUTING.md) and [Code of Conduct](https://github.com/PerchAnton/breakfree-office/blob/main/CODE_OF_CONDUCT.md).

Use this repository for writing workflows, the document model, editing and persistence. Suite-wide identity and shared policy belong in [BreakFree Office](https://github.com/PerchAnton/breakfree-office).

Start with [ROADMAP.md](ROADMAP.md) and [the first milestone](docs/REQUIREMENTS.md). Discuss major architecture or dependencies in an issue before implementing them. Licensing must be agreed with the owner before substantive third-party code is accepted.

The owner has approved Rust for every runtime layer and Word 2024 for Windows as
the visual reference. Do not ask to approve the language again. Follow
[ADR 0002](docs/adr/0002-rust-native-ribbon.md) and [UI_SPEC](docs/UI_SPEC.md).
GUI toolkit, dependency licensing and native-platform reviews remain open; no
project license is adopted. No third-party implementation source is vendored.

Keep pull requests focused. Explain the user outcome and checks actually run. Use
synthetic documents, never customer data or private correspondence. Run the Cargo
checks in [README](README.md); keep GUI edits routed through shared model commands.
The core builds with `--no-default-features`. CPU egui event tests are distinct from
native input, dialogs, accessibility and Windows scaling verification.

Follow [versioning and owner approval](docs/VERSIONING.md). Every work report,
including documentation-only work, ends with the current application version for
owner approval. Do not bump automatically, infer approval or publish a tag/release
without a separate explicit instruction. Keep candidate scope in CHANGELOG.md.

Keep project history in [docs/PROJECT_LOG.md](docs/PROJECT_LOG.md), using Europe/Kyiv
timestamps and direct event links. Read the current log and instructions before
editing, safely fetch updates and preserve other authors' work. Commit log updates
separately on the review branch in this repository. After every significant stage
and journal update, also publish its full text-identical mirror to
`PerchAnton/gitwork`, branch `main`, at
`breakfree/breakfree-docs/PROJECT_LOG.md`, with separate `SYNC.md` provenance.
Follow the two-copy workflow in [AGENTS.md](AGENTS.md); one copy alone is incomplete.
Never force push or publish
public replies/assignments without separate owner authorization. After the first
audit, check events since the last recorded audit, including replies to old issues
and PRs. Record inaccessible sources rather than claiming they contain no events.
