"""Build a local review archive with provenance and dependency notices; never publish."""
import argparse
import hashlib
import json
import platform
import shutil
import subprocess
import tempfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def run(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if output.exists():
        raise SystemExit("Output already exists; choose a new path.")
    host = next(x.split(": ", 1)[1] for x in run("rustc", "-vV").splitlines() if x.startswith("host:"))
    subprocess.run(["cargo", "build", "--locked", "--release"], cwd=ROOT, check=True)
    metadata = json.loads(run("cargo", "metadata", "--locked", "--offline",
                              "--format-version", "1", "--filter-platform", host))
    active = {n["id"] for n in metadata["resolve"]["nodes"]}
    binary = ROOT / "target/release" / ("breakfree-docs.exe" if platform.system() == "Windows" else "breakfree-docs")
    version = subprocess.check_output([str(binary), "--version"], text=True).strip()
    with tempfile.TemporaryDirectory(prefix="breakfree-review-") as temp:
        stage = Path(temp) / "contents"
        stage.mkdir()
        shutil.copy2(binary, stage / binary.name)
        for name in ["README.md", "CHANGELOG.md", "Cargo.lock", "docs/DOCX_SUBSET.md",
                     "docs/VALIDATION.md", "docs/THIRD_PARTY_NOTICES.md", "docs/REVIEW_PACKAGE.md"]:
            dst = stage / name
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / name, dst)
        shutil.copytree(ROOT / "docs/licenses", stage / "docs/licenses")
        inventory = []
        for package in sorted(metadata["packages"], key=lambda p: (p["name"], p["version"])):
            if package["id"] not in active or package["name"] == "breakfree-docs":
                continue
            source = Path(package["manifest_path"]).parent
            files = sorted({p for p in source.rglob("*") if p.is_file()
                            and p.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE"))
                            and p.stat().st_size < 2_000_000})
            copied = []
            for file in files:
                relative = Path("third-party") / (package["name"] + "-" + package["version"]) / file.relative_to(source)
                dst = stage / relative
                dst.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(file, dst)
                copied.append(relative.as_posix())
            inventory.append({key: package.get(key) for key in ("name", "version", "license", "repository")} |
                             {"notice_files": copied, "review_required": not files})
        (stage / "DEPENDENCIES.json").write_text(json.dumps(inventory, indent=2) + "\n")
        files = {p.relative_to(stage).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                 for p in sorted(stage.rglob("*")) if p.is_file()}
        manifest = {
            "purpose": "Local review only; unsigned, not a release or redistribution approval.",
            "version": version, "source_commit": run("git", "rev-parse", "HEAD"),
            "dirty": bool(run("git", "status", "--porcelain", "--untracked-files=normal")),
            "target": host, "rustc": run("rustc", "--version"),
            "license_review": "Pending owner review; inspect DEPENDENCIES.json and notices.",
            "files_sha256": files,
        }
        (stage / "MANIFEST.json").write_text(json.dumps(manifest, indent=2) + "\n")
        output.parent.mkdir(parents=True, exist_ok=True)
        built = Path(temp) / "review.zip"
        with zipfile.ZipFile(built, "w", compression=zipfile.ZIP_DEFLATED, strict_timestamps=False) as archive:
            for p in sorted(stage.rglob("*")):
                if p.is_file():
                    archive.write(p, p.relative_to(stage))
        with output.open("xb") as destination, built.open("rb") as source:
            shutil.copyfileobj(source, destination)
    print(json.dumps({"archive": str(output), "sha256": hashlib.sha256(output.read_bytes()).hexdigest(),
                      "dependency_count": len(inventory),
                      "missing_notice_files": sum(p["review_required"] for p in inventory)}))


if __name__ == "__main__":
    main()
