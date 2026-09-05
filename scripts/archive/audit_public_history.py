#!/usr/bin/env python3
"""Scan all reachable Git history without checking out or executing old code.

Identity terms are optional private JSON outside the repository. This bounded
scan complements the snapshot audit; it cannot establish complete anonymity.
"""
import argparse
import hashlib
import json
import re
import subprocess
from collections import defaultdict
from pathlib import Path

from audit_public_snapshot import SECRET_PATTERNS, FORBIDDEN_PARTS, FORBIDDEN_PREFIXES, FORBIDDEN_SUFFIXES


def git(root, *args):
    return subprocess.check_output(['git', *args], cwd=root)


def audit(root, terms):
    findings, binaries = [], []
    term_re = re.compile('|'.join(re.escape(t) for t in terms), re.I) if terms else None
    secret_res = {kind: re.compile(pattern) for kind, pattern in SECRET_PATTERNS.items()}
    allow_path = root / 'scripts/archive/scan_allowlist.json'
    allowed = set()
    if allow_path.exists():
        for row in json.loads(allow_path.read_text()):
            if row['kind'] not in SECRET_PATTERNS or not row.get('reason'):
                raise ValueError('Invalid exact synthetic-secret allowance')
            allowed.add((row['path'], row['kind'], row['value_sha256']))

    # --no-renames makes each raw record have exactly one following pathname.
    raw = git(root, 'log', '--all', '--format=', '--raw', '--no-abbrev', '--no-renames', '--root', '-m', '-z')
    tokens = iter(raw.split(b'\0'))
    blob_paths = defaultdict(set)
    all_paths = set()
    for token in tokens:
        token = token.lstrip(b'\n')
        if not token:
            continue
        if not token.startswith(b':'):
            raise ValueError('Unexpected raw Git history record')
        fields = token[1:].split()
        path = next(tokens).decode('utf-8')
        all_paths.add(path)
        new_mode, new_oid = fields[1], fields[3].decode()
        if new_mode != b'000000':
            if new_mode not in (b'100644', b'100755'):
                findings.append({'kind':'unsupported-mode','path':path,'mode':new_mode.decode()})
            blob_paths[new_oid].add(path)

    for path in sorted(all_paths):
        p = Path(path)
        if (set(p.parts) & FORBIDDEN_PARTS or p.suffix.lower() in FORBIDDEN_SUFFIXES
                or path.startswith(FORBIDDEN_PREFIXES) or p.name.startswith('.env')
                or p.name in {'.mcp.json','settings.local.json'}):
            findings.append({'kind':'excluded-artifact','path':path})
        if term_re and term_re.search(path):
            findings.append({'kind':'identity-in-path','path_sha256':hashlib.sha256(path.encode()).hexdigest()})

    commits = git(root, 'rev-list', '--all').decode().splitlines()
    objects = [(oid, 'commit') for oid in commits] + [(oid, 'blob') for oid in sorted(blob_paths)]
    batch = subprocess.Popen(['git','cat-file','--batch'], cwd=root, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    scanned_bytes, reviewed = 0, 0
    try:
        for oid, expected_type in objects:
            batch.stdin.write(oid.encode() + b'\n'); batch.stdin.flush()
            header = batch.stdout.readline().split()
            if len(header) != 3 or header[1].decode() != expected_type:
                raise ValueError('Unexpected Git object type')
            size = int(header[2]); data = batch.stdout.read(size)
            if len(data) != size or batch.stdout.read(1) != b'\n':
                raise ValueError('Incomplete Git object')
            scanned_bytes += size
            if size > 50_000_000:
                findings.append({'kind':'oversized-object','object':oid,'bytes':size})
            paths = sorted(blob_paths.get(oid, []))
            try:
                if b'\0' in data:
                    raise UnicodeError
                text = data.decode('utf-8')
            except UnicodeError:
                binaries.append({'object':oid,'paths':paths,'bytes':size,'sha256':hashlib.sha256(data).hexdigest()})
                continue
            if term_re and term_re.search(text):
                findings.append({'kind':'identity','object':oid,'object_type':expected_type,'paths':paths})
            for kind, regex in secret_res.items():
                for match in regex.finditer(text):
                    digest = hashlib.sha256(match.group().encode()).hexdigest()
                    # Every historical path for a shared blob must be reviewed.
                    if paths and all((path,kind,digest) in allowed for path in paths):
                        reviewed += 1
                    else:
                        findings.append({'kind':kind,'object':oid,'paths':paths,'value_sha256':digest})
    finally:
        batch.stdin.close()
        batch.stdout.close()
        batch.wait()
    refs = git(root,'for-each-ref','--format=%(refname)').decode().splitlines()
    remotes = git(root,'remote').decode().splitlines()
    return {'commits':len(commits),'blobs':len(blob_paths),'paths':len(all_paths),'bytes':scanned_bytes,
            'refs':refs,'remotes':remotes,'findings':findings,'binary_objects':binaries,
            'reviewed_synthetic_matches':reviewed,
            'scope':'All refs: changed paths, unique reachable file blobs, and complete commit metadata/messages. Binary payloads require separate review. No product execution.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('repository', type=Path)
    parser.add_argument('--terms-file', type=Path)
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    root = args.repository.resolve(strict=True)
    terms = json.loads(args.terms_file.read_text()) if args.terms_file else []
    if not isinstance(terms,list) or not all(isinstance(t,str) and t for t in terms):
        parser.error('Private terms must be a JSON array of nonempty strings')
    result = audit(root, terms)
    output = json.dumps(result,indent=2) + '\n'
    if args.report:
        args.report.write_text(output)
        print(json.dumps({k:result[k] for k in ('commits','blobs','paths','bytes','refs','remotes')}))
        print(f"Unresolved findings: {len(result['findings'])}; binary objects: {len(result['binary_objects'])}")
    else:
        print(output,end='')
    return 1 if result['findings'] else 0


if __name__ == '__main__':
    raise SystemExit(main())
