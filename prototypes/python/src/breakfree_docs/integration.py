"""Optional Office process adapter. No Office modules or GUI imports on headless paths."""

import argparse
import json
from importlib.metadata import PackageNotFoundError, distribution
from pathlib import Path

from . import __version__

APP_ID = "breakfree.docs"
DISPLAY_NAME = "BreakFree Docs"


def capabilities() -> dict:
    try:
        distribution("PySide6-Essentials")
        launch_actions = ["new", "open"]
    except PackageNotFoundError:
        launch_actions = []
    return {
        "protocol_version": 1,
        "app_id": APP_ID,
        "app_version": __version__,
        "display_name": DISPLAY_NAME,
        "capabilities": launch_actions,
        "formats": [
            {"id": "breakfree.docs.native", "extensions": ["bfdoc"], "read": True, "write": True}
        ],
    }


def arguments(argv=None):
    parser = argparse.ArgumentParser(prog="breakfree-docs", description=DISPLAY_NAME)
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--new", action="store_true", help="Create a new document (the default).")
    group.add_argument("--open", type=Path, metavar="ABSOLUTE_PATH", help="Open a native .bfdoc.")
    group.add_argument(
        "--capabilities", action="store_true", help="Print protocol v1 JSON and exit."
    )
    group.add_argument("--version", action="store_true", help="Print the build version and exit.")
    args = parser.parse_args(argv)
    if args.open is not None and not args.open.is_absolute():
        parser.error("--open requires an absolute path")
    return args


def main(argv=None) -> int:
    args = arguments(argv)
    if args.capabilities:
        print(json.dumps(capabilities(), ensure_ascii=False))
        return 0
    if args.version:
        print(f"{DISPLAY_NAME} {__version__}")
        return 0
    # Validate the input before importing Qt or starting a window.
    from .session import Session
    from .storage import StorageError, load

    try:
        session = Session(load(args.open), args.open) if args.open else Session()
    except StorageError as error:
        import sys

        print(f"{DISPLAY_NAME}: {error}", file=sys.stderr)
        return 1
    try:
        from .gui import run
    except ImportError:
        import sys

        print('Desktop dependencies missing. Install "breakfree-docs[desktop]".', file=sys.stderr)
        return 1
    return run(session)
