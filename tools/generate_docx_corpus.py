"""Generate original fixtures using macOS textutil, never normalize its output.
Run only to refresh evidence; normal Cargo tests use checked-in files offline.
"""
import hashlib
import json
import platform
import subprocess
import tempfile
import shutil
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'tests/fixtures/docx-corpus'

def run(*args):
    return subprocess.check_output(args, text=True)


def main():
    if platform.system() != 'Darwin':
        raise SystemExit('Generation requires macOS textutil; fixture tests do not.')
    cases = []
    # Independent, original OOXML authored without the BreakFree encoder.
    w = 'http://schemas.openxmlformats.org/wordprocessingml/2006/main'
    types = '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Override PartName="/body.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>'
    rels = '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="document" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="body.xml"/></Relationships>'
    body = f'<w:document xmlns:w="{w}"><w:body><w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Independent Україна é</w:t></w:r><w:r><w:rPr><w:i/></w:rPr><w:t xml:space="preserve"> italic</w:t></w:r></w:p><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:t>Centered</w:t></w:r></w:p><w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:left="1440" w:right="1440" w:top="1440" w:bottom="1440"/></w:sectPr></w:body></w:document>'
    with zipfile.ZipFile(OUT / 'independent.docx', 'w') as z:
        for name, text in [('body.xml', body), ('_rels/.rels', rels), ('[Content_Types].xml', types)]:
            z.writestr(zipfile.ZipInfo(name, (2026, 10, 24, 0, 0, 0)), text.encode())
    cases.append(dict(file='independent.docx', producer='Original independent OOXML via Python zipfile; not external application evidence', accepted=True))
    for name, source in [('cocoa-unicode', OUT / 'sources/unicode.txt'), ('cocoa-emphasis', OUT / 'sources/emphasis.rtf'), ('cocoa-saveback', OUT / 'independent.docx'), ('cocoa-docs-saveback', ROOT / 'examples/DocxExchange.docx')]:
        target = OUT / (name + '.docx')
        run('/usr/bin/textutil', '-convert', 'docx', '-font', 'Arial', '-fontsize', '12', '-output', str(target), str(source))
        cases.append(dict(file=target.name, producer='macOS textutil / CocoaOOXMLWriter', source=str(source.relative_to(ROOT)), accepted=False, error_contains='customXml'))
    with tempfile.TemporaryDirectory(prefix='docx-external-probe-') as temporary:
        output = Path(temporary) / 'probe'
        run(sys.executable, str(ROOT / 'tools/check_external_docx.py'), str(output))
        for name, accepted in [('docs-saved.docx', True), ('cocoa-returned.docx', False)]:
            shutil.copyfile(output / name, OUT / name)
            cases.append(dict(file=name, producer='BreakFree session export' if accepted else 'macOS textutil / CocoaOOXMLWriter after BreakFree edit/save', accepted=accepted, error_contains=None if accepted else 'customXml'))
        shutil.copyfile(output / 'report.json', OUT / 'saveback-report.json')
    for case in cases:
        if case['accepted']:
            case['expected_text'] = ('Edited · ' if case['file'] == 'docs-saved.docx' else '') + 'Independent Україна é italic'
        if 'source' in case:
            case['source_sha256'] = hashlib.sha256((ROOT / case['source']).read_bytes()).hexdigest()
        case['sha256'] = hashlib.sha256((OUT / case['file']).read_bytes()).hexdigest()
    data = dict(schema_version=1, platform=run('/usr/bin/sw_vers').strip(), tool_sha256=hashlib.sha256(Path('/usr/bin/textutil').read_bytes()).hexdigest(), cases=cases)
    (OUT / 'manifest.json').write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(data, ensure_ascii=False, indent=2))

if __name__ == '__main__':
    main()
