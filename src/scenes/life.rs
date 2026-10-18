//! 生命的轨迹 — one life, 0 to 100, told as a field of glowing trajectories.
//!
//! The timeline runs left to right.  Six lights start together at the birth
//! line (me, both grandparents, both parents and my brother); relatives leave
//! the timeline as comets that become stars at the end line, while a spouse,
//! three children, grandchildren and passing classmates/friends weave around
//! the central path.  Everything is analytic and event-driven, so the scene is
//! deterministic and the trailing light-threads can be reconstructed at any
//! age.
//!
//! `--life-cycle <seconds>` sets the length of one 0–100 life (default 120s).

use std::f32::consts::PI;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::{hash01, value_noise, Rng};
use crate::scene::{Frame, Scene};

const C_BG_TOP: Rgb = Rgb::hex(0x060914);
const C_BG_LOW: Rgb = Rgb::hex(0x0e1326);
const C_STARDUST: Rgb = Rgb::hex(0x6f8bb5);
const C_GRID: Rgb = Rgb::hex(0x18233a);
const C_NEBULA_A: Rgb = Rgb::hex(0x233a63);
const C_NEBULA_B: Rgb = Rgb::hex(0x3b204d);
const C_BIRTH: Rgb = Rgb::hex(0xbfefff);
const C_END: Rgb = Rgb::hex(0xfbbf24);
const C_ME: Rgb = Rgb::hex(0xffffff);
const C_GPA: Rgb = Rgb::hex(0xfbbf24);
const C_GMA: Rgb = Rgb::hex(0xf472b6);
const C_DAD: Rgb = Rgb::hex(0x38bdf8);
const C_MOM: Rgb = Rgb::hex(0xa78bfa);
const C_BRO: Rgb = Rgb::hex(0x22d3ee);
const C_SPOUSE: Rgb = Rgb::hex(0xfb7185);
const C_CHILD: [Rgb; 3] = [Rgb::hex(0x4ade80), Rgb::hex(0xfacc15), Rgb::hex(0xfb923c)];
const C_GRAND: [Rgb; 2] = [Rgb::hex(0x86efac), Rgb::hex(0xfde68a)];
const C_PAL: Rgb = Rgb::hex(0xf59e0b);
const C_UNI: Rgb = Rgb::hex(0xc084fc);
const C_WORK: Rgb = Rgb::hex(0x34d399);
const C_FRIEND: Rgb = Rgb::hex(0x94a3b8);
const C_DOG: Rgb = Rgb::hex(0xc2410c);
const C_CAT: Rgb = Rgb::hex(0xf0abfc);

/// Friend lanes are planned on a small integer grid around my row.  A
/// trajectory that ends frees its row, so a later character can be slotted
/// into exactly the same lane.
const LANE_MIN: i32 = -10;
const LANE_MAX: i32 = 10;

/// A child counts as a minor until this age.  While a minor exists, friends
/// and colleagues stay outside the child on their own side of my row.
const KID_MINOR_AGE: f32 = 20.0;

/// The shared friend row moves a little before a newcomer arrives, after the
/// parents have finished sliding down.  Half a year is enough to clear the row
/// by the time the KID / pet / grandchild actually appears.
const FRIEND_LOOKAHEAD: f32 = 0.5;
const FRIEND_SIM_STEP: f32 = 0.25;

/// Parents and the brother slide to their new row over this many years.
const YIELD_GLIDE: f32 = 1.5;

/// The end-point symbol and name blink once every second.
const DEATH_BLINK_SECS: f32 = 1.0;

/// The oldest part of a life stays as dim, scattered dots.  The bright beam
/// grows with the life, but the early section is capped so a very long life
/// does not lose its whole opening.
const EARLY_DOTS_MAX: f32 = 22.0;

/// The main character's supernova sequence, measured in real seconds.
const ME_FINALE_SECS: f32 = 6.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Me,
    Family,
    Brother,
    Spouse,
    Child,
    Grandchild,
    Friend,
    Dog,
    Cat,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Anchor {
    Main,
    Child(usize),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EndKind {
    Star,
    Exit,
    Continue,
}

#[derive(Clone, Copy)]
struct Actor {
    role: Role,
    label: &'static str,
    alt_label: &'static str,
    alt_at: f32,
    color: Rgb,
    birth: f32,
    death: f32,
    end: EndKind,
    anchor: Anchor,
    base: f32,
    /// Age added to the timeline age for family roles: their age at my age
    /// zero, or `-birth` for characters born later in the timeline.
    age_offset: f32,
    amp: f32,
    phase: f32,
    freq: f32,
    diverge_at: f32,
    diverge: f32,
    diverge_dur: f32,
    entry_side: f32,
    exit_side: f32,
    entry_dur: f32,
    exit_dur: f32,
    trail_len: f32,
    yield_at: f32,
    yield_by: f32,
}

impl Actor {
    fn label(&self, age: f32) -> &'static str {
        if !self.alt_label.is_empty() && age >= self.alt_at {
            self.alt_label
        } else {
            self.label
        }
    }

    fn shows_age(&self) -> bool {
        self.role != Role::Friend
    }

    /// The character's own age at timeline age `my_age`.
    fn age_at(&self, my_age: f32) -> f32 {
        (my_age + self.age_offset).max(0.0)
    }

    /// `NAME AGE` for relatives, plain `NAME` for classmates and colleagues.
    fn display_label(&self, my_age: f32) -> String {
        let label = self.label(my_age);
        if self.shows_age() {
            format!("{label} {}", self.age_at(my_age).round() as i32)
        } else {
            label.to_string()
        }
    }
}

#[derive(Clone, Copy)]
struct Event {
    age: f32,
    text: &'static str,
    color: Rgb,
}

#[derive(Clone, Copy)]
struct Pos {
    x: f32,
    y: f32,
}

/// A friend's row over time.  Lane changes are deliberate, rare steps: the
/// friend steps outside a new minor child on its side, yields a row a child or
/// pet actually takes, and moves back inward after all children grow up.
#[derive(Clone, Copy)]
struct LaneKf {
    age: f32,
    lane: f32,
}

struct Plan {
    actors: Vec<Actor>,
    events: Vec<Event>,
    wife_at: f32,
    /// Per-actor lane keyframes.  Empty for everyone except friends.
    lane_keys: Vec<Vec<LaneKf>>,
}

pub struct LifeScene {
    seed: u64,
    w: usize,
    h: usize,
    cycle: u64,
    plan: Plan,
}

#[allow(clippy::too_many_arguments)]
fn base_actor(
    role: Role,
    label: &'static str,
    color: Rgb,
    birth: f32,
    death: f32,
    end: EndKind,
    anchor: Anchor,
    base: f32,
) -> Actor {
    Actor {
        role,
        label,
        alt_label: "",
        alt_at: 0.0,
        color,
        birth,
        death,
        end,
        anchor,
        base,
        age_offset: 0.0,
        amp: 0.8,
        phase: 0.0,
        freq: 0.35,
        diverge_at: 1000.0,
        diverge: 0.0,
        diverge_dur: 8.0,
        entry_side: 0.0,
        exit_side: 0.0,
        entry_dur: 2.8,
        exit_dur: 4.8,
        trail_len: 12.0,
        yield_at: 1000.0,
        yield_by: 0.0,
    }
}

