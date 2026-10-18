# Contributing to trix-tui

Thanks for taking the time to improve trix-tui. Issues, bug reports, effect
ideas and pull requests are all welcome.

## Development setup

- Rust 1.70+ (stable recommended)
- macOS, Linux or WSL
- A truecolor terminal is recommended, but everything must also work in
  256 / 16 / monochrome and in ASCII.

```bash
git clone https://github.com/Doswind/trix-tui
cd trix-tui
cargo build --release
./target/release/trix --help
cargo test
```

## Before opening a pull request

Run all of these from the repository root:

```bash
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
python3 scripts/check_docs.py
```

Please also smoke-test the change in a real terminal:

```bash
./target/release/trix --help
./target/release/trix --help rain
./target/release/trix rain
```

## Adding or changing an effect

1. Implement the scene in `src/scenes/`.
2. Register it in `src/scene.rs` (`EffectId`, `ALL`, `key`, titles, theme).
3. Add its summary, description, category, shortcut, charset support and
   tuning flags in `src/meta.rs`. `--help`, `--list`, the `i` panel and the
   interactive `h` panel read that metadata.
4. Add or update tests in the scene and in `src/meta.rs`.
5. Update both READMEs, both changelogs and `docs/hud-and-keys.md`.
6. Regenerate the screenshots and GIFs (see below).
7. Run `python3 scripts/check_docs.py`.

## ASCII first

Every effect that supports ASCII and Unicode must start in ASCII. Unicode is
an explicit opt-in via `--unicode` / `--charset unicode`. `--ascii` forces
ASCII again.

README screenshots use the **default ASCII glyphs** and the current code.
Unicode screenshots belong only in the “ASCII / Unicode” section, where the
text explicitly introduces `--unicode`.

## Regenerating screenshots

The release binary and the bundled Python environment are enough:

```bash
cargo build --release

# One ANSI frame, then a PNG
./target/release/trix rain --snapshot /tmp/rain.ans \
  --size 120x36 --frames 90 --seed 7 --color truecolor
python3 tools/ansi2png.py /tmp/rain.ans docs/rain.png \
  --size 120x36 --label "trix rain"

# An animated GIF
python3 tools/render_gif.py rain docs/rain.gif \
  --start 60 --frames 24 --step 3 --size 80x24 --width 600 --duration 66
```

`tools/ansi2png.py` and `tools/render_gif.py` need Pillow. Unicode previews
must be generated with the explicit `--unicode` flag and kept in the Unicode
section of the README.

## Commit and PR style

- Keep commits focused; a short imperative subject is enough.
- Describe user-visible changes and the tests you ran.
- For UI/art changes, include a screenshot or GIF in the PR.
- Do not commit generated `target/`, caches, `*.ans` files or local design
  drafts under `docs/proposals/` unless they are part of the change.

## Code of conduct

Be kind and constructive. Harassment, personal attacks and discriminatory
language are not acceptable. This project follows the spirit of the
[Contributor Covenant](https://www.contributor-covenant.org/).
