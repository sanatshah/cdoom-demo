#!/usr/bin/env python3
"""Build a vanilla Doom-format PWAD that replaces E1M1.

Geometry is four convex rectangles (start, door, hall, arena) so the
BSP, REJECT, and BLOCKMAP lumps can be emitted without an external
node builder. Load with:

    ./run.sh -file wads/custom.wad -warp 1 1 -skill 3
"""

from __future__ import annotations

import argparse
import math
import struct
from pathlib import Path

ML_BLOCKING = 1
ML_TWOSIDED = 4
ML_DONTPEGTOP = 8
NF_SUBSECTOR = 0x8000
BLOCK_SIZE = 128
THING_ALL_SKILLS = 7

# Doom thing types
PLAYER1 = 1
SHOTGUN = 2001
CLIP = 2007
STIMPACK = 2011
HEALTH_BONUS = 2014
ARMOR_BONUS = 2015
IMP = 3001
ZOMBIEMAN = 3004
SHOTGUN_GUY = 9
BARREL = 2035


def nam8(s: str) -> bytes:
    b = s.encode("ascii")
    if len(b) > 8:
        raise ValueError(s)
    return b.ljust(8, b"\0")


def i16(n: int) -> bytes:
    return struct.pack("<h", n)


def u16(n: int) -> bytes:
    return struct.pack("<H", n)


def i32(n: int) -> bytes:
    return struct.pack("<i", n)


def bam_angle(x1: int, y1: int, x2: int, y2: int) -> int:
    ang = math.atan2(y2 - y1, x2 - x1)
    return int(ang * 0x8000 / math.pi) & 0xFFFF


def dist(x1: int, y1: int, x2: int, y2: int) -> int:
    return int(round(math.hypot(x2 - x1, y2 - y1)))


class MapBuilder:
    def __init__(self) -> None:
        self.verts: dict[tuple[int, int], int] = {}
        self.vert_list: list[tuple[int, int]] = []
        self.sectors: list[bytes] = []
        self.sector_aabb: list[tuple[int, int, int, int]] = []
        self.sidedefs: list[bytes] = []
        self.linedefs: list[bytes] = []
        self.line_meta: list[dict] = []
        self.things: list[bytes] = []

    def v(self, x: int, y: int) -> int:
        key = (x, y)
        if key not in self.verts:
            self.verts[key] = len(self.vert_list)
            self.vert_list.append(key)
        return self.verts[key]

    def sector(
        self,
        floor: int,
        ceil: int,
        floorpic: str,
        ceilpic: str,
        light: int,
        *,
        aabb: tuple[int, int, int, int],
        special: int = 0,
        tag: int = 0,
    ) -> int:
        self.sectors.append(
            i16(floor)
            + i16(ceil)
            + nam8(floorpic)
            + nam8(ceilpic)
            + i16(light)
            + i16(special)
            + i16(tag)
        )
        self.sector_aabb.append(aabb)
        return len(self.sectors) - 1

    def side(
        self,
        sector: int,
        mid: str = "-",
        top: str = "-",
        bottom: str = "-",
        xoff: int = 0,
        yoff: int = 0,
    ) -> int:
        self.sidedefs.append(
            i16(xoff)
            + i16(yoff)
            + nam8(top)
            + nam8(bottom)
            + nam8(mid)
            + i16(sector)
        )
        return len(self.sidedefs) - 1

    def line(
        self,
        x1: int,
        y1: int,
        x2: int,
        y2: int,
        flags: int,
        front_sec: int,
        *,
        back_sec: int | None = None,
        special: int = 0,
        tag: int = 0,
        front_mid: str = "-",
        front_top: str = "-",
        front_bot: str = "-",
        back_mid: str = "-",
        back_top: str = "-",
        back_bot: str = "-",
    ) -> None:
        v1 = self.v(x1, y1)
        v2 = self.v(x2, y2)
        front = self.side(front_sec, mid=front_mid, top=front_top, bottom=front_bot)
        back = -1
        if back_sec is not None:
            back = self.side(back_sec, mid=back_mid, top=back_top, bottom=back_bot)
        self.linedefs.append(
            i16(v1)
            + i16(v2)
            + i16(flags)
            + i16(special)
            + i16(tag)
            + i16(front)
            + i16(back)
        )
        self.line_meta.append(
            {
                "v1": v1,
                "v2": v2,
                "x1": x1,
                "y1": y1,
                "x2": x2,
                "y2": y2,
                "front": front_sec,
                "back": back_sec,
            }
        )

    def wall(self, x1: int, y1: int, x2: int, y2: int, sec: int, tex: str, special: int = 0) -> None:
        self.line(
            x1,
            y1,
            x2,
            y2,
            ML_BLOCKING,
            sec,
            special=special,
            front_mid=tex,
        )

    def thing(self, x: int, y: int, angle: int, typ: int, flags: int = THING_ALL_SKILLS) -> None:
        self.things.append(i16(x) + i16(y) + i16(angle) + i16(typ) + i16(flags))


