# trix

**English** | [简体中文](README.zh-CN.md)

> Formerly **texiao** (the original project name); renamed to **trix** starting with v0.5.0.

> A sci-fi ASCII / Unicode effects engine for the terminal — code rain, fire, a flying dragon, a world map, a NASA login, a black hole, light beams, fireworks, the solar system, a galaxy, Buddha's halo, Tom & Jerry, Super Mario, a heartbeat, a dandelion, an aquarium, a landscape, the Oriental Pearl, Tiananmen, traffic, a petal-scattering fairy, tornadoes, a thunderstorm, Mona Lisa, ice ballet, a demon eye, and the threads of life.
> Single-threaded Rust, no async runtime, diff rendering. Starts in about **2 ms**, resident memory around **2.1–2.3 MB**, CPU around **0.2–1.4%** of one core at 30 FPS. Current release **v0.8.0**; the development build adds “Threads of Life”, for **27** effects total. Effects that support both glyph sets start in ASCII.

![release](https://github.com/Doswind/trix/actions/workflows/release.yml/badge.svg)

![trix overview](docs/overview.png)

> GIFs / previews: [Code Rain](docs/rain.gif) · [Blazing Fire](docs/fire.gif) · [Coiling Dragon](docs/dragon.gif) · [NASA Login](docs/matrix.gif) · [Black Hole](docs/blackhole.gif) · [Light Beams](docs/beams.gif) · [Fireworks](docs/fireworks.gif) · [Solar System](docs/solar.gif) · [Starry Galaxy](docs/galaxy.gif) · [Buddha's Halo](docs/buddha.gif) · [Tom & Jerry](docs/tomjerry.gif) · [Super Mario](docs/mario.gif) · [Heartbeat](docs/ecg.gif) · [Dandelion](docs/dandelion.gif) · [Aquarium](docs/fish.gif) · [Landscape](docs/landscape.gif) · [Oriental Pearl](docs/shanghai.gif) · [Tiananmen](docs/tiananmen.gif) · [Traffic Flow](docs/traffic.gif) · [Fairy](docs/fairy.gif) · [Tornado](docs/tornado.gif) · [Thunderstorm](docs/storm.gif) · [Mona Lisa](docs/monalisa.png) · [Ghost edition](docs/monalisa-ghost.gif) · [Ice Ballet](docs/ballet.gif) · [Demon Eye](docs/demoneye.gif) · [Threads of Life](docs/life.gif)

## Effects

Every effect name below is given in English and Chinese.

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
| `fairy` | Fairy Scattering Petals | 仙女散花 |
| `tornado` | Tornado | 龙卷风 |
| `storm` | Thunderstorm | 电闪雷鸣 |
| `monalisa` | Mona Lisa | 蒙娜丽莎 |
| `ballet` | Ice Ballet | 冰上芭蕾 |
| `demoneye` | Demon Eye | 恶魔之眼 |
| `life` | Threads of Life | 生命的轨迹 |

| Code Rain (代码雨) `rain` | Blazing Fire (燃烧火焰) `fire` | Coiling Dragon (巨龙盘旋) `dragon` |
| --- | --- | --- |
| ![rain](docs/rain.png) | ![fire](docs/fire.png) | ![dragon](docs/dragon.png) |

| World Map (世界地图) `map` | The Matrix (黑客帝国) `matrix` | Black Hole (黑洞) `blackhole` |
| --- | --- | --- |
| ![map](docs/map.png) | ![matrix](docs/matrix.png) | ![blackhole](docs/blackhole.png) |

| Light Beams (光束) `beams` | Fireworks (烟花) `fireworks` | Solar System (太阳系) `solar` |
| --- | --- | --- |
| ![beams](docs/beams.png) | ![fireworks](docs/fireworks.png) | ![solar](docs/solar.png) |

| Starry Galaxy (满天星河) `galaxy` | Buddha's Halo (佛光普照) `buddha` | Tom & Jerry (猫和老鼠) `tomjerry` |
| --- | --- | --- |
| ![galaxy](docs/galaxy.png) | ![buddha](docs/buddha.png) | ![tomjerry](docs/tomjerry.png) |

| Super Mario (超级马里奥) `mario` | Heartbeat Monitor (心率跳动) `ecg` | Dandelion (蒲公英) `dandelion` |
| --- | --- | --- |
| ![mario](docs/mario.png) | ![ecg](docs/ecg.png) | ![dandelion](docs/dandelion.png) |

| Aquarium (水族馆) `fish` | Colorful Landscape (五光十色) `landscape` | Oriental Pearl (东方明珠) `shanghai` |
| --- | --- | --- |
| ![fish](docs/fish.png) | ![landscape](docs/landscape.png) | ![shanghai](docs/shanghai.png) |

| Tiananmen Radiance (光芒万丈) `tiananmen` | Traffic Flow (车水马龙) `traffic` | Fairy Scattering Petals (仙女散花) `fairy` |
| --- | --- | --- |
| ![tiananmen](docs/tiananmen.png) | ![traffic](docs/traffic.png) | ![fairy](docs/fairy.png) |

| Tornado (龙卷风) `tornado` | Thunderstorm (电闪雷鸣) `storm` | Mona Lisa (蒙娜丽莎) `monalisa` |
| --- | --- | --- |
| ![tornado](docs/tornado.png) | ![storm](docs/storm.png) | ![monalisa](docs/monalisa.png) |

| Ice Ballet (冰上芭蕾) `ballet` | Demon Eye (恶魔之眼) `demoneye` | Threads of Life (生命的轨迹) `life` |
| --- | --- | --- |
| ![ballet](docs/ballet.png) | ![demoneye](docs/demoneye.png) | ![life](docs/life.png) |

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
| Super Mario (超级马里奥) `mario` | **A pure-glyph 1-1 level**: clouds, hills and bushes, a brick floor with mortar seams, a **question-block / brick / used-block** row, green pipes, spinning coins, Goombas and mushrooms; auto running and jumping, blocks that pop out coins or mushrooms, stomped enemies, `+200/+100/+1000` score popups. **Uses no terminal background color at all**: ASCII colored glyphs by default (`# o = *`), `--unicode` uses `█ ▀ ▄` solid blocks; hills, clouds, pipes and bricks are each colored per object |
| Heartbeat Monitor (心率跳动) `ecg` | A green ECG monitor; **about 56 BPM with random beat intervals** and clearly random amplitude, QRS width and flat-segment length; **ASCII by default**, `--unicode` to switch |
| Dandelion (蒲公英) `dandelion` | 3 clumps × 2 stems, 6 flower heads; the roots and lower stems of the left and right clumps bend toward the center; **flower heads are 1.5× larger**, everything else unchanged |
| Aquarium (水族馆) `fish` | A full-screen aquarium: **5 hand-drawn sprite fish** (small 20×5 / medium 30×7 / large 38×8, forked tails, bright eyes and mouths; only the tails swish, the head and mouth stay fixed), plus 8–30 small `<==o>` / `<=-o>` fish (about 35% double-layer, 15% triple-layer bold, one eye only); bubbles use `o` / `O` / `8` with **a random shade per bubble**, each rising **slowly in a straight line** along a fixed column and swaying no more than 3 times over the whole climb; the count settles at 10–36; plants get random heights and spacing and grow in small patches; ASCII by default, `--unicode` switches to `█ ▓ ▒` |
| Colorful Landscape (五光十色) `landscape` | Layered distant mountains, a calm lake, a boat, white geese, jungle on both sides; animated water, mist and petals |
| Oriental Pearl (东方明珠) `shanghai` | The Shanghai Bund at night, the Oriental Pearl Tower, neon high-rises, Huangpu River reflections; **two random boats on the river, each with a different size, shape and position** (sailboat / freighter / ferry, with deck, portholes, lit cabin windows, smoking funnel, mast lights, wake and water reflection); **1–2 meteors cross the night sky every 3–5 seconds** (white-hot head plus fading tail); **ASCII by default**, `--unicode` available |
| Tiananmen Radiance (光芒万丈) `tiananmen` | Tiananmen gate enlarged to about **92% of the width and 62% of the height**: double golden-tile eaves with upturned corners, a red-pillared gate tower with the national emblem, red walls, five archways with white stone surrounds, white-framed slogans and the central portrait, red flags and side buildings; golden beams first grow outward and upward from behind the roof, then light up in order from the inside out, cycling |
| Traffic Flow (车水马龙) `traffic` | City street with high-rises on both sides, neon and traffic; horizontal traffic is 3 rows of cars (roof / body / wheels, 8–16 cells long) across 4–5 lanes with dashed dividers and alternating directions; vertical traffic is 6 columns of top-down cars (5–9 rows, front/rear lights and side wheels); `--street horizontal\|vertical`; `--density` controls traffic density; **ASCII by default**, `--unicode` available |
| Fairy Scattering Petals (仙女散花) `fairy` | A 30×34 hand-pixelled lady (low bun, profile brows and eyes, raised basket, folded arm, crossed blue collar, white skirt hem); a slow elliptical orbit plus a 6-second turn every 24 seconds (`scaleX`, no somersault); petals are thrown from the basket, follow her, then scatter and fall; `--petal-count` / `--petal-speed` |
| Tornado (龙卷风) `tornado` | 1–8 tornadoes move from left to right; twisted, pointed at the base and wider at the top; overall **one step shorter** than before (0.28–0.58 of the screen height) and **lighter and sparser** (brightness 0.42–0.85, 20 particles per row); 30 debris particles **drift outward** at the base |
| Thunderstorm (电闪雷鸣) `storm` | Dark clouds, slanting rain, a city silhouette, lightning; a layer of **gray-white, irregular high cloud** sits above the screen (denser, fading out at the bottom, **drifting slowly to the right** at about 3 columns per second) to add depth against the existing cloud band; lightning lights the whole screen white ([still](docs/storm-flash.png)) |
| Mona Lisa (蒙娜丽莎) `monalisa` | A background-free portrait: the figure (hair, face, neck and shoulders, folded hands) is baked from a reference pixel painting; ANSI color blocks carry skin tones and shading, with ASCII glyphs for detail; no landscape, frame or floating dust; the figure is about 12% wider, closer to the original proportions; **the default block version is static and does not blink**; `--unicode` uses half blocks for more vertical resolution; `--ghost` switches to the original pure-ASCII “ghost” version (no background blocks, blinking kept, [still](docs/monalisa-ghost.png) / [GIF](docs/monalisa-ghost.gif)); optional `--gallery` adds a faint warm gallery halo and floor glow behind the portrait ([preview](docs/monalisa-gallery.png)) |
| Ice Ballet (冰上芭蕾) `ballet` | An ice-ballet duet: a 39-second choreography loops through 13 sections (glide, crossover, arabesque, promenade, **overhead lift**, pair spin, death spiral, sissonne, Ina Bauer, final pose); joints use two-bone IK, clasped hands align automatically, limbs have soft edges and lag behind the torso; the routine covers most of the screen, three rows of spectators behind the boards sway, raise their hands and fire flashbulbs, and skate marks and reflections stay on the ice; `--speed` |
| Demon Eye (恶魔之眼) `demoneye` | One huge bloodshot eye fills the screen: it blinks every 8–15 seconds, singly or doubly (about 7:3); between blinks the eyeball darts only 1–2 times (darts at least 5 seconds apart), reaching the target in about 0.1 s and always returning to center within 0.45–0.85 s; the vertical pupil scales, eyelids press in, blood vessels and a blink shockwave; `--eye-motion dart\|still` (default `dart`), `--eye-still` pins the eyeball dead center |
| Threads of Life (生命的轨迹) `life` | A 0–100 age timeline with glowing life lines on both sides; every human uses `@` with a colored uppercase abbreviation on the same row and **the relative's own age** (friends who are not family carry no age); the bright beam grows with age (late in life it can cover most of the lifetime), fading from dark dots / dashes at the tail up to `=` at the head, with a `****@` comet tail; the first ~22%–42% of life (at most about 22 years) stays as sparse dim dots; when a track ends its row can be reused by someone later (school-age `PAL/UNI` +1 row → GF/WIFE, `DOG` −1 row → KID2, and the cat / parents' rows are reclaimed by later friends); `DOG` stays at −1 next to me for life; when `PAL/UNI/WORK/FRIEND` coexist they **share one row**: during the school years it sits on the other side at +1, and while a minor KID exists the whole group retreats to the nearest free row outside KID/CAT (usually +4; only rows actually used by children/pets are yielded, no longer fixed −5/−7), and all KIDs move back inside once grown; `ME 0 / WIFE +1`; `KID1 +2`, `KID2 −1` (reusing the row next to me that DOG gave up), `KID3 −2`; before adulthood `KID3 −2 / KID2 −1 / ME 0 / WIFE +1 / KID1 +2` is **contiguous with no empty rows**, and `GRAND1/GRAND2` always hug their parent's row (moving with them as they drift); `CAT +3` is born with the first child, just below KID1; `DAD/MOM +3/+4` start yielding to `+5/+6` two years before the child is born, and `BRO +5→+7` follows; GPA/GMA/DAD/MOM get suitable random starting ages when I am 0 (58–70 / 56–68 / 25–33 / 23–31), BRO 0–4, ME 0; KID/GRAND/CAT/DOG count from their own birth, GF/WIFE are within ±2 years of my age; every row is fixed at one terminal row (it does not scale with terminal height, and tall terminals never produce empty rows from rounding); when `ME` ends, a roughly 6-second **supernova finale** plays (the flashback and stardust slow down while the comet keeps its speed: white core, life flashback, shockwave, stardust, brightened comet and a permanent halo at the top; the flashback beam that reaches ME's stop line stays three columns wide and never disappears); the death point and the top star keep `@` + name + final age and appear/disappear in sync once per second; all the dead rise to the top in death order, one row apart, and their end labels fade in 0.5 s after the meteor arrives; humans blink as `*`, cats and dogs as `+`; GF approaches for 2 years, then stays 1–2 years before becoming WIFE; WIFE lives to 75–90 and dies after I become a star; AGE has no leading zeros; `--life-cycle` sets the length of one round |

