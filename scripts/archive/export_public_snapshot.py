#!/usr/bin/env python3
"""Export exactly the Git index to a NEW directory, without history or remotes.

Review the staged set before invoking. Does not stage, commit, publish, execute
repository code, or overwrite an existing directory. Run the privacy audit on the
result. Old source hashes and historical signatures are not publication checksums.
"""
import argparse
import hashlib
import io
import subprocess
import tarfile
from pathlib import Path, PurePosixPath


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('destination', type=Path)
    args = parser.parse_args()
    root = Path(subprocess.check_output(['git', 'rev-parse', '--show-toplevel'], text=True).strip()).resolve()
    destination = args.destination.absolute()
    if destination.exists() or destination.is_symlink():
        parser.error('destination must not already exist')
    destination = destination.resolve()
    if destination.is_relative_to(root):
        parser.error('destination must be outside the development repository')
    tree = subprocess.check_output(['git', 'write-tree'], cwd=root, text=True).strip()
    archive = subprocess.check_output(['git', 'archive', '--format=tar', tree], cwd=root)
    # Validate the entire tar BEFORE creating any destination files.
    with tarfile.open(fileobj=io.BytesIO(archive)) as source:
        members = source.getmembers()
        for item in members:
            path = PurePosixPath(item.name)
            if path.is_absolute() or '..' in path.parts or '.git' in path.parts or not (item.isfile() or item.isdir()):
                parser.error(f'unsupported archive member: {item.name!r}')
        destination.mkdir(parents=True)
        for item in members:
            path = destination / item.name
            if item.isdir():
                path.mkdir(parents=True, exist_ok=True)
                continue
            path.parent.mkdir(parents=True, exist_ok=True)
            with source.extractfile(item) as data:
                path.write_bytes(data.read())
            path.chmod(0o755 if item.mode & 0o111 else 0o644)
    rows = []
    for path in sorted(destination.rglob('*')):
        if path.is_file() and path != destination / 'ARCHIVE-MANIFEST.sha256':
            rows.append(f'{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.relative_to(destination).as_posix()}')
    (destination / 'ARCHIVE-MANIFEST.sha256').write_text('\n'.join(rows) + '\n')
    print(f'Exported {len(rows)} staged files; index tree {tree}.')
    print('Added ARCHIVE-MANIFEST.sha256 (covers all exported files, excluding itself).')
    print('History and remotes were not exported. Review and audit before publishing.')


if __name__ == '__main__':
    main()
