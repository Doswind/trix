# trix-tui project conventions

> 项目约定：凡是同时支持 ASCII 与 Unicode 的特效，默认必须以 ASCII 启动；Unicode 只能通过 `--unicode` / `--charset unicode` 显式开启。

## ASCII first

When an effect supports both ASCII and Unicode glyphs, it **must start in
ASCII by default**.

- `EffectId::default_charset()` returns `Charset::Ascii`; do not special-case
  individual effects back to Unicode.
- Unicode is an explicit opt-in via `--unicode` / `--charset unicode`.
- `--ascii` remains available to force ASCII when a caller has previously set
  another charset mode.
- When adding or changing a dual-form effect, update:
  - `src/scene.rs` (`default_charset` test coverage) and `src/meta.rs`
    (`charset_support`),
  - `README.md` and `README.zh-CN.md` charset examples/tables,
  - `CHANGELOG.md` and `CHANGELOG.zh-CN.md` under the current version.

## Help, metadata and screenshots

- Effect summaries, descriptions, categories, shortcuts, charset support and
  per-effect flags live in `src/meta.rs`. `--help`, `--help <effect>`,
  `--help all`, `--list`, the `i` panel and the interactive `h` panel must use
  that metadata instead of their own copies.
- `trix --help` and `trix --help <effect>` must keep working when stdout is
  not a TTY: no ANSI, aligned ASCII, no preview. `--ascii`, `--unicode`,
  `--no-color` and `NO_COLOR` are the documented escape hatches.
- README screenshots use the **default ASCII glyphs** and the current code.
  Unicode screenshots belong only in the “ASCII / Unicode” section, where the
  text explicitly introduces `--unicode`.
- The project name is `trix-tui`; the command stays `trix`. Release assets are
  `trix-tui-<version>-...`, while the binary inside is `trix`.

## Docs and localization

- English is the primary documentation language: `README.md` and
  `CHANGELOG.md`.
- Keep `README.zh-CN.md` and `CHANGELOG.zh-CN.md` in sync with their English
  counterparts; both versions must contain the same sections and facts.
- Both READMEs must list every effect with its English **and** Chinese name;
  `trix --list` uses the same names (`EffectId::title_en` / `title_zh`).
- Linux release builds are static musl binaries for `x86_64` and `aarch64`;
  update both READMEs and `.github/workflows/release.yml` together when
  targets change.
- Run `python3 scripts/check_docs.py` after changing effect names, flags,
  help text, screenshots or either README.

## HUD and keys

- Row 0 and row `h-1` belong to the shared HUD (`src/hud.rs`); scenes must
  not draw text there. Dynamic counters go through `Scene::hud_status()`.
- The bottom key bar is the single source of runtime key hints. Do not add
  per-scene key hints or CLI flag reminders to the artwork; put scene
  parameters in `app::info_params` so the `i` panel can show them.
- `rain`, `fire` and `monalisa` are deliberately text-free (`HudMode::Ambient`):
  no top bar and no scene labels, but keep the bottom key bar.
- When changing a shortcut, update `src/meta.rs`, `app.rs`, the help panel,
  `src/help.rs`, both READMEs, and `docs/hud-and-keys.md` together. Shortcuts
  are case-insensitive and must work on Linux and macOS terminals.

## General checks

Before finishing a change run:

```bash
cargo fmt --all
cargo test
cargo clippy --all-targets
cargo build --release
python3 scripts/check_docs.py
```
