#!/usr/bin/env python3
"""Bundle index.html into a single self-contained file for publishing.

The repo layout keeps the XM player as js/xm.js; published artifacts must be
one file with no external requests, so the module import is replaced by the
player source inlined (export keyword stripped).
"""
import pathlib
import sys

here = pathlib.Path(__file__).parent
html = (here / "index.html").read_text()
xm = (here / "js" / "xm.js").read_text().replace("export class XmPlayer", "class XmPlayer")

needle = "import { XmPlayer } from './js/xm.js';"
if needle not in html:
    sys.exit("import statement not found in index.html")
html = html.replace(needle, "// --- inlined js/xm.js ---\n" + xm)

out = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else here / "heaven7-artifact.html"
out.write_text(html)
print(f"wrote {out} ({out.stat().st_size} bytes)")
