# texiao

> 终端里的科幻 ASCII 特效引擎 — 代码雨、火焰、巨龙、世界地图、NASA 登录、黑洞、光束、烟花、太阳系、银河、佛光、Tom & Jerry。
> Rust 单线程、无异步运行时、diff 渲染，启动约 **2 ms**，常驻内存约 **2.1 MB**，30 FPS 下 CPU 约 **0.4–2.9%** 单核。当前 **v0.3.0**。

![texiao overview](docs/overview.png)

> 动图：[火焰](docs/fire.gif) · [巨龙](docs/dragon.gif) · [NASA 登录](docs/matrix.gif) · [黑洞](docs/blackhole.gif) · [光束](docs/beams.gif) · [烟花](docs/fireworks.gif) · [太阳系](docs/solar.gif) · [银河](docs/galaxy.gif) · [佛光](docs/buddha.gif) · [Tom & Jerry](docs/tomjerry.gif)

## 特效

| 代码雨 `rain` | 燃烧火焰 `fire` | 巨龙盘旋 `dragon` |
| --- | --- | --- |
| ![rain](docs/rain.png) | ![fire](docs/fire.png) | ![dragon](docs/dragon.png) |

| 世界地图 `map` | 黑客帝国 `matrix` | 黑洞 `blackhole` |
| --- | --- | --- |
| ![map](docs/map.png) | ![matrix](docs/matrix.png) | ![blackhole](docs/blackhole.png) |

| 光束 `beams` | 烟花 `fireworks` | 太阳系 `solar` |
| --- | --- | --- |
| ![beams](docs/beams.png) | ![fireworks](docs/fireworks.png) | ![solar](docs/solar.png) |

| 满天星河 `galaxy` | 佛光普照 `buddha` | 猫和老鼠 `tomjerry` |
| --- | --- | --- |
| ![galaxy](docs/galaxy.png) | ![buddha](docs/buddha.png) | ![tomjerry](docs/tomjerry.png) |

| 特效 | 说明 |
| --- | --- |
| `rain` 代码雨 | 逐列头/尾、随机速度与深度、白热头、青紫尾、二进制雾、横向数据包、CRT 扫描 |
| `fire` 燃烧火焰 | Doom-fire 热扩散 + 多个随机燃料堆；每堆独立宽度/热量/漂移，形成多股细长火舌；火星与爆燃 |
| `dragon` 巨龙盘旋 | 整屏 Lissajous 飞行的中国龙：弧长跟随龙身、鹿角、龙须、鬃毛、獠牙、四爪、背鳍、尾鳍、龙鳞 |
| `map` 世界地图 | 内嵌 Natural Earth 陆地掩膜、城市节点、大圆弧数据包、扫描波束、UTC 时钟 |
| `matrix` 黑客帝国 | **NASA 后台登录大屏**：大号 ASCII NASA 字样、OPERATOR/PASSWORD/2FA、AUTH/DECRYPT 进度、雷达、ACCESS GRANTED |
| `blackhole` 黑洞 | 事件视界、连续光子环、吸积盘、引力透镜弧、喷流；恒星不断被吸入并计数 |
| `beams` 光束 | 横向不规则光束，随机亮色，白热头/渐隐尾，星空与科技网格 |
| `fireworks` 烟花 | 火箭升空，内外双层粒子环绽放，延迟爆裂，城市天际线、星空与烟雾 |
| `solar` 太阳系 | 太阳 + 九大行星在一条轨道线上；不同颜色/大小，土星环、地球/木星卫星、彗星、星空网格 |
| `galaxy` 满天星河 | 满屏 2000+ 星星，中央稠密带形成银河；星团、暗尘、流星、星座连线 |
| `buddha` 佛光普照 | 正中的 ASCII 佛祖、大号头光/光环、莲花座、曼陀罗、金色粒子与飘落花瓣，全身发光 |
| `tomjerry` 猫和老鼠 | Tom 追逐 Jerry 的太空站走廊动画；速度受 `--speed` / `+` `-` 控制，HUD 实时显示倍率 |

