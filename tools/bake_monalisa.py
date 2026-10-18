#!/usr/bin/env python3
"""Bake the Mona Lisa figure for the `monalisa` effect (A 方案).

The reference mosaic is cropped to the figure, cut out from the landscape,
de-grouted, colour graded and resampled to a 148x134 cell grid.  The runtime
draws each cell as an ANSI background colour plus one ASCII glyph; there is no
scene background and no image decoder in the Rust binary.

Output:
    src/data/monalisa.bin   header + 148x134 premultiplied RGBA cells
    --preview out.png       optional 66x60 colour-cell preview

Usage:
    python3 tools/bake_monalisa.py --preview docs/proposals/monalisa-v2/a-preview.png
"""
from __future__ import annotations

import argparse
import os
import struct

import numpy as np
from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REF = os.path.join(ROOT, "docs/proposals/monalisa-v2/ref.webp")
OUT = os.path.join(ROOT, "src/data/monalisa.bin")

# Figure crop in the 1280x1920 reference (watermark excluded).
BOX = (200, 110, 1140, 1800)

# Hand-traced silhouette.  The upper-left shoulder is kept tight so the grey
# mountain behind the arm does not leak into the cut-out.
POLY = [
    (610, 158), (640, 145), (690, 158), (735, 195), (775, 255), (805, 330),
    (825, 420), (832, 510), (820, 590), (790, 665), (755, 715), (850, 735),
    (950, 770), (1030, 830), (1080, 910), (1105, 1010), (1110, 1120),
    (1090, 1260), (1060, 1400), (1035, 1530), (1000, 1635), (950, 1700),
    (900, 1745), (840, 1748), (780, 1700), (710, 1645), (640, 1610),
    (560, 1590), (485, 1560), (420, 1530), (370, 1480), (330, 1400),
    (305, 1300), (295, 1180), (290, 1060), (350, 950), (380, 850),
    (405, 760), (420, 675), (402, 580), (402, 490), (415, 400), (440, 315),
    (470, 240), (515, 190), (560, 165),
]

SRC_W, SRC_H = 148, 134
ALPHA_CUT = 0.42


def build() -> tuple[Image.Image, Image.Image]:
    im = Image.open(REF).convert("RGB")
    crop = im.crop(BOX)
    cw, ch = crop.size

    mask = Image.new("L", (cw, ch), 0)
    ImageDraw.Draw(mask).polygon(
        [(x - BOX[0], y - BOX[1]) for x, y in POLY], fill=255
    )

    arr = np.asarray(crop).astype(np.float32)
    mx = arr.max(axis=2)
    mn = arr.min(axis=2)
    sat = (mx - mn) / np.maximum(mx, 1)
    lum = (0.2126 * arr[:, :, 0] + 0.7152 * arr[:, :, 1] + 0.0722 * arr[:, :, 2]) / 255

    # Bright, desaturated sky / haze is not part of the figure.
    m = np.asarray(mask).copy()
    m[(lum > 0.52) & (sat < 0.14)] = 0
    mask = (
        Image.fromarray(m)
        .filter(ImageFilter.MaxFilter(7))
        .filter(ImageFilter.MinFilter(7))
        .filter(ImageFilter.GaussianBlur(3))
    )

    # The reference is a mosaic with white grout.  A median filter removes the
    # grout while keeping eyes, lips and finger creases.
    clean = crop.filter(ImageFilter.MedianFilter(5)).filter(
        ImageFilter.GaussianBlur(2.4)
    )
    a = np.asarray(clean).astype(np.float32) / 255
    a = np.clip(a * 1.08 + 0.008, 0, 1)
    clean = Image.fromarray((a * 255).astype(np.uint8))

    # Premultiply at full resolution so edge cells fade into the dark terminal
    # instead of carrying landscape colour.
    mf = np.asarray(mask).astype(np.float32)[:, :, None] / 255
    rgba = np.dstack([np.asarray(clean).astype(np.float32) * mf, mf * 255])
    figure = Image.fromarray(np.clip(rgba, 0, 255).astype(np.uint8), "RGBA")

    color = figure.convert("RGB").resize((SRC_W, SRC_H), Image.LANCZOS)
    alpha = figure.getchannel("A").resize((SRC_W, SRC_H), Image.LANCZOS)
    return color, alpha


def pack(color: Image.Image, alpha: Image.Image) -> None:
    rgb = np.asarray(color).astype(np.uint8)
    al = np.asarray(alpha).astype(np.uint8)
    # Drop very transparent cells entirely; the runtime also checks this, but
    # zeroing makes the preview and diagnostics unambiguous.
    al = np.where(al >= int(ALPHA_CUT * 255), al, 0).astype(np.uint8)
    # `color` is already premultiplied by build(); do not multiply twice.
    with open(OUT, "wb") as fh:
        fh.write(b"MONA")
        fh.write(struct.pack("<B3xHH", 1, SRC_W, SRC_H))
        for y in range(SRC_H):
            for x in range(SRC_W):
                r, g, b = rgb[y, x]
                fh.write(bytes((int(r), int(g), int(b), int(al[y, x]))))
    print(f"wrote {OUT} ({SRC_W}x{SRC_H}, {os.path.getsize(OUT)} bytes)")


RAMP = " .:-=+*#%@"


def preview(color: Image.Image, alpha: Image.Image, path: str, cols: int = 66, rows: int = 60) -> None:
    palette = (0, 0, 0)
    font = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", 20)
    cw = max(1, round(font.getlength("M")))
    asc, desc = font.getmetrics()
    chh = asc + desc

    rgb = np.asarray(color).astype(np.float32)
    al = np.asarray(alpha).astype(np.float32) / 255
    sw, sh = color.size
    img = Image.new("RGB", (cols * cw, rows * chh), palette)
    dr = ImageDraw.Draw(img)
    for ty in range(rows):
        y0, y1 = ty * sh // rows, (ty + 1) * sh // rows
        for tx in range(cols):
            x0, x1 = tx * sw // cols, (tx + 1) * sw // cols
            block = rgb[y0:y1, x0:x1]
            a = al[y0:y1, x0:x1]
            if float(a.mean()) < ALPHA_CUT:
                continue
            c = block.mean(axis=(0, 1))
            r, g, b = [int(min(255, v * 1.12 + 4)) for v in c]
            lum = (0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]) / 255
            ch = RAMP[min(len(RAMP) - 1, int(lum * (len(RAMP) - 1) + 0.5))]
            px, py = tx * cw, ty * chh
            dr.rectangle([px, py, px + cw - 1, py + chh - 1], fill=tuple(int(v) for v in c))
            if ch != " ":
                dr.text((px, py - 1), ch, font=font, fill=(r, g, b))
    img.save(path)
    print(f"preview {path} ({cols}x{rows})")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--preview", default=None)
    args = ap.parse_args()
    color, alpha = build()
    pack(color, alpha)
    if args.preview:
        preview(color, alpha, args.preview)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
