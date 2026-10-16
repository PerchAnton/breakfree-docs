import unittest

from breakfree_docs.commands import Editor
from breakfree_docs.model import Document, DocumentError, Style, boundaries
from breakfree_docs.presentation import scalar_offset, snap_selection, spans, utf16_offsets


class EditingTests(unittest.TestCase):
    def test_edit_format_undo_redo_exact_states(self):
        editor = Editor()
        states = [editor.document]
        editor.replace_text(0, 0, "Hello\nВітаю\nПривет")
        states.append(editor.document)
        editor.emphasize(0, 5, "bold", True)
        states.append(editor.document)
        editor.emphasize(6, 11, "italic", True)
        states.append(editor.document)
        editor.align(6, 11, "center")
        states.append(editor.document)
        editor.replace_text(0, 5, "Welcome")
        states.append(editor.document)
        editor.replace_text(7, 8, "")
        states.append(editor.document)
        for expected in reversed(states[:-1]):
            editor.undo()
            self.assertEqual(editor.document, expected)
        for expected in states[1:]:
            editor.redo()
            self.assertEqual(editor.document, expected)
        self.assertTrue(editor.document.paragraphs[0].text.startswith("WelcomeВітаю"))

    def test_unicode_graphemes_and_scalars(self):
        for text in ("English", "Українська ї є ґ", "Русский ё", "e\u0301", "🙂", "👩‍💻", "🇺🇦"):
            with self.subTest(text=text):
                editor = Editor()
                editor.replace_text(0, 0, text)
                self.assertEqual(editor.document.text, text)
                for offset in range(len(text) + 1):
                    if offset not in boundaries(text):
                        with self.assertRaises(DocumentError):
                            editor.replace_text(offset, offset, "x")
                        with self.assertRaises(DocumentError):
                            editor.emphasize(0, offset, "bold", True)
                editor.replace_text(0, len(text), "new")
                editor.undo()
                self.assertEqual(editor.document.text, text)
        self.assertEqual(utf16_offsets("a🙂b"), (0, 1, 3, 4))
        self.assertEqual(scalar_offset("a🙂b", 3), 2)
        with self.assertRaises(DocumentError):
            scalar_offset("a🙂b", 2)
        self.assertEqual(snap_selection("e\u0301", 1, 1), (0, 0))
        self.assertEqual(snap_selection("e\u0301", 1, 2), (0, 2))

    def test_paragraph_split_merge_and_exclusive_alignment_end(self):
        editor = Editor()
        editor.replace_text(0, 0, "one\ntwo\nthree")
        editor.align(0, 4, "right")
        self.assertEqual(
            [p.alignment for p in editor.document.paragraphs], ["right", "left", "left"]
        )
        editor.replace_text(1, 7, "A\nB")
        self.assertEqual(editor.document.text, "oA\nB\nthree")
        self.assertEqual(
            [p.alignment for p in editor.document.paragraphs], ["right", "right", "left"]
        )
        editor.undo()
        self.assertEqual(editor.document.text, "one\ntwo\nthree")

    def test_invalid_edit_is_transactional_and_new_edit_clears_redo(self):
        editor = Editor()
        editor.replace_text(0, 0, "abc")
        before = editor.document
        for text in ("\x00", "\r", "\ud800", "\u2029"):
            with self.assertRaises(DocumentError):
                editor.replace_text(0, 0, text)
            self.assertIs(editor.document, before)
        editor.undo()
        editor.replace_text(0, 0, "different")
        self.assertFalse(editor.redo_stack)

    def test_typing_style_and_history_boundaries(self):
        editor = Editor()
        editor.emphasize(0, 0, "bold", True)
        self.assertEqual(editor.document, Document())
        editor.replace_text(0, 0, "ab")
        self.assertEqual(editor.document.paragraphs[0].styles, (Style(bold=True),) * 2)
        self.assertEqual(len(spans(editor.document)), 1)
        editor.replace_text(2, 2, "c")
        editor.undo()
        self.assertEqual(editor.document.text, "ab")
        editor.undo()
        self.assertEqual(editor.document.text, "")

    def test_insertion_joining_clusters_keeps_caret_at_a_boundary(self):
        editor = Editor()
        editor.replace_text(0, 0, "👩💻")
        editor.replace_text(1, 1, "\u200d")
        self.assertEqual(editor.document.text, "👩‍💻")
        self.assertEqual(editor.selection, (3, 3))
        editor.replace_text(*editor.selection, "!")
        self.assertEqual(editor.document.text, "👩‍💻!")


if __name__ == "__main__":
    unittest.main()
