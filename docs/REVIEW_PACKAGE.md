# Local review package

This script prepares a local unsigned review archive. It does not upload, tag,
release, install, adopt a project license or approve redistribution.
Run from the checkout with the pinned Rust toolchain and Python 3:

macOS/Linux:
```sh
python3 tools/package_review.py --output /tmp/breakfree-docs-review.zip
```

Windows PowerShell (native acceptance remains deferred):
```powershell
py -3 tools/package_review.py --output "$env:TEMP\breakfree-docs-review.zip"
```

Choose a new output path each time. The script builds the host release binary,
uses locked/offline Cargo metadata after that build, collects target dependency
license/notice files, retains embedded-font notices, and writes source SHA, dirty
state, compiler/target/version and per-file SHA-256 hashes. It never bundles host
font files. The archive has the executable at its root; extract into a writable
folder and run it there with `--new`, `--open <absolute_path>`, `--version` or
`--capabilities`. No Office or sibling checkout is needed.

Review DEPENDENCIES.json entries without notice files manually. Inclusion of
upstream files is evidence collection, not a legal completeness guarantee.
A release still requires license review, clean-source provenance, owner approval,
platform acceptance, signing/distribution decisions and external DOCX save-back.
Repeated commands reproduce the build procedure; byte-identical ZIPs/builds are
not promised. CI artifacts and local archives are distinct from public releases.


Alpha.95 uses a pinned local notice supplement and validates source revisions,
SPDX declarations and SHA-256 before packaging. See [license review](LICENSE_REVIEW.md).
The result reports both missing notice files and unresolved notice review;
statement-only LICENSE.md files do not resolve a full-text gap.
