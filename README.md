# texiao

> 终端里的科幻 ASCII 特效引擎 — 代码雨、燃烧火焰、巨龙盘旋、世界地图、黑客帝国。
> Rust 单线程、无异步运行时、diff 渲染，启动仅约 **2 ms**，常驻内存约 **2.1 MB**，30 FPS 下 CPU 约 **0.7–2.5%** 单核。当前 **v0.2.0**。

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

- **5 个可切换特效**：`rain` / `fire` / `dragon` / `map` / `matrix`；支持参数、运行时热切换与主题。
- **低资源**：单线程、无 async、无事件循环框架；默认 30 FPS；仅依赖 `libc`。
- **快速启动**：`--version` 中位数约 **2.0 ms**；发布版二进制约 **507 KB**。
- **低内存**：交互模式 RSS 约 **2.05–2.20 MB**（120×36，truecolor）；160×48 离屏快照最大约 2.5 MB。
- **diff 渲染**：双缓冲 Canvas，只重绘变化的单元格，并合并颜色序列，降低终端写入与 CPU。
- **颜色动画**：24-bit truecolor；8 套主题；可降级到 256 色、16 色或单色。
- **科幻感 / 中国风**：扫描波束、数据流、城市脉冲、NASA 入侵控制台、中国龙、龙鳞与龙须。
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
texiao matrix                        # 横向 NASA/JPL 入侵控制台
texiao matrix --charset kana         # 混入半角片假名数据流
texiao --list                        # 列出特效和主题
```

## 命令行

```text
texiao <effect> [options]

  -e, --effect <name>     与位置参数等价
  -f, --fps <1-240>       目标帧率                          [默认: 30]
  -s, --speed <0.05-8>    动画速度倍率                      [默认: 1.0]
  -t, --theme <name>      配色主题                          [特效默认]
      --charset <mode>    kana | ascii                      [matrix: ascii]
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
| rain | 0.092 ms | 0.133 ms | 15.1 KiB/帧 |
| fire | 0.282 ms | 0.297 ms | 66.7 KiB/帧 |
| dragon | 0.113 ms | 0.124 ms | 14.6 KiB/帧 |
| map | 0.150 ms | 0.163 ms | 43.7 KiB/帧 |
| matrix | 0.045 ms | 0.055 ms | 7.5 KiB/帧 |

真实 PTY、truecolor、120×36、30 FPS、运行 5 秒（CPU = 子进程 `utime+stime / 5s`，含 PTY 写入，不含终端模拟器自身绘制）：

| 特效 | CPU（单核） | RSS | 5 秒输出 |
| --- | ---: | ---: | ---: |
| rain | 0.88% | 2.03 MiB | 1.5 MiB |
| fire | 2.47% | 2.09 MiB | 6.1 MiB |
| dragon | 1.12% | 2.22 MiB | 1.3 MiB |
| map | 1.55% | 2.22 MiB | 3.8 MiB |
| matrix | 0.71% | 2.20 MiB | 0.5 MiB |

- 30 FPS 下约 33.3 ms/帧预算；最重的 `fire` 离屏渲染约 **0.8% 单核**（0.282 ms/帧），实际交互 truecolor 约 **2.5%**（含 PTY 写入）。
- 启动：`--version` 40 次中位数 **1.99 ms**；真实 PTY 中首屏字节中位 **1.6–2.4 ms**（不含终端模拟器自身绘制）。
- 内存：真实 PTY 最大 RSS **2.03–2.22 MiB**；160×48 离屏快照约 2.3–2.5 MiB。
- 二进制：**507 KiB**。
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
- **火焰**：经典 Doom-fire 元胞传播（每格复制下方邻居、随机衰减与横移），叠加中央高、两侧低的燃料床和下宽上窄的软包络，形成更高、更亮、下宽上窄的火焰；配 5 段火焰色板、火星与周期爆燃。
- **巨龙**：按飞行轨迹弧长重采样的长蛇身（整屏 Lissajous 飞行），splat 到强度/深度缓冲；大型中国龙头（上下颚、嘴缝、獠牙、眼窝、鹿角、龙须、鬃毛）、四爪、背鳍、尾鳍、龙鳞与金色脊线；云海与星空。
- **世界地图**：内嵌 Natural Earth 1:110m 陆地掩膜（720×360，1 bit/px），按终端尺寸用覆盖率采样生成等距圆柱投影；城市节点、脉冲环、大圆弧数据包、扫描波束、UTC 时钟、极区自动压暗。
- **黑客帝国**：横向代码/十六进制数据流（可混入半角片假名）；NASA/JPL 入侵控制台含 TARGET/NODE/CHANNEL、FIREWALL/DECRYPT 进度条、日志打字机、雷达扫描、ACCESS GRANTED 与 glitch 故障带。
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
- `matrix` 默认使用 ASCII 数据流；`--charset kana` 会混入半角片假名（需要终端字体支持，否则会出现方框）。
- 非 TTY（管道、CI）会提示使用 `--snapshot`。
- 画布使用黑底；请在深色终端中使用，效果最佳。

## 测试

```bash
cargo test
cargo clippy --all-targets
cargo fmt --check
```

当前包含 30 个单元/集成测试：颜色转换、终端按键解析、CLI 校验、Canvas 裁剪与差分、地图掩膜地理坐标、5 个特效在多种尺寸下的稳定性、小终端不 panic、确定性种子等。

交互部分也在真实 PTY 中做了实测：`q` / `Esc` / `Ctrl-C` / `SIGTERM` 退出、暂停、帮助面板、特效切换、`i` 状态行、窗口缩放、`--no-alt`、颜色自动检测/256/单色/`NO_COLOR`，以及退出时的光标、自动换行和备用屏幕恢复。

## 版本与回退

- 当前版本：**0.2.0**（`texiao --version`），tag **`v0.2.0`**。
- 修改前基线：git tag **`v0.1.0`**（commit `bb57b76`，tag 信息包含 “baseline snapshot”）。回退：`git checkout v0.1.0` 或 `git reset --hard v0.1.0`。
- 额外备份（不进入 git）：`backups/texiao-v0.1.0-source.tar.gz` 与 `backups/texiao-v0.1.0-aarch64-apple-darwin`，附带 SHA-256 校验。
- 本次改动记录在 [CHANGELOG.md](CHANGELOG.md)。

## 数据来源、参考与许可

- 陆地轮廓：[Natural Earth](https://www.naturalearthdata.com/) 1:110m land，**public domain**，经 [natural-earth-vector](https://github.com/nvkelso/natural-earth-vector) GeoJSON 转换。
- 火焰灵感：经典 Doom-fire 元胞算法，参考了 [pserey/doom-fire-algorithm](https://github.com/pserey/doom-fire-algorithm) 的 Python 实现与 [horizonwiki/fire](https://github.com/horizonwiki/fire) 的终端思路；本项目的 Rust 实现、配色、火星/爆燃与渲染均为独立编写。
- 本项目代码：[MIT](LICENSE)。

---

```text
  t e x i a o  ::  stay curious, look up.
```
