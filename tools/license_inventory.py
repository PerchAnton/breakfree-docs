"""Validate pinned local supplemental notices. No network is used during packaging."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def supplement(package, source):
    index = json.loads((ROOT / "docs/licenses/upstream/index.json").read_text())
    matches = [p for p in index["packages"]
               if (p["name"], p["version"]) == (package["name"], package["version"])]
    if not matches:
        return None
    if len(matches) != 1:
        raise ValueError("Duplicate supplemental license entry")
    record = matches[0]
    if record["license"] != package["license"]:
        raise ValueError("License declaration changed; review supplement")
    vcs = json.loads((source / ".cargo_vcs_info.json").read_text())
    if vcs.get("git", {}).get("sha1") != record["vcs"]["git"]["sha1"]:
        raise ValueError("Crate source revision differs from pinned license evidence")
    for notice in record["files"]:
        path = (ROOT / notice["path"]).resolve()
        if not path.is_relative_to((ROOT / "docs/licenses").resolve()):
            raise ValueError("Notice escapes the local license directory")
        if hashlib.sha256(path.read_bytes()).hexdigest() != notice["sha256"]:
            raise ValueError("Supplemental notice checksum mismatch: " + notice["path"])
    return record