> 截图均为 120×36、truecolor、离屏渲染；动图见上。

## 特性

- **12 个可切换特效**：支持命令行参数、运行时热切换与主题。
- **低资源**：单线程、无 async、无事件循环框架；默认 30 FPS；仅依赖 `libc`。
- **快速启动**：`--version` 中位数约 **2.0 ms**；发布版二进制约 **587 KB**。
- **低内存**：交互模式 RSS 约 **2.05–2.27 MB**（120×36，truecolor）。
- **diff 渲染**：双缓冲 Canvas，只重绘变化单元格并合并 SGR，降低终端写入与 CPU。
- **颜色动画**：24-bit truecolor；8 套主题；可降级到 256 色、16 色或单色。
- **满屏点缀**：每个场景都有星空、粒子、网格、光晕或城市/走廊背景，避免大面积空白。
- **科幻 / 中国风**：NASA 大屏、黑洞吸积、粒子烟花、全息太阳系、银河、佛光、中国龙。
- **可录制 / 可压测**：内置 `--snapshot` 与 `--bench`。
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
texiao fire --fps 60                 # 多堆细长火焰
texiao dragon --theme gold           # 金色巨龙
texiao map --theme ice               # 全息世界地图
texiao matrix                        # NASA 后台登录大屏（大号 NASA 字样）
texiao blackhole                     # 黑洞吞噬恒星
texiao beams                         # 横向随机亮色光束
texiao fireworks                     # 双层烟花绽放
texiao solar                         # 太阳系九大行星一线
texiao galaxy                        # 满天星河 / 银河
texiao buddha --theme gold           # 佛光普照
texiao tomjerry --speed 2            # Tom 追 Jerry，2 倍速
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
| `1`–`9`、`0`、`[`、`]` | 依次跳到 12 个特效（见下） |
| `t` | 循环主题 |
| `+` / `-` | 加速 / 减速（0.1×–8×），Tom & Jerry 同样生效 |
| `r` | 用新种子重开当前特效 |
| `i` | 显示 / 隐藏状态行（FPS、耗时、主题、尺寸、种子） |
| `h` / `?` | 帮助面板 |

数字键顺序：`1 rain`、`2 fire`、`3 dragon`、`4 map`、`5 matrix`、`6 blackhole`、`7 beams`、`8 fireworks`、`9 solar`、`0 galaxy`、`[ buddha`、`] tomjerry`。

## 主题

| 主题 | 风格 | 默认用于 |
| --- | --- | --- |
| `neon` | 霓虹青 / 紫 | rain、beams、tomjerry |
| `matrix` | 矩阵绿 | matrix |
| `ember` | 真实火焰（红→橙→黄→白） | fire |
| `aurora` | 翡翠龙 / 极光金 | dragon |
| `ice` | 全息冰蓝 | map、solar、galaxy |
| `plasma` | 品红 / 高能 | blackhole、fireworks |
| `gold` | 帝王金 | buddha |
| `chrome` | 银灰单色 | 任意 |

`fire` 的火焰颜色来自主题调色板的 5 个锚点；`--theme ice` 会得到“冷焰”，`--theme ember` 是默认燃烧火焰。

## 性能

环境：Apple Silicon (ARM64)、macOS、Rust 1.96.1、`--release`（LTO + `codegen-units=1` + `panic=abort` + `strip`）。
画布 160×48，240 帧离屏压测：

| 特效 | 平均渲染 | p95 | diff 输出 |
| --- | ---: | ---: | ---: |
| rain | 0.078 ms | 0.097 ms | 15.4 KiB/帧 |
| fire | 0.323 ms | 0.505 ms | 40.8 KiB/帧 |
| dragon | 0.123 ms | 0.158 ms | 15.0 KiB/帧 |
| map | 0.156 ms | 0.194 ms | 43.6 KiB/帧 |
| matrix | 0.057 ms | 0.080 ms | 9.0 KiB/帧 |
| blackhole | 0.100 ms | 0.124 ms | 21.3 KiB/帧 |
| beams | 0.068 ms | 0.081 ms | 19.9 KiB/帧 |
| fireworks | 0.039 ms | 0.058 ms | 8.6 KiB/帧 |
| solar | 0.027 ms | 0.033 ms | 5.7 KiB/帧 |
| galaxy | 0.083 ms | 0.103 ms | 19.9 KiB/帧 |
| buddha | 0.058 ms | 0.070 ms | 11.0 KiB/帧 |
| tomjerry | 0.020 ms | 0.027 ms | 2.7 KiB/帧 |

