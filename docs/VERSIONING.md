# Application versioning and release approval

The owner requested this release-numbering policy on 2026-10-16. It is separate
from the feature roadmap: a milestone identifier or a successful build does not
publish a release or approve its version.

## First release cycle

Use Cargo-compatible semantic versions, with positive numeric candidate counters:

| Stage | Version pattern | Meaning |
| :--- | :--- | :--- |
| Alpha | `0.1.0-alpha.1`, `0.1.0-alpha.2`, … | Working development candidates; features and native-platform validation may be incomplete |
| Beta | `0.1.0-beta.1`, `0.1.0-beta.2`, … | Owner-approved feature scope reached; hardening and documented acceptance work remain |
| Release candidate | `0.1.0-rc.1`, `0.1.0-rc.2`, … | Candidate for the first stable release; declared release gates must be reviewed |
| First stable release | `0.1.0` | Requires explicit owner approval and a separate publication instruction |
| Next development cycle | `0.2.0-alpha.1` onward | Starts only after the first stable `0.1.0` has actually been published; next stable is `0.2.0` |

Here the owner's `0.1.0x` denotes the first release family. Alpha, beta and RC
candidates sort before stable `0.1.0`; do not use a fourth numeric Cargo component
or invent `0.1.0x` as a literal executable version. Restart the candidate counter
at 1 when entering an explicitly approved new stage. Future maintenance release
numbers require a separate decision.

## Every change gets a version and changelog entry

The owner's updated instruction on 2026-10-16 requires a version increment and a
dated CHANGELOG.md entry for **every delivered change set**, including code,
design, documentation, configuration and journal-only work. Increment the current
stage's numeric counter by one: `0.1.0-alpha.1` becomes `0.1.0-alpha.2`, then
`0.1.0-alpha.3`. Do not wait for a release or another reminder to apply this rule.

A change set is one coherent work iteration. Edits, fixes found during its
validation, and the separately committed journal recording that work share its
new version. This avoids a recursive version increment for documenting the same
increment. A subsequent change set requires the next number, even if the previous
candidate is still awaiting approval. Read-only reviews with no repository changes
report the existing version. Do not renumber published candidates or silently
move to beta, RC, stable or the next release cycle.

This replaces the earlier policy that exempted documentation-only work and did
not require an increment after each task. Historical entries retain the policy
and version that applied when they were recorded.

## Source of truth and compatibility

`Cargo.toml` `[package].version` is authoritative. The checked-in Cargo.lock root
package must match. Rust's `CARGO_PKG_VERSION` supplies `--version`, capabilities
`app_version` and the GUI Help display. Keep these views consistent; do not hardcode
a second runtime version. Historical captures, validation results, archived
prototypes and journal entries retain their original version context.

The earlier **unreleased** `0.2.0-dev.1` is superseded by `0.1.0-alpha.1` to enter
the requested first cycle. This is application renumbering, not a native-document
migration. At that checkpoint protocol and content versions both remained 1.
Alpha.3 separately introduces native content v2 for size/underline under
[ADR 0003](adr/0003-character-size-and-underline.md), with v1 reading retained.
Alpha.18 introduces native content v3 for paragraph-style identity and direct
overrides under [ADR 0006](adr/0006-built-in-paragraph-styles.md), retaining v1/v2
readers. Protocol `protocol_version: 1` stays unchanged. Neither compatibility version is
inferred from the application version.

## End-of-work report and approval

Every final work report must end with the exact checked-in application version,
including reports that only change documentation or journals. Submit it for the
owner's approval and identify pending approval accurately. Do not treat lack of a
reply, test success, CI success, a PR, or a source commit as approval/publication.
For a read-only check, report the unchanged version. For changes, report the newly
incremented version and link its changelog entry. Record an explicit approval with
its scope in the project journal when received. Attribute owner instructions to
their actual source; only label them GitHub comments when they were posted there.

The first candidate was `0.1.0-alpha.1`; the per-change policy was introduced in
`0.1.0-alpha.2`. The current candidate is defined by Cargo.toml and CHANGELOG.
No alpha/beta/RC/stable
publication is authorized by this policy itself.
Approval of a number does not authorize a merge, tag, upload or GitHub Release.

## Candidate preparation

1. Increment the current stage's counter for the change set; update Cargo.toml
   and the root package in Cargo.lock together. Add a dated, unreleased changelog
   entry describing actual changes. Keep dependency versions unchanged unless
   they are part of the task. Stage transitions need explicit owner approval.
2. Run locked tests, lint and builds required by AGENTS. Verify that `--version`
   and capabilities `app_version` equal Cargo metadata. Keep protocol/content
   version tests unchanged unless a separate format/contract decision requires it.
3. Send the change for review, update the project journal, and report
   the current version with actual checks, limits and approval status.
4. After explicit publication authorization, verify the approved commit, target
   platform evidence, dependency/license gates and artifact version. A future tag
   is `v<application-version>` at that approved commit; never move/reuse a published
   tag or replace artifacts under the same version. No tag or release is created
   automatically by the existing CI workflow.

Alpha.19 introduces native content v4 for document-owned style definitions and
parent graphs. It reads v1/v2/v3/v4 and saves v4; earlier builds reject v4. Process
protocol and preferences remain v1. Version approval does not imply file-format
backward compatibility or authorize a release.

Alpha.20 adds native content v5 for named custom style IDs/definitions. It reads
v1–v5 and saves v5; alpha.19 and earlier reject new saves. The application candidate
is `0.1.0-alpha.24`; process/settings remain v1. Alpha.23 was approved in chat on
2026-10-18; alpha.24 awaits explicit owner approval. Alpha.21 renaming and alpha.22
deletion use existing v5 IDs/references without a content-version change.

Alpha.23 adds native v6 following-style references, retaining strict v1–v5
readers. Process/settings remain v1; alpha.22 and earlier reject v6 saves.

Alpha.24 writes native v7 character-style references and reads v1–v7. Alpha.23 and
earlier reject v7 saves. This is independent of application-version approval.
