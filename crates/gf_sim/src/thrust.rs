//! Thruster mixing, deterministic rigid-body integration and ship collisions.

use crate::body::{Body, BodyId, BodyKind};
use crate::math::{Vec2, facing_vec};
use crate::world::World;
use gf_content::{Block, ContentDb};

pub const LINEAR_DAMPING: f32 = 0.25;
pub const ANGULAR_DAMPING: f32 = 1.2;
const RESTITUTION: f32 = 0.2;
const RAM_DAMAGE_SPEED: f32 = 5.0;

/// Thrust multiplier per tier.
pub fn thrust_mult(tier: u8) -> f32 {
    1.0 + 0.8 * (tier.max(1) as f32 - 1.0)
}

/// Compute throttles from the control input and apply thruster forces.
///
/// Each thruster scores how well it helps the request: alignment of its push
/// with the desired translation plus whether its torque matches the desired
/// turn. That gives a cheap, stable mixer for arbitrary layouts.
pub fn mix_and_fire(content: &ContentDb, body: &mut Body, dt: f32) {
    let want = body.control.thrust.clamp_length(1.0);
    let turn = body.control.turn.clamp(-1.0, 1.0);
    let thrusters = body.slots_where(content, |k| matches!(k, Block::Thruster(_)));
    for s in thrusters {
        let b = body.block(s).unwrap();
        let Block::Thruster(t) = &content.blocks[b.def as usize].kind else {
            continue;
        };
        let dir = facing_vec(b.rot);
        let at = b.center();
        let lever = (at - body.com).cross(dir);
        let torque_sign = if lever.abs() > 0.3 {
            lever.signum()
        } else {
            0.0
        };
        let mut throttle = (dir.dot(want) + turn * torque_sign).clamp(0.0, 1.0);
        if b.disabled > 0.0 || b.heat >= t.def.heat_capacity {
            throttle = 0.0;
        }
        let tier = b.feed.tier;
        let demand = t.def.growth[tier as usize - 1].demand;
        let need = demand * throttle * dt;
        let have = b.amount(t.fuel);
        let frac = if need > 0.0 {
            (have / need).min(1.0)
        } else {
            0.0
        };
        throttle *= frac;
        let mut bonus = 1.0;
        let bm = body.block_mut(s).unwrap();
        if throttle > 0.0 {
            bm.take(t.fuel, need * frac);
            if let Some(o) = t.oxidizer
                && bm.amount(o) > 0.0
            {
                bm.take(o, need * 0.5 * frac);
                bonus = 1.25;
            }
            bm.heat += t.def.heat_per_sec * throttle * dt;
        }
        bm.heat = (bm.heat - t.def.cool_rate * dt).max(0.0);
        bm.throttle = throttle;
        let force_local = dir * (t.def.thrust * thrust_mult(tier) * throttle * bonus);
        if throttle > 0.0 {
            let f = body.dir_to_world(force_local);
            let p = body.to_world(at);
            body.apply_force(f, p);
        }
    }
}

pub fn apply_thrusters(world: &mut World, dt: f32) {
    let content = world.content.clone();
    for b in world.bodies.values_mut() {
        if b.kind == BodyKind::Ship {
            mix_and_fire(&content, b, dt);
        } else {
            b.control = Default::default();
        }
    }
}

pub fn integrate(world: &mut World, dt: f32) {
    for b in world.bodies.values_mut() {
        b.vel += b.force * (dt / b.mass);
        b.angvel += b.torque * dt / b.inertia;
        b.vel = b.vel * (1.0 - LINEAR_DAMPING * dt);
        b.angvel *= 1.0 - ANGULAR_DAMPING * dt;
        b.pos += b.vel * dt;
        b.angle = crate::math::wrap_angle(b.angle + b.angvel * dt);
        b.force = Vec2::ZERO;
        b.torque = 0.0;
    }
}

/// Sweep-and-prune on x, then a cell-overlap test from the smaller body
/// against the larger body's grid; resolves with an impulse along the average
/// contact normal and a positional push-out. Hard impacts damage the blocks
/// in contact.
pub fn collide(world: &mut World, _dt: f32) {
    let mut spans: Vec<(f32, f32, BodyId)> = world
        .bodies
        .values()
        .map(|b| (b.pos.x - b.radius, b.pos.x + b.radius, b.id))
        .collect();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.2.cmp(&b.2)));
    let mut pairs = Vec::new();
    for i in 0..spans.len() {
        for j in i + 1..spans.len() {
            if spans[j].0 > spans[i].1 {
                break;
            }
            let (a, b) = (spans[i].2.min(spans[j].2), spans[i].2.max(spans[j].2));
            let ba = &world.bodies[&a];
            let bb = &world.bodies[&b];
            if ba.pos.distance(bb.pos) < ba.radius + bb.radius {
                pairs.push((a, b));
            }
        }
    }
    pairs.sort_unstable();
    for (a, b) in pairs {
        resolve_pair(world, a, b);
    }
}

