"""Regression checks for stale, altered or duplicate notice evidence."""
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
import license_inventory


class NoticeTests(unittest.TestCase):
    def test_pinned_source_and_bytes_are_required(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            notices = root / "docs/licenses/upstream"
            notices.mkdir(parents=True)
            notice = notices / "LICENSE"
            notice.write_text("Test notice")
            import hashlib
            record = {"name": "test", "version": "1", "license": "MIT",
                      "vcs": {"git": {"sha1": "a" * 40}},
                      "files": [{"path": "docs/licenses/upstream/LICENSE",
                                 "sha256": hashlib.sha256(notice.read_bytes()).hexdigest()}]}
            index = notices / "index.json"
            index.write_text(json.dumps({"packages": [record]}))
            source = root / "crate"
            source.mkdir()
            (source / ".cargo_vcs_info.json").write_text(json.dumps(record["vcs"]))
            package = {"name": "test", "version": "1", "license": "MIT"}
            with patch.object(license_inventory, "ROOT", root):
                self.assertEqual(license_inventory.supplement(package, source), record)
                notice.write_text("Altered")
                with self.assertRaises(ValueError):
                    license_inventory.supplement(package, source)
                notice.write_text("Test notice")
                with self.assertRaises(ValueError):
                    license_inventory.supplement(package | {"license": "Apache-2.0"}, source)
                (source / ".cargo_vcs_info.json").write_text('{"git":{"sha1":"wrong"}}')
                with self.assertRaises(ValueError):
                    license_inventory.supplement(package, source)
                index.write_text(json.dumps({"packages": [record, record]}))
                with self.assertRaises(ValueError):
                    license_inventory.supplement(package, source)


if __name__ == "__main__":
    unittest.main()