def build_geometry() -> MapBuilder:
    m = MapBuilder()
    start = m.sector(0, 128, "FLOOR4_8", "CEIL3_5", 160, aabb=(0, 0, 640, 384))
    door = m.sector(0, 0, "FLOOR4_8", "CEIL3_5", 144, aabb=(256, 384, 384, 400))
    hall = m.sector(0, 128, "FLAT5_4", "CEIL3_5", 120, aabb=(256, 400, 384, 640))
    arena = m.sector(0, 256, "GRASS1", "F_SKY1", 192, aabb=(0, 640, 768, 1152))

    # Start room — interior on the right.
    m.wall(0, 0, 0, 384, start, "STARTAN2")
    m.wall(0, 384, 256, 384, start, "STARTAN2")
    m.line(
        256,
        384,
        384,
        384,
        ML_BLOCKING | ML_TWOSIDED | ML_DONTPEGTOP,
        start,
        back_sec=door,
        special=1,
        front_top="BIGDOOR2",
        back_top="BIGDOOR2",
    )
    m.wall(384, 384, 640, 384, start, "STARTAN2")
    m.wall(640, 384, 640, 0, start, "STARTAN2")
    m.wall(640, 0, 0, 0, start, "STARTAN2")

    # Door tracks + north leaf.
    m.wall(256, 384, 256, 400, door, "DOORTRAK")
    m.wall(384, 400, 384, 384, door, "DOORTRAK")
    m.line(
        256,
        400,
        384,
        400,
        ML_BLOCKING | ML_TWOSIDED | ML_DONTPEGTOP,
        door,
        back_sec=hall,
        special=1,
        front_top="BIGDOOR2",
        back_top="BIGDOOR2",
    )

    # Hall.
    m.wall(256, 400, 256, 640, hall, "BROWN1")
    m.wall(384, 640, 384, 400, hall, "BROWN1")
    m.line(
        256,
        640,
        384,
        640,
        ML_TWOSIDED,
        hall,
        back_sec=arena,
    )

    # Arena.
    m.wall(256, 640, 0, 640, arena, "STONE2")
    m.wall(0, 640, 0, 1152, arena, "STONE2")
    m.wall(0, 1152, 320, 1152, arena, "STONE2")
    m.wall(320, 1152, 384, 1152, arena, "SW1EXIT", special=11)
    m.wall(384, 1152, 512, 1152, arena, "EXITSIGN")
    m.wall(512, 1152, 768, 1152, arena, "STONE2")
    m.wall(768, 1152, 768, 640, arena, "STONE2")
    m.wall(768, 640, 384, 640, arena, "STONE2")

    m.thing(320, 192, 90, PLAYER1)
    m.thing(96, 96, 0, SHOTGUN)
    m.thing(96, 128, 0, CLIP)
    m.thing(544, 96, 0, STIMPACK)
    m.thing(320, 480, 90, HEALTH_BONUS)
    m.thing(320, 520, 90, ARMOR_BONUS)
    m.thing(96, 800, 0, ZOMBIEMAN)
    m.thing(160, 960, 180, ZOMBIEMAN)
    m.thing(672, 800, 180, SHOTGUN_GUY)
    m.thing(600, 1000, 225, IMP)
    m.thing(120, 1080, 270, IMP)
    m.thing(480, 880, 0, BARREL)
    m.thing(420, 1088, 90, HEALTH_BONUS)
    m.thing(448, 1088, 90, ARMOR_BONUS)
    return m


