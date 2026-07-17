"""Minimal msgpack decoder for SSB payloads.

Handles only the subset emitted by the ELAI SSB store (rmp_serde named maps):
fixmap, map16, map32, fixstr, str8, str16, str32, fixint, uint8/16/32/64,
int8/16/32/64, nil, true, false, bin8/16/32, array variants.
Sufficient for CitationAnchorsBoundPayload and other simple SSB payloads.
"""

from __future__ import annotations
import struct
from typing import Any

_NIL = 0xC0
_TRUE = 0xC3
_FALSE = 0xC2
_BIN8 = 0xC4
_BIN16 = 0xC5
_BIN32 = 0xC6
_UINT8 = 0xCC
_UINT16 = 0xCD
_UINT32 = 0xCE
_UINT64 = 0xCF
_INT8 = 0xD0
_INT16 = 0xD1
_INT32 = 0xD2
_INT64 = 0xD3
_STR8 = 0xD9
_STR16 = 0xDA
_STR32 = 0xDB
_FLOAT32 = 0xCA
_FLOAT64 = 0xCB


def _read_str(data: bytes, off: int, n: int) -> tuple[str, int]:
    end = off + n
    if end > len(data):
        raise ValueError("msgpack: truncated string")
    return data[off:end].decode("utf-8"), end


def _read_bytes(data: bytes, off: int, n: int) -> tuple[bytes, int]:
    end = off + n
    if end > len(data):
        raise ValueError("msgpack: truncated bytes")
    return data[off:end], end


def decode(data: bytes) -> Any:
    val, off = _decode_one(data, 0)
    if off != len(data):
        raise ValueError(f"msgpack: trailing bytes at offset {off}/{len(data)}")
    return val


def _decode_one(data: bytes, off: int) -> tuple[Any, int]:
    if off >= len(data):
        raise ValueError("msgpack: unexpected end of data")
    b = data[off]
    off += 1

    if b <= 0x7F:
        return b, off
    if b >= 0xE0:
        return b - 0x100, off
    if 0x80 <= b <= 0x8F:
        return _decode_map(data, off, b & 0x0F)
    if 0x90 <= b <= 0x9F:
        return _decode_array(data, off, b & 0x0F)
    if 0xA0 <= b <= 0xBF:
        return _read_str(data, off, b & 0x1F)

    if b == _NIL:
        return None, off
    if b == _TRUE:
        return True, off
    if b == _FALSE:
        return False, off

    if b == _FLOAT32:
        v = struct.unpack_from(">f", data, off)[0]
        return v, off + 4
    if b == _FLOAT64:
        v = struct.unpack_from(">d", data, off)[0]
        return v, off + 8

    if b == _BIN8:
        n = data[off]; off += 1
        return _read_bytes(data, off, n)
    if b == _BIN16:
        n = int.from_bytes(data[off:off + 2], "big"); off += 2
        return _read_bytes(data, off, n)
    if b == _BIN32:
        n = int.from_bytes(data[off:off + 4], "big"); off += 4
        return _read_bytes(data, off, n)

    if b == _UINT8:
        return data[off], off + 1
    if b == _UINT16:
        return int.from_bytes(data[off:off + 2], "big"), off + 2
    if b == _UINT32:
        return int.from_bytes(data[off:off + 4], "big"), off + 4
    if b == _UINT64:
        return int.from_bytes(data[off:off + 8], "big"), off + 8

    if b == _INT8:
        v = data[off]
        return v - 0x100 if v >= 0x80 else v, off + 1
    if b == _INT16:
        v = int.from_bytes(data[off:off + 2], "big", signed=True)
        return v, off + 2
    if b == _INT32:
        v = int.from_bytes(data[off:off + 4], "big", signed=True)
        return v, off + 4
    if b == _INT64:
        v = int.from_bytes(data[off:off + 8], "big", signed=True)
        return v, off + 8

    if b == _STR8:
        n = data[off]; off += 1
        return _read_str(data, off, n)
    if b == _STR16:
        n = int.from_bytes(data[off:off + 2], "big"); off += 2
        return _read_str(data, off, n)
    if b == _STR32:
        n = int.from_bytes(data[off:off + 4], "big"); off += 4
        return _read_str(data, off, n)

    if b == 0xDE:
        n = int.from_bytes(data[off:off + 2], "big"); off += 2
        return _decode_map(data, off, n)
    if b == 0xDF:
        n = int.from_bytes(data[off:off + 4], "big"); off += 4
        return _decode_map(data, off, n)
    if b == 0xDC:
        n = int.from_bytes(data[off:off + 2], "big"); off += 2
        return _decode_array(data, off, n)
    if b == 0xDD:
        n = int.from_bytes(data[off:off + 4], "big"); off += 4
        return _decode_array(data, off, n)

    raise ValueError(f"msgpack: unsupported type byte 0x{b:02x} at offset {off - 1}")


def _decode_map(data: bytes, off: int, n: int) -> tuple[dict, int]:
    result: dict = {}
    for _ in range(n):
        key, off = _decode_one(data, off)
        val, off = _decode_one(data, off)
        result[key] = val
    return result, off


def _decode_array(data: bytes, off: int, n: int) -> tuple[list, int]:
    result: list = []
    for _ in range(n):
        val, off = _decode_one(data, off)
        result.append(val)
    return result, off
