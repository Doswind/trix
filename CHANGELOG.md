# Changelog

## 0.5.1 — 2026-10-17

- 新增 GitHub Actions 自动构建/发布：`.github/workflows/release.yml`。
- tag `v*` 自动生成 **macOS universal**（arm64 + x86_64）与 **Linux x86_64 static musl** 发布包，并创建 GitHub Release。
- 版本号提升到 `0.5.1`。

## 0.5.0 — 2026-10-17

- 项目正式从 `texiao` 更名为 **`trix`**；Rust package/binary、CLI 帮助、文档、脚本与工具全部同步改名。
- 版本号提升到 `0.5.0`；此前 **v0.1.0 / v0.2.0 / v0.3.0 / v0.4.0** 的提交、tag 与备份全部保留，可回溯。
- 功能与 v0.4.0 一致（Unicode NASA 登录、连通火焰、多组可调参数等）。

## 0.4.0 — 2026-10-17

- **火焰 `fire`**：底部火焰连成连续火床，上方保留多股分开的细长火舌；中间燃料堆固定最高，其余随机高度。
- **NASA `matrix`**：改为 **Unicode 蓝色 NASA 横幅**（白色大字、红色 swoosh、白色星点、蓝白流光），NASA 与登录框整体下移并居中；登录框、进度条、雷达改用 Unicode 边框/块字符；保留 `--ascii` 纯 ASCII 回退。
- **黑洞 `blackhole`**：夜空随机方向流星；`--spin` 调整吸积盘/内落旋转速度，默认比旧版慢；`--meteors` 调整流星数量。
- **光束 `beams`**：改为单向（左→右），`--density` 调密度、`--beam-speed` 调速度。
- **烟花 `fireworks`**：`--density` 调烟花密度。
- **太阳系 `solar`**：行星改为更圆的球体；左上角增加金色发散太阳光；太阳/行星比例修正。
- **银河 `galaxy`**：增加动态星团弧、星光闪烁、流星；`--density`、`--twinkle`、`--meteors` 可调。
- **佛光 `buddha`**：佛祖整体垂直居中。
- **Tom & Jerry**：默认速度放慢，默认不跳跃；`--speed`/`+`/`-` 调速，`--jump` 开启跳跃；HUD 显示有效追逐倍率。
- 新增全局参数：`--density`、`--spin`、`--twinkle`、`--meteors`、`--beam-speed`、`--jump/--no-jump`、`--unicode/--ascii`、`--charset unicode`。
- 版本提升到 `0.4.0`；`v0.3.0` 及更早 tag/备份保留用于回退。
- 39 个测试通过；真实 PTY CPU 约 **0.32–1.61%** 单核、RSS **2.05–2.28 MiB**、二进制约 **603 KiB**。

## 0.3.0 — 2026-10-17

- **火焰 `fire`**：改为多个随机燃料堆（位置/宽度/热量/漂移独立），形成多股细长火舌，不再是单一火堆；保留真实火焰色、火星与爆燃。
- **黑客帝国 `matrix`**：重做为 **NASA 后台登录大屏**。大号 ASCII NASA 字样（蓝/白/红流光）、OPERATOR/PASSWORD/2FA 字段、AUTH/DECRYPT 进度、日志、雷达、ACCESS GRANTED。
- **新增 7 个特效**：
  - `blackhole` 黑洞吞噬恒星：事件视界、连续光子环、吸积盘、透镜弧、喷流、内落恒星与计数。
  - `beams` 横向随机亮色不规则光束。
  - `fireworks` 双层粒子烟花、延迟爆裂、城市/星空。
  - `solar` 太阳与九大行星一维排列，独立颜色/大小、土星环、卫星、彗星。
  - `galaxy` 满屏星河、中央稠密银河带、暗尘与流星。
  - `buddha` 佛光普照：ASCII 佛祖、头光/光环、莲座、曼陀罗、金色粒子。
  - `tomjerry` Tom 追 Jerry 的太空站动画，`--speed`/`+`/`-` 调整追逐速度。
- 所有新场景增加满屏 ASCII 点缀（星空、网格、粒子、光晕、城市/走廊）。
- 快捷键扩展到 `1`–`9`、`0`、`[`、`]`；帮助面板列出 12 个特效。
- 版本号提升到 `0.3.0`；`v0.1.0` / `v0.2.0` tag 与备份保留。
- 真实 PTY 实测：CPU 约 **0.43–2.93%** 单核、RSS 约 **2.03–2.27 MiB**、二进制约 **587 KiB**、启动约 2 ms；37 个测试通过。

## 0.2.0 — 2026-10-17

- **火焰 `fire`**：更高、更明亮的火舌；底部宽、顶部收窄的三角轮廓；保留真实红/橙/黄/白火焰色、火星与爆燃。
- **黑客帝国 `matrix`**：改为横向 NASA/JPL 入侵控制台。横向代码/十六进制数据流、防火墙与解密进度、日志打字机、雷达、ACCESS GRANTED、glitch；默认 ASCII，`--charset kana` 可混入半角片假名。
- **巨龙 `dragon`**：重做为全屏飞行的中国龙。长蛇形龙身按弧长跟随飞行轨迹，带鹿角、龙须、鬃毛、獠牙、四爪、背鳍、尾鳍、龙鳞与金色脊线；在整屏 Lissajous 航线上飞行。
- 版本号提升到 `0.2.0`；`v0.1.0` 的 git tag、源码 tar 包与二进制备份保留用于回退。
- 新增本 CHANGELOG。

## 0.1.0 — 2026-10-17

- 初始版本：`rain` / `fire` / `dragon` / `map` / `matrix` 五个特效。
- libc raw mode、diff canvas、truecolor/256/16/mono、8 套主题、snapshot/bench、交互快捷键。
- 真实 PTY 验证与性能测量。
