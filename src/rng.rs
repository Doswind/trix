//! Tiny deterministic RNG (xoshiro256++) plus cheap spatial hashing.
//!
//! We deliberately avoid pulling in `rand`: scenes generate millions of
//! numbers per minute and only need uniform floats/indices.

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut sm = seed ^ 0x9e37_79b9_7f4a_7c15;
        let mut next = || {
            sm = sm.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = sm;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        };
        let s = [next(), next(), next(), next()];
        Rng { s }
    }

    /// Seed from wall clock and pid, adequate for eye candy.
    pub fn from_entropy() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x5eed);
        let pid = std::process::id() as u64;
        Self::new(nanos ^ (pid << 32) ^ 0xa5a5_5a5a_dead_beef)
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Uniform in [0, 1).
    #[inline]
    pub fn f32(&mut self) -> f32 {
        ((self.next_u64() >> 40) as f32) * (1.0 / 16_777_216.0)
    }

    /// Uniform in [a, b).
    #[inline]
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f32()
    }

    /// Uniform integer in 0..n (n must be > 0).
    #[inline]
    pub fn below(&mut self, n: usize) -> usize {
        if n <= 1 {
            return 0;
        }
        (self.next_u64() % n as u64) as usize
    }

    #[inline]
    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }

    #[inline]
    pub fn sign(&mut self) -> f32 {
        if self.next_u64() & 1 == 0 {
            1.0
        } else {
            -1.0
        }
    }
}

/// Fast deterministic 2D hash (good enough for texture/noise).
#[inline]
pub fn hash2(x: i64, y: i64, seed: u64) -> u32 {
    let mut h = (x as u64)
        .wrapping_mul(0x9e37_79b9_7f4a_7c15)
        .wrapping_add((y as u64).wrapping_mul(0xc2b2_ae3d_27d4_eb4f))
        .wrapping_add(seed.wrapping_mul(0x1656_67b1_9e37_79f9));
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    (h >> 32) as u32
}

/// Hash to a float in [0, 1).
#[inline]
pub fn hash01(x: i64, y: i64, seed: u64) -> f32 {
    (hash2(x, y, seed) as f32) * (1.0 / 4_294_967_296.0)
}

/// Smooth 2D value noise in [0, 1].
pub fn value_noise(x: f32, y: f32, seed: u64) -> f32 {
    let xi = x.floor() as i64;
    let yi = y.floor() as i64;
    let xf = x - xi as f32;
    let yf = y - yi as f32;
    let u = xf * xf * (3.0 - 2.0 * xf);
    let v = yf * yf * (3.0 - 2.0 * yf);
    let a = hash01(xi, yi, seed);
    let b = hash01(xi + 1, yi, seed);
    let c = hash01(xi, yi + 1, seed);
    let d = hash01(xi + 1, yi + 1, seed);
    let ab = a + (b - a) * u;
    let cd = c + (d - c) * u;
    ab + (cd - ab) * v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn floats_in_range() {
        let mut r = Rng::new(7);
        for _ in 0..10_000 {
            let f = r.f32();
            assert!((0.0..1.0).contains(&f));
            let x = r.range(-3.0, 9.0);
            assert!((-3.0..9.0).contains(&x));
        }
    }

    #[test]
    fn below_bounds() {
        let mut r = Rng::new(3);
        for _ in 0..1000 {
            assert!(r.below(7) < 7);
            assert_eq!(r.below(1), 0);
        }
    }

    #[test]
    fn noise_range() {
        for i in 0..500 {
            let v = value_noise(i as f32 * 0.37, i as f32 * 0.11, 9);
            assert!((0.0..=1.0).contains(&v), "{v}");
        }
    }
}
