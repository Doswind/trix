# trix-tui HUD 与快捷键设计

本文记录 trix-tui 的统一 HUD、文字归属和跨平台按键契约。修改场景文字或快捷键时，请同步更新本文、`src/hud.rs`、`src/help.rs`、`src/meta.rs`、README 与 CHANGELOG。

## 1. 三种 HUD 模式

| 模式 | 顶栏 | 底栏 | 场景文字 | 适用 |
| --- | --- | --- | --- | --- |
| Full | 有 | 有 | 仅图内安全区 | 其余 23 个特效 |
| Ambient | 无 | 有 | 无 | `rain`、`fire`、`monalisa` |
| Clean | 无 | 无 | 保留（只隐藏 HUD） | 交互时加 `--no-hud` |

`Ambient` 只去掉顶栏和场景文字，底栏与播放列表行保持和其他特效完全一致；`--no-hud` 只隐藏顶栏/底栏，场景内的拟真文字仍然绘制，`h` 帮助、`i` 信息面板、暂停框也仍然可用。

## 2. 版式

```text
row 0     Code Rain  // rain                    1.00x | neon | ascii
row 1     ┌──────────── 场景安全区 ────────────────────────────┐
 ...      │  画面 + 图内拟真标签（ECG/Matrix/Mario/Life 等）  │
row h-3   │                                                   │
row h-2   │  playlist 03/26  Code Rain  next 8s                │
row h-1   └  q quit | space pause | n/p next | t theme | h help   i info
```

规则：

1. row 0 只归顶栏；`Ambient` 不用 row 0。
2. row h-1 只归底栏。
3. row h-2 保留给播放列表；diegetic 文字放在 `1..=h-3`（Ambient 为 `0..=h-3`）。
4. 中心区域只放模态：暂停框、帮助面板、`i` 信息面板、切换闪屏。
5. 场景不得绘制运行时按键提示或 CLI 参数提示。
6. 动态计数通过 `Scene::hud_status()` 返回；短、ASCII、≤60 字符。
7. 底栏纯 ASCII，窄屏按固定档位降级；顶栏右侧按“场景遥测 → 速度 → 主题 → 字符集”的顺序丢弃。

## 3. 文字迁移

| 场景 | 处理 |
| --- | --- |
| rain / fire / monalisa | Ambient；无顶栏、无场景文字、保留底栏 |
| beams / buddha / dragon / solar | 删除标题和右上装饰口号；solar 保留行星标签 |
| blackhole / fireworks / galaxy | 标题删除；`CONSUMED` / `BURSTS` / `STARS` 迁顶栏 |
| ballet / dandelion / demoneye / storm / tornado | 标题删除；`TEMPO` / `SEEDS` / `SCANNING` / `STRIKES` / `FUNNELS` 迁顶栏 |
| fish / traffic | 标题删除；`FISH` / `HORIZONTAL|ASCII` 迁顶栏；删除底部假提示和 `--street` 提示 |
| dandelion / tornado | 删除底部 `--seed-*` / `--tornado-*` 提示；参数改由 `i` 面板显示 |
| landscape / shanghai / tiananmen | 删除标题和底部装饰口号 |
| ecg | `HR` 迁顶栏；SpO2/血压移到 h-4，走纸速度移到 h-3 |
| mario | 计分板从 row 0 下移到 row 1；删除 `MUSHROOM RUN // PRESS JUMP` 和底部字符集徽章 |
| matrix | 顶部 NASA 标题删除；`AUTH` / `ACCESS GRANTED` / `TRACE DETECTED` 迁顶栏；底部状态条删除 |
| map | 顶部标题/UTC 和底部统计删除；`NODES/LINKS/SCAN/UTC` 迁顶栏 |
| life | 标题删除；`AGE/CYCLE` 迁顶栏；故事字幕保留在 h-4 |

## 4. 快捷键契约

| 功能 | 规范键 | 兼容别名 | 说明 |
| --- | --- | --- | --- |
| 退出 | `q` | `Q`、`Ctrl-C` | 任何面板打开时都直接退出 |
| 关闭面板 | `Esc` | — | 有面板先关闭；没有面板时退出 |
| 暂停/继续 | `Space` | — | 底栏显示 `space resume` |
| 下一个 | `n` | `Right`、`Down` | |
| 上一个 | `p` | `Left`、`Up` | |
| 帮助 | `h` | `?` | toggle |
| 信息/参数 | `i` | — | 打开/关闭 Info 面板 |
| 主题 | `t` | — | |
| 加速 | `+` | `=` | 0.1×–8× |
| 减速 | `-` | `_` | 0.1×–8× |
| 重开 | `r` | — | 新种子 |
| 直接切换 | `1-9 0 [ ]` + 字母 | 大小写均可 | 映射见 README |

快捷键不区分大小写；不提供 `,`/`.` 和 PageUp/PageDown 别名。

## 5. 输入解析

`src/term.rs` 中的 `InputParser` 是一个带状态、带超时的字节解码器：

