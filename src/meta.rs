//! Effect metadata shared by `--help`, `--list`, the interactive HUD and the
//! README consistency checks.  Keeping it here means an effect is described
//! in one place instead of three.

use crate::scene::EffectId;

/// Broad grouping used by the help screens and the styled catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Nature,
    SciFi,
    Art,
}

impl Category {
    pub const ALL: [Category; 3] = [Category::Nature, Category::SciFi, Category::Art];

    pub fn title(self) -> &'static str {
        match self {
            Category::Nature => "SPACE & NATURE",
            Category::SciFi => "SCI-FI & CITY",
            Category::Art => "CHARACTERS & ART",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Category::Nature => "nature",
            Category::SciFi => "sci-fi",
            Category::Art => "art",
        }
    }
}

/// How an effect reacts to the charset flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharsetSupport {
    /// ASCII-only: `--unicode` is accepted but changes nothing.
    Ascii,
    /// Both glyph sets, starting in ASCII.
    Dual,
}

impl CharsetSupport {
    pub fn badge(self) -> &'static str {
        match self {
            CharsetSupport::Ascii => "A",
            CharsetSupport::Dual => "A+U",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CharsetSupport::Ascii => "ascii only",
            CharsetSupport::Dual => "ascii default, --unicode available",
        }
    }
}

/// A knob that only matters for one effect.
#[derive(Clone, Copy, Debug)]
pub struct Tuning {
    pub flag: &'static str,
    pub value: &'static str,
    pub default: &'static str,
    pub desc: &'static str,
}

const BEAMS_TUNING: &[Tuning] = &[
    Tuning {
        flag: "--density",
        value: "<f>",
        default: "1.0",
        desc: "beam density",
    },
    Tuning {
        flag: "--beam-speed",
        value: "<f>",
        default: "1.0",
        desc: "horizontal beam speed",
    },
    Tuning {
        flag: "--beam-motion",
        value: "<m>",
        default: "straight",
        desc: "straight | jump",
    },
];

const BLACKHOLE_TUNING: &[Tuning] = &[
    Tuning {
        flag: "--spin",
        value: "<f>",
        default: "0.5",
        desc: "rotation speed",
    },
    Tuning {
        flag: "--meteors",
        value: "<n>",
        default: "3",
        desc: "meteor count",
    },
];

const GALAXY_TUNING: &[Tuning] = &[
    Tuning {
        flag: "--density",
        value: "<f>",
        default: "1.0",
        desc: "star density",
    },
    Tuning {
        flag: "--twinkle",
        value: "<f>",
        default: "1.0",
        desc: "twinkle rate",
    },
    Tuning {
        flag: "--meteors",
        value: "<n>",
        default: "3",
        desc: "meteor count",
    },
];

const FIREWORKS_TUNING: &[Tuning] = &[Tuning {
    flag: "--density",
    value: "<f>",
    default: "1.0",
    desc: "firework density",
}];

const TRAFFIC_TUNING: &[Tuning] = &[
    Tuning {
        flag: "--density",
        value: "<f>",
        default: "1.0",
        desc: "traffic density",
    },
    Tuning {
        flag: "--street",
        value: "<dir>",
        default: "horizontal",
        desc: "horizontal | vertical",
    },
];

const DANDELION_TUNING: &[Tuning] = &[
    Tuning {
        flag: "--seed-count",
        value: "<n>",
        default: "90",
        desc: "drifting seeds",
    },
    Tuning {
        flag: "--seed-speed",
        value: "<f>",
        default: "1.0",
        desc: "seed drift speed",
    },
];

const TORNADO_TUNING: &[Tuning] = &[
    Tuning {
        flag: "--tornado-count",
        value: "<n>",
        default: "3",
        desc: "tornado count",
    },
    Tuning {
        flag: "--tornado-speed",
        value: "<f>",
        default: "1.0",
        desc: "travel speed",
    },
];

const LIFE_TUNING: &[Tuning] = &[Tuning {
    flag: "--life-cycle",
    value: "<secs>",
    default: "120",
    desc: "seconds per 0-100 lifetime",
}];

const DEMONEYE_TUNING: &[Tuning] = &[Tuning {
    flag: "--eye-motion",
    value: "<m>",
    default: "dart",
    desc: "dart | still",
}];

const MONALISA_TUNING: &[Tuning] = &[
    Tuning {
        flag: "--ghost",
        value: "",
        default: "off",
        desc: "flat pure-ASCII portrait",
    },
    Tuning {
        flag: "--gallery",
        value: "",
        default: "off",
        desc: "soft gallery glow",
    },
];

