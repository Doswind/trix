# trix

[English](README.md) | **简体中文**

> 前身：texiao（原项目名）；从 v0.5.0 起正式更名为 **trix**。

> 终端里的科幻 ASCII / Unicode 特效引擎 — 代码雨、火焰、巨龙、世界地图、NASA 登录、黑洞、光束、烟花、太阳系、银河、佛光、Tom & Jerry、超级马里奥、心率跳动、蒲公英、水族馆、山水、东方明珠、天安门、车水马龙、仙女散花、龙卷风、电闪雷鸣、蒙娜丽莎、冰上芭蕾、恶魔之眼、生命的轨迹。
> Rust 单线程、无异步运行时、diff 渲染，启动约 **2 ms**，常驻内存约 **2.1–2.3 MB**，30 FPS 下 CPU 约 **0.2–1.4%** 单核。当前 **v0.8.0**；开发版新增「生命的轨迹」，共 **27** 个特效，双字形特效默认 ASCII。

![release](https://github.com/Doswind/trix/actions/workflows/release.yml/badge.svg)

![trix overview](docs/overview.png)

> 动图/预览：[代码雨](docs/rain.gif) · [火焰](docs/fire.gif) · [巨龙](docs/dragon.gif) · [NASA 登录](docs/matrix.gif) · [黑洞](docs/blackhole.gif) · [光束](docs/beams.gif) · [烟花](docs/fireworks.gif) · [太阳系](docs/solar.gif) · [银河](docs/galaxy.gif) · [佛光](docs/buddha.gif) · [Tom & Jerry](docs/tomjerry.gif) · [马里奥](docs/mario.gif) · [心电](docs/ecg.gif) · [蒲公英](docs/dandelion.gif) · [水族馆](docs/fish.gif) · [山水](docs/landscape.gif) · [东方明珠](docs/shanghai.gif) · [天安门](docs/tiananmen.gif) · [车水马龙](docs/traffic.gif) · [仙女](docs/fairy.gif) · [龙卷风](docs/tornado.gif) · [电闪雷鸣](docs/storm.gif) · [蒙娜丽莎](docs/monalisa.png) · [鬼影版](docs/monalisa-ghost.gif) · [冰上芭蕾](docs/ballet.gif) · [恶魔之眼](docs/demoneye.gif) · [生命的轨迹](docs/life.gif)

## 特效

下表给出所有特效的中英名称对照。

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
| `fairy` | 仙女散花 | Fairy Scattering Petals |
| `tornado` | 龙卷风 | Tornado |
| `storm` | 电闪雷鸣 | Thunderstorm |
| `monalisa` | 蒙娜丽莎 | Mona Lisa |
| `ballet` | 冰上芭蕾 | Ice Ballet |
| `demoneye` | 恶魔之眼 | Demon Eye |
| `life` | 生命的轨迹 | Threads of Life |

| 代码雨 (Code Rain) `rain` | 燃烧火焰 (Blazing Fire) `fire` | 巨龙盘旋 (Coiling Dragon) `dragon` |
| --- | --- | --- |
| ![rain](docs/rain.png) | ![fire](docs/fire.png) | ![dragon](docs/dragon.png) |

| 世界地图 (World Map) `map` | 黑客帝国 (The Matrix) `matrix` | 黑洞 (Black Hole) `blackhole` |
| --- | --- | --- |
| ![map](docs/map.png) | ![matrix](docs/matrix.png) | ![blackhole](docs/blackhole.png) |

| 光束 (Light Beams) `beams` | 烟花 (Fireworks) `fireworks` | 太阳系 (Solar System) `solar` |
| --- | --- | --- |
| ![beams](docs/beams.png) | ![fireworks](docs/fireworks.png) | ![solar](docs/solar.png) |

| 满天星河 (Starry Galaxy) `galaxy` | 佛光普照 (Buddha's Halo) `buddha` | 猫和老鼠 (Tom & Jerry) `tomjerry` |
| --- | --- | --- |
| ![galaxy](docs/galaxy.png) | ![buddha](docs/buddha.png) | ![tomjerry](docs/tomjerry.png) |

| 超级马里奥 (Super Mario) `mario` | 心率跳动 (Heartbeat Monitor) `ecg` | 蒲公英 (Dandelion) `dandelion` |
| --- | --- | --- |
| ![mario](docs/mario.png) | ![ecg](docs/ecg.png) | ![dandelion](docs/dandelion.png) |

| 水族馆 (Aquarium) `fish` | 五光十色 (Colorful Landscape) `landscape` | 东方明珠 (Oriental Pearl) `shanghai` |
| --- | --- | --- |
| ![fish](docs/fish.png) | ![landscape](docs/landscape.png) | ![shanghai](docs/shanghai.png) |

| 光芒万丈 (Tiananmen Radiance) `tiananmen` | 车水马龙 (Traffic Flow) `traffic` | 仙女散花 (Fairy Scattering Petals) `fairy` |
| --- | --- | --- |
| ![tiananmen](docs/tiananmen.png) | ![traffic](docs/traffic.png) | ![fairy](docs/fairy.png) |

| 龙卷风 (Tornado) `tornado` | 电闪雷鸣 (Thunderstorm) `storm` | 蒙娜丽莎 (Mona Lisa) `monalisa` |
| --- | --- | --- |
| ![tornado](docs/tornado.png) | ![storm](docs/storm.png) | ![monalisa](docs/monalisa.png) |

| 冰上芭蕾 (Ice Ballet) `ballet` | 恶魔之眼 (Demon Eye) `demoneye` | 生命的轨迹 (Threads of Life) `life` |
| --- | --- | --- |
| ![ballet](docs/ballet.png) | ![demoneye](docs/demoneye.png) | ![life](docs/life.png) |

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
| 超级马里奥 (Super Mario) `mario` | **纯字符版 1-1 关卡**：云、山、灌木，砖面带灰缝的地面，**问号块/砖块/已用块**砖行，绿管道、旋转金币、板栗仔与蘑菇；自动跑跳、顶砖出币/蘑菇、踩扁敌人、`+200/+100/+1000` 飘字。**完全不使用终端背景色**：默认 ASCII 彩色字符（`# o = *`），`--unicode` 用 `█ ▀ ▄` 实心块；山/云/管道/砖块都按物体配色 |
| 心率跳动 (Heartbeat Monitor) `ecg` | 绿色心电监护仪；**约 56 BPM，随机心搏间隔**，上下幅度、QRS 宽度、平直段长度明显随机；**默认 ASCII**，`--unicode` 切换 |
| 蒲公英 (Dandelion) `dandelion` | 3 丛×2 茎、6 个花头；左右两丛根/下段向中间弯曲；**花头尺寸 ×1.5**，其它不变 |
| 水族馆 (Aquarium) `fish` | 全屏水族箱：**5 条手绘 sprite 大鱼**（small 20×5 / medium 30×7 / large 38×8，分叉尾 + 亮眼 + 嘴，只摆尾不转头、嘴和头固定不动），外加 8～30 条 `<==o>` / `<=-o>` 小鱼（约 35% 双层、15% 三层加粗，只留一个眼睛）；气泡用 `o` / `O` / `8` 三种字符、**每条随机深浅色**，各自沿固定的一列**缓慢直线上升**，整个上升过程左右摆动不超过 3 次；数量收敛到 10～36 颗；水草随机高矮/间距并成小片；默认 ASCII，`--unicode` 切换 `█ ▓ ▒` |
| 五光十色 (Colorful Landscape) `landscape` | 层叠远山、平湖、小船、白鹅、两侧丛林；水面/雾气/花瓣动态 |
| 东方明珠 (Oriental Pearl) `shanghai` | 上海外滩夜景、东方明珠、高楼霓虹、黄浦江倒影；**江面随机两条大小/形状/位置都不同的船**（帆船 / 货轮 / 渡轮三种船型，含甲板、舷窗、船楼窗光、烟囱冒烟、桅杆航灯、尾迹与水面倒影）；**每 3–5 秒夜空中飘出 1–2 颗流星**（白热头部 + 渐隐尾迹）；**默认 ASCII**，`--unicode` 可切换 |
| 光芒万丈 (Tiananmen Radiance) `tiananmen` | 放大到约 92% 宽 / 62% 高的天安门：双层金瓦飞檐、红柱城楼与国徽、红墙、白石门套五门洞、白框标语与中央画像、红旗与两侧小楼；金色光束先从城楼后方向外向上生成，再按光束由内向外依次循环点亮 |
| 车水马龙 (Traffic Flow) `traffic` | 城市街道两侧高楼、霓虹、车流；横向 3 行车（车顶/车身/车轮，8–16 格长）分 4–5 条车道、车道间有虚线，方向按车道交替；纵向 6 列俯视车（5–9 行，带前后灯与侧轮）；`--street horizontal\|vertical`；`--density` 调车流密度；**默认 ASCII**，`--unicode` 可切换 |
| 仙女散花 (Fairy Scattering Petals) `fairy` | 30×34 手工像素仕女（低发髻、侧脸眉眼、举篮、收臂、交领蓝襟、白裙裙裾）；缓慢椭圆环绕 + 每 24 秒一次 6 秒的转身（`scaleX`，不翻跟斗）；花瓣从篮口跟随抛出后四散飘落；`--petal-count` / `--petal-speed` |
| 龙卷风 (Tornado) `tornado` | 1–8 股龙卷风从左向右；扭曲、底部收尖、顶部更宽；整体比原来**更矮一档**（0.28～0.58 屏高）且**更淡更疏**（亮度 0.42～0.85、每行 20 颗）；底部有 30 颗碎屑**向外飘散** |
| 电闪雷鸣 (Thunderstorm) `storm` | 乌云、斜雨、城市剪影、闪电；屏幕上方叠一层**灰白、不规则的高云**（较密、底部淡出、**缓慢向右飘移**（约 3 列/秒））与原有云带形成层次；落雷时全屏白色亮闪（[效果图](docs/storm-flash.png)） |
| 蒙娜丽莎 (Mona Lisa) `monalisa` | 去背景人物像：从参考像素画烘焙的人物（头发、脸、颈肩、交叠双手），ANSI 色块承载肤色/明暗 + ASCII 字符勾细节；没有山水/画框/浮尘；人物横向加宽约 12%，更接近原作体态；**画面静态，不眨眼**；`--unicode` 用半块提高垂直分辨率；`--ghost` 进入原纯 ASCII“鬼影”版（无背景色块、保留眨眼，[静态预览](docs/monalisa-ghost.png) / [动图](docs/monalisa-ghost.gif)）；可选 `--gallery` 在人物后方加淡暖色展厅光晕与地面微光（[预览](docs/monalisa-gallery.png)） |
| 冰上芭蕾 (Ice Ballet) `ballet` | 冰场双人舞：39 秒编舞循环 13 个舞段（滑行、压步、arabesque、promenade、**头顶托举**、联合旋转、death spiral、sissonne 跳跃、Ina Bauer、收势），关节用两骨 IK 求解、牵手自动对齐，四肢带柔边并延迟跟随躯干；整场滑行覆盖大部分画面，围板后有三排会摇晃/举手/打闪光灯的观众，冰面留下冰刀刻痕与倒影；`--speed` 调速 |
| 恶魔之眼 (Demon Eye) `demoneye` | 整屏一只巨大血眼：眨眼每 8～15 秒一次、单眨或双眨（约 7:3），每两次眨眼之间只甩出 1～2 次（两次跳动相隔 ≥5 秒），约 0.1 秒到位、0.45～0.85 秒内必回正中直视观众；竖瞳缩放、眼睑压暗、血丝与眨眼冲击波；`--eye-motion dart\|still`（默认 dart），`--eye-still` 让眼球钉在正中 |
| 生命的轨迹 (Threads of Life) `life` | 0～100 岁时间轴与左右生命线；人类统一 `@` 符号 + 同排彩色大写简写 + **家属本人年龄**（非亲属朋友不带年龄）；亮光束随年龄增长（年长后可覆盖大部分生命），由尾部的暗点/短划线渐亮到 `=`，头部为 `****@` 彗尾；生命最初约 22%～42%（最多约 22 年）保留为稀疏暗淡的点；每条轨迹结束后其行位可被后来者复用（学龄 `PAL/UNI` 的 +1 行 → GF/WIFE，`DOG` 的 -1 行 → KID2，猫/父母的行位也会被后来的朋友接回）；`DOG` 终身在 -1 紧挨我；`PAL/UNI/WORK/FRIEND` 同时出现时**共用同一行**：学龄期在 +1 另一侧，有未成年 KID 时整组退到 KID/CAT 外侧最近的空行（通常为 +4；只让出孩子/宠物实际占用的行，不再固定 -5/-7），所有 KID 成年后回到内侧；`ME 0 / WIFE +1`；`KID1 +2`、`KID2 -1`（复用 DOG 让出的紧邻行）、`KID3 -2`；成年前 `KID3 -2 / KID2 -1 / ME 0 / WIFE +1 / KID1 +2` **连续无空行**，`GRAND1/GRAND2` 始终紧贴父辈一行（父辈成年漂移后随之移动）；`CAT +3` 与第一个孩子同时出生，紧贴 KID1 下方；`DAD/MOM +3/+4` 在孩子出生前两年开始让位到 `+5/+6`，`BRO +5→+7` 紧随让位；GPA/GMA/DAD/MOM 在我 0 岁时随机得到合适起始年龄（58～70 / 56～68 / 25～33 / 23～31），BRO 0～4，ME 0；KID/GRAND/CAT/DOG 从各自出生起算，GF/WIFE 与我同龄上下 2 岁；所有行固定为 1 个终端行（不随终端高度缩放，高终端也不会因取整出现空行）；`ME` 结束时触发约 6 秒的**超新星终章**（回闪与星屑放慢、彗星保持原速；白光核心、一生回闪、冲击波、星屑、加亮彗星与顶部永久光晕；回闪扫描光柱到达 ME 的停止线后以三列宽永久保留，不再消失）；死亡点与顶部星标保留 `@` + 名称 + 最终年龄，并以每秒 1 次同步显现/消失；所有死亡角色升到顶部按死亡顺序隔行排列，终点标签在流星抵达后 0.5 秒淡入；人类 `*`、猫狗 `+` 闪烁；GF 用 2 年靠近、再停留 1～2 年变 WIFE；WIFE 75～90 岁、晚于我化星；AGE 无前导零；`--life-cycle` 控制一轮秒数 |

> 截图多为 120×36、truecolor、离屏渲染；`monalisa` 为 120×60 竖构图。部分截图/动图展示 Unicode 字形，实际运行默认 ASCII（加 `--unicode` 切换）。

光束两种运动模式：

| 直线（默认） | 上下跳跃 |
| --- | --- |
| ![beams straight](docs/beams.png) | ![beams jump](docs/beams-jump.png) |

### ASCII / Unicode 双版本

**所有同时支持两种字形的特效默认以 ASCII 启动**：`matrix`、`mario`、`ecg`、`dandelion`、`fish`、`landscape`、`tiananmen`、`fairy`、`tornado`、`storm`、`monalisa`、`ballet`、`demoneye`、`life`、`shanghai`、`traffic`。需要 Unicode 时显式加 `--unicode`（或 `--charset unicode`）；`--ascii` 可再次强制 ASCII。`monalisa` 的默认色块人物像是静态的；`monalisa --ghost` 是另一套纯 ASCII 字符画（保留眨眼），不输出 Unicode 或背景色块。

`mario` / `ecg` 两种字形：

| ASCII（默认） | Unicode（`--unicode`） |
| --- | --- |
| ![mario ascii](docs/mario-ascii.png) | ![mario unicode](docs/mario.png) |
| ![ecg ascii](docs/ecg-ascii.png) | ![ecg unicode](docs/ecg.png) |

`shanghai` / `traffic` 两种字形：

| ASCII（默认） | Unicode（`--unicode`） |
| --- | --- |
| ![shanghai ascii](docs/shanghai.png) | ![shanghai unicode](docs/shanghai-unicode.png) |
| ![traffic ascii](docs/traffic.png) | ![traffic unicode](docs/traffic-unicode.png) |

纵向街道：

![traffic vertical](docs/traffic-vertical.png)

雷暴全屏亮闪：

![storm flash](docs/storm-flash.png)

```bash
trix mario              # 默认 ASCII 经典字符
trix mario --unicode    # Unicode 实心精灵
trix ecg                # 默认 ASCII 心电监护
trix ecg --unicode      # Unicode 心电监护
```

## 特性

- **27 个可切换特效**：支持命令行参数、运行时热切换与主题。
- **平滑换场**：每次切换特效都可以选择渐隐渐显或交叉溶解（`--transition fade|dissolve|none`，默认 0.6 秒）；旧场景在淡出期间继续动画。
- **ASCII 优先**：同时支持 ASCII / Unicode 的特效默认以 ASCII 启动；需要 Unicode 时显式加 `--unicode`。
- **低资源**：单线程、无 async、无事件循环框架；默认 30 FPS；仅依赖 `libc`。
- **快速启动**：`--version` 中位数约 **2.0 ms**；发布版二进制约 **733 KB**。
- **低内存**：交互模式 RSS 约 **2.08–2.27 MB**（120×36，truecolor）。
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
trix dragon --plain                # 原纯星空背景
trix map --theme ice               # 全息世界地图
trix matrix                        # NASA 登录大屏（默认 ASCII；--unicode 大屏）
trix blackhole                     # 黑洞吞噬恒星
trix beams                         # 横向随机亮色光束
trix fireworks                     # 双层烟花绽放
trix solar                         # 太阳系八大行星一线
trix galaxy                        # 满天星河 / 银河
trix buddha --theme gold           # 佛光普照
trix tomjerry --speed 0.8          # 默认 0.6x 左右，可调
trix mario                         # 超级马里奥（默认 ASCII；--unicode 实心精灵）
trix ecg                           # 心率跳动（默认 ASCII；--unicode 实心字形）
trix dandelion                     # 蒲公英飘散
trix fish                          # 水族馆
trix landscape                     # 五光十色山水
trix shanghai                      # 东方明珠（ASCII）
trix tiananmen                     # 光芒万丈
trix traffic --street vertical     # 车水马龙纵向
trix fairy                         # 仙女散花
trix tornado                       # 龙卷风
trix storm                         # 电闪雷鸣
trix monalisa                      # 蒙娜丽莎人物像（色块 + 字符，静态）
trix ballet --speed 1.3            # 冰上芭蕾（双人冰舞，39 秒编舞循环）
trix demoneye                      # 恶魔之眼（随机注视 / 眨眼）
trix demoneye --eye-still          # 眼球钉在正中，只保留眨眼与瞳孔动态
trix life                          # 生命的轨迹（默认 120 秒一轮）
trix life --life-cycle 60          # 更快，1 分钟走完 0～100 岁
trix tomjerry --jump               # 开启跳跃
trix --list                        # 列出特效和主题

# 参数示例
trix matrix --unicode                       # Unicode 大屏
trix blackhole --spin 0.25 --meteors 8      # 更慢旋转 + 更多流星
trix beams --density 1.5 --beam-speed 1.2   # 更密、更快
trix beams --beam-motion jump              # 上下起伏
trix fireworks --density 2                  # 更多烟花
trix galaxy --density 1.5 --twinkle 2 --meteors 8
trix tomjerry --chase-speed 0.8 --jump      # 追逐速度 / 跳跃
trix mario --unicode                        # 马里奥 Unicode 版
trix ecg --unicode                          # 心电 Unicode 版
trix dandelion --seed-count 180 --seed-speed 1.4
trix shanghai --unicode                       # 东方明珠 Unicode
trix traffic --street vertical --unicode      # 纵向 Unicode
trix fairy --petal-count 160 --petal-speed 1.5
trix tornado --tornado-count 5 --tornado-speed 1.6
trix monalisa --unicode                      # 半块 Unicode 版（更细腻）
trix monalisa --ghost                        # 纯字符鬼影版（会眨眼）
trix monalisa --gallery                      # 人物后方加可选展厅光晕
trix ballet --speed 1.3                      # 更快节奏
trix demoneye                                # 恶魔之眼
```

## 直接运行：自动循环播放

不带特效名直接运行 `trix`，会在**当前终端循环播放全部 27 个特效**：

```bash
trix                                   # 默认 cycle，每 12 秒切换
trix --playlist random --switch 8      # 随机顺序，每 8 秒切换
trix --playlist cycle --switch 20      # 循环顺序，每 20 秒切换
trix --transition dissolve --switch 6  # 用交叉溶解代替渐隐渐显
```

切换默认是 0.6 秒的 fade（先渐隐到黑，再让下一个特效渐显）；`--transition dissolve` 会让两个画面逐格交叉溶解，`--transition none` 则恢复原来带白闪的硬切。过渡时长最多占 `--switch` 的一半，因此短间隔下每个特效仍有完整可见时间。按 `n`/`p` 手动切换，会自动重置计时。指定特效名时（如 `trix fire`）不会自动轮播。

## 命令行

```text
trix <effect> [options]

  -e, --effect <name>     与位置参数等价；省略则循环播放
      --playlist <mode>   cycle | random                      [默认: cycle]
      --switch <secs>     每个特效停留秒数                    [默认: 12]
      --transition <mode> fade | dissolve | none              [默认: fade]
      --transition-secs <s> 过渡时长（秒）                    [默认: 0.6]
      --no-fade           --transition none 的别名
  -f, --fps <1-240>       目标帧率                          [默认: 30]
  -s, --speed <0.05-8>    动画速度倍率                      [默认: 1.0]
      --chase-speed <f>   --speed 的别名（Tom & Jerry）
  -t, --theme <name>      配色主题                          [特效默认]
      --charset <mode>    kana | ascii | unicode                [默认: ascii]
      --unicode           Opt in to Unicode single-cell glyphs
      --ascii             Force ASCII glyphs (the default)
      --color <mode>      auto | truecolor | 256 | 16 | mono [默认: auto]
      --density <f>       beams/fireworks/galaxy/traffic 密度  [默认: 1.0]
      --spin <f>          black hole rotation speed          [默认: 0.5]
      --twinkle <f>       galaxy twinkle rate                [默认: 1.0]
      --meteors <n>       black hole / galaxy meteors        [默认: 3]
      --beam-speed <f>    beams horizontal speed             [默认: 1.0]
      --beam-motion <m>   straight | jump                    [默认: straight]
      --beam-jump         --beam-motion jump 的别名
      --life-cycle <secs> life: 0～100 岁一轮时长              [默认: 120]
      --eye-motion <m>    demoneye: dart | still              [默认: dart]
      --eye-still         --eye-motion still 的别名
      --ghost             monalisa: 纯 ASCII 鬼影版（无背景色块）
      --gallery           monalisa: 淡暖色展厅光晕（可选）
      --plain             dragon: 纯星空背景（无云海/远山）
      --seed-count <n>    dandelion seed count                [默认: 90]
      --seed-speed <f>    dandelion seed drift speed          [默认: 1.0]
      --street <dir>      horizontal | vertical               [默认: horizontal]
      --petal-count <n>   仙女散花花瓣数量                    [默认: 80]
      --petal-speed <f>   花瓣下落速度                        [默认: 1.0]
      --tornado-count <n> 龙卷风数量                          [默认: 3]
      --tornado-speed <f> 龙卷风移动速度                      [默认: 1.0]
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
| `f` / `l` / `b` / `j` | 水族馆 / 山水 / 东方明珠 / 天安门 |
| `c` / `x` / `z` / `k` | 车水马龙 / 仙女散花 / 龙卷风 / 电闪雷鸣 |
| `o` / `v` / `g` | 蒙娜丽莎 / 冰上芭蕾 / 恶魔之眼 |
| `y` | 生命的轨迹 |
| `t` | 循环主题 |
| `+` / `-` | 加速 / 减速（0.1×–8×），Tom & Jerry 同样生效 |
| `r` | 用新种子重开当前特效 |
| `i` | 显示 / 隐藏状态行（FPS、耗时、主题、尺寸、种子） |
| `h` / `?` | 帮助面板 |

数字键顺序：`1 rain`、`2 fire`、`3 dragon`、`4 map`、`5 matrix`、`6 blackhole`、`7 beams`、`8 fireworks`、`9 solar`、`0 galaxy`、`[ buddha`、`] tomjerry`；字母键：`m mario`、`e ecg`、`d dandelion`、`f fish`、`l landscape`、`b shanghai`、`j tiananmen`、`c traffic`、`x fairy`、`z tornado`、`k storm`、`o monalisa`、`v ballet`、`g demoneye`、`y life`。

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
- **Linux x86_64**：`x86_64-unknown-linux-musl` 静态二进制，不依赖目标机器的 glibc。
- **Linux ARM64**：`aarch64-unknown-linux-musl` 静态二进制，用 `cross` 交叉编译并在 QEMU 下做启动冒烟测试，不依赖目标机器的 glibc；可直接运行在 ARM64 服务器、64 位树莓派 4/5、AWS Graviton 等设备上。
- Release 资产：
  - `trix-<version>-macos-universal.tar.gz`
  - `trix-<version>-linux-x86_64.tar.gz`
  - `trix-<version>-linux-aarch64.tar.gz`
  - `SHA256SUMS`
- 也可以在 GitHub Actions 页面手动 `Run workflow` 只构建 artifacts（不会创建 Release）。

发布新版本：

```bash
git tag -a v0.8.0 -m "trix v0.8.0"
git push origin master
git push origin v0.8.0
```

> macOS 二进制未做 Apple Developer 签名/公证；首次下载如被 Gatekeeper 拦截，可右键打开，或 `xattr -d com.apple.quarantine trix`。

## 性能

环境：Apple Silicon (ARM64)、macOS、Rust 1.96.1、`--release`（LTO + `codegen-units=1` + `panic=abort` + `strip`）。
画布 160×48，240 帧离屏压测：

| 特效 | 平均渲染 | p95 | diff 输出 |
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
| fairy | 0.018 ms | 0.022 ms | 2.0 KiB/帧 |
| tornado | 0.043 ms | 0.051 ms | 7.8 KiB/帧 |
| storm | 0.109 ms | 0.122 ms | 29.9 KiB/帧 |
| monalisa | 0.021 ms | 0.032 ms | 0.0 KiB/帧（静态后） |
| ballet | 0.120 ms | 0.151 ms | 14.5 KiB/帧 |
| demoneye | 0.273 ms | 0.398 ms | 15.4 KiB/帧 |
| life | 0.158 ms | 0.209 ms | 5.6 KiB/帧 |

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
| dandelion | 0.28% | 2.08 MiB | 0.5 MiB |
| fish | 0.21% | 2.14 MiB | 0.4 MiB |
| landscape | 0.25% | 2.08 MiB | 0.2 MiB |
| shanghai | 0.32% | 2.09 MiB | 0.3 MiB |
| tiananmen | 0.35% | 2.08 MiB | 0.8 MiB |
| traffic | 0.26% | 2.14 MiB | 0.3 MiB |
| fairy | 0.18% | 2.08 MiB | 0.1 MiB |
| tornado | 0.27% | 2.08 MiB | 0.7 MiB |
| storm | 0.63% | 2.19 MiB | 3.6 MiB |

- 30 FPS 下约 33.3 ms/帧预算；最重的 `fire` 离屏渲染约 **1.3% 单核**（0.433 ms/帧），实际交互约 **1.4%**（含 PTY 写入）。
- 启动：`--version` 中位数约 **2.0 ms**；真实 PTY 首屏字节中位约 **1.6–2.4 ms**。
- 内存：真实 PTY 最大 RSS **2.08–2.27 MiB**。
- 二进制：**733 KiB**。
- 交互模式会遵守 `NO_COLOR`；离屏 `--snapshot` / `--bench` 默认 truecolor，可用 `--color` 覆盖。

```bash
scripts/bench.sh
./target/release/trix fire --bench --size 160x48 --frames 300
```

## 实现

- **终端驱动**（`src/term.rs`）：libc 设置 raw mode、备用屏幕、隐藏光标、关闭自动换行；非阻塞读键、`TIOCGWINSZ` 尺寸检测、SIGINT/SIGTERM/SIGHUP 标志、panic hook。
- **Canvas**（`src/canvas.rs`）：`Cell { char, fg, bg }` 双缓冲；逐行变化区间；前景/背景变化都发 SGR，背景回落时发 `40` 回到程序启动时的黑底（避免切场景后残留上一幕的底色）；支持 truecolor/256/16/mono（mono 只画字形）。`monalisa` 的色块人物像使用背景色，`mario` 不使用背景色，纯靠字形与前景色。
- **零框架**：唯一依赖 `libc`；没有 tokio/crossterm/线程任务。
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
- 同时支持 ASCII / Unicode 的特效**默认一律 ASCII**；需要 Unicode 时加 `--unicode` 或 `--charset unicode`。`matrix` 的 Unicode 版包含蓝白大字、Unicode 边框/进度条；`--charset kana` 可混入半角片假名。
- Linux 二进制是 `x86_64` 与 `aarch64` 的静态 musl 版本，可直接运行在这两种架构的任意发行版（包括 Alpine）上。
- 非 TTY 会提示使用 `--snapshot`。
- 画布黑底；建议深色终端。

## 测试

```bash
cargo test
cargo clippy --all-targets
cargo fmt --check
```

当前 **144 个**单元/集成测试：颜色转换、按键解析、CLI/新参数校验、Canvas 裁剪/差分、地图掩膜、27 个特效多尺寸/主题/颜色/Unicode 降级、默认字符集 ASCII-only、小终端不 panic、确定性种子、Tom & Jerry 速度/跳跃、龙卷风高不见顶/恶魔之眼 8～15 秒眨眼节奏/注视/看出去再回正/瞳孔不被高光遮挡/静止开关、芭蕾牵手/满屏路径、生命的轨迹世代/时间轴/狗猫/符号/核心 0 次变轨/死亡头部/终点符号等。

真实 PTY 实测：`q`/`Esc`/`Ctrl-C`/`SIGTERM`、暂停、帮助、数字/`[`/`]` 切换、`i` 状态、缩放、`--no-alt`、颜色降级、退出恢复。

## 版本与回退

- **当前版本：0.8.0**（`trix --version`），tag **`v0.8.0`**。
- 上一版：tag **`v0.7.0`**（水族馆/山水/东方明珠/天安门/车水马龙/仙女/龙卷风/雷暴、播放列表）；**`v0.6.0`**（马里奥/心电/蒲公英、蓝绿代码雨）；**`v0.5.1`**（自动发布流水线）。
- 更早：tag **`v0.4.0`**（Unicode NASA 登录、连通火焰、可调参数）；**`v0.3.0`**（多堆火焰、7 个新特效）；**`v0.2.0`**（中国龙、NASA 入侵）；**`v0.1.0`**（初始基线）。
- 回退：
  ```bash
  git checkout v0.7.0     # 只看上一版（detached HEAD）
  git switch master
  git reset --hard v0.7.0 # 彻底退回上一版
  ```
- 额外备份（不进 git）：`backups/trix-v0.1.0-*` 到 `trix-v0.7.0-*`，含源码 tar 与 aarch64 macOS 二进制。
- 改动记录：[中文](CHANGELOG.zh-CN.md) · [English](CHANGELOG.md)。

## 数据来源、参考与许可

- 陆地轮廓：[Natural Earth](https://www.naturalearthdata.com/) 1:110m land，**public domain**，经 [natural-earth-vector](https://github.com/nvkelso/natural-earth-vector) GeoJSON 转换。
- Doom-fire 灵感：[pserey/doom-fire-algorithm](https://github.com/pserey/doom-fire-algorithm)、[horizonwiki/fire](https://github.com/horizonwiki/fire)；本项目的 Rust 实现、多燃料堆、配色与渲染均为独立编写。
- 佛祖 ASCII 艺术改编自 [ascii.co.uk/art/buddha](https://ascii.co.uk/art/buddha)（原作者署名 `hjw`），用于 `buddha` 场景；本项目补充了颜色、光环、莲座、粒子与动画。
- 本项目代码：[MIT](LICENSE)。

---

```text
  t r i x  ::  stay curious, look up.
```
