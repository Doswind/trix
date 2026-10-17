# texiao

> 终端里的科幻 ASCII 特效引擎 — 代码雨、燃烧火焰、巨龙盘旋、世界地图、黑客帝国。
> Rust 单线程、无异步运行时、diff 渲染，启动仅约 **2 ms**，常驻内存约 **2.1 MB**，30 FPS 下 CPU 约 **0.7–2.6%** 单核。

![texiao overview](docs/overview.png)

> 动图演示：[火焰](docs/fire.gif) · [巨龙](docs/dragon.gif) · [Matrix](docs/matrix.gif)

## 特效

| 代码雨 `rain` | 燃烧火焰 `fire` |
| --- | --- |
| ![rain](docs/rain.png) | ![fire](docs/fire.png) |

| 巨龙盘旋 `dragon` | 世界地图 `map` |
| --- | --- |
| ![dragon](docs/dragon.png) | ![map](docs/map.png) |

| 黑客帝国 `matrix` |
| --- |
| ![matrix](docs/matrix.png) |

> 截图均为 120×36、truecolor、离屏渲染；动图见上。

## 特性

- **5 个可切换特效**：`rain` / `fire` / `dragon` / `map` / `matrix`，支持命令参数与运行时热切换。
- **低资源**：单线程、无 async、无事件循环框架；默认 30 FPS；仅依赖 `libc`。
- **快速启动**：`--version` 中位数约 **2.0 ms**；发布版二进制约 **506 KB**。
- **低内存**：交互模式 RSS 约 **2.05–2.20 MB**（120×36，truecolor）；160×48 离屏快照最大约 2.5 MB。
- **diff 渲染**：双缓冲 Canvas，只重绘变化的单元格，并合并颜色序列，降低终端写入与 CPU。
- **颜色动画**：24-bit truecolor；8 套主题；可降级到 256 色、16 色或单色。
- **科幻感**：扫描波束、数据流、城市脉冲、轨道环、等离子核心、火焰呼吸。
- **可录制 / 可压测**：内置 `--snapshot` 与 `--bench`，方便生成文档、做 CI 性能回归。
- **安全退出**：raw mode、备用屏幕、光标/自动换行在正常退出、Ctrl-C、SIGTERM 和 panic 时都会恢复。

## 快速开始

要求 Rust 1.70+（推荐 stable）。

```bash
cargo build --release
./target/release/texiao rain
```

安装到 `PATH`：

```bash
cargo install --path . --locked
texiao dragon
```

### 一条命令一个特效

```bash
texiao rain                          # 代码雨
texiao fire --fps 60                 # 60 FPS 火焰
texiao dragon --theme gold           # 金色巨龙
texiao map --theme ice               # 全息世界地图
texiao matrix                        # 黑客帝国（半角片假名）
texiao matrix --charset ascii        # 字体不支持片假名时用 ASCII
texiao --list                        # 列出特效和主题
```

## 命令行

```text
texiao <effect> [options]

  -e, --effect <name>     与位置参数等价
  -f, --fps <1-240>       目标帧率                          [默认: 30]
  -s, --speed <0.05-8>    动画速度倍率                      [默认: 1.0]
  -t, --theme <name>      配色主题                          [特效默认]
      --charset <mode>    kana | ascii                      [matrix: kana]
      --color <mode>      auto | truecolor | 256 | 16 | mono [默认: auto]
      --seed <u64>        固定随机种子（可复现）
      --duration <secs>   运行 N 秒后自动退出
      --no-alt            不使用备用屏幕（保留终端历史）
  -l, --list              列出特效/主题
  -h, --help              帮助
  -V, --version           版本

录制 / 压测
      --snapshot <path>   模拟 --frames 步后输出一帧 ANSI 到文件（'-' = stdout）
      --frames <n>        模拟步数（不是输出帧数）
      --size <WxH>        离屏画布尺寸，例如 120x36
      --bench             离屏渲染压测（包含 diff ANSI 编码）
```

### 运行时快捷键

