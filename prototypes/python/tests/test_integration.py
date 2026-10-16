import contextlib
import io
import json
import os
import subprocess
import sys
import tempfile
import unittest
from importlib.metadata import PackageNotFoundError, distribution
from pathlib import Path

from breakfree_docs import __version__
from breakfree_docs.integration import arguments, main


class IntegrationTests(unittest.TestCase):
    def test_capabilities_without_gui_import_or_neighbor_repository(self):
        script = """
import importlib.abc, sys
class NoGUI(importlib.abc.MetaPathFinder):
    def find_spec(self, fullname, path=None, target=None):
        if fullname.startswith(('PySide6', 'breakfree_office', 'tkinter')):
            raise RuntimeError('GUI or Office import on a headless path')
sys.meta_path.insert(0, NoGUI())
from breakfree_docs.integration import main
raise SystemExit(main(['--capabilities']))
"""
        with tempfile.TemporaryDirectory() as folder:
            environment = dict(os.environ)
            environment.pop("PYTHONPATH", None)
            result = subprocess.run(
                [sys.executable, "-c", script],
                cwd=folder,
                env=environment,
                capture_output=True,
                text=True,
                check=False,
            )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr, "")
        value = json.loads(result.stdout)
        try:
            distribution("PySide6-Essentials")
            expected_actions = ["new", "open"]
        except PackageNotFoundError:
            expected_actions = []
        self.assertEqual(
            value,
            {
                "protocol_version": 1,
                "app_id": "breakfree.docs",
                "app_version": __version__,
                "display_name": "BreakFree Docs",
                "capabilities": expected_actions,
                "formats": [
                    {
                        "id": "breakfree.docs.native",
                        "extensions": ["bfdoc"],
                        "read": True,
                        "write": True,
                    }
                ],
            },
        )

    def test_arguments_and_version(self):
        self.assertTrue(arguments(["--new"]).new)
        absolute = str(Path.cwd() / "test.bfdoc")
        self.assertEqual(arguments(["--open", absolute]).open, Path(absolute))
        for args in (
            ["--open", "relative.bfdoc"],
            ["--new", "--open", absolute],
            ["--unknown"],
            ["--open"],
            ["--version", "--new"],
            ["stray"],
        ):
            with self.subTest(args=args), contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as raised:
                    arguments(args)
                self.assertEqual(raised.exception.code, 2)
        stream = io.StringIO()
        with contextlib.redirect_stdout(stream):
            self.assertEqual(main(["--version"]), 0)
        self.assertEqual(stream.getvalue(), f"BreakFree Docs {__version__}\n")

    def test_missing_and_corrupt_open_exit_before_gui(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "bad.bfdoc"
            for exists in (False, True):
                if exists:
                    path.write_bytes(b"broken")
                stream = io.StringIO()
                with contextlib.redirect_stderr(stream):
                    self.assertEqual(main(["--open", str(path)]), 1)
                self.assertIn("BreakFree Docs", stream.getvalue())
                if exists:
                    self.assertEqual(path.read_bytes(), b"broken")


if __name__ == "__main__":
    unittest.main()
