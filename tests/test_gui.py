"""Qt event-loop tests; run with QT_QPA_PLATFORM=offscreen on a headless host."""

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

try:
    from PySide6.QtCore import Qt
    from PySide6.QtGui import QInputMethodEvent, QKeySequence, QTextCursor, QTextDocument
    from PySide6.QtTest import QTest
    from PySide6.QtWidgets import QApplication, QMessageBox

    from breakfree_docs.gui import MainWindow
    from breakfree_docs.layout import project, raw_text
except ImportError:
    QApplication = None

from breakfree_docs.commands import Editor
from breakfree_docs.integration import main
from breakfree_docs.presentation import spans, utf16_offsets
from breakfree_docs.session import Session
from breakfree_docs.storage import StorageError, load


@unittest.skipIf(QApplication is None, "Install the desktop extra to run Qt tests")
class DesktopTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.application = QApplication.instance() or QApplication([])

    def setUp(self):
        self.window = MainWindow()
        self.window.show()
        self.window.activateWindow()
        self.window.view.setFocus()
        self.application.processEvents()
        QTest.qWaitForWindowActive(self.window, 1000)
        self.errors = []
        self.window.show_error = self.errors.append
        self.directory = tempfile.TemporaryDirectory()
        self.path = Path(self.directory.name) / "keyboard.bfdoc"

    def tearDown(self):
        self.window.session.saved_document = self.window.session.editor.document
        self.window.close()
        self.application.processEvents()
        self.directory.cleanup()
        self.assertEqual(self.errors, [])

    def shortcut(self, key):
        QTest.keySequence(self.window.view, QKeySequence(key))
        self.application.processEvents()

    def select(self, start, end):
        offsets = utf16_offsets(self.window.session.editor.document.text)
        cursor = self.window.view.textCursor()
        cursor.setPosition(offsets[start])
        cursor.setPosition(offsets[end], QTextCursor.MoveMode.KeepAnchor)
        self.window.view.setTextCursor(cursor)

    def commit_unicode(self, text):
        event = QInputMethodEvent()
        event.setCommitString(text)
        QApplication.sendEvent(self.window.view, event)
        self.application.processEvents()

    def test_keyboard_workflow_and_native_roundtrip(self):
        window = self.window
        QTest.keyClicks(window.view, "First paragraph")
        QTest.keyClick(window.view, Qt.Key.Key_Return)
        self.commit_unicode("Вітаю Україно! Привет. e\u0301 🙂")
        expected_text = "First paragraph\nВітаю Україно! Привет. e\u0301 🙂"
        self.assertEqual(window.session.editor.document.text, expected_text)
        self.assertEqual(raw_text(window.view.document()), expected_text)
        self.select(0, 5)
        self.shortcut(QKeySequence.StandardKey.Bold)
        self.shortcut(QKeySequence.StandardKey.Italic)
        self.assertTrue(
            all(
                s.bold and s.italic for s in window.session.editor.document.paragraphs[0].styles[:5]
            )
        )
        self.assertTrue(window.actions["bold"].isChecked())
        # Repeating the same shortcut removes emphasis rather than toggling only the widget.
        self.shortcut(QKeySequence.StandardKey.Bold)
        self.assertTrue(
            all(
                not s.bold and s.italic
                for s in window.session.editor.document.paragraphs[0].styles[:5]
            )
        )
        self.shortcut(QKeySequence.StandardKey.Undo)
        self.assertTrue(
            all(s.bold for s in window.session.editor.document.paragraphs[0].styles[:5])
        )
        window.actions["center"].trigger()
        formatted = window.session.editor.document
        self.assertEqual(formatted.paragraphs[0].alignment, "center")
        self.select(0, 5)
        QTest.keyClicks(window.view, "New")
        self.assertTrue(window.session.editor.document.text.startswith("New paragraph"))
        # Each key is an explicit operation; replacement of the selection is one operation.
        for _ in range(3):
            self.shortcut(QKeySequence.StandardKey.Undo)
        self.assertEqual(window.session.editor.document, formatted)
        for _ in range(3):
            self.shortcut(QKeySequence.StandardKey.Redo)
        final = window.session.editor.document
        window.choose_save_path = lambda: self.path
        self.shortcut(QKeySequence.StandardKey.Save)
        self.assertFalse(window.session.dirty)
        self.assertEqual(load(self.path), final)
        window.new_document()
        self.assertEqual(window.session.editor.document.text, "")
        with patch(
            "breakfree_docs.gui.QFileDialog.getOpenFileName", return_value=(str(self.path), "")
        ):
            self.shortcut(QKeySequence.StandardKey.Open)
        self.assertEqual(window.session.editor.document, final)
        self.select(len(final.text), len(final.text))
        QTest.keyClicks(window.view, "!")
        self.assertTrue(window.session.dirty)

    def test_paste_cut_unicode_boundaries_and_undo(self):
        QApplication.clipboard().setText("ї\nРусский\ne\u0301🙂\u00a0")
        self.shortcut(QKeySequence.StandardKey.Paste)
        original = self.window.session.editor.document
        self.assertEqual(original.text, "ї\nРусский\ne\u0301🙂\u00a0")
        self.assertEqual(len(self.window.session.editor.undo_stack), 1)
        self.select(len(original.text), len(original.text))
        QTest.keyClick(self.window.view, Qt.Key.Key_Backspace)
        QTest.keyClick(self.window.view, Qt.Key.Key_Backspace)
        QTest.keyClick(self.window.view, Qt.Key.Key_Backspace)
        self.assertEqual(self.window.session.editor.document.text, "ї\nРусский\n")
        for _ in range(3):
            self.shortcut(QKeySequence.StandardKey.Undo)
        self.assertEqual(self.window.session.editor.document, original)
        self.select(0, 1)
        self.shortcut(QKeySequence.StandardKey.Cut)
        self.assertEqual(QApplication.clipboard().text(), "ї")
        self.assertEqual(self.window.session.editor.document.text, original.text[1:])
        self.shortcut(QKeySequence.StandardKey.Undo)
        self.assertEqual(self.window.session.editor.document, original)

    def test_save_discard_cancel_close_and_new(self):
        window = self.window
        QTest.keyClicks(window.view, "keep me")
        original = window.session
        with patch(
            "breakfree_docs.gui.QMessageBox.warning", return_value=QMessageBox.StandardButton.Cancel
        ):
            self.assertFalse(window.close())
            window.new_document()
        self.assertIs(window.session, original)
        self.assertTrue(window.isVisible())
        window.choose_save_path = lambda: None
        with patch(
            "breakfree_docs.gui.QMessageBox.warning", return_value=QMessageBox.StandardButton.Save
        ):
            self.assertFalse(window.close())
        window.choose_save_path = lambda: self.path
        with patch(
            "breakfree_docs.gui.QMessageBox.warning", return_value=QMessageBox.StandardButton.Save
        ):
            window.new_document()
        self.assertEqual(load(self.path).text, "keep me")
        self.assertEqual(window.session.editor.document.text, "")
        QTest.keyClicks(window.view, "discard me")
        with patch(
            "breakfree_docs.gui.QMessageBox.warning",
            return_value=QMessageBox.StandardButton.Discard,
        ):
            window.new_document()
        self.assertEqual(window.session.editor.document.text, "")
        self.assertEqual(load(self.path).text, "keep me")

    def test_gui_save_failure_and_corrupt_open_keep_session(self):
        window = self.window
        QTest.keyClicks(window.view, "keep me")
        before = window.session
        window.choose_save_path = lambda: self.path
        with patch(
            "breakfree_docs.gui.QMessageBox.warning", return_value=QMessageBox.StandardButton.Save
        ):
            with patch("breakfree_docs.session.storage.save", side_effect=StorageError("injected")):
                self.assertFalse(window.close())
        self.assertIs(window.session, before)
        self.assertTrue(window.session.dirty)
        self.assertEqual(self.errors.pop(), "injected")
        self.path.write_bytes(b"corrupt")
        with patch(
            "breakfree_docs.gui.QFileDialog.getOpenFileName", return_value=(str(self.path), "")
        ):
            window.open_document()
        self.assertIs(window.session, before)
        self.assertIn("Cannot read", self.errors.pop())
        self.assertEqual(self.path.read_bytes(), b"corrupt")

    def test_layout_projection_wrap_and_styles_without_a_window(self):
        editor = Editor()
        editor.replace_text(0, 0, "English Українська Русский e\u0301 🙂 " * 10 + "\nNext")
        editor.emphasize(0, 7, "bold", True)
        editor.align(0, 7, "right")
        target = QTextDocument()
        target.setTextWidth(180)
        project(editor.document, target)
        self.assertEqual(raw_text(target), editor.document.text)
        self.assertGreater(target.size().height(), 0)
        self.assertGreater(target.begin().layout().lineCount(), 1)
        self.assertEqual(target.begin().blockFormat().alignment(), Qt.AlignmentFlag.AlignRight)
        self.assertTrue(spans(editor.document)[0].style.bold)
        cursor = QTextCursor(target)
        cursor.setPosition(1)
        self.assertGreaterEqual(cursor.charFormat().fontWeight(), 700)

    def test_pending_emphasis_and_ime_preedit_commit(self):
        window = self.window
        self.shortcut(QKeySequence.StandardKey.Bold)
        QTest.keyClicks(window.view, "a")
        self.assertTrue(window.session.editor.document.paragraphs[0].styles[0].bold)
        count = len(window.session.editor.undo_stack)
        preedit = QInputMethodEvent("ї", [])
        QApplication.sendEvent(window.view, preedit)
        self.assertEqual(window.session.editor.document.text, "a")
        self.assertEqual(len(window.session.editor.undo_stack), count)
        self.commit_unicode("ї")
        self.assertEqual(window.session.editor.document.text, "aї")
        self.assertEqual(len(window.session.editor.undo_stack), count + 1)
        self.assertTrue(window.session.editor.document.paragraphs[0].styles[-1].bold)

    def test_process_launch_commands_use_the_editor_session(self):
        self.assertFalse(self.window.windowIcon().isNull())
        with patch("breakfree_docs.gui.run", return_value=0) as launch:
            self.assertEqual(main(["--new"]), 0)
            session = launch.call_args.args[0]
            self.assertIsInstance(session, Session)
            self.assertEqual(session.editor.document.text, "")
            session.editor.replace_text(0, 0, "Launch from a local path")
            session.save(self.path)
            self.assertEqual(main(["--open", str(self.path)]), 0)
            reopened = launch.call_args.args[0]
            self.assertEqual(reopened.editor.document, session.editor.document)
            self.assertEqual(reopened.path, self.path)

    def test_open_current_file_after_save_uses_latest_committed_document(self):
        window = self.window
        QTest.keyClicks(window.view, "saved")
        window.session.save(self.path)
        QTest.keyClicks(window.view, " changes")
        latest = window.session.editor.document
        with (
            patch(
                "breakfree_docs.gui.QFileDialog.getOpenFileName", return_value=(str(self.path), "")
            ),
            patch(
                "breakfree_docs.gui.QMessageBox.warning",
                return_value=QMessageBox.StandardButton.Save,
            ),
        ):
            window.open_document()
        self.assertEqual(window.session.editor.document, latest)
        self.assertEqual(load(self.path), latest)
        self.assertFalse(window.session.dirty)


if __name__ == "__main__":
    unittest.main()