真实 PTY、truecolor、120×36、30 FPS、运行 5 秒（CPU = 子进程 `utime+stime / 5s`，含 PTY 写入，不含终端模拟器自身绘制）：

| 特效 | CPU（单核） | RSS | 5 秒输出 |
| --- | ---: | ---: | ---: |
| rain | 1.22% | 2.08 MiB | 1.5 MiB |
| fire | 2.93% | 2.06 MiB | 3.7 MiB |
| dragon | 1.37% | 2.27 MiB | 1.3 MiB |
| map | 1.98% | 2.25 MiB | 3.8 MiB |
| matrix | 0.79% | 2.27 MiB | 0.7 MiB |
| blackhole | 0.83% | 2.11 MiB | 2.2 MiB |
| beams | 0.66% | 2.05 MiB | 2.0 MiB |
| fireworks | 0.45% | 2.19 MiB | 0.9 MiB |
| solar | 0.48% | 2.06 MiB | 0.5 MiB |
| galaxy | 0.76% | 2.23 MiB | 2.1 MiB |
| buddha | 0.64% | 2.06 MiB | 1.0 MiB |
| tomjerry | 0.43% | 2.03 MiB | 0.3 MiB |

- 30 FPS 下约 33.3 ms/帧预算；最重的 `fire` 离屏渲染约 **1.0% 单核**（0.323 ms/帧），实际交互约 **2.9%**（含 PTY 写入）。
- 启动：`--version` 中位数约 **2.0 ms**；真实 PTY 首屏字节中位约 **1.6–2.4 ms**。
- 内存：真实 PTY 最大 RSS **2.03–2.27 MiB**。
- 二进制：**587 KiB**。
- 交互模式会遵守 `NO_COLOR`；离屏 `--snapshot` / `--bench` 默认 truecolor，可用 `--color` 覆盖。

```bash
scripts/bench.sh
./target/release/texiao fire --bench --size 160x48 --frames 300
```

## 实现

- **终端驱动**（`src/term.rs`）：libc 设置 raw mode、备用屏幕、隐藏光标、关闭自动换行；非阻塞读键、`TIOCGWINSZ` 尺寸检测、SIGINT/SIGTERM/SIGHUP 标志、panic hook。
- **Canvas**（`src/canvas.rs`）：`Cell { char, fg }` 双缓冲；逐行变化区间；颜色变化才发 SGR；支持 truecolor/256/16/mono。
- **零框架**：唯一依赖 `libc`；没有 tokio/crossterm/线程任务。
- **代码雨**：逐列头/尾、随机速度与深度、二进制雾、横向数据包、CRT 扫描。
- **火焰**：Doom-fire 元胞传播；随机 3–5 个燃料堆（位置、宽度、热量、漂移独立），每堆细长火舌，燃料堆间留暗缝；火星与周期爆燃。
- **巨龙**：飞行轨迹按弧长重采样，长蛇身 splat 到强度/深度缓冲；大型中国龙头、鹿角、龙须、鬃毛、四爪、背鳍、尾鳍、龙鳞；整屏 Lissajous 飞行。
- **世界地图**：Natural Earth 陆地掩膜、覆盖率采样、城市节点、大圆弧、扫描波束。
- **黑客帝国**：NASA 后台登录大屏。横向代码流背景；大号 ASCII NASA 字样（NASA 蓝/白/红流光）；OPERATOR/PASSWORD/2FA 登录字段；AUTH/DECRYPT 进度、日志、雷达、ACCESS GRANTED、glitch。
- **黑洞**：事件视界遮罩、连续光子环、吸积盘、引力透镜弧、喷流、恒星螺旋内落与“CONSUMED”计数。
- **光束**：对象池式随机光束，亮色头、渐隐尾、宽度/波动/速度独立；星空和网格。
- **烟花**：火箭 + 内外双环粒子 + 延迟爆裂；粒子拖尾、重力、城市剪影、星空。
- **太阳系**：太阳与九大行星一维排列；ASCII 球体、土星环、卫星、彗星、小行星带、全息星图网格。
- **银河**：预计算星点沿斜向银河带 + 中央核球分布；暗尘、流星、星座线。
- **佛光**：ASCII 佛祖（头光、面部、肩部、莲座）、多层光环/曼陀罗/射线、金色粒子与花瓣；逐字符发光。
- **Tom & Jerry**：太空站走廊（星空窗、面板、透视地板、终端、奶酪全息）；Tom/Jerry 运行帧动画、追逐/跳跃、速度倍率 HUD。
- **地图数据离线生成**：`tools/gen_land.py` 下载 Natural Earth 并栅格化为 `src/data/land_720x360.bin`，运行时不访问网络。