def build_segs(m: MapBuilder) -> tuple[bytes, bytes]:
    segs = bytearray()
    ssectors = bytearray()
    first = 0
    for sec in range(len(m.sectors)):
        count = 0
        for meta in m.line_meta:
            if meta["front"] == sec:
                x1, y1, x2, y2 = meta["x1"], meta["y1"], meta["x2"], meta["y2"]
                segs += (
                    i16(meta["v1"])
                    + i16(meta["v2"])
                    + u16(bam_angle(x1, y1, x2, y2))
                    + i16(m.line_meta.index(meta))
                    + i16(0)
                    + i16(0)
                )
                count += 1
            elif meta["back"] == sec:
                x1, y1, x2, y2 = meta["x2"], meta["y2"], meta["x1"], meta["y1"]
                segs += (
                    i16(meta["v2"])
                    + i16(meta["v1"])
                    + u16(bam_angle(x1, y1, x2, y2))
                    + i16(m.line_meta.index(meta))
                    + i16(1)
                    + i16(dist(meta["x1"], meta["y1"], meta["x2"], meta["y2"]))
                )
                count += 1
        if count == 0:
            raise RuntimeError(f"sector {sec} has no segs")
        ssectors += i16(count) + i16(first)
        first += count
    return bytes(segs), bytes(ssectors)


def union_aabb(ids: list[int], m: MapBuilder) -> tuple[int, int, int, int]:
    minx = min(m.sector_aabb[i][0] for i in ids)
    miny = min(m.sector_aabb[i][1] for i in ids)
    maxx = max(m.sector_aabb[i][2] for i in ids)
    maxy = max(m.sector_aabb[i][3] for i in ids)
    return minx, miny, maxx, maxy


def bbox_bytes(aabb: tuple[int, int, int, int]) -> bytes:
    minx, miny, maxx, maxy = aabb
    return i16(maxy) + i16(miny) + i16(minx) + i16(maxx)


def build_nodes_postorder(m: MapBuilder) -> bytes:
    """Build a chain of Y-splits; root is the last node."""
    # node 0: y=640 hall vs arena
    # node 1: y=400 door vs (hall+arena via node 0)
    # node 2 (root): y=384 start vs the rest via node 1
    hall_aabb = union_aabb([2], m)
    arena_aabb = union_aabb([3], m)
    door_aabb = union_aabb([1], m)
    start_aabb = union_aabb([0], m)
    rest_after_door = union_aabb([2, 3], m)
    rest_after_start = union_aabb([1, 2, 3], m)

    def node(y: int, c0: int, c1: int, b0: tuple[int, int, int, int], b1: tuple[int, int, int, int]) -> bytes:
        return (
            i16(0)
            + i16(y)
            + i16(128)
            + i16(0)
            + bbox_bytes(b0)
            + bbox_bytes(b1)
            + u16(c0)
            + u16(c1)
        )

    n0 = node(640, NF_SUBSECTOR | 2, NF_SUBSECTOR | 3, hall_aabb, arena_aabb)
    n1 = node(400, NF_SUBSECTOR | 1, 0, door_aabb, rest_after_door)
    n2 = node(384, NF_SUBSECTOR | 0, 1, start_aabb, rest_after_start)
    return n0 + n1 + n2


