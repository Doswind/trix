# Launch kit

Copy-paste drafts for announcing trix-effects. Adjust the tone to the community and
follow each platform's self-promotion rules. The best asset is the `trix --help`
screenshot and a short GIF; lead with those.

## One-liners

- **English**: 26 animated ASCII/Unicode effects for your terminal, in one ~1.1 MB Rust binary. No dependencies, ~2 ms startup, `trix --help` is a live showcase.
- **中文**：26 个终端 ASCII/Unicode 动态特效，装进一个约 1.1 MB 的 Rust 单文件；零依赖、约 2 ms 启动，连 `--help` 本身都是特效界面。

## Show HN

**Title**

> Show HN: Trix – 26 ASCII effects in a 1.1 MB Rust binary

**Body**

> I built a terminal effects engine with 26 scenes: code rain, fire, a coiling
> dragon, the Matrix, a black hole, fireworks, the solar system, Mona Lisa, an
> ice-ballet duet, and more. It is a single ~1.1 MB Rust binary with no runtime
> dependencies, about 2 ms startup and a diff renderer that only repaints
> changed cells.
>
> Two things I had the most fun with:
>
> - `trix --help` is a styled overview with a real HUD frame and a colour-coded
>   effect grid; `trix --help rain` renders a preview using the actual scene.
> - Everything starts in ASCII, with `--unicode` as an opt-in. The README
>   screenshots are ASCII, and Unicode previews are isolated in their own
>   section.
>
> It runs on macOS, Linux and WSL. Source and prebuilt static binaries:
> https://github.com/Doswind/trix-effects
>
> Happy to answer questions about the renderer, the no-dependency terminal
> driver, or the help layout.

## Reddit

### r/rust

**Title**

> Trix: 26 animated ASCII/Unicode effects for the terminal, no dependencies

**Body**

> A single-threaded Rust effects engine: code rain, fire, dragon, Matrix, black
> hole, fireworks, solar system, Mona Lisa, ballet, and 18 more. `libc` is the
> only dependency; the release binary is about 1.1 MB.
>
> The part I would love feedback on is the help system: `trix --help` renders a
> wordmark, a real HUD frame, a colour-coded effect grid and grouped options;
> `trix --help <effect>` embeds a preview rendered by the actual scene. It
> degrades to aligned plain text off a TTY.
>
> Repo: https://github.com/Doswind/trix-effects

### r/commandline

**Title**

> trix-effects: 26 ASCII/Unicode effects for your terminal (single binary, no deps)

**Body**

> Code rain, fire, a dragon, the Matrix, a black hole, fireworks, Mona Lisa, and
> 20 more. `trix` cycles everything, `trix --help <effect>` shows a live preview,
> and `--snapshot` writes a deterministic ANSI frame for recording.
>
> macOS, Linux and WSL. Static Linux x86_64/aarch64 builds. MIT.

### r/unixporn

**Title**

> [OC] trix-effects – 26 terminal effects (code rain, dragon, Mona Lisa)

**Body**

> A small Rust terminal effects engine. The screenshots are the default ASCII
> edition; `--unicode` is opt-in. Eight colour themes, a shared HUD, and a
> `--snapshot` mode for clean captures.
>
> Repo: https://github.com/Doswind/trix-effects

## Lobsters

**Title**

> trix-effects: 26 ASCII effects in a dependency-free Rust binary

**Tags**: `rust`, `show`, `visualization`

## This Week in Rust / Rust Weekly

Submit the repo link with a one-paragraph description:

> trix-effects is a 26-effect ASCII/Unicode terminal engine in Rust. One ~1.1 MB
> binary, `libc` as the only dependency, a diff renderer, eight themes, and a
> metadata-driven help system that renders a real preview of each effect.
> https://github.com/Doswind/trix-effects

## 中文渠道

### V2EX（分享创造）

**标题**

> 分享创造：用 Rust 写了 26 个终端 ASCII 特效，单文件约 1.1MB，无依赖

**正文**

> 做了一个终端特效引擎 trix-effects：代码雨、火焰、巨龙、黑客帝国、黑洞、烟花、
> 太阳系、蒙娜丽莎、冰上芭蕾、生命的轨迹等 26 个场景。
>
> - Rust 单线程，唯一依赖 `libc`，发布二进制约 1.1MB
> - 默认 ASCII，`--unicode` 显式开启
> - 8 套主题、diff 渲染、`--snapshot` 可录制
> - `trix --help` 本身也是界面：字标、真实 HUD 画面、特效网格、分组参数
> - `trix --help rain` 会用真实场景渲染一张预览
>
> macOS / Linux / WSL 可用，Release 有静态 Linux x86_64/aarch64 和 macOS
> universal 包。仓库：https://github.com/Doswind/trix-effects
>
> 欢迎提意见，尤其是帮助排版、终端兼容性和新特效的点子。

### 少数派

**标题**

> 在终端里看代码雨、巨龙和蒙娜丽莎：trix-effects

**导语**

> 26 个终端 ASCII/Unicode 动态特效，一个约 1.1MB 的 Rust 单文件。默认 ASCII，
> 不依赖任何运行时，连 `--help` 都是一张会动的界面。

### 知乎

**标题**

> 如何用 Rust 写一个 26 种特效的终端动画引擎？

**回答开头**

> 最近把 trix-effects 整理到了 0.9.1：26 个场景、8 套主题、单线程、只依赖 libc，
> 发布二进制约 1.1MB。比较有意思的是帮助系统——`trix --help` 会渲染真实 HUD
> 画面，`trix --help rain` 直接用场景本身生成预览。仓库：
> https://github.com/Doswind/trix-effects

### Bilibili / 小红书 / 即刻

- 30–45 秒录屏：`trix --help` → `trix rain` → `trix dragon` → `trix monalisa` → `trix life`
- 标题：`26 个终端特效，装进 1.1MB 的 Rust 单文件`
- 文案关键词：终端、ASCII、代码雨、Rust、程序员、命令行、复古未来

## Awesome lists

Suggested entry (adjust to each list's format):

```markdown
- [trix-effects](https://github.com/Doswind/trix-effects) - 26 animated ASCII/Unicode
  effects for the terminal in one fast, dependency-free Rust binary.
```

Targets: `awesome-rust`, `awesome-cli-apps`, `awesome-terminal`,
`awesome-ascii-art`, `awesome-tui`, `awesome-command-line-apps`.

## Launch checklist

- [ ] `v0.9.1` release is green and the three archives + `SHA256SUMS` exist.
- [ ] README first screen has the overview image, one-liner and install command.
- [ ] `docs/social-preview.png` uploaded in repository settings.
- [ ] Topics and description set.
- [ ] `trix --help` and `trix --help rain` screenshots ready.
- [ ] 15–30 s GIF ready (help + three effects).
- [ ] Post on a weekday morning (US time for HN/Reddit, evening for China).
- [ ] Reply to comments for the first 2–3 hours.
- [ ] Submit to 2–3 awesome lists, not all at once.
- [ ] Track GitHub Insights → Traffic → referrers and iterate the README.

Do not buy stars or spam the same link across communities. A useful tool plus a
clear first screen is the durable growth loop.
