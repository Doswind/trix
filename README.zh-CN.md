# trix-effects

[English](README.md) | **简体中文**

> **终端里的 35 个 ASCII / Unicode 动态特效，装进一个快速、零依赖的 Rust 单文件。** 代码雨、火焰、巨龙、黑客帝国、黑洞、烟花、太阳系、蒙娜丽莎，再加巨型 DeepSeek Harness 字标和自动打字的 Claude Code 会话——默认 ASCII，Unicode 显式开启。

![release](https://github.com/Doswind/trix-effects/actions/workflows/release.yml/badge.svg)
![CI](https://github.com/Doswind/trix-effects/actions/workflows/ci.yml/badge.svg)
![license](https://img.shields.io/badge/license-MIT-blue.svg)
![rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
![platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20WSL-lightgrey.svg)

![trix-effects 总览](docs/overview-ascii.png)

**35 个特效 · 8 套主题 · 零依赖 · 启动约 2 ms · 二进制约 1.1 MB · macOS / Linux / WSL**

[安装](#安装) · [快速开始](#快速开始) · [帮助](#像产品一样的帮助) · [特效](#特效) · [ASCII / Unicode](#ascii--unicode-双版本) · [快捷键](#运行时快捷键) · [主题](#主题) · [性能](#性能) · [常见问题](#兼容性与常见问题)

> 命令叫 **`trix`**，包名和仓库叫 **`trix-effects`**——和 `ripgrep` / `rg` 一样，短命令 + 唯一包名。项目原名 `texiao`，v0.5.0 改名为 `trix`，v0.9.1 起包名改为 `trix-effects`。

## 安装

```bash
# macOS / Linux，预编译（使用最新 GitHub Release）
curl -fsSL https://raw.githubusercontent.com/Doswind/trix-effects/master/install.sh | sh

# 有 Rust 工具链，直接从 git 安装
cargo install --git https://github.com/Doswind/trix-effects --locked

# 从克隆安装
git clone https://github.com/Doswind/trix-effects
cd trix-effects
cargo install --path . --locked
```

安装后的命令是 `trix`，没有运行时依赖。Release 资产包含 Linux `x86_64` / `aarch64` 静态二进制和 macOS universal 二进制；`install.sh` 会先校验 `SHA256SUMS`，默认装到 `$HOME/.local/bin`（可用 `TRIX_INSTALL_DIR` 覆盖）。

从源码构建需要 Rust 1.70+。

## 快速开始

```bash
trix                              # 循环播放全部 35 个特效
trix rain                         # 代码雨
trix dragon --theme gold          # 金色巨龙
trix blackhole --spin 0.25        # 更慢的黑洞
trix monalisa                     # 蒙娜丽莎（静态，默认 ASCII）
trix monalisa --ghost             # 纯 ASCII 鬼影版
trix matrix --unicode             # 黑客帝国 Unicode 版
trix --playlist random --switch 8 # 随机顺序，每个 8 秒
trix --help rain                  # 单特效说明 + 实时预览
trix --list                       # 特效目录 + 主题
trix rain --snapshot rain.ans --size 120x36 --frames 90
```

更多单条命令：

```bash
trix fire --theme ember            # 暖色火焰
trix dragon --plain                # 纯星空背景
trix map --theme ice               # 全息世界地图
trix beams --beam-motion jump      # 起伏光束
trix fireworks --density 2         # 更多烟花
trix galaxy --density 1.5 --twinkle 2 --meteors 8
trix mario                         # 默认 ASCII
trix ecg                           # 默认 ASCII
trix dandelion --seed-count 180 --seed-speed 1.4
trix traffic --street vertical     # 纵向车流
trix tornado --tornado-count 5 --tornado-speed 1.6
trix ballet --speed 1.3            # 更快的冰上芭蕾
trix demoneye --eye-still          # 眼球固定，保留眨眼
trix life --life-cycle 60          # 1 分钟走完 0～100 岁
trix dsh                           # DEEPSEEK HARNESS 字标
trix dsh --trail 0                 # 关闭丢帧残影，只留光束
trix vibe --unicode                # Claude Code 符号，自读源码
trix tunnel                        # 星际隧道
trix plasma                        # 等离子场
trix launch                        # 从发射台到轨道
trix tomjerry --jump               # 开启跳跃
```

## 像产品一样的帮助

`trix --help` 不是一段朴素的参数列表：它会渲染字标、真实 HUD 画面、按主题着色的特效网格、分组参数、主题色带和可直接复制的示例。`trix --help <effect>` 还会用**真实场景渲染一张预览图**，并列出该特效自己的参数。

![trix --help](docs/help.png)

![trix --help rain](docs/help-rain.png)

[完整帮助截图](docs/help-full.png)

```bash
trix --help            # 总览
trix --help rain       # 单特效详情 + 真实预览
trix --help all        # 完整参数手册
trix --list            # TTY 下彩色目录；管道下纯文本
```

输出不是 TTY、设置了 `NO_COLOR` 或加了 `--no-color` 时，会自动退化为对齐的纯文本；`--ascii` 强制 ASCII 边框，`--unicode` 强制 Unicode 边框。未知特效/参数会给出“did you mean”建议。

## 特效

所有特效都给出中英文名；`trix --list` 使用同一套名称。

| id | 中文 | English |
| --- | --- | --- |
| `rain` | 代码雨 | Code Rain |
| `fire` | 燃烧火焰 | Blazing Fire |
| `dragon` | 巨龙盘旋 | Coiling Dragon |
| `map` | 世界地图 | World Map |
| `matrix` | 黑客帝国 | The Matrix |
| `blackhole` | 黑洞 | Black Hole |
| `beams` | 光束 | Light Beams |
| `fireworks` | 烟花 | Fireworks |
| `solar` | 太阳系 | Solar System |
| `galaxy` | 满天星河 | Starry Galaxy |
| `buddha` | 佛光普照 | Buddha's Halo |
| `tomjerry` | 猫和老鼠 | Tom & Jerry |
| `mario` | 超级马里奥 | Super Mario |
| `ecg` | 心率跳动 | Heartbeat Monitor |
| `dandelion` | 蒲公英 | Dandelion |
| `fish` | 水族馆 | Aquarium |
| `landscape` | 五光十色 | Colorful Landscape |
| `shanghai` | 东方明珠 | Oriental Pearl |
| `tiananmen` | 光芒万丈 | Tiananmen Radiance |
| `traffic` | 车水马龙 | Traffic Flow |
| `tornado` | 龙卷风 | Tornado |
| `storm` | 电闪雷鸣 | Thunderstorm |
| `monalisa` | 蒙娜丽莎 | Mona Lisa |
| `ballet` | 冰上芭蕾 | Ice Ballet |
| `demoneye` | 恶魔之眼 | Demon Eye |
| `life` | 生命的轨迹 | Threads of Life |
| `dsh` | 深度求索 | DeepSeek Harness |
| `vibe` | 氛围编程 | Vibe Coding |
| `tunnel` | 星际隧道 | Warp Tunnel |
| `helix` | DNA 双螺旋 | DNA Helix |
| `lights` | 极光 | Aurora |
| `volcano` | 火山爆发 | Volcano |
| `tesseract` | 四维超立方 | Tesseract |
| `launch` | 火箭发射 | Rocket Launch |
| `plasma` | 等离子 | Plasma |

> **截图使用默认 ASCII 字形。** Unicode 版本只出现在 [ASCII / Unicode](#ascii--unicode-双版本) 一节。所有预览都由当前代码渲染，120×36（蒙娜丽莎 120×60；深度求索 160×48），truecolor。

| 代码雨 `rain` | 燃烧火焰 `fire` | 巨龙盘旋 `dragon` |
| --- | --- | --- |
| ![代码雨](docs/rain-ascii.png) | ![燃烧火焰](docs/fire-ascii.png) | ![巨龙盘旋](docs/dragon-ascii.png) |

| 世界地图 `map` | 黑客帝国 `matrix` | 黑洞 `blackhole` |
| --- | --- | --- |
| ![世界地图](docs/map-ascii.png) | ![黑客帝国](docs/matrix-ascii.png) | ![黑洞](docs/blackhole-ascii.png) |

| 光束 `beams` | 烟花 `fireworks` | 太阳系 `solar` |
| --- | --- | --- |
| ![光束](docs/beams-ascii.png) | ![烟花](docs/fireworks-ascii.png) | ![太阳系](docs/solar-ascii.png) |

| 满天星河 `galaxy` | 佛光普照 `buddha` | 猫和老鼠 `tomjerry` |
| --- | --- | --- |
| ![满天星河](docs/galaxy-ascii.png) | ![佛光普照](docs/buddha-ascii.png) | ![猫和老鼠](docs/tomjerry-ascii.png) |

| 超级马里奥 `mario` | 心率跳动 `ecg` | 蒲公英 `dandelion` |
| --- | --- | --- |
| ![超级马里奥](docs/mario-ascii.png) | ![心率跳动](docs/ecg-ascii.png) | ![蒲公英](docs/dandelion-ascii.png) |

| 水族馆 `fish` | 五光十色 `landscape` | 东方明珠 `shanghai` |
| --- | --- | --- |
| ![水族馆](docs/fish-ascii.png) | ![五光十色](docs/landscape-ascii.png) | ![东方明珠](docs/shanghai-ascii.png) |

| 光芒万丈 `tiananmen` | 车水马龙 `traffic` | 龙卷风 `tornado` |
| --- | --- | --- |
| ![光芒万丈](docs/tiananmen-ascii.png) | ![车水马龙](docs/traffic-ascii.png) | ![龙卷风](docs/tornado-ascii.png) |

| 电闪雷鸣 `storm` | 蒙娜丽莎 `monalisa` | 冰上芭蕾 `ballet` |
| --- | --- | --- |
| ![电闪雷鸣](docs/storm-ascii.png) | ![蒙娜丽莎](docs/monalisa-ascii.png) | ![冰上芭蕾](docs/ballet-ascii.png) |

| 恶魔之眼 `demoneye` | 生命的轨迹 `life` | 深度求索 `dsh` |
| --- | --- | --- |
| ![恶魔之眼](docs/demoneye-ascii.png) | ![生命的轨迹](docs/life-ascii.png) | ![深度求索](docs/dsh-ascii.png) |

| 氛围编程 `vibe` |
| --- |
| ![氛围编程](docs/vibe-ascii.png) |

| 星际隧道 `tunnel` | DNA 双螺旋 `helix` | 极光 `lights` |
| --- | --- | --- |
| ![星际隧道](docs/tunnel-ascii.png) | ![DNA 双螺旋](docs/helix-ascii.png) | ![极光](docs/lights-ascii.png) |

| 火山爆发 `volcano` | 四维超立方 `tesseract` | 火箭发射 `launch` |
| --- | --- | --- |
| ![火山爆发](docs/volcano-ascii.png) | ![四维超立方](docs/tesseract-ascii.png) | ![火箭发射](docs/launch-ascii.png) |

| 等离子 `plasma` |
| --- |
| ![等离子](docs/plasma-ascii.png) |

<details>
<summary><b>完整特效说明（35 个）</b></summary>

| 特效 | 说明 |
| --- | --- |
| 代码雨 (Code Rain) `rain` | 逐列头/尾、随机速度与深度、白热头、**蓝/绿双色尾**、二进制雾、横向数据包、CRT 扫描 |
| 燃烧火焰 (Blazing Fire) `fire` | **底部合并成一整片宽火床**；中间火苗最高，两侧火舌向左右倾斜散开；火星与爆燃 |
| 巨龙盘旋 (Coiling Dragon) `dragon` | 整屏 Lissajous 飞行的中国龙：弧长跟随龙身、鹿角、龙须、鬃毛、獠牙、四爪、背鳍、尾鳍、龙鳞；全身行波蜿蜒（从头向尾传播）；默认背景加入两层水墨远山与更浓的流动云海；`--plain` 恢复原纯星空背景 |
| 世界地图 (World Map) `map` | 内嵌 Natural Earth 陆地掩膜、城市节点、大圆弧数据包、扫描波束、UTC 时钟 |
| 黑客帝国 (The Matrix) `matrix` | **NASA 后台登录大屏**：**默认 ASCII**，`--unicode` 切换蓝色 NASA 横幅 + 白色大字 + 红色 swoosh、Unicode 登录框；OPERATOR/PASSWORD/2FA、AUTH/DECRYPT、雷达、ACCESS GRANTED |
| 黑洞 (Black Hole) `blackhole` | 事件视界、光子环、吸积盘、透镜弧、喷流、流星；**外围星星呈 "/" 倾斜的饱满椭圆环绕**，整体铺满屏幕、外缘压住上下边；最外圈为 `.` / `:` / `+` 低密度散点，中段补密；整体同向差速旋转（内快外慢），看起来是一个完整转动的椭圆盘 |
| 光束 (Light Beams) `beams` | **单向**横向光束，随机亮色；`--beam-motion straight\|jump`（默认直线），`--density`/`--beam-speed` |
| 烟花 (Fireworks) `fireworks` | 火箭升空，内外双层粒子环绽放，延迟爆裂；`--density` 调烟花密度；城市天际线/星空 |
| 太阳系 (Solar System) `solar` | **八大行星**（无冥王星）；太阳在左上角，光线向右下扇形分层扩散，并截止在行星线上方；圆形行星、土星环、卫星、彗星 |
| 满天星河 (Starry Galaxy) `galaxy` | 满屏星海、中央稠密银河带；动态星团/暗尘/星座线；`--density`、`--twinkle`、`--meteors` 可调 |
| 佛光普照 (Buddha's Halo) `buddha` | 外扩曼陀罗环**等距扩散**：所有环共用扩散相位、径向间距恒定；**首层从整尊佛像（佛身、头顶光环、莲座）之外起扩**，不压佛像也不与头光重叠；最多 5 层；环亮度压在佛像金色之下（内层 0.6），并由内向外逐层变暗；头光/莲座/粒子保留 |
| 猫和老鼠 (Tom & Jerry) `tomjerry` | Tom 追逐 Jerry 的太空站动画；默认放慢且不跳跃，`--speed` / `+` `-` 调速，`--jump` 开启跳跃 |
| 超级马里奥 (Super Mario) `mario` | **纯字符版 1-1 关卡**：云、山、灌木，砖面带灰缝的地面，**问号块/砖块/已用块**砖行，绿管道、旋转金币、板栗仔与蘑菇；自动跑跳、顶砖出币/蘑菇、踩扁敌人、`+200/+100/+1000` 飘字。**完全不使用终端背景色**：默认 ASCII 彩色字符（`# o = *`）；`--unicode` 只把**精灵**（马里奥、板栗仔、蘑菇、金币）换成 `█ ▀ ▄` 实心块，山/云/管道/砖块仍是字符并按物体配色 |
| 心率跳动 (Heartbeat Monitor) `ecg` | 绿色心电监护仪；**约 56 BPM，随机心搏间隔**，上下幅度、QRS 宽度、平直段长度明显随机；**默认 ASCII**，`--unicode` 切换 |
| 蒲公英 (Dandelion) `dandelion` | 3 丛×2 茎、6 个花头；左右两丛根/下段向中间弯曲；**花头尺寸 ×1.5**，其它不变 |
| 水族馆 (Aquarium) `fish` | 全屏水族箱：**5 条手绘 sprite 大鱼**（small 20×5 / medium 30×7 / large 38×8，分叉尾 + 亮眼 + 嘴，只摆尾不转头、嘴和头固定不动），外加 8～30 条 `<==o>` / `<=-o>` 小鱼（约 35% 双层、15% 三层加粗，只留一个眼睛）；气泡用 `o` / `O` / `8` 三种字符、**每条随机深浅色**，各自沿固定的一列**缓慢直线上升**，整个上升过程左右摆动不超过 3 次；数量收敛到 10～36 颗；水草随机高矮/间距并成小片；默认 ASCII，`--unicode` 切换 `█ ▓ ▒` |
| 五光十色 (Colorful Landscape) `landscape` | 层叠远山、平湖、小船、白鹅、两侧丛林；水面/雾气/花瓣动态 |
| 东方明珠 (Oriental Pearl) `shanghai` | 上海外滩夜景、东方明珠、高楼霓虹、黄浦江倒影；**江面随机两条大小/形状/位置都不同的船**（中式帆船 / 货船 / 渡轮三种船型，含甲板、舷窗、船楼窗光、烟囱冒烟、桅杆航灯、尾迹与水面倒影）；**每 3–5 秒夜空中飘出 1–2 颗流星**（白热头部 + 渐隐尾迹）；**默认 ASCII**，`--unicode` 可切换 |
| 光芒万丈 (Tiananmen Radiance) `tiananmen` | 放大到约 92% 宽 / 62% 高的天安门：双层金瓦飞檐、红柱城楼与国徽、红墙、白石门套五门洞、白框标语与中央画像、红旗与两侧小楼；金色光束先从城楼后方向外向上生成，再按光束由内向外依次循环点亮 |
| 车水马龙 (Traffic Flow) `traffic` | 城市街道两侧高楼、霓虹、车流；横向 3 行车（车顶/车身/车轮，8–16 格长）分 4–5 条车道、车道间有虚线，方向按车道交替；纵向 6 列俯视车（5–9 行，带前后灯与侧轮）；`--street horizontal\|vertical`；`--density` 调车流密度；**默认 ASCII**，`--unicode` 可切换 |
| 龙卷风 (Tornado) `tornado` | 1–8 股龙卷风从左向右；扭曲、底部收尖、顶部更宽；整体比原来**更矮一档**（0.28～0.58 屏高）且**更淡更疏**（亮度 0.42～0.85、每行 20 颗）；底部有 30 颗碎屑**向外飘散** |
| 电闪雷鸣 (Thunderstorm) `storm` | 乌云、斜雨、城市剪影、闪电；屏幕上方叠一层**灰白、不规则的高云**（较密、底部淡出、**缓慢向右飘移**（约 3 列/秒））与原有云带形成层次；落雷时全屏白色亮闪（[效果图](docs/storm-flash-ascii.png)） |
| 蒙娜丽莎 (Mona Lisa) `monalisa` | 去背景人物像：从参考像素画烘焙的人物（头发、脸、颈肩、交叠双手），ANSI 色块承载肤色/明暗 + ASCII 字符勾细节；没有山水/画框/浮尘；人物横向加宽约 12%，更接近原作体态；**画面静态，不眨眼**；`--unicode` 用半块提高垂直分辨率；`--ghost` 进入原纯 ASCII“鬼影”版（无背景色块、保留眨眼，[静态预览](docs/monalisa-ghost.png) / [动图](docs/monalisa-ghost.gif)）；可选 `--gallery` 在人物后方加淡暖色展厅光晕与地面微光（[预览](docs/monalisa-gallery.png)） |
| 冰上芭蕾 (Ice Ballet) `ballet` | 冰场双人舞：39 个场景秒的编舞循环 13 个舞段；默认 2× 节奏约 19.5 秒一轮（滑行、压步、arabesque、promenade、**头顶托举**、联合旋转、death spiral、sissonne 跳跃、Ina Bauer、收势），关节用两骨 IK 求解、牵手自动对齐，四肢带柔边并延迟跟随躯干；整场滑行覆盖大部分画面，围板后有三排会摇晃/举手/打闪光灯的观众，冰面留下冰刀刻痕与倒影；`--speed` 调速 |
| 恶魔之眼 (Demon Eye) `demoneye` | 整屏一只巨大血眼：眨眼每 8～15 秒一次、单眨或双眨（约 7:3），每两次眨眼之间只甩出 1～2 次（两次跳动相隔 ≥5 秒），约 0.1 秒到位、0.45～0.85 秒内必回正中直视观众；竖瞳缩放、眼睑压暗、血丝与眨眼冲击波；`--eye-motion dart\|still`（默认 dart），`--eye-still` 让眼球钉在正中 |
| 生命的轨迹 (Threads of Life) `life` | 0～100 岁时间轴与左右生命线；人类统一 `@` 符号 + 同排彩色大写简写 + **家属本人年龄**（非亲属朋友不带年龄）；亮光束随年龄增长（年长后可覆盖大部分生命），由尾部的暗点/短划线渐亮到 `=`，头部为 `****@` 彗尾；生命最初约 22%～42%（最多约 22 年）保留为稀疏暗淡的点；每条轨迹结束后其行位可被后来者复用（学龄 `PAL/UNI` 的 +1 行 → GF/WIFE，`DOG` 的 -1 行 → KID2，猫/父母的行位也会被后来的朋友接回）；`DOG` 终身在 -1 紧挨我；`PAL/UNI/WORK/FRIEND` 同时出现时**共用同一行**：学龄期在 +1 另一侧，有未成年 KID 时整组退到 KID/CAT 外侧最近的空行（通常为 +4；只让出孩子/宠物实际占用的行，不再固定 -5/-7），所有 KID 成年后回到内侧；`ME 0 / WIFE +1`；`KID1 +2`、`KID2 -1`（复用 DOG 让出的紧邻行）、`KID3 -2`；成年前 `KID3 -2 / KID2 -1 / ME 0 / WIFE +1 / KID1 +2` **连续无空行**，`GRAND1/GRAND2` 始终紧贴父辈一行（父辈成年漂移后随之移动）；`CAT +3` 与第一个孩子同时出生，紧贴 KID1 下方；`DAD/MOM +3/+4` 在孩子出生前两年开始让位到 `+5/+6`，`BRO +5→+7` 紧随让位；GPA/GMA/DAD/MOM 在我 0 岁时随机得到合适起始年龄（58～70 / 56～68 / 25～33 / 23～31），BRO 0～4，ME 0；KID/GRAND/CAT/DOG 从各自出生起算，GF/WIFE 与我同龄上下 2 岁；所有行固定为 1 个终端行（不随终端高度缩放，高终端也不会因取整出现空行）；`ME` 结束时触发约 6 秒的**超新星终章**（回闪与星屑放慢、彗星保持原速；白光核心、一生回闪、冲击波、星屑、加亮彗星与顶部永久光晕；回闪扫描光柱到达 ME 的停止线后以三列宽永久保留，不再消失）；死亡点与顶部星标保留 `@` + 名称 + 最终年龄，并以每秒 1 次同步显现/消失；所有死亡角色升到顶部按死亡顺序隔行排列，终点标签在流星抵达后 0.5 秒淡入；人类 `*`、猫狗 `+` 闪烁；GF 用 2 年靠近、再停留 1～2 年变 WIFE；WIFE 75～90 岁、晚于我化星；AGE 无前导零；`--life-cycle` 控制一轮秒数 |
| 深度求索 (DeepSeek Harness) `dsh` | 巨型**两行**字标：上行 `DEEPSEEK`，下行 `HARNESS` 并明显右缩进，约占 **85–90% 屏宽、超过一半屏高**。入场时字母**逐个由隐到现**，整块字标以**阶梯式每次下落 1 行**的方式丢帧，并在 HARNESS 下方留下残影；之后字标**不再消失**：7 秒一轮的**十字光束**循环——横向光束沿字标向下浮过，在中央与纵向光束汇成明亮十字，随后纵向光束从左向右扫过，横向光束淡出；没有烟花/粒子爆发。`--trail <f>` 调残影强度；**默认 ASCII**，`--unicode` 用实心块（[GIF](docs/dsh.gif)） |
| 氛围编程 (Vibe Coding) `vibe` | 单栏 **Claude Code 转录**：读取 `src/scenes/vibe.rs`（找不到时用编译期内置副本），用**手写 Rust 语法高亮**逐字打出源码，每轮循环 6 个短窗口；每轮重新开始时先清空转录，再打新内容；包含 prompt、assistant/tool 行、`+` diff 风格代码、旋转 spinner 与 token 数、输入框和 bypass 权限 footer；**默认 ASCII**（`>`、`*`、`L`、`>>`），`--unicode` 切换为 Claude Code 风格的 `❯ ◉ └ ✻ ▶▶`（[GIF](docs/vibe.gif)） |
| 星际隧道 (Warp Tunnel) `tunnel` | 16 圈按深度排布的环向观察者扩张，10 条径向辐条缓慢旋转，投影星点向外拖出短线。**纯 ASCII**：`+`、`*`、`#`、`@` 撑起整个透视。 |
| DNA 双螺旋 (DNA Helix) `helix` | 两条正弦主链绕垂直轴缠绕，A/T/C/G 碱基对连接两链，亮度由每条链的余弦深度决定，整体原地旋转。**纯 ASCII。** |
| 极光 (Aurora) `lights` | 三条波浪状极光带和短垂帘在星空、雪顶山脊、月亮与湖面倒影上方缓慢流动。**纯 ASCII。** |
| 火山爆发 (Volcano) `volcano` | 锯齿状火山锥、发光火山口、抛物线熔岩喷泉、两道熔岩流、漂移火山灰柱和周期性闪电。**默认 ASCII**；`--unicode` 用实心块画山体。 |
| 四维超立方 (Tesseract) `tesseract` | 16 个顶点、32 条棱在四维中旋转，画成带深度排序的线框：外立方青、内立方紫、8 条连接线金、顶点白。幽灵超立方、透视星轨、彩色尘埃、三圈轨道和沿线光脉冲填满空间。**默认 ASCII**；`--unicode` 用框线字符和实心顶点。 |
| 火箭发射 (Rocket Launch) `launch` | 24 秒发射大片：更大的两级火箭与双助推器、蓝调发射台、点火火球、多层尾焰与世界坐标烟雾、跟随镜头与震动、Max-Q 云层、MECO / 级间分离、二级爬升和地球弧线日出；`--rocket-scale`、`--shake`。**纯 ASCII。** |
| 等离子 (Plasma) `plasma` | 9 个漂移的 metaball 叠加成柔和标量场，再叠正弦细节和循环调色板。**默认 ASCII**；`--unicode` 用紧凑块字符 ramp。 |

</details>

光束有两种运动模式：

| 直线（默认） | 上下起伏（`--beam-motion jump`） |
| --- | --- |
| ![beams straight](docs/beams-ascii.png) | ![beams jump](docs/beams-jump.png) |

## ASCII / Unicode 双版本

**所有双字形特效默认 ASCII 启动。** 加 `--unicode`（或 `--charset unicode`）显式开启；`--ascii` 再强制回 ASCII。双字形特效包括 `matrix`、`mario`、`ecg`、`dandelion`、`fish`、`landscape`、`shanghai`、`tiananmen`、`traffic`、`tornado`、`storm`、`monalisa`、`ballet`、`demoneye`、`life`、`dsh`、`vibe`、`volcano`、`tesseract`、`plasma`。

`mario` / `ecg` 两版对比：

| ASCII（默认） | Unicode（`--unicode`） |
| --- | --- |
| ![mario ascii](docs/mario-ascii.png) | ![mario unicode](docs/mario.png) |
| ![ecg ascii](docs/ecg-ascii.png) | ![ecg unicode](docs/ecg.png) |

`shanghai` / `traffic` 两版对比：

| ASCII（默认） | Unicode（`--unicode`） |
| --- | --- |
| ![shanghai ascii](docs/shanghai-ascii.png) | ![shanghai unicode](docs/shanghai-unicode.png) |
| ![traffic ascii](docs/traffic-ascii.png) | ![traffic unicode](docs/traffic-unicode.png) |

`dsh` / `vibe` / `tesseract` 两版对比：

| ASCII（默认） | Unicode（`--unicode`） |
| --- | --- |
| ![dsh ascii](docs/dsh-ascii.png) | ![dsh unicode](docs/dsh-unicode.png) |
| ![vibe ascii](docs/vibe-ascii.png) | ![vibe unicode](docs/vibe-unicode.png) |
| ![tesseract ascii](docs/tesseract-ascii.png) | ![tesseract unicode](docs/tesseract-unicode.png) |

纵向车流：

![traffic vertical](docs/traffic-vertical.png)

全屏闪电：

![storm flash](docs/storm-flash-ascii.png)

```bash
trix mario              # 默认 ASCII 经典字形
trix mario --unicode    # Unicode 实心精灵
trix ecg                # 默认 ASCII 心电
trix ecg --unicode      # Unicode 心电
```

## 特性

- **35 个可切换特效**：命令行参数、运行时热切换、主题。
- **帮助本身也是界面**：风格化总览、单特效详情 + 真实预览、风格化目录、拼写建议。
- **ASCII 优先**：双字形特效默认 ASCII，`--unicode` 显式开启。
- **低资源**：单线程、无异步运行时、无事件循环框架；默认 30 FPS；唯一依赖是 `libc`。
- **快速启动**：`--version` 中位数约 **2.2 ms**（Apple Silicon，release）；发布二进制约 **1.1 MB**（1,104 KiB）。
- **低内存**：交互 RSS 约 **2.1–2.3 MB**（120×36，truecolor）。
- **Diff 渲染**：双缓冲画布只重绘变化单元格并合并 SGR 序列。
- **彩色动画**：24-bit truecolor；8 套主题；自动降级到 256 / 16 / 单色。
- **可录制 / 可压测**：内置 `--snapshot` 和 `--bench`。
- **干净退出**：raw mode、备用屏、光标和自动换行在正常退出、Ctrl-C、SIGTERM 和 panic 时恢复。

## 直接运行：自动循环播放

不带特效名直接运行 `trix`，会循环播放全部 35 个特效：

```bash
trix                                   # 默认顺序，每个 12 秒
trix --playlist random --switch 8      # 随机顺序，每个 8 秒
trix --playlist cycle --switch 20      # 顺序播放，每个 20 秒
trix --transition dissolve --switch 6  # 交叉溶解
```

切换默认是 0.6 秒淡入淡出；`--transition dissolve` 逐格交叉溶解；`--transition none` 恢复带白闪的瞬时切换。过渡时长上限是 `--switch` 的一半。按 `n` / `p` 手动切换并重置计时。

## 命令行

```text
trix <effect> [options]

  -e, --effect <name>     要运行的特效；省略则循环全部
      --playlist <mode>   cycle | random                    [默认: cycle]
      --switch <secs>     每个特效停留秒数                  [默认: 12]
      --transition <mode> fade | dissolve | none            [默认: fade]
      --transition-secs <s> 过渡时长                        [默认: 0.6]
  -f, --fps <1-240>       目标帧率                        [默认: 30]
  -s, --speed <0.05-8>    动画速度倍率                    [默认: 1.0]
  -t, --theme <name>      配色主题                        [特效默认]
      --charset <mode>    kana | ascii | unicode           [默认: ascii]
      --unicode           使用 Unicode 字形
      --ascii             强制 ASCII 字形
      --color <mode>      auto | truecolor | 256 | 16 | mono [默认: auto]
      --no-color          --color mono 的别名
      --density <f>       beams/fireworks/galaxy/traffic 密度 [默认: 1.0]
      --spin <f>          黑洞旋转速度                      [默认: 0.5]
      --twinkle <f>       银河闪烁频率                      [默认: 1.0]
      --meteors <n>       黑洞 / 银河流星数                 [默认: 3]
      --beam-speed <f>    光束水平速度                      [默认: 1.0]
      --beam-motion <m>   straight | jump                  [默认: straight]
      --trail <f>         dsh：丢帧残影强度                  [默认: 1.0]
      --seed-count <n>    蒲公英种子数                      [默认: 90]
      --seed-speed <f>    蒲公英种子漂移速度                [默认: 1.0]
      --street <dir>      horizontal | vertical            [默认: horizontal]
      --tornado-count <n> 龙卷风数量                        [默认: 3]
      --tornado-speed <f> 龙卷风移动速度                    [默认: 1.0]
      --life-cycle <secs> life：0～100 岁一轮秒数            [默认: 120]
      --eye-motion <m>    demoneye: dart | still           [默认: dart]
      --ghost             monalisa：纯 ASCII 鬼影版
      --gallery           monalisa：淡暖色展厅光晕
      --plain             dragon：纯星空背景
      --rocket-scale <f>  launch：火箭尺寸倍率            [默认: 1.0]
      --shake <f>         launch：镜头震动强度            [默认: 1.0]
      --jump              Tom & Jerry：开启跳跃
      --no-jump           Tom & Jerry：关闭跳跃
      --seed <u64>        固定随机种子
      --duration <secs>   运行 N 秒后自动退出
      --no-alt            不使用备用屏幕
      --no-hud            隐藏顶栏/底栏
  -l, --list              特效目录 + 主题
  -h, --help [effect|all] 帮助、单特效详情或完整参数
  -V, --version           版本
      --completions <sh>  输出 bash / zsh / fish 补全脚本

录制 / 压测
      --snapshot <path>   模拟 --frames 步后输出一帧 ANSI（'-' = stdout）
      --frames <n>        模拟步数
      --size <WxH>        离屏画布尺寸，例如 120x36
      --bench             离屏渲染压测
```

完整参数、别名和默认值见 `trix --help all`。

### HUD

- **顶栏**：左侧特效名 + key，右侧实时遥测 / 速度 / 主题 / 字符集。
- **底栏**：常驻按键提示（`q quit`、`space pause`、`n/p next`、`t theme`、`h help`、`i info`）。
- **播放列表行**：进度和下一次切换倒计时。
- **`i` 面板**：运行状态 + 当前特效真正在用的 CLI 参数。

`rain`、`fire`、`monalisa` 是刻意无文字的 Ambient 场景：没有顶栏和场景文字，但保留底栏和播放列表行。`--no-hud` 隐藏顶栏/底栏用于录制，`h` / `i` 面板和暂停框仍然可用；`--snapshot` 不绘制 HUD。

完整 HUD 约定见 [docs/hud-and-keys.md](docs/hud-and-keys.md)。

### 运行时快捷键

| 按键 | 动作 |
| --- | --- |
| `q` / `Ctrl-C` | 退出 |
| `Esc` | 关闭帮助/信息面板；没有面板时退出 |
| `Space` | 暂停 / 继续 |
| `n` / `p`、方向键 | 下一个 / 上一个特效 |
| `1`–`9`、`0`、`[`、`]` | 前 12 个特效 |
| `m` / `e` / `d` | 马里奥 / 心电图 / 蒲公英 |
| `f` / `l` / `b` / `j` | 水族馆 / 山水 / 东方明珠 / 天安门 |
| `c` / `z` / `k` | 车水马龙 / 龙卷风 / 雷暴 |
| `o` / `v` / `g` / `y` | 蒙娜丽莎 / 冰上芭蕾 / 恶魔之眼 / 生命的轨迹 |
| `u` / `w` | 深度求索 / 氛围编程 |
| `t` | 切换主题 |
| `+` / `-` | 加速 / 减速（0.1×–8×） |
| `r` | 用新种子重开当前特效 |
| `i` | 信息 / 参数面板 |
| `h` / `?` | 帮助面板 |

快捷键不区分大小写；支持 CSI/SS3 方向键，忽略方向键修饰；Alt/Option 组合和鼠标/粘贴上报不会误触发。

七个新场景（`tunnel`、`helix`、`lights`、`volcano`、`tesseract`、`launch`、`plasma`）**没有单键快捷键**；用 `trix <effect>` 或 `n` / `p` 循环进入。

## 主题

| 主题 | 风格 | 默认用于 |
| --- | --- | --- |
| `neon` | 霓虹青 / 紫 | rain、beams、tomjerry、shanghai、traffic、tunnel、tesseract |
| `matrix` | 矩阵绿 | matrix、ecg |
| `ember` | 真实火焰（红→橙→黄→白） | fire、demoneye、volcano、launch |
| `aurora` | 翡翠龙 / 极光金 | dragon、fish、landscape、ballet、life、helix、lights |
| `ice` | 全息冰蓝 | map、solar、galaxy、tornado、storm、dsh |
| `plasma` | 品红 / 高能 | blackhole、fireworks、plasma |
| `gold` | 帝王金 | buddha、mario、dandelion、tiananmen、monalisa、vibe |
| `chrome` | 银灰单色 | 手动选择 |

`fire` 的火焰颜色来自主题调色板的 5 个锚点；`--theme ice` 会得到“冷焰”，`--theme ember` 是默认燃烧火焰。

## 性能

环境：Apple Silicon（ARM64）、macOS、Rust 1.96.1、`--release`（LTO + `codegen-units=1` + `panic=abort` + `strip`）。画布 160×48，240 帧离屏压测：

| 特效 | 平均渲染 | p95 | Diff 输出 |
| --- | ---: | ---: | ---: |
| rain | 0.080 ms | 0.101 ms | 14.3 KiB/帧 |
| fire | 0.454 ms | 0.548 ms | 68.3 KiB/帧 |
| dragon | 0.112 ms | 0.134 ms | 8.9 KiB/帧 |
| map | 0.161 ms | 0.189 ms | 43.6 KiB/帧 |
| matrix | 0.071 ms | 0.087 ms | 12.0 KiB/帧 |
| blackhole | 0.133 ms | 0.172 ms | 28.0 KiB/帧 |
| beams | 0.085 ms | 0.113 ms | 24.9 KiB/帧 |
| fireworks | 0.040 ms | 0.064 ms | 8.6 KiB/帧 |
| solar | 0.039 ms | 0.045 ms | 7.6 KiB/帧 |
| galaxy | 0.129 ms | 0.163 ms | 23.4 KiB/帧 |
| buddha | 0.061 ms | 0.072 ms | 11.4 KiB/帧 |
| tomjerry | 0.019 ms | 0.022 ms | 2.4 KiB/帧 |
| mario | 0.013 ms | 0.016 ms | 0.7 KiB/帧 |
| ecg | 0.038 ms | 0.046 ms | 7.3 KiB/帧 |
| dandelion | 0.024 ms | 0.028 ms | 3.6 KiB/帧 |
| fish | 0.023 ms | 0.045 ms | 2.1 KiB/帧 |
| landscape | 0.038 ms | 0.043 ms | 1.7 KiB/帧 |
| shanghai | 0.032 ms | 0.036 ms | 3.3 KiB/帧 |
| tiananmen | 0.095 ms | 0.105 ms | 7.0 KiB/帧 |
| traffic | 0.029 ms | 0.036 ms | 1.7 KiB/帧 |
| tornado | 0.043 ms | 0.051 ms | 7.8 KiB/帧 |
| storm | 0.109 ms | 0.122 ms | 29.9 KiB/帧 |
| monalisa | 0.021 ms | 0.032 ms | 0.0 KiB/帧（静态） |
| ballet | 0.120 ms | 0.151 ms | 14.5 KiB/帧 |
| demoneye | 0.273 ms | 0.398 ms | 15.4 KiB/帧 |
| life | 0.158 ms | 0.209 ms | 5.6 KiB/帧 |

真实 PTY，truecolor，120×36，30 FPS，5 秒（CPU = 子进程 `utime+stime / 5s`，包含 PTY 写入，不含终端自身绘制）。下表是那一轮测到的 22 个特效；`monalisa`、`ballet`、`demoneye`、`life`、`dsh`、`vibe`、`tunnel`、`helix`、`lights`、`volcano`、`tesseract`、`launch`、`plasma` 是后来加入的，未包含在这轮数据里：

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
| dandelion | 0.28% | 2.08 MiB | 0.5 MiB |
| fish | 0.21% | 2.14 MiB | 0.4 MiB |
| landscape | 0.25% | 2.08 MiB | 0.2 MiB |
| shanghai | 0.32% | 2.09 MiB | 0.3 MiB |
| tiananmen | 0.35% | 2.08 MiB | 0.8 MiB |
| traffic | 0.26% | 2.14 MiB | 0.3 MiB |
| tornado | 0.27% | 2.08 MiB | 0.7 MiB |
| storm | 0.63% | 2.19 MiB | 3.6 MiB |

- 30 FPS 下约 33.3 ms/帧预算；最重的 `fire` 离屏渲染约 **1.38% 单核**（0.454 ms/帧），实际交互约 **1.4%**。
- 启动：`--version` 中位数约 **2.2 ms**；发布二进制约 **1.1 MB**（1,104 KiB）。
- 内存：真实 PTY 下峰值 RSS **2.08–2.27 MiB**。
- 交互模式遵守 `NO_COLOR`；离屏 `--snapshot` / `--bench` 默认 truecolor，可用 `--color` 覆盖。

```bash
scripts/bench.sh
./target/release/trix fire --bench --size 160x48 --frames 300
```

## 兼容性与常见问题

- 支持 macOS、Linux、WSL（Unix termios）；原生 Windows 未实现。
- 推荐 24-bit truecolor；`--color auto` 会自动降级到 256 / 16 / 单色。
- 双字形特效默认 ASCII；加 `--unicode` 或 `--charset unicode` 开启 Unicode。`--charset kana` 可以在 `matrix` 中混入半角片假名。
- Linux 二进制是 `x86_64` / `aarch64` 的静态 musl 构建，任意发行版（包括 Alpine）可用。
- 非 TTY 运行会提示 `--snapshot`；`trix --snapshot -` 输出 ANSI 到 stdout。
- 画布是黑色，建议深色终端。
- **为什么命令叫 `trix`、包叫 `trix-effects`？** 短命令 + 唯一包名，类似 `ripgrep` / `rg`。安装这个包（例如 `cargo install --git …`）装出来的命令是 `trix`。
- **为什么截图是 ASCII？** ASCII 是默认，兼容所有终端/字体；Unicode 预览只放在 ASCII / Unicode 一节。
- **怎么录制 GIF？** 用 `--snapshot` + `tools/ansi2png.py`，或 `tools/render_gif.py`；见[录制与文档工具](#录制与文档工具)。
- **Shell 补全？** `trix --completions bash`（也支持 `zsh`、`fish`），把输出放到对应 shell 的补全目录。

## 实现

- **终端驱动**（`src/term.rs`）：libc 设置 raw mode、备用屏幕、隐藏光标、关闭自动换行；非阻塞读键、`TIOCGWINSZ` 尺寸检测、SIGINT/SIGTERM/SIGHUP 标志、panic hook。
- **Canvas**（`src/canvas.rs`）：`Cell { char, fg, bg }` 双缓冲；逐行变化区间；前景/背景变化都发 SGR，背景回落时发 `40` 回到程序启动时的黑底（避免切场景后残留上一幕的底色）；支持 truecolor/256/16/mono（mono 只画字形）。`monalisa` 的色块人物像使用背景色，`mario` 不使用背景色，纯靠字形与前景色。
- **零框架**：唯一依赖 `libc`；没有 tokio/crossterm/线程任务。
- **帮助**（`src/help.rs`、`src/meta.rs`）：特效元数据驱动 `--help`、`--help <effect>`（真实场景预览）、`--help all`、风格化 `--list`、`i` 面板和交互式 `h` 面板；布局按终端宽度自适应，非 TTY 时退化为 ASCII / 单色。
- **代码雨**：逐列头/尾、随机速度与深度、二进制雾、横向数据包、CRT 扫描。
- **火焰**：Doom-fire 元胞传播；底部合并成一整片宽火床，中间火苗最高，两侧火舌向左右倾斜散开。
- **巨龙**：飞行轨迹按弧长重采样，长蛇身 splat 到强度/深度缓冲；大型中国龙头、鹿角、龙须、鬃毛、四爪、背鳍、尾鳍、龙鳞；整屏 Lissajous 飞行。
- **世界地图**：Natural Earth 陆地掩膜、覆盖率采样、城市节点、大圆弧、扫描波束。
- **黑客帝国**：NASA 后台登录大屏。默认 ASCII；`--unicode` 切换蓝色 NASA 横幅（白色大字、红色 swoosh、白色星点、蓝白流光）、居中下移的 Unicode 边框登录框；OPERATOR/PASSWORD/2FA、AUTH/DECRYPT 进度、日志、雷达、ACCESS GRANTED。
- **黑洞**：事件视界、光子环、吸积盘、透镜弧、喷流、流星、内落；外围星尘沿 "/" 倾斜的饱满椭圆环绕，整体铺满屏幕；最外圈用 `.` / `:` / `+` 随机低密度散点，内层与外层之间补一层中段星尘；整体同向差速旋转（内快外慢，外圈与吸积带同步提速），看起来是一个完整转动的椭圆盘；整体公转速度 ×1.7；`--spin` 调整体旋转。
- **光束**：单向从左向右；默认直线，`--beam-motion jump` 上下起伏；`--density`/`--beam-speed` 调密度与速度。
- **烟花**：火箭 + 内外双环粒子 + 延迟爆裂；粒子拖尾、重力、城市剪影、星空。
- **太阳系**：太阳在左上角；八大行星在下方轨道线；阳光向右下扇形分层传播，且在行星线上方截止。
- **银河**：预计算星点沿斜向银河带 + 中央核球；动态星团弧、暗尘、流星、星座线；`--density`/`--twinkle`/`--meteors` 可调。
- **佛光**：回退上一版；原三层曼陀罗粒子环改为动态向外扩散，头光/莲座/粒子保留。
- **Tom & Jerry**：太空站走廊；运行帧动画；默认慢速、不跳跃；`--speed`/`+`/`-` 调速，`--jump` 开启跳跃；HUD 显示倍率。
- **超级马里奥**：自动奔跑/跳跃，问号砖块顶出蘑菇、金币与分数；滚动天空/山丘/地面/板栗仔；默认 ASCII 经典字符，`--unicode` 切换实心精灵。
- **心率跳动**：ECG 网格、P-QRS-T 波形、扫描线、HR/SpO2/SYS-DIA；默认 ASCII 波形，`--unicode` 切换 Unicode 框线/实心波形。
- **蒲公英**：3 丛×2 茎/6 花头；左右根向中间弯曲；花头尺寸 ×1.5。
- **东方明珠**：外滩夜景 + 东方明珠 + 黄浦江倒影；江面两条随机船只——船型（帆船/货轮/渡轮）、尺寸、船体配色、航道深度与左右漂移方向都随机，远景小船贴水、近景大船压屏，带甲板/舷窗/船楼窗光/烟囱冒烟/桅杆航灯/船首灯笼/尾迹与水面倒影；每 3–5 秒夜空随机 1–2 颗流星（约 18% 的时段空过），白热头部 + 冰蓝或暖金渐隐尾迹，飞入天际线或淡出后回收。
- **深度求索**：两行字标（`DEEPSEEK` / 右缩进的 `HARNESS`）由 6×9 / 8×11 显示字模横向放大 2 倍排布，宽字形占据大部分屏幕。入场是逐字出现（像素从上到下刷出）与每次 1 行的量化丢帧，并在 HARNESS 下方留下哈希丢弃残影。之后字标**永久保留**；7 秒一轮的光束状态机让横向光束向下浮过，在中央与纵向光束汇成明亮十字，再让纵向光束从左向右扫过并淡出横向光束；没有烟花/粒子爆发。`--trail` 缩放残影。
- **火箭发射**：程序化缩放的两级火箭（整流罩、芯级、双助推器、发动机喷管）由 7/9/11 列的字符网格生成；镜头跟随升空，并在点火与级间分离时震动。尾焰是带白热核心与马赫环的分层 ASCII 火焰，配合世界坐标烟雾、蒸汽与火星。周围还有蓝调天空渐变、视差星层、云层、MECO 闪光、翻滚的芯级与助推器、地球弧线大气和轨道日出。`--rocket-scale` 与 `--shake` 调整构图和震动。
- **氛围编程**：读取 `src/scenes/vibe.rs`（找不到时用编译期内置副本），归一化为 ASCII、跳过测试模块，每轮生成 6 个 10 行窗口。手写 Rust tokenizer 为注释、字符串、数字、关键字、类型、宏和函数名上色；转录是一个只显示到当前打字位置的队列（下方保持空白），下一轮开始前会清空，带 Claude Code 风格的 spinner、token 数和输入框。
- **地图数据离线生成**：`tools/gen_land.py` 下载 Natural Earth 并栅格化为 `src/data/land_720x360.bin`，运行时不访问网络。

## 录制与文档工具

```bash
trix dragon --snapshot dragon.ans --size 120x36 --frames 145 --seed 7
python3 tools/ansi2png.py dragon.ans dragon.png --size 120x36 --label "trix dragon"
python3 tools/render_gif.py dragon docs/dragon.gif --start 100 --frames 24 --step 3
```

文档素材：`docs/help.png`、`docs/help-rain.png` 与 `docs/help-full.png` 分别由 `TRIX_HELP_TTY=1 trix --help`、`trix --help rain`、`trix --help all` 加上 `--color true` 渲染，再经 `tools/ansi2png.py` 转换（`help-rain` 使用 `--size 80x40`）。务必显式指定 `--color true`：在 `NO_COLOR` 环境下样式化帮助会正确降级为单色，直接截图会得到纯白画面。`docs/social-preview.png` 是 1280×640 的 GitHub 社交预览图。

## 测试

```bash
cargo test
cargo clippy --all-targets
cargo fmt --check
```

当前 **224** 个单元/集成测试。CI 矩阵见 `.github/workflows/ci.yml`。

## 版本与回退

- **当前未发布版本：0.11.0**（`trix --version`），尚未打 tag 或创建 GitHub Release。
- **0.11.0**：新增七个生成式场景：`tunnel`、`helix`、`lights`、`volcano`、`tesseract`、`launch`、`plasma`。
- **0.10.0**：新增 `dsh`（深度求索 / DeepSeek Harness）与 `vibe`（氛围编程 / Vibe Coding），以及 `--trail` 参数。
- **0.9.1**：包名/仓库改为 `trix-effects`（命令仍是 `trix`），加固 `install.sh`，新增缓存安全的 ASCII 预览和贡献/发布文档。
- **0.9.0** 是短暂使用的 `trix-tui` 名字；新增风格化帮助系统，`fairy` 入口移除（源码保留），`life` 进入 26 个特效。
- 上一版：tag **`v0.9.0`**（`trix-tui`）；**`v0.8.0`**（蒙娜丽莎 / 冰上芭蕾 / 恶魔之眼；26 个特效，含 `fairy`）；**`v0.7.0`**（水族馆/山水/东方明珠/天安门/车水马龙/仙女/龙卷风/雷暴、播放列表）；**`v0.6.0`**（马里奥/心电/蒲公英、蓝绿代码雨）；**`v0.5.1`**（自动发布流水线）。
- 回退：
  ```bash
  git checkout v0.9.0     # 只查看上一版（detached HEAD）
  git switch master
  git reset --hard v0.9.0 # 完全回退到上一版
  ```
- 额外备份（不进 git）：`backups/trix-v0.1.0-*` 到 `trix-v0.7.0-*`，含源码 tar 与 aarch64 macOS 二进制。
- 变更日志：[English](CHANGELOG.md) · [中文](CHANGELOG.zh-CN.md)。

### 发布

- 推送 `v*` tag 会自动构建二进制并创建 GitHub Release。
- **macOS**：`aarch64-apple-darwin` + `x86_64-apple-darwin`，用 `lipo` 合成 universal 二进制。
- **Linux**：`x86_64` / `aarch64` 静态 musl 构建（ARM64 用 `cross` 交叉编译，并在 QEMU 下做启动冒烟）。
- 资产：`trix-effects-<version>-macos-universal.tar.gz`、`trix-effects-<version>-linux-x86_64.tar.gz`、`trix-effects-<version>-linux-aarch64.tar.gz`，以及 `SHA256SUMS`。
- 在 Actions 页面手动 `Run workflow` 只构建产物，不创建 Release。

## 参与贡献

欢迎提 Issue 和 PR。开发环境、提交前检查和截图规范见 [CONTRIBUTING.md](CONTRIBUTING.md)，社区约定见 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)；发布文案和渠道草稿在 [docs/LAUNCH.md](docs/LAUNCH.md)。

## 数据来源、参考与许可

- 陆地轮廓：[Natural Earth](https://www.naturalearthdata.com/) 1:110m land，**public domain**，经 [natural-earth-vector](https://github.com/nvkelso/natural-earth-vector) GeoJSON 转换。
- Doom-fire 灵感：[pserey/doom-fire-algorithm](https://github.com/pserey/doom-fire-algorithm)、[horizonwiki/fire](https://github.com/horizonwiki/fire)；本项目的 Rust 实现、多燃料堆、配色与渲染均为独立编写。
- 佛祖 ASCII 艺术改编自 [ascii.co.uk/art/buddha](https://ascii.co.uk/art/buddha)（原作者署名 `hjw`），用于 `buddha` 场景；本项目补充了颜色、光环、莲座、粒子与动画。
- 本项目代码：[MIT](LICENSE)。
