//! Open world: seeded sector generation, streaming of the active area around
//! the player and a floating origin that keeps coordinates small.

use crate::body::{Body, BodyId, BodyKind, PIRATE_FACTION};
use crate::math::{Cell, Vec2};
use crate::rng::Rng;
use crate::world::{Event, World};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Sector edge length in cells (world units).
pub const SECTOR_SIZE: f32 = 400.0;
/// Sectors within this Chebyshev distance of the player are simulated.
pub const ACTIVE_RADIUS: i64 = 1;

pub type SectorKey = (i64, i64);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sectors {
    /// When false (tests / sandbox) no streaming or generation happens.
    pub enabled: bool,
    /// Global sector of the local origin (floating origin offset).
    pub origin: SectorKey,
    pub generated: BTreeSet<SectorKey>,
    /// Frozen bodies of inactive sectors; positions are relative to the
    /// sector centre.
    pub dormant: BTreeMap<SectorKey, Vec<Body>>,
}

impl Sectors {
    /// Global sector containing a local world position.
    pub fn sector_of(&self, p: Vec2) -> SectorKey {
        let sx = libm::floorf(p.x / SECTOR_SIZE + 0.5) as i64;
        let sy = libm::floorf(p.y / SECTOR_SIZE + 0.5) as i64;
        (sx + self.origin.0, sy + self.origin.1)
    }
    /// Local position of a global sector's centre.
    pub fn center_of(&self, k: SectorKey) -> Vec2 {
        Vec2::new((k.0 - self.origin.0) as f32 * SECTOR_SIZE, (k.1 - self.origin.1) as f32 * SECTOR_SIZE)
    }
}

/// Danger level grows with distance from home.
pub fn danger(k: SectorKey) -> i64 {
    k.0.abs().max(k.1.abs())
}

/// Ensure sectors around the player are generated / awake and put far
/// sectors to sleep.
pub fn stream(world: &mut World) {
    if !world.sectors.enabled {
        return;
    }
    let Some(player_pos) = world.player_body().map(|b| b.pos) else { return };
    let here = world.sectors.sector_of(player_pos);
    let mut active = BTreeSet::new();
    for dy in -ACTIVE_RADIUS..=ACTIVE_RADIUS {
        for dx in -ACTIVE_RADIUS..=ACTIVE_RADIUS {
            active.insert((here.0 + dx, here.1 + dy));
        }
    }
    // Freeze bodies that left the active area.
    let player = world.player;
    let leaving: Vec<BodyId> = world
        .bodies
        .iter()
        .filter(|(id, b)| Some(**id) != player && !active.contains(&world.sectors.sector_of(b.pos)))
        .map(|(id, _)| *id)
        .collect();
    for id in leaving {
        let mut b = world.bodies.remove(&id).unwrap();
        let k = world.sectors.sector_of(b.pos);
        b.pos -= world.sectors.center_of(k);
        world.sectors.dormant.entry(k).or_default().push(b);
    }
    world.projectiles.retain(|p| active.contains(&world.sectors.sector_of(p.pos)));
    // Wake / generate active sectors.
    for k in active {
        if let Some(bodies) = world.sectors.dormant.remove(&k) {
            let c = world.sectors.center_of(k);
            for mut b in bodies {
                b.pos += c;
                b.dirty = true;
                b.refresh(&world.content.clone());
                world.bodies.insert(b.id, b);
            }
        }
        if world.sectors.generated.insert(k) {
            generate(world, k);
        }
    }
}

/// Shift everything when the player strays more than a sector from the
/// origin so f32 precision stays good anywhere in the universe.
pub fn floating_origin(world: &mut World) {
    if !world.sectors.enabled {
        return;
    }
    let Some(p) = world.player_body().map(|b| b.pos) else { return };
    if p.x.abs() < SECTOR_SIZE && p.y.abs() < SECTOR_SIZE {
        return;
    }
    let k = world.sectors.sector_of(p);
    let o = world.sectors.origin;
    let d = (k.0 - o.0, k.1 - o.1);
    let delta = Vec2::new(d.0 as f32 * SECTOR_SIZE, d.1 as f32 * SECTOR_SIZE);
    world.sectors.origin = k;
    for b in world.bodies.values_mut() {
        b.pos -= delta;
        if let Some(ai) = b.ai.as_mut() {
            ai.home -= delta;
            ai.waypoint -= delta;
        }
    }
    for p in world.projectiles.iter_mut() {
        p.pos -= delta;
        p.dest -= delta;
    }
    world.events.push(Event::OriginShift { delta });
}