def build_reject(nsectors: int) -> bytes:
    return bytes((nsectors * nsectors + 7) // 8)


def line_aabb_overlap(
    x1: int, y1: int, x2: int, y2: int, bx: int, by: int, size: int
) -> bool:
    minx, maxx = (x1, x2) if x1 <= x2 else (x2, x1)
    miny, maxy = (y1, y2) if y1 <= y2 else (y2, y1)
    return not (maxx < bx or minx > bx + size or maxy < by or miny > by + size)


def build_blockmap(m: MapBuilder) -> bytes:
    xs = [p[0] for p in m.vert_list]
    ys = [p[1] for p in m.vert_list]
    orgx = min(xs) - 8
    orgy = min(ys) - 8
    maxx = max(xs) + 8
    maxy = max(ys) + 8
    cols = (maxx - orgx) // BLOCK_SIZE + 1
    rows = (maxy - orgy) // BLOCK_SIZE + 1
    lists: list[list[int]] = []
    for row in range(rows):
        for col in range(cols):
            bx = orgx + col * BLOCK_SIZE
            by = orgy + row * BLOCK_SIZE
            hits = [0]
            for i, meta in enumerate(m.line_meta):
                if line_aabb_overlap(meta["x1"], meta["y1"], meta["x2"], meta["y2"], bx, by, BLOCK_SIZE):
                    hits.append(i)
            hits.append(0xFFFF)
            lists.append(hits)
    header_words = 4 + cols * rows
    offsets = []
    blob = bytearray()
    cursor = header_words
    for lst in lists:
        offsets.append(cursor)
        for w in lst:
            blob += u16(w)
        cursor += len(lst)
    out = i16(orgx) + i16(orgy) + i16(cols) + i16(rows)
    out += b"".join(u16(o) for o in offsets)
    out += bytes(blob)
    return out


def pack_lump(name: str, data: bytes) -> tuple[str, bytes]:
    return name, data


def write_wad(path: Path, lumps: list[tuple[str, bytes]]) -> None:
    table = bytearray()
    blob = bytearray()
    for name, data in lumps:
        table += i32(12 + len(blob)) + i32(len(data)) + nam8(name)
        blob += data
    header = b"PWAD" + i32(len(lumps)) + i32(12 + len(blob))
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(header + bytes(blob) + bytes(table))


def make_wad(path: Path) -> dict[str, int]:
    m = build_geometry()
    segs, ssectors = build_segs(m)
    nodes = build_nodes_postorder(m)
    reject = build_reject(len(m.sectors))
    blockmap = build_blockmap(m)
    lumps = [
        pack_lump("E1M1", b""),
        pack_lump("THINGS", b"".join(m.things)),
        pack_lump("LINEDEFS", b"".join(m.linedefs)),
        pack_lump("SIDEDEFS", b"".join(m.sidedefs)),
        pack_lump("VERTEXES", b"".join(i16(x) + i16(y) for x, y in m.vert_list)),
        pack_lump("SEGS", segs),
        pack_lump("SSECTORS", ssectors),
        pack_lump("NODES", nodes),
        pack_lump("SECTORS", b"".join(m.sectors)),
        pack_lump("REJECT", reject),
        pack_lump("BLOCKMAP", blockmap),
    ]
    write_wad(path, lumps)
    return {
        "things": len(m.things),
        "linedefs": len(m.linedefs),
        "sidedefs": len(m.sidedefs),
        "vertexes": len(m.vert_list),
        "sectors": len(m.sectors),
        "bytes": path.stat().st_size,
    }


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        default=root / "wads" / "custom.wad",
    )
    args = parser.parse_args()
    stats = make_wad(args.output)
    print(f"wrote {args.output} ({stats['bytes']} bytes)")
    print(
        "  {things} things, {linedefs} linedefs, {sectors} sectors, {vertexes} verts".format(
            **stats
        )
    )


if __name__ == "__main__":
    main()