## 录制与文档工具

```bash
texiao dragon --snapshot dragon.ans --size 120x36 --frames 145 --seed 7
python3 tools/ansi2png.py dragon.ans dragon.png --size 120x36 --label "texiao dragon"
python3 tools/render_gif.py dragon docs/dragon.gif --start 100 --frames 24 --step 3
```

## 兼容性

- 支持 macOS、Linux、WSL（Unix termios）；原生 Windows 未实现。
- 推荐 24-bit truecolor；`--color auto` 会自动降级。
- `matrix` 默认 ASCII 数据流；`--charset kana` 可混入半角片假名。
- 非 TTY 会提示使用 `--snapshot`。
- 画布黑底；建议深色终端。

## 测试

```bash
cargo test
cargo clippy --all-targets
cargo fmt --check
```

当前 **37 个**单元/集成测试：颜色转换、终端按键解析、CLI 校验、Canvas 裁剪/差分、地图掩膜、12 个特效在多种尺寸/主题/颜色模式下的稳定性、小终端不 panic、确定性种子、Tom & Jerry 速度等。

真实 PTY 实测：`q`/`Esc`/`Ctrl-C`/`SIGTERM`、暂停、帮助、数字/`[`/`]` 切换、`i` 状态、缩放、`--no-alt`、颜色降级、退出恢复。

## 版本与回退

- **当前版本：0.3.0**（`texiao --version`），tag **`v0.3.0`**。
- 上一版：tag **`v0.2.0`**（火焰变高、横向 NASA 入侵、中国龙）。
- 最早基线：tag **`v0.1.0`**（commit `bb57b76`）。
- 回退：
  ```bash
  git checkout v0.2.0     # 只看上一版（detached HEAD）
  git switch main
  git reset --hard v0.2.0 # 彻底退回上一版
  ```
- 额外备份（不进 git）：`backups/texiao-v0.1.0-*`、`texiao-v0.2.0-*`、`texiao-v0.3.0-*`，含源码 tar 与 aarch64 macOS 二进制。
- 改动记录：[CHANGELOG.md](CHANGELOG.md)。

## 数据来源、参考与许可

- 陆地轮廓：[Natural Earth](https://www.naturalearthdata.com/) 1:110m land，**public domain**，经 [natural-earth-vector](https://github.com/nvkelso/natural-earth-vector) GeoJSON 转换。
- Doom-fire 灵感：[pserey/doom-fire-algorithm](https://github.com/pserey/doom-fire-algorithm)、[horizonwiki/fire](https://github.com/horizonwiki/fire)；本项目的 Rust 实现、多燃料堆、配色与渲染均为独立编写。
- 佛祖 ASCII 艺术改编自 [ascii.co.uk/art/buddha](https://ascii.co.uk/art/buddha)（原作者署名 `hjw`），用于 `buddha` 场景；本项目补充了颜色、光环、莲座、粒子与动画。
- 本项目代码：[MIT](LICENSE)。

---

```text
  t e x i a o  ::  stay curious, look up.
```