> Screenshots are mostly 120×36, truecolor, offscreen renders; `monalisa` uses a 120×60 portrait canvas. Some screenshots / GIFs show Unicode glyphs; at runtime every dual-glyph effect starts in ASCII (add `--unicode` to switch).

Light Beams has two motion modes:

| Straight (default) | Jumping up and down |
| --- | --- |
| ![beams straight](docs/beams.png) | ![beams jump](docs/beams-jump.png) |

### ASCII / Unicode editions

**Every effect that supports both glyph sets starts in ASCII**: `matrix`, `mario`, `ecg`, `dandelion`, `fish`, `landscape`, `tiananmen`, `fairy`, `tornado`, `storm`, `monalisa`, `ballet`, `demoneye`, `life`, `shanghai`, `traffic`. Add `--unicode` (or `--charset unicode`) to opt in; `--ascii` forces ASCII again. `monalisa`'s default block portrait is static; `monalisa --ghost` is a separate pure-ASCII drawing (blinking kept) that emits neither Unicode nor background blocks.

`mario` / `ecg` in both glyph sets:

| ASCII (default) | Unicode (`--unicode`) |
| --- | --- |
| ![mario ascii](docs/mario-ascii.png) | ![mario unicode](docs/mario.png) |
| ![ecg ascii](docs/ecg-ascii.png) | ![ecg unicode](docs/ecg.png) |

