#!/usr/bin/env python3
"""Small consistency check for the docs, help screens and the effect registry.

Run from the repository root after `cargo build` (or let it build):

    python3 scripts/check_docs.py
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

PRIMARY_FLAGS = [
    "--effect", "--playlist", "--switch", "--transition", "--transition-secs",
    "--no-fade", "--fps", "--speed", "--chase-speed", "--theme", "--charset",
    "--unicode", "--ascii", "--color", "--no-color", "--no-hud", "--no-alt",
    "--density", "--spin", "--twinkle", "--meteors", "--beam-speed",
    "--beam-motion", "--beam-jump", "--trail", "--seed-count", "--seed-speed", "--street",
    "--tornado-count", "--tornado-speed", "--life-cycle", "--eye-motion",
    "--eye-still", "--ghost", "--gallery", "--plain", "--jump", "--no-jump",
    "--snapshot", "--frames", "--size", "--bench", "--seed", "--duration",
    "--list", "--help", "--version", "--completions",
]


def run(*args: str) -> str:
    proc = subprocess.run(
        ["cargo", "run", "--quiet", "--", *args],
        cwd=ROOT,
        text=True,
        capture_output=True,
    )
    if proc.returncode != 0:
        print(proc.stderr, file=sys.stderr)
        raise SystemExit(f"trix {' '.join(args)} failed")
    return proc.stdout


def effect_rows() -> list[tuple[str, str, str]]:
    text = run("--list", "--no-color")
    rows = []
    for line in text.splitlines():
        if not line.startswith("  "):
            continue
        parts = re.split(r"\s{2,}", line.strip())
        if len(parts) >= 3 and re.fullmatch(r"[a-z]+", parts[0]):
            rows.append((parts[0], parts[1], parts[2]))
    if len(rows) != 35:
        raise SystemExit(f"expected 35 effects in --list, got {len(rows)}")
    return rows


def check_readme(path: Path, rows: list[tuple[str, str, str]]) -> None:
    text = path.read_text()
    for key, en, zh in rows:
        for value, label in ((key, "key"), (en, "English name"), (zh, "Chinese name")):
            if value not in text:
                raise SystemExit(f"{path.name}: missing {label} {value!r}")
        # Main previews must use the cache-safe ASCII image names.
        ascii_shot = f"docs/{key}-ascii.png"
        if ascii_shot not in text:
            raise SystemExit(f"{path.name}: main preview must use {ascii_shot}")
    headings = [line for line in text.splitlines() if line.startswith("## ")]
    if len(headings) < 8:
        raise SystemExit(f"{path.name}: expected more sections, got {len(headings)}")

    # Local Markdown links and images must exist.
    for target in re.findall(r"\]\(([^)\s]+)\)", text):
        if target.startswith(("http://", "https://", "#", "mailto:")):
            continue
        target = target.split("#", 1)[0]
        if target and not (ROOT / target).exists():
            raise SystemExit(f"{path.name}: broken local link {target}")

    # Screenshots are ASCII by default; the Unicode pair must live in the
    # explicit ASCII / Unicode section.
    for ascii_shot, unicode_shot in (
        ("docs/mario-ascii.png", "docs/mario.png"),
        ("docs/ecg-ascii.png", "docs/ecg.png"),
    ):
        if ascii_shot not in text or unicode_shot not in text:
            raise SystemExit(f"{path.name}: missing {ascii_shot} / {unicode_shot}")


def main() -> int:
    rows = effect_rows()
    help_text = run("--help", "--no-color")
    all_text = run("--help", "all", "--no-color")

    for key, _, _ in rows:
        if key not in help_text:
            raise SystemExit(f"--help: missing effect {key}")
    for flag in PRIMARY_FLAGS:
        if flag not in all_text:
            raise SystemExit(f"--help all: missing flag {flag}")

    for readme in ("README.md", "README.zh-CN.md"):
        check_readme(ROOT / readme, rows)

    en = (ROOT / "README.md").read_text()
    zh = (ROOT / "README.zh-CN.md").read_text()
    en_headings = sum(1 for line in en.splitlines() if line.startswith("## "))
    zh_headings = sum(1 for line in zh.splitlines() if line.startswith("## "))
    if en_headings != zh_headings:
        raise SystemExit(
            f"README section mismatch: English {en_headings}, Chinese {zh_headings}"
        )

    for shot in ("docs/help.png", "docs/help-rain.png", "docs/overview.png"):
        if not (ROOT / shot).exists():
            raise SystemExit(f"missing screenshot {shot}")

    print(
        f"docs OK: {len(rows)} effects, {len(PRIMARY_FLAGS)} primary flags, "
        f"{en_headings} English sections / {zh_headings} Chinese sections"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
