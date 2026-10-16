//! Ship AI: target selection, approach / orbit at preferred range, lead
//! targeting on high-value blocks, retreat when badly damaged.

use crate::body::{Body, BodyId, BodyKind, Control};
use crate::math::{Vec2, wrap_angle};
use crate::world::World;
use gf_content::{Block, ContentDb, WeaponKind};
use serde::{Deserialize, Serialize};

const RETARGET_SECS: f32 = 0.5;
const SENSE_RANGE: f32 = 140.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiMode {
    Patrol,
    Engage,
    Retreat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiState {
    pub mode: AiMode,
    pub target: Option<BodyId>,
    /// Slot of the block being aimed at on the target.
    pub target_slot: Option<u32>,
    pub timer: f32,
    pub home: Vec2,
    pub waypoint: Vec2,
    pub initial_blocks: usize,
}

impl AiState {
    pub fn new(home: Vec2, initial_blocks: usize) -> Self {
        Self {
            mode: AiMode::Patrol,
            target: None,
            target_slot: None,
            timer: 0.0,
            home,
            waypoint: home,
            initial_blocks,
        }
    }
}

/// Weapon range / projectile speed summary of a ship (for engagement logic).
fn weapon_profile(content: &ContentDb, body: &Body) -> (f32, f32) {
    let mut range: f32 = 0.0;
    let mut speed: f32 = 0.0;
    for s in body.slots_where(
        content,
        |k| matches!(k, Block::Weapon(w) if w.def.kind != WeaponKind::MiningLaser),
    ) {
        if let Block::Weapon(w) = &content.blocks[body.block(s).unwrap().def as usize].kind {
            range = range.max(w.def.range);
            speed = speed.max(w.def.projectile_speed);
        }
    }
    (range.max(10.0), speed.max(10.0))
}

/// Preferred block to shoot at: weapons, then thrusters, then the core.
fn pick_target_block(content: &ContentDb, target: &Body, from: Vec2) -> Option<u32> {
    let rank = |k: &Block| match k {
        Block::Weapon(_) => 0,
        Block::Thruster(_) => 1,
        Block::Core => 2,
        _ => 3,
    };
    target.topo.live.iter().copied().min_by(|a, b| {
        let ba = target.block(*a).unwrap();
        let bb = target.block(*b).unwrap();
        let ka = rank(&content.blocks[ba.def as usize].kind);
        let kb = rank(&content.blocks[bb.def as usize].kind);
        let da = target.to_world(ba.center()).distance(from);
        let db = target.to_world(bb.center()).distance(from);
        ka.cmp(&kb).then(da.total_cmp(&db)).then(a.cmp(b))
    })
}

/// Steering helper: turn to face a world direction (ship forward is local +y).
pub fn steer_towards(body: &Body, dir: Vec2) -> f32 {
    let forward = body.dir_to_world(Vec2::new(0.0, 1.0));
    let err = wrap_angle(dir.angle() - forward.angle());
    (err * 2.5 - body.angvel * 1.2).clamp(-1.0, 1.0)
}

pub fn think(world: &mut World, dt: f32) {
    let content = world.content.clone();
    let ids: Vec<BodyId> = world
        .bodies
        .iter()
        .filter(|(_, b)| b.ai.is_some() && b.kind == BodyKind::Ship)
        .map(|(id, _)| *id)
        .collect();
    for id in ids {
        let me = &world.bodies[&id];
        let mut st = me.ai.clone().unwrap();
        st.timer -= dt;
        if st.timer <= 0.0 {
            st.timer = RETARGET_SECS;
            // Nearest hostile ship.
            let mut best: Option<(f32, BodyId)> = None;
            for (oid, o) in &world.bodies {
                if *oid == id || o.kind != BodyKind::Ship || !me.is_hostile_to(o) {
                    continue;
                }
                let d = o.pos.distance(me.pos);
                if d < SENSE_RANGE && best.is_none_or(|(bd, _)| d < bd) {
                    best = Some((d, *oid));
                }
            }
            st.target = best.map(|b| b.1);
            st.target_slot = st
                .target
                .and_then(|t| pick_target_block(&content, &world.bodies[&t], me.pos));
            st.mode = if st.target.is_none() {
                AiMode::Patrol
            } else if me.block_count() * 10 < st.initial_blocks * 3 {
                AiMode::Retreat
            } else {
                AiMode::Engage
            };
            if st.mode == AiMode::Patrol && st.waypoint.distance(me.pos) < 15.0 {
                let a = world.rng.range_f32(0.0, std::f32::consts::TAU);
                let r = world.rng.range_f32(10.0, 60.0);
                st.waypoint = st.home + Vec2::from_angle(a) * r;
            }
        }
        let me = &world.bodies[&id];
        let (range, speed) = weapon_profile(&content, me);
        let mut c = Control {
            auto_suppress: true,
            ..Default::default()
        };
        let target = st
            .target
            .and_then(|t| world.bodies.get(&t))
            .filter(|t| t.kind == BodyKind::Ship);
        match (st.mode, target) {
            (AiMode::Engage, Some(t)) => {
                let aim_at = st
                    .target_slot
                    .and_then(|s| t.block(s))
                    .map(|b| t.to_world(b.center()))
                    .unwrap_or(t.pos);
                let to = aim_at - me.pos;
                let dist = to.length();
                let lead = aim_at + (t.vel - me.vel) * (dist / speed);
                c.aim = Some(lead);
                c.fire = dist < range * 1.05;
                c.turn = steer_towards(me, to);
                let pref = range * 0.7;
                let fwd = if dist > pref {
                    1.0
                } else if dist < pref * 0.6 {
                    -0.6
                } else {
                    0.0
                };
                // Thrust in the local frame toward the preferred range, and damp drift.
                let local_vel = me.vel.rotate(-me.angle);
                c.thrust =
                    Vec2::new(-local_vel.x * 0.3, fwd - local_vel.y * 0.05).clamp_length(1.0);
            }
            (AiMode::Retreat, Some(t)) => {
                let away = me.pos - t.pos;
                c.turn = steer_towards(me, away);
                c.thrust = Vec2::new(0.0, 1.0);
                c.aim = Some(t.pos);
                c.fire = t.pos.distance(me.pos) < range;
            }
            _ => {
                let to = st.waypoint - me.pos;
                c.turn = steer_towards(me, to);
                let local_vel = me.vel.rotate(-me.angle);
                let fwd = if to.length() > 8.0 { 0.6 } else { 0.0 };
                c.thrust = Vec2::new(-local_vel.x * 0.3, fwd - local_vel.y * 0.1).clamp_length(1.0);
            }
        }
        let me = world.bodies.get_mut(&id).unwrap();
        me.control = c;
        me.ai = Some(st);
    }
}
