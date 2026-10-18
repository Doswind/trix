//! Character and prop art for the `mario` scene.
//!
//! One char-block cast, drawn twice:
//!
//! * ASCII (default) — glyphs chosen by material (`#` red, `o` skin, `=`
//!   overalls, `#` ink, `*` button), pure ASCII.
//! * `--unicode` — the same blocks with `█` in place of the ASCII glyphs.
//!
//! Nothing here paints a background colour: the screen is glyphs only, so the
//! terminal background is left untouched.
//!
//! Names here are deliberately short: the scene indexes them per frame.

use crate::color::Rgb;

/// Dim star dots for the empty terminal background.
pub const STAR: Rgb = Rgb::hex(0x2f5fa8);
pub const CLOUD: Rgb = Rgb::hex(0xf8fcff);
pub const CLOUD_SH: Rgb = Rgb::hex(0xcfe2ff);
pub const HILL: Rgb = Rgb::hex(0x0b5c1a);
pub const HILL_HI: Rgb = Rgb::hex(0x2f9e4f);
pub const HILL_DK: Rgb = Rgb::hex(0x04300d);
pub const BRICK: Rgb = Rgb::hex(0xc8501a);
pub const BRICK_HI: Rgb = Rgb::hex(0xf08a3a);
pub const BRICK_DK: Rgb = Rgb::hex(0x6b2406);
pub const SOIL: Rgb = Rgb::hex(0x24120a);
pub const SOIL_HI: Rgb = Rgb::hex(0x461c08);
pub const INK: Rgb = Rgb::hex(0x1a0e08);
pub const RED: Rgb = Rgb::hex(0xe43b44);
pub const RED_DK: Rgb = Rgb::hex(0xa81c24);
pub const SKIN: Rgb = Rgb::hex(0xf8c08c);
pub const SKIN_DK: Rgb = Rgb::hex(0xd08f5c);
pub const BLUE: Rgb = Rgb::hex(0x3b5dc9);
pub const BLUE_DK: Rgb = Rgb::hex(0x223388);
pub const GOLD: Rgb = Rgb::hex(0xf7d51d);
pub const GOLD_DK: Rgb = Rgb::hex(0xb88600);
pub const WHITE: Rgb = Rgb::hex(0xf8f8f8);
pub const BROWN: Rgb = Rgb::hex(0x6b2e10);
pub const PIPE: Rgb = Rgb::hex(0x00a800);
pub const PIPE_HI: Rgb = Rgb::hex(0x8cf08c);
pub const PIPE_DK: Rgb = Rgb::hex(0x005800);
pub const GOOMBA: Rgb = Rgb::hex(0xe89a22);
pub const GOOMBA_DK: Rgb = Rgb::hex(0x8a5a1e);

/// Material table: art key -> (ascii glyph, unicode glyph, colour).
pub fn material(key: char) -> Option<(char, char, Rgb)> {
    Some(match key {
        'R' => ('#', '▄', RED),
        'r' => ('#', '▄', RED_DK),
        'S' => ('o', '█', SKIN),
        's' => ('o', '█', SKIN_DK),
        'K' => ('#', '█', INK),
        'B' => ('=', '█', BLUE),
        'b' => ('=', '█', BLUE_DK),
        'Y' => ('*', '█', GOLD),
        'W' => ('o', '█', WHITE),
        'N' => ('=', '▀', BROWN),
        'O' => ('%', '█', GOOMBA),
        'o' => ('%', '█', GOOMBA_DK),
        // Coin furniture: parentheses in ASCII, solid edges in Unicode.
        'L' => ('(', '▐', GOLD_DK),
        'J' => (')', '▌', GOLD_DK),
        'T' => ('-', '█', GOLD_DK),
        _ => return None,
    })
}

/// 12 columns wide, 7 rows tall: small Mario, facing right.
pub const MARIO_STAND: &[&str] = &[
    "..RRRRRRR...",
    ".RRRRRRRRRRR",
    ".KSSSSSSKSSS",
    ".KSSSKKKSSSS",
    ".SRRRBBBBRRS",
    "SSBBYBBYBBSS",
    "..NNN..NNN..",
];

pub const MARIO_RUN_A: &[&str] = &[
    "..RRRRRRR...",
    ".RRRRRRRRRRR",
    ".KSSSSSSKSSS",
    ".KSSSKKKSSSS",
    ".SRRRBBBBRRS",
    ".SBBYBBYBBSS",
    "..NNN.NNN...",
];

pub const MARIO_RUN_B: &[&str] = &[
    "..RRRRRRR...",
    ".RRRRRRRRRRR",
    ".KSSSSSSKSSS",
    ".KSSSKKKSSSS",
    ".SRRRBBBBRRS",
    "SSBBYBBYBBSS",
    ".NNN...NNN..",
];

pub const MARIO_JUMP: &[&str] = &[
    "..RRRRRRR...",
    ".RRRRRRRRRRR",
    ".KSSSSSSKSSS",
    ".KSSSKKKSSSS",
    "SSRRRBBBBRRS",
    "SSBBYBBYBBSS",
    ".NNN....NNN.",
];

/// Goomba: 10 wide, 6 tall.
pub const GOOMBA_A: &[&str] = &[
    "..KKKKKK..",
    ".KOOOOOOK.",
    "KOWWKKWWOK",
    "KOOOKKOOOK",
    ".KKKKKKKK.",
    "..KK..KK..",
];

pub const GOOMBA_B: &[&str] = &[
    "..KKKKKK..",
    ".KOOOOOOK.",
    "KOWWKKWWOK",
    "KOOOKKOOOK",
    ".KKKKKKKK.",
    ".KK....KK.",
];

pub const GOOMBA_FLAT: &[&str] = &[
    "..........",
    "..........",
    "..KKKKKK..",
    ".KOOOOOOK.",
    "KOKOKOKOKO",
    ".KK....KK.",
];

/// Super mushroom: 10 wide, 6 tall.
pub const MUSHROOM: &[&str] = &[
    "..KKKKKK..",
    ".KRRRRRRK.",
    "KRWWWRRRRK",
    "KRWWWRRWWK",
    "KRRRRWWWWK",
    ".KWWWWWWK.",
];

/// Spinning coin: three silhouettes sell the turn.
pub const COIN_WIDE: &[&str] = &[".TTT.", "LYWYJ", "LYWYJ", "LYWYJ", ".TTT."];
pub const COIN_MID: &[&str] = &[".T.", "LYJ", "LWJ", "LYJ", ".T."];
pub const COIN_THIN: &[&str] = &["T", "Y", "W", "Y", "T"];
