//! Asteroid mining (drills), salvage of derelicts, tractor beams and labs.

use crate::body::{Body, BodyId, BodyKind, PLAYER_FACTION};
use crate::math::{Vec2, facing_vec};
use crate::progress::Progress;
use crate::world::{Event, World, split_if_needed};
use gf_content::{Block, ContentDb};

const TOOL_BUFFER: f32 = 10.0;
const SALVAGE_BUFFER: f32 = 40.0;
/// Fraction of a block's build cost recovered by salvaging it.
pub const SALVAGE_RETURN: f32 = 0.5;

/// Labs convert research items (e.g. data cores) into research points.
pub fn step_labs(content: &ContentDb, body: &mut Body, progress: &mut Progress, dt: f32) {
    let player = body.faction == Some(PLAYER_FACTION);
    for s in body.slots_where(content, |k| matches!(k, Block::Lab { .. })) {
        let def = body.block(s).unwrap().def;
        let Block::Lab { power, rate } = content.blocks[def as usize].kind else {
            continue;
        };
        let b = body.block_mut(s).unwrap();
        let has_input = b
            .stock
            .iter()
            .any(|(r, q)| *q > 0.0 && content.resources[*r as usize].research_value > 0.0);
        b.power_req = if has_input && player { power } else { 0.0 };
        if !player || !has_input || b.disabled > 0.0 {
            continue;
        }
        let mut budget = rate * b.power_sat * dt;
        let stock = b.stock.clone();
        for (r, q) in stock {
            let v = content.resources[r as usize].research_value;
            if v <= 0.0 || budget <= 0.0 {
                continue;
            }
            let used = b.take(r, q.min(budget));
            budget -= used;
            progress.add_points(used * v);
        }
    }
}

/// The body (other than `skip`) with a block at a world point.
fn body_at(world: &World, p: Vec2, skip: BodyId) -> Option<(BodyId, u32)> {
    for b in world.bodies.values() {
        if b.id == skip || b.pos.distance(p) > b.radius + 1.0 {
            continue;
        }
        if let Some(s) = b.block_at(b.local_cell_at(p)) {
            return Some((b.id, s));
        }
    }
    None
}

