"""Document lifecycle, shared unsaved-change decisions and clean checkpoints."""

from pathlib import Path
from typing import Literal

from . import storage
from .commands import Editor
from .model import Document


class Session:
    def __init__(self, document: Document | None = None, path: Path | None = None):
        self.editor = Editor(document)
        self.path = path
        self.saved_document = self.editor.document

    @property
    def dirty(self) -> bool:
        return self.editor.document != self.saved_document

    def save(self, path: Path | None = None) -> None:
        target = path or self.path
        if target is None:
            raise storage.StorageError("Choose a location for the document.")
        storage.save(self.editor.document, target)
        self.path = target
        self.saved_document = self.editor.document

    def can_leave(self, choice: Literal["save", "discard", "cancel"], path=None) -> bool:
        if not self.dirty:
            return True
        if choice == "cancel":
            return False
        if choice == "discard":
            return True
        if choice == "save":
            self.save(path)
            return True
        raise ValueError("Unknown unsaved-change choice.")
