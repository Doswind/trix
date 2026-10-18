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

struct Plan {
    actors: Vec<Actor>,
    events: Vec<Event>,
    wife_at: f32,
    first_child: f32,
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

        // Love, marriage and three children.  The three- and four-year gaps
        // are kept, so the birth order tells a coherent story.
        let spouse_birth = rng.range(25.0, 34.0);
        let wife_at = spouse_birth + 3.2 + rng.range(1.0, 2.0);
        let wife_death = rng.range((me_death + 2.0).max(75.0), 90.0);
        let first_child = wife_at + rng.range(1.0, 2.0);
        let second_child = first_child + rng.range(3.0, 4.0);
        let third_child = second_child + rng.range(3.0, 4.0);
        let child_births = [first_child, second_child, third_child];

        let dog_birth = rng.range(1.0, 3.0);
        let dog_death = dog_birth + rng.range(10.0, 12.0);
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
        dad.amp = 0.0;
        dad.phase = 1.3;
        dad.trail_len = 36.0;
        dad.yield_at = first_child;
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
        mom.amp = 0.0;
        mom.phase = 3.4;
        mom.trail_len = 36.0;
        mom.yield_at = first_child;
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
        bro.amp = 0.0;
        bro.phase = 4.0;
        bro.freq = 0.27;
        bro.yield_at = first_child;
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
        wife.amp = 0.0;
        wife.phase = 1.1;
        wife.freq = 0.30;
        wife.entry_side = if rng.chance(0.5) { -1.0 } else { 1.0 };
        wife.entry_dur = 3.2;
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
        let kid_rows = [2.0f32, -2.0, -4.0];
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
            child.amp = 0.0;
            child.phase = i as f32 * 1.7 + 0.5;
            child.freq = 0.31 + i as f32 * 0.04;
            child.diverge_at = birth + 20.0;
            child.diverge_dur = 9.0;
            child.trail_len = 24.0;

            if i == stay_child {
                child.diverge = if rng.chance(0.5) { 4.5 } else { -4.5 };
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
        dog.amp = 0.0;
        dog.phase = rng.range(0.0, 2.0 * PI);
        dog.freq = 0.30;
        dog.trail_len = 14.0;
        actors.push(dog);

        let cat_child = 0;
        let cat_side = if actors[child_actor[cat_child]].base > 0.0 {
            2.0
        } else {
            -2.0
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
                age: first_child + 1.0,
                text: "GRANDPARENTS STEP BACK FOR THE CHILD",
                color: C_DAD,
            },
            Event {
                age: first_child + 1.5,
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
            first_child,
        }
    }

