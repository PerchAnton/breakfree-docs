# Dependency notice review — alpha.95

Scope: locked Cargo resolution for aarch64-apple-darwin, 2026-10-24.
This is a technical evidence review, not project-license adoption or legal
clearance to distribute. Windows/Linux transitive review remains separate.
The inventory conservatively includes build/dev and potentially unused resolved
packages, so 184 is not a claim about exact binary linkage.

## Findings

The alpha.94 archive flagged 32 packages with no locally packaged notice files.
Inspection of their Cargo .cargo_vcs_info.json identified exact upstream revisions.
Official license files from those revisions are now stored unchanged under
[licenses/upstream](licenses/upstream/index.json), with package/version, SPDX
declaration, repository, source commit, source URL and SHA-256 for each file.

| Result | Packages | Meaning |
| :--- | ---: | :--- |
| Full upstream license text recovered | 22 | Missing-text gap addressed; not overall redistribution approval |
| Upstream statement only | 8 | Modern objc2-family LICENSE.md links to terms and describes Apple SDK concerns; full package-specific notice review remains open |
| No license text found in inspected tree | 2 | dispatch 0.2.0 (MIT declaration), hexf-parse 0.2.1 (CC0-1.0 declaration); manifests alone do not supply full notices |

The eight statement-only packages are block2 0.6.2, dispatch2 0.3.1, objc2 0.6.5,
objc2-app-kit 0.3.2, objc2-core-foundation 0.3.2, objc2-core-graphics 0.3.2,
objc2-encode 4.1.0 and objc2-io-surface 0.3.2. Do not replace their evidence with
an unrelated version's MIT text or assume their SDK-derived provenance is settled.
Older objc2-family full-license files also do not settle SDK distribution questions.

The two no-text cases were checked in the complete pinned repository trees and
README files. No maintainer request was sent. Resolve attribution/full terms with
additional upstream evidence or an explicitly authorized clarification before
distribution; do not infer permission from a missing file.

## Packaging and repeatable checks

The package tool uses only the local supplement: no current upstream main or
network download during notice collection. It verifies exact crate source SHA,
SPDX declaration and every notice checksum. A changed/missing file, stale source
revision, changed declaration or duplicate entry stops the archive build.
Existing embedded-font notices remain included and hash-checked for the font
package. Host fonts are never redistributed.

Run: python3 tools/test_license_inventory.py, then the
[review-package procedure](REVIEW_PACKAGE.md). DEPENDENCIES.json distinguishes
missing notice files, supplemental evidence and unresolved notice review.
A file named LICENSE.md is not automatically treated as full license text.
The archive manifest continues to say distribution review is pending.

## Remaining gate

Review the ten incomplete notice entries, the SDK provenance caveat, retained
copyright/attribution requirements for the whole distribution and the exact
release target/features. Confirm project/brand license, signing and distribution
policy with the owner separately. No dependency version or project license was
changed, and no public binary release is produced by this work.