`shanghai` / `traffic` in both glyph sets:

| ASCII (default) | Unicode (`--unicode`) |
| --- | --- |
| ![shanghai ascii](docs/shanghai.png) | ![shanghai unicode](docs/shanghai-unicode.png) |
| ![traffic ascii](docs/traffic.png) | ![traffic unicode](docs/traffic-unicode.png) |

Vertical street:

![traffic vertical](docs/traffic-vertical.png)

Full-screen lightning flash:

![storm flash](docs/storm-flash.png)

```bash
trix mario              # ASCII classic glyphs by default
trix mario --unicode    # solid Unicode sprites
trix ecg                # ASCII ECG monitor by default
trix ecg --unicode      # Unicode ECG monitor
```

## Features

- **27 switchable effects**: command-line arguments, runtime hot-switching and themes.
- **Smooth hand-off**: every effect change can fade through black or cross-dissolve (`--transition fade|dissolve|none`, 0.6 s by default); the outgoing scene keeps animating while it fades.
- **ASCII first**: effects that support both ASCII and Unicode start in ASCII; opt into Unicode explicitly with `--unicode`.
- **Low resource use**: single-threaded, no async, no event-loop framework; 30 FPS by default; `libc` is the only dependency.
- **Fast startup**: `--version` median about **2.0 ms**; the release binary is about **733 KB**.
- **Low memory**: interactive RSS about **2.08–2.27 MB** (120×36, truecolor).
- **Diff rendering**: double-buffered canvas that repaints only changed cells and merges SGR sequences, cutting terminal writes and CPU.
- **Color animation**: 24-bit truecolor; 8 themes; degrades to 256 colors, 16 colors or monochrome.
- **Full-screen detail**: every scene has stars, particles, grids, glows or a city / corridor backdrop, so large empty areas never appear.
- **Sci-fi / Chinese style**: NASA wall, black-hole accretion, particle fireworks, a holographic solar system, a galaxy, Buddha's halo, a Chinese dragon.
- **Recordable / benchmarkable**: built-in `--snapshot` and `--bench`.
- **Clean exit**: raw mode, the alternate screen, cursor and autowrap are restored on normal exit, Ctrl-C, SIGTERM and panic.

