#!/usr/bin/env python3
"""Sanity-check the custom E1M1 PWAD lump layout and player start."""

from __future__ import annotations

import struct
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from make_custom_map import make_wad  # noqa: E402

REQUIRED = [
    "E1M1",
    "THINGS",
    "LINEDEFS",
    "SIDEDEFS",
    "VERTEXES",
    "SEGS",
    "SSECTORS",
    "NODES",
    "SECTORS",
    "REJECT",
    "BLOCKMAP",
]


def lumps(data: bytes) -> list[tuple[str, int, int]]:
    ident, n, ofs = struct.unpack_from("<4sii", data, 0)
    if ident != b"PWAD":
        raise AssertionError(ident)
    out = []
    for i in range(n):
        pos, size, name = struct.unpack_from("<ii8s", data, ofs + i * 16)
        out.append((name.split(b"\0")[0].decode(), pos, size))
    return out


def main() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "custom.wad"
        stats = make_wad(path)
        data = path.read_bytes()
    names = [n for n, _, _ in lumps(data)]
    assert names == REQUIRED, names
    things = next(t for t in lumps(data) if t[0] == "THINGS")
    x, y, ang, typ, flags = struct.unpack_from("<hhhhh", data, things[1])
    assert typ == 1, typ
    assert (x, y, ang) == (320, 192, 90)
    assert flags == 7
    assert stats["sectors"] == 4
    assert stats["linedefs"] == 20
    committed = ROOT / "wads" / "custom.wad"
    if committed.exists():
        # Rebuild must stay bit-identical so the committed WAD cannot drift.
        assert committed.read_bytes() == data, "wads/custom.wad is stale; rerun tools/make_custom_map.py"
    print("custom map wad ok")


if __name__ == "__main__":
    main()
