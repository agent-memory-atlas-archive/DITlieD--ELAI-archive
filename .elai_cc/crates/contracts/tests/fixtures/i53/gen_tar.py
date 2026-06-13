#!/usr/bin/env python3
"""Generate archive.tar fixture for I-53 contract tests.

Entries:
  dir/           (directory)
  dir/hello.txt  (text file: "hello from archive\n")
  dir/data.bin   (binary: bytes 0x00 0xFF 0xAB)

Also generates malicious.tar with a "../escape.txt" traversal entry for the negative test.
"""
import struct
import os
import sys

TAR_BLOCK = 512

def null_pad(s: bytes, length: int) -> bytes:
    return s[:length].ljust(length, b'\x00')

def make_header(name: bytes, typeflag: bytes, size: int, prefix: bytes = b'') -> bytes:
    h = bytearray(TAR_BLOCK)
    # Name
    h[0:100] = null_pad(name, 100)
    # Mode
    h[100:108] = b'0000644\x00'
    # UID/GID
    h[108:116] = b'0001750\x00'
    h[116:124] = b'0001750\x00'
    # Size (octal, 12 bytes)
    size_str = ('%011o\x00' % size).encode()
    h[124:136] = size_str
    # mtime
    h[136:148] = b'00000000000\x00'
    # Checksum placeholder (8 spaces)
    h[148:156] = b'        '
    # Type flag
    h[156:157] = typeflag
    # Magic + version
    h[257:263] = b'ustar\x00'
    h[263:265] = b'00'
    # Prefix
    h[345:500] = null_pad(prefix, 155)

    # Compute checksum
    cs = sum(h)
    cs_str = ('%06o\x00 ' % cs).encode()
    h[148:156] = cs_str
    return bytes(h)

def pad_to_block(data: bytes) -> bytes:
    rem = len(data) % TAR_BLOCK
    if rem > 0:
        data += b'\x00' * (TAR_BLOCK - rem)
    return data

def build_tar(entries):
    buf = bytearray()
    for name, typeflag, data, prefix in entries:
        h = make_header(name, typeflag, len(data), prefix)
        buf.extend(h)
        if data:
            buf.extend(pad_to_block(data))
    # Two zero blocks = EOF
    buf.extend(b'\x00' * (TAR_BLOCK * 2))
    return bytes(buf)

out_dir = os.path.dirname(os.path.abspath(__file__))

# archive.tar: 3 valid entries
entries = [
    (b'dir/', b'5', b'', b''),
    (b'dir/hello.txt', b'0', b'hello from archive\n', b''),
    (b'dir/data.bin', b'0', bytes([0x00, 0xFF, 0xAB]), b''),
]
tar_data = build_tar(entries)
with open(os.path.join(out_dir, 'archive.tar'), 'wb') as f:
    f.write(tar_data)
print(f"Wrote archive.tar ({len(tar_data)} bytes, {len(entries)} entries)")

# malicious.tar: one traversal entry
mal_entries = [
    (b'../escape.txt', b'0', b'evil content', b''),
]
mal_data = build_tar(mal_entries)
with open(os.path.join(out_dir, 'malicious.tar'), 'wb') as f:
    f.write(mal_data)
print(f"Wrote malicious.tar ({len(mal_data)} bytes, traversal entry)")