## Quick start

Requires Rust 1.70+ (stable recommended).

```bash
cargo build --release
./target/release/trix rain
```

Install onto `PATH`:

```bash
cargo install --path . --locked
trix dragon
```

### One command per effect

```bash
trix rain                          # Code Rain
trix fire --fps 60                 # more, slimmer fire stacks
trix dragon --theme gold           # golden dragon
trix dragon --plain                # old starfield-only backdrop
trix map --theme ice               # holographic world map
trix matrix                        # NASA login wall (ASCII; --unicode for the full wall)
trix blackhole                     # a black hole devouring a star
trix beams                         # horizontal beams in random bright colors
trix fireworks                     # double-layer firework bursts
trix solar                         # the eight planets in a line
trix galaxy                        # starry galaxy / Milky Way
trix buddha --theme gold           # Buddha's halo
trix tomjerry --speed 0.8          # around 0.6x by default, adjustable
trix mario                         # Super Mario (ASCII by default; --unicode for solid sprites)
trix ecg                           # Heartbeat Monitor (ASCII by default; --unicode for solid glyphs)
trix dandelion                     # drifting dandelion seeds
trix fish                          # Aquarium
trix landscape                     # colorful landscape
trix shanghai                      # Oriental Pearl (ASCII)
trix tiananmen                     # Tiananmen Radiance
trix traffic --street vertical     # vertical Traffic Flow
trix fairy                         # Fairy Scattering Petals
trix tornado                       # Tornado
trix storm                         # Thunderstorm
trix monalisa                      # Mona Lisa portrait (blocks + glyphs, static)
trix ballet --speed 1.3            # Ice Ballet (duet, 39-second choreography loop)
trix demoneye                      # Demon Eye (random gaze / blinks)
trix demoneye --eye-still          # pin the eyeball to the center, keep blinks and pupil motion
trix life                          # Threads of Life (120 s per round by default)
trix life --life-cycle 60          # faster: 0–100 in one minute
trix tomjerry --jump               # enable jumping
trix --list                        # list effects and themes

# Option examples
trix matrix --unicode                       # Unicode wall
trix blackhole --spin 0.25 --meteors 8      # slower rotation + more meteors
trix beams --density 1.5 --beam-speed 1.2   # denser and faster
trix beams --beam-motion jump              # rise and fall
trix fireworks --density 2                  # more fireworks
trix galaxy --density 1.5 --twinkle 2 --meteors 8
trix tomjerry --chase-speed 0.8 --jump      # chase speed / jumping
trix mario --unicode                        # Super Mario Unicode edition
trix ecg --unicode                          # ECG Unicode edition
trix dandelion --seed-count 180 --seed-speed 1.4
trix shanghai --unicode                       # Oriental Pearl Unicode
trix traffic --street vertical --unicode      # vertical Unicode
trix fairy --petal-count 160 --petal-speed 1.5
trix tornado --tornado-count 5 --tornado-speed 1.6
trix monalisa --unicode                      # half-block Unicode edition (finer detail)
trix monalisa --ghost                        # pure-ASCII ghost edition (still blinks)
trix monalisa --gallery                      # optional gallery halo behind the portrait
trix ballet --speed 1.3                      # faster tempo
trix demoneye                                # Demon Eye
```

