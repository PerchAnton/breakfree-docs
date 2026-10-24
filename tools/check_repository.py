"""Check repository-local documentation references and packaged branding."""

import hashlib
import json
import re
from pathlib import Path
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[1]
errors = []
for path in [
    ROOT / "README.md",
    ROOT / "ROADMAP.md",
    ROOT / "AGENTS.md",
    ROOT / "CONTRIBUTING.md",
    *ROOT.glob("docs/**/*.md"),
    *ROOT.glob("contracts/**/*.md"),
]:
    content = path.read_text(encoding="utf-8")
    references = re.findall(r"\[[^\]]*\]\(([^\s)]+)", content)
    references += re.findall(r'<img[^>]+src="([^"]+)"', content)
    for reference in references:
        if reference.startswith(("http:", "https:", "#", "mailto:")):
            continue
        target = unquote(reference.split("#")[0])
        if target and not (path.parent / target).exists():
            errors.append(f"{path.relative_to(ROOT)}: missing {reference}")

source = ROOT / "docs/brand/breakfree-docs.webp"
packaged = ROOT / "prototypes/python/src/breakfree_docs/assets/breakfree-docs.webp"
assert (
    hashlib.sha256(source.read_bytes()).digest() == hashlib.sha256(packaged.read_bytes()).digest()
)
transparent = ROOT / "docs/brand/breakfree-docs-transparent.webp"
assert transparent.read_bytes() == source.read_bytes()
embedded = re.findall(
    r'include_bytes!\(\s*"([^"]+)"\s*\)',
    (ROOT / "src/gui.rs").read_text(encoding="utf-8"),
)
assert "../docs/brand/breakfree-docs-transparent.webp" in embedded
snapshot = json.loads((ROOT / "contracts/v1/snapshot.json").read_text(encoding="utf-8"))
assert snapshot["protocol_version"] == 1
assert snapshot["app_id"] == "breakfree.docs"
assert snapshot["office_source_commit"] == "cfaaf419e84d9981acd421d130113ca2265061fd"
# Supplemental third-party evidence must match its checked-in bytes.
notice_index = json.loads((ROOT / "docs/licenses/upstream/index.json").read_text())
seen_packages = set()
for package in notice_index["packages"]:
    identity = (package["name"], package["version"])
    assert identity not in seen_packages, "Duplicate supplemental notice entry"
    seen_packages.add(identity)
    assert re.fullmatch(r"[0-9a-f]{40}", package["vcs"]["git"]["sha1"])
    for notice in package["files"]:
        path = (ROOT / notice["path"]).resolve()
        assert path.is_relative_to((ROOT / "docs/licenses").resolve())
        assert hashlib.sha256(path.read_bytes()).hexdigest() == notice["sha256"]
corpus = ROOT / "tests/fixtures/docx-corpus"
manifest = json.loads((corpus / "manifest.json").read_text())
for case in manifest["cases"]:
    path = (corpus / case["file"]).resolve()
    assert path.is_relative_to(corpus.resolve())
    assert hashlib.sha256(path.read_bytes()).hexdigest() == case["sha256"]
    if "source" in case:
        assert hashlib.sha256((ROOT / case["source"]).read_bytes()).hexdigest() == case["source_sha256"]
report = json.loads((corpus / "saveback-report.json").read_text())
for name, digest in report["files"].items():
    assert hashlib.sha256((corpus / name).read_bytes()).hexdigest() == digest
if errors:
    raise SystemExit("\n".join(errors))
print("Local documentation references, contract snapshot, approved artwork, notice and corpus hashes: OK")
