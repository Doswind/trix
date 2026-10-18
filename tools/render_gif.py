#!/usr/bin/env python3
"""Render an animated GIF from deterministic trix snapshots.

Documentation helper, not part of the Rust binary.

Usage:
    python3 tools/render_gif.py fire docs/fire.gif --start 60 --frames 30 --step 2
    python3 tools/render_gif.py dragon docs/dragon.gif --start 100 --frames 30 --step 2
    python3 tools/render_gif.py demoneye docs/demoneye.gif --start 10 --frames 24 \
        --step 4 --size 88x26 --width 520 --unicode
"""
from __future__ import annotations

import argparse
import os
import subprocess
import sys
import tempfile

from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ansi2png  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BIN = os.path.join(ROOT, "target", "release", "trix")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("effect")
    ap.add_argument("out")
    ap.add_argument("--start", type=int, default=60)
    ap.add_argument("--frames", type=int, default=30)
    ap.add_argument("--step", type=int, default=2)
    ap.add_argument("--size", default="80x24")
    ap.add_argument("--width", type=int, default=800)
    ap.add_argument("--duration", type=int, default=66)
    ap.add_argument(
        "--unicode",
        action="store_true",
        help="render the Unicode charset (only for a Unicode-specific doc; the binary defaults to ASCII)",
    )
    ap.add_argument(
        "--ghost",
        action="store_true",
        help="render the monalisa pure-ASCII ghost mode",
    )
    args = ap.parse_args()
    w, h = map(int, args.size.lower().split("x"))
    images = []
    with tempfile.TemporaryDirectory() as td:
        for i in range(args.frames):
            frame = args.start + i * args.step
            ans = os.path.join(td, f"f{i}.ans")
            png = os.path.join(td, f"f{i}.png")
            cmd = [
                BIN,
                args.effect,
                "--snapshot",
                ans,
                "--size",
                f"{w}x{h}",
                "--frames",
                str(frame),
                "--fps",
                "30",
                "--seed",
                "7",
            ]
            if args.unicode:
                cmd.append("--unicode")
            if args.ghost:
                cmd.append("--ghost")
            subprocess.run(
                cmd,
                check=True,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            with open(os.devnull, "w") as devnull:
                old = sys.stdout
                sys.stdout = devnull
                try:
                    ansi2png.render(ans, png, (w, h), None)
                finally:
                    sys.stdout = old
            im = Image.open(png).convert("RGB")
            if im.width > args.width:
                im = im.resize((args.width, round(im.height * args.width / im.width)), Image.LANCZOS)
            images.append(im.convert("P", palette=Image.Palette.ADAPTIVE, colors=128))
    if not images:
        return 1
    images[0].save(
        args.out,
        save_all=True,
        append_images=images[1:],
        duration=args.duration,
        loop=0,
        optimize=True,
        disposal=2,
    )
    size = os.path.getsize(args.out)
    print(f"wrote {args.out} ({len(images)} frames, {size / 1024:.0f} KiB)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