## Run with no arguments: automatic playback

Running `trix` without an effect name **loops through all 27 effects in the current terminal**:

```bash
trix                                   # cycle by default, 12 s per effect
trix --playlist random --switch 8      # random order, 8 s per effect
trix --playlist cycle --switch 20      # sequential order, 20 s per effect
trix --transition dissolve --switch 6  # cross-dissolve instead of a fade
```

Switches blend with a 0.6 s fade (fade to black, then fade the next effect in) by default; `--transition dissolve` cross-dissolves the two scenes cell by cell, and `--transition none` restores the old instant cut with a white flash. The blend is capped at half of `--switch`, so short slots still show each effect. Press `n` / `p` to switch manually and reset the timer. Naming an effect (for example `trix fire`) disables automatic playback.

## Command line

```text
trix <effect> [options]

  -e, --effect <name>     same as the positional argument; cycles when omitted
      --playlist <mode>   cycle | random                      [default: cycle]
      --switch <secs>     seconds per effect                  [default: 12]
      --transition <mode> fade | dissolve | none              [default: fade]
      --transition-secs <s> blend length in seconds           [default: 0.6]
      --no-fade           alias for --transition none
  -f, --fps <1-240>       target frame rate                  [default: 30]
  -s, --speed <0.05-8>    animation speed multiplier         [default: 1.0]
      --chase-speed <f>   alias for --speed (Tom & Jerry)
  -t, --theme <name>      color theme                        [effect default]
      --charset <mode>    kana | ascii | unicode                [default: ascii]
      --unicode           Opt in to Unicode single-cell glyphs
      --ascii             Force ASCII glyphs (the default)
      --color <mode>      auto | truecolor | 256 | 16 | mono [default: auto]
      --density <f>       beams/fireworks/galaxy/traffic density  [default: 1.0]
      --spin <f>          black hole rotation speed          [default: 0.5]
      --twinkle <f>       galaxy twinkle rate                [default: 1.0]
      --meteors <n>       black hole / galaxy meteors        [default: 3]
      --beam-speed <f>    beams horizontal speed             [default: 1.0]
      --beam-motion <m>   straight | jump                    [default: straight]
      --beam-jump         alias for --beam-motion jump
      --life-cycle <secs> life: seconds per 0-100 round      [default: 120]
      --eye-motion <m>    demoneye: dart | still              [default: dart]
      --eye-still         alias for --eye-motion still
      --ghost             monalisa: pure-ASCII ghost edition (no background blocks)
      --gallery           monalisa: soft warm gallery glow (optional)
      --plain             dragon: plain starfield (no mountains/cloud sea)
      --seed-count <n>    dandelion seed count                [default: 90]
      --seed-speed <f>    dandelion seed drift speed          [default: 1.0]
      --street <dir>      horizontal | vertical               [default: horizontal]
      --petal-count <n>   fairy petals                        [default: 80]
      --petal-speed <f>   petal fall speed                    [default: 1.0]
      --tornado-count <n> number of tornadoes                 [default: 3]
      --tornado-speed <f> tornado travel speed                [default: 1.0]
      --jump              Tom & Jerry jumps                  [default: off]
      --seed <u64>        fixed random seed (reproducible)
      --duration <secs>   exit automatically after N seconds
      --no-alt            do not use the alternate screen (keep scrollback)
  -l, --list              list effects / themes
  -h, --help              help
  -V, --version           version

Recording / benchmarking
      --snapshot <path>   simulate --frames steps and write one ANSI frame to a file ('-' = stdout)
      --frames <n>        number of simulation steps (not output frames)
      --size <WxH>        offscreen canvas size, e.g. 120x36
      --bench             offscreen render benchmark (including diff ANSI encoding)
```

