//! Delivery-rate driven power tiers – the signature mechanic.
//!
//! Every "fed" block (weapons, thrusters, mining lasers) measures how fast its
//! primary feed arrives. A smoothed rate is compared against the `demand` of
//! each growth stage; sustained supply raises the tier (and grows the block's
//! footprint), starvation decays it. A block whose local buffer is nearly full
//! counts as saturated, so a well-stocked weapon idles at its cap and only
//! sustained throughput keeps it there once it starts firing.

use gf_content::GrowthStage;
use serde::{Deserialize, Serialize};

/// Smoothing time constant (seconds) for the delivery-rate average.
pub const FEED_TAU: f32 = 2.0;
/// Seconds the target tier must exceed the current tier before growing.
pub const GROW_HOLD: f32 = 1.0;
/// Seconds of under-supply before shrinking a tier.
pub const SHRINK_HOLD: f32 = 2.0;
/// Hysteresis: fall below this fraction of the current stage's demand to shrink.
pub const SHRINK_FRACTION: f32 = 0.75;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Feed {
    /// Smoothed delivery rate (units/s, or kW for power-fed blocks).
    pub rate: f32,
    /// Current tier, 1-based.
    pub tier: u8,
    /// Maximum tier allowed by the free growth zone.
    pub cap: u8,
    pub grow_timer: f32,
    pub shrink_timer: f32,
    /// Units delivered during the current tick.
    pub delivered: f32,
}

impl Default for Feed {
    fn default() -> Self {
        Self {
            rate: 0.0,
            tier: 1,
            cap: 1,
            grow_timer: 0.0,
            shrink_timer: 0.0,
            delivered: 0.0,
        }
    }
}

impl Feed {
    /// Advance one tick. Returns `Some(new_tier)` when the tier changed.
    pub fn update(&mut self, stages: &[GrowthStage], dt: f32, saturated: bool) -> Option<u8> {
        let cap = self.cap.clamp(1, stages.len() as u8);
        let mut input = self.delivered / dt;
        if saturated {
            let next = (self.tier as usize).min(cap as usize - 1);
            input = input.max(stages[next].demand);
        }
        self.delivered = 0.0;
        let alpha = dt / (FEED_TAU + dt);
        self.rate += (input - self.rate) * alpha;

        let mut target = 1u8;
        for (i, s) in stages.iter().enumerate().take(cap as usize) {
            if self.rate >= s.demand * 0.999 {
                target = i as u8 + 1;
            }
        }
        let old = self.tier;
        if self.tier > cap {
            self.tier = cap;
        } else if target > self.tier {
            self.shrink_timer = 0.0;
            self.grow_timer += dt;
            if self.grow_timer >= GROW_HOLD {
                self.tier += 1;
                self.grow_timer = 0.0;
            }
        } else if self.tier > 1 && self.rate < stages[self.tier as usize - 1].demand * SHRINK_FRACTION {
            self.grow_timer = 0.0;
            self.shrink_timer += dt;
            if self.shrink_timer >= SHRINK_HOLD {
                self.tier -= 1;
                self.shrink_timer = 0.0;
            }
        } else {
            self.grow_timer = 0.0;
            self.shrink_timer = 0.0;
        }
        (self.tier != old).then_some(self.tier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stages() -> Vec<GrowthStage> {
        [(1, 1.0), (2, 2.0), (3, 4.0)]
            .iter()
            .map(|&(s, d)| GrowthStage { size: (s, s), demand: d })
            .collect()
    }

    #[test]
    fn grows_with_supply_and_respects_cap() {
        let st = stages();
        let mut f = Feed { cap: 2, ..Default::default() };
        let dt = 1.0 / 30.0;
        for _ in 0..30 * 20 {
            f.delivered = 10.0 * dt;
            f.update(&st, dt, false);
        }
        assert_eq!(f.tier, 2, "capped at 2");
        f.cap = 3;
        for _ in 0..30 * 5 {
            f.delivered = 10.0 * dt;
            f.update(&st, dt, false);
        }
        assert_eq!(f.tier, 3);
    }

    #[test]
    fn decays_when_starved() {
        let st = stages();
        let mut f = Feed { cap: 3, tier: 3, rate: 4.0, ..Default::default() };
        let dt = 1.0 / 30.0;
        for _ in 0..30 * 30 {
            f.update(&st, dt, false);
        }
        assert_eq!(f.tier, 1);
    }

    #[test]
    fn saturation_counts_as_supplied() {
        let st = stages();
        let mut f = Feed { cap: 3, ..Default::default() };
        let dt = 1.0 / 30.0;
        for _ in 0..30 * 30 {
            f.update(&st, dt, true);
        }
        assert_eq!(f.tier, 3);
    }
}
