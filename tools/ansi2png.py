#!/usr/bin/env python3
"""Render a trix ANSI snapshot to a PNG (for README screenshots).

This is a documentation helper, not part of the Rust tool.

Usage:
    python3 tools/ansi2png.py snap.ans out.png --size 120x36 --label "rain / neon"
"""
from __future__ import annotations

import argparse
import re
import sys

from PIL import Image, ImageDraw, ImageFont

ESC = re.compile(r"\x1b\[([0-9;]*)m")

BG = (5, 7, 11)
DEFAULT_FG = (200, 205, 215)
ANSI16 = [
    (0, 0, 0), (128, 0, 0), (0, 128, 0), (128, 128, 0),
    (0, 0, 128), (128, 0, 128), (0, 128, 128), (192, 192, 192),
]
ANSI16_BRIGHT = [
    (128, 128, 128), (255, 0, 0), (0, 255, 0), (255, 255, 0),
    (0, 0, 255), (255, 0, 255), (0, 255, 255), (255, 255, 255),
]
PAD = 20
GAP = 6


def parse_ansi(text: str):
    rows = text.split("\n")
    cells = []
    for row in rows:
        line = []
        fg = DEFAULT_FG
        bg = None
        i = 0
        while i < len(row):
            m = ESC.match(row, i)
            if m:
                params = [int(p) if p else 0 for p in m.group(1).split(";")] if m.group(1) else [0]
                j = 0
                while j < len(params):
                    p = params[j]
                    if p == 0:
                        fg = DEFAULT_FG
                        bg = None
                    elif p == 38 and j + 4 < len(params) and params[j + 1] == 2:
                        fg = (params[j + 2], params[j + 3], params[j + 4])
                        j += 4
                    elif p == 48 and j + 4 < len(params) and params[j + 1] == 2:
                        bg = (params[j + 2], params[j + 3], params[j + 4])
                        j += 4
                    elif p == 39:
                        fg = DEFAULT_FG
                    elif 40 <= p <= 47:
                        bg = ANSI16[p - 40]
                    elif 100 <= p <= 107:
                        bg = ANSI16_BRIGHT[p - 100]
                    elif p == 49:
                        bg = None
                    j += 1
                i = m.end()
                continue
            ch = row[i]
            line.append((ch, fg, bg))
            i += 1
        cells.append(line)
    return cells


def load_fonts(size: int):
    ascii_font = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", size, index=0)
    kana_font = ImageFont.truetype("/System/Library/Fonts/ヒラギノ角ゴシック W8.ttc", size, index=0)
    return ascii_font, kana_font


def render(ansi_path: str, out_path: str, size: tuple[int, int] | None, label: str | None):
    text = open(ansi_path, "r", encoding="utf-8", errors="replace").read()
    cells = parse_ansi(text.rstrip("\n"))
    if size:
        w, h = size
        cells = cells[:h]
        while len(cells) < h:
            cells.append([])
        cells = [row[:w] + [(" ", DEFAULT_FG, None)] * (w - len(row[:w])) for row in cells]
    else:
        h = len(cells)
        w = max((len(r) for r in cells), default=0)

    font_size = 20
    ascii_font, kana_font = load_fonts(font_size)
    cell_w = max(1, round(ascii_font.getlength("M")))
    ascent, descent = ascii_font.getmetrics()
    cell_h = ascent + descent
    img_w = PAD * 2 + cell_w * w
    img_h = PAD * 2 + cell_h * h
    if label:
        img_h += 34
    img = Image.new("RGB", (img_w, img_h), BG)
    draw = ImageDraw.Draw(img)

    # subtle frame
    draw.rectangle([2, 2, img_w - 3, img_h - 3], outline=(20, 30, 44))

    for y, row in enumerate(cells):
        for x, (ch, fg, bg) in enumerate(row):
            if bg is not None:
                px = PAD + x * cell_w
                py = PAD + y * cell_h
                draw.rectangle([px, py, px + cell_w - 1, py + cell_h - 1], fill=bg)
            if ch == " ":
                continue
            use_kana = 0x3000 <= ord(ch) or 0xFF61 <= ord(ch) <= 0xFF9F
            font = kana_font if use_kana else ascii_font
            color = fg if isinstance(fg, tuple) else DEFAULT_FG
            px = PAD + x * cell_w
            py = PAD + y * cell_h
            if use_kana:
                # Half-width katakana in a CJK font can be wider than the
                # Menlo cell; centre and shrink it to fit.
                bbox = draw.textbbox((0, 0), ch, font=font)
                gw = bbox[2] - bbox[0]
                gh = bbox[3] - bbox[1]
                if gw > cell_w or gh > cell_h:
                    scale = min(cell_w / max(gw, 1), cell_h / max(gh, 1), 1.0)
                    if scale < 1.0:
                        f2 = kana_font.font_variant(size=max(8, int(font_size * scale)))
                        bbox = draw.textbbox((0, 0), ch, font=f2)
                        gw = bbox[2] - bbox[0]
                        gh = bbox[3] - bbox[1]
                        draw.text(
                            (px + (cell_w - gw) / 2 - bbox[0], py + (cell_h - gh) / 2 - bbox[1]),
                            ch,
                            font=f2,
                            fill=color,
                        )
                        continue
                draw.text(
                    (px + (cell_w - gw) / 2 - bbox[0], py + (cell_h - gh) / 2 - bbox[1]),
                    ch,
                    font=font,
                    fill=color,
                )
            else:
                draw.text((px, py - 1), ch, font=font, fill=color)

    if label:
        small = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", 14, index=0)
        draw.text((PAD, img_h - 26), label, font=small, fill=(90, 120, 150))
        draw.text(
            (img_w - PAD - 120, img_h - 26),
            "trix // ascii fx",
            font=small,
            fill=(45, 65, 85),
        )

    img.save(out_path)
    print(f"{out_path}: {img_w}x{img_h}, {w}x{h} cells")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("ansi")
    ap.add_argument("png")
    ap.add_argument("--size", default=None, help="WxH grid, e.g. 120x36")
    ap.add_argument("--label", default=None)
    args = ap.parse_args()
    size = None
    if args.size:
        w, h = args.size.lower().split("x")
        size = (int(w), int(h))
    render(args.ansi, args.png, size, args.label)
    return 0


if __name__ == "__main__":
    sys.exit(main())
