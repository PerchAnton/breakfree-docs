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
packaged = ROOT / "src/breakfree_docs/assets/breakfree-docs.webp"
assert (
    hashlib.sha256(source.read_bytes()).digest() == hashlib.sha256(packaged.read_bytes()).digest()
)
snapshot = json.loads((ROOT / "contracts/v1/snapshot.json").read_text(encoding="utf-8"))
assert snapshot["protocol_version"] == 1
assert snapshot["app_id"] == "breakfree.docs"
assert snapshot["office_source_commit"] == "cfaaf419e84d9981acd421d130113ca2265061fd"
if errors:
    raise SystemExit("\n".join(errors))
print("Local documentation references, contract snapshot and approved artwork: OK")