### Runtime keys

| Key | Action |
| --- | --- |
| `q` / `Esc` | Quit (`Esc` closes the help panel first when it is open) |
| `Space` | Pause / resume |
| `n` / `p`, arrow keys | Next / previous effect |
| `1`–`9`, `0`, `[`, `]` | First 12 effects |
| `m` / `e` / `d` | Super Mario / Heartbeat Monitor / Dandelion |
| `f` / `l` / `b` / `j` | Aquarium / Landscape / Oriental Pearl / Tiananmen |
| `c` / `x` / `z` / `k` | Traffic Flow / Fairy Scattering Petals / Tornado / Thunderstorm |
| `o` / `v` / `g` | Mona Lisa / Ice Ballet / Demon Eye |
| `y` | Threads of Life |
| `t` | Cycle themes |
| `+` / `-` | Faster / slower (0.1×–8×); Tom & Jerry included |
| `r` | Restart the current effect with a new seed |
| `i` | Show / hide the status line (FPS, elapsed, theme, size, seed) |
| `h` / `?` | Help panel |

Number keys: `1 rain`, `2 fire`, `3 dragon`, `4 map`, `5 matrix`, `6 blackhole`, `7 beams`, `8 fireworks`, `9 solar`, `0 galaxy`, `[ buddha`, `] tomjerry`; letter keys: `m mario`, `e ecg`, `d dandelion`, `f fish`, `l landscape`, `b shanghai`, `j tiananmen`, `c traffic`, `x fairy`, `z tornado`, `k storm`, `o monalisa`, `v ballet`, `g demoneye`, `y life`.

## Themes

| Theme | Style | Default for |
| --- | --- | --- |
| `neon` | Neon cyan / purple | rain, beams, tomjerry |
| `matrix` | Matrix green | matrix |
| `ember` | Realistic fire (red → orange → yellow → white) | fire |
| `aurora` | Emerald dragon / aurora gold | dragon |
| `ice` | Holographic ice blue | map, solar, galaxy |
| `plasma` | Magenta / high energy | blackhole, fireworks |
| `gold` | Imperial gold | buddha |
| `chrome` | Silver-gray monochrome | any |

