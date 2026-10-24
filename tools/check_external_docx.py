"""Run a bounded macOS external save-back probe; never assert full fidelity.
Usage: python3 tools/check_external_docx.py NEW_OUTPUT_DIRECTORY
Requires Cargo and macOS textutil. Does not modify any committed fixture.
"""
import hashlib
import json
import platform
import subprocess
import sys
import zipfile
from pathlib import Path
from xml.etree import ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
W = '{http://schemas.openxmlformats.org/wordprocessingml/2006/main}'

def projection(path):
    with zipfile.ZipFile(path) as z:
        body = ET.fromstring(z.read('word/document.xml')).find(W + 'body')
        result = []
        for p in body.findall(W + 'p'):
            runs = []
            for r in p.findall(W + 'r'):
                text = ''.join(t.text or '' for t in r.findall(W + 't'))
                pr = r.find(W + 'rPr')
                flags = {}
                for key in ['b', 'i']:
                    flag = None if pr is None else pr.find(W + key)
                    flags[key] = flag is not None and flag.get(W + 'val', '1') not in ['0', 'false', 'off']
                runs.extend((ch, flags['b'], flags['i']) for ch in text)
            jc = p.find(W + 'pPr/' + W + 'jc')
            result.append((runs, 'left' if jc is None else jc.get(W + 'val')))
        return result

def main():
    if platform.system() != 'Darwin' or len(sys.argv) != 2:
        raise SystemExit('Requires macOS and one new output directory')
    out = Path(sys.argv[1]).resolve()
    out.mkdir(parents=True, exist_ok=False)
    source = ROOT / 'tests/fixtures/docx-corpus/independent.docx'
    saved = out / 'docs-saved.docx'
    subprocess.run(['cargo', 'run', '--locked', '--no-default-features', '--example', 'docx_saveback', '--', str(source), str(saved)], cwd=ROOT, check=True)
    returned = out / 'cocoa-returned.docx'
    subprocess.run(['/usr/bin/textutil', '-convert', 'docx', '-output', str(returned), str(saved)], check=True)
    # Compare only named semantics. This is not a whole-model or pagination oracle.
    before = projection(saved)
    assert len(before) == 2 and before[0][0], 'Missing baseline paragraphs'
    assert any(b for _, b, _ in before[0][0]) and any(i for _, _, i in before[0][0])
    assert before == projection(returned), 'External text/emphasis/alignment changed'
    returned_bytes = returned.read_bytes()
    probe = subprocess.run(['cargo', 'run', '--locked', '--no-default-features', '--example', 'docx_saveback', '--', str(returned), str(out / 'unexpected.docx')], cwd=ROOT, capture_output=True, text=True)
    assert probe.returncode != 0 and 'customXml' in probe.stderr
    assert not (out / 'unexpected.docx').exists()
    assert returned.read_bytes() == returned_bytes, 'Rejected source changed'
    report = dict(platform=subprocess.check_output(['/usr/bin/sw_vers'], text=True).strip(), external_tool='textutil / CocoaOOXMLWriter', compared=['paragraph text, including Unicode', 'per-character bold and italic', 'paragraph alignment'], preserved=True, reimport='Rejected: unsupported customXml relationship; no output written', full_fidelity=False, files={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in [source, saved, returned]})
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))

if __name__ == '__main__':
    main()