fn resolve_pair(world: &mut World, a: BodyId, b: BodyId) {
    let (small, big) = {
        let (ba, bb) = (&world.bodies[&a], &world.bodies[&b]);
        if ba.grid.len() <= bb.grid.len() {
            (a, b)
        } else {
            (b, a)
        }
    };
    let s = &world.bodies[&small];
    let g = &world.bodies[&big];
    let mut n = Vec2::ZERO;
    let mut point = Vec2::ZERO;
    let mut count = 0u32;
    let mut hits: Vec<(u32, u32)> = Vec::new();
    for (cell, slot) in s.grid.iter() {
        let wp = s.to_world(cell.center());
        if wp.distance(g.pos) > g.radius + 0.75 {
            continue;
        }
        let gc = g.local_cell_at(wp);
        if let Some(gs) = g.block_at(gc) {
            let other = g.to_world(gc.center());
            n += (wp - other).normalize_or_zero();
            point += wp;
            count += 1;
            if hits.len() < 8 {
                hits.push((slot, gs));
            }
        }
    }
    if count == 0 {
        return;
    }
    let mut normal = n.normalize_or_zero();
    if normal == Vec2::ZERO {
        normal = (s.pos - g.pos).normalize_or_zero();
    }
    let point = point * (1.0 / count as f32);
    let rel = s.velocity_at(point) - g.velocity_at(point);
    let vn = rel.dot(normal);
    let (ms, mg) = (s.mass, g.mass);
    let (is, ig) = (s.inertia, g.inertia);
    let rs = point - s.pos;
    let rg = point - g.pos;
    let rsn = rs.cross(normal);
    let rgn = rg.cross(normal);
    let inv = 1.0 / ms + 1.0 / mg + rsn * rsn / is + rgn * rgn / ig;
    let impact = -vn;
    if vn < 0.0 {
        let j = -(1.0 + RESTITUTION) * vn / inv;
        let imp = normal * j;
        let s = world.bodies.get_mut(&small).unwrap();
        s.vel += imp * (1.0 / ms);
        s.angvel += rsn * j / is;
        let g = world.bodies.get_mut(&big).unwrap();
        g.vel -= imp * (1.0 / mg);
        g.angvel -= rgn * j / ig;
    }
    // Positional correction proportional to overlap depth (cells in contact).
    let depth = (count as f32).sqrt().min(4.0) * 0.08;
    let total = ms + mg;
    world.bodies.get_mut(&small).unwrap().pos += normal * (depth * mg / total);
    world.bodies.get_mut(&big).unwrap().pos -= normal * (depth * ms / total);
    if impact > RAM_DAMAGE_SPEED {
        let dmg = (impact - RAM_DAMAGE_SPEED) * 4.0;
        for (ss, gs) in hits {
            crate::combat::damage_block(world, small, ss, dmg, None);
            crate::combat::damage_block(world, big, gs, dmg, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::Control;
    use std::sync::Arc;

    #[test]
    fn forward_thrust_accelerates_forward_without_spin() {
        let c = Arc::new(ContentDb::load_dir(gf_content::default_content_dir()).unwrap());
        let mut w = World::new(c, 9);
        let id = w
            .spawn_blueprint("forge_runner", Vec2::ZERO, 0.0, Some(0))
            .unwrap();
        w.bodies.get_mut(&id).unwrap().control = Control {
            thrust: Vec2::new(0.0, 1.0),
            ..Default::default()
        };
        let start = w.bodies[&id].pos;
        for _ in 0..90 {
            let ctrl = w.bodies[&id].control.clone();
            w.queue(crate::world::Command::Control {
                body: id,
                control: ctrl,
            });
            w.step();
        }
        let b = &w.bodies[&id];
        assert!(b.pos.y - start.y > 1.0, "moved forward: {:?}", b.pos);
        assert!(b.angle.abs() < 0.3, "little spin: {}", b.angle);
    }

    #[test]
    fn turning_rotates_ship() {
        let c = Arc::new(ContentDb::load_dir(gf_content::default_content_dir()).unwrap());
        let mut w = World::new(c, 9);
        let id = w
            .spawn_blueprint("forge_runner", Vec2::ZERO, 0.0, Some(0))
            .unwrap();
        for _ in 0..60 {
            w.queue(crate::world::Command::Control {
                body: id,
                control: Control {
                    turn: 1.0,
                    ..Default::default()
                },
            });
            w.step();
        }
        assert!(
            w.bodies[&id].angle > 0.05,
            "turned CCW: {}",
            w.bodies[&id].angle
        );
    }

    #[test]
    fn bodies_do_not_pass_through_each_other() {
        let c = Arc::new(ContentDb::load_dir(gf_content::default_content_dir()).unwrap());
        let mut w = World::new(c, 9);
        let a = w
            .spawn_blueprint("micro_bandit", Vec2::new(0.0, 0.0), 0.0, None)
            .unwrap();
        let b = w
            .spawn_blueprint("micro_bandit", Vec2::new(12.0, 0.0), 0.0, None)
            .unwrap();
        w.bodies.get_mut(&a).unwrap().vel = Vec2::new(6.0, 0.0);
        w.bodies.get_mut(&b).unwrap().vel = Vec2::new(-6.0, 0.0);
        for _ in 0..90 {
            w.step();
        }
        assert!(
            w.bodies[&a].pos.x < w.bodies[&b].pos.x,
            "still ordered after impact"
        );
    }
}
