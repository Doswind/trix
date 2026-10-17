#!/usr/bin/env python3
"""Generate the compact land/sea bitmask used by the `map` effect.

Source data: Natural Earth 1:110m land polygons (public domain)
    https://www.naturalearthdata.com/
    https://github.com/nvkelso/natural-earth-vector

The output is src/data/land_720x360.bin: a 720x360 equirectangular bitmap,
one bit per pixel, row-major, MSB first.  1 = land, 0 = water.

Usage:
    python3 tools/gen_land.py            # fetch (cached) + generate
    python3 tools/gen_land.py --preview  # also print an ASCII preview

No third-party Python dependencies are required.
"""
from __future__ import annotations

import argparse
import json
import math
import os
import sys
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "tools", "cache", "ne_110m_land.geojson")
OUT = os.path.join(ROOT, "src", "data", "land_720x360.bin")
URL = (
    "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/"
    "master/geojson/ne_110m_land.geojson"
)

W, H = 720, 360          # stored bitmap
SS = 2                   # supersampling factor used while rasterising


def fetch() -> None:
    if os.path.exists(CACHE) and os.path.getsize(CACHE) > 10_000:
        return
    os.makedirs(os.path.dirname(CACHE), exist_ok=True)
    print(f"downloading {URL}")
    with urllib.request.urlopen(URL, timeout=60) as r:
        data = r.read()
    with open(CACHE, "wb") as f:
        f.write(data)
    print(f"cached {len(data)} bytes -> {CACHE}")


def rings_of(geojson: dict):
    """Yield every linear ring in the collection (outer rings and holes).

    Even-odd scanline filling over *all* rings handles holes and overlapping
    polygons without needing ring winding conventions.
    """
    for feature in geojson["features"]:
        geom = feature["geometry"]
        if geom["type"] == "Polygon":
            polys = [geom["coordinates"]]
        elif geom["type"] == "MultiPolygon":
            polys = geom["coordinates"]
        else:
            continue
        for poly in polys:
            for ring in poly:
                yield ring


def rasterize(geojson: dict, width: int, height: int) -> list[list[bool]]:
    rings = []
    for ring in rings_of(geojson):
        # Drop the duplicated closing point so edge iteration is clean.
        pts = [(float(p[0]), float(p[1])) for p in ring]
        if len(pts) > 1 and pts[0] == pts[-1]:
            pts.pop()
        if len(pts) >= 3:
            rings.append(pts)

    rows: list[list[bool]] = []
    for y in range(height):
        lat = 90.0 - (y + 0.5) * 180.0 / height
        xs = []
        for pts in rings:
            n = len(pts)
            for i in range(n):
                x1, y1 = pts[i]
                x2, y2 = pts[(i + 1) % n]
                if (y1 > lat) != (y2 > lat):
                    x = x1 + (lat - y1) * (x2 - x1) / (y2 - y1)
                    xs.append(x)
        xs.sort()
        row = [False] * width
        for i in range(0, len(xs) - 1, 2):
            xa, xb = xs[i], xs[i + 1]
            if xb <= xa:
                continue
            # pixel centres: lon = -180 + (x + 0.5) * 360 / width
            ia = max(0, int(math.ceil((xa + 180.0) * width / 360.0 - 0.5)))
            ib = min(width - 1, int(math.floor((xb + 180.0) * width / 360.0 - 0.5)))
            for x in range(ia, ib + 1):
                row[x] = True
        rows.append(row)
    return rows


def downsample(rows: list[list[bool]], sw: int, sh: int, factor: int):
    out = []
    for y in range(sh):
        row = []
        for x in range(sw):
            count = 0
            for dy in range(factor):
                for dx in range(factor):
                    if rows[y * factor + dy][x * factor + dx]:
                        count += 1
            row.append(count * 2 >= factor * factor)
        out.append(row)
    return out


def pack(rows: list[list[bool]], width: int, height: int) -> bytes:
    data = bytearray((width * height + 7) // 8)
    i = 0
    for y in range(height):
        for x in range(width):
            if rows[y][x]:
                data[i >> 3] |= 1 << (7 - (i & 7))
            i += 1
    return bytes(data)


def preview(rows: list[list[bool]], width: int, height: int) -> None:
    ramp = " .:-=+*#%@"
    out_w = 120
    out_h = 34
    for oy in range(out_h):
        line = []
        for ox in range(out_w):
            x0 = ox * width // out_w
            x1 = max(x0 + 1, (ox + 1) * width // out_w)
            y0 = oy * height // out_h
            y1 = max(y0 + 1, (oy + 1) * height // out_h)
            n = 0
            t = 0
            for y in range(y0, y1):
                for x in range(x0, x1):
                    n += rows[y][x]
                    t += 1
            v = n / t
            line.append(ramp[min(len(ramp) - 1, int(v * len(ramp)))])
        print("".join(line))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--preview", action="store_true")
    args = ap.parse_args()
    fetch()
    with open(CACHE) as f:
        geojson = json.load(f)
    print(f"rasterising at {W * SS}x{H * SS} ...")
    ss_rows = rasterize(geojson, W * SS, H * SS)
    rows = downsample(ss_rows, W, H, SS)
    data = pack(rows, W, H)
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "wb") as f:
        f.write(data)
    land = sum(sum(r) for r in rows)
    print(f"wrote {OUT}: {len(data)} bytes, land coverage {land / (W * H):.1%}")
    if args.preview:
        preview(rows, W, H)
    return 0


if __name__ == "__main__":
    sys.exit(main())
