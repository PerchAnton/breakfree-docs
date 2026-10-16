"""Qt text layout projection, testable without editor widgets or application windows."""

from PySide6.QtCore import Qt
from PySide6.QtGui import QFont, QTextBlockFormat, QTextCharFormat, QTextCursor, QTextDocument

from .model import Document, Style
from .presentation import spans, utf16_offsets

QT_ALIGNMENTS = {
    "left": Qt.AlignmentFlag.AlignLeft,
    "center": Qt.AlignmentFlag.AlignHCenter,
    "right": Qt.AlignmentFlag.AlignRight,
    "justify": Qt.AlignmentFlag.AlignJustify,
}


def character_format(style: Style) -> QTextCharFormat:
    fmt = QTextCharFormat()
    fmt.setFontWeight(QFont.Weight.Bold if style.bold else QFont.Weight.Normal)
    fmt.setFontItalic(style.italic)
    return fmt


def raw_text(document: QTextDocument) -> str:
    # toPlainText normalizes NBSP to space; toRawText preserves it.
    return document.toRawText().replace("\u2029", "\n")


def project(source: Document, target: QTextDocument) -> None:
    """Rebuild text when necessary and always derive all formatting from the model."""
    target.setUndoRedoEnabled(False)
    if raw_text(target) != source.text:
        target.setPlainText(source.text)
    cursor = QTextCursor(target)
    cursor.select(QTextCursor.SelectionType.Document)
    cursor.setCharFormat(character_format(Style()))
    offsets = utf16_offsets(source.text)
    for span in spans(source):
        cursor.setPosition(offsets[span.start])
        cursor.setPosition(offsets[span.end], QTextCursor.MoveMode.KeepAnchor)
        cursor.setCharFormat(character_format(span.style))
    block = target.begin()
    for paragraph in source.paragraphs:
        fmt = QTextBlockFormat()
        fmt.setAlignment(QT_ALIGNMENTS[paragraph.alignment])
        fmt.setBottomMargin(12)
        cursor = QTextCursor(block)
        cursor.setBlockFormat(fmt)
        block = block.next()