| 键 | 作用 |
| --- | --- |
| `q` / `Esc` | 退出（帮助面板打开时 `Esc` 先关闭面板） |
| `Space` | 暂停 / 继续 |
| `n` / `p`、方向键 | 下一个 / 上一个特效 |
| `1`–`5` | 直接跳到 `rain` / `fire` / `dragon` / `map` / `matrix` |
| `t` | 循环主题 |
| `+` / `-` | 加速 / 减速（0.1×–8×） |
| `r` | 用新种子重开当前特效 |
| `i` | 显示 / 隐藏状态行（FPS、耗时、主题、尺寸、种子） |
| `h` / `?` | 帮助面板 |

## 主题

| 主题 | 风格 | 默认用于 |
| --- | --- | --- |
| `neon` | 霓虹青 / 紫 | rain |
| `matrix` | 矩阵绿 | matrix |
| `ember` | 真实火焰（红→橙→黄→白） | fire |
| `aurora` | 翡翠龙 / 极光金 | dragon |
| `ice` | 全息冰蓝 | map |
| `plasma` | 品红 / 等高能 | 任意 |
| `gold` | 帝王金 | 任意 |
| `chrome` | 银灰单色 | 任意 |

`fire` 的火焰颜色来自主题调色板的 5 个锚点，所以 `--theme ice` 会得到“冷焰”，`--theme ember` 是默认的燃烧火焰。

## 性能

环境：Apple Silicon (ARM64)、macOS、Rust 1.96.1、`--release`（LTO + `codegen-units=1` + `panic=abort` + `strip`）。
画布 160×48，300 帧离屏压测：

| 特效 | 平均渲染 | p95 | diff 输出 |
| --- | ---: | ---: | ---: |
| rain | 0.146 ms | 0.244 ms | 15.1 KiB/帧 |
| fire | 0.307 ms | 0.350 ms | 81.1 KiB/帧 |
| dragon | 0.112 ms | 0.130 ms | 13.3 KiB/帧 |
| map | 0.156 ms | 0.186 ms | 43.7 KiB/帧 |
| matrix | 0.096 ms | 0.133 ms | 19.4 KiB/帧 |

真实 PTY、truecolor、120×36、30 FPS、运行 5 秒（CPU = 子进程 `utime+stime / 5s`，含 PTY 写入，不含终端模拟器自身绘制）：

| 特效 | CPU（单核） | RSS | 5 秒输出 |
| --- | ---: | ---: | ---: |
| rain | 0.73% | 2.05 MiB | 1.5 MiB |
| fire | 2.59% | 2.12 MiB | 8.3 MiB |
| dragon | 1.10% | 2.16 MiB | 1.3 MiB |
| map | 1.47% | 2.20 MiB | 3.8 MiB |
| matrix | 0.95% | 2.05 MiB | 1.7 MiB |

- 30 FPS 下约 33.3 ms/帧预算；最重的 `fire` 离屏渲染约 **0.9% 单核**（0.307 ms/帧），实际交互 truecolor 约 **2.6%**（含 PTY 写入）。
- 启动：`--version` 40 次中位数 **1.99 ms**；真实 PTY 中首屏字节中位 **1.6–2.4 ms**（不含终端模拟器自身绘制）。
- 内存：真实 PTY 最大 RSS **2.05–2.20 MiB**；`/usr/bin/time -l` 160×48 离屏快照 **2.26–2.49 MiB**。
- 二进制：**506 KiB**。
- 交互模式会遵守 `NO_COLOR`；离屏 `--snapshot` / `--bench` 默认 truecolor，可用 `--color` 覆盖。

```bash
# 复现自己的数字
scripts/bench.sh
./target/release/texiao fire --bench --size 160x48 --frames 300
```

## 实现

