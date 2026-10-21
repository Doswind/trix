# trix-effects

**English** | [简体中文](README.zh-CN.md)

> **35 animated ASCII / Unicode effects for your terminal, in one fast, dependency-free Rust binary.** Code rain, fire, a coiling dragon, the Matrix, a black hole, fireworks, the solar system, Mona Lisa, a giant DeepSeek Harness wordmark, a self-typing Claude Code session, and 25 more — in ASCII by default, with Unicode as an opt-in.

![release](https://github.com/Doswind/trix-effects/actions/workflows/release.yml/badge.svg)
![CI](https://github.com/Doswind/trix-effects/actions/workflows/ci.yml/badge.svg)
![license](https://img.shields.io/badge/license-MIT-blue.svg)
![rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
![platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20WSL-lightgrey.svg)

![trix-effects overview](docs/overview-ascii.png)

**35 effects · 8 themes · zero dependencies · ~2 ms startup · ~1.1 MB binary · macOS / Linux / WSL**

[Install](#install) · [Quick start](#quick-start) · [Help](#help-that-looks-like-the-product) · [Effects](#effects) · [ASCII / Unicode](#ascii--unicode-editions) · [Keys](#runtime-keys) · [Themes](#themes) · [Performance](#performance) · [FAQ](#compatibility-and-faq)

> The command is **`trix`**; the package and repository are **`trix-effects`** — the same short-command split as `ripgrep` / `rg`. The old name `texiao` was renamed to `trix` in v0.5.0 and to the package name `trix-effects` in v0.9.1.

## Install

```bash
# macOS / Linux, prebuilt (uses the latest GitHub release)
curl -fsSL https://raw.githubusercontent.com/Doswind/trix-effects/master/install.sh | sh

# Rust toolchain, straight from git
cargo install --git https://github.com/Doswind/trix-effects --locked

# From a clone
git clone https://github.com/Doswind/trix-effects
cd trix-effects
cargo install --path . --locked
```

The binary is installed as `trix` and needs no runtime dependencies. Release assets are static Linux `x86_64` / `aarch64` builds and a macOS universal binary; `install.sh` verifies `SHA256SUMS` before installing to `$HOME/.local/bin` (override with `TRIX_INSTALL_DIR`).

Requires Rust 1.70+ when building from source.

## Quick start

```bash
trix                              # cycle all 35 effects
trix rain                         # Code Rain
trix dragon --theme gold          # golden dragon
trix blackhole --spin 0.25        # slower black hole
trix monalisa                     # Mona Lisa (static, ASCII by default)
trix monalisa --ghost             # pure-ASCII ghost edition
trix matrix --unicode             # NASA wall, Unicode edition
trix --playlist random --switch 8 # random order, 8 s each
trix --help rain                  # effect details + live preview
trix --list                       # catalog + themes
trix rain --snapshot rain.ans --size 120x36 --frames 90
```

More one-command examples:

```bash
trix fire --theme ember            # warm fire
trix dragon --plain                # starfield-only backdrop
trix map --theme ice               # holographic world map
trix beams --beam-motion jump      # rising and falling beams
trix fireworks --density 2         # more fireworks
trix galaxy --density 1.5 --twinkle 2 --meteors 8
trix mario                         # ASCII by default
trix ecg                           # ASCII by default
trix dandelion --seed-count 180 --seed-speed 1.4
trix traffic --street vertical     # top-down traffic
trix tornado --tornado-count 5 --tornado-speed 1.6
trix ballet --speed 1.3            # faster ice ballet
trix demoneye --eye-still          # pin the eyeball, keep blinking
trix life --life-cycle 60          # 0–100 in one minute
trix dsh                           # DEEPSEEK HARNESS wordmark
trix dsh --trail 0                 # no drop ghost, beam only
trix vibe --unicode                # Claude Code glyphs, self-typing source
trix tunnel                        # warp tunnel
trix plasma                        # plasma field
trix launch                        # pad to orbit
trix tomjerry --jump               # enable jumping
```

## Help that looks like the product

`trix --help` is not a plain flag dump. It renders the wordmark, a real frame of the HUD, a colour-coded effect grid, grouped options, theme swatches and copy-paste examples. `trix --help <effect>` adds a **live preview rendered by the actual scene**, plus that effect's own tuning flags.

![trix --help](docs/help.png)

![trix --help rain](docs/help-rain.png)

[Full help screenshot](docs/help-full.png)

```bash
trix --help            # overview
trix --help rain       # effect dossier + real preview
trix --help all        # full option reference
trix --list            # styled catalog on a TTY, plain text when piped
```

The same layout degrades to aligned plain text when stdout is not a TTY, when `NO_COLOR` is set, or with `--no-color`. `--ascii` forces ASCII borders; `--unicode` forces Unicode borders. Unknown effects and options get a “did you mean?” hint.

## Effects

Every effect name is given in English and Chinese; `trix --list` uses the same names.

| id | English | 中文 |
| --- | --- | --- |
| `rain` | Code Rain | 代码雨 |
| `fire` | Blazing Fire | 燃烧火焰 |
| `dragon` | Coiling Dragon | 巨龙盘旋 |
| `map` | World Map | 世界地图 |
| `matrix` | The Matrix | 黑客帝国 |
| `blackhole` | Black Hole | 黑洞 |
| `beams` | Light Beams | 光束 |
| `fireworks` | Fireworks | 烟花 |
| `solar` | Solar System | 太阳系 |
| `galaxy` | Starry Galaxy | 满天星河 |
| `buddha` | Buddha's Halo | 佛光普照 |
| `tomjerry` | Tom & Jerry | 猫和老鼠 |
| `mario` | Super Mario | 超级马里奥 |
| `ecg` | Heartbeat Monitor | 心率跳动 |
| `dandelion` | Dandelion | 蒲公英 |
| `fish` | Aquarium | 水族馆 |
| `landscape` | Colorful Landscape | 五光十色 |
| `shanghai` | Oriental Pearl | 东方明珠 |
| `tiananmen` | Tiananmen Radiance | 光芒万丈 |
| `traffic` | Traffic Flow | 车水马龙 |
| `tornado` | Tornado | 龙卷风 |
| `storm` | Thunderstorm | 电闪雷鸣 |
| `monalisa` | Mona Lisa | 蒙娜丽莎 |
| `ballet` | Ice Ballet | 冰上芭蕾 |
| `demoneye` | Demon Eye | 恶魔之眼 |
| `life` | Threads of Life | 生命的轨迹 |
| `dsh` | DeepSeek Harness | 深度求索 |
| `vibe` | Vibe Coding | 氛围编程 |
| `tunnel` | Warp Tunnel | 星际隧道 |
| `helix` | DNA Helix | DNA 双螺旋 |
| `lights` | Aurora | 极光 |
| `volcano` | Volcano | 火山爆发 |
| `tesseract` | Tesseract | 四维超立方 |
| `launch` | Rocket Launch | 火箭发射 |
| `plasma` | Plasma | 等离子 |

> **Screenshots use the default ASCII glyphs.** Unicode variants are shown only in the [ASCII / Unicode](#ascii--unicode-editions) section. All previews are rendered from the current code at 120×36 (Mona Lisa at 120×60; DeepSeek Harness at 160×48), truecolor.

| Code Rain `rain` | Blazing Fire `fire` | Coiling Dragon `dragon` |
| --- | --- | --- |
| ![Code Rain](docs/rain-ascii.png) | ![Blazing Fire](docs/fire-ascii.png) | ![Coiling Dragon](docs/dragon-ascii.png) |

| World Map `map` | The Matrix `matrix` | Black Hole `blackhole` |
| --- | --- | --- |
| ![World Map](docs/map-ascii.png) | ![The Matrix](docs/matrix-ascii.png) | ![Black Hole](docs/blackhole-ascii.png) |

| Light Beams `beams` | Fireworks `fireworks` | Solar System `solar` |
| --- | --- | --- |
| ![Light Beams](docs/beams-ascii.png) | ![Fireworks](docs/fireworks-ascii.png) | ![Solar System](docs/solar-ascii.png) |

| Starry Galaxy `galaxy` | Buddha's Halo `buddha` | Tom & Jerry `tomjerry` |
| --- | --- | --- |
| ![Starry Galaxy](docs/galaxy-ascii.png) | ![Buddha's Halo](docs/buddha-ascii.png) | ![Tom & Jerry](docs/tomjerry-ascii.png) |

| Super Mario `mario` | Heartbeat Monitor `ecg` | Dandelion `dandelion` |
| --- | --- | --- |
| ![Super Mario](docs/mario-ascii.png) | ![Heartbeat Monitor](docs/ecg-ascii.png) | ![Dandelion](docs/dandelion-ascii.png) |

| Aquarium `fish` | Colorful Landscape `landscape` | Oriental Pearl `shanghai` |
| --- | --- | --- |
| ![Aquarium](docs/fish-ascii.png) | ![Colorful Landscape](docs/landscape-ascii.png) | ![Oriental Pearl](docs/shanghai-ascii.png) |

| Tiananmen Radiance `tiananmen` | Traffic Flow `traffic` | Tornado `tornado` |
| --- | --- | --- |
| ![Tiananmen Radiance](docs/tiananmen-ascii.png) | ![Traffic Flow](docs/traffic-ascii.png) | ![Tornado](docs/tornado-ascii.png) |

| Thunderstorm `storm` | Mona Lisa `monalisa` | Ice Ballet `ballet` |
| --- | --- | --- |
| ![Thunderstorm](docs/storm-ascii.png) | ![Mona Lisa](docs/monalisa-ascii.png) | ![Ice Ballet](docs/ballet-ascii.png) |

| Demon Eye `demoneye` | Threads of Life `life` | DeepSeek Harness `dsh` |
| --- | --- | --- |
| ![Demon Eye](docs/demoneye-ascii.png) | ![Threads of Life](docs/life-ascii.png) | ![DeepSeek Harness](docs/dsh-ascii.png) |

| Vibe Coding `vibe` |
| --- |
| ![Vibe Coding](docs/vibe-ascii.png) |

| Warp Tunnel `tunnel` | DNA Helix `helix` | Aurora `lights` |
| --- | --- | --- |
| ![Warp Tunnel](docs/tunnel-ascii.png) | ![DNA Helix](docs/helix-ascii.png) | ![Aurora](docs/lights-ascii.png) |

| Volcano `volcano` | Tesseract `tesseract` | Rocket Launch `launch` |
| --- | --- | --- |
| ![Volcano](docs/volcano-ascii.png) | ![Tesseract](docs/tesseract-ascii.png) | ![Rocket Launch](docs/launch-ascii.png) |

| Plasma `plasma` |
| --- |
| ![Plasma](docs/plasma-ascii.png) |

<details>
<summary><b>Full effect notes (35)</b></summary>

| Effect | Description |
| --- | --- |
| Code Rain (代码雨) `rain` | Per-column heads and tails, randomized speed and depth, white-hot heads, **blue/green two-tone tails**, binary mist, horizontal data packets, CRT scanline |
| Blazing Fire (燃烧火焰) `fire` | **The bottom merges into one wide fire bed**; the tallest flames sit in the middle, while the tongues on both sides lean outward; embers and flare-ups |
| Coiling Dragon (巨龙盘旋) `dragon` | A Chinese dragon flying across the whole screen on a Lissajous path: arc-length body tracking, antlers, whiskers, mane, fangs, four claws, dorsal fin, tail fin, scales; a traveling wave runs from head to tail along the entire body; the default backdrop adds two ink-wash mountain ridges and a denser drifting cloud sea; `--plain` restores the old starfield-only backdrop |
| World Map (世界地图) `map` | Embedded Natural Earth land mask, city nodes, great-circle data packets, scan beams, UTC clock |
| The Matrix (黑客帝国) `matrix` | **A NASA backend login wall**: **ASCII by default**; `--unicode` switches to a blue NASA banner with white wordmark, red swoosh and a Unicode login frame; OPERATOR/PASSWORD/2FA, AUTH/DECRYPT, radar, ACCESS GRANTED |
| Black Hole (黑洞) `blackhole` | Event horizon, photon ring, accretion disk, lensing arcs, jets, meteors; **the outer stars form a full "/"-tilted ellipse** that fills the screen and presses past the top and bottom edges; the outermost ring is sparse `.` / `:` / `+` scatter with a denser middle band; every layer rotates in the same direction with differential speed (fast inside, slow outside), reading as one rotating elliptical disk |
| Light Beams (光束) `beams` | **One-way** horizontal beams in random bright colors; `--beam-motion straight\|jump` (straight by default), `--density` / `--beam-speed` |
| Fireworks (烟花) `fireworks` | Rockets climb, then inner and outer particle rings bloom with delayed bursts; `--density` controls the firework density; city skyline and starfield |
| Solar System (太阳系) `solar` | **Eight planets** (no Pluto); the Sun sits in the top-left, light spreads in layered fans toward the lower right and stops above the planet line; round planets, Saturn's rings, moons, a comet |
| Starry Galaxy (满天星河) `galaxy` | A full-screen star sea with a dense central Milky Way band; animated star clusters, dark dust, constellation lines; `--density`, `--twinkle`, `--meteors` |
| Buddha's Halo (佛光普照) `buddha` | Outward mandala rings **expand at equal spacing**: every ring shares one expansion phase and the radial gap stays constant; **the first ring starts outside the whole Buddha figure (body, head halo, lotus base)**, so it never covers the figure or overlaps the halo; at most 5 rings; ring brightness stays below the Buddha's gold (0.6 on the inner ring) and fades toward the outside; head halo, lotus base and particles are kept |
| Tom & Jerry (猫和老鼠) `tomjerry` | Tom chasing Jerry through a space station; slowed down and non-jumping by default, `--speed` / `+` `-` to adjust, `--jump` to enable jumping |
| Super Mario (超级马里奥) `mario` | **A pure-glyph 1-1 level**: clouds, hills and bushes, a brick floor with mortar seams, a **question-block / brick / used-block** row, green pipes, spinning coins, Goombas and mushrooms; auto running and jumping, blocks that pop out coins or mushrooms, stomped enemies, `+200/+100/+1000` score popups. **Uses no terminal background color at all**: ASCII colored glyphs by default (`# o = *`); `--unicode` switches the **sprites** (Mario, Goombas, mushrooms, coins) to `█ ▀ ▄` solid blocks, while hills, clouds, pipes and bricks stay glyph-based and are colored per object |
| Heartbeat Monitor (心率跳动) `ecg` | A green ECG monitor; **about 56 BPM with random beat intervals** and clearly random amplitude, QRS width and flat-segment length; **ASCII by default**, `--unicode` to switch |
| Dandelion (蒲公英) `dandelion` | 3 clumps × 2 stems, 6 flower heads; the roots and lower stems of the left and right clumps bend toward the center; **flower heads are 1.5× larger**, everything else unchanged |
| Aquarium (水族馆) `fish` | A full-screen aquarium: **5 hand-drawn sprite fish** (small 20×5 / medium 30×7 / large 38×8, forked tails, bright eyes and mouths; only the tails swish, the head and mouth stay fixed), plus 8–30 small `<==o>` / `<=-o>` fish (about 35% double-layer, 15% triple-layer bold, one eye only); bubbles use `o` / `O` / `8` with **a random shade per bubble**, each rising **slowly in a straight line** along a fixed column and swaying no more than 3 times over the whole climb; the count settles at 10–36; plants get random heights and spacing and grow in small patches; ASCII by default, `--unicode` switches to `█ ▓ ▒` |
| Colorful Landscape (五光十色) `landscape` | Layered distant mountains, a calm lake, a boat, white geese, jungle on both sides; animated water, mist and petals |
| Oriental Pearl (东方明珠) `shanghai` | The Shanghai Bund at night, the Oriental Pearl Tower, neon high-rises, Huangpu River reflections; **two random boats on the river, each with a different size, shape and position** (junk / barge / ferry, with deck, portholes, lit cabin windows, smoking funnel, mast lights, wake and water reflection); **1–2 meteors cross the night sky every 3–5 seconds** (white-hot head plus fading tail); **ASCII by default**, `--unicode` available |
| Tiananmen Radiance (光芒万丈) `tiananmen` | Tiananmen gate enlarged to about **92% of the width and 62% of the height**: double golden-tile eaves with upturned corners, a red-pillared gate tower with the national emblem, red walls, five archways with white stone surrounds, white-framed slogans and the central portrait, red flags and side buildings; golden beams first grow outward and upward from behind the roof, then light up in order from the inside out, cycling |
| Traffic Flow (车水马龙) `traffic` | City street with high-rises on both sides, neon and traffic; horizontal traffic is 3 rows of cars (roof / body / wheels, 8–16 cells long) across 4–5 lanes with dashed dividers and alternating directions; vertical traffic is 6 columns of top-down cars (5–9 rows, front/rear lights and side wheels); `--street horizontal\|vertical`; `--density` controls traffic density; **ASCII by default**, `--unicode` available |
| Tornado (龙卷风) `tornado` | 1–8 tornadoes move from left to right; twisted, pointed at the base and wider at the top; overall **one step shorter** than before (0.28–0.58 of the screen height) and **lighter and sparser** (brightness 0.42–0.85, 20 particles per row); 30 debris particles **drift outward** at the base |
| Thunderstorm (电闪雷鸣) `storm` | Dark clouds, slanting rain, a city silhouette, lightning; a layer of **gray-white, irregular high cloud** sits above the screen (denser, fading out at the bottom, **drifting slowly to the right** at about 3 columns per second) to add depth against the existing cloud band; lightning lights the whole screen white ([still](docs/storm-flash-ascii.png)) |
| Mona Lisa (蒙娜丽莎) `monalisa` | A background-free portrait: the figure (hair, face, neck and shoulders, folded hands) is baked from a reference pixel painting; ANSI color blocks carry skin tones and shading, with ASCII glyphs for detail; no landscape, frame or floating dust; the figure is about 12% wider, closer to the original proportions; **the default block version is static and does not blink**; `--unicode` uses half blocks for more vertical resolution; `--ghost` switches to the original pure-ASCII “ghost” version (no background blocks, blinking kept, [still](docs/monalisa-ghost.png) / [GIF](docs/monalisa-ghost.gif)); optional `--gallery` adds a faint warm gallery halo and floor glow behind the portrait ([preview](docs/monalisa-gallery.png)) |
| Ice Ballet (冰上芭蕾) `ballet` | An ice-ballet duet: a 39-scene-second choreography loops through 13 sections; the default 2x tempo makes one loop about 19.5 seconds (glide, crossover, arabesque, promenade, **overhead lift**, pair spin, death spiral, sissonne, Ina Bauer, final pose); joints use two-bone IK, clasped hands align automatically, limbs have soft edges and lag behind the torso; the routine covers most of the screen, three rows of spectators behind the boards sway, raise their hands and fire flashbulbs, and skate marks and reflections stay on the ice; `--speed` |
| Demon Eye (恶魔之眼) `demoneye` | One huge bloodshot eye fills the screen: it blinks every 8–15 seconds, singly or doubly (about 7:3); between blinks the eyeball darts only 1–2 times (darts at least 5 seconds apart), reaching the target in about 0.1 s and always returning to center within 0.45–0.85 s; the vertical pupil scales, eyelids press in, blood vessels and a blink shockwave; `--eye-motion dart\|still` (default `dart`), `--eye-still` pins the eyeball dead center |
| Threads of Life (生命的轨迹) `life` | A 0–100 age timeline with glowing life lines on both sides; every human uses `@` with a colored uppercase abbreviation on the same row and **the relative's own age** (friends who are not family carry no age); the bright beam grows with age (late in life it can cover most of the lifetime), fading from dark dots / dashes at the tail up to `=` at the head, with a `****@` comet tail; the first ~22%–42% of life (at most about 22 years) stays as sparse dim dots; when a track ends its row can be reused by someone later (school-age `PAL/UNI` +1 row → GF/WIFE, `DOG` −1 row → KID2, and the cat / parents' rows are reclaimed by later friends); `DOG` stays at −1 next to me for life; when `PAL/UNI/WORK/FRIEND` coexist they **share one row**: during the school years it sits on the other side at +1, and while a minor KID exists the whole group retreats to the nearest free row outside KID/CAT (usually +4; only rows actually used by children/pets are yielded, no longer fixed −5/−7), and all KIDs move back inside once grown; `ME 0 / WIFE +1`; `KID1 +2`, `KID2 −1` (reusing the row next to me that DOG gave up), `KID3 −2`; before adulthood `KID3 −2 / KID2 −1 / ME 0 / WIFE +1 / KID1 +2` is **contiguous with no empty rows**, and `GRAND1/GRAND2` always hug their parent's row (moving with them as they drift); `CAT +3` is born with the first child, just below KID1; `DAD/MOM +3/+4` start yielding to `+5/+6` two years before the child is born, and `BRO +5→+7` follows; GPA/GMA/DAD/MOM get suitable random starting ages when I am 0 (58–70 / 56–68 / 25–33 / 23–31), BRO 0–4, ME 0; KID/GRAND/CAT/DOG count from their own birth, GF/WIFE are within ±2 years of my age; every row is fixed at one terminal row (it does not scale with terminal height, and tall terminals never produce empty rows from rounding); when `ME` ends, a roughly 6-second **supernova finale** plays (the flashback and stardust slow down while the comet keeps its speed: white core, life flashback, shockwave, stardust, brightened comet and a permanent halo at the top; the flashback beam that reaches ME's stop line stays three columns wide and never disappears); the death point and the top star keep `@` + name + final age and appear/disappear in sync once per second; all the dead rise to the top in death order, one row apart, and their end labels fade in 0.5 s after the meteor arrives; humans blink as `*`, cats and dogs as `+`; GF approaches for 2 years, then stays 1–2 years before becoming WIFE; WIFE lives to 75–90 and dies after I become a star; AGE has no leading zeros; `--life-cycle` sets the length of one round |
| DeepSeek Harness (深度求索) `dsh` | A giant **two-line** wordmark: `DEEPSEEK` on top, `HARNESS` below with a clear indent, filling about **85–90% of the width and over half the screen height**. The intro **fades the letters in one by one**, then **drops the lockup in stuttered one-row steps** and leaves a ghost stack below HARNESS. After that the wordmark **never disappears**: a seven-second loop floats a **horizontal beam** down the lockup, joins it with a **vertical beam** at the centre to form a bright **cross**, then sweeps the vertical beam left to right; no fireworks or particle burst. `--trail <f>` sets the drop-ghost strength; **ASCII by default**, `--unicode` uses solid blocks ([GIF](docs/dsh.gif)) |
| Vibe Coding (氛围编程) `vibe` | A single-column **Claude Code transcript** that reads `src/scenes/vibe.rs` (or an embedded copy), types the source with a hand-written **Rust syntax highlighter**, and loops through six short windows per turn; each loop starts from a clean transcript; it shows the prompt, assistant/tool lines, `+` diff-style code, a rotating spinner with token count, the input box and the bypass-permissions footer; **ASCII by default** (`>`, `*`, `L`, `>>`), `--unicode` switches to Claude Code-style `❯ ◉ └ ✻ ▶▶` glyphs ([GIF](docs/vibe.gif)) |
| Warp Tunnel (星际隧道) `tunnel` | Sixteen depth-spaced rings expand toward the viewer while ten radial spokes rotate and projected stars streak past. **Pure ASCII**: `+`, `*`, `#` and `@` carry the perspective. |
| DNA Helix (DNA 双螺旋) `helix` | Two sine strands wrap around a vertical axis; A/T/C/G base pairs join them, depth comes from each strand's cosine, and the whole helix rotates in place. **Pure ASCII.** |
| Aurora (极光) `lights` | Three wavy aurora ribbons with short hanging curtains drift over a starry sky, snow-capped ridges, a moon and a rippling lake reflection. **Pure ASCII.** |
| Volcano (火山爆发) `volcano` | A jagged cone, a glowing crater, ballistic lava fountains, two lava streams, a drifting ash plume and periodic lightning. **ASCII by default**; `--unicode` uses solid blocks for the mountain. |
| Tesseract (四维超立方) `tesseract` | Sixteen vertices and thirty-two edges rotate in four dimensions and draw as a depth-sorted wireframe: the outer cube is cyan, the inner cube violet, the eight connectors gold and the vertices white. A ghost hypercube, perspective starfield, coloured dust, three orbit rings and travelling edge pulses fill the space. **ASCII by default**; `--unicode` uses box-drawing edges and solid vertices. |
| Rocket Launch (火箭发射) `launch` | A 24-second launch film: a larger two-stage rocket with twin boosters, blue-hour pad, ignition fireball, layered exhaust and world-space smoke, a following camera with shake, max-Q clouds, MECO / stage separation, a second-stage climb and an Earth-limb sunrise; `--rocket-scale` and `--shake`. **Pure ASCII.** |
| Plasma (等离子) `plasma` | Nine drifting metaballs add into a soft scalar field, textured with sine detail and rendered through a cycling palette. **ASCII by default**; `--unicode` uses a compact block ramp. |

</details>

Light Beams has two motion modes:

| Straight (default) | Jumping (`--beam-motion jump`) |
| --- | --- |
| ![beams straight](docs/beams-ascii.png) | ![beams jump](docs/beams-jump.png) |

## ASCII / Unicode editions

**Every dual-form effect starts in ASCII.** Add `--unicode` (or `--charset unicode`) to opt in; `--ascii` forces ASCII again. The dual-form effects are `matrix`, `mario`, `ecg`, `dandelion`, `fish`, `landscape`, `shanghai`, `tiananmen`, `traffic`, `tornado`, `storm`, `monalisa`, `ballet`, `demoneye`, `life`, `dsh`, `vibe`, `volcano`, `tesseract` and `plasma`.

`mario` / `ecg` in both glyph sets:

| ASCII (default) | Unicode (`--unicode`) |
| --- | --- |
| ![mario ascii](docs/mario-ascii.png) | ![mario unicode](docs/mario.png) |
| ![ecg ascii](docs/ecg-ascii.png) | ![ecg unicode](docs/ecg.png) |

`shanghai` / `traffic` in both glyph sets:

| ASCII (default) | Unicode (`--unicode`) |
| --- | --- |
| ![shanghai ascii](docs/shanghai-ascii.png) | ![shanghai unicode](docs/shanghai-unicode.png) |
| ![traffic ascii](docs/traffic-ascii.png) | ![traffic unicode](docs/traffic-unicode.png) |

`dsh` / `vibe` / `tesseract` in both glyph sets:

| ASCII (default) | Unicode (`--unicode`) |
| --- | --- |
| ![dsh ascii](docs/dsh-ascii.png) | ![dsh unicode](docs/dsh-unicode.png) |
| ![vibe ascii](docs/vibe-ascii.png) | ![vibe unicode](docs/vibe-unicode.png) |
| ![tesseract ascii](docs/tesseract-ascii.png) | ![tesseract unicode](docs/tesseract-unicode.png) |

Vertical street:

![traffic vertical](docs/traffic-vertical.png)

Full-screen lightning flash:

![storm flash](docs/storm-flash-ascii.png)

```bash
trix mario              # ASCII classic glyphs by default
trix mario --unicode    # solid Unicode sprites
trix ecg                # ASCII ECG monitor by default
trix ecg --unicode      # Unicode ECG monitor
```

## Features

- **35 switchable effects**: command-line arguments, runtime hot-switching and themes.
- **Help as a first-class surface**: styled overview, per-effect dossier with a real scene preview, styled catalog and “did you mean?” errors.
- **ASCII first**: dual-form effects start in ASCII; opt into Unicode explicitly with `--unicode`.
- **Low resource use**: single-threaded, no async, no event-loop framework; 30 FPS by default; `libc` is the only dependency.
- **Fast startup**: `--version` median about **2.2 ms** (Apple Silicon, release build); the release binary is about **1.1 MB** (1,104 KiB).
- **Low memory**: interactive RSS about **2.1–2.3 MB** (120×36, truecolor).
- **Diff rendering**: double-buffered canvas that repaints only changed cells and merges SGR sequences.
- **Color animation**: 24-bit truecolor; 8 themes; degrades to 256 colors, 16 colors or monochrome.
- **Recordable / benchmarkable**: built-in `--snapshot` and `--bench`.
- **Clean exit**: raw mode, alternate screen, cursor and autowrap restored on normal exit, Ctrl-C, SIGTERM and panic.

## Run with no arguments: automatic playback

Running `trix` without an effect name loops through all 35 effects:

```bash
trix                                   # cycle by default, 12 s per effect
trix --playlist random --switch 8      # random order, 8 s per effect
trix --playlist cycle --switch 20      # sequential order, 20 s per effect
trix --transition dissolve --switch 6  # cross-dissolve instead of a fade
```

Switches blend with a 0.6 s fade by default; `--transition dissolve` cross-dissolves cell by cell, and `--transition none` restores the instant cut with a white flash. Blends are capped at half of `--switch`. Press `n` / `p` to switch manually and reset the timer.

## Command line

```text
trix <effect> [options]

  -e, --effect <name>     effect to run; omit to cycle all
      --playlist <mode>   cycle | random                    [default: cycle]
      --switch <secs>     seconds per effect                [default: 12]
      --transition <mode> fade | dissolve | none            [default: fade]
      --transition-secs <s> blend length                    [default: 0.6]
  -f, --fps <1-240>       target frame rate                [default: 30]
  -s, --speed <0.05-8>    animation speed multiplier       [default: 1.0]
  -t, --theme <name>      color theme                      [effect default]
      --charset <mode>    kana | ascii | unicode           [default: ascii]
      --unicode           use Unicode glyphs
      --ascii             force ASCII glyphs
      --color <mode>      auto | truecolor | 256 | 16 | mono [default: auto]
      --no-color          alias for --color mono
      --density <f>       beams/fireworks/galaxy/traffic density [default: 1.0]
      --spin <f>          black hole rotation speed        [default: 0.5]
      --twinkle <f>       galaxy twinkle rate              [default: 1.0]
      --meteors <n>       black hole / galaxy meteors      [default: 3]
      --beam-speed <f>    beams horizontal speed           [default: 1.0]
      --beam-motion <m>   straight | jump                  [default: straight]
      --trail <f>         dsh: drop-ghost strength          [default: 1.0]
      --seed-count <n>    dandelion drifting seeds         [default: 90]
      --seed-speed <f>    dandelion seed drift speed       [default: 1.0]
      --street <dir>      horizontal | vertical            [default: horizontal]
      --tornado-count <n> tornado count                    [default: 3]
      --tornado-speed <f> tornado travel speed             [default: 1.0]
      --life-cycle <secs> life: seconds per 0-100 round    [default: 120]
      --eye-motion <m>    demoneye: dart | still           [default: dart]
      --ghost             monalisa: pure-ASCII ghost edition
      --gallery           monalisa: soft gallery glow
      --plain             dragon: plain starfield
      --rocket-scale <f>  launch: rocket size multiplier   [default: 1.0]
      --shake <f>         launch: camera shake strength    [default: 1.0]
      --jump              Tom & Jerry: enable jumping
      --no-jump           Tom & Jerry: disable jumping
      --seed <u64>        deterministic random seed
      --duration <secs>   exit automatically after N seconds
      --no-alt            do not use the alternate screen
      --no-hud            hide the top/bottom HUD
  -l, --list              catalog + themes
  -h, --help [effect|all] help, effect dossier, or full reference
  -V, --version           print version
      --completions <sh>  print bash / zsh / fish completion script

Capture / benchmark
      --snapshot <path>   render --frames steps to ANSI ('-' = stdout)
      --frames <n>        simulation steps
      --size <WxH>        offscreen canvas size, e.g. 120x36
      --bench             offscreen render benchmark
```

Run `trix --help all` for every flag, aliases and defaults.

### HUD

- **Top bar**: effect name and key on the left, live telemetry / speed / theme / charset on the right.
- **Bottom bar**: always-on key bar (`q quit`, `space pause`, `n/p next`, `t theme`, `h help`, `i info`).
- **Playlist row**: progress and the next switch countdown above the key bar.
- **`i` panel**: runtime state plus the CLI knobs the current effect actually uses.

`rain`, `fire` and `monalisa` are deliberately text-free ambient scenes: no top bar and no scene labels, but the bottom key bar and playlist row remain. `--no-hud` hides the top and bottom bars for recording; the `h` / `i` panels and pause box still work. `--snapshot` renders without the HUD.

See [docs/hud-and-keys.md](docs/hud-and-keys.md) for the full HUD contract.

### Runtime keys

| Key | Action |
| --- | --- |
| `q` / `Ctrl-C` | Quit |
| `Esc` | Close the help/info panel; quit when no panel is open |
| `Space` | Pause / resume |
| `n` / `p`, arrow keys | Next / previous effect |
| `1`–`9`, `0`, `[`, `]` | First 12 effects |
| `m` / `e` / `d` | Super Mario / Heartbeat Monitor / Dandelion |
| `f` / `l` / `b` / `j` | Aquarium / Landscape / Oriental Pearl / Tiananmen |
| `c` / `z` / `k` | Traffic Flow / Tornado / Thunderstorm |
| `o` / `v` / `g` / `y` | Mona Lisa / Ice Ballet / Demon Eye / Threads of Life |
| `u` / `w` | DeepSeek Harness / Vibe Coding |
| `t` | Cycle themes |
| `+` / `-` | Faster / slower (0.1×–8×) |
| `r` | Restart the current effect with a new seed |
| `i` | Info / parameters panel |
| `h` / `?` | Help panel |

Shortcuts are case-insensitive, so Shift and Caps Lock work. The parser accepts CSI and SS3 cursor keys, ignores modifiers on arrows, and treats Alt/Option chords and mouse/paste reports as non-shortcuts.

The seven newest scenes (`tunnel`, `helix`, `lights`, `volcano`, `tesseract`, `launch`, `plasma`) have **no single-key shortcut**; use `trix <effect>` or cycle with `n` / `p`.

## Themes

| Theme | Style | Default for |
| --- | --- | --- |
| `neon` | Neon cyan / violet | rain, beams, tomjerry, shanghai, traffic, tunnel, tesseract |
| `matrix` | Matrix green | matrix, ecg |
| `ember` | Realistic fire (red → orange → yellow → white) | fire, demoneye, volcano, launch |
| `aurora` | Emerald dragon / aurora gold | dragon, fish, landscape, ballet, life, helix, lights |
| `ice` | Holographic ice blue | map, solar, galaxy, tornado, storm, dsh |
| `plasma` | Magenta / high energy | blackhole, fireworks, plasma |
| `gold` | Imperial gold | buddha, mario, dandelion, tiananmen, monalisa, vibe |
| `chrome` | Silver-gray monochrome | manual selection |

`fire` derives its flame colors from five theme anchors; `--theme ice` gives a “cold flame”, and `--theme ember` is the default fire.

## Performance

Environment: Apple Silicon (ARM64), macOS, Rust 1.96.1, `--release` (LTO + `codegen-units=1` + `panic=abort` + `strip`). Canvas 160×48, 240-frame offscreen benchmark:

| Effect | Mean render | p95 | Diff output |
| --- | ---: | ---: | ---: |
| rain | 0.080 ms | 0.101 ms | 14.3 KiB/frame |
| fire | 0.454 ms | 0.548 ms | 68.3 KiB/frame |
| dragon | 0.112 ms | 0.134 ms | 8.9 KiB/frame |
| map | 0.161 ms | 0.189 ms | 43.6 KiB/frame |
| matrix | 0.071 ms | 0.087 ms | 12.0 KiB/frame |
| blackhole | 0.133 ms | 0.172 ms | 28.0 KiB/frame |
| beams | 0.085 ms | 0.113 ms | 24.9 KiB/frame |
| fireworks | 0.040 ms | 0.064 ms | 8.6 KiB/frame |
| solar | 0.039 ms | 0.045 ms | 7.6 KiB/frame |
| galaxy | 0.129 ms | 0.163 ms | 23.4 KiB/frame |
| buddha | 0.061 ms | 0.072 ms | 11.4 KiB/frame |
| tomjerry | 0.019 ms | 0.022 ms | 2.4 KiB/frame |
| mario | 0.013 ms | 0.016 ms | 0.7 KiB/frame |
| ecg | 0.038 ms | 0.046 ms | 7.3 KiB/frame |
| dandelion | 0.024 ms | 0.028 ms | 3.6 KiB/frame |
| fish | 0.023 ms | 0.045 ms | 2.1 KiB/frame |
| landscape | 0.038 ms | 0.043 ms | 1.7 KiB/frame |
| shanghai | 0.032 ms | 0.036 ms | 3.3 KiB/frame |
| tiananmen | 0.095 ms | 0.105 ms | 7.0 KiB/frame |
| traffic | 0.029 ms | 0.036 ms | 1.7 KiB/frame |
| tornado | 0.043 ms | 0.051 ms | 7.8 KiB/frame |
| storm | 0.109 ms | 0.122 ms | 29.9 KiB/frame |
| monalisa | 0.021 ms | 0.032 ms | 0.0 KiB/frame (static) |
| ballet | 0.120 ms | 0.151 ms | 14.5 KiB/frame |
| demoneye | 0.273 ms | 0.398 ms | 15.4 KiB/frame |
| life | 0.158 ms | 0.209 ms | 5.6 KiB/frame |

Real PTY, truecolor, 120×36, 30 FPS, 5 seconds (CPU = child `utime+stime / 5s`, including PTY writes but not the terminal's own drawing). The sample below covers the 22 effects measured in that run; `monalisa`, `ballet`, `demoneye`, `life`, `dsh`, `vibe`, `tunnel`, `helix`, `lights`, `volcano`, `tesseract`, `launch` and `plasma` were added later and are not in this table:

| Effect | CPU (one core) | RSS | 5 s output |
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

- At 30 FPS the frame budget is about 33.3 ms; the heaviest effect, `fire`, costs about **1.38% of one core** offscreen (0.454 ms/frame) and about **1.4%** interactively.
- Startup: `--version` median about **2.2 ms**; the release binary is about **1.1 MB** (1,104 KiB).
- Memory: peak RSS under a real PTY **2.08–2.27 MiB**.
- Interactive mode honors `NO_COLOR`; offscreen `--snapshot` / `--bench` default to truecolor and can be overridden with `--color`.

```bash
scripts/bench.sh
./target/release/trix fire --bench --size 160x48 --frames 300
```

## Compatibility and FAQ

- Runs on macOS, Linux and WSL (Unix termios); native Windows is not implemented.
- 24-bit truecolor is recommended; `--color auto` degrades automatically to 256 / 16 / mono.
- Dual-form effects always start in ASCII; add `--unicode` or `--charset unicode` for Unicode. `--charset kana` can mix half-width katakana into `matrix`.
- Linux binaries are static musl builds for `x86_64` and `aarch64`, so they run on any distribution (including Alpine) on those architectures.
- Non-TTY runs suggest `--snapshot`; `trix --snapshot -` writes ANSI to stdout.
- The canvas is black; a dark terminal is recommended.
- **Why is the command `trix` but the package `trix-effects`?** Short command, unique package name — the `ripgrep` / `rg` split. Installing the package (for example with `cargo install --git …`) installs the `trix` binary.
- **Why do screenshots show ASCII?** ASCII is the default and works in every terminal/font. Unicode previews are isolated in the ASCII / Unicode section.
- **How do I record a GIF?** Use `--snapshot` plus `tools/ansi2png.py`, or `tools/render_gif.py`; see [Recording and documentation tools](#recording-and-documentation-tools).
- **Shell completions?** `trix --completions bash` (also `zsh`, `fish`); install the output in your shell's completion directory.

## Implementation

- **Terminal driver** (`src/term.rs`): libc sets raw mode, the alternate screen, a hidden cursor and disabled autowrap; non-blocking key reads, `TIOCGWINSZ` size detection, SIGINT/SIGTERM/SIGHUP flags and a panic hook.
- **Canvas** (`src/canvas.rs`): `Cell { char, fg, bg }` double buffering; per-row changed spans; SGR is emitted for both foreground and background changes, and the background falls back to `40` (the black the program started with) so the previous scene's background never lingers; truecolor / 256 / 16 / mono (mono draws glyphs only). The `monalisa` block portrait uses background color; `mario` does not.
- **Zero framework**: `libc` is the only dependency; no tokio, crossterm or worker threads.
- **Help** (`src/help.rs`, `src/meta.rs`): effect metadata drives `--help`, `--help <effect>` (with a real scene preview), `--help all`, the styled `--list`, the `i` panel and the interactive `h` panel; the layout is width-aware and degrades to ASCII / mono.
- **Code rain**: per-column heads and tails, random speed and depth, binary mist, horizontal data packets, CRT scanline.
- **Fire**: Doom-fire cellular propagation; the bottom merges into one wide fire bed, the middle flames are tallest, and the side tongues lean outward.
- **Dragon**: the flight path is resampled by arc length and the long serpent body is splatted into intensity / depth buffers; a large Chinese dragon head, antlers, whiskers, mane, four claws, dorsal fin, tail fin and scales; full-screen Lissajous flight.
- **World map**: Natural Earth land mask, coverage sampling, city nodes, great-circle arcs, scan beams.
- **The Matrix**: a NASA backend login wall. ASCII by default; `--unicode` switches to a blue NASA banner (white wordmark, red swoosh, white star points, blue-white light sweep) and a centered, lowered Unicode-framed login box; OPERATOR/PASSWORD/2FA, AUTH/DECRYPT progress, logs, radar, ACCESS GRANTED.
- **Black hole**: event horizon, photon ring, accretion disk, lensing arcs, jets, meteors, infall; outer stardust follows a full "/"-tilted ellipse that fills the screen; the outermost ring uses random low-density `.` / `:` / `+` scatter, with a middle stardust band between the inner and outer layers; all layers rotate in the same direction with differential speed (fast inside, slow outside, while the outer ring and accretion band speed up together), reading as one rotating elliptical disk; overall orbital speed ×1.7; `--spin` adjusts the whole rotation.
- **Beams**: one-way, left to right; straight by default, `--beam-motion jump` makes them rise and fall; `--density` / `--beam-speed` adjust density and speed.
- **Fireworks**: rockets + inner/outer particle rings + delayed bursts; particle trails, gravity, a city silhouette, a starfield.
- **Solar system**: the Sun in the top-left; the eight planets on orbital lines below; sunlight spreads in layered fans toward the lower right and stops above the planet line.
- **Galaxy**: precomputed stars along a diagonal Milky Way band plus a central bulge; animated cluster arcs, dark dust, meteors, constellation lines; `--density` / `--twinkle` / `--meteors` are adjustable.
- **Buddha's halo**: reverted to the previous edition; the three static mandala particle rings now expand outward dynamically; head halo, lotus base and particles are kept.
- **Tom & Jerry**: a space-station corridor; frame animation; slow and non-jumping by default; `--speed` / `+` / `-` adjust speed, `--jump` enables jumping; the HUD shows the multiplier.
- **Super Mario**: automatic running and jumping; question blocks pop out mushrooms, coins and score; scrolling sky / hills / ground / Goombas; ASCII classic glyphs by default, `--unicode` switches to solid sprites.
- **Heartbeat Monitor**: ECG grid, P-QRS-T waveform, scan line, HR/SpO2/SYS-DIA; ASCII waveform by default, `--unicode` switches to Unicode box drawing / solid waveform.
- **Dandelion**: 3 clumps × 2 stems / 6 flower heads; left and right roots bend toward the center; flower heads ×1.5.
- **Oriental Pearl**: Bund night view + Oriental Pearl + Huangpu River reflections; two random boats on the river — hull type (junk / barge / ferry), size, hull palette, channel depth and drift direction are all random; distant boats sit low on the water while near boats are large and press against the frame, with decks / portholes / lit cabin windows / smoking funnels / mast lights / bow lanterns / wakes and water reflections; 1–2 meteors cross the night sky every 3–5 seconds (about 18% of intervals are skipped), with a white-hot head and an ice-blue or warm-gold fading tail, flying into the skyline or fading out and being recycled.
- **DeepSeek Harness**: a two-line lockup (`DEEPSEEK` / indented `HARNESS`) is laid out from 6×9 / 8×11 display faces with horizontal pixel doubling, so the wide letterforms fill most of the screen. The intro is a per-letter reveal with a top-to-bottom wipe and a quantised one-row drop that leaves a hashed ghost stack below HARNESS. The wordmark then stays on screen forever; a seven-second beam state machine floats a horizontal beam down the lockup, joins it with a vertical beam at the centre to form a bright cross, then sweeps the vertical beam left to right and fades the horizontal one. No flare or particle burst. `--trail` scales the ghosts.
- **Rocket Launch**: a procedurally scaled two-stage stack (fairing, core, twin boosters and engine bells) is drawn from a 7/9/11-column glyph grid; the camera follows the ascent and shakes on ignition and separation. Exhaust is a layered ASCII plume with a white-hot spine and shock diamonds, backed by world-space smoke, steam and sparks. The blue-hour sky gradient, parallax star layers, cloud deck, MECO flash, tumbling core and boosters, Earth-limb atmosphere and orbital sunrise are drawn around it. `--rocket-scale` and `--shake` tune the framing.
- **Vibe Coding**: reads `src/scenes/vibe.rs` (falling back to a compile-time copy), normalises it to ASCII, skips the test module and builds six 10-line windows per turn. A hand-written Rust tokenizer colours comments, strings, numbers, keywords, types, macros and function names; the transcript is a typed queue that only reveals lines up to the typing head (the area below stays blank) and clears before the next loop, with a Claude Code-style spinner, token count and composer.
- **Offline map data**: `tools/gen_land.py` downloads Natural Earth and rasterizes it into `src/data/land_720x360.bin`; runtime never touches the network.

## Recording and documentation tools

```bash
trix dragon --snapshot dragon.ans --size 120x36 --frames 145 --seed 7
python3 tools/ansi2png.py dragon.ans dragon.png --size 120x36 --label "trix dragon"
python3 tools/render_gif.py dragon docs/dragon.gif --start 100 --frames 24 --step 3
```

Documentation assets: `docs/help.png`, `docs/help-rain.png` and `docs/help-full.png` are rendered from `trix --help`, `trix --help rain` and `trix --help all` with `TRIX_HELP_TTY=1` and `--color true`, then converted with `tools/ansi2png.py` (`--size 80x40` for `help-rain`). Always force `--color true`: under a `NO_COLOR` shell the styled help correctly falls back to mono, which would bake a white-only screenshot. `docs/social-preview.png` is the 1280×640 GitHub social preview.

## Testing

```bash
cargo test
cargo clippy --all-targets
cargo fmt --check
```

There are currently **224** unit / integration tests. See `.github/workflows/ci.yml` for the Linux/macOS CI matrix.

## Versions and rollback

- **Current unreleased version: 0.11.0** (`trix --version`); no tag or GitHub Release yet.
- **0.11.0** adds seven generative scenes: `tunnel`, `helix`, `lights`, `volcano`, `tesseract`, `launch` and `plasma`.
- **0.10.0** adds `dsh` (DeepSeek Harness) and `vibe` (Vibe Coding), plus the `--trail` knob.
- **0.9.1** renames the package/repository to `trix-effects` (command stays `trix`), hardens `install.sh`, adds cache-safe ASCII previews and contributor/launch docs.
- **0.9.0** was the short-lived `trix-tui` name; it added the styled help system, removed the `fairy` entry (source retained), and made `life` part of the 26-effect set.
- Previous: tag **`v0.9.0`** (`trix-tui`); **`v0.8.0`** (Mona Lisa / Ice Ballet / Demon Eye; 26 effects including `fairy`); **`v0.7.0`** (aquarium / landscape / Oriental Pearl / Tiananmen / traffic / fairy / tornado / thunderstorm, playlists); **`v0.6.0`** (Mario / ECG / dandelion, blue-green code rain); **`v0.5.1`** (automated release pipeline).
- Rollback:
  ```bash
  git checkout v0.9.0     # inspect the previous version only (detached HEAD)
  git switch master
  git reset --hard v0.9.0 # fully revert to the previous version
  ```
- Extra backups (not in git): `backups/trix-v0.1.0-*` through `trix-v0.7.0-*`, containing source tarballs and aarch64 macOS binaries.
- Change log: [English](CHANGELOG.md) · [中文](CHANGELOG.zh-CN.md).

### Releases

- Pushing a `v*` tag builds the binaries and creates a GitHub Release.
- **macOS**: `aarch64-apple-darwin` + `x86_64-apple-darwin`, merged with `lipo` into a universal binary.
- **Linux**: static musl builds for `x86_64` and `aarch64` (the ARM64 build is cross-compiled and smoke-tested under QEMU).
- Assets: `trix-effects-<version>-macos-universal.tar.gz`, `trix-effects-<version>-linux-x86_64.tar.gz`, `trix-effects-<version>-linux-aarch64.tar.gz`, plus `SHA256SUMS`.
- `Run workflow` on the Actions page builds artifacts without creating a Release.

## Contributing

Issues and pull requests are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for the development setup and the checks to run, and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for the community rules. Launch copy and channel drafts live in [docs/LAUNCH.md](docs/LAUNCH.md).

## Data sources, references and license

- Land outlines: [Natural Earth](https://www.naturalearthdata.com/) 1:110m land, **public domain**, converted from GeoJSON via [natural-earth-vector](https://github.com/nvkelso/natural-earth-vector).
- Doom-fire inspiration: [pserey/doom-fire-algorithm](https://github.com/pserey/doom-fire-algorithm), [horizonwiki/fire](https://github.com/horizonwiki/fire); this project's Rust implementation, multiple fuel stacks, palette and rendering are independently written.
- The Buddha ASCII art is adapted from [ascii.co.uk/art/buddha](https://ascii.co.uk/art/buddha) (original author credited as `hjw`) for the `buddha` scene; this project adds color, halo, lotus base, particles and animation.
- This project's code: [MIT](LICENSE).