const DRAGON_TUNING: &[Tuning] = &[Tuning {
    flag: "--plain",
    value: "",
    default: "off",
    desc: "plain starfield backdrop",
}];

const TOMJERRY_TUNING: &[Tuning] = &[Tuning {
    flag: "--jump",
    value: "",
    default: "off",
    desc: "enable jumping",
}];

const DSH_TUNING: &[Tuning] = &[Tuning {
    flag: "--trail",
    value: "<f>",
    default: "1.0",
    desc: "drop-ghost strength (0 = off)",
}];

const LAUNCH_TUNING: &[Tuning] = &[
    Tuning {
        flag: "--rocket-scale",
        value: "<f>",
        default: "1.0",
        desc: "rocket size multiplier (0.55-1.6)",
    },
    Tuning {
        flag: "--shake",
        value: "<f>",
        default: "1.0",
        desc: "camera shake strength (0 = off)",
    },
];

impl EffectId {
    /// One short line for the overview grid and `--list`.
    pub fn summary(self) -> &'static str {
        match self {
            EffectId::Rain => "Blue/green code rain with source tokens and data packets",
            EffectId::Fire => "Wide bottom fire bed with slender, leaning flame tips",
            EffectId::Dragon => "Chinese dragon over ink-wash mountains and a cloud sea",
            EffectId::Map => "Holographic Earth with city nodes and great-circle links",
            EffectId::Matrix => "NASA backend login wall with auth, radar and ACCESS GRANTED",
            EffectId::BlackHole => "Photon ring, accretion disk, jets and tilted star field",
            EffectId::Beams => "One-way light beams; straight or jumping",
            EffectId::Fireworks => "Layered fireworks over a neon city skyline",
            EffectId::Solar => "The Sun and eight planets, with a sweeping light fan",
            EffectId::Galaxy => "Full-screen Milky Way with clusters, dust and meteors",
            EffectId::Buddha => "Glowing Buddha, lotus base and expanding mandala rings",
            EffectId::TomJerry => "Tom chasing Jerry through a space station",
            EffectId::Mario => "Pure-glyph Super Mario 1-1 platformer",
            EffectId::Ecg => "Green ECG monitor with an irregular ~56 BPM heartbeat",
            EffectId::Dandelion => "Six dandelion heads, bending stems and drifting seeds",
            EffectId::FishTank => "Full aquarium of sprite fish, bubbles and plants",
            EffectId::Landscape => "Layered mountains, a calm lake, a boat and geese",
            EffectId::Shanghai => "Shanghai Bund at night, boats and shooting stars",
            EffectId::Tiananmen => "Tiananmen gate with twin eaves and two-stage radiance",
            EffectId::Traffic => "City street with side-view or top-down traffic",
            EffectId::Tornado => "Twisted funnels drifting left to right with debris",
            EffectId::Storm => "Dark clouds, slanting rain and full-screen lightning",
            EffectId::MonaLisa => "Background-free Mona Lisa portrait; --ghost for ASCII",
            EffectId::Ballet => "Ice-ballet duet looping through 13 sections",
            EffectId::DemonEye => "Giant bloodshot eye that blinks and darts its gaze",
            EffectId::Life => "A 0-100 life timeline of family, love and generations",
            EffectId::Dsh => "Two-line DEEPSEEK HARNESS: reveal, drop, looping cross beam",
            EffectId::Vibe => "Claude Code transcript typing this scene's own Rust source",
            EffectId::Lights => "Aurora curtains over a moonlit mountain lake",
            EffectId::Volcano => "Night eruption with lava fountains, ash and lightning",
            EffectId::Launch => "Big two-stage rocket: ignition, max-Q, separation, Earth limb",
            EffectId::Plasma => "Classic demoscene plasma with a shifting palette",
        }
    }

    /// Two or three sentences for `--help <effect>`.
    pub fn about(self) -> &'static str {
        match self {
            EffectId::Rain => "Per-column heads and tails with randomized speed and depth, white-hot heads, blue/green two-tone tails, binary mist, horizontal data packets and a CRT scanline.",
            EffectId::Fire => "Doom-fire propagation: the bottom merges into one wide bed, the middle flames are tallest, and the side tongues lean outward. Embers and flare-ups keep it alive.",
            EffectId::Dragon => "Arc-length body tracking with antlers, whiskers, mane, fangs, claws, fins and scales; a traveling wave runs from head to tail. The default backdrop adds two ink-wash ridges and a drifting cloud sea; --plain restores the old starfield.",
            EffectId::Map => "An embedded Natural Earth land mask, city nodes, great-circle data packets, scan beams and a UTC clock in the HUD.",
            EffectId::Matrix => "ASCII by default; --unicode switches to the blue NASA banner, white wordmark and Unicode login frame. OPERATOR/PASSWORD/2FA, AUTH/DECRYPT, radar and ACCESS GRANTED.",
            EffectId::BlackHole => "Event horizon, photon ring, accretion disk, lensing arcs and jets. The outer stars form a tilted ellipse with differential rotation; --spin and --meteors tune it.",
            EffectId::Beams => "Horizontal beams in random bright colors. --beam-motion straight|jump; --density and --beam-speed adjust the flow.",
            EffectId::Fireworks => "Rockets climb, then inner and outer particle rings bloom with delayed bursts over a city skyline and starfield.",
            EffectId::Solar => "The Sun sits in the top-left; eight planets sit on the orbital line below. Sunlight spreads in layered fans and stops above the planet line.",
            EffectId::Galaxy => "A full-screen star sea with a dense central band, animated clusters, dark dust, meteors and constellation lines.",
            EffectId::Buddha => "Mandala rings expand at equal spacing outside the whole figure; at most five rings, dimmer than the Buddha's gold. Head halo, lotus base and particles remain.",
            EffectId::TomJerry => "Tom chases Jerry through a space station; slow and non-jumping by default. --speed and + / - adjust speed, --jump enables jumping.",
            EffectId::Mario => "A pure-glyph level with clouds, hills, bushes, a brick floor, question blocks, pipes, coins, Goombas and mushrooms. No terminal background color; ASCII by default, --unicode for solid-block sprites.",
            EffectId::Ecg => "A green ECG grid with a P-QRS-T waveform, sweep line and HR/SpO2/SYS-DIA readouts. ASCII by default; --unicode for box drawing and a solid waveform.",
            EffectId::Dandelion => "Three clumps x two stems, six flower heads; the left and right roots bend toward the center. --seed-count and --seed-speed tune the drift.",
            EffectId::FishTank => "Five hand-drawn sprite fish plus 8-30 small fish, bubbles with random shades, and plants. ASCII by default; --unicode for blocks.",
            EffectId::Landscape => "Layered distant mountains, a calm lake, a boat, white geese, jungle on both sides, animated water, mist and petals.",
            EffectId::Shanghai => "The Bund at night with the Oriental Pearl Tower, neon high-rises and Huangpu reflections. Two random boats and 1-2 meteors every 3-5 seconds. ASCII by default.",
            EffectId::Tiananmen => "The gate fills about 92% of the width and 62% of the height: double golden eaves, red pillars, five archways, slogans and the portrait. Golden beams grow, then light up inside-out.",
            EffectId::Traffic => "Horizontal traffic is three-row side-view cars in real lanes with dashed dividers; vertical traffic is top-down cars. --street horizontal|vertical, --density controls flow.",
            EffectId::Tornado => "1-8 twisted funnels move left to right, pointed at the base and wider at the top, with debris drifting outward. --tornado-count and --tornado-speed.",
            EffectId::Storm => "Dark clouds, slanting rain, a city silhouette and jagged bolts. A gray-white high-cloud layer drifts right; lightning lights the whole screen.",
            EffectId::MonaLisa => "A background-free portrait baked from a reference pixel painting, sampled into ANSI color cells and ASCII detail. The default block version is static; --ghost is the flat pure-ASCII edition; --gallery adds a warm halo.",
            EffectId::Ballet => "A 39-scene-second duet looping through 13 sections: glide, crossover, arabesque, promenade, overhead lift, pair spin, death spiral, sissonne, Ina Bauer and finale. Joints use two-bone IK; the default tempo is 2x, about 19.5 seconds per loop.",
            EffectId::DemonEye => "One huge bloodshot eye fills the screen; it blinks every 8-15 seconds and darts its gaze 1-2 times between blinks. --eye-motion dart|still.",
            EffectId::Life => "A 0-100 age timeline with glowing life lines; family, love and generations enter and leave on schedule. The bright beam grows with age; a supernova finale closes ME's timeline. --life-cycle sets the round length.",
            EffectId::Dsh => "A giant two-line DEEPSEEK HARNESS lockup with the second line indented: DEEPSEEK on top, HARNESS below, about 85-90% of the width and over half the screen height. The intro reveals the letters one by one and drops the lockup in stuttered one-row steps, leaving a ghost stack below HARNESS. After that the wordmark never leaves the screen: a seven-second loop floats a horizontal beam down the lockup, joins it with a vertical beam at the centre to form a bright cross, then sweeps the vertical beam left to right while the horizontal one fades. No fireworks or particle burst. ASCII by default; --unicode uses solid blocks, and --trail sets the drop-ghost strength.",
            EffectId::Vibe => "A Claude Code-style transcript reads src/scenes/vibe.rs (or an embedded copy), types it with a small Rust syntax highlighter and loops through six windows per turn; only the lines up to the typing head are visible, and each loop starts from a clean transcript. ASCII by default; --unicode swaps in the Claude Code prompt, tool and spinner glyphs.",
            EffectId::Lights => "Three noisy aurora curtains drift over a starry sky, snow-capped ridges and a moonlit lake reflection. ASCII only: the vertical glyphs and stars are the effect.",
            EffectId::Volcano => "A jagged mountain silhouette, a glowing crater, ballistic lava fountains, two lava streams, drifting ash and periodic lightning. ASCII by default; --unicode uses solid blocks for the mountain.",
            EffectId::Launch => "A twenty-four second launch film: blue-hour pad, ignition flash, a full-height two-stage rocket with twin boosters, layered exhaust and world-space smoke, max-Q, MECO and booster separation, a second-stage climb, then an Earth-limb sunrise. The camera follows and shakes; --rocket-scale and --shake tune the framing. Pure ASCII.",
            EffectId::Plasma => "Four interfering sine fields drive a glyph ramp and a slowly cycling palette, with a subtle scanline. ASCII by default; --unicode uses a compact block ramp.",
        }
    }

    pub fn category(self) -> Category {
        match self {
            EffectId::Rain
            | EffectId::Fire
            | EffectId::Dragon
            | EffectId::Map
            | EffectId::Solar
            | EffectId::Galaxy
            | EffectId::Tornado
            | EffectId::Storm
            | EffectId::Dandelion
            | EffectId::Landscape
            | EffectId::FishTank
            | EffectId::Lights
            | EffectId::Volcano => Category::Nature,
            EffectId::Matrix
            | EffectId::BlackHole
            | EffectId::Beams
            | EffectId::Fireworks
            | EffectId::Ecg
            | EffectId::Traffic
            | EffectId::Shanghai
            | EffectId::Tiananmen
            | EffectId::Dsh
            | EffectId::Vibe
            | EffectId::Launch => Category::SciFi,
            EffectId::Buddha
            | EffectId::TomJerry
            | EffectId::Mario
            | EffectId::MonaLisa
            | EffectId::Ballet
            | EffectId::DemonEye
            | EffectId::Life
            | EffectId::Plasma => Category::Art,
        }
    }

    /// Optional single-key shortcut; the newest generative scenes have none.
    pub fn shortcut(self) -> Option<char> {
        match self {
            EffectId::Rain => Some('1'),
            EffectId::Fire => Some('2'),
            EffectId::Dragon => Some('3'),
            EffectId::Map => Some('4'),
            EffectId::Matrix => Some('5'),
            EffectId::BlackHole => Some('6'),
            EffectId::Beams => Some('7'),
            EffectId::Fireworks => Some('8'),
            EffectId::Solar => Some('9'),
            EffectId::Galaxy => Some('0'),
            EffectId::Buddha => Some('['),
            EffectId::TomJerry => Some(']'),
            EffectId::Mario => Some('m'),
            EffectId::Ecg => Some('e'),
            EffectId::Dandelion => Some('d'),
            EffectId::FishTank => Some('f'),
            EffectId::Landscape => Some('l'),
            EffectId::Shanghai => Some('b'),
            EffectId::Tiananmen => Some('j'),
            EffectId::Traffic => Some('c'),
            EffectId::Tornado => Some('z'),
            EffectId::Storm => Some('k'),
            EffectId::MonaLisa => Some('o'),
            EffectId::Ballet => Some('v'),
            EffectId::DemonEye => Some('g'),
            EffectId::Life => Some('y'),
            EffectId::Dsh => Some('u'),
            EffectId::Vibe => Some('w'),
            EffectId::Lights | EffectId::Volcano | EffectId::Launch | EffectId::Plasma => None,
        }
    }

    pub fn from_shortcut(c: char) -> Option<Self> {
        let c = c.to_ascii_lowercase();
        EffectId::ALL
            .iter()
            .copied()
            .find(|e| e.shortcut() == Some(c))
    }

    pub fn charset_support(self) -> CharsetSupport {
        match self {
            EffectId::Matrix
            | EffectId::Mario
            | EffectId::Ecg
            | EffectId::Dandelion
            | EffectId::FishTank
            | EffectId::Landscape
            | EffectId::Tiananmen
            | EffectId::Tornado
            | EffectId::Storm
            | EffectId::MonaLisa
            | EffectId::Ballet
            | EffectId::DemonEye
            | EffectId::Life
            | EffectId::Shanghai
            | EffectId::Traffic
            | EffectId::Dsh
            | EffectId::Vibe
            | EffectId::Volcano
            | EffectId::Plasma => CharsetSupport::Dual,
            _ => CharsetSupport::Ascii,
        }
    }

    /// Flags that only affect this effect, with their defaults.
    pub fn tuning(self) -> &'static [Tuning] {
        match self {
            EffectId::Beams => BEAMS_TUNING,
            EffectId::BlackHole => BLACKHOLE_TUNING,
            EffectId::Galaxy => GALAXY_TUNING,
            EffectId::Fireworks => FIREWORKS_TUNING,
            EffectId::Traffic => TRAFFIC_TUNING,
            EffectId::Dandelion => DANDELION_TUNING,
            EffectId::Tornado => TORNADO_TUNING,
            EffectId::Life => LIFE_TUNING,
            EffectId::DemonEye => DEMONEYE_TUNING,
            EffectId::MonaLisa => MONALISA_TUNING,
            EffectId::Dragon => DRAGON_TUNING,
            EffectId::TomJerry => TOMJERRY_TUNING,
            EffectId::Dsh => DSH_TUNING,
            EffectId::Launch => LAUNCH_TUNING,
            _ => &[],
        }
    }

    /// How much scene time the help preview should simulate.
    pub fn preview_secs(self) -> f32 {
        match self {
            EffectId::Life => 30.0,
            EffectId::Ballet => 12.0,
            EffectId::MonaLisa => 2.0,
            EffectId::Vibe => 10.0,
            EffectId::Dsh => 5.6,
            EffectId::Lights => 8.0,
            EffectId::Volcano => 8.0,
            EffectId::Launch => 8.0,
            EffectId::Plasma => 5.0,
            EffectId::Fireworks => 6.0,
            EffectId::Storm => 6.0,
            EffectId::Shanghai => 6.0,
            EffectId::Tiananmen => 8.0,
            EffectId::Mario => 6.0,
            EffectId::Traffic => 5.0,
            _ => 4.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_effect_has_metadata() {
        for id in EffectId::ALL {
            assert!(!id.summary().is_empty(), "{id:?} summary");
            assert!(!id.about().is_empty(), "{id:?} about");
            if let Some(c) = id.shortcut() {
                assert!(c.is_ascii(), "{id:?} shortcut");
            }
            assert!(id.preview_secs() >= 1.0, "{id:?} preview");
        }
    }

    #[test]
    fn categories_cover_every_effect_once() {
        let mut seen = Vec::new();
        for cat in Category::ALL {
            for id in EffectId::ALL
                .iter()
                .copied()
                .filter(|e| e.category() == cat)
            {
                assert!(!seen.contains(&id), "{id:?} in two categories");
                seen.push(id);
            }
        }
        assert_eq!(seen.len(), EffectId::ALL.len());
    }

    #[test]
    fn shortcuts_are_unique_and_round_trip() {
        let mut seen = Vec::new();
        for id in EffectId::ALL {
            if let Some(c) = id.shortcut() {
                assert!(!seen.contains(&c), "duplicate shortcut {c}");
                seen.push(c);
                assert_eq!(EffectId::from_shortcut(c), Some(id));
            }
        }
    }

    #[test]
    fn dual_charset_list_matches_the_scenes() {
        let dual: Vec<_> = EffectId::ALL
            .iter()
            .filter(|e| e.charset_support() == CharsetSupport::Dual)
            .map(|e| e.key())
            .collect();
        assert_eq!(
            dual,
            [
                "matrix",
                "mario",
                "ecg",
                "dandelion",
                "fish",
                "landscape",
                "shanghai",
                "tiananmen",
                "traffic",
                "tornado",
                "storm",
                "monalisa",
                "ballet",
                "demoneye",
                "life",
                "dsh",
                "vibe",
                "volcano",
                "plasma"
            ]
        );
    }
}
