"""Desktop controls and dialogs over the shared document session."""

import sys
from importlib.resources import files
from pathlib import Path

from PySide6.QtCore import QMimeData, Qt
from PySide6.QtGui import QAction, QFontDatabase, QIcon, QKeySequence, QTextCursor
from PySide6.QtWidgets import (
    QApplication,
    QFileDialog,
    QLabel,
    QMainWindow,
    QMenu,
    QMessageBox,
    QTextEdit,
    QToolBar,
)

from . import __version__
from .layout import character_format, project, raw_text
from .model import DocumentError, boundaries, validate_text
from .presentation import scalar_offset, snap_selection, spans, utf16_offsets
from .session import Session
from .storage import StorageError, load


class DocumentView(QTextEdit):
    """Qt handles text input/navigation; every committed change enters the engine."""

    def __init__(self, window):
        super().__init__(window)
        self.window = window
        self.setAcceptRichText(False)
        self.setAcceptDrops(False)
        self.setAccessibleName("Document text")
        self.setPlaceholderText("Start writing…")
        font = QFontDatabase.systemFont(QFontDatabase.SystemFont.GeneralFont)
        font.setPointSize(13)
        self.setFont(font)
        self.setTabChangesFocus(True)
        self.document().setDocumentMargin(48)
        self.textChanged.connect(window.text_changed)
        self.cursorPositionChanged.connect(window.cursor_changed)

    def insertFromMimeData(self, source: QMimeData) -> None:
        if source.hasText():
            text = source.text().replace("\r\n", "\n").replace("\r", "\n")
            self.window.replace_selection(text, "Paste text")

    def inputMethodEvent(self, event):
        try:
            validate_text(event.commitString())
        except DocumentError as error:
            self.window.show_error(str(error))
            event.accept()
            return
        super().inputMethodEvent(event)

    def keyPressEvent(self, event) -> None:
        window = self.window
        if event.matches(QKeySequence.StandardKey.Undo):
            window.history(False)
        elif event.matches(QKeySequence.StandardKey.Redo):
            window.history(True)
        elif event.matches(QKeySequence.StandardKey.Cut):
            window.cut()
        elif event.matches(QKeySequence.StandardKey.Copy):
            window.copy()
        elif event.key() in (Qt.Key.Key_Backspace, Qt.Key.Key_Delete):
            start, end = window.selection()
            edges = boundaries(window.session.editor.document.text)
            if start == end:
                index = edges.index(start)
                if event.key() == Qt.Key.Key_Backspace and index:
                    start = edges[index - 1]
                elif event.key() == Qt.Key.Key_Delete and index < len(edges) - 1:
                    end = edges[index + 1]
            # Modifier deletion uses the same safe grapheme operation in this prototype.
            window.replace_range(start, end, "", "Delete text")
        else:
            super().keyPressEvent(event)

    def contextMenuEvent(self, event) -> None:
        menu = QMenu(self)
        for name in ("undo", "redo", "cut", "copy", "paste", "select_all"):
            menu.addAction(self.window.actions[name])
        menu.exec(event.globalPos())


