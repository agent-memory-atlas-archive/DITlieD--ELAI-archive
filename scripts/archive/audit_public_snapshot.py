#!/usr/bin/env python3
"""Inspect a publication directory without running its code; emit no matched values.

Private identity terms may be supplied as a JSON array outside the publication
folder. This is a bounded pattern scan, not proof that all private data is absent.
"""
import argparse
import hashlib
import json
import re
from pathlib import Path

SECRET_PATTERNS = {
    'provider-token': r'(?<![\w-])(?:sk-(?:proj-|ant-api\d+-)?[A-Za-z0-9_-]{20,}|gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,})(?![\w-])',
    'aws-key': r'\b(?:AKIA|ASIA)[A-Z0-9]{16}\b',
    'private-key': r'-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----',
    'credential-url': r'https?://[^\s/@:]+:[^\s/@]+@[^\s/]+',
}
FORBIDDEN_PARTS = {'.git', 'node_modules', '__pycache__', '.venv', 'venvs', '.hypothesis', '.elai-agents', '.source', '.source_files_aider'}
FORBIDDEN_SUFFIXES = {'.db', '.sqlite', '.sqlite3', '.key', '.pem', '.gguf', '.onnx', '.exe', '.dll', '.zip', '.rar', '.log'}
FORBIDDEN_PREFIXES = ('.elai_cc/.elai/', '.elai_cc/projects/', '.elai_cc/crates/dashboard-v2/workflow/', '.elai_cc/indexes/', '.elai_cc/workflow/', 'workflow/sessions/', 'workflow/audit-scratch/')


def inspect(root, terms):
    findings, binary, file_count, byte_count = [], [], 0, 0
    for path in sorted(root.rglob('*')):
        rel = path.relative_to(root).as_posix()
        # Do not inspect the fresh repository's Git internals as ordinary files.
        if '.git' in path.relative_to(root).parts:
            continue
        if path.is_symlink():
            findings.append({'path': rel, 'kind': 'symlink'})
            continue
        if not path.is_file():
            continue
        file_count += 1
        size = path.stat().st_size
        byte_count += size
        parts = path.relative_to(root).parts
        if (set(parts) & FORBIDDEN_PARTS or path.suffix.lower() in FORBIDDEN_SUFFIXES
                or rel.startswith(FORBIDDEN_PREFIXES) or path.name.startswith('.env')
                or path.name in {'.mcp.json', 'settings.local.json'}):
            findings.append({'path': rel, 'kind': 'excluded-artifact'})
        if size > 50_000_000:
            findings.append({'path': rel, 'kind': 'oversized-file', 'bytes': size})
        for i, term in enumerate(terms):
            if re.search(re.escape(term), rel, re.I):
                findings.append({'path': rel, 'kind': 'identity-in-path', 'term_index': i})
        data = path.read_bytes()
        try:
            if b'\0' in data[:8000]:
                raise UnicodeError
            content = data.decode('utf-8')
        except UnicodeError:
            binary.append({'path': rel, 'bytes': size, 'sha256': hashlib.sha256(data).hexdigest()})
            continue
        for lineno, line in enumerate(content.splitlines(), 1):
            for i, term in enumerate(terms):
                if re.search(re.escape(term), line, re.I):
                    findings.append({'path': rel, 'line': lineno, 'kind': 'identity', 'term_index': i})
            for kind, pattern in SECRET_PATTERNS.items():
                for match in re.finditer(pattern, line):
                    findings.append({'path': rel, 'line': lineno, 'kind': kind,
                                     'value_sha256': hashlib.sha256(match.group().encode()).hexdigest()})
    return {'files': file_count, 'bytes': byte_count, 'findings': findings, 'binary_files': binary,
            'scope': 'UTF-8 text and filenames; binary payloads and images need separate review. Secret-shaped test fixtures require contextual triage.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('snapshot', type=Path)
    parser.add_argument('--terms-file', type=Path, help='Private JSON array of identifiers; keep outside the snapshot')
    parser.add_argument('--allowlist', type=Path, help='Exact reviewed synthetic secret fingerprints; defaults to the snapshot copy')
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    root = args.snapshot.resolve(strict=True)
    terms = json.loads(args.terms_file.read_text()) if args.terms_file else []
    if not isinstance(terms, list) or not all(isinstance(x, str) and x for x in terms):
        parser.error('terms-file must be a JSON array of nonempty strings')
    result = inspect(root, terms)
    allowlist = args.allowlist or root / 'scripts/archive/scan_allowlist.json'
    allowed = set()
    if allowlist.exists():
        rows = json.loads(allowlist.read_text())
        for row in rows:
            if row.get('kind') not in SECRET_PATTERNS or not row.get('reason'):
                parser.error('allowlist entries require a secret kind and review reason')
            allowed.add((row['path'], row['kind'], row['value_sha256']))
    reviewed, unresolved = [], []
    for finding in result['findings']:
        key = (finding['path'], finding['kind'], finding.get('value_sha256'))
        (reviewed if key in allowed else unresolved).append(finding)
    result['findings'] = unresolved
    result['reviewed_synthetic_matches'] = len(reviewed)
    output = json.dumps(result, indent=2) + '\n'
    if args.report:
        args.report.write_text(output)
        print(json.dumps({k: result[k] for k in ('files', 'bytes', 'scope')}))
        print(f"Review candidates: {len(result['findings'])}; binary files: {len(result['binary_files'])}")
    else:
        print(output, end='')
    return 1 if result['findings'] else 0


if __name__ == '__main__':
    raise SystemExit(main())
