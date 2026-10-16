"""Install the built wheel in a fresh environment outside the checkout and smoke test it.

Requires package-index access for the pinned regex dependency. No Qt, Office,
neighbor repository, source-tree import or editable install is used in this check.
"""

import json
import os
import subprocess
import tempfile
import venv
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
wheels = list((ROOT / "dist").glob("breakfree_docs-*.whl"))
if len(wheels) != 1:
    raise SystemExit("Build exactly one wheel in dist first (python -m build).")

with tempfile.TemporaryDirectory(prefix="breakfree-docs-wheel-") as directory:
    folder = Path(directory)
    environment = folder / "environment"
    venv.EnvBuilder(with_pip=True).create(environment)
    binaries = environment / ("Scripts" if os.name == "nt" else "bin")
    python = binaries / ("python.exe" if os.name == "nt" else "python")
    command = binaries / ("breakfree-docs.exe" if os.name == "nt" else "breakfree-docs")
    subprocess.run([str(python), "-m", "pip", "install", str(wheels[0])], cwd=folder, check=True)
    result = subprocess.run(
        [str(command), "--capabilities"], cwd=folder, capture_output=True, text=True, check=True
    )
    assert result.stderr == "", result.stderr
    capabilities = json.loads(result.stdout)
    assert capabilities["app_id"] == "breakfree.docs"
    assert capabilities["capabilities"] == [], "A core-only install cannot launch a desktop."
    missing_desktop = subprocess.run(
        [str(command), "--new"], cwd=folder, capture_output=True, text=True, check=False
    )
    assert missing_desktop.returncode == 1
    assert "Desktop dependencies missing" in missing_desktop.stderr
    script = """
from importlib.metadata import version
from importlib.resources import files
from pathlib import Path
from breakfree_docs.commands import Editor
from breakfree_docs.storage import save, load
import breakfree_docs, sys
assert 'PySide6' not in sys.modules
assert Path(breakfree_docs.__file__).is_relative_to(Path(sys.prefix))
assert files('breakfree_docs').joinpath('assets/breakfree-docs.webp').is_file()
editor = Editor()
editor.replace_text(0, 0, 'English\\nУкраїнська\\nРусский e\\u0301 🙂')
editor.emphasize(0, 7, 'bold', True)
path = Path.cwd() / 'independent.bfdoc'
save(editor.document, path)
assert load(path) == editor.document
print('Wheel, artwork, native roundtrip and no GUI: OK', version('breakfree-docs'))
"""
    subprocess.run([str(python), "-I", "-c", script], cwd=folder, check=True)
print("Standalone executable outside the source checkout: OK")