pub fn step_tools(world: &mut World, dt: f32) {
    let content = world.content.clone();
    let ids: Vec<BodyId> = world.bodies.keys().copied().collect();
    for id in ids {
        let Some(body) = world.bodies.get(&id) else {
            continue;
        };
        if body.kind != BodyKind::Ship {
            continue;
        }
        let player = body.faction == Some(PLAYER_FACTION);
        let tools = body.slots_where(&content, |k| {
            matches!(
                k,
                Block::Drill { .. } | Block::Salvager { .. } | Block::Tractor { .. }
            )
        });
        for s in tools {
            let body = &world.bodies[&id];
            let Some(b) = body.block(s) else { continue };
            let kind = content.blocks[b.def as usize].kind.clone();
            let front = body.to_world(b.center() + facing_vec(b.rot) * 1.0);
            let origin = body.to_world(b.center());
            let (power_sat, disabled, stock) = (b.power_sat, b.disabled > 0.0, b.total_stock());
            match kind {
                Block::Drill { rate, power } => {
                    let target = body_at(world, front, id);
                    let active = target.is_some() && stock < TOOL_BUFFER && !disabled;
                    world
                        .bodies
                        .get_mut(&id)
                        .unwrap()
                        .block_mut(s)
                        .unwrap()
                        .power_req = if active { power } else { 0.0 };
                    let Some((tb, ts)) = target.filter(|_| active) else {
                        continue;
                    };
                    let dmg = rate * power_sat * dt;
                    if dmg <= 0.0 {
                        continue;
                    }
                    let tblk = world.bodies[&tb].block(ts).unwrap();
                    if let Block::Mineral { yields } = &content.blocks[tblk.def as usize].kind {
                        let max = tblk.max_hp(&content);
                        let frac = dmg.min(tblk.hp.max(0.0)) / max;
                        let yields = yields.clone();
                        let d = world.bodies.get_mut(&id).unwrap().block_mut(s).unwrap();
                        for (r, q) in yields {
                            d.add(r, q * frac);
                            if player {
                                world.progress.note_mined(r, q * frac);
                            }
                        }
                    }
                    let fac = world.bodies[&id].faction;
                    crate::combat::damage_block(world, tb, ts, dmg, fac);
                }
                Block::Salvager { rate, power } => {
                    let target = body_at(world, front, id)
                        .filter(|(tb, _)| world.bodies[tb].kind == BodyKind::Derelict);
                    let active = target.is_some() && stock < SALVAGE_BUFFER && !disabled;
                    world
                        .bodies
                        .get_mut(&id)
                        .unwrap()
                        .block_mut(s)
                        .unwrap()
                        .power_req = if active { power } else { 0.0 };
                    let Some((tb, ts)) = target.filter(|_| active) else {
                        continue;
                    };
                    let tblk = world.bodies[&tb].block(ts).unwrap();
                    let max = tblk.max_hp(&content);
                    let pos = world.bodies[&tb].to_world(tblk.center());
                    let w = world.bodies.get_mut(&id).unwrap().block_mut(s).unwrap();
                    w.work += rate * power_sat * dt;
                    if w.work < max {
                        continue;
                    }
                    w.work = 0.0;
                    let inst = world.bodies.get_mut(&tb).unwrap().remove_block(ts).unwrap();
                    let info = &content.blocks[inst.def as usize];
                    let w = world.bodies.get_mut(&id).unwrap().block_mut(s).unwrap();
                    for (r, q) in &info.cost {
                        let back = libm::floorf(*q as f32 * SALVAGE_RETURN);
                        if back > 0.0 {
                            w.add(*r, back);
                        }
                    }
                    for (r, q) in &inst.stock {
                        if content.res_kind(*r) == gf_content::ResourceKind::Item {
                            w.add(*r, *q);
                        }
                    }
                    if player {
                        world.progress.blocks_salvaged += 1;
                        if let Some(sample) = info.sample {
                            world.progress.add_sample(sample);
                            world.events.push(Event::Message(format!(
                                "Recovered sample: {}",
                                content.resources[sample as usize].name
                            )));
                        }
                    }
                    world.events.push(Event::Salvaged { body: tb, pos });
                    split_if_needed(world, tb);
                }
                Block::Tractor {
                    range,
                    force,
                    power,
                } => {
                    // Pull the nearest loose body (asteroid / derelict) in range.
                    let me = &world.bodies[&id];
                    let target = world
                        .bodies
                        .values()
                        .filter(|o| o.id != id && o.kind != BodyKind::Ship)
                        .map(|o| (o.pos.distance(origin) - o.radius, o.id))
                        .filter(|(d, _)| *d < range && *d > 1.5)
                        .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
                    let active = target.is_some() && !disabled && me.control.fire;
                    let (me_vel,) = (me.vel,);
                    world
                        .bodies
                        .get_mut(&id)
                        .unwrap()
                        .block_mut(s)
                        .unwrap()
                        .power_req = if active { power } else { 0.0 };
                    let Some((_, tid)) = target.filter(|_| active) else {
                        continue;
                    };
                    let t = &world.bodies[&tid];
                    let dir = (origin - t.pos).normalize_or_zero();
                    let rel = t.vel - me_vel;
                    let f = (dir * force - rel * (force * 0.1)) * power_sat;
                    let tp = t.pos;
                    world.bodies.get_mut(&tid).unwrap().apply_force(f, tp);
                    world.bodies.get_mut(&id).unwrap().apply_force(-f, origin);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::Body;
    use crate::math::Cell;
    use std::sync::Arc;

    fn world() -> World {
        World::new(
            Arc::new(ContentDb::load_dir(gf_content::default_content_dir()).unwrap()),
            5,
        )
    }

    fn rock_wall(w: &mut World, at: Vec2, block: &str) -> BodyId {
        let c = w.content.clone();
        let mut a = Body::new(0, "asteroid", BodyKind::Asteroid, None, at, 0.0);
        for x in -3..=3 {
            for y in 0..3 {
                a.insert(&c, c.block(block).unwrap(), Cell::new(x, y), 0)
                    .unwrap();
            }
        }
        a.refresh(&c);
        a.pos = at + a.com;
        w.add_body(a)
    }

    #[test]
    fn drill_mines_ore_into_ship() {
        let mut w = world();
        let c = w.content.clone();
        let p = w
            .spawn_blueprint("forge_runner", Vec2::ZERO, 0.0, Some(PLAYER_FACTION))
            .unwrap();
        let drill = w.bodies[&p].slots_where(&c, |k| matches!(k, Block::Drill { .. }))[0];
        let tip = {
            let b = &w.bodies[&p];
            let d = b.block(drill).unwrap();
            b.to_world(d.center() + facing_vec(d.rot) * 1.0)
        };
        rock_wall(&mut w, tip - Vec2::new(0.0, 0.0), "ore_vein");
        let ore = c.res("ore").unwrap();
        for _ in 0..300 {
            w.step();
        }
        assert!(
            w.progress.mined.get(&ore).copied().unwrap_or(0.0) > 0.5,
            "mined {:?}",
            w.progress.mined
        );
    }

    #[test]
    fn salvager_recovers_materials_and_samples() {
        let mut w = world();
        let c = w.content.clone();
        let mut ship = Body::new(
            0,
            "salv",
            BodyKind::Ship,
            Some(PLAYER_FACTION),
            Vec2::ZERO,
            0.0,
        );
        ship.insert(&c, c.block("ship_core").unwrap(), Cell::new(0, 0), 0)
            .unwrap();
        ship.insert(&c, c.block("salvager").unwrap(), Cell::new(0, 1), 0)
            .unwrap();
        ship.refresh(&c);
        let sid = w.add_body(ship);
        let front = {
            let b = &w.bodies[&sid];
            b.to_world(Cell::new(0, 2).center())
        };
        let mut wreck = Body::new(0, "wreck", BodyKind::Derelict, None, front, 0.0);
        wreck
            .insert(&c, c.block("rocket_launcher").unwrap(), Cell::new(0, 0), 0)
            .unwrap();
        wreck.refresh(&c);
        wreck.pos = front;
        w.add_body(wreck);
        for _ in 0..600 {
            w.step();
        }
        assert!(w.progress.blocks_salvaged >= 1);
        let guidance = c.res("sample_guidance").unwrap();
        assert_eq!(w.progress.samples.get(&guidance).copied(), Some(1));
        let plates = c.res("plates").unwrap();
        let sb = &w.bodies[&sid];
        let salv = sb.slots_where(&c, |k| matches!(k, Block::Salvager { .. }))[0];
        assert!(sb.block(salv).unwrap().amount(plates) >= 3.0);
    }
}