`fire` derives its flame colors from five anchors in the theme palette; `--theme ice` gives a “cold flame”, and `--theme ember` is the default fire.

## Automated builds and releases

The repository contains [`.github/workflows/release.yml`](.github/workflows/release.yml):

- Pushing a `v*` tag builds the binaries and creates a GitHub Release.
- **macOS**: `aarch64-apple-darwin` + `x86_64-apple-darwin`, merged with `lipo` into a **universal** binary.
- **Linux x86_64**: static `x86_64-unknown-linux-musl`, independent of the target machine's glibc.
- **Linux ARM64**: static `aarch64-unknown-linux-musl`, cross-compiled with `cross` and smoke-tested under QEMU, independent of glibc; runs natively on ARM64 servers, 64-bit Raspberry Pi 4/5, AWS Graviton and similar machines.
- Release assets:
  - `trix-<version>-macos-universal.tar.gz`
  - `trix-<version>-linux-x86_64.tar.gz`
  - `trix-<version>-linux-aarch64.tar.gz`
  - `SHA256SUMS`
- You can also run `Run workflow` manually on the GitHub Actions page to build artifacts only (no Release is created).

Release a new version:

```bash
git tag -a v0.8.0 -m "trix v0.8.0"
git push origin master
git push origin v0.8.0
```

> The macOS binary is not signed or notarized by Apple. If Gatekeeper blocks the first launch, right-click to open it or run `xattr -d com.apple.quarantine trix`.

## Performance

Environment: Apple Silicon (ARM64), macOS, Rust 1.96.1, `--release` (LTO + `codegen-units=1` + `panic=abort` + `strip`).
Canvas 160×48, 240-frame offscreen benchmark:

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
| fairy | 0.018 ms | 0.022 ms | 2.0 KiB/frame |
| tornado | 0.043 ms | 0.051 ms | 7.8 KiB/frame |
| storm | 0.109 ms | 0.122 ms | 29.9 KiB/frame |
| monalisa | 0.021 ms | 0.032 ms | 0.0 KiB/frame (after going static) |
| ballet | 0.120 ms | 0.151 ms | 14.5 KiB/frame |
| demoneye | 0.273 ms | 0.398 ms | 15.4 KiB/frame |
| life | 0.158 ms | 0.209 ms | 5.6 KiB/frame |