- **终端驱动**（`src/term.rs`）：直接用 `libc` 设置 raw mode、备用屏幕、隐藏光标、关闭自动换行；每帧非阻塞读键、`TIOCGWINSZ` 每帧检测尺寸；安装 SIGINT/SIGTERM/SIGHUP 标志和 panic hook 恢复终端。
- **Canvas**（`src/canvas.rs`）：`Cell { char, fg }` 双缓冲；逐行找变化区间，移动光标一次后批量写字符；颜色变化才发 SGR；空格不参与颜色差分；支持 truecolor / 256 / 16 / mono 四级降级。
- **零框架**：只有一个 crate 依赖 `libc`。没有 tokio、没有 crossterm、没有 per-frame 线程/任务。
- **代码雨**：逐列头/尾、随机速度与深度、白热头、青紫尾、二进制雾、横向数据包、CRT 扫描。
- **火焰**：经典 Doom-fire 元胞传播（每格复制下方邻居、随机衰减与横移），配 5 段火焰色板、火星粒子、周期爆燃；整屏更新 O(W×H)，极低开销。
- **巨龙**：190 个采样点的 3D 螺旋路径，投影后 splat 到强度/深度缓冲，实现近大远小与核心遮挡；程序化头部（上下颚、嘴缝、牙、眼窝、龙角、龙须、鬃毛）、背鳍、爪足、尾鳍；头部朝向核心，周期性喷出火焰锥；中心为带环的等离子球。
- **世界地图**：内嵌 Natural Earth 1:110m 陆地掩膜（720×360，1 bit/px），按终端尺寸用覆盖率采样生成等距圆柱投影；城市节点、脉冲环、大圆弧数据包、扫描波束、UTC 时钟、极区自动压暗。
- **黑客帝国**：半角片假名+数字雨、白色头部、发光拖尾、随机 glitch 条带、打字机消息（“Wake up, Neo…”、“The Matrix has you…”）。
- **地图数据离线生成**：`tools/gen_land.py` 下载 Natural Earth GeoJSON 并栅格化为 `src/data/land_720x360.bin`，结果已内嵌；构建只需拉取 `libc`，运行时不访问网络。

## 录制与文档工具

```bash
# 输出一帧 ANSI（可 cat，也可进一步转图）
texiao dragon --snapshot dragon.ans --size 120x36 --frames 145 --seed 7

# ANSI → PNG（需要 Pillow；仅文档用）
python3 tools/ansi2png.py dragon.ans dragon.png --size 120x36 --label "texiao dragon"

# 连拍 → GIF
python3 tools/render_gif.py dragon docs/dragon.gif --start 100 --frames 24 --step 3
```

## 兼容性

- 支持 macOS、Linux、WSL（Unix termios）。原生 Windows 未实现。
- 推荐 24-bit truecolor 终端；`--color auto` 会根据 `COLORTERM`/`TERM` 自动降级。
- Matrix 的片假名需要终端字体包含半角片假名（大多数终端会用回退字体）。若出现方框，使用 `--charset ascii`。
- 非 TTY（管道、CI）会提示使用 `--snapshot`。
- 画布使用黑底；请在深色终端中使用，效果最佳。

## 测试

```bash
cargo test
cargo clippy --all-targets
cargo fmt --check
```

当前包含 28 个单元/集成测试：颜色转换、终端按键解析、CLI 校验、Canvas 裁剪与差分、地图掩膜地理坐标、5 个特效在多种尺寸下的稳定性、小终端不 panic、确定性种子等。

交互部分也在真实 PTY 中做了实测：`q` / `Esc` / `Ctrl-C` / `SIGTERM` 退出、暂停、帮助面板、特效切换、`i` 状态行、窗口缩放、`--no-alt`、颜色自动检测/256/单色/`NO_COLOR`，以及退出时的光标、自动换行和备用屏幕恢复。

## 数据来源、参考与许可

- 陆地轮廓：[Natural Earth](https://www.naturalearthdata.com/) 1:110m land，**public domain**，经 [natural-earth-vector](https://github.com/nvkelso/natural-earth-vector) GeoJSON 转换。
- 火焰灵感：经典 Doom-fire 元胞算法，参考了 [pserey/doom-fire-algorithm](https://github.com/pserey/doom-fire-algorithm) 的 Python 实现与 [horizonwiki/fire](https://github.com/horizonwiki/fire) 的终端思路；本项目的 Rust 实现、配色、火星/爆燃与渲染均为独立编写。
- 本项目代码：[MIT](LICENSE)。

---

```text
  t e x i a o  ::  stay curious, look up.
```
