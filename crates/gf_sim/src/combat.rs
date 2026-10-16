//! Weapons, projectiles, damage, fire, suppression and destruction.
//!
//! Weapons grow with sustained supply (see [`crate::feed`]). Fire rate,
//! damage and projectile speed scale with tier, and the ammo mix that was
//! delivered recently decides the effects (pierce, splash, incendiary, frost,
//! EMP). Cannons fed mostly frost ammo (ice / coolant) can turn on their own
//! ship to put out fires.

use crate::body::{Body, BodyId, BodyKind, Faction, PLAYER_FACTION, WeaponMode};
use crate::logistics::{thruster_fuel_cap, weapon_ammo_cap};
use crate::math::{Cell, Vec2, wrap_angle};
use crate::world::{Event, World, split_if_needed};
use gf_content::{Block, BlockId, ContentDb, WeaponKind};
use serde::{Deserialize, Serialize};

/// Minimum frost share of the ammo mix that enables suppression mode.
pub const SUPPRESS_FROST_SHARE: f32 = 0.3;
const MIN_DAMAGE_FRACTION: f32 = 0.1;
const PIERCE_FALLOFF: f32 = 0.7;
const LASER_BUFFER: f32 = 20.0;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Effects {
    pub damage_mult: f32,
    pub splash: f32,
    pub pierce: u8,
    pub incendiary: f32,
    pub frost: f32,
    pub emp: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Projectile {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub damage: f32,
    pub owner: BodyId,
    pub faction: Option<Faction>,
    pub fx: Effects,
    pub homing: f32,
    pub target: Option<BodyId>,
    /// Suppression shots fly to `dest` without colliding and extinguish fires.
    pub suppress: bool,
    pub dest: Vec2,
    pub def: BlockId,
    pub tier: u8,
}

/// Effects of the recently delivered ammo mix (falls back to current stock).
pub fn ammo_effects(content: &ContentDb, b: &crate::body::BlockInst) -> Effects {
    let src = if b.ammo_mix.iter().any(|e| e.1 > 1e-3) { &b.ammo_mix } else { &b.stock };
    let total: f32 = src.iter().map(|e| e.1.max(0.0)).sum();
    let mut fx = Effects { damage_mult: 1.0, ..Default::default() };
    if total <= 1e-6 {
        return fx;
    }
    fx.damage_mult = 0.0;
    for (r, q) in src {
        let share = q.max(0.0) / total;
        let a = content.resources[*r as usize].ammo.clone().unwrap_or_default();
        let dm = if content.resources[*r as usize].ammo.is_some() { a.damage_mult } else { 1.0 };
        fx.damage_mult += share * dm;
        fx.splash += share * a.splash_radius;
        fx.incendiary += share * a.incendiary;
        fx.frost += share * a.frost;
        fx.emp += share * a.emp;
        if share >= 0.3 {
            fx.pierce = fx.pierce.max(a.pierce);
        }
    }
    fx
}

/// Shots per second multiplier at a tier.
pub fn rate_mult(tier: u8) -> f32 {
    1.0 + 0.5 * (tier.max(1) as f32 - 1.0)
}
pub fn damage_mult(tier: u8) -> f32 {
    libm::powf(tier.max(1) as f32, 1.3)
}
pub fn speed_mult(tier: u8) -> f32 {
    1.0 + 0.15 * (tier.max(1) as f32 - 1.0)
}

/// Feed-driven growth / shrink of weapons and thrusters.
pub fn update_tiers(content: &ContentDb, body: &mut Body, dt: f32, events: &mut Vec<Event>) {
    let slots = body.slots_where(content, |k| matches!(k, Block::Weapon(_) | Block::Thruster(_)));
    for s in slots {
        let b = body.block(s).unwrap();
        let info = &content.blocks[b.def as usize];
        let stages = info.growth().unwrap();
        let saturated = match &info.kind {
            Block::Weapon(w) if w.def.kind == WeaponKind::MiningLaser => false,
            Block::Weapon(_) => b.total_stock() >= 0.9 * weapon_ammo_cap(content, b),
            Block::Thruster(t) => b.amount(t.fuel) >= 0.9 * thruster_fuel_cap(content, b),
            _ => false,
        };
        let old = b.feed.tier;
        let cap = body.growth_cap(content, s);
        let b = body.block_mut(s).unwrap();
        b.feed.cap = cap;
        if let Some(t) = b.feed.update(stages, dt, saturated) {
            let size = stages[t as usize - 1].size;
            body.resize_block(s, size);
            events.push(Event::TierChanged { body: body.id, slot: s, tier: t });
            let _ = old;
        }
    }
}

struct ShotReq {
    body: BodyId,
    slot: u32,
    from: Vec2,
    dir: Vec2,
    speed: f32,
    life: f32,
    damage: f32,
    fx: Effects,
    homing: f32,
    suppress: Option<Vec2>,
    def: BlockId,
    tier: u8,
    aim: Option<Vec2>,
}

struct LaserReq {
    body: BodyId,
    slot: u32,
    from: Vec2,
    dir: Vec2,
    range: f32,
    dps: f32,
    tier: u8,
}

/// Hottest burning block of the body within range of a point.
fn hottest_fire(body: &Body, from: Vec2, range: f32) -> Option<Vec2> {
    let mut best: Option<(f32, Vec2)> = None;
    for &s in &body.topo.live {
        let b = body.block(s).unwrap();
        if b.fire > 0.05 {
            let p = body.to_world(b.center());
            if p.distance(from) <= range && best.is_none_or(|(f, _)| b.fire > f) {
                best = Some((b.fire, p));
            }
        }
    }
    best.map(|b| b.1)
}

pub fn step_weapons(world: &mut World, dt: f32) {
    let content = world.content.clone();
    let mut shots = Vec::new();
    let mut lasers = Vec::new();
    let ids: Vec<BodyId> = world.bodies.keys().copied().collect();
    for id in ids {
        let body = world.bodies.get_mut(&id).unwrap();
        if body.kind != BodyKind::Ship {
            continue;
        }
        let weapons = body.slots_where(&content, |k| matches!(k, Block::Weapon(_)));
        for s in weapons {
            let b = body.block(s).unwrap();
            let Block::Weapon(w) = &content.blocks[b.def as usize].kind else { continue };
            let def = &w.def;
            let tier = b.feed.tier;
            let center_l = b.center();
            let center = body.to_world(center_l);
            let half = b.size.0 as f32 * 0.5;
            // Decay the ammo-mix memory.
            let decay = 1.0 - dt / (crate::feed::FEED_TAU + dt);
            let fx = ammo_effects(&content, b);

            // Choose a target point: suppression first, then the control aim.
            let can_suppress = def.kind == WeaponKind::Cannon && fx.frost >= SUPPRESS_FROST_SHARE;
            let range = def.range * (1.0 + 0.1 * (tier as f32 - 1.0));
            let suppress_target = if can_suppress && (b.mode == WeaponMode::Suppress || body.control.auto_suppress) {
                hottest_fire(body, center, range)
            } else {
                None
            };
            let attack = b.mode == WeaponMode::Attack && body.control.fire;
            let aim_point = suppress_target.or(if attack || def.kind == WeaponKind::MiningLaser { body.control.aim } else { None });

            let b = body.block(s).unwrap();
            let mut aim = b.aim;
            let mut aligned = false;
            if let Some(p) = aim_point {
                let desired = (body.to_local(p) - center_l).angle();
                let err = wrap_angle(desired - aim);
                let step = def.turn_rate * dt;
                aim = wrap_angle(aim + err.clamp(-step, step));
                aligned = err.abs() < 0.15;
            }
            let disabled = b.disabled > 0.0;
            let heat_frac = b.heat / def.heat_capacity;
            let dir = body.dir_to_world(Vec2::from_angle(aim));
            let muzzle = center + dir * half;
            let bvel = body.vel;
            let power_ok = def.power <= 0.0 || b.power_sat >= 0.5;
            let want_fire = body.control.fire;

            let bm = body.block_mut(s).unwrap();
            bm.aim = aim;
            for e in bm.ammo_mix.iter_mut() {
                e.1 *= decay;
            }
            bm.ammo_mix.retain(|e| e.1 > 1e-4);
            bm.cooldown = (bm.cooldown - dt).max(0.0);
            bm.heat = (bm.heat - def.cool_rate * dt).max(0.0);

            if def.kind == WeaponKind::MiningLaser {
                let stages = &def.growth;
                let firing = want_fire && aligned && !disabled && aim_point.is_some() && bm.total_stock() < LASER_BUFFER;
                if firing {
                    let next = (tier as usize).min(bm.feed.cap.max(1) as usize - 1).min(stages.len() - 1);
                    bm.power_req = stages[next].demand;
                    bm.feed.delivered += bm.power_req * bm.power_sat * dt;
                    bm.heat += def.heat_per_shot * dt * 10.0;
                    if bm.power_sat > 0.05 {
                        lasers.push(LaserReq {
                            body: id,
                            slot: s,
                            from: muzzle,
                            dir,
                            range,
                            dps: def.damage * damage_mult(tier) * bm.power_sat,
                            tier,
                        });
                    }
                } else {
                    bm.power_req = 0.0;
                }
                continue;
            }

            let ready = bm.cooldown <= 0.0 && !disabled && heat_frac < 1.0 && power_ok;
            if !(ready && aligned && aim_point.is_some()) {
                continue;
            }
            let interval = def.interval / rate_mult(tier);
            let per_shot = (def.growth[tier as usize - 1].demand * interval).max(0.05);
            let have = bm.total_stock();
            if have < per_shot {
                continue;
            }
            // Consume proportionally from the buffered ammo types.
            let frac = per_shot / have;
            for e in bm.stock.iter_mut() {
                e.1 -= e.1 * frac;
            }
            bm.stock.retain(|e| e.1 > 1e-5);
            let heat_factor = 1.0 + (heat_frac - 0.7).max(0.0) * 3.0;
            bm.cooldown = interval * heat_factor;
            bm.heat += def.heat_per_shot;
            let speed = def.projectile_speed * speed_mult(tier);
            shots.push(ShotReq {
                body: id,
                slot: s,
                from: muzzle,
                dir,
                speed,
                life: range / speed.max(1.0),
                damage: def.damage * fx.damage_mult * damage_mult(tier),
                fx: Effects { splash: fx.splash * (1.0 + 0.2 * (tier as f32 - 1.0)), ..fx },
                homing: def.homing,
                suppress: suppress_target,
                def: bm.def,
                tier,
                aim: aim_point,
            });
            let _ = bvel;
        }
    }

    for sh in shots {
        let body = &world.bodies[&sh.body];
        let faction = body.faction;
        let target = if sh.homing > 0.0 {
            sh.aim.and_then(|a| {
                world
                    .bodies
                    .values()
                    .filter(|o| o.id != sh.body && o.kind == BodyKind::Ship && body.is_hostile_to(o))
                    .min_by(|x, y| x.pos.distance(a).total_cmp(&y.pos.distance(a)).then(x.id.cmp(&y.id)))
                    .map(|o| o.id)
            })
        } else {
            None
        };
        world.events.push(Event::Shot { body: sh.body, pos: sh.from, dir: sh.dir, def: sh.def, tier: sh.tier });
        let _ = sh.slot;
        world.projectiles.push(Projectile {
            pos: sh.from,
            vel: sh.dir * sh.speed + body.vel,
            life: sh.life,
            damage: if sh.suppress.is_some() { 0.0 } else { sh.damage },
            owner: sh.body,
            faction,
            fx: sh.fx,
            homing: sh.homing,
            target,
            suppress: sh.suppress.is_some(),
            dest: sh.suppress.unwrap_or(Vec2::ZERO),
            def: sh.def,
            tier: sh.tier,
        });
    }

    for l in lasers {
        let end = l.from + l.dir * l.range;
        let owner = l.body;
        let hit = raycast(world, l.from, end, |b| b.id == owner);
        let to = hit.as_ref().map_or(end, |h| h.point);
        world.events.push(Event::Laser { from: l.from, to, tier: l.tier });
        let Some(h) = hit else { continue };
        let dmg = l.dps * crate::world::DT;
        let tb = &world.bodies[&h.body];
        let tblk = tb.block(h.slot).unwrap();
        if let Block::Mineral { yields } = &content.blocks[tblk.def as usize].kind {
            let frac = (dmg / tblk.max_hp(&content)).min(tblk.hp.max(0.0) / tblk.max_hp(&content));
            let yields = yields.clone();
            let player = world.bodies[&owner].faction == Some(PLAYER_FACTION);
            let lb = world.bodies.get_mut(&owner).unwrap().block_mut(l.slot).unwrap();
            for (r, q) in yields {
                lb.add(r, q * frac);
                if player {
                    world.progress.note_mined(r, q * frac);
                }
            }
        }
        let fac = world.bodies[&owner].faction;
        damage_block(world, h.body, h.slot, dmg, fac);
    }
}

pub struct RayHit {
    pub body: BodyId,
    pub slot: u32,
    pub t: f32,
    pub point: Vec2,
}

/// Grid DDA through one body in its local frame. Returns (slot, t in 0..1).
fn dda(body: &Body, a: Vec2, b: Vec2) -> Option<(u32, f32)> {
    let (ax, ay) = (a.x + 0.5, a.y + 0.5);
    let (bx, by) = (b.x + 0.5, b.y + 0.5);
    let mut cx = libm::floorf(ax) as i32;
    let mut cy = libm::floorf(ay) as i32;
    let ex = libm::floorf(bx) as i32;
    let ey = libm::floorf(by) as i32;
    let dx = bx - ax;
    let dy = by - ay;
    let sx = if dx > 0.0 { 1 } else { -1 };
    let sy = if dy > 0.0 { 1 } else { -1 };
    let tdx = if dx != 0.0 { 1.0 / dx.abs() } else { f32::INFINITY };
    let tdy = if dy != 0.0 { 1.0 / dy.abs() } else { f32::INFINITY };
    let mut tmx = if dx > 0.0 {
        (cx as f32 + 1.0 - ax) / dx
    } else if dx < 0.0 {
        (ax - cx as f32) / -dx
    } else {
        f32::INFINITY
    };
    let mut tmy = if dy > 0.0 {
        (cy as f32 + 1.0 - ay) / dy
    } else if dy < 0.0 {
        (ay - cy as f32) / -dy
    } else {
        f32::INFINITY
    };
    let mut t = 0.0;
    let n = (ex - cx).abs() + (ey - cy).abs() + 2;
    for _ in 0..n {
        if let Some(s) = body.block_at(Cell::new(cx, cy)) {
            return Some((s, t));
        }
        if cx == ex && cy == ey {
            break;
        }
        if tmx < tmy {
            t = tmx;
            cx += sx;
            tmx += tdx;
        } else {
            t = tmy;
            cy += sy;
            tmy += tdy;
        }
        if t > 1.0 {
            break;
        }
    }
    None
}

/// First block hit along a world-space segment, skipping bodies for which
/// `skip` returns true.
pub fn raycast(world: &World, from: Vec2, to: Vec2, skip: impl Fn(&Body) -> bool) -> Option<RayHit> {
    let seg = to - from;
    let len2 = seg.length_sq().max(1e-9);
    let mut best: Option<RayHit> = None;
    for b in world.bodies.values() {
        if skip(b) {
            continue;
        }
        let t = ((b.pos - from).dot(seg) / len2).clamp(0.0, 1.0);
        if (from + seg * t).distance(b.pos) > b.radius {
            continue;
        }
        if let Some((slot, t)) = dda(b, b.to_local(from), b.to_local(to)) {
            if best.as_ref().is_none_or(|h| t < h.t) {
                best = Some(RayHit { body: b.id, slot, t, point: from + seg * t });
            }
        }
    }
    best
}

/// Apply raw damage to a block (armor reduces it). Returns damage dealt.
pub fn damage_block(world: &mut World, body: BodyId, slot: u32, dmg: f32, by: Option<Faction>) -> f32 {
    let content = world.content.clone();
    let Some(b) = world.bodies.get_mut(&body) else { return 0.0 };
    if by.is_some() {
        b.last_hit_by = by;
    }
    let Some(inst) = b.block_mut(slot) else { return 0.0 };
    if inst.hp <= 0.0 {
        return 0.0;
    }
    let armor = content.blocks[inst.def as usize].def.armor;
    let eff = (dmg - armor).max(dmg * MIN_DAMAGE_FRACTION);
    inst.hp -= eff;
    if inst.hp <= 0.0 {
        world.dead.push((body, slot));
    }
    eff
}

pub fn apply_effects(world: &mut World, body: BodyId, slot: u32, fx: &Effects) {
    let Some(inst) = world.bodies.get_mut(&body).and_then(|b| b.block_mut(slot)) else { return };
    if fx.incendiary > 0.0 {
        inst.fire = (inst.fire + fx.incendiary * 0.6).min(1.0);
    }
    if fx.frost > 0.0 {
        inst.chill = inst.chill.max(1.5 + 3.0 * fx.frost);
        inst.fire *= (1.0 - fx.frost).max(0.0);
        inst.heat *= 0.5;
    }
    if fx.emp > 0.0 {
        inst.disabled = inst.disabled.max(fx.emp * 3.0);
    }
}

/// Area damage around a world point. Bodies of `spare` faction are untouched.
pub fn splash(world: &mut World, center: Vec2, radius: f32, dmg: f32, spare: Option<Faction>, by: Option<Faction>, fx: &Effects) {
    let mut hits: Vec<(BodyId, u32, f32)> = Vec::new();
    for b in world.bodies.values() {
        if spare.is_some() && b.faction == spare {
            continue;
        }
        if b.pos.distance(center) > b.radius + radius {
            continue;
        }
        let l = b.to_local(center);
        let (x0, x1) = (libm::floorf(l.x - radius) as i32, libm::ceilf(l.x + radius) as i32);
        let (y0, y1) = (libm::floorf(l.y - radius) as i32, libm::ceilf(l.y + radius) as i32);
        let mut local: Vec<(u32, f32)> = Vec::new();
        for y in y0..=y1 {
            for x in x0..=x1 {
                let c = Cell::new(x, y);
                let d = c.center().distance(l);
                if d > radius {
                    continue;
                }
                if let Some(s) = b.block_at(c) {
                    let f = 1.0 - 0.5 * d / radius.max(1e-3);
                    match local.iter_mut().find(|e| e.0 == s) {
                        Some(e) => e.1 = e.1.max(f),
                        None => local.push((s, f)),
                    }
                }
            }
        }
        hits.extend(local.into_iter().map(|(s, f)| (b.id, s, f)));
    }
    for (b, s, f) in hits {
        damage_block(world, b, s, dmg * f, by);
        apply_effects(world, b, s, fx);
    }
}

/// Put out fires around a point on one body.
pub fn extinguish(world: &mut World, body: BodyId, at: Vec2, radius: f32) {
    let Some(b) = world.bodies.get_mut(&body) else { return };
    for s in b.topo.live.clone() {
        let p = b.to_world(b.block(s).unwrap().center());
        if p.distance(at) <= radius {
            let inst = b.block_mut(s).unwrap();
            inst.fire = 0.0;
            inst.heat *= 0.3;
        }
    }
    world.events.push(Event::Extinguish { pos: at, radius });
}

pub fn step_projectiles(world: &mut World, dt: f32) {
    let mut projectiles = std::mem::take(&mut world.projectiles);
    let mut keep = Vec::with_capacity(projectiles.len());
    for mut p in projectiles.drain(..) {
        p.life -= dt;
        if p.suppress {
            let to = p.dest - p.pos;
            let step = p.vel.length() * dt;
            if to.length() <= step || p.life <= 0.0 {
                let r = 1.5 + 0.5 * p.tier as f32;
                extinguish(world, p.owner, p.dest, r);
                continue;
            }
            p.pos += to.normalize_or_zero() * step;
            keep.push(p);
            continue;
        }
        if p.homing > 0.0 {
            if let Some(t) = p.target.and_then(|t| world.bodies.get(&t)) {
                let desired = (t.pos - p.pos).angle();
                let cur = p.vel.angle();
                let err = wrap_angle(desired - cur);
                let turn = err.clamp(-p.homing * dt, p.homing * dt);
                p.vel = p.vel.rotate(turn);
            }
        }
        let end = p.pos + p.vel * dt;
        let (owner, faction) = (p.owner, p.faction);
        let hit = raycast(world, p.pos, end, |b| b.id == owner || (faction.is_some() && b.faction == faction));
        match hit {
            Some(h) => {
                world.events.push(Event::Hit { pos: h.point, damage: p.damage });
                damage_block(world, h.body, h.slot, p.damage, faction);
                apply_effects(world, h.body, h.slot, &p.fx);
                if p.fx.splash > 0.0 {
                    splash(world, h.point, p.fx.splash, p.damage * 0.6, faction, faction, &p.fx);
                    world.events.push(Event::Explosion { pos: h.point, radius: p.fx.splash });
                }
                if p.fx.pierce > 0 {
                    p.fx.pierce -= 1;
                    p.damage *= PIERCE_FALLOFF;
                    p.pos = h.point + p.vel.normalize_or_zero() * 1.0;
                    if p.life > 0.0 {
                        keep.push(p);
                    }
                }
            }
            None => {
                p.pos = end;
                if p.life > 0.0 {
                    keep.push(p);
                }
            }
        }
    }
    keep.append(&mut world.projectiles);
    world.projectiles = keep;
}

/// Fire growth / spread / damage, status timers and passive cooling.
pub fn step_fire(world: &mut World, dt: f32) {
    let content = world.content.clone();
    let ids: Vec<BodyId> = world.bodies.keys().copied().collect();
    for id in ids {
        let mut ignite: Vec<u32> = Vec::new();
        let mut killed: Vec<u32> = Vec::new();
        {
            let body = world.bodies.get_mut(&id).unwrap();
            if body.kind == BodyKind::Asteroid {
                continue;
            }
            for s in body.topo.live.clone() {
                let flam = body.block(s).unwrap().flammability(&content);
                let is_hot_kind = matches!(content.blocks[body.block(s).unwrap().def as usize].kind, Block::Weapon(_) | Block::Thruster(_));
                let b = body.block_mut(s).unwrap();
                b.chill = (b.chill - dt).max(0.0);
                b.disabled = (b.disabled - dt).max(0.0);
                if !is_hot_kind {
                    b.heat = (b.heat - 4.0 * dt).max(0.0);
                }
                if b.fire <= 0.0 && b.heat > 120.0 && flam > 0.05 {
                    b.fire = 0.2;
                }
                if b.fire <= 0.0 {
                    continue;
                }
                let quench = if b.chill > 0.0 { 0.3 } else { 0.0 };
                b.fire = (b.fire + (flam * 0.12 - 0.04 - quench) * dt).clamp(0.0, 1.0);
                if b.fire < 0.01 {
                    b.fire = 0.0;
                    continue;
                }
                b.heat += b.fire * 10.0 * dt;
                let was_alive = b.hp > 0.0;
                b.hp -= b.fire * 8.0 * dt;
                if was_alive && b.hp <= 0.0 {
                    killed.push(s);
                }
                let fire = b.fire;
                for &n in &body.topo.neighbors[s as usize] {
                    let nf = body.block(n).unwrap().flammability(&content);
                    if nf > 0.0 && world.rng.chance(fire * nf * 0.25 * dt) {
                        ignite.push(n);
                    }
                }
            }
        }
        let body = world.bodies.get_mut(&id).unwrap();
        for n in ignite {
            if let Some(b) = body.block_mut(n) {
                if b.chill <= 0.0 {
                    b.fire = b.fire.max(0.15);
                }
            }
        }
        for s in killed {
            world.dead.push((id, s));
        }
    }
}

/// Remove destroyed blocks, chain-explode flammable stock, split broken
/// bodies and turn core-less ships into derelicts.
pub fn process_destruction(world: &mut World) {
    let content = world.content.clone();
    for _round in 0..8 {
        if world.dead.is_empty() {
            return;
        }
        let mut dead = std::mem::take(&mut world.dead);
        dead.sort_unstable();
        dead.dedup();
        let mut touched: Vec<BodyId> = Vec::new();
        let mut blasts: Vec<(Vec2, f32, f32)> = Vec::new();
        for (bid, slot) in dead {
            let Some(body) = world.bodies.get_mut(&bid) else { continue };
            let Some(inst) = body.block(slot) else { continue };
            if inst.hp > 0.0 {
                continue;
            }
            let pos = body.to_world(inst.center());
            let inst = body.remove_block(slot).unwrap();
            let flam_stock: f32 = inst
                .stock
                .iter()
                .map(|(r, q)| content.resources[*r as usize].flammability * q)
                .sum();
            if flam_stock > 5.0 {
                let radius = 1.0 + (flam_stock / 40.0).min(3.0);
                let dmg = (15.0 + flam_stock * 0.5).min(120.0);
                blasts.push((pos, radius, dmg));
            }
            world.events.push(Event::BlockDestroyed { body: bid, pos, def: inst.def });
            if matches!(content.blocks[inst.def as usize].kind, Block::Core) && body.kind == BodyKind::Ship {
                let by = body.last_hit_by;
                let was = body.faction;
                body.kind = BodyKind::Derelict;
                body.faction = None;
                body.ai = None;
                body.control = Default::default();
                if by == Some(PLAYER_FACTION) && was != Some(PLAYER_FACTION) {
                    world.progress.ships_destroyed += 1;
                }
                world.events.push(Event::ShipDestroyed { body: bid, by });
            }
            if !touched.contains(&bid) {
                touched.push(bid);
            }
        }
        touched.sort_unstable();
        for bid in touched {
            split_if_needed(world, bid);
        }
        for (pos, radius, dmg) in blasts {
            world.events.push(Event::Explosion { pos, radius });
            let fx = Effects { incendiary: 0.5, ..Default::default() };
            splash(world, pos, radius, dmg, None, None, &fx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::Control;
    use crate::world::Command;
    use std::sync::Arc;

    fn world() -> World {
        World::new(Arc::new(ContentDb::load_dir(gf_content::default_content_dir()).unwrap()), 5)
    }

    #[test]
    fn dda_hits_first_cell_along_segment() {
        let mut w = world();
        let c = w.content.clone();
        let id = w.spawn_blueprint("micro_bandit", Vec2::ZERO, 0.0, None).unwrap();
        let b = &w.bodies[&id];
        let from = b.pos + Vec2::new(-30.0, 0.0);
        let hit = raycast(&w, from, b.pos + Vec2::new(30.0, 0.0), |_| false).expect("hit");
        assert!(hit.point.x < b.pos.x);
        let _ = c;
        w.step();
    }

    #[test]
    fn fed_cannon_grows_then_fires_at_enemy() {
        let mut w = world();
        let p = w.spawn_blueprint("forge_runner", Vec2::ZERO, 0.0, Some(PLAYER_FACTION)).unwrap();
        let e = w.spawn_blueprint("micro_bandit", Vec2::new(0.0, 30.0), 0.0, Some(1)).unwrap();
        w.bodies.get_mut(&e).unwrap().ai = None;
        let mut max_tier = 1;
        let mut shots = 0;
        for _ in 0..300 {
            let aim = w.bodies.get(&e).map_or(Vec2::new(0.0, 30.0), |b| b.pos);
            w.queue(Command::Control { body: p, control: Control { fire: true, aim: Some(aim), ..Default::default() } });
            w.step();
            shots += w.events.iter().filter(|e| matches!(e, Event::Shot { .. })).count();
            for b in w.bodies[&p].blocks.iter().flatten() {
                if matches!(w.content.blocks[b.def as usize].kind, Block::Weapon(_)) {
                    max_tier = max_tier.max(b.feed.tier);
                }
            }
        }
        assert!(shots > 5, "fired {shots} shots");
        assert!(max_tier >= 2, "a well-fed cannon grows (max tier {max_tier})");
    }

    #[test]
    fn fire_spreads_and_frost_cannon_suppresses() {
        let mut w = world();
        let c = w.content.clone();
        let p = w.spawn_blueprint("forge_runner", Vec2::ZERO, 0.0, Some(PLAYER_FACTION)).unwrap();
        // Load cannons with coolant so they qualify for suppression.
        let coolant = c.res("coolant").unwrap();
        let body = w.bodies.get_mut(&p).unwrap();
        body.control.auto_suppress = true;
        let cannons = body.slots_where(&c, |k| matches!(k, Block::Weapon(wd) if wd.def.kind == WeaponKind::Cannon));
        for &s in &cannons {
            let b = body.block_mut(s).unwrap();
            b.stock.clear();
            b.add(coolant, 3.0);
            b.ammo_mix = vec![(coolant, 10.0)];
        }
        // Set a fuel tank on fire.
        let tank = body.slots_where(&c, |k| matches!(k, Block::Tank { .. }))[0];
        body.block_mut(tank).unwrap().fire = 0.8;
        let mut extinguished = false;
        for _ in 0..90 {
            let body = w.bodies.get_mut(&p).unwrap();
            for &s in &cannons {
                if let Some(b) = body.block_mut(s) {
                    if b.amount(coolant) < 2.0 {
                        b.add(coolant, 2.0);
                    }
                    b.ammo_mix = vec![(coolant, 10.0)];
                }
            }
            w.queue(Command::Control { body: p, control: Control { auto_suppress: true, ..Default::default() } });
            w.step();
            if w.events.iter().any(|e| matches!(e, Event::Extinguish { .. })) {
                extinguished = true;
            }
        }
        assert!(extinguished, "frost-fed cannon put out the fire");
        assert!(w.bodies[&p].block(tank).is_none_or(|b| b.fire < 0.2));
    }

    #[test]
    fn destroyed_core_makes_derelict_and_splits() {
        let mut w = world();
        let c = w.content.clone();
        let e = w.spawn_blueprint("raider", Vec2::ZERO, 0.0, Some(1)).unwrap();
        let core = w.bodies[&e].slots_where(&c, |k| matches!(k, Block::Core))[0];
        w.bodies.get_mut(&e).unwrap().last_hit_by = Some(PLAYER_FACTION);
        damage_block(&mut w, e, core, 10_000.0, Some(PLAYER_FACTION));
        process_destruction(&mut w);
        assert_eq!(w.bodies[&e].kind, BodyKind::Derelict);
        assert_eq!(w.progress.ships_destroyed, 1);
        // Cut a line through the derelict to force a split.
        let before = w.bodies.len();
        let ids: Vec<u32> = w.bodies[&e].topo.live.clone();
        let row_y = w.bodies[&e].block(ids[0]).unwrap().anchor.y;
        for s in ids {
            if w.bodies[&e].block(s).unwrap().cells().any(|c| c.y == row_y) {
                damage_block(&mut w, e, s, 10_000.0, None);
            }
        }
        process_destruction(&mut w);
        assert!(w.bodies.len() >= before, "pieces survive as bodies");
    }
}