impl Plan {
    fn generate(seed: u64, cycle: u64) -> Self {
        let mut rng =
            Rng::new(seed ^ 0x4c49_4645_5f54_5241 ^ cycle.wrapping_mul(0x9e37_79b9_7f4a_7c15));
        // A separate stream keeps the ages independent from the existing
        // death/birth/lane plan, so adding ages does not reshuffle the scene.
        let mut age_rng =
            Rng::new(seed ^ 0x4147_4553_5f41_4745 ^ cycle.wrapping_mul(0xd1b5_4a32_d192_ed03));

        // Death ages.  Grandparents go first, then parents, brother and me.
        let (mut g1, mut g2) = (rng.range(15.5, 22.0), rng.range(25.0, 30.0));
        if rng.chance(0.5) {
            std::mem::swap(&mut g1, &mut g2);
        }
        let (gpa_death, gma_death) = (g1, g2);

        let (mut p1, mut p2) = (rng.range(37.0, 49.0), rng.range(51.0, 60.0));
        if rng.chance(0.5) {
            std::mem::swap(&mut p1, &mut p2);
        }
        let (dad_death, mom_death) = (p1, p2);

        let bro_death = rng.range(60.0, 75.0);
        let me_death = rng.range(75.0, 85.0);
        let bro_diverge = rng.range(20.0, 24.0);

        // Love, marriage and three children.  The girlfriend takes two years
        // to reach my row, stays beside me for another one or two years, and
        // only then becomes WIFE.  The children follow two or three years
        // after the wedding, which leaves the parents time to step back.
        let spouse_birth = rng.range(25.0, 34.0);
        let wife_at = spouse_birth + 2.0 + rng.range(1.0, 2.0);
        let wife_death = rng.range((me_death + 2.0).max(75.0), 90.0);
        let first_child = wife_at + rng.range(2.0, 3.0);
        let second_child = first_child + rng.range(3.0, 4.0);
        let third_child = second_child + rng.range(3.0, 4.0);
        let child_births = [first_child, second_child, third_child];

        let dog_birth = rng.range(1.0, 3.0);
        let dog_death = dog_birth + rng.range(10.0, 12.0);
        // The cat arrives with the first child.  The parents have already
        // finished sliding down, so CAT can sit directly under KID1 with no
        // empty row between them.
        let cat_birth = first_child;
        let cat_death = cat_birth + rng.range(10.0, 12.0);

        let mut actors = Vec::new();
        let mut me_actor = base_actor(
            Role::Me,
            "ME",
            C_ME,
            0.0,
            me_death,
            EndKind::Star,
            Anchor::Main,
            0.0,
        );
        me_actor.amp = 0.0;
        me_actor.trail_len = 38.0;
        actors.push(me_actor);

        let core_top = -3.0 - rng.below(2) as f32;
        let mut gpa = base_actor(
            Role::Family,
            "GPA",
            C_GPA,
            0.0,
            gpa_death,
            EndKind::Star,
            Anchor::Main,
            core_top,
        );
        gpa.age_offset = age_rng.range(58.0, 70.0);
        gpa.amp = 0.0;
        gpa.phase = 0.4;
        gpa.trail_len = 36.0;
        actors.push(gpa);

        let mut gma = base_actor(
            Role::Family,
            "GMA",
            C_GMA,
            0.0,
            gma_death,
            EndKind::Star,
            Anchor::Main,
            core_top - 1.0,
        );
        gma.age_offset = age_rng.range(56.0, 68.0);
        gma.amp = 0.0;
        gma.phase = 2.1;
        gma.trail_len = 36.0;
        actors.push(gma);

        let mut dad = base_actor(
            Role::Family,
            "DAD",
            C_DAD,
            0.0,
            dad_death,
            EndKind::Star,
            Anchor::Main,
            3.0,
        );
        dad.age_offset = age_rng.range(25.0, 33.0);
        dad.amp = 0.0;
        dad.phase = 1.3;
        dad.trail_len = 36.0;
        // The parents start moving two years before the first child, so by the
        // birth their block has already cleared the rows the child and cat
        // will use.  BRO keeps the same offset and follows them down.
        dad.yield_at = first_child - 2.0;
        dad.yield_by = 2.0;
        actors.push(dad);

        let mut mom = base_actor(
            Role::Family,
            "MOM",
            C_MOM,
            0.0,
            mom_death,
            EndKind::Star,
            Anchor::Main,
            4.0,
        );
        mom.age_offset = age_rng.range(23.0, 31.0);
        mom.amp = 0.0;
        mom.phase = 3.4;
        mom.trail_len = 36.0;
        mom.yield_at = first_child - 2.0;
        mom.yield_by = 2.0;
        actors.push(mom);

        let mut bro = base_actor(
            Role::Brother,
            "BRO",
            C_BRO,
            0.0,
            bro_death,
            EndKind::Star,
            Anchor::Main,
            5.0,
        );
        // BRO is the sibling born around my age zero, usually a few years
        // older.
        bro.age_offset = age_rng.range(0.0, 4.0);
        bro.amp = 0.0;
        bro.phase = 4.0;
        bro.freq = 0.27;
        bro.yield_at = first_child - 2.0;
        bro.yield_by = 2.0;
        bro.diverge_at = bro_diverge;
        bro.diverge = 6.0;
        bro.diverge_dur = 8.0;
        bro.trail_len = 34.0;
        actors.push(bro);

        let mut wife = base_actor(
            Role::Spouse,
            "GF",
            C_SPOUSE,
            spouse_birth,
            wife_death,
            EndKind::Star,
            Anchor::Main,
            1.0,
        );
        wife.alt_label = "WIFE";
        wife.alt_at = wife_at;
        // GF/WIFE is a peer, so her age tracks mine within about two years.
        wife.age_offset = age_rng.range(-2.0, 2.0);
        wife.amp = 0.0;
        wife.phase = 1.1;
        wife.freq = 0.30;
        wife.entry_side = if rng.chance(0.5) { -1.0 } else { 1.0 };
        // Two years to reach my row, then one or two years as GF before the
        // label changes to WIFE (handled by alt_at = wife_at).
        wife.entry_dur = 2.0;
        wife.trail_len = 28.0;
        actors.push(wife);

        // Three children, one per fate: stay close, drift away, leave the
        // frame.  Keeping all three makes those three branches visible.
        // Keep the fates ordered so KID1/KID2 can reserve adjacent rows for
        // their grandchildren and the cat can sit outside KID1.
        let stay_child = 0;
        let drift_child = 1;
        let exit_child = 2;

        let mut child_actor = [usize::MAX; 3];
        // A gapless stack around ME/WIFE:
        //   KID3 -2 / KID2 -1 / ME 0 / WIFE +1 / KID1 +2
        // KID2 is the second child and takes the dog's freed -1 row; KID3
        // arrives later one row above KID2.  Before KID3 is born, KID2 stays
        // at -1, so there is never a blank row between a minor child and the
        // couple.
        let kid_rows = [2.0f32, -1.0, -2.0];
        for i in 0..3 {
            let birth = child_births[i];
            let mut child = base_actor(
                Role::Child,
                match i {
                    0 => "KID1",
                    1 => "KID2",
                    _ => "KID3",
                },
                C_CHILD[i],
                birth,
                100.0,
                EndKind::Continue,
                Anchor::Main,
                kid_rows[i],
            );
            child.age_offset = -birth;
            child.amp = 0.0;
            child.phase = i as f32 * 1.7 + 0.5;
            child.freq = 0.31 + i as f32 * 0.04;
            child.diverge_at = birth + 20.0;
            child.diverge_dur = 9.0;
            child.trail_len = 24.0;

            if i == stay_child {
                // Always drift downwards, away from the KID2/KID3 stack above
                // ME, so GRAND1 never crosses into KID2's row.
                child.diverge = 4.5;
            } else if i == drift_child {
                child.diverge = if rng.chance(0.5) { 12.0 } else { -12.0 };
            } else {
                child.diverge = if rng.chance(0.5) { 2.0 } else { -2.0 };
                child.end = EndKind::Exit;
                child.death = birth + 20.0 + rng.range(0.5, 4.0);
                child.exit_side = if rng.chance(0.5) { -1.0 } else { 1.0 };
                child.exit_dur = 6.0;
            }
            child_actor[i] = actors.len();
            actors.push(child);
        }

        // Grandchildren for the two children who remain in the frame.
        let mut grand_no = 0;
        for child_i in [stay_child, drift_child] {
            let birth = child_births[child_i] + rng.range(28.0, 34.0);
            if birth > 98.5 {
                continue;
            }
            let label = if grand_no == 0 { "GRAND1" } else { "GRAND2" };
            let parent_base = actors[child_actor[child_i]].base;
            // Grandchildren always sit one row outside their parent.  By the
            // time GRAND2 is born, KID2 has already drifted away from the
            // childhood stack, so the fixed -1 offset cannot collide with
            // KID3; the grandchild follows the parent's drifted position.
            let grand_base = if parent_base > 0.0 { 1.0 } else { -1.0 };
            let mut grand = base_actor(
                Role::Grandchild,
                label,
                C_GRAND[grand_no % C_GRAND.len()],
                birth,
                100.0,
                EndKind::Continue,
                Anchor::Child(child_actor[child_i]),
                grand_base,
            );
            grand.age_offset = -birth;
            grand.amp = 0.0;
            grand.phase = grand_no as f32 * 2.3 + 1.0;
            grand.freq = 0.38;
            grand.trail_len = 16.0;
            actors.push(grand);
            grand_no += 1;
        }

        // A dog joins the first twenty years; a cat follows the first child's
        // early life.  Both leave the timeline as stars, but with their own
        // species glyphs.
        let mut dog = base_actor(
            Role::Dog,
            "DOG",
            C_DOG,
            dog_birth,
            dog_death,
            EndKind::Star,
            Anchor::Main,
            -1.0,
        );
        dog.age_offset = -dog_birth;
        dog.amp = 0.0;
        dog.phase = rng.range(0.0, 2.0 * PI);
        dog.freq = 0.30;
        dog.trail_len = 14.0;
        actors.push(dog);

        let cat_child = 0;
        // CAT is one row under KID1 (+3), never two: KID1 +2 / CAT +3 /
        // shared friends +4 / DAD +5 / MOM +6 / BRO +7.
        let cat_side = if actors[child_actor[cat_child]].base > 0.0 {
            1.0
        } else {
            -1.0
        };
        let mut cat = base_actor(
            Role::Cat,
            "CAT",
            C_CAT,
            cat_birth,
            cat_death,
            EndKind::Star,
            Anchor::Child(child_actor[cat_child]),
            cat_side,
        );
        cat.age_offset = -cat_birth;
        cat.amp = 0.0;
        cat.yield_at = first_child;
        cat.yield_by = 0.0;
        cat.phase = rng.range(0.0, 2.0 * PI);
        cat.freq = 0.38;
        cat.trail_len = 14.0;
        actors.push(cat);

        // Friends, classmates and colleagues.  Each category appears once or
        // twice, follows for the requested number of years, then leaves from
        // above or below the frame.
        let friend_specs: [(Role, &'static str, Rgb, f32, f32, f32); 4] = [
            (Role::Friend, "PAL", C_PAL, 6.0, 10.0, 6.0),
            (Role::Friend, "UNI", C_UNI, 18.0, 21.0, 4.0),
            (Role::Friend, "WORK", C_WORK, 24.0, 50.0, 5.0),
            (Role::Friend, "FRIEND", C_FRIEND, 12.0, 70.0, 8.0),
        ];
        for (role, label, color, age0, age1, years) in friend_specs {
            let count = 1 + rng.below(2);
            for n in 0..count {
                let entry = rng.range(age0, age1);
                let mut friend = base_actor(
                    role,
                    label,
                    color,
                    entry,
                    entry + years,
                    EndKind::Exit,
                    Anchor::Main,
                    1.0,
                );
                friend.amp = 0.0;
                friend.phase = rng.range(0.0, 2.0 * PI);
                friend.freq = rng.range(0.25, 0.55);
                friend.entry_side = if rng.chance(0.5) { -1.0 } else { 1.0 };
                friend.exit_side = if rng.chance(0.5) { -1.0 } else { 1.0 };
                friend.entry_dur = 2.4 + n as f32 * 0.4;
                friend.exit_dur = 2.8 + n as f32 * 0.4;
                friend.trail_len = 12.0 + n as f32 * 1.5;
                actors.push(friend);
            }
        }

        // Friends do not get a hard-coded row.  Their lanes are planned
        // against the whole cast below, using the rows freed by earlier
        // characters and yielding only the rows a child or pet actually takes.
        let lane_keys = plan_friend_lanes(&actors);

        // Story captions.  They are short enough for a terminal HUD and make
        // each life feel authored rather than merely procedural.
        let mut events = vec![
            Event {
                age: 0.0,
                text: "LIFE BEGINS",
                color: C_ME,
            },
            Event {
                age: gpa_death,
                text: "GRANDFATHER BECOMES A STAR",
                color: C_GPA,
            },
            Event {
                age: gma_death,
                text: "GRANDMOTHER BECOMES A STAR",
                color: C_GMA,
            },
            Event {
                age: bro_diverge,
                text: "BROTHER DRIFTS AWAY",
                color: C_BRO,
            },
            Event {
                age: spouse_birth,
                text: "A NEW LIGHT: LOVE",
                color: C_SPOUSE,
            },
            Event {
                age: wife_at,
                text: "MARRIAGE",
                color: C_SPOUSE,
            },
            Event {
                age: first_child,
                text: "FIRST CHILD IS BORN",
                color: C_CHILD[0],
            },
            Event {
                age: first_child - 1.8,
                text: "PARENTS STEP BACK FOR THE CHILD",
                color: C_DAD,
            },
            Event {
                age: first_child - 0.8,
                text: "FRIENDS MAKE ROOM FOR THE CHILD",
                color: C_FRIEND,
            },
            Event {
                age: second_child,
                text: "SECOND CHILD IS BORN",
                color: C_CHILD[1],
            },
            Event {
                age: third_child,
                text: "THIRD CHILD IS BORN",
                color: C_CHILD[2],
            },
            Event {
                age: third_child + 20.0,
                text: "THE CHILDREN HAVE GROWN; FRIENDS RETURN",
                color: C_FRIEND,
            },
            Event {
                age: dad_death,
                text: "FATHER BECOMES A STAR",
                color: C_DAD,
            },
            Event {
                age: mom_death,
                text: "MOTHER BECOMES A STAR",
                color: C_MOM,
            },
            Event {
                age: actors[child_actor[stay_child]].diverge_at,
                text: "ONE CHILD STAYS CLOSE",
                color: C_CHILD[stay_child],
            },
            Event {
                age: actors[child_actor[drift_child]].diverge_at,
                text: "ONE CHILD DRIFTS FAR AWAY",
                color: C_CHILD[drift_child],
            },
            Event {
                age: actors[child_actor[exit_child]].death,
                text: "ONE CHILD GOES TO A FARTHER PLACE",
                color: C_CHILD[exit_child],
            },
            Event {
                age: bro_death,
                text: "BROTHER BECOMES A STAR",
                color: C_BRO,
            },
            Event {
                age: me_death,
                text: "MY LIGHT BECOMES A STAR",
                color: C_ME,
            },
            Event {
                age: wife_death,
                text: "WIFE BECOMES A STAR",
                color: C_SPOUSE,
            },
            Event {
                age: dog_birth,
                text: "DOG JOINS THE FAMILY",
                color: C_DOG,
            },
            Event {
                age: dog_death,
                text: "DOG BECOMES A STAR",
                color: C_DOG,
            },
            Event {
                age: cat_birth,
                text: "CAT JOINS THE FAMILY",
                color: C_CAT,
            },
            Event {
                age: cat_death,
                text: "CAT BECOMES A STAR",
                color: C_CAT,
            },
            Event {
                age: 99.0,
                text: "THE NEXT GENERATION CONTINUES",
                color: C_END,
            },
        ];
        // Spell out the short labels at the moment they enter so the
        // bottom-left caption remains understandable.
        for actor in &actors {
            if actor.role != Role::Friend {
                continue;
            }
            let text = match actor.label {
                "PAL" => "PAL: PRIMARY-SCHOOL CLASSMATE",
                "UNI" => "UNI: COLLEGE CLASSMATE",
                "WORK" => "WORK: A COLLEAGUE",
                _ => "FRIEND: A CLOSE FRIEND",
            };
            events.push(Event {
                age: actor.birth,
                text,
                color: actor.color,
            });
        }
        events.sort_by(|a, b| {
            a.age
                .partial_cmp(&b.age)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Plan {
            actors,
            events,
            wife_at,
            lane_keys,
        }
    }

    fn positions(&self, age: f32, w: usize, h: usize) -> Vec<Option<Pos>> {
        let mut out: Vec<Option<Pos>> = vec![None; self.actors.len()];
        let hh = h as f32;
        let main = main_y(age, h);
        let x0 = 2.0;
        let x1 = (w as f32 - 3.0).max(x0 + 1.0);
        let x = x0 + (age / 100.0).clamp(0.0, 1.0) * (x1 - x0);
        // One lane is exactly one terminal row at every terminal size.  Never
        // scale this: rounding a 1.2-row lane gap can silently leave an empty
        // row between WIFE/KID1 or KID1/CAT.
        let gap = 1.0;
        for i in 0..self.actors.len() {
            let a = self.actors[i];
            if age < a.birth || age >= 100.0 {
                continue;
            }
            if a.end == EndKind::Star && age >= a.death {
                continue;
            }
            if a.end == EndKind::Exit && age >= a.death + a.exit_dur {
                continue;
            }

            let anchor = match a.anchor {
                Anchor::Main => main,
                Anchor::Child(parent) => out
                    .get(parent)
                    .copied()
                    .flatten()
                    .map(|p| p.y)
                    .unwrap_or(main),
            };
            // Friends use their planned row; everyone else keeps the authored
            // base row (and any staged yield).
            let mut base = self.lane_of(i, age);
            if age > a.yield_at {
                base += a.yield_by * yield_progress(&a, age);
            }
            let mut y = anchor + (base + a.amp * (age * a.freq + a.phase).sin()) * gap;

            if age > a.diverge_at {
                let p = smoothstep((age - a.diverge_at) / a.diverge_dur.max(0.1));
                y += a.diverge * gap * p;
            }

            if a.entry_side != 0.0 && age < a.birth + a.entry_dur {
                let p = smoothstep((age - a.birth) / a.entry_dur.max(0.1));
                let off = if a.entry_side < 0.0 { -4.0 } else { hh + 4.0 };
                y = off + (y - off) * p;
            }

            let exiting = a.end == EndKind::Exit && age >= a.death;
            if exiting {
                let p = smoothstep((age - a.death) / a.exit_dur.max(0.1));
                y += a.exit_side * p * (hh + 6.0);
            } else if a.entry_side == 0.0 || age >= a.birth + a.entry_dur {
                y = y.clamp(2.0, (hh - 5.0).max(3.0));
            }

            out[i] = Some(Pos { x, y });
        }
        out
    }

    fn lane_of(&self, i: usize, age: f32) -> f32 {
        match self.lane_keys.get(i) {
            Some(keys) if !keys.is_empty() => lane_at(keys, age),
            _ => self.actors[i].base,
        }
    }

    fn latest_event(&self, age: f32) -> Option<Event> {
        self.events
            .iter()
            .rev()
            .find(|e| age >= e.age && age - e.age <= 4.5)
            .copied()
    }
}

fn actor_end(a: &Actor) -> f32 {
    match a.end {
        EndKind::Star => a.death,
        EndKind::Exit => a.death + a.exit_dur,
        EndKind::Continue => 100.0,
    }
}

/// Length in years of the bright part of a life thread.  It scales with how
/// much life has already passed, so an old life keeps a long glowing beam;
/// the earliest years stay behind as dim, scattered dots.
fn beam_span(actor: &Actor, beam_age: f32) -> f32 {
    let lived = (beam_age - actor.birth).max(0.0);
    // The authored trail length is now a weight: main characters keep a
    // slightly longer bright section than friends and pets.
    let fraction = (0.58 + actor.trail_len / 170.0).clamp(0.58, 0.78);
    let early = (lived * (1.0 - fraction)).min(EARLY_DOTS_MAX);
    (lived - early).max(0.1)
}

/// Real seconds since the main character's death, while the supernova
/// sequence is playing.  `None` before death or after the finale has ended.
fn me_finale_elapsed(age: f32, death: f32, cycle_secs: f32) -> Option<f32> {
    let real = (age - death) * cycle_secs / 100.0;
    (0.0..=ME_FINALE_SECS).contains(&real).then_some(real)
}

fn active_at(a: &Actor, age: f32) -> bool {
    age >= a.birth && age < actor_end(a) && age < 100.0
}

/// The row an actor would occupy on its own, before any smooth yielding.
/// Anchored actors (cat, grandchildren) are one or two rows outside a parent.
fn lane_offset(actors: &[Actor], i: usize) -> f32 {
    match actors[i].anchor {
        Anchor::Main => actors[i].base,
        Anchor::Child(parent) => actors[parent].base + actors[i].base,
    }
}

/// How far through a staged row change an actor is, from 0 to 1.
fn yield_progress(a: &Actor, age: f32) -> f32 {
    smoothstep(((age - a.yield_at) / YIELD_GLIDE).clamp(0.0, 1.0))
}

/// The outermost minor child on each side of me.  Friends must stay outside
/// this boundary on whichever side they occupy, but they only have to give up
/// a row that a child or pet actually takes.
#[derive(Clone, Copy)]
struct Guards {
    pos: i32,
    neg: i32,
}

impl Guards {
    fn active(self) -> bool {
        self.pos > 0 || self.neg > 0
    }
}

fn guards(actors: &[Actor], age: f32) -> Guards {
    let mut g = Guards { pos: 0, neg: 0 };
    for (i, a) in actors.iter().enumerate() {
        if a.role != Role::Child {
            continue;
        }
        let due = a.birth <= age + FRIEND_LOOKAHEAD;
        let minor = age < a.birth + KID_MINOR_AGE;
        let alive = age < actor_end(a);
        if !(due && minor && alive) {
            continue;
        }
        let lane = lane_offset(actors, i).round() as i32;
        if lane > 0 {
            g.pos = g.pos.max(lane);
        } else if lane < 0 {
            g.neg = g.neg.max(-lane);
        }
    }
    g
}

fn lane_violates(lane: i32, g: Guards) -> bool {
    if lane > 0 {
        g.pos > 0 && lane <= g.pos
    } else if lane < 0 {
        g.neg > 0 && lane >= -g.neg
    } else {
        true
    }
}

/// Rows unavailable to the shared friend row at `age`.  Friends themselves are
/// skipped: they all sit on one row, so only the authored cast can block it.
/// Dead trajectories are omitted on purpose: their end marker stays at the age
/// they left, so a later life can start on the same row without ever
/// overlapping the older line.
fn occupied_lanes(actors: &[Actor], age: f32, g: Guards) -> Vec<i32> {
    let mut occ = Vec::with_capacity(24);
    let mut push = |lane: i32| {
        if !occ.contains(&lane) {
            occ.push(lane);
        }
    };
    push(0);
    for (i, a) in actors.iter().enumerate() {
        if a.role == Role::Friend {
            continue;
        }
        let soon = a.birth > age && a.birth <= age + FRIEND_LOOKAHEAD;
        // The cat is part of the first child's rows; hold its lane as soon as
        // the child era starts, even though the cat itself arrives a little
        // later.
        let cat_early = a.role == Role::Cat && g.active() && age < a.birth;
        if !(active_at(a, age) || soon || cat_early) {
            continue;
        }
        let lane = lane_offset(actors, i).round() as i32;
        if a.role == Role::Family && matches!(a.label, "DAD" | "MOM" | "BRO") {
            // DAD/MOM/BRO move as one block.  Reserve the row the block is
            // currently on and the row it is heading for, so the shared
            // friend row can take a row the block has already cleared without
            // sitting in its path.
            let base = lane_offset(actors, i);
            let current = base + a.yield_by * yield_progress(a, age);
            let target = base + a.yield_by;
            let lo = current.min(target).round() as i32;
            let hi = current.max(target).round() as i32;
            for l in lo..=hi {
                push(l);
            }
        } else {
            push(lane);
        }
    }
    occ
}

fn choose_friend_lane(occ: &[i32], g: Guards, prefer_side: i32) -> i32 {
    let mut best: Option<(u8, i32, u8, i32)> = None;
    for lane in LANE_MIN..=LANE_MAX {
        if lane == 0 || occ.contains(&lane) || lane_violates(lane, g) {
            continue;
        }
        let side_pen = if prefer_side != 0 && lane.signum() != prefer_side {
            1
        } else {
            0
        };
        let key = (side_pen, lane.abs(), if lane > 0 { 0u8 } else { 1u8 }, lane);
        if best.map_or(true, |b| key < b) {
            best = Some(key);
        }
    }
    best.map(|(_, _, _, lane)| lane)
        .unwrap_or(if prefer_side < 0 { -1 } else { 1 })
}

fn lane_at(keys: &[LaneKf], age: f32) -> f32 {
    let mut lane = keys.first().map(|k| k.lane).unwrap_or(0.0);
    for k in keys {
        if k.age <= age {
            lane = k.lane;
        } else {
            break;
        }
    }
    lane
}

/// Plan the single row shared by every classmate, colleague and friend.  All
/// active friends travel on the same trajectory row; when one category joins
/// while another is still present they simply share that row.  The row is
/// chosen against the authored cast, steps outward just before a minor child
/// or pet needs it, and moves back inward after every child has grown up.
fn plan_friend_lanes(actors: &[Actor]) -> Vec<Vec<LaneKf>> {
    let mut keys = vec![Vec::new(); actors.len()];
    let friends: Vec<usize> = actors
        .iter()
        .enumerate()
        .filter(|(_, a)| a.role == Role::Friend)
        .map(|(i, _)| i)
        .collect();
    if friends.is_empty() {
        return keys;
    }

    let start = friends
        .iter()
        .map(|&i| actors[i].birth)
        .fold(f32::INFINITY, f32::min);
    let max_age = friends
        .iter()
        .map(|&i| actor_end(&actors[i]))
        .fold(0.0f32, f32::max)
        .min(100.0);
    let mut shared: Vec<LaneKf> = Vec::new();
    let mut cur: Option<i32> = None;
    let mut prev_guard_active = false;
    let mut prev_age = start - 0.001;
    let mut age = start;
    loop {
        // A newcomer can appear between two simulation steps.  Activate the
        // shared row at the exact birth so the friend is never briefly drawn
        // on the row it is about to vacate.
        if cur.is_none() {
            let birth = friends
                .iter()
                .map(|&i| actors[i].birth)
                .filter(|b| *b > prev_age && *b <= age + 0.0001)
                .fold(f32::INFINITY, f32::min);
            if birth.is_finite() {
                let g = guards(actors, birth);
                let occ = occupied_lanes(actors, birth, g);
                // The first choice is the other side of the dog, as close to
                // me as the current cast allows.
                let lane = choose_friend_lane(&occ, g, 1);
                cur = Some(lane);
                shared.push(LaneKf {
                    age: birth,
                    lane: lane as f32,
                });
            }
        }

        let active = friends.iter().any(|&i| active_at(&actors[i], age));
        let g = guards(actors, age);
        let guard_active = g.active();
        let return_now = prev_guard_active && !guard_active;
        let occ = occupied_lanes(actors, age, g);

        if active {
            if let Some(lane) = cur {
                let forced = return_now || lane_violates(lane, g) || occ.contains(&lane);
                if forced {
                    let side = if lane > 0 { 1 } else { -1 };
                    let want = if return_now {
                        Guards { pos: 0, neg: 0 }
                    } else {
                        g
                    };
                    let next = choose_friend_lane(&occ, want, side);
                    if next != lane {
                        cur = Some(next);
                        shared.push(LaneKf {
                            age,
                            lane: next as f32,
                        });
                    }
                }
            }
        } else {
            // Nobody is on the row; the next arrival will choose afresh.
            cur = None;
        }

        prev_guard_active = guard_active;
        prev_age = age;
        if age >= max_age {
            break;
        }
        age = (age + FRIEND_SIM_STEP).min(max_age);
    }

    for &i in &friends {
        keys[i] = shared.clone();
    }
    keys
}

impl LifeScene {
    pub fn new(seed: u64) -> Self {
        LifeScene {
            seed,
            w: 0,
            h: 0,
            cycle: 0,
            plan: Plan::generate(seed, 0),
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
    }

    fn axis_y(&self) -> f32 {
        self.h as f32 - 2.5
    }

    fn x0(&self) -> f32 {
        2.0
    }

    fn x1(&self) -> f32 {
        (self.w as f32 - 3.0).max(self.x0() + 1.0)
    }

    fn x_for_age(&self, age: f32) -> f32 {
        self.x0() + (age / 100.0).clamp(0.0, 1.0) * (self.x1() - self.x0())
    }

    fn draw_background(&self, canvas: &mut Canvas, t: f32, age: f32, unicode: bool) {
        let hh = self.h as f32;
        for y in 0..self.h as isize {
            let v = y as f32 / hh.max(1.0);
            let base = C_BG_TOP.lerp(C_BG_LOW, v);
            for x in 0..self.w as isize {
                let n = hash01(x as i64 / 2, y as i64 / 2, self.seed ^ self.cycle);
                let neb = value_noise(
                    x as f32 * 0.045 + t * 0.004,
                    y as f32 * 0.09,
                    self.seed ^ (self.cycle << 7),
                );
                if n > 0.91 || (neb > 0.72 && n > 0.62) {
                    let col = base.add(C_STARDUST, neb * 0.16 + 0.04);
                    canvas.set(x, y, if unicode { '·' } else { '.' }, col);
                }
            }
        }

        // A soft, slowly moving nebula keeps the empty future from feeling flat.
        for y in 0..self.h as isize {
            for x in 0..self.w as isize {
                let n1 = value_noise(
                    x as f32 * 0.020 + t * 0.004,
                    y as f32 * 0.060 - t * 0.002,
                    self.seed ^ 0x55,
                );
                let n2 = value_noise(
                    x as f32 * 0.052 - t * 0.003,
                    y as f32 * 0.031 + 8.0,
                    self.seed ^ 0x99,
                );
                let neb = (n1 * 0.62 + n2 * 0.38).powf(1.7);
                if neb > 0.24 {
                    let col = C_NEBULA_A
                        .lerp(C_NEBULA_B, n2)
                        .scale(0.14 + (neb - 0.24) * 0.58);
                    canvas.set_if_blank(x, y, if unicode { '░' } else { ':' }, col);
                    if neb > 0.68 {
                        canvas.add_light(x, y, C_STARDUST, 0.04, ' ');
                    }
                }
            }
        }

        // Decade guides connect the bottom axis to the trajectories.
        for decade in (10..100).step_by(10) {
            let x = self.x_for_age(decade as f32) as isize;
            for y in 2..self.axis_y() as isize {
                if y % 2 == 0 {
                    canvas.set_if_blank(x, y, ':', C_GRID.scale(0.9));
                }
            }
        }

        // A faint vertical guide marks the present moment.
        let current_x = self.x_for_age(age) as isize;
        for y in 2..self.axis_y() as isize {
            if y % 2 == 0 {
                canvas.set_if_blank(
                    current_x,
                    y,
                    if unicode { '┆' } else { ':' },
                    C_ME.scale(0.07),
                );
            }
        }

        // A very dim ribbon suggests the central river of a life.
        let steps = self.w.max(20);
        for k in 0..steps {
            let age = k as f32 / steps as f32 * 100.0;
            let x = self.x_for_age(age);
            let y = main_y(age, self.h);
            canvas.add_light(
                x.round() as isize,
                y.round() as isize,
                C_ME,
                0.032,
                if unicode { '·' } else { '.' },
            );
        }
    }

    fn draw_life_lines(&self, canvas: &mut Canvas, t: f32, age: f32, unicode: bool) {
        let axis = self.axis_y() as isize;
        for y in 1..axis {
            let pulse = (y as f32 * 0.37 + t * 2.2).sin() * 0.5 + 0.5;
            let col = C_BIRTH.scale(0.62 + pulse * 0.38);
            canvas.set(self.x0() as isize, y, if unicode { '┃' } else { '|' }, col);
            canvas.add_light(self.x0() as isize, y, C_BIRTH, 0.20 + pulse * 0.24, ' ');
            for dx in [-3, -2, -1, 1, 2, 3] {
                canvas.set(
                    self.x0() as isize + dx,
                    y,
                    if unicode { '░' } else { ':' },
                    C_BIRTH.scale((0.05 + pulse * 0.08) / dx.abs() as f32),
                );
            }
            let pulse2 = (y as f32 * 0.29 - t * 1.7).sin() * 0.5 + 0.5;
            let col2 = C_END.scale(0.58 + pulse2 * 0.42);
            canvas.set(self.x1() as isize, y, if unicode { '┃' } else { '|' }, col2);
            canvas.add_light(self.x1() as isize, y, C_END, 0.17 + pulse2 * 0.22, ' ');
            for dx in [-3, -2, -1, 1, 2, 3] {
                canvas.set(
                    self.x1() as isize + dx,
                    y,
                    if unicode { '░' } else { ':' },
                    C_END.scale((0.05 + pulse2 * 0.08) / dx.abs() as f32),
                );
            }
        }

        // Birth / end markers and the moving age cursor.
        let axis_i = axis;
        canvas.set(
            self.x0() as isize,
            axis_i,
            if unicode { '╬' } else { '+' },
            C_BIRTH,
        );
        canvas.set(
            self.x1() as isize,
            axis_i,
            if unicode { '╬' } else { '+' },
            C_END,
        );
        canvas.put_str(self.x0() as isize + 2, 1, "BIRTH", C_BIRTH.scale(0.46));
        let end_label = "END";
        canvas.put_str(
            (self.x1() as isize - end_label.len() as isize - 1).max(2),
            1,
            end_label,
            C_END.scale(0.60),
        );

        // Time axis and adaptive labels.
        for x in self.x0() as isize..=self.x1() as isize {
            canvas.set(
                x,
                axis_i,
                if unicode { '═' } else { '-' },
                C_END.scale(0.60),
            );
        }
        let step = if self.w >= 68 { 10 } else { 20 };
        let mut year = 0;
        while year <= 100 {
            let x = self.x_for_age(year as f32) as isize;
            canvas.set(
                x,
                axis_i,
                if unicode { '╪' } else { '+' },
                C_END.scale(0.80),
            );
            let label = format!("{year}");
            let lx = (x - label.len() as isize / 2).max(1);
            canvas.put_str(lx, axis_i + 1, &label, C_END.scale(0.55));
            year += step;
        }
        let label = "100";
        let x100 = (self.x_for_age(100.0) as isize - 1).max(1);
        canvas.put_str(x100, axis_i + 1, label, C_END.scale(0.55));

        for event in &self.plan.events {
            if event.age > 0.0 && event.age <= age {
                let ex = self.x_for_age(event.age) as isize;
                canvas.set(
                    ex,
                    axis_i - 1,
                    if unicode { '◆' } else { '+' },
                    event.color.scale(0.48),
                );
            }
        }
        let cx = self.x_for_age(age) as isize;
        canvas.set(
            cx,
            axis_i - 1,
            if unicode { '▲' } else { '^' },
            C_ME.scale(0.90),
        );
        for d in 1..=2 {
            canvas.add_light(cx, axis_i - 1 - d, C_ME, 0.10, ' ');
        }
    }

    /// The distant past is only a faint field of dots; it stays on screen as
    /// a memory of the path instead of a hard line.  Towards the beam the dots
    /// start mixing with dim dashes, so the hand-off is gradual.
    fn draw_history(&self, canvas: &mut Canvas, age: f32, unicode: bool, fade: f32) {
        for (i, actor) in self.plan.actors.iter().enumerate() {
            let start = actor.birth.max(0.0);
            let end = match actor.end {
                EndKind::Star | EndKind::Exit => actor.death.min(age),
                EndKind::Continue => age.min(100.0),
            };
            let old_end = (end - beam_span(actor, end)).max(start);
            if old_end <= start {
                continue;
            }
            let span = old_end - start;
            let steps = ((span / 0.6).ceil() as usize).clamp(10, 160);
            for k in 0..=steps {
                let f = k as f32 / steps as f32;
                let a = start + span * f;
                let past = self.plan.positions(a, self.w, self.h);
                if let Some(Some(p)) = past.get(i) {
                    let x = p.x.round() as isize;
                    let y = p.y.round() as isize;
                    let ch = if f > 0.72 && k % 2 == 0 {
                        if unicode {
                            '─'
                        } else {
                            '-'
                        }
                    } else if unicode {
                        '·'
                    } else {
                        '.'
                    };
                    let col = actor.color.scale((0.08 + f * 0.12) * fade);
                    canvas.set(x, y, ch, col);
                    if k % 9 == 0 {
                        canvas.add_light(x, y, actor.color, 0.035 * fade, ' ');
                    }
                }
            }
        }
    }

    /// The recent life is a long bright beam.  Its tail fades into the dim
    /// dots of the early history, so the whole trajectory reads as one
    /// continuous thread: scattered dots → dashes → bright `=` → `****@`.
    fn draw_beams(
        &self,
        canvas: &mut Canvas,
        positions: &[Option<Pos>],
        age: f32,
        unicode: bool,
        fade: f32,
    ) {
        let mut sparkles: Vec<(isize, isize, Rgb)> = Vec::new();
        for (i, actor) in self.plan.actors.iter().enumerate() {
            let memory_end = if (actor.end == EndKind::Star && age >= actor.death)
                || (actor.end == EndKind::Exit && age >= actor.death + actor.exit_dur)
            {
                Some(actor.death)
            } else {
                None
            };
            if memory_end.is_none() && positions[i].is_none() {
                continue;
            }
            let beam_age = memory_end.unwrap_or(age);
            // The bright beam now grows with the life instead of being a short
            // fixed tail.  The opening years stay as dim dots above.
            let beam_len = beam_span(actor, beam_age)
                .min(beam_age - actor.birth + 0.01)
                .max(0.1);
            let steps = ((beam_len * 2.2).ceil() as usize).clamp(12, 84);
            let mut prev: Option<(isize, isize)> = None;
            for k in 0..=steps {
                let f = k as f32 / steps as f32;
                let a = beam_age - beam_len * (1.0 - f);
                if a < actor.birth - 0.001 {
                    continue;
                }
                let past = self.plan.positions(a, self.w, self.h);
                if let Some(Some(p)) = past.get(i) {
                    let x = p.x.round() as isize;
                    let y = p.y.round() as isize;
                    let dim = if memory_end.is_some() { 0.45 } else { 1.0 };
                    // Dim at the tail, bright at the living head.
                    let col = actor.color.scale((0.16 + f * 0.74) * fade * dim);
                    if let Some((px, py)) = prev {
                        let dx = x - px;
                        let dy = y - py;
                        // The history already supplied the early dots; the
                        // beam itself is dashes that brighten into `=`.
                        let mut ch = beam_glyph(dx, dy, f > 0.62, unicode);
                        // A few wave glyphs break up long horizontal runs, the
                        // way the reference thread mixes `-` and `~`.
                        if f > 0.18 && k % 11 == 5 && matches!(ch, '-' | '=') {
                            ch = if unicode { '≈' } else { '~' };
                        }
                        canvas.line(px, py, x, y, ch, col);
                    } else {
                        canvas.set(x, y, if unicode { '·' } else { '.' }, col);
                    }
                    if k < steps && steps - k <= 4 {
                        // Four stars immediately behind the @ head.
                        sparkles.push((x, y, actor.color.scale(0.95 * fade * dim)));
                    }
                    if k % 4 == 0 && f > 0.20 {
                        canvas.add_light(
                            x,
                            y,
                            actor.color,
                            (0.05 + f * 0.14)
                                * fade
                                * if memory_end.is_some() { 0.55 } else { 1.0 },
                            ' ',
                        );
                    }
                    prev = Some((x, y));
                } else {
                    prev = None;
                }
            }
        }
        for (x, y, col) in sparkles {
            canvas.set(x, y, if unicode { '✧' } else { '*' }, col);
            canvas.add_light(x, y, col, 0.22, ' ');
        }
    }

    fn draw_actors(
        &self,
        canvas: &mut Canvas,
        positions: &[Option<Pos>],
        age: f32,
        unicode: bool,
        fade: f32,
    ) {
        for (i, actor) in self.plan.actors.iter().enumerate() {
            let Some(p) = positions[i] else {
                continue;
            };
            if actor.end == EndKind::Star && age >= actor.death {
                continue;
            }
            let since_birth = (age - actor.birth).max(0.0);
            let birth_boost = if since_birth < 1.2 {
                1.0 + (1.0 - since_birth / 1.2) * 1.3
            } else {
                1.0
            };
            let col = actor.color.scale(fade * birth_boost);
            let ch = actor_glyph(actor.role, unicode);
            let x = p.x.round() as isize;
            let y = p.y.round() as isize;
            canvas.set(x, y, ch, col);
            canvas.add_light(x, y, actor.color, 0.55 * birth_boost, ' ');
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                canvas.add_light(x + dx, y + dy, actor.color, 0.20, ' ');
            }

            // The birth of a child (and the six simultaneous lives at age 0)
            // gets a small expanding ring.
            let is_new_life = matches!(
                actor.role,
                Role::Me | Role::Family | Role::Child | Role::Grandchild | Role::Dog | Role::Cat
            );
            if is_new_life && age >= actor.birth && age - actor.birth < 1.8 {
                let prog = (age - actor.birth) / 1.8;
                self.ring(
                    canvas,
                    p.x,
                    p.y,
                    1.0 + prog * 4.5,
                    actor.color,
                    (1.0 - prog) * 0.62,
                    unicode,
                );
            }

            // Marriage gets a one-off bloom around the couple.
            if actor.role == Role::Spouse
                && age >= self.plan.wife_at
                && age - self.plan.wife_at < 2.2
            {
                let prog = (age - self.plan.wife_at) / 2.2;
                self.ring(
                    canvas,
                    p.x,
                    p.y,
                    1.0 + prog * 5.0,
                    C_SPOUSE,
                    (1.0 - prog) * 0.72,
                    unicode,
                );
            }
        }
    }

    fn star_rank(&self, index: usize) -> usize {
        let mut order: Vec<usize> = self
            .plan
            .actors
            .iter()
            .enumerate()
            .filter(|(_, a)| a.end == EndKind::Star)
            .map(|(i, _)| i)
            .collect();
        order.sort_by(|&a, &b| {
            self.plan.actors[a]
                .death
                .partial_cmp(&self.plan.actors[b].death)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        order.iter().position(|&i| i == index).unwrap_or(0)
    }

    fn draw_departures(&self, canvas: &mut Canvas, age: f32, t: f32, unicode: bool, fade: f32) {
        for (i, actor) in self.plan.actors.iter().enumerate() {
            if actor.end != EndKind::Star || age < actor.death {
                continue;
            }
            let death_age = (actor.death - 0.001).max(0.0);
            let death_pos = self
                .plan
                .positions(death_age, self.w, self.h)
                .get(i)
                .and_then(|p| *p);
            let Some(dp) = death_pos else {
                continue;
            };
            let sy = star_top_y(self.star_rank(i), self.h);
            let comet_end = comet_end(actor);
            let comet_dur = comet_end - actor.death;

            // The last position keeps its species glyph and alternates with
            // the end-point name.
            let dx = dp.x.round() as isize;
            let dy = dp.y.round() as isize;
            if death_marker_visible(t, actor) {
                canvas.set(
                    dx,
                    dy,
                    actor_glyph(actor.role, unicode),
                    actor.color.scale(0.62 * fade),
                );
                canvas.add_light(dx, dy, actor.color, 0.28 * fade, ' ');
            }

            if age < comet_end {
                let p = smoothstep(((age - actor.death) / comet_dur).clamp(0.0, 1.0));
                let ex = self.x1() - 1.5;
                for k in 0..14 {
                    let q = (p - k as f32 * 0.025).max(0.0);
                    let x = dp.x + (ex - dp.x) * q;
                    let y = dp.y + (sy - dp.y) * q;
                    let a = (1.0 - k as f32 / 14.0) * if k == 0 { 1.0 } else { 0.72 } * fade;
                    let ch = if k == 0 {
                        star_glyph(actor.role, unicode)
                    } else if unicode {
                        if k < 4 {
                            '·'
                        } else {
                            '░'
                        }
                    } else if k < 4 {
                        '*'
                    } else {
                        '-'
                    };
                    let xi = x.round() as isize;
                    let yi = y.round() as isize;
                    canvas.set(xi, yi, ch, actor.color.scale(a));
                    if k < 5 {
                        canvas.set(
                            xi,
                            yi + 1,
                            if unicode { '·' } else { ':' },
                            actor.color.scale(a * 0.45),
                        );
                    }
                    canvas.add_light(xi, yi, actor.color, a * 0.52, ' ');
                }
                if p < 0.8 {
                    self.ring(
                        canvas,
                        dp.x,
                        dp.y,
                        1.0 + p * 5.5,
                        actor.color,
                        (1.0 - p) * 0.6,
                        unicode,
                    );
                }
            } else {
                // Stars keep their species glyph but visibly breathe and
                // throw light rays; this is deliberately stronger than a
                // static terminal glyph.
                let tw = (t * 6.0 + actor.phase * 2.0).sin() * 0.5 + 0.5;
                let pulse = (t * 11.0 + actor.phase * 3.0).sin() * 0.5 + 0.5;
                let col = actor
                    .color
                    .lerp(C_ME, 0.18 + tw * 0.46)
                    .scale(fade * (0.65 + tw * 0.35));
                let x = (self.x1() - 1.0) as isize;
                let y = sy.round() as isize;
                canvas.set(x, y, star_glyph(actor.role, unicode), col);
                canvas.add_light(x, y, actor.color, 0.85 + tw * 0.75, ' ');
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    canvas.set(
                        x + dx,
                        y + dy,
                        if unicode { '✦' } else { '*' },
                        col.scale(0.35 + pulse * 0.65),
                    );
                }
                for (dx, dy) in [(-1, -1), (1, 1), (-1, 1), (1, -1)] {
                    canvas.set(
                        x + dx,
                        y + dy,
                        if unicode { '·' } else { '.' },
                        col.scale(0.20 + pulse * 0.45),
                    );
                }
                for (dx, dy) in [(-2, 0), (2, 0), (0, -2), (0, 2)] {
                    canvas.set(
                        x + dx,
                        y + dy,
                        if unicode { '·' } else { '.' },
                        col.scale(0.10 + tw * 0.30),
                    );
                }
                if pulse > 0.78 {
                    self.ring(
                        canvas,
                        x as f32,
                        y as f32,
                        1.0 + pulse * 2.6,
                        actor.color,
                        (pulse - 0.78) * 1.8 * fade,
                        unicode,
                    );
                }
                if age - comet_end < 1.5 {
                    let p = (age - comet_end) / 1.5;
                    self.ring(
                        canvas,
                        x as f32,
                        y as f32,
                        1.0 + p * 4.0,
                        C_ME,
                        (1.0 - p) * 0.55,
                        unicode,
                    );
                }
            }
        }
    }

    /// Supernova finale for ME: white-hot core, a life-review sweep, rings,
    /// star dust, a brighter comet and a permanent halo on the final star.
    fn draw_me_finale(
        &self,
        canvas: &mut Canvas,
        age: f32,
        t: f32,
        cycle_secs: f32,
        unicode: bool,
        fade: f32,
    ) {
        let Some(me_i) = self.plan.actors.iter().position(|a| a.role == Role::Me) else {
            return;
        };
        let me = self.plan.actors[me_i];
        if age < me.death {
            return;
        }
        let death_age = (me.death - 0.001).max(0.0);
        let Some(dp) = self
            .plan
            .positions(death_age, self.w, self.h)
            .get(me_i)
            .and_then(|p| *p)
        else {
            return;
        };
        let comet_end = comet_end(&me);
        let top_x = self.x1() - 1.0;
        let top_y = star_top_y(self.star_rank(me_i), self.h);
        let gold = C_ME.lerp(C_END, 0.55);
        let cx = dp.x.round() as isize;
        let cy = dp.y.round() as isize;

        if let Some(real) = me_finale_elapsed(age, me.death, cycle_secs) {
            // 1. White-hot core plus a short radial glow.
            if real < 0.30 {
                let k = (1.0 - real / 0.30).clamp(0.0, 1.0) * fade;
                for dy in -6..=6 {
                    for dx in -10..=10 {
                        let d = ((dx as f32 / 10.0).powi(2) + (dy as f32 / 6.0).powi(2)).sqrt();
                        if d <= 1.0 {
                            canvas.add_light(cx + dx, cy + dy, C_ME, (1.0 - d) * 0.45 * k, ' ');
                        }
                    }
                }
                for (dx, dy) in [
                    (-1, 0),
                    (1, 0),
                    (0, -1),
                    (0, 1),
                    (-1, -1),
                    (1, 1),
                    (-1, 1),
                    (1, -1),
                ] {
                    canvas.set(
                        cx + dx,
                        cy + dy,
                        if unicode { '✦' } else { '*' },
                        C_ME.scale((0.35 + k * 0.65).min(1.2)),
                    );
                }
                canvas.set(
                    cx,
                    cy,
                    if unicode { '✧' } else { '*' },
                    C_ME.scale(k.max(0.2)),
                );
                canvas.add_light(cx, cy, C_ME, 1.2 * k, ' ');
                if real < 0.22 {
                    let burst = (1.0 - real / 0.22).clamp(0.0, 1.0) * fade;
                    for dx in -9..=9isize {
                        let a = (1.0 - dx.abs() as f32 / 10.0) * burst;
                        let ch = if dx.abs() <= 2 {
                            '*'
                        } else if unicode {
                            '─'
                        } else {
                            '-'
                        };
                        canvas.set(cx + dx, cy, ch, C_ME.scale(a));
                    }
                    for dy in -4..=4isize {
                        let a = (1.0 - dy.abs() as f32 / 5.0) * burst;
                        let ch = if dy.abs() <= 1 {
                            '*'
                        } else if unicode {
                            '│'
                        } else {
                            '|'
                        };
                        canvas.set(cx, cy + dy, ch, C_ME.scale(a));
                    }
                    for kk in 0..16 {
                        let ang = kk as f32 / 16.0 * 2.0 * PI;
                        let x = cx + (ang.cos() * 7.0).round() as isize;
                        let y = cy + (ang.sin() * 3.0).round() as isize;
                        canvas.set(
                            x,
                            y,
                            if unicode { '✦' } else { '*' },
                            C_ME.scale(burst * 0.8),
                        );
                    }
                }
            }

            // 2. Life review: a scan line travels from BIRTH to the death
            // point and re-lights ME's thread behind it.
            let review_start = 0.15;
            let review_end = 2.75;
            if real >= review_start && real <= review_end {
                let p = smoothstep(((real - review_start) / 2.60).clamp(0.0, 1.0));
                let sweep_age = me.death * p;
                let sx = self.x_for_age(sweep_age).round() as isize;
                let main = main_y(age, self.h).round() as isize;
                let axis = self.axis_y() as isize;
                for y in 2..axis {
                    if y % 2 == 0 || y == main {
                        for dx in -1..=1isize {
                            let a = (0.32 - dx.abs() as f32 * 0.10) * fade;
                            canvas.set(sx + dx, y, if unicode { '│' } else { '|' }, C_ME.scale(a));
                        }
                    }
                }
                let steps = 40;
                for k in 0..=steps {
                    let f = k as f32 / steps as f32;
                    let a = sweep_age * f;
                    let past = self.plan.positions(a, self.w, self.h);
                    if let Some(Some(pos)) = past.get(me_i) {
                        let x = pos.x.round() as isize;
                        let y = pos.y.round() as isize;
                        let col = if f > 0.70 {
                            C_ME.lerp(C_END, 0.45)
                        } else {
                            me.color
                        };
                        let ch = if f > 0.90 {
                            if unicode {
                                '✧'
                            } else {
                                '*'
                            }
                        } else if f > 0.50 {
                            if unicode {
                                '═'
                            } else {
                                '='
                            }
                        } else if unicode {
                            '─'
                        } else {
                            '-'
                        };
                        canvas.set(x, y, ch, col.scale((0.35 + f * 0.65) * fade));
                        if k % 8 == 0 {
                            canvas.add_light(x, y, col, 0.25 * fade, ' ');
                        }
                    }
                }
                canvas.set(sx, main, if unicode { '✧' } else { '*' }, C_ME.scale(0.95));
                canvas.add_light(sx, main, C_ME, 0.9 * fade, ' ');
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    canvas.set(
                        sx + dx,
                        main + dy,
                        if unicode { '✦' } else { '*' },
                        C_ME.scale(0.55),
                    );
                }

                // As the sweep crosses an already-finished family member,
                // their top star gets a brief sympathetic pulse.
                for (j, other) in self.plan.actors.iter().enumerate() {
                    if j == me_i || other.end != EndKind::Star || other.death > sweep_age + 0.5 {
                        continue;
                    }
                    let ox = self.x_for_age(other.death);
                    if (ox - self.x_for_age(sweep_age)).abs() <= 2.5 {
                        let oy = star_top_y(self.star_rank(j), self.h);
                        canvas.add_light(
                            top_x.round() as isize,
                            oy.round() as isize,
                            other.color,
                            0.9 * fade,
                            ' ',
                        );
                    }
                }
            }

            // 3. Expanding shockwave rings.
            for ring in 0..4 {
                let lt = real - (0.20 + ring as f32 * 0.15);
                let life = 1.3;
                if lt < 0.0 || lt > life {
                    continue;
                }
                let u = lt / life;
                let radius = 2.0 + u * (16.0 + ring as f32 * 2.0);
                let alpha = (1.0 - u) * 0.65 * fade;
                for pass in 0..2 {
                    let rr = radius * if pass == 0 { 1.0 } else { 0.55 };
                    let aa = alpha * if pass == 0 { 1.0 } else { 0.45 };
                    let n = if pass == 0 { 28 } else { 20 };
                    for k in 0..n {
                        let ang = k as f32 / n as f32 * 2.0 * PI;
                        let x = dp.x + ang.cos() * rr * 2.0;
                        let y = dp.y + ang.sin() * rr;
                        let ch = if k % 4 == 0 {
                            if unicode {
                                '✦'
                            } else {
                                '*'
                            }
                        } else if unicode {
                            '·'
                        } else {
                            '.'
                        };
                        canvas.set(x.round() as isize, y.round() as isize, ch, gold.scale(aa));
                        if k % 6 == 0 {
                            canvas.add_light(
                                x.round() as isize,
                                y.round() as isize,
                                gold,
                                0.30 * aa,
                                ' ',
                            );
                        }
                    }
                }
            }

            // 4. Deterministic star-dust burst.
            let spark_t = real - 0.25;
            if spark_t >= 0.0 {
                for k in 0..64i64 {
                    let h1 = hash01(k, 1, self.seed ^ self.cycle);
                    let h2 = hash01(k, 2, self.seed ^ self.cycle);
                    let h3 = hash01(k, 3, self.seed ^ self.cycle);
                    let h4 = hash01(k, 4, self.seed ^ self.cycle);
                    let life = 3.2 + h3 * 1.3;
                    if spark_t > life {
                        continue;
                    }
                    let u = spark_t / life;
                    let angle = h1 * 2.0 * PI;
                    let speed = 9.0 + h2 * 18.0;
                    let x = dp.x + angle.cos() * speed * spark_t * 1.35;
                    let y = dp.y + angle.sin() * speed * 0.65 * spark_t + 1.1 * spark_t * spark_t;
                    let col = C_ME.lerp(C_END, h4).scale((1.0 - u) * fade);
                    let ch = if h4 > 0.66 {
                        if unicode {
                            '✧'
                        } else {
                            '*'
                        }
                    } else if h4 > 0.33 {
                        '+'
                    } else if unicode {
                        '·'
                    } else {
                        '.'
                    };
                    let xi = x.round() as isize;
                    let yi = y.round() as isize;
                    canvas.set(xi, yi, ch, col);
                    if k % 4 == 0 {
                        canvas.add_light(xi, yi, col, 0.35 * (1.0 - u), ' ');
                    }
                }
            }
        }

        if age < comet_end {
            // 5. A thicker, white-gold comet for ME.
            let p =
                smoothstep(((age - me.death) / (comet_end - me.death).max(0.1)).clamp(0.0, 1.0));
            let hx = dp.x + (top_x - 1.5 - dp.x) * p;
            let hy = dp.y + (top_y - dp.y) * p;
            for k in 0..6 {
                let q = (p - k as f32 * 0.020).max(0.0);
                let x = dp.x + (top_x - 1.5 - dp.x) * q;
                let y = dp.y + (top_y - dp.y) * q;
                let ch = if k == 0 {
                    if unicode {
                        '✧'
                    } else {
                        '*'
                    }
                } else if unicode {
                    '·'
                } else {
                    '.'
                };
                canvas.set(
                    x.round() as isize,
                    y.round() as isize,
                    ch,
                    gold.scale((1.0 - k as f32 / 6.0) * 0.9 * fade),
                );
                if k > 0 {
                    let xi = x.round() as isize;
                    let yi = y.round() as isize;
                    for off in [-1isize, 1isize] {
                        canvas.set(
                            xi,
                            yi + off,
                            if unicode { '·' } else { '.' },
                            gold.scale((1.0 - k as f32 / 6.0) * 0.35 * fade),
                        );
                    }
                }
            }
            canvas.add_light(
                hx.round() as isize,
                hy.round() as isize,
                C_ME,
                1.1 * fade,
                ' ',
            );
        } else {
            // 6. Permanent halo on ME's finished star, breathing at 1 Hz.
            let pulse = (t * 2.0 * PI).sin() * 0.5 + 0.5;
            let x = top_x.round() as isize;
            let y = top_y.round() as isize;
            let col = C_ME
                .lerp(C_END, 0.35 + pulse * 0.35)
                .scale(0.55 + pulse * 0.40);
            for (dx, dy) in [
                (-2, 0),
                (2, 0),
                (0, -2),
                (0, 2),
                (-1, -1),
                (1, 1),
                (-1, 1),
                (1, -1),
            ] {
                canvas.set(x + dx, y + dy, if unicode { '✧' } else { '*' }, col);
            }
            self.ring(
                canvas,
                x as f32,
                y as f32,
                2.0 + pulse * 1.8,
                gold,
                (0.18 + pulse * 0.22) * fade,
                unicode,
            );
            self.ring(
                canvas,
                x as f32,
                y as f32,
                3.2 + pulse * 1.5,
                gold,
                (0.10 + pulse * 0.15) * fade,
                unicode,
            );
            for (dx, dy) in [(-3, 0), (3, 0), (0, -3), (0, 3)] {
                canvas.set(
                    x + dx,
                    y + dy,
                    if unicode { '·' } else { '.' },
                    col.scale(0.55),
                );
            }
            canvas.add_light(x, y, C_ME, 1.0 + pulse * 0.7, ' ');
        }

        // The life-review scan does not vanish when it reaches ME's stop
        // line: park the same three-column beam there and keep it for the
        // rest of the cycle, breathing gently instead of blinking out.
        let since_death = (age - me.death) * cycle_secs / 100.0;
        if since_death >= 2.75 {
            let pulse = (t * 2.0 * PI).sin() * 0.5 + 0.5;
            let axis = self.axis_y() as isize;
            let stop_main = main_y(age, self.h).round() as isize;
            let stop_x = cx;
            let beam = C_ME.lerp(C_END, 0.22);
            for y in 2..axis {
                let distance = (y - stop_main).abs() as f32 / (self.h as f32 * 0.5).max(1.0);
                let fade_y = (1.0 - distance * 0.55).clamp(0.35, 1.0);
                let center = (0.20 + pulse * 0.10) * fade_y * fade;
                let side = (0.11 + pulse * 0.05) * fade_y * fade;
                canvas.set(
                    stop_x,
                    y,
                    if unicode { '│' } else { '|' },
                    beam.scale(center),
                );
                for dx in [-1isize, 1isize] {
                    canvas.set(
                        stop_x + dx,
                        y,
                        if unicode { '│' } else { '|' },
                        beam.scale(side),
                    );
                }
                if y % 5 == 0 {
                    canvas.add_light(stop_x, y, C_ME, 0.08 * fade_y * fade, ' ');
                }
            }
        }
    }

    /// All names are painted after every core/trail/star, so no glyph or trail
    /// can cut holes in the middle of a label.
    fn draw_labels(
        &self,
        canvas: &mut Canvas,
        positions: &[Option<Pos>],
        age: f32,
        cycle_secs: f32,
        fade: f32,
    ) {
        let mut rows: Vec<(isize, isize, isize)> = Vec::new();
        for (i, actor) in self.plan.actors.iter().enumerate() {
            let is_star = actor.end == EndKind::Star && age >= actor.death;
            let (pos, reveal) = if is_star {
                let end = comet_end(actor);
                if age < end {
                    (None, 0.0)
                } else {
                    let real_age = (age - end) * cycle_secs / 100.0;
                    let reveal = smoothstep((real_age / 0.5).clamp(0.0, 1.0));
                    let death_age = (actor.death - 0.001).max(0.0);
                    let p = self
                        .plan
                        .positions(death_age, self.w, self.h)
                        .get(i)
                        .and_then(|p| *p)
                        .map(|_dp| (self.x1() - 1.0, star_top_y(self.star_rank(i), self.h)));
                    (p, reveal)
                }
            } else {
                (positions[i].map(|p| (p.x, p.y)), 1.0)
            };
            let Some((px, py)) = pos else {
                continue;
            };
            if py < 1.0 || py > self.h as f32 - 1.0 {
                continue;
            }
            // A finished life keeps its final age; a living one keeps
            // counting up with the timeline.
            let label = actor.display_label(if is_star { actor.death } else { age });
            let len = label.len() as isize;
            let mut row = py.round() as isize;
            let hi = (self.h as isize - 5).max(1);
            if row > hi {
                row = hi;
            }
            let head = px.round() as isize;
            // Moving beams carry their name at the leading edge.  Stars have
            // the finish line to their right, so their name sits just inside.
            let mut anchor = if is_star {
                (head - len - 1).max(1)
            } else {
                head + 2
            };
            if !is_star && anchor + len >= self.x1() as isize - 1 {
                anchor = (head - len - 1).max(1);
            }
            let mut placed = false;
            for attempt in 0..4isize {
                let lx = if is_star || anchor < head {
                    (anchor - attempt * (len + 1)).max(1)
                } else {
                    anchor + attempt * (len + 1)
                };
                let end = lx + len;
                if end >= self.w as isize - 1 {
                    continue;
                }
                let clash = rows
                    .iter()
                    .any(|&(r, a, b)| r == row && lx <= b && end >= a);
                if !clash {
                    rows.push((row, lx, end));
                    canvas.put_str(lx, row, &label, actor.color.scale(0.74 * fade * reveal));
                    placed = true;
                    break;
                }
            }
            if !placed {
                canvas.put_str(anchor, row, &label, actor.color.scale(0.74 * fade * reveal));
            }
        }
    }

    fn draw_death_labels(&self, canvas: &mut Canvas, age: f32, t: f32, fade: f32) {
        for (i, actor) in self.plan.actors.iter().enumerate() {
            if actor.end != EndKind::Star || age < actor.death {
                continue;
            }
            if !death_marker_visible(t, actor) {
                continue;
            }
            let death_age = (actor.death - 0.001).max(0.0);
            if let Some(Some(dp)) = self.plan.positions(death_age, self.w, self.h).get(i) {
                // The final age is part of the blinking end label.
                let label = actor.display_label(actor.death);
                let len = label.len() as isize;
                let dx = dp.x.round() as isize;
                let x = if dx + 2 + len >= self.w as isize - 1 {
                    (dx - len - 1).max(1)
                } else {
                    dx + 2
                };
                let y = dp.y.round() as isize;
                if y >= 1 && y <= self.h as isize - 5 {
                    canvas.put_str(x, y, &label, actor.color.scale(0.58 * fade));
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn ring(
        &self,
        canvas: &mut Canvas,
        cx: f32,
        cy: f32,
        r: f32,
        color: Rgb,
        alpha: f32,
        unicode: bool,
    ) {
        let n = 14;
        for k in 0..n {
            let a = k as f32 / n as f32 * 2.0 * PI;
            let x = cx + a.cos() * r * 2.0;
            let y = cy + a.sin() * r;
            canvas.set(
                x.round() as isize,
                y.round() as isize,
                if unicode { '·' } else { '.' },
                color.scale(alpha),
            );
        }
    }

    fn draw_caption(&self, canvas: &mut Canvas, age: f32) {
        let caption = if let Some(e) = self.plan.latest_event(age) {
            e.text
        } else {
            chapter(age)
        };
        let col = self
            .plan
            .latest_event(age)
            .map(|e| e.color)
            .unwrap_or(C_STARDUST);
        let cap = format!("> {caption}");
        canvas.put_str(2, self.h as isize - 4, &cap, col.scale(0.62));
    }
}

impl Scene for LifeScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn hud_status(&self, f: &Frame) -> Option<String> {
        let cycle_secs = f.tune.life_cycle.clamp(5.0, 600.0);
        let age = ((f.t as f32 / cycle_secs).rem_euclid(1.0) * 100.0).clamp(0.0, 100.0);
        Some(format!("AGE {:.1} | CYCLE {:.0}s", age, cycle_secs))
    }

    fn update(&mut self, f: &Frame) {
        let cycle_secs = f.tune.life_cycle.clamp(5.0, 600.0) as f64;
        let cycle = (f.t / cycle_secs).floor() as u64;
        if cycle != self.cycle {
            self.cycle = cycle;
            self.plan = Plan::generate(self.seed, cycle);
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        let cycle_secs = f.tune.life_cycle.clamp(5.0, 600.0);
        let raw = f.t as f32 / cycle_secs;
        let cycle = raw.floor();
        let age = ((raw - cycle) * 100.0).clamp(0.0, 100.0);
        if cycle as u64 != self.cycle {
            self.cycle = cycle as u64;
            self.plan = Plan::generate(self.seed, self.cycle);
        }
        let t = f.t as f32;
        let fade = if age > 98.0 {
            ((100.0 - age) / 2.0).clamp(0.0, 1.0)
        } else {
            1.0
        };

        self.draw_background(canvas, t, age, unicode);
        let positions = self.plan.positions(age, self.w, self.h);
        self.draw_history(canvas, age, unicode, fade);
        self.draw_beams(canvas, &positions, age, unicode, fade);
        self.draw_life_lines(canvas, t, age, unicode);
        self.draw_actors(canvas, &positions, age, unicode, fade);
        self.draw_departures(canvas, age, t, unicode, fade);
        self.draw_me_finale(canvas, age, t, cycle_secs, unicode, fade);
        self.draw_labels(canvas, &positions, age, cycle_secs, fade);
        self.draw_death_labels(canvas, age, t, fade);
        self.draw_caption(canvas, age);
    }
}

fn beam_glyph(dx: isize, dy: isize, near_head: bool, unicode: bool) -> char {
    let ax = dx.abs();
    let ay = dy.abs();
    if ax >= ay * 3 {
        if ay == 0 {
            if unicode {
                if near_head {
                    '═'
                } else {
                    '─'
                }
            } else if near_head {
                '='
            } else {
                '-'
            }
        } else if unicode {
            '≈'
        } else {
            '~'
        }
    } else if ay >= ax * 2 {
        if unicode {
            '│'
        } else {
            '|'
        }
    } else {
        let falling = (dx > 0) == (dy > 0);
        if unicode {
            if falling {
                '╲'
            } else {
                '╱'
            }
        } else if falling {
            '\\'
        } else {
            '/'
        }
    }
}

fn main_y(_age: f32, h: usize) -> f32 {
    h as f32 * 0.50
}

fn star_top_y(rank: usize, h: usize) -> f32 {
    let hi = (h as f32 - 5.0).max(3.0);
    (3.0 + rank as f32 * 2.0).min(hi)
}

fn comet_end(actor: &Actor) -> f32 {
    actor.death + (2.0 + (actor.phase.sin().abs()) * 0.6) * 2.0
}

/// The end-point marker and its name blink together, on for half a second and
/// off for half a second: exactly one blink per second.
fn death_marker_visible(t: f32, _actor: &Actor) -> bool {
    (t / DEATH_BLINK_SECS).rem_euclid(1.0) < 0.5
}

fn actor_glyph(role: Role, unicode: bool) -> char {
    if unicode {
        match role {
            Role::Dog => '♣',
            Role::Cat => '❂',
            _ => '◉',
        }
    } else {
        match role {
            Role::Dog => 'd',
            Role::Cat => 'c',
            _ => '@',
        }
    }
}

fn star_glyph(role: Role, _unicode: bool) -> char {
    match role {
        Role::Dog | Role::Cat => '+',
        _ => '*',
    }
}

fn chapter(age: f32) -> &'static str {
    match age {
        a if a < 12.0 => "CHILDHOOD",
        a if a < 22.0 => "SCHOOL YEARS",
        a if a < 35.0 => "LOVE AND BEGINNINGS",
        a if a < 60.0 => "FAMILY YEARS",
        a if a < 75.0 => "LETTING GO",
        _ => "LEGACY",
    }
}

fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The row an actor is planned to occupy at `age`, including staged
    /// yields and anchors but ignoring the temporary entry/exit/diverge
    /// animations.
    fn current_lane(p: &Plan, i: usize, age: f32) -> Option<f32> {
        let a = p.actors[i];
        if !active_at(&a, age) {
            return None;
        }
        let mut lane = p.lane_of(i, age);
        if age > a.yield_at {
            lane += a.yield_by * yield_progress(&a, age);
        }
        match a.anchor {
            Anchor::Main => Some(lane),
            Anchor::Child(parent) => {
                let mut parent_lane = current_lane(p, parent, age)?;
                let pa = p.actors[parent];
                if age > pa.diverge_at {
                    let t = smoothstep((age - pa.diverge_at) / pa.diverge_dur.max(0.1));
                    parent_lane += pa.diverge * t;
                }
                Some(parent_lane + lane)
            }
        }
    }

    /// Unlike the planner's lookahead version, this only counts children who
    /// actually exist at `age`; that is the rule the audience sees.
    fn active_guards(p: &Plan, age: f32) -> Guards {
        let mut g = Guards { pos: 0, neg: 0 };
        for (i, a) in p.actors.iter().enumerate() {
            if a.role != Role::Child || !active_at(a, age) || age >= a.birth + KID_MINOR_AGE {
                continue;
            }
            let lane = lane_offset(&p.actors, i).round() as i32;
            if lane > 0 {
                g.pos = g.pos.max(lane);
            } else if lane < 0 {
                g.neg = g.neg.max(-lane);
            }
        }
        g
    }

    #[test]
    fn three_children_and_family_plan() {
        for cycle in 0..5 {
            let p = Plan::generate(7, cycle);
            let kids = p.actors.iter().filter(|a| a.role == Role::Child).count();
            let relays = p
                .actors
                .iter()
                .filter(|a| a.role == Role::Family || a.role == Role::Brother)
                .count();
            assert_eq!(kids, 3);
            assert_eq!(relays, 5, "grandparents + parents + brother");
            assert!(p.actors.len() >= 17);
            let wife = p
                .actors
                .iter()
                .find(|a| a.role == Role::Spouse)
                .expect("wife");
            assert!(wife.birth >= 25.0);
            assert!(wife.birth <= 34.0);
            assert_eq!(
                p.actors.iter().filter(|a| a.end == EndKind::Star).count(),
                9,
                "grandparents, parents, brother, me, wife, dog and cat become stars"
            );
            let gpa = p.actors.iter().find(|a| a.label == "GPA").expect("gpa");
            let gma = p.actors.iter().find(|a| a.label == "GMA").expect("gma");
            assert!((gma.base - gpa.base + 1.0).abs() < 0.01);
            assert!((-5.0..=-3.0).contains(&gpa.base));
            assert_eq!(p.actors.iter().filter(|a| a.role == Role::Dog).count(), 1);
            assert_eq!(p.actors.iter().filter(|a| a.role == Role::Cat).count(), 1);
            let first_child = p
                .actors
                .iter()
                .filter(|a| a.role == Role::Child)
                .map(|a| a.birth)
                .fold(f32::INFINITY, f32::min);
            for a in &p.actors {
                assert!(
                    a.label
                        .chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == ' '),
                    "label is not lowercase words: {}",
                    a.label
                );
                if !a.alt_label.is_empty() {
                    assert!(a
                        .alt_label
                        .chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == ' '));
                }
                if a.role == Role::Dog {
                    assert!((0.0..20.0).contains(&a.birth));
                    assert!((10.0..=12.0).contains(&(a.death - a.birth)));
                }
                if a.role == Role::Cat {
                    assert!(a.birth >= first_child);
                    assert!(a.birth <= first_child + 20.0);
                    assert!((10.0..=12.0).contains(&(a.death - a.birth)));
                }
            }
            assert_eq!(
                p.actors
                    .iter()
                    .filter(|a| a.role == Role::Child && a.end == EndKind::Exit)
                    .count(),
                1,
                "one child leaves the frame"
            );
            assert!(
                p.actors
                    .iter()
                    .filter(|a| a.role == Role::Grandchild)
                    .count()
                    >= 1
            );
            let friends = p.actors.iter().filter(|a| a.role == Role::Friend).count();
            assert!((4..=8).contains(&friends), "friends {friends}");

            let me = p.actors.iter().find(|a| a.role == Role::Me).expect("me");
            let wife = p
                .actors
                .iter()
                .find(|a| a.role == Role::Spouse)
                .expect("wife");
            assert!(wife.end == EndKind::Star);
            assert!(wife.death > me.death);
            assert!((75.0..=90.0).contains(&wife.death));
            assert!(wife.alt_at >= wife.birth + wife.entry_dur + 0.99);
            assert!(wife.alt_at <= wife.birth + wife.entry_dur + 2.01);
            assert!(first_child >= wife.alt_at + 1.99);
            assert!(first_child <= wife.alt_at + 3.01);
            assert_eq!(wife.label(wife.alt_at - 0.1), "GF");
            assert_eq!(wife.label(wife.alt_at + 0.1), "WIFE");

            // Rows closest to me: children first, then dog/cat, then
            // classmates/friends.  All rows are measured from my baseline.
            let dog = p.actors.iter().find(|a| a.role == Role::Dog).expect("dog");
            assert!((dog.base + 1.0).abs() < 0.01);
            let dad = p.actors.iter().find(|a| a.label == "DAD").expect("dad");
            let mom = p.actors.iter().find(|a| a.label == "MOM").expect("mom");
            let bro = p.actors.iter().find(|a| a.label == "BRO").expect("bro");
            assert!((mom.base - dad.base - 1.0).abs() < 0.01);
            assert!((bro.base - mom.base - 1.0).abs() < 0.01);
            for a in [dad, mom, bro] {
                assert!(a.yield_at <= first_child - 1.0);
                assert!((a.yield_by - 2.0).abs() < 0.01);
            }
            let first_child = p
                .actors
                .iter()
                .filter(|a| a.role == Role::Child)
                .map(|a| a.birth)
                .fold(f32::INFINITY, f32::min);
            for a in p.actors.iter().filter(|a| a.role == Role::Friend) {
                if a.label == "PAL" || a.label == "UNI" {
                    assert!((a.base - 1.0).abs() < 0.01);
                    assert!(a.death < first_child);
                }
            }
            for a in p.actors.iter().filter(|a| a.role == Role::Grandchild) {
                assert!(matches!(a.anchor, Anchor::Child(_)));
                assert!((a.base.abs() - 1.0).abs() < 0.01);
                if let Anchor::Child(parent) = a.anchor {
                    let parent_base = p.actors[parent].base;
                    assert_eq!(a.base > 0.0, parent_base > 0.0);
                }
            }
            let cat = p.actors.iter().find(|a| a.role == Role::Cat).expect("cat");
            assert!((cat.birth - first_child).abs() < 0.01);
            assert!(matches!(cat.anchor, Anchor::Child(_)));
            let kids: Vec<f32> = p
                .actors
                .iter()
                .filter(|a| a.role == Role::Child)
                .map(|a| a.base)
                .collect();
            assert!((kids[0] - 2.0).abs() < 0.01);
            assert!((kids[1] + 1.0).abs() < 0.01);
            assert!((kids[2] + 2.0).abs() < 0.01);
            for a in &p.actors {
                assert_eq!(a.amp, 0.0, "{} should follow the shared track", a.label);
            }
        }
    }

    #[test]
    fn beams_grow_with_the_life_and_keep_early_dots() {
        let p = Plan::generate(7, 0);
        let me = p.actors.iter().find(|a| a.role == Role::Me).unwrap();
        let young = beam_span(me, 20.0);
        let old = beam_span(me, 90.0);
        assert!(old > 60.0, "an old life should keep a long beam: {old}");
        assert!(old > young, "the beam should grow with the life");
        assert!(90.0 - old > 5.0, "early life should stay as dots");
        assert!((90.0 - old) <= EARLY_DOTS_MAX + 0.01);
        let friend = p.actors.iter().find(|a| a.role == Role::Friend).unwrap();
        assert!(
            beam_span(friend, 90.0) > 40.0,
            "friends should also get a longer thread"
        );
    }

    #[test]
    fn family_labels_carry_their_own_ages() {
        for cycle in 0..5 {
            let p = Plan::generate(7, cycle);
            let find = |label: &str| p.actors.iter().find(|a| a.label == label).unwrap();

            let me = find("ME");
            assert_eq!(me.display_label(0.0), "ME 0");
            assert_eq!(me.display_label(40.0), "ME 40");

            let gpa = find("GPA");
            let gma = find("GMA");
            let dad = find("DAD");
            let mom = find("MOM");
            assert!((58.0..=70.0).contains(&gpa.age_offset));
            assert!((56.0..=68.0).contains(&gma.age_offset));
            assert!((25.0..=33.0).contains(&dad.age_offset));
            assert!((23.0..=31.0).contains(&mom.age_offset));
            assert!(gpa.age_offset > dad.age_offset + 20.0);
            assert!(gma.age_offset > mom.age_offset + 20.0);
            assert_eq!(
                gpa.display_label(gpa.death),
                format!("GPA {}", (gpa.age_offset + gpa.death).round() as i32),
                "the end label should keep the final age"
            );

            // Born-later relatives count from their own birth.
            let kid1 = find("KID1");
            assert_eq!(kid1.display_label(kid1.birth), "KID1 0");
            assert_eq!(kid1.display_label(kid1.birth + 7.0), "KID1 7");
            let grand1 = find("GRAND1");
            assert_eq!(grand1.display_label(grand1.birth), "GRAND1 0");
            let cat = p.actors.iter().find(|a| a.role == Role::Cat).unwrap();
            assert_eq!(cat.display_label(cat.birth), "CAT 0");
            assert_eq!(
                cat.display_label(cat.death),
                format!("CAT {}", (cat.death - cat.birth).round() as i32),
                "the end label should keep the pet's final age"
            );
            let dog = p.actors.iter().find(|a| a.role == Role::Dog).unwrap();
            assert_eq!(dog.display_label(dog.birth + 5.0), "DOG 5");
            assert_eq!(
                dog.display_label(dog.death),
                format!("DOG {}", (dog.death - dog.birth).round() as i32)
            );

            // GF/WIFE is a peer, not a newborn.
            let wife = find("GF");
            assert!((-2.0..=2.0).contains(&wife.age_offset));
            assert!((38.0..=42.0).contains(&wife.age_at(40.0)));

            // Non-relatives never get an age suffix.
            let friend = p.actors.iter().find(|a| a.role == Role::Friend).unwrap();
            assert_eq!(friend.display_label(50.0), friend.label(50.0));
        }
    }

    #[test]
    fn me_finale_timing_and_draw() {
        assert_eq!(me_finale_elapsed(80.0, 80.0, 100.0), Some(0.0));
        assert_eq!(me_finale_elapsed(79.9, 80.0, 100.0), None);
        assert!((me_finale_elapsed(81.0, 80.0, 100.0).unwrap() - 1.0).abs() < 0.001);
        assert!(me_finale_elapsed(84.1, 80.0, 100.0).is_some());
        assert_eq!(me_finale_elapsed(86.1, 80.0, 100.0), None);

        let mut s = LifeScene::new(7);
        s.resize(100, 30);
        let me = s
            .plan
            .actors
            .iter()
            .find(|a| a.role == Role::Me)
            .expect("me");
        assert!((75.0..=85.0).contains(&me.death));
        let mut canvas = Canvas::new(100, 30);
        for unicode in [false, true] {
            for real in [0.0, 0.25, 0.8, 1.6, 2.5, 3.9, 5.5] {
                let age = me.death + real * 120.0 / 100.0;
                s.draw_me_finale(&mut canvas, age, real, 120.0, unicode, 1.0);
            }
            // The permanent halo after the comet has arrived.
            s.draw_me_finale(&mut canvas, 100.0, 20.0, 120.0, unicode, 1.0);
        }
    }

    #[test]
    fn human_and_animal_glyphs_are_distinct() {
        assert_eq!(
            actor_glyph(Role::Me, false),
            actor_glyph(Role::Friend, false)
        );
        assert_eq!(actor_glyph(Role::Me, true), actor_glyph(Role::Child, true));
        assert_ne!(actor_glyph(Role::Dog, false), actor_glyph(Role::Me, false));
        assert_ne!(actor_glyph(Role::Cat, false), actor_glyph(Role::Dog, false));
        assert_eq!(
            star_glyph(Role::Family, false),
            star_glyph(Role::Brother, false)
        );
        assert_eq!(star_glyph(Role::Me, false), '*');
        assert_eq!(star_glyph(Role::Dog, false), '+');
        assert_eq!(star_glyph(Role::Cat, false), '+');
    }

    #[test]
    fn core_people_never_change_track() {
        for cycle in 0..5 {
            let p = Plan::generate(7, cycle);
            for (i, actor) in p.actors.iter().enumerate() {
                let fixed = actor.role == Role::Me
                    || actor.role == Role::Spouse
                    || (actor.role == Role::Family
                        && (actor.label == "GPA" || actor.label == "GMA"));
                if !fixed {
                    continue;
                }
                let mut lo = f32::INFINITY;
                let mut hi = f32::NEG_INFINITY;
                let end = actor.death.min(99.0) - 0.01;
                let mut age = if actor.role == Role::Spouse {
                    actor.birth + actor.entry_dur + 0.1
                } else {
                    0.0
                };
                while age <= end {
                    let pos = p.positions(age, 120, 36);
                    if let Some(Some(p)) = pos.get(i) {
                        lo = lo.min(p.y);
                        hi = hi.max(p.y);
                    }
                    age += 0.25;
                }
                assert!(hi - lo <= 0.01, "{} moves {} rows", actor.label, hi - lo);
            }
        }
    }

    #[test]
    fn end_marker_blinks_once_per_second() {
        let p = Plan::generate(7, 0);
        let a = p
            .actors
            .iter()
            .find(|a| a.end == EndKind::Star)
            .expect("star");
        for k in 0..20 {
            let sec = k as f32;
            assert!(death_marker_visible(sec, a), "visible at {sec}s");
            assert!(
                !death_marker_visible(sec + 0.5, a),
                "hidden at {}s",
                sec + 0.5
            );
            assert!(death_marker_visible(sec + 0.25, a));
            assert!(!death_marker_visible(sec + 0.75, a));
        }
        // One rising edge per second, no more.
        let mut rises = 0;
        let mut last = death_marker_visible(0.0, a);
        for i in 1..500 {
            let now = death_marker_visible(i as f32 * 0.01, a);
            if now && !last {
                rises += 1;
            }
            last = now;
        }
        assert!((4..=5).contains(&rises), "rises {rises}");
    }

    #[test]
    fn friends_stay_outside_minor_children_and_pets() {
        for seed in [7u64, 0x7e51_a017, 42] {
            let p = Plan::generate(seed, 0);
            for step in 0..=400 {
                let age = step as f32 * 0.25;
                let g = active_guards(&p, age);
                for (i, a) in p.actors.iter().enumerate() {
                    if a.role != Role::Friend || !active_at(a, age) {
                        continue;
                    }
                    let lane = current_lane(&p, i, age).unwrap().round() as i32;
                    assert!(
                        !lane_violates(lane, g),
                        "{seed}: {} at lane {lane} is inside the minor-child band (pos {}, neg {}) at {age}",
                        a.label,
                        g.pos,
                        g.neg
                    );
                    for (j, b) in p.actors.iter().enumerate() {
                        if i == j || !active_at(b, age) {
                            continue;
                        }
                        let occupied = match b.role {
                            Role::Child => age < b.birth + KID_MINOR_AGE,
                            Role::Cat | Role::Dog | Role::Grandchild => true,
                            _ => false,
                        };
                        if !occupied {
                            continue;
                        }
                        let bl = current_lane(&p, j, age).unwrap().round() as i32;
                        assert_ne!(
                            lane, bl,
                            "{seed}: {} takes {}'s row at {age}",
                            a.label, b.label
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn dog_stays_next_to_me_until_it_dies() {
        for seed in [7u64, 0x7e51_a017, 42] {
            let p = Plan::generate(seed, 0);
            let dog_i = p.actors.iter().position(|a| a.role == Role::Dog).unwrap();
            let dog = p.actors[dog_i];
            let me_i = p.actors.iter().position(|a| a.role == Role::Me).unwrap();
            let mut age = 0.5;
            while age < dog.death - 0.1 {
                if active_at(&dog, age) {
                    assert!(
                        (current_lane(&p, dog_i, age).unwrap() + 1.0).abs() < 0.01,
                        "{seed}: dog left the row next to me at {age}"
                    );
                    for (j, a) in p.actors.iter().enumerate() {
                        if j == dog_i || j == me_i || !active_at(a, age) {
                            continue;
                        }
                        let lane = current_lane(&p, j, age).unwrap();
                        assert!(
                            (lane + 1.0).abs() > 0.01,
                            "{seed}: {} took the dog's row at {age}",
                            a.label
                        );
                    }
                }
                age += 0.25;
            }
        }
    }

    #[test]
    fn parents_stay_adjacent_and_brother_follows() {
        for seed in [7u64, 0x7e51_a017, 42] {
            let p = Plan::generate(seed, 0);
            let find = |label: &str| p.actors.iter().position(|a| a.label == label).unwrap();
            let (d, m, b) = (find("DAD"), find("MOM"), find("BRO"));
            let first_child = p
                .actors
                .iter()
                .filter(|a| a.role == Role::Child)
                .map(|a| a.birth)
                .fold(f32::INFINITY, f32::min);
            let end = p.actors[d].death.min(p.actors[m].death);
            let mut age = 0.5;
            while age < end - 0.1 {
                let dl = current_lane(&p, d, age).unwrap();
                let ml = current_lane(&p, m, age).unwrap();
                let bl = current_lane(&p, b, age).unwrap();
                assert!(
                    (ml - dl - 1.0).abs() < 0.01,
                    "{seed}: DAD/MOM not adjacent at {age}: {dl}/{ml}"
                );
                assert!(
                    (bl - ml - 1.0).abs() < 0.01,
                    "{seed}: BRO not directly below MOM at {age}: {ml}/{bl}"
                );
                age += 0.25;
            }
            // The block really steps two rows back for the first child.
            if active_at(&p.actors[d], first_child + 0.5) {
                assert!((current_lane(&p, d, first_child + 0.5).unwrap() - 5.0).abs() < 0.01);
                assert!((current_lane(&p, b, first_child + 0.5).unwrap() - 7.0).abs() < 0.01);
            }
        }
    }

    #[test]
    fn children_stay_close_until_they_grow_up() {
        for seed in [7u64, 0x7e51_a017, 42] {
            let p = Plan::generate(seed, 0);
            for (i, a) in p.actors.iter().enumerate() {
                if a.role != Role::Child {
                    continue;
                }
                let mut age = a.birth + 0.1;
                while age < a.birth + KID_MINOR_AGE - 0.1 {
                    if active_at(a, age) {
                        let lane = current_lane(&p, i, age).unwrap();
                        assert!(
                            lane.abs() <= 4.01,
                            "{seed}: {} drifted to {lane} while still a minor at {age}",
                            a.label
                        );
                        for (j, friend) in p.actors.iter().enumerate() {
                            if friend.role != Role::Friend || !active_at(friend, age) {
                                continue;
                            }
                            let fl = current_lane(&p, j, age).unwrap();
                            assert!(
                                (fl - lane).abs() > 0.01,
                                "{seed}: {} shares {}'s row at {age}",
                                friend.label,
                                a.label
                            );
                        }
                    }
                    age += 0.5;
                }
            }
            // KID1 is directly beside WIFE.
            let kid1 = p.actors.iter().position(|a| a.label == "KID1").unwrap();
            let wife = p
                .actors
                .iter()
                .position(|a| a.role == Role::Spouse)
                .unwrap();
            let age = p.actors[kid1].birth + 1.0;
            let kl = current_lane(&p, kid1, age).unwrap();
            let wl = current_lane(&p, wife, age).unwrap();
            assert!(
                (kl - wl - 1.0).abs() < 0.01,
                "{seed}: KID1 ({kl}) is not beside WIFE ({wl})"
            );

            // Gapless stack: KID3 -2 / KID2 -1 / ME 0 / WIFE +1 / KID1 +2 /
            // CAT +3.  No empty rows between any of them.
            let kid2 = p.actors.iter().position(|a| a.label == "KID2").unwrap();
            let kid3 = p.actors.iter().position(|a| a.label == "KID3").unwrap();
            let me = p.actors.iter().position(|a| a.role == Role::Me).unwrap();
            let cat = p.actors.iter().position(|a| a.role == Role::Cat).unwrap();
            let age = p.actors[kid3].birth + 1.0;
            let k1 = current_lane(&p, kid1, age).unwrap();
            let k2 = current_lane(&p, kid2, age).unwrap();
            let k3 = current_lane(&p, kid3, age).unwrap();
            let ml = current_lane(&p, me, age).unwrap();
            let cl = current_lane(&p, cat, age).unwrap();
            assert!(
                (k2 - ml + 1.0).abs() < 0.01,
                "{seed}: KID2 ({k2}) is not directly above ME ({ml})"
            );
            assert!(
                (k3 - k2 + 1.0).abs() < 0.01,
                "{seed}: KID3 ({k3}) is not directly above KID2 ({k2})"
            );
            assert!(
                (cl - k1 - 1.0).abs() < 0.01,
                "{seed}: CAT ({cl}) is not directly below KID1 ({k1})"
            );
        }
    }

    #[test]
    fn ended_rows_are_reused_by_later_lives() {
        for cycle in 0..10 {
            let p = Plan::generate(7, cycle);
            let kid2_i = p.actors.iter().position(|a| a.label == "KID2").unwrap();
            let kid3_i = p.actors.iter().position(|a| a.label == "KID3").unwrap();
            let kid2 = p.actors[kid2_i];
            // KID2 takes the -1 row the dog leaves behind, which is also the
            // row closest to ME.  KID3 then stacks above KID2 at -2.
            let dog_i = p.actors.iter().position(|a| a.role == Role::Dog).unwrap();
            let dog = p.actors[dog_i];
            let kid2_lane = lane_offset(&p.actors, kid2_i).round() as i32;
            let dog_lane = lane_offset(&p.actors, dog_i).round() as i32;
            assert_eq!(
                kid2_lane, dog_lane,
                "cycle {cycle}: KID2 should take the dog's freed -1 row"
            );
            assert!(dog.death < kid2.birth);
            assert_eq!(
                lane_offset(&p.actors, kid3_i).round() as i32,
                -2,
                "cycle {cycle}: KID3 should stack at -2 above KID2"
            );

            // The school friend occupies +1, then WIFE moves into the same
            // row after that friend has left the timeline.
            let pal_i = p
                .actors
                .iter()
                .enumerate()
                .find(|(i, a)| {
                    a.label == "PAL"
                        && (current_lane(&p, *i, a.birth + a.entry_dur + 0.1).unwrap() - 1.0).abs()
                            < 0.01
                })
                .map(|(i, _)| i)
                .expect("a PAL in the +1 row");
            let wife_i = p
                .actors
                .iter()
                .position(|a| a.role == Role::Spouse)
                .unwrap();
            assert!(p.actors[pal_i].death < p.actors[wife_i].birth);
            assert!((lane_offset(&p.actors, wife_i) - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn active_friends_share_one_row() {
        for seed in [7u64, 0x7e51_a017, 42] {
            let p = Plan::generate(seed, 0);
            for step in 0..=396 {
                let age = step as f32 * 0.25;
                let lanes: Vec<f32> = p
                    .actors
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| a.role == Role::Friend && active_at(a, age))
                    .filter_map(|(i, _)| current_lane(&p, i, age))
                    .collect();
                if lanes.len() > 1 {
                    let first = lanes[0].round() as i32;
                    for lane in &lanes[1..] {
                        assert_eq!(
                            lane.round() as i32,
                            first,
                            "{seed}: friends split across rows at {age}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn minor_kids_and_cat_leave_no_blank_rows() {
        for seed in [7u64, 0x7e51_a017, 42] {
            let p = Plan::generate(seed, 0);
            let find = |label: &str| p.actors.iter().position(|a| a.label == label).unwrap();
            let me = p.actors.iter().position(|a| a.role == Role::Me).unwrap();
            let wife = p
                .actors
                .iter()
                .position(|a| a.role == Role::Spouse)
                .unwrap();
            let cat = p.actors.iter().position(|a| a.role == Role::Cat).unwrap();
            let kid1 = find("KID1");
            let kid2 = find("KID2");
            let kid3 = find("KID3");

            // Before KID2, ME/WIFE/KID1/CAT are already consecutive.
            // After KID2, KID2 fills -1.  After KID3, it stacks at -2.
            for (age, ids) in [
                (
                    p.actors[kid1].birth + 1.0,
                    vec![("ME", me), ("WIFE", wife), ("KID1", kid1), ("CAT", cat)],
                ),
                (
                    p.actors[kid2].birth + 1.0,
                    vec![
                        ("KID2", kid2),
                        ("ME", me),
                        ("WIFE", wife),
                        ("KID1", kid1),
                        ("CAT", cat),
                    ],
                ),
                (
                    p.actors[kid3].birth + 1.0,
                    vec![
                        ("KID3", kid3),
                        ("KID2", kid2),
                        ("ME", me),
                        ("WIFE", wife),
                        ("KID1", kid1),
                        ("CAT", cat),
                    ],
                ),
            ] {
                for h in [30usize, 36, 40, 46] {
                    let ps = p.positions(age, 120, h);
                    let mut rows: Vec<(&str, isize)> = ids
                        .iter()
                        .map(|(name, i)| (*name, ps[*i].unwrap().y.round() as isize))
                        .collect();
                    rows.sort_by_key(|(_, row)| *row);
                    for pair in rows.windows(2) {
                        assert_eq!(
                            pair[1].1 - pair[0].1,
                            1,
                            "{seed} at {age}, h={h}: blank row between {} and {} ({rows:?})",
                            pair[0].0,
                            pair[1].0
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn no_two_lives_share_a_row() {
        for seed in [7u64, 0x7e51_a017, 42] {
            for cycle in 0..3 {
                let p = Plan::generate(seed, cycle);
                for step in 0..=396 {
                    let age = step as f32 * 0.25;
                    let mut used: Vec<(i32, &str, bool)> = Vec::new();
                    for (i, a) in p.actors.iter().enumerate() {
                        let Some(lane) = current_lane(&p, i, age) else {
                            continue;
                        };
                        let row = lane.round() as i32;
                        let is_friend = a.role == Role::Friend;
                        if let Some((_, other, other_friend)) =
                            used.iter().find(|(r, _, _)| *r == row)
                        {
                            if !(is_friend && *other_friend) {
                                panic!(
                                    "{seed}/{cycle} at {age}: {} and {} share row {row}",
                                    a.label, other
                                );
                            }
                        }
                        used.push((row, a.label, is_friend));
                    }
                }
            }
        }
    }

    #[test]
    fn dead_stars_stack_at_the_top_with_one_blank_row() {
        assert!((star_top_y(0, 36) - 3.0).abs() < 0.01);
        assert!((star_top_y(1, 36) - 5.0).abs() < 0.01);
        assert!((star_top_y(2, 36) - 7.0).abs() < 0.01);
    }

    #[test]
    fn ends_are_ordered() {
        let p = Plan::generate(7, 0);
        for a in &p.actors {
            assert!(a.birth <= 100.0);
            if a.end == EndKind::Star {
                assert!(
                    (10.0..=95.0).contains(&a.death),
                    "{} died {}",
                    a.label,
                    a.death
                );
            }
        }
    }

    #[test]
    fn positions_stay_in_frame() {
        let mut s = LifeScene::new(7);
        s.resize(100, 30);
        for step in 0..600 {
            let age = step as f32 / 600.0 * 99.9;
            let ps = s.plan.positions(age, 100, 30);
            for p in ps.into_iter().flatten() {
                assert!(p.x >= 0.0 && p.x <= 100.0, "x {}", p.x);
                // Entering / exiting characters are allowed to slip outside
                // the frame; they should still be finite and bounded.
                assert!(p.y.is_finite() && p.y > -80.0 && p.y < 80.0, "y {}", p.y);
            }
        }
    }
}
