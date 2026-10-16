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
at 1 when entering a new stage. Increment it for a distinct candidate, without
renumbering existing published candidates. Documentation-only updates do not
require a bump. Future maintenance release numbers require a separate decision.

## Source of truth and compatibility

`Cargo.toml` `[package].version` is authoritative. The checked-in Cargo.lock root
package must match. Rust's `CARGO_PKG_VERSION` supplies `--version`, capabilities
`app_version` and the GUI Help display. Keep these views consistent; do not hardcode
a second runtime version. Historical captures, validation results, archived
prototypes and journal entries retain their original version context.

The earlier **unreleased** `0.2.0-dev.1` is superseded by `0.1.0-alpha.1` to enter
the requested first cycle. This is application renumbering, not a native-document
migration. Protocol `protocol_version: 1` and `.bfdoc` `content_version: 1` stay
unchanged. Neither is inferred from the application version.

## End-of-work report and approval

Every final work report must end with the exact checked-in application version,
including reports that only change documentation or journals. Submit it for the
owner's approval and identify pending approval accurately. Do not treat lack of a
reply, test success, CI success, a PR, or a source commit as approval/publication.
If the version is unchanged, report that same version; no automatic task-by-task
increment is required. Record an explicit approval with its scope in the project
journal when received. Do not claim a private chat instruction is a GitHub comment.

The first candidate under this policy is **`0.1.0-alpha.1`, submitted for owner
approval**. No alpha/beta/RC/stable publication is authorized by this policy itself.
Approval of a number does not authorize a merge, tag, upload or GitHub Release.

## Candidate preparation

1. Choose the stage/counter under the current owner-approved cycle; update Cargo
   metadata and add a dated, unreleased changelog entry describing actual changes.
2. Run locked tests, lint and builds required by AGENTS. Verify that `--version`
   and capabilities `app_version` equal Cargo metadata. Keep protocol/content
   version tests unchanged unless a separate format/contract decision requires it.
3. Send the change for review, update both complete project journals, and report
   the current version with actual checks, limits and approval status.
4. After explicit publication authorization, verify the approved commit, target
   platform evidence, dependency/license gates and artifact version. A future tag
   is `v<application-version>` at that approved commit; never move/reuse a published
   tag or replace artifacts under the same version. No tag or release is created
   automatically by the existing CI workflow.
