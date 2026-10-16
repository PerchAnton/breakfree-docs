import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from breakfree_docs import storage
from breakfree_docs.commands import Editor
from breakfree_docs.session import Session


class StorageTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.path = Path(self.directory.name) / "document.bfdoc"
        self.editor = Editor()
        self.editor.replace_text(0, 0, "English\nУкраїнська ї є ґ\nРусский ё\ne\u0301 🙂 👩‍💻\n")
        self.editor.emphasize(0, 7, "bold", True)
        self.editor.emphasize(8, 18, "italic", True)
        self.editor.align(8, 18, "justify")

    def test_save_reopen_preserves_all_supported_content(self):
        storage.save(self.editor.document, self.path)
        self.assertEqual(storage.load(self.path), self.editor.document)
        self.assertEqual(storage.encode(storage.load(self.path)), self.path.read_bytes())
        self.assertEqual(list(self.path.parent.glob("*.tmp")), [])

    def test_rejects_bad_foreign_future_and_unknown_fields_without_mutation(self):
        original = json.loads(storage.encode(self.editor.document))
        cases = [
            b"not JSON",
            b"\xff",
            b"[]",
            b"{}",
            b"[" * 2000,
            b'{"format":1,"format":2}',
        ]
        for key, value in (
            ("format", "breakfree.sheets.native"),
            ("content_version", 2),
            ("content_version", True),
            ("paragraphs", []),
            ("unsupported", "preserve me"),
        ):
            modified = dict(original)
            modified[key] = value
            cases.append(json.dumps(modified).encode())
        for data in cases:
            with self.subTest(data=data[:80]):
                self.path.write_bytes(data)
                with self.assertRaises(storage.StorageError):
                    storage.load(self.path)
                self.assertEqual(self.path.read_bytes(), data)

    def test_strict_nested_schema_and_text(self):
        for run in (
            {"text": "a", "bold": "yes", "italic": False},
            {"text": "\ud800", "bold": False, "italic": False},
            {"text": "a\nb", "bold": False, "italic": False},
            {"text": "a", "bold": False, "italic": False, "image": "x"},
        ):
            data = {
                "format": storage.FORMAT_ID,
                "content_version": 1,
                "paragraphs": [{"alignment": "left", "runs": [run]}],
            }
            with self.assertRaises(storage.StorageError):
                storage.decode(json.dumps(data).encode())

    def test_write_failures_preserve_previous_file_and_dirty_state(self):
        session = Session(self.editor.document)
        session.save(self.path)
        previous = self.path.read_bytes()
        session.editor.replace_text(0, 0, "Changed ")
        checkpoint = session.saved_document
        for operation in ("fsync", "replace", "chmod"):
            with self.subTest(operation=operation):
                with patch(
                    f"breakfree_docs.storage.os.{operation}", side_effect=OSError("injected")
                ):
                    with self.assertRaises(storage.StorageError):
                        session.save()
                self.assertEqual(self.path.read_bytes(), previous)
                self.assertEqual(storage.load(self.path), checkpoint)
                self.assertTrue(session.dirty)
                self.assertEqual(list(self.path.parent.glob(".*.tmp")), [])
        with self.assertRaises(storage.StorageError):
            session.save(self.path.parent / "missing" / "new.bfdoc")
        self.assertEqual(session.path, self.path)
        self.assertEqual(self.path.read_bytes(), previous)

    def test_partial_write_failure(self):
        storage.save(self.editor.document, self.path)
        original = self.path.read_bytes()
        real_open = storage.tempfile.NamedTemporaryFile

        class BrokenStream:
            def __enter__(self):
                self.stream = real_open(mode="wb", dir=self_dir, delete=False)
                self.name = self.stream.name
                return self

            def write(self, data):
                self.stream.write(data[:8])
                raise OSError("disk full during write")

            def __exit__(self, *args):
                self.stream.close()

        self_dir = self.path.parent
        with patch(
            "breakfree_docs.storage.tempfile.NamedTemporaryFile", return_value=BrokenStream()
        ):
            with self.assertRaises(storage.StorageError):
                storage.save(self.editor.document, self.path)
        self.assertEqual(self.path.read_bytes(), original)
        self.assertEqual(list(self.path.parent.iterdir()), [self.path])

    def test_unsupported_extension_and_symbolic_link(self):
        path = self.path.with_suffix(".docx")
        path.write_bytes(b"original")
        with self.assertRaises(storage.StorageError):
            storage.save(self.editor.document, path)
        with self.assertRaises(storage.StorageError):
            storage.load(path)
        self.assertEqual(path.read_bytes(), b"original")
        if os.name != "nt":
            self.path.symlink_to(path)
            with self.assertRaises(storage.StorageError):
                storage.save(self.editor.document, self.path)
            self.assertEqual(path.read_bytes(), b"original")

    def test_save_discard_cancel_and_clean_checkpoint(self):
        session = Session()
        session.editor.replace_text(0, 0, "unsaved")
        before = session.editor.document
        self.assertFalse(session.can_leave("cancel"))
        self.assertTrue(session.can_leave("discard"))
        self.assertTrue(session.dirty)
        self.assertEqual(session.editor.document, before)
        with self.assertRaises(storage.StorageError):
            session.can_leave("save")
        self.assertTrue(session.dirty)
        self.assertTrue(session.can_leave("save", self.path))
        self.assertFalse(session.dirty)
        session.editor.replace_text(0, 0, "new ")
        session.editor.undo()
        self.assertFalse(session.dirty)
        session.editor.redo()
        self.assertTrue(session.dirty)


if __name__ == "__main__":
    unittest.main()