    fn positions(&self, age: f32, w: usize, h: usize) -> Vec<Option<Pos>> {
        let mut out: Vec<Option<Pos>> = vec![None; self.actors.len()];
        let hh = h as f32;
        let main = main_y(age, h);
        let x0 = 2.0;
        let x1 = (w as f32 - 3.0).max(x0 + 1.0);
        let x = x0 + (age / 100.0).clamp(0.0, 1.0) * (x1 - x0);
        let scale = (hh / 36.0).clamp(0.55, 1.35);
        let child_around = self.actors.iter().any(|c| {
            if c.role != Role::Child {
                return false;
            }
            let until = if c.end == EndKind::Exit {
                c.death + c.exit_dur
            } else {
                c.birth + 20.0 + c.diverge_dur
            };
            age >= c.birth && age < until
        });
        let child_started = self.first_child <= age;
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
            let mut base = a.base;
            if a.role == Role::Friend && (a.label == "WORK" || a.label == "FRIEND") {
                base = if child_started && child_around {
                    -7.0
                } else {
                    2.0
                };
            }
            if age > a.yield_at {
                let p = smoothstep(((age - a.yield_at) / 2.0).clamp(0.0, 1.0));
                base += a.yield_by * p;
            }
            let mut y = anchor + (base + a.amp * (age * a.freq + a.phase).sin()) * scale;

            if age > a.diverge_at {
                let p = smoothstep((age - a.diverge_at) / a.diverge_dur.max(0.1));
                y += a.diverge * scale * p;
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

    fn latest_event(&self, age: f32) -> Option<Event> {
        self.events
            .iter()
            .rev()
            .find(|e| age >= e.age && age - e.age <= 4.5)
            .copied()
    }
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
    /// a memory of the path instead of a hard line.
    fn draw_history(&self, canvas: &mut Canvas, age: f32, unicode: bool, fade: f32) {
        for (i, actor) in self.plan.actors.iter().enumerate() {
            let start = actor.birth.max(0.0);
            let end = match actor.end {
                EndKind::Star | EndKind::Exit => actor.death.min(age),
                EndKind::Continue => age.min(100.0),
            };
            let old_end = (end - actor.trail_len).max(start);
            if old_end <= start {
                continue;
            }
            let span = old_end - start;
            let steps = ((span / 1.2).ceil() as usize).clamp(6, 96);
            for k in 0..=steps {
                let f = k as f32 / steps as f32;
                let a = start + span * f;
                let past = self.plan.positions(a, self.w, self.h);
                if let Some(Some(p)) = past.get(i) {
                    let x = p.x.round() as isize;
                    let y = p.y.round() as isize;
                    let col = actor.color.scale((0.18 + f * 0.13) * fade);
                    canvas.set(x, y, if unicode { '·' } else { '.' }, col);
                    if k % 7 == 0 {
                        canvas.add_light(x, y, actor.color, 0.06 * fade, ' ');
                    }
                }
            }
        }
    }

    /// Recent life is drawn as a bright beam.  Early life is intentionally
    /// left as faint dots above; the beam is what carries the eye.
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
            let beam_len = actor.trail_len.min(beam_age - actor.birth + 0.01).max(0.1);
            let steps = ((beam_len * 2.5).ceil() as usize).clamp(8, 36);
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
                    let col = actor.color.scale((0.30 + f * 0.52) * fade * dim);
                    if let Some((px, py)) = prev {
                        let dx = x - px;
                        let dy = y - py;
                        let ch = beam_glyph(dx, dy, f > 0.55, unicode);
                        canvas.line(px, py, x, y, ch, col);
                        if dx.abs() >= dy.abs() * 2 {
                            canvas.line(
                                px,
                                py + 1,
                                x,
                                y + 1,
                                if unicode { '~' } else { '-' },
                                col.scale(0.30),
                            );
                        }
                    } else {
                        canvas.set(x, y, if unicode { '·' } else { '.' }, col);
                    }
                    if k % 5 == 0 && f > 0.15 {
                        sparkles.push((x, y, actor.color.scale((0.55 + f * 0.40) * fade * dim)));
                    }
                    if k % 5 == 0 {
                        canvas.add_light(
                            x,
                            y,
                            actor.color,
                            0.20 * fade * if memory_end.is_some() { 0.55 } else { 1.0 },
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
            let (pos, reveal) = if actor.end == EndKind::Star && age >= actor.death {
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
            let label = actor.label(age);
            let len = label.len() as isize;
            let mut row = py.round() as isize;
            let hi = (self.h as isize - 5).max(1);
            if row > hi {
                row = hi;
            }
            let is_star = actor.end == EndKind::Star && age >= actor.death;
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
                    canvas.put_str(lx, row, label, actor.color.scale(0.74 * fade * reveal));
                    placed = true;
                    break;
                }
            }
            if !placed {
                canvas.put_str(anchor, row, label, actor.color.scale(0.74 * fade * reveal));
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
                let x = dp.x.round() as isize + 2;
                let y = dp.y.round() as isize;
                if y >= 1 && y <= self.h as isize - 5 {
                    canvas.put_str(x, y, actor.label(age), actor.color.scale(0.58 * fade));
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

    fn draw_hud(&self, canvas: &mut Canvas, age: f32, cycle_secs: f32, unicode: bool) {
        let sep = if unicode { "·" } else { "|" };
        canvas.put_str(2, 0, "LIFE // TRAJECTORIES", C_ME.scale(0.76));
        let status = format!("AGE {:.1} / 100  {sep}  CYCLE {:.0}s", age, cycle_secs);
        let sx = (self.w as isize - status.len() as isize - 2).max(2);
        canvas.put_str(sx, 0, &status, C_END.scale(0.62));

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
        self.draw_labels(canvas, &positions, age, cycle_secs, fade);
        self.draw_death_labels(canvas, age, t, fade);
        self.draw_hud(canvas, age, cycle_secs, unicode);
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

/// The end-point marker is alternated on/off in place.
fn death_marker_visible(t: f32, actor: &Actor) -> bool {
    (t * 4.0 * PI + actor.phase * 2.0).sin() > 0.0
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
            assert!(first_child >= wife.alt_at + 1.0);

            // Rows closest to me: children first, then dog/cat, then
            // classmates/friends.  All rows are measured from my baseline.
            let dog = p.actors.iter().find(|a| a.role == Role::Dog).expect("dog");
            assert!((dog.base + 1.0).abs() < 0.01);
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
            assert!((kids[1] + 2.0).abs() < 0.01);
            assert!((kids[2] + 4.0).abs() < 0.01);
            for a in &p.actors {
                assert_eq!(a.amp, 0.0, "{} should follow the shared track", a.label);
            }
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
    fn death_marker_alternates_visible_and_hidden() {
        let p = Plan::generate(7, 0);
        let a = p
            .actors
            .iter()
            .find(|a| a.end == EndKind::Star)
            .expect("star");
        let mut on = false;
        let mut off = false;
        for i in 0..120 {
            if death_marker_visible(i as f32 * 0.1, a) {
                on = true;
            } else {
                off = true;
            }
        }
        assert!(on && off, "end marker must alternate");
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