/// Build a roughly round asteroid. Deeper sectors have richer veins.
pub fn make_asteroid(world: &World, rng: &mut Rng, center: Vec2, radius: i32, depth: i64) -> Body {
    let c = world.content.clone();
    let rock = c.block("rock").unwrap();
    let ore = c.block("ore_vein").unwrap();
    let ice = c.block("ice_vein").unwrap();
    let rich = c.block("rich_vein").unwrap();
    let mut a = Body::new(0, "Asteroid", BodyKind::Asteroid, None, center, rng.range_f32(0.0, 6.28));
    let wobble: Vec<f32> = (0..8).map(|_| rng.range_f32(0.7, 1.15)).collect();
    for y in -radius..=radius {
        for x in -radius..=radius {
            let ang = libm::atan2f(y as f32, x as f32);
            let idx = (((ang + std::f32::consts::PI) / std::f32::consts::TAU * 8.0) as usize).min(7);
            let r = radius as f32 * wobble[idx];
            if ((x * x + y * y) as f32) > r * r {
                continue;
            }
            let roll = rng.f32();
            let def = if depth >= 2 && roll < 0.04 {
                rich
            } else if roll < 0.18 {
                ore
            } else if roll < 0.28 {
                ice
            } else {
                rock
            };
            let _ = a.insert(&c, def, Cell::new(x, y), 0);
        }
    }
    a.refresh(&c);
    a.pos = center;
    a.vel = Vec2::new(rng.range_f32(-0.3, 0.3), rng.range_f32(-0.3, 0.3));
    a.angvel = rng.range_f32(-0.05, 0.05);
    a
}

fn generate(world: &mut World, k: SectorKey) {
    let mut rng = Rng::derive(world.seed, k.0, k.1);
    let depth = danger(k);
    let center = world.sectors.center_of(k);
    let spread = SECTOR_SIZE * 0.45;
    // Asteroid fields.
    let fields = if depth == 0 { 3 } else { rng.range_i32(2, 4) };
    for _ in 0..fields {
        let fc = center + Vec2::new(rng.range_f32(-spread, spread), rng.range_f32(-spread, spread));
        let n = rng.range_i32(3, 6);
        for _ in 0..n {
            let p = fc + Vec2::new(rng.range_f32(-40.0, 40.0), rng.range_f32(-40.0, 40.0));
            // Keep the player's spawn point clear.
            if depth == 0 && p.length() < 35.0 {
                continue;
            }
            let r = rng.range_i32(2, 4 + depth.min(4) as i32);
            let a = make_asteroid(world, &mut rng, p, r, depth);
            world.add_body(a);
        }
    }
    if depth == 0 {
        // One easy wreck near home to teach salvage.
        let p = center + Vec2::new(rng.range_f32(50.0, 90.0), rng.range_f32(-60.0, 60.0));
        world.spawn_blueprint("wreck_hauler", p, rng.range_f32(0.0, 6.28), None);
        return;
    }
    // Derelicts to salvage.
    if rng.chance(0.6) {
        let p = center + Vec2::new(rng.range_f32(-spread, spread), rng.range_f32(-spread, spread));
        let bp = if depth >= 2 && rng.chance(0.5) { "raider" } else { "wreck_hauler" };
        if let Some(id) = world.spawn_blueprint(bp, p, rng.range_f32(0.0, 6.28), None) {
            // Knock out some blocks so it looks battle-damaged.
            let b = world.bodies.get_mut(&id).unwrap();
            for s in b.topo.live.clone() {
                if rng.chance(0.2) && !matches!(world.content.blocks[b.block(s).unwrap().def as usize].kind, gf_content::Block::Core) {
                    b.remove_block(s);
                }
            }
            crate::world::split_if_needed(world, id);
        }
    }
    // Pirate patrols.
    let patrols = depth.min(3);
    for _ in 0..patrols {
        if !rng.chance(0.6) {
            continue;
        }
        let p = center + Vec2::new(rng.range_f32(-spread, spread), rng.range_f32(-spread, spread));
        let bp = if depth >= 2 && rng.chance(0.6) { "raider" } else { "micro_bandit" };
        world.spawn_blueprint(bp, p, rng.range_f32(0.0, 6.28), Some(PIRATE_FACTION));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gf_content::ContentDb;
    use std::sync::Arc;

    #[test]
    fn travelling_streams_sectors_and_shifts_origin() {
        let c = Arc::new(ContentDb::load_dir(gf_content::default_content_dir()).unwrap());
        let mut w = World::new_game(c, 11);
        let p = w.player.unwrap();
        let home_bodies = w.bodies.len();
        assert!(home_bodies > 3);
        // Teleport the player two sectors east.
        w.bodies.get_mut(&p).unwrap().pos = Vec2::new(SECTOR_SIZE * 2.0, 0.0);
        w.step();
        stream(&mut w);
        assert_eq!(w.sectors.origin, (2, 0), "origin follows the player");
        assert!(w.bodies[&p].pos.x.abs() < SECTOR_SIZE);
        assert!(w.sectors.generated.contains(&(3, 0)));
        assert!(!w.sectors.dormant.is_empty(), "home sector went to sleep");
        // Return home: dormant bodies wake up at the same global place.
        w.bodies.get_mut(&p).unwrap().pos = Vec2::new(-SECTOR_SIZE * 2.0, 0.0);
        w.step();
        stream(&mut w);
        assert_eq!(w.sectors.origin, (0, 0));
        assert!(w.sectors.dormant.keys().all(|k| danger(*k) > 1 || k.0 > 1));
    }
}
