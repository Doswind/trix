# trix project conventions

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
  - `src/scene.rs` (`default_charset` test coverage),
  - `README.md` and `README.zh-CN.md` charset examples/tables,
  - `CHANGELOG.md` and `CHANGELOG.zh-CN.md` under `Unreleased`.

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

## General checks

Before finishing a change run:

```bash
cargo fmt --all
cargo test
cargo clippy --all-targets
cargo build --release
```