Real PTY, truecolor, 120×36, 30 FPS, 5 seconds (CPU = child process `utime+stime / 5s`, including PTY writes but not the terminal emulator's own drawing):

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
| fairy | 0.18% | 2.08 MiB | 0.1 MiB |
| tornado | 0.27% | 2.08 MiB | 0.7 MiB |
| storm | 0.63% | 2.19 MiB | 3.6 MiB |

- At 30 FPS the frame budget is about 33.3 ms; the heaviest effect, `fire`, costs about **1.3% of one core** offscreen (0.433 ms/frame) and about **1.4%** interactively (including PTY writes).
- Startup: `--version` median about **2.0 ms**; real-PTY first-screen bytes about **1.6–2.4 ms** median.
- Memory: peak RSS under a real PTY **2.08–2.27 MiB**.
- Binary: **733 KiB**.
- Interactive mode honors `NO_COLOR`; offscreen `--snapshot` / `--bench` default to truecolor and can be overridden with `--color`.

```bash
scripts/bench.sh
./target/release/trix fire --bench --size 160x48 --frames 300
```

## Implementation

- **Terminal driver** (`src/term.rs`): libc sets raw mode, the alternate screen, a hidden cursor and disabled autowrap; non-blocking key reads, `TIOCGWINSZ` size detection, SIGINT/SIGTERM/SIGHUP flags and a panic hook.
- **Canvas** (`src/canvas.rs`): `Cell { char, fg, bg }` double buffering; per-row changed spans; SGR is emitted for both foreground and background changes, and the background falls back to `40` (the black the program started with) so the previous scene's background never lingers; truecolor / 256 / 16 / mono (mono draws glyphs only). The `monalisa` block portrait uses background color; `mario` does not.
- **Zero framework**: `libc` is the only dependency; no tokio, crossterm or worker threads.
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
- **Oriental Pearl**: Bund night view + Oriental Pearl + Huangpu River reflections; two random boats on the river — hull type (sailboat / freighter / ferry), size, hull palette, channel depth and drift direction are all random; distant boats sit low on the water while near boats are large and press against the frame, with decks / portholes / lit cabin windows / smoking funnels / mast lights / bow lanterns / wakes and water reflections; 1–2 meteors cross the night sky every 3–5 seconds (about 18% of intervals are skipped), with a white-hot head and an ice-blue or warm-gold fading tail, flying into the skyline or fading out and being recycled.
- **Offline map data**: `tools/gen_land.py` downloads Natural Earth and rasterizes it into `src/data/land_720x360.bin`; runtime never touches the network.

## Recording and documentation tools

```bash
trix dragon --snapshot dragon.ans --size 120x36 --frames 145 --seed 7
python3 tools/ansi2png.py dragon.ans dragon.png --size 120x36 --label "trix dragon"
python3 tools/render_gif.py dragon docs/dragon.gif --start 100 --frames 24 --step 3
```

## Compatibility

- Runs on macOS, Linux and WSL (Unix termios); native Windows is not implemented.
- 24-bit truecolor is recommended; `--color auto` degrades automatically.
- Effects that support both ASCII and Unicode **always start in ASCII**; add `--unicode` or `--charset unicode` for Unicode. The Unicode `matrix` includes the blue/white wordmark and Unicode borders / progress bars; `--charset kana` can mix in half-width katakana.
- Linux binaries are static musl builds for `x86_64` and `aarch64`, so they run on any distribution (including Alpine) on those architectures.
- Non-TTY runs suggest `--snapshot`.
- The canvas is black; a dark terminal is recommended.

## Testing

```bash
cargo test
cargo clippy --all-targets
cargo fmt --check
```

There are currently **144** unit / integration tests: color conversion, key parsing, CLI / new-argument validation, canvas clipping / diffing, the map mask, 27 effects across sizes / themes / colors / Unicode fallback, ASCII-only default charsets, no panic in small terminals, deterministic seeds, Tom & Jerry speed / jump, tornado height, Demon Eye blink rhythm (8–15 s) / gaze / dart-and-return / pupil never covered by the highlight / still switch, ballet hand-holding / full-screen path, and Threads of Life generations / timeline / dog and cat / symbols / zero lane changes for the core / death heads / end markers.

Real-PTY checks: `q` / `Esc` / `Ctrl-C` / `SIGTERM`, pause, help, number / `[` / `]` switching, `i` status, resize, `--no-alt`, color fallback, and terminal restoration on exit.

## Versions and rollback

- **Current version: 0.8.0** (`trix --version`), tag **`v0.8.0`**.
- Previous: tag **`v0.7.0`** (aquarium / landscape / Oriental Pearl / Tiananmen / traffic / fairy / tornado / thunderstorm, playlists); **`v0.6.0`** (Mario / ECG / dandelion, blue-green code rain); **`v0.5.1`** (automated release pipeline).
- Earlier: tag **`v0.4.0`** (Unicode NASA login, connected fire, adjustable parameters); **`v0.3.0`** (multi-stack fire, 7 new effects); **`v0.2.0`** (Chinese dragon, NASA intrusion); **`v0.1.0`** (initial baseline).
- Rollback:
  ```bash
  git checkout v0.7.0     # inspect the previous version only (detached HEAD)
  git switch master
  git reset --hard v0.7.0 # fully revert to the previous version
  ```
- Extra backups (not in git): `backups/trix-v0.1.0-*` through `trix-v0.7.0-*`, containing source tarballs and aarch64 macOS binaries.
- Change log: [English](CHANGELOG.md) · [中文](CHANGELOG.zh-CN.md).

## Data sources, references and license

- Land outlines: [Natural Earth](https://www.naturalearthdata.com/) 1:110m land, **public domain**, converted from GeoJSON via [natural-earth-vector](https://github.com/nvkelso/natural-earth-vector).
- Doom-fire inspiration: [pserey/doom-fire-algorithm](https://github.com/pserey/doom-fire-algorithm), [horizonwiki/fire](https://github.com/horizonwiki/fire); this project's Rust implementation, multiple fuel stacks, palette and rendering are independently written.
- The Buddha ASCII art is adapted from [ascii.co.uk/art/buddha](https://ascii.co.uk/art/buddha) (original author credited as `hjw`) for the `buddha` scene; this project adds color, halo, lotus base, particles and animation.
- This project's code: [MIT](LICENSE).

---

```text
  t r i x  ::  stay curious, look up.
```
