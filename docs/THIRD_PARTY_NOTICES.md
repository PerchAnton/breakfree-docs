# Dependency and asset notices

The BreakFree Docs software and brand licenses remain owner decisions. This file
does not change them. Project implementation and tests are original. No dependency
implementation source, font, alternate product branding or Office engine is copied
into this repository.

| Dependency | Pin | Purpose | Upstream license / source |
| :--- | :--- | :--- | :--- |
| regex | 2026.9.29 | Extended grapheme segmentation in the model | Apache-2.0 AND CNRI-Python; [PyPI](https://pypi.org/project/regex/2026.9.29/), [source](https://github.com/mrabarnett/mrab-regex) |
| PySide6-Essentials / shiboken6 / Qt | 6.11.1 | Optional desktop and QtTest bindings; Qt dynamically loaded from the installed wheels | PySide metadata: LGPL-3.0-only OR GPL-2.0-only OR GPL-3.0-only; Qt components have their own notices. [Qt for Python licensing](https://doc.qt.io/qtforpython-6/licenses.html), [Qt licensing](https://doc.qt.io/qt-6/licensing.html) |
| setuptools | 82.0.1 | Build backend | MIT; [source](https://github.com/pypa/setuptools) |
| build | 1.3.0 | Developer wheel/sdist command | MIT; [source](https://github.com/pypa/build) |
| Ruff | 0.14.0 | Developer formatting/lint checks | MIT; [source](https://github.com/astral-sh/ruff) |

Python itself and transitive build tools retain their respective upstream licenses.
Dependency distributions carry upstream copyright/license materials; do not strip
them when redistributing. This prototype publishes no bundled runtime, installer
or release. Before distributing one, inventory the exact Qt components and bundled
third-party notices, satisfy the chosen licensing terms and review replacement and
relinking requirements. Optional desktop installation is not an owner license decision.

`src/breakfree_docs/assets/breakfree-docs.webp` is a byte-identical copy of the
approved `docs/brand/breakfree-docs.webp` needed in the installed wheel. Both retain
the existing pending brand-license status. No font files are bundled; Qt uses the
system general UI font, whose fallback and shaping depend on the host.
