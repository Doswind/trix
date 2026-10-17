# trix

> 前身：texiao（原项目名）；从 v0.5.0 起正式更名为 **trix**。

> 终端里的科幻 ASCII / Unicode 特效引擎 — 代码雨、火焰、巨龙、世界地图、NASA 登录、黑洞、光束、烟花、太阳系、银河、佛光、Tom & Jerry、超级马里奥、心率跳动、蒲公英。
> Rust 单线程、无异步运行时、diff 渲染，启动约 **2 ms**，常驻内存约 **2.0–2.3 MB**，30 FPS 下 CPU 约 **0.2–1.4%** 单核。当前 **v0.6.0**。

![release](https://github.com/Doswind/trix/actions/workflows/release.yml/badge.svg)

![trix overview](docs/overview.png)

> 动图：[代码雨](docs/rain.gif) · [火焰](docs/fire.gif) · [巨龙](docs/dragon.gif) · [NASA 登录](docs/matrix.gif) · [黑洞](docs/blackhole.gif) · [光束](docs/beams.gif) · [烟花](docs/fireworks.gif) · [太阳系](docs/solar.gif) · [银河](docs/galaxy.gif) · [佛光](docs/buddha.gif) · [Tom & Jerry](docs/tomjerry.gif) · [马里奥](docs/mario.gif) · [心电](docs/ecg.gif) · [蒲公英](docs/dandelion.gif)

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

| 超级马里奥 `mario` | 心率跳动 `ecg` | 蒲公英 `dandelion` |
| --- | --- | --- |
| ![mario](docs/mario.png) | ![ecg](docs/ecg.png) | ![dandelion](docs/dandelion.png) |

| 特效 | 说明 |
| --- | --- |
| `rain` 代码雨 | 逐列头/尾、随机速度与深度、白热头、**蓝/绿双色尾**、二进制雾、横向数据包、CRT 扫描 |
| `fire` 燃烧火焰 | Doom-fire 热扩散 + **更宽连续底部火床** + 多个随机燃料堆；中间最高，顶部多股细长分开火舌；火星与爆燃 |
| `dragon` 巨龙盘旋 | 整屏 Lissajous 飞行的中国龙：弧长跟随龙身、鹿角、龙须、鬃毛、獠牙、四爪、背鳍、尾鳍、龙鳞 |
| `map` 世界地图 | 内嵌 Natural Earth 陆地掩膜、城市节点、大圆弧数据包、扫描波束、UTC 时钟 |
| `matrix` 黑客帝国 | **NASA 后台登录大屏**：Unicode 蓝色 NASA 横幅 + 白色大字 + 红色 swoosh、居中下移的登录框、OPERATOR/PASSWORD/2FA、AUTH/DECRYPT、雷达、ACCESS GRANTED |
| `blackhole` 黑洞 | 事件视界、连续光子环、吸积盘、引力透镜弧、喷流；随机方向流星；恒星被吸入计数；`--spin` 控制旋转速度 |
| `beams` 光束 | **单向**横向不规则光束，随机亮色，白热头/渐隐尾；`--density`/`--beam-speed` 调密度/速度 |
| `fireworks` 烟花 | 火箭升空，内外双层粒子环绽放，延迟爆裂；`--density` 调烟花密度；城市天际线/星空 |
| `solar` 太阳系 | 太阳 + 九大行星在一条轨道线上；圆形行星、不同颜色/大小，土星环、卫星、彗星；左上金色发散阳光 |
| `galaxy` 满天星河 | 满屏星海、中央稠密银河带；动态星团/暗尘/星座线；`--density`、`--twinkle`、`--meteors` 可调 |
| `buddha` 佛光普照 | **垂直居中**的 ASCII 佛祖、大号头光/光环、莲花座、曼陀罗、金色粒子与飘落花瓣，全身发光 |
| `tomjerry` 猫和老鼠 | Tom 追逐 Jerry 的太空站动画；默认放慢且不跳跃，`--speed` / `+` `-` 调速，`--jump` 开启跳跃 |
| `mario` 超级马里奥 | 自动闯关：顶问号砖块、弹出蘑菇、吃金币；Unicode 实心精灵，`--ascii` 经典字符版 |
| `ecg` 心率跳动 | 绿色心电监护仪：网格、P-QRS-T 波形、扫描线、HR/SpO2；Unicode / `--ascii` 两版 |
| `dandelion` 蒲公英 | 左侧多株盛开蒲公英，白色种子向右飘散；`--seed-count` / `--seed-speed` 调数量和速度 |

> 截图均为 120×36、truecolor、离屏渲染；动图见上。

### ASCII / Unicode 双版本

`mario` 和 `ecg` 默认 Unicode，`--ascii` 切换纯 ASCII：

| Unicode | ASCII |
| --- | --- |
| ![mario unicode](docs/mario.png) | ![mario ascii](docs/mario-ascii.png) |
| ![ecg unicode](docs/ecg.png) | ![ecg ascii](docs/ecg-ascii.png) |

```bash
trix mario            # Unicode 实心精灵
trix mario --ascii    # ASCII 经典字符
trix ecg              # Unicode 心电监护
trix ecg --ascii      # ASCII 心电监护
```

## 特性

- **15 个可切换特效**：支持命令行参数、运行时热切换与主题。
- **低资源**：单线程、无 async、无事件循环框架；默认 30 FPS；仅依赖 `libc`。
- **快速启动**：`--version` 中位数约 **2.0 ms**；发布版二进制约 **636 KB**。
- **低内存**：交互模式 RSS 约 **2.00–2.25 MB**（120×36，truecolor）。
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
./target/release/trix rain
```

安装到 `PATH`：

```bash
cargo install --path . --locked
trix dragon
```

### 一条命令一个特效

```bash
trix rain                          # 代码雨
trix fire --fps 60                 # 多堆细长火焰
trix dragon --theme gold           # 金色巨龙
trix map --theme ice               # 全息世界地图
trix matrix                        # NASA Unicode 大屏（--ascii 可降级）
trix blackhole                     # 黑洞吞噬恒星
trix beams                         # 横向随机亮色光束
trix fireworks                     # 双层烟花绽放
trix solar                         # 太阳系九大行星一线
trix galaxy                        # 满天星河 / 银河
trix buddha --theme gold           # 佛光普照
trix tomjerry --speed 0.8          # 默认 0.6x 左右，可调
trix mario                         # 超级马里奥（Unicode）
trix ecg                           # 心率跳动（Unicode）
trix dandelion                     # 蒲公英飘散
trix tomjerry --jump               # 开启跳跃
trix --list                        # 列出特效和主题

# 参数示例
trix matrix --ascii                         # 纯 ASCII 回退
trix blackhole --spin 0.25 --meteors 8      # 更慢旋转 + 更多流星
trix beams --density 1.5 --beam-speed 1.2   # 更密、更快
trix fireworks --density 2                  # 更多烟花
trix galaxy --density 1.5 --twinkle 2 --meteors 8
trix tomjerry --chase-speed 0.8 --jump      # 追逐速度 / 跳跃
trix mario --ascii                          # 马里奥 ASCII 版
trix ecg --ascii                            # 心电 ASCII 版
trix dandelion --seed-count 180 --seed-speed 1.4
```

## 命令行

```text
trix <effect> [options]

  -e, --effect <name>     与位置参数等价
  -f, --fps <1-240>       目标帧率                          [默认: 30]
  -s, --speed <0.05-8>    动画速度倍率                      [默认: 1.0]
      --chase-speed <f>   --speed 的别名（Tom & Jerry）
  -t, --theme <name>      配色主题                          [特效默认]
      --charset <mode>    kana | ascii | unicode              [matrix/mario/ecg/dandelion: unicode]
      --unicode           Force Unicode single-cell glyphs
      --ascii             Force ASCII fallback
      --color <mode>      auto | truecolor | 256 | 16 | mono [默认: auto]
      --density <f>       beams/fireworks/galaxy density    [默认: 1.0]
      --spin <f>          black hole rotation speed          [默认: 0.5]
      --twinkle <f>       galaxy twinkle rate                [默认: 1.0]
      --meteors <n>       black hole / galaxy meteors        [默认: 3]
      --beam-speed <f>    beams horizontal speed             [默认: 1.0]
      --seed-count <n>    dandelion seed count                [默认: 90]
      --seed-speed <f>    dandelion seed drift speed          [默认: 1.0]
      --jump              Tom & Jerry jumps                  [默认: off]
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
| `1`–`9`、`0`、`[`、`]` | 前 12 个特效 |
| `m` / `e` / `d` | 超级马里奥 / 心率跳动 / 蒲公英 |
| `t` | 循环主题 |
| `+` / `-` | 加速 / 减速（0.1×–8×），Tom & Jerry 同样生效 |
| `r` | 用新种子重开当前特效 |
| `i` | 显示 / 隐藏状态行（FPS、耗时、主题、尺寸、种子） |
| `h` / `?` | 帮助面板 |

数字键顺序：`1 rain`、`2 fire`、`3 dragon`、`4 map`、`5 matrix`、`6 blackhole`、`7 beams`、`8 fireworks`、`9 solar`、`0 galaxy`、`[ buddha`、`] tomjerry`；新增 `m mario`、`e ecg`、`d dandelion`。

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

## 自动构建与发布

仓库包含 [`.github/workflows/release.yml`](.github/workflows/release.yml)：

- 推送 `v*` tag 时自动构建并创建 GitHub Release。
- **macOS**：`aarch64-apple-darwin` + `x86_64-apple-darwin`，用 `lipo` 合并为 **universal** 二进制。
- **Linux**：`x86_64-unknown-linux-musl` 静态二进制，不依赖目标机器的 glibc。
- Release 资产：
  - `trix-<version>-macos-universal.tar.gz`
  - `trix-<version>-linux-x86_64.tar.gz`
  - `SHA256SUMS`
- 也可以在 GitHub Actions 页面手动 `Run workflow` 只构建 artifacts（不会创建 Release）。

发布新版本：

```bash
git tag -a v0.6.0 -m "trix v0.6.0"
git push origin master
git push origin v0.6.0
```

> macOS 二进制未做 Apple Developer 签名/公证；首次下载如被 Gatekeeper 拦截，可右键打开，或 `xattr -d com.apple.quarantine trix`。

## 性能

环境：Apple Silicon (ARM64)、macOS、Rust 1.96.1、`--release`（LTO + `codegen-units=1` + `panic=abort` + `strip`）。
画布 160×48，240 帧离屏压测：

| 特效 | 平均渲染 | p95 | diff 输出 |
| --- | ---: | ---: | ---: |
| rain | 0.080 ms | 0.101 ms | 14.3 KiB/帧 |
| fire | 0.454 ms | 0.548 ms | 68.3 KiB/帧 |
| dragon | 0.124 ms | 0.157 ms | 15.0 KiB/帧 |
| map | 0.161 ms | 0.189 ms | 43.6 KiB/帧 |
| matrix | 0.071 ms | 0.087 ms | 12.0 KiB/帧 |
| blackhole | 0.100 ms | 0.118 ms | 21.0 KiB/帧 |
| beams | 0.085 ms | 0.113 ms | 24.9 KiB/帧 |
| fireworks | 0.040 ms | 0.064 ms | 8.6 KiB/帧 |
| solar | 0.039 ms | 0.045 ms | 7.6 KiB/帧 |
| galaxy | 0.129 ms | 0.163 ms | 23.4 KiB/帧 |
| buddha | 0.061 ms | 0.072 ms | 11.4 KiB/帧 |
| tomjerry | 0.019 ms | 0.022 ms | 2.4 KiB/帧 |
| mario | 0.013 ms | 0.016 ms | 0.7 KiB/帧 |
| ecg | 0.038 ms | 0.046 ms | 7.3 KiB/帧 |
| dandelion | 0.025 ms | 0.030 ms | 4.0 KiB/帧 |

真实 PTY、truecolor、120×36、30 FPS、运行 5 秒（CPU = 子进程 `utime+stime / 5s`，含 PTY 写入，不含终端模拟器自身绘制）：

| 特效 | CPU（单核） | RSS | 5 秒输出 |
| --- | ---: | ---: | ---: |
| rain | 0.46% | 2.06 MiB | 1.4 MiB |
| fire | 1.38% | 2.09 MiB | 6.0 MiB |
| dragon | 0.70% | 2.25 MiB | 1.3 MiB |
| map | 0.67% | 2.22 MiB | 3.8 MiB |
| matrix | 0.35% | 2.22 MiB | 0.9 MiB |
| blackhole | 0.60% | 2.09 MiB | 2.2 MiB |
| beams | 0.52% | 2.03 MiB | 2.1 MiB |
| fireworks | 0.26% | 2.14 MiB | 0.7 MiB |
| solar | 0.33% | 2.05 MiB | 0.8 MiB |
| galaxy | 0.61% | 2.22 MiB | 1.9 MiB |
| buddha | 0.34% | 2.05 MiB | 1.0 MiB |
| tomjerry | 0.24% | 2.03 MiB | 0.2 MiB |
| mario | 0.24% | 2.03 MiB | 0.1 MiB |
| ecg | 0.28% | 2.05 MiB | 0.7 MiB |
| dandelion | 0.25% | 2.00 MiB | 0.5 MiB |

- 30 FPS 下约 33.3 ms/帧预算；最重的 `fire` 离屏渲染约 **1.4% 单核**（0.454 ms/帧），实际交互约 **1.4%**（含 PTY 写入）。
- 启动：`--version` 中位数约 **2.0 ms**；真实 PTY 首屏字节中位约 **1.6–2.4 ms**。
- 内存：真实 PTY 最大 RSS **2.00–2.25 MiB**。
- 二进制：**636 KiB**。
- 交互模式会遵守 `NO_COLOR`；离屏 `--snapshot` / `--bench` 默认 truecolor，可用 `--color` 覆盖。

```bash
scripts/bench.sh
./target/release/trix fire --bench --size 160x48 --frames 300
```

## 实现

- **终端驱动**（`src/term.rs`）：libc 设置 raw mode、备用屏幕、隐藏光标、关闭自动换行；非阻塞读键、`TIOCGWINSZ` 尺寸检测、SIGINT/SIGTERM/SIGHUP 标志、panic hook。
- **Canvas**（`src/canvas.rs`）：`Cell { char, fg }` 双缓冲；逐行变化区间；颜色变化才发 SGR；支持 truecolor/256/16/mono。
- **零框架**：唯一依赖 `libc`；没有 tokio/crossterm/线程任务。
- **代码雨**：逐列头/尾、随机速度与深度、二进制雾、横向数据包、CRT 扫描。
- **火焰**：Doom-fire 元胞传播；连续底部火床 + 随机 3–5 个燃料堆，中央堆最高；每堆细长火舌，顶部彼此分开；火星与周期爆燃。
- **巨龙**：飞行轨迹按弧长重采样，长蛇身 splat 到强度/深度缓冲；大型中国龙头、鹿角、龙须、鬃毛、四爪、背鳍、尾鳍、龙鳞；整屏 Lissajous 飞行。
- **世界地图**：Natural Earth 陆地掩膜、覆盖率采样、城市节点、大圆弧、扫描波束。
- **黑客帝国**：NASA 后台登录大屏。Unicode 蓝色 NASA 横幅（白色大字、红色 swoosh、白色星点、蓝白流光）、居中下移的 Unicode 边框登录框；OPERATOR/PASSWORD/2FA、AUTH/DECRYPT 进度、日志、雷达、ACCESS GRANTED；`--ascii` 提供纯 ASCII 回退。
- **黑洞**：事件视界遮罩、连续光子环、吸积盘、引力透镜弧、喷流、随机方向流星、恒星螺旋内落与“CONSUMED”计数；`--spin` 调吸积盘/内落旋转速度。
- **光束**：单向从左向右的随机光束，亮色头、渐隐尾；`--density`/`--beam-speed` 调密度与速度；星空和网格。
- **烟花**：火箭 + 内外双环粒子 + 延迟爆裂；粒子拖尾、重力、城市剪影、星空。
- **太阳系**：太阳与九大行星一维排列；圆形球体、土星环、卫星、彗星、小行星带；左上角金色发散阳光与全息星图网格。
- **银河**：预计算星点沿斜向银河带 + 中央核球；动态星团弧、暗尘、流星、星座线；`--density`/`--twinkle`/`--meteors` 可调。
- **佛光**：垂直居中的 ASCII 佛祖（头光、面部、肩部、莲座）、多层光环/曼陀罗/射线、金色粒子与花瓣；逐字符发光。
- **Tom & Jerry**：太空站走廊；运行帧动画；默认慢速、不跳跃；`--speed`/`+`/`-` 调速，`--jump` 开启跳跃；HUD 显示倍率。
- **超级马里奥**：自动奔跑/跳跃，问号砖块顶出蘑菇、金币与分数；滚动天空/山丘/地面/板栗仔；Unicode 实心精灵与 `--ascii` 版本。
- **心率跳动**：ECG 网格、P-QRS-T 波形、扫描线、HR/SpO2/SYS-DIA；Unicode 框线/实心波形与 `--ascii` 版本。
- **蒲公英**：左侧 4 株蒲公英，白色种子向右飘散；`--seed-count`/`--seed-speed` 调种子数量/速度。
- **地图数据离线生成**：`tools/gen_land.py` 下载 Natural Earth 并栅格化为 `src/data/land_720x360.bin`，运行时不访问网络。

## 录制与文档工具

```bash
trix dragon --snapshot dragon.ans --size 120x36 --frames 145 --seed 7
python3 tools/ansi2png.py dragon.ans dragon.png --size 120x36 --label "trix dragon"
python3 tools/render_gif.py dragon docs/dragon.gif --start 100 --frames 24 --step 3
```

## 兼容性

- 支持 macOS、Linux、WSL（Unix termios）；原生 Windows 未实现。
- 推荐 24-bit truecolor；`--color auto` 会自动降级。
- `matrix` 默认 Unicode 单宽字符（蓝白大字、Unicode 边框/进度条）；`--ascii` 或 `--charset ascii` 回退；`--charset kana` 可混入半角片假名；`mario`/`ecg`/`dandelion` 默认 Unicode，`--ascii` 切换 ASCII 版。
- 非 TTY 会提示使用 `--snapshot`。
- 画布黑底；建议深色终端。

## 测试

```bash
cargo test
cargo clippy --all-targets
cargo fmt --check
```

当前 **42 个**单元/集成测试：颜色转换、按键解析、CLI/新参数校验、Canvas 裁剪/差分、地图掩膜、15 个特效多尺寸/主题/颜色/Unicode 降级、小终端不 panic、确定性种子、Tom & Jerry 速度/跳跃等。

真实 PTY 实测：`q`/`Esc`/`Ctrl-C`/`SIGTERM`、暂停、帮助、数字/`[`/`]` 切换、`i` 状态、缩放、`--no-alt`、颜色降级、退出恢复。

## 版本与回退

- **当前版本：0.6.0**（`trix --version`），tag **`v0.6.0`**。
- 上一版：tag **`v0.5.1`**（自动发布流水线）；**`v0.5.0`**（项目更名为 trix）。
- 更早：tag **`v0.4.0`**（Unicode NASA 登录、连通火焰、可调参数）；**`v0.3.0`**（多堆火焰、7 个新特效）；**`v0.2.0`**（中国龙、NASA 入侵）；**`v0.1.0`**（初始基线）。
- 回退：
  ```bash
  git checkout v0.5.1     # 只看上一版（detached HEAD）
  git switch master
  git reset --hard v0.5.1 # 彻底退回上一版
  ```
- 额外备份（不进 git）：`backups/trix-v0.1.0-*` 到 `trix-v0.6.0-*`，含源码 tar 与 aarch64 macOS 二进制。
- 改动记录：[CHANGELOG.md](CHANGELOG.md)。

## 数据来源、参考与许可

- 陆地轮廓：[Natural Earth](https://www.naturalearthdata.com/) 1:110m land，**public domain**，经 [natural-earth-vector](https://github.com/nvkelso/natural-earth-vector) GeoJSON 转换。
- Doom-fire 灵感：[pserey/doom-fire-algorithm](https://github.com/pserey/doom-fire-algorithm)、[horizonwiki/fire](https://github.com/horizonwiki/fire)；本项目的 Rust 实现、多燃料堆、配色与渲染均为独立编写。
- 佛祖 ASCII 艺术改编自 [ascii.co.uk/art/buddha](https://ascii.co.uk/art/buddha)（原作者署名 `hjw`），用于 `buddha` 场景；本项目补充了颜色、光环、莲座、粒子与动画。
- 本项目代码：[MIT](LICENSE)。

---

```text
  t r i x  ::  stay curious, look up.
```
