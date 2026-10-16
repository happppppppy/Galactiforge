//! Seeded, platform-independent PRNG (SplitMix64). Every random decision in
//! the simulation draws from a stream owned by the world so runs are
//! reproducible from (seed, commands).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed ^ 0x9E37_79B9_7F4A_7C15 }
    }
    /// Derive an independent stream from a seed and a key (e.g. sector coords).
    pub fn derive(seed: u64, a: i64, b: i64) -> Self {
        let mut r = Rng::new(seed);
        r.state ^= (a as u64).wrapping_mul(0xA24B_AED4_963E_E407);
        r.next_u64();
        r.state ^= (b as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25);
        r.next_u64();
        r
    }
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in [0, 1).
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn range_f32(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f32()
    }
    pub fn range_i32(&mut self, lo: i32, hi_inclusive: i32) -> i32 {
        let span = (hi_inclusive - lo + 1).max(1) as u64;
        lo + (self.next_u64() % span) as i32
    }
    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }
    pub fn state(&self) -> u64 {
        self.state
    }
}