class MainWindow(QMainWindow):
    def __init__(self, session: Session | None = None):
        super().__init__()
        self.session = session or Session()
        self.rendering = True
        self.actions = {}
        self.resize(1050, 790)
        icon = str(files("breakfree_docs").joinpath("assets/breakfree-docs.webp"))
        self.setWindowIcon(QIcon(icon))
        self.view = DocumentView(self)
        self.setCentralWidget(self.view)
        self._create_controls()
        self.setStyleSheet("""
            QMainWindow { background: #f3eff8; }
            QTextEdit { background: #fff; color: #292233; border: 1px solid #d8cee8;
                        margin: 18px 50px; selection-background-color: #d5bafa; }
            QTextEdit:focus { border: 2px solid #8055c8; }
            QToolBar { background: #f3eff8; spacing: 6px; padding: 8px; border: none; }
            QToolButton { padding: 6px; border-radius: 4px; color: #392451; }
            QToolButton:checked { background: #ddc9f3; }
            QToolButton:focus { border: 2px solid #8055c8; }
            QStatusBar { color: #59476c; }
        """)
        self.rendering = False
        self.render()
        self.view.setFocus()

    def _action(self, name, text, shortcut, callback, *, checkable=False):
        action = QAction(text, self)
        action.setObjectName(name)
        if shortcut:
            action.setShortcut(shortcut)
        action.setCheckable(checkable)
        action.triggered.connect(callback)
        self.actions[name] = action
        return action

    def _create_controls(self):
        file_menu = self.menuBar().addMenu("&File")
        edit_menu = self.menuBar().addMenu("&Edit")
        format_menu = self.menuBar().addMenu("F&ormat")
        toolbar = QToolBar("Document controls", self)
        toolbar.setMovable(False)
        self.addToolBar(toolbar)
        brand = QLabel()
        brand.setPixmap(self.windowIcon().pixmap(32, 32))
        brand.setAccessibleName("BreakFree Docs approved logo")
        toolbar.addWidget(brand)
        toolbar.addSeparator()
        definitions = [
            ("new", "&New", QKeySequence.StandardKey.New, self.new_document),
            ("open", "&Open…", QKeySequence.StandardKey.Open, self.open_document),
            ("save", "&Save", QKeySequence.StandardKey.Save, self.save_document),
            ("save_as", "Save &As…", QKeySequence.StandardKey.SaveAs, self.save_as),
        ]
        for name, text, shortcut, callback in definitions:
            action = self._action(name, text, shortcut, callback)
            file_menu.addAction(action)
            if name != "save_as":
                toolbar.addAction(action)
        file_menu.addSeparator()
        file_menu.addAction(
            self._action("close", "&Close", QKeySequence.StandardKey.Close, self.close)
        )
        definitions = [
            ("undo", "&Undo", QKeySequence.StandardKey.Undo, lambda: self.history(False)),
            ("redo", "&Redo", QKeySequence.StandardKey.Redo, lambda: self.history(True)),
            ("cut", "Cu&t", QKeySequence.StandardKey.Cut, self.cut),
            ("copy", "&Copy", QKeySequence.StandardKey.Copy, self.copy),
            ("paste", "&Paste", QKeySequence.StandardKey.Paste, self.view.paste),
            ("select_all", "Select &All", QKeySequence.StandardKey.SelectAll, self.view.selectAll),
        ]
        toolbar.addSeparator()
        for name, text, shortcut, callback in definitions:
            action = self._action(name, text, shortcut, callback)
            edit_menu.addAction(action)
            if name in ("undo", "redo"):
                toolbar.addAction(action)
        toolbar.addSeparator()
        for name, text in (("bold", "&Bold"), ("italic", "&Italic")):
            # StandardKey supplies the platform shortcut, including Command on macOS.
            key = (
                QKeySequence.StandardKey.Bold if name == "bold" else QKeySequence.StandardKey.Italic
            )
            action = self._action(
                name, text, key, lambda checked, n=name: self.emphasize(n, checked), checkable=True
            )
            toolbar.addAction(action)
            format_menu.addAction(action)
        toolbar.addSeparator()
        for alignment, key in (("left", "L"), ("center", "E"), ("right", "R"), ("justify", "J")):
            action = self._action(
                alignment,
                alignment.title(),
                QKeySequence(f"{'Meta' if sys.platform == 'darwin' else 'Ctrl'}+Shift+{key}"),
                lambda checked, a=alignment: self.align(a),
                checkable=True,
            )
            toolbar.addAction(action)
            format_menu.addAction(action)
        self.status = QLabel()
        self.statusBar().addWidget(self.status, 1)
        self.statusBar().addPermanentWidget(QLabel(f"Local files · {__version__}"))

    def selection(self):
        text = self.session.editor.document.text
        cursor = self.view.textCursor()
        start = scalar_offset(text, cursor.selectionStart())
        end = scalar_offset(text, cursor.selectionEnd())
        return snap_selection(text, start, end)

    def cursor_changed(self):
        if self.rendering or raw_text(self.view.document()) != self.session.editor.document.text:
            return
        start, end = self.selection()
        editor = self.session.editor
        editor.selection = (start, end)
        editor.typing_style = editor.document.style_at(start)
        self.update_controls()

    def text_changed(self):
        if self.rendering:
            return
        editor = self.session.editor
        old, new = editor.document.text, raw_text(self.view.document())
        if old == new:
            return
        start = 0
        while start < min(len(old), len(new)) and old[start] == new[start]:
            start += 1
        old_end, new_end = len(old), len(new)
        while old_end > start and new_end > start and old[old_end - 1] == new[new_end - 1]:
            old_end -= 1
            new_end -= 1
        old_edges, new_edges = boundaries(old), boundaries(new)
        while start not in old_edges or start not in new_edges:
            start -= 1
        while old_end not in old_edges or new_end not in new_edges:
            old_end += 1
            new_end += 1
        try:
            editor.replace_text(start, old_end, new[start:new_end], "Type text / input commit")
        except DocumentError as error:
            self.render()
            self.show_error(str(error))
            return
        self.render()

    def render(self):
        self.rendering = True
        try:
            editor = self.session.editor
            scroll = self.view.verticalScrollBar().value()
            project(editor.document, self.view.document())
            offsets = utf16_offsets(editor.document.text)
            start, end = snap_selection(editor.document.text, *editor.selection)
            cursor = self.view.textCursor()
            cursor.setPosition(offsets[start])
            cursor.setPosition(offsets[end], QTextCursor.MoveMode.KeepAnchor)
            self.view.setTextCursor(cursor)
            if start == end:
                self.view.setCurrentCharFormat(character_format(editor.typing_style))
            self.view.verticalScrollBar().setValue(scroll)
        finally:
            self.rendering = False
        self.update_controls()

    def update_controls(self):
        editor = self.session.editor
        name = self.session.path.name if self.session.path else "Untitled"
        self.setWindowTitle(f"{'• ' if self.session.dirty else ''}{name} — BreakFree Docs")
        self.actions["undo"].setEnabled(bool(editor.undo_stack))
        self.actions["redo"].setEnabled(bool(editor.redo_stack))
        start, end = editor.selection
        selected_styles = (
            [span.style for span in spans(editor.document) if span.start < end and span.end > start]
            if start != end
            else []
        )
        for attribute in ("bold", "italic"):
            enabled = (
                all(getattr(style, attribute) for style in selected_styles)
                if selected_styles
                else getattr(editor.typing_style, attribute)
            )
            self.actions[attribute].setChecked(enabled)
        paragraph, _ = editor.document.locate(editor.selection[0])
        for alignment in ("left", "center", "right", "justify"):
            self.actions[alignment].setChecked(
                editor.document.paragraphs[paragraph].alignment == alignment
            )
        text = editor.document.text
        save_state = (
            "Unsaved changes"
            if self.session.dirty
            else ("Saved" if self.session.path else "New document")
        )
        self.status.setText(
            f"{len(editor.document.paragraphs)} paragraphs · {len(text.split())} words · "
            f"{save_state}"
        )

    def replace_range(self, start, end, text, label):
        try:
            self.session.editor.replace_text(start, end, text, label)
        except DocumentError as error:
            self.show_error(str(error))
        self.render()

    def replace_selection(self, text, label="Replace text"):
        self.replace_range(*self.selection(), text, label)

    def copy(self):
        start, end = self.selection()
        QApplication.clipboard().setText(self.session.editor.document.text[start:end])

    def cut(self):
        self.copy()
        self.replace_selection("", "Cut text")

    def history(self, redo):
        editor = self.session.editor
        editor.redo() if redo else editor.undo()
        self.render()

    def emphasize(self, attribute, enabled):
        self.session.editor.emphasize(*self.selection(), attribute, enabled)
        self.render()
        self.view.setFocus()

    def align(self, alignment):
        self.session.editor.align(*self.selection(), alignment)
        self.render()
        self.view.setFocus()

    def show_error(self, message):
        QMessageBox.critical(self, "BreakFree Docs — file or editing error", message)

    def choose_save_path(self):
        dialog = QFileDialog(self, "Save document", str(self.session.path or "Untitled.bfdoc"))
        dialog.setAcceptMode(QFileDialog.AcceptMode.AcceptSave)
        dialog.setNameFilter("BreakFree Docs (*.bfdoc)")
        dialog.setDefaultSuffix("bfdoc")
        if dialog.exec() and dialog.selectedFiles():
            return Path(dialog.selectedFiles()[0])
        return None

    def save_document(self):
        target = self.session.path or self.choose_save_path()
        if target is None:
            return False
        try:
            self.session.save(target)
        except StorageError as error:
            self.show_error(str(error))
            return False
        self.update_controls()
        return True

    def save_as(self):
        target = self.choose_save_path()
        if target is None:
            return False
        try:
            self.session.save(target)
        except StorageError as error:
            self.show_error(str(error))
            return False
        self.update_controls()
        return True

    def confirm_leave(self):
        if not self.session.dirty:
            return True
        choice = QMessageBox.warning(
            self,
            "Unsaved changes",
            "Save changes before leaving this document?",
            QMessageBox.StandardButton.Save
            | QMessageBox.StandardButton.Discard
            | QMessageBox.StandardButton.Cancel,
            QMessageBox.StandardButton.Cancel,
        )
        if choice == QMessageBox.StandardButton.Save:
            return self.save_document()
        decision = "discard" if choice == QMessageBox.StandardButton.Discard else "cancel"
        return self.session.can_leave(decision)

    def new_document(self):
        if self.confirm_leave():
            self.session = Session()
            self.render()
            self.view.setFocus()

    def open_document(self):
        filename, _ = QFileDialog.getOpenFileName(
            self, "Open document", "", "BreakFree Docs (*.bfdoc)"
        )
        if not filename:
            return
        try:
            document = load(Path(filename))
        except StorageError as error:
            self.show_error(str(error))
            return
        if self.confirm_leave():
            # Saving the current document can change the selected file (including
            # Save As to that path). Read the committed file after that decision.
            try:
                document = load(Path(filename))
            except StorageError as error:
                self.show_error(str(error))
                return
            self.session = Session(document, Path(filename))
            self.render()
            self.view.setFocus()

    def closeEvent(self, event):
        event.accept() if self.confirm_leave() else event.ignore()


def run(session: Session) -> int:
    application = QApplication(["breakfree-docs"])
    application.setApplicationName("BreakFree Docs")
    application.setOrganizationName("BreakFree")
    application.setApplicationVersion(__version__)
    window = MainWindow(session)
    window.show()
    return application.exec()