- **分片序列**：单独的 `ESC` 等待约 40 ms；完整的 CSI/SS3 序列在 140 ms 内等待后续字节。SSH/tmux/macOS 下不会把方向键拆成“Esc 退出”。
- **CSI/SS3**：支持 `ESC[A`、`ESC OA`、`ESC[1;2A`、`ESC[1~`、`ESC[7~`、`ESC OH` 等；箭头上的修饰参数忽略。
- **Alt/Option**：`ESC` + 普通字符视为未知组合键吞掉，不拆成 `Esc` + 字符。
- **非按键上报**：鼠标 `ESC[<...M/m`、括号粘贴 `ESC[200~...201~`、焦点事件 `ESC[I/O`、OSC/DCS 字符串全部忽略。
- **UTF-8**：完整解码一个字符；非 ASCII 输入不触发快捷键。
- **控制字节**：只有 `0x03` → `Ctrl-C`；Backspace（`0x08`/`0x7f`）、Enter、Tab、Ctrl-Z 等保持忽略，避免 Backspace 误开帮助。
- **大小写**：应用层统一 `to_ascii_lowercase()`，Shift、Caps Lock、外接键盘一致。

macOS `Cmd` 组合键被 Terminal.app/iTerm2 截获，TUI 收不到，只能在文档中说明。

## 6. `i` 信息/参数面板

`i` 打开一个居中面板，显示：

- `EFFECT`：英文名 + key；
- `RUNNING`：fps、帧耗时、速度、主题、字符集、尺寸、颜色模式；
- `SEED`：当前种子；
- `STATUS`：当前场景遥测（与顶栏一致，窄屏顶栏放不下时仍可在这里看到）；
- `PLAYLIST`：播放列表模式下的进度、切换秒数、过渡模式、倒计时；
- `PARAMS`：当前场景真正相关的 CLI 参数；
- `KEYS`：关闭/帮助/退出提示。

场景专属参数：

| 场景 | 参数 |
| --- | --- |
| beams | `--density`、`--beam-speed`、`--beam-motion` |
| blackhole | `--spin`、`--meteors` |
| galaxy | `--twinkle`、`--meteors`、`--density` |
| fireworks | `--density` |
| traffic | `--density`、`--street` |
| dandelion | `--seed-count`、`--seed-speed` |
| tornado | `--tornado-count`、`--tornado-speed` |
| life | `--life-cycle` |
| demoneye | `--eye-motion` |
| monalisa | `--ghost`、`--gallery` |
| dragon | `--plain` |
| tomjerry | `--jump` |

公共参数：`--theme`、`--charset`、`--fps`、`--speed`、`--color`；播放列表模式下追加 `--playlist`、`--switch`、`--transition`、`--transition-secs`；设置 `--duration` 时也会显示。

## 7. 维护约束

- 场景不得向 row 0 / h-1 写文字。测试用 `Canvas` 的 chrome guard 在 `scene.draw()` 期间强制检查。
- 所有场景的 `hud_status()` 必须是 ASCII 且不超过 60 字符。
- 帮助面板、`--help`、README 的按键表必须与 `app.rs` 的分发逻辑一致；快捷键和默认值来自 `src/meta.rs`。
- 新增带参数的 effect 时，在 `src/meta.rs::EffectId::tuning()` 增加对应项，并更新本文；`app.rs::info_params` 和 `--help <effect>` 会共用这份元数据。
- 新增刻意无文字的特效时，在 `EffectId::hud_mode()` 中返回 `HudMode::Ambient`，并更新 `only_the_text_free_effects_are_ambient` 测试。

## 8. 命令行帮助

- `trix --help`：总览（字标、真实 HUD 预览、26 个特效网格、分组参数、主题色带、快捷键、示例）。
- `trix --help <effect>`：单特效详情 + 用真实场景渲染的预览 + 该特效专属参数；`trix <effect> --help` 等价。
- `trix --help all`：完整参数手册；`trix --list`：TTY 下彩色目录，管道下纯文本。
- `trix --completions bash|zsh|fish`：从同一份特效/参数元数据生成 shell 补全。
- 非 TTY、`NO_COLOR`、`--no-color` 时输出对齐的 ASCII/单色文本，不输出 ANSI；`--ascii` / `--unicode` 只影响帮助边框，不改变特效默认字符集。
- 未知特效/参数会给出编辑距离建议；帮助布局按 `COLUMNS`/ioctl 在 40–140 列间自适应。
- README 截图默认使用 ASCII 字形；Unicode 截图只出现在明确介绍 `--unicode` 的章节。

## 9. 验证

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets`
- `python3 scripts/check_docs.py`
- 26 个特效 × 80×24 / 120×36 / 200×60 × ASCII/Unicode 的 `--snapshot` 冒烟
- 真实终端：macOS Terminal.app / iTerm2，Linux xterm / VTE / tmux；重点检查分片方向键、Shift+方向键、Option+q、鼠标/粘贴上报、`--no-hud` 与面板。
