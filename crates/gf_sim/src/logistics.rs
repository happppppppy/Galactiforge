//! Three logistics layers – items, fluids and power – plus recipe crafting.
//!
//! * Items move one unit at a time: producers (machines, drills, salvagers,
//!   mining lasers) offload round-robin into adjacent acceptors; conveyors carry
//!   items forward and unload storage placed directly behind them; storage
//!   feeds adjacent consumers (machines, weapons, generators, labs).
//! * Fluids flow through networks of pipes and tanks. Each connected network is
//!   solved once per tick: producer outputs and tank contents are pooled and
//!   shared out to attached consumers, surplus is stored in tanks.
//! * Power flows through networks of cables (plus the core, generators, solar
//!   panels and batteries). Supply is shared out by priority class
//!   (weapons → factories → utilities); shortfalls brown out the lower classes
//!   proportionally and batteries buffer the difference.

use crate::body::{BlockInst, Body};
use crate::math::{Cell, facing, rotate_offset};
use gf_content::{Block, ContentDb, ResId, ResourceKind};

/// Power supplied by every ship core (kW).
pub const CORE_POWER: f32 = 25.0;
pub const CONVEYOR_CAPACITY: f32 = 3.0;
const PUSH_PER_TICK: u32 = 2;
const CONSUMER_FUEL_CAP: f32 = 10.0;

pub fn weapon_ammo_cap(content: &ContentDb, b: &BlockInst) -> f32 {
    if let Block::Weapon(w) = &content.blocks[b.def as usize].kind {
        let cap = (b.feed.cap as usize).clamp(1, w.def.growth.len());
        (w.def.buffer_secs * w.def.growth[cap - 1].demand).max(4.0)
    } else {
        0.0
    }
}

pub fn thruster_fuel_cap(content: &ContentDb, b: &BlockInst) -> f32 {
    if let Block::Thruster(t) = &content.blocks[b.def as usize].kind {
        let cap = (b.feed.cap as usize).clamp(1, t.def.growth.len());
        (2.0 * t.def.growth[cap - 1].demand).max(4.0)
    } else {
        0.0
    }
}

/// Record a delivery of ammo for the ammo-mix modifiers.
pub fn note_ammo(b: &mut BlockInst, r: ResId, q: f32) {
    match b.ammo_mix.iter_mut().find(|e| e.0 == r) {
        Some(e) => e.1 += q,
        None => {
            b.ammo_mix.push((r, q));
            b.ammo_mix.sort_by_key(|e| e.0);
        }
    }
}

fn is_storage(content: &ContentDb, b: &BlockInst) -> bool {
    matches!(content.blocks[b.def as usize].kind, Block::Storage { .. })
}

/// Free space for item `r` arriving from block `from` (None = internal).
fn item_space(content: &ContentDb, body: &Body, target: u32, r: ResId, from: Option<u32>) -> f32 {
    if content.res_kind(r) != ResourceKind::Item {
        return 0.0;
    }
    let Some(b) = body.block(target) else { return 0.0 };
    match &content.blocks[b.def as usize].kind {
        Block::Conveyor { .. } | Block::Splitter { .. } => {
            if let (Some(f), Block::Conveyor { .. }) = (from, &content.blocks[b.def as usize].kind) {
                if body.block_at(b.anchor.offset(facing(b.rot))) == Some(f) {
                    return 0.0;
                }
            }
            CONVEYOR_CAPACITY - b.total_stock()
        }
        Block::Storage { capacity, filter } => {
            if !filter.is_empty() && !filter.contains(&r) {
                return 0.0;
            }
            *capacity as f32 - b.total_stock()
        }
        Block::Machine { buffer, .. } => match b.recipe {
            Some(rid) if content.recipes[rid as usize].inputs.iter().any(|(i, _)| *i == r) => *buffer as f32 - b.amount(r),
            _ => 0.0,
        },
        Block::Weapon(w) => {
            if w.ammo.contains(&r) {
                weapon_ammo_cap(content, b) - b.total_stock()
            } else {
                0.0
            }
        }
        Block::Generator { fuel: Some((f, _)), .. } if *f == r => CONSUMER_FUEL_CAP - b.amount(r),
        Block::Lab { .. } if content.resources[r as usize].research_value > 0.0 => CONSUMER_FUEL_CAP - b.amount(r),
        _ => 0.0,
    }
}

fn deliver(content: &ContentDb, body: &mut Body, target: u32, r: ResId, q: f32) {
    let b = body.block_mut(target).unwrap();
    b.add(r, q);
    if let Block::Weapon(_) = content.blocks[b.def as usize].kind {
        b.feed.delivered += q;
        note_ammo(b, r, q);
    }
}

/// Items this block is willing to offload this tick.
fn offload_items(content: &ContentDb, b: &BlockInst) -> Vec<ResId> {
    match &content.blocks[b.def as usize].kind {
        Block::Machine { .. } => match b.recipe {
            Some(rid) => content.recipes[rid as usize]
                .outputs
                .iter()
                .filter(|(r, _)| content.res_kind(*r) == ResourceKind::Item && b.amount(*r) >= 1.0)
                .map(|(r, _)| *r)
                .collect(),
            None => Vec::new(),
        },
        Block::Drill { .. } | Block::Salvager { .. } | Block::Storage { .. } => {
            b.stock.iter().filter(|(_, q)| *q >= 1.0).map(|(r, _)| *r).collect()
        }
        Block::Weapon(w) if w.def.kind == gf_content::WeaponKind::MiningLaser => {
            b.stock.iter().filter(|(_, q)| *q >= 1.0).map(|(r, _)| *r).collect()
        }
        _ => Vec::new(),
    }
}

pub fn step_items(content: &ContentDb, body: &mut Body, dt: f32) {
    let live = body.topo.live.clone();
    // 1. Conveyors & splitters move items forward / unload storage behind them.
    for &s in &live {
        let b = body.block(s).unwrap();
        let kind = &content.blocks[b.def as usize].kind;
        let speed = match kind {
            Block::Conveyor { speed } | Block::Splitter { speed } => *speed,
            _ => continue,
        };
        let is_splitter = matches!(kind, Block::Splitter { .. });
        let slow = if b.chill > 0.0 { 0.5 } else { 1.0 };
        let anchor = b.anchor;
        let rot = b.rot;
        // Unload a storage block directly behind (loader behaviour).
        let back = anchor.offset(rotate_offset((0, -1), rot));
        if let Some(src) = body.block_at(back) {
            if is_storage(content, body.block(src).unwrap()) && body.block(s).unwrap().total_stock() < CONVEYOR_CAPACITY {
                let items = offload_items(content, body.block(src).unwrap());
                if !items.is_empty() {
                    let rr = body.block(s).unwrap().rr as usize;
                    let r = items[rr % items.len()];
                    let got = body.block_mut(src).unwrap().take(r, 1.0);
                    let me = body.block_mut(s).unwrap();
                    me.add(r, got);
                    me.rr = me.rr.wrapping_add(1);
                }
            }
        }
        let me = body.block_mut(s).unwrap();
        me.move_acc = (me.move_acc + speed * slow * dt).min(2.0);
        while body.block(s).unwrap().move_acc >= 1.0 {
            let me = body.block(s).unwrap();
            let Some(&(r, q)) = me.stock.first() else { break };
            if q < 1.0 {
                break;
            }
            let dirs: &[(i32, i32)] = if is_splitter { &[(0, 1), (-1, 0), (1, 0)] } else { &[(0, 1)] };
            let mut moved = false;
            for k in 0..dirs.len() {
                let d = dirs[(me.rr as usize + k) % dirs.len()];
                let tc: Cell = anchor.offset(rotate_offset(d, rot));
                let Some(t) = body.block_at(tc) else { continue };
                if item_space(content, body, t, r, Some(s)) >= 1.0 {
                    body.block_mut(s).unwrap().take(r, 1.0);
                    deliver(content, body, t, r, 1.0);
                    let me = body.block_mut(s).unwrap();
                    me.rr = me.rr.wrapping_add(k as u32 + 1);
                    moved = true;
                    break;
                }
            }
            if !moved {
                break;
            }
            body.block_mut(s).unwrap().move_acc -= 1.0;
        }
    }

    // 2. Producers and storage offload into adjacent acceptors.
    for &s in &live {
        let b = body.block(s).unwrap();
        let items = offload_items(content, b);
        if items.is_empty() {
            continue;
        }
        let from_storage = is_storage(content, b);
        let neighbors = body.topo.neighbors[s as usize].clone();
        if neighbors.is_empty() {
            continue;
        }
        let mut pushed = 0;
        let mut attempts = 0;
        let total = neighbors.len() * items.len();
        while pushed < PUSH_PER_TICK && attempts < total {
            let rr = body.block(s).unwrap().rr as usize;
            let t = neighbors[rr % neighbors.len()];
            let r = items[(rr / neighbors.len()) % items.len()];
            body.block_mut(s).unwrap().rr = (rr as u32).wrapping_add(1);
            attempts += 1;
            if body.block(s).unwrap().amount(r) < 1.0 {
                continue;
            }
            let tb = body.block(t).unwrap();
            let tkind = &content.blocks[tb.def as usize].kind;
            if from_storage
                && matches!(
                    tkind,
                    Block::Storage { .. } | Block::Conveyor { .. } | Block::Splitter { .. }
                )
            {
                continue;
            }
            if item_space(content, body, t, r, Some(s)) >= 1.0 {
                body.block_mut(s).unwrap().take(r, 1.0);
                deliver(content, body, t, r, 1.0);
                pushed += 1;
            }
        }
    }
}

fn can_craft(content: &ContentDb, kind: &Block, b: &BlockInst) -> bool {
    let Block::Machine { buffer, .. } = kind else { return false };
    let Some(rid) = b.recipe else { return false };
    let rec = &content.recipes[rid as usize];
    b.disabled <= 0.0
        && rec.inputs.iter().all(|(r, q)| b.amount(*r) >= *q - 1e-4)
        && rec.outputs.iter().all(|(r, q)| b.amount(*r) + q <= *buffer as f32 * 2.0)
}

pub fn step_machines(content: &ContentDb, body: &mut Body, dt: f32) -> Vec<(ResId, f32)> {
    let mut produced = Vec::new();
    for &s in &body.topo.live.clone() {
        let b = body.block(s).unwrap();
        let kind = &content.blocks[b.def as usize].kind;
        if !can_craft(content, kind, b) {
            continue;
        }
        let rec = &content.recipes[b.recipe.unwrap() as usize];
        let mut speed = if rec.power > 0.0 { b.power_sat } else { 1.0 };
        if b.chill > 0.0 {
            speed *= 0.5;
        }
        let b = body.block_mut(s).unwrap();
        b.progress += dt * speed / rec.time;
        if b.progress >= 1.0 {
            b.progress -= 1.0;
            for (r, q) in &rec.inputs {
                b.take(*r, *q);
            }
            for (r, q) in &rec.outputs {
                b.add(*r, *q);
                produced.push((*r, *q));
            }
            b.heat += 2.0;
        }
    }
    produced
}

/// Priority class for power allocation (lower = served first).
fn power_class(kind: &Block) -> usize {
    match kind {
        Block::Weapon(w) if w.def.kind != gf_content::WeaponKind::MiningLaser => 0,
        Block::Thruster(_) => 0,
        Block::Machine { .. } => 1,
        _ => 2,
    }
}

pub fn step_power(content: &ContentDb, body: &mut Body, dt: f32) {
    // Machines request power when they are able to work.
    for &s in &body.topo.live.clone() {
        let b = body.block(s).unwrap();
        let kind = &content.blocks[b.def as usize].kind;
        let req = match kind {
            Block::Machine { .. } => {
                if can_craft(content, kind, b) {
                    content.recipes[b.recipe.unwrap() as usize].power
                } else {
                    0.0
                }
            }
            Block::Weapon(w) if w.def.kind == gf_content::WeaponKind::Rocket => w.def.power * b.feed.tier as f32,
            _ => b.power_req,
        };
        let b = body.block_mut(s).unwrap();
        b.power_req = req;
        b.power_sat = 0.0;
    }
    let nets = std::mem::take(&mut body.topo.power_nets);
    for net in &nets {
        let mut gen_cap = 0.0;
        let mut batt_out = 0.0;
        let mut batt_in = 0.0;
        let mut demand = [0.0f32; 3];
        for &s in net.conductors.iter().chain(net.attached.iter()) {
            let b = body.block(s).unwrap();
            let kind = &content.blocks[b.def as usize].kind;
            if b.disabled > 0.0 {
                continue;
            }
            match kind {
                Block::Core => gen_cap += CORE_POWER,
                Block::Generator { output, fuel } => {
                    let fueled = match fuel {
                        Some((f, _)) => b.amount(*f) > 0.0,
                        None => true,
                    };
                    if fueled {
                        gen_cap += output;
                    }
                }
                Block::Solar { output } => gen_cap += output,
                Block::Battery { capacity, rate } => {
                    batt_out += rate.min(b.energy / dt);
                    batt_in += rate.min((capacity - b.energy) / dt);
                }
                _ => {}
            }
            if b.power_req > 0.0 {
                demand[power_class(kind)] += b.power_req;
            }
        }
        let total_demand: f32 = demand.iter().sum();
        let mut avail = gen_cap + batt_out;
        let mut sat = [0.0f32; 3];
        for c in 0..3 {
            if demand[c] <= 0.0 {
                sat[c] = 1.0;
                continue;
            }
            let give = avail.min(demand[c]);
            sat[c] = give / demand[c];
            avail -= give;
        }
        let used = total_demand.min(gen_cap + batt_out);
        let from_gen = used.min(gen_cap);
        let from_batt = used - from_gen;
        let surplus = (gen_cap - from_gen).min(batt_in);
        let load = if gen_cap > 0.0 { (from_gen + surplus) / gen_cap } else { 0.0 };
        let n_batt = net
            .conductors
            .iter()
            .filter(|s| matches!(content.blocks[body.block(**s).unwrap().def as usize].kind, Block::Battery { .. }))
            .count()
            .max(1) as f32;
        for &s in net.conductors.iter().chain(net.attached.iter()) {
            let def = body.block(s).unwrap().def;
            let kind = &content.blocks[def as usize].kind;
            let b = body.block_mut(s).unwrap();
            match kind {
                Block::Generator { fuel: Some((f, rate)), .. } if b.amount(*f) > 0.0 => {
                    b.take(*f, rate * load * dt);
                    b.heat += 1.0 * load * dt;
                }
                Block::Battery { capacity, .. } => {
                    b.energy = (b.energy - from_batt / n_batt * dt + surplus / n_batt * dt).clamp(0.0, *capacity);
                }
                _ => {}
            }
            if b.power_req > 0.0 {
                b.power_sat = b.power_sat.max(sat[power_class(kind)]);
            }
        }
    }
    body.topo.power_nets = nets;
}

pub fn step_fluids(content: &ContentDb, body: &mut Body) {
    let nets = std::mem::take(&mut body.topo.fluid_nets);
    for net in &nets {
        // (slot, res, amount)
        let mut requests: Vec<(u32, ResId, f32)> = Vec::new();
        let mut supplies: Vec<(u32, ResId, f32)> = Vec::new();
        for &s in &net.attached {
            let b = body.block(s).unwrap();
            match &content.blocks[b.def as usize].kind {
                Block::Machine { buffer, .. } => {
                    if let Some(rid) = b.recipe {
                        let rec = &content.recipes[rid as usize];
                        for (r, _) in &rec.inputs {
                            if content.res_kind(*r) == ResourceKind::Fluid {
                                let space = *buffer as f32 - b.amount(*r);
                                if space > 0.0 {
                                    requests.push((s, *r, space));
                                }
                            }
                        }
                        for (r, _) in &rec.outputs {
                            if content.res_kind(*r) == ResourceKind::Fluid && b.amount(*r) > 0.0 {
                                supplies.push((s, *r, b.amount(*r)));
                            }
                        }
                    }
                }
                Block::Thruster(t) => {
                    let cap = thruster_fuel_cap(content, b);
                    let space = cap - b.amount(t.fuel);
                    if space > 0.0 {
                        requests.push((s, t.fuel, space));
                    }
                    if let Some(o) = t.oxidizer {
                        let space = cap * 0.5 - b.amount(o);
                        if space > 0.0 {
                            requests.push((s, o, space));
                        }
                    }
                }
                Block::Generator { fuel: Some((f, _)), .. } if content.res_kind(*f) == ResourceKind::Fluid => {
                    let space = CONSUMER_FUEL_CAP * 2.0 - b.amount(*f);
                    if space > 0.0 {
                        requests.push((s, *f, space));
                    }
                }
                Block::Weapon(w) => {
                    let space = weapon_ammo_cap(content, b) - b.total_stock();
                    if space > 0.0 {
                        for &r in &w.ammo {
                            if content.res_kind(r) == ResourceKind::Fluid {
                                requests.push((s, r, space));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        let mut res_list: Vec<ResId> = requests
            .iter()
            .map(|e| e.1)
            .chain(supplies.iter().map(|e| e.1))
            .chain(
                net.conductors
                    .iter()
                    .flat_map(|s| body.block(*s).unwrap().stock.iter().map(|e| e.0)),
            )
            .collect();
        res_list.sort_unstable();
        res_list.dedup();
        for r in res_list {
            let supply: f32 = supplies.iter().filter(|e| e.1 == r).map(|e| e.2).sum();
            let in_tanks: f32 = net.conductors.iter().map(|s| body.block(*s).unwrap().amount(r)).sum();
            let req_total: f32 = requests.iter().filter(|e| e.1 == r).map(|e| e.2).sum();
            let pool = supply + in_tanks;
            if pool <= 0.0 {
                continue;
            }
            let give = pool.min(req_total);
            if give > 0.0 {
                let ratio = give / req_total;
                for &(s, rr, q) in requests.iter().filter(|e| e.1 == r) {
                    let amt = q * ratio;
                    let def = body.block(s).unwrap().def;
                    let b = body.block_mut(s).unwrap();
                    b.add(rr, amt);
                    match &content.blocks[def as usize].kind {
                        Block::Thruster(t) if t.fuel == rr => b.feed.delivered += amt,
                        Block::Weapon(_) => {
                            b.feed.delivered += amt;
                            note_ammo(b, rr, amt);
                        }
                        _ => {}
                    }
                }
            }
            // Pull everything out of producers and tanks, then refill tanks.
            for &(s, rr, _) in supplies.iter().filter(|e| e.1 == r) {
                body.block_mut(s).unwrap().take(rr, f32::MAX);
            }
            for &s in &net.conductors {
                body.block_mut(s).unwrap().take(r, f32::MAX);
            }
            let mut remaining = pool - give;
            for &s in &net.conductors {
                if remaining <= 0.0 {
                    break;
                }
                let def = body.block(s).unwrap().def;
                if let Block::Tank { capacity, filter } = &content.blocks[def as usize].kind {
                    if !filter.is_empty() && !filter.contains(&r) {
                        continue;
                    }
                    let b = body.block_mut(s).unwrap();
                    let space = (capacity - b.total_stock()).max(0.0);
                    let put = space.min(remaining);
                    b.add(r, put);
                    remaining -= put;
                }
            }
            // Anything that does not fit goes back to the producers.
            if remaining > 0.0 && supply > 0.0 {
                for &(s, rr, q) in supplies.iter().filter(|e| e.1 == r) {
                    body.block_mut(s).unwrap().add(rr, remaining * q / supply);
                }
            }
        }
    }
    body.topo.fluid_nets = nets;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{Body, BodyKind};
    use crate::math::{Cell, Vec2};

    fn db() -> ContentDb {
        ContentDb::load_dir(gf_content::default_content_dir()).unwrap()
    }

    fn put(body: &mut Body, db: &ContentDb, id: &str, x: i32, y: i32, rot: u8) -> u32 {
        body.insert(db, db.block(id).unwrap(), Cell::new(x, y), rot).unwrap()
    }

    #[test]
    fn conveyor_chain_moves_items_from_storage_to_machine() {
        let db = db();
        let mut b = Body::new(1, "t", BodyKind::Ship, Some(0), Vec2::ZERO, 0.0);
        let core = put(&mut b, &db, "ship_core", -1, 0, 0);
        let store = put(&mut b, &db, "ore_container", 0, 0, 0);
        put(&mut b, &db, "conveyor", 0, 1, 0);
        put(&mut b, &db, "conveyor", 0, 2, 0);
        let crusher = put(&mut b, &db, "crusher", 0, 3, 0);
        put(&mut b, &db, "cable", -1, 1, 0);
        put(&mut b, &db, "cable", -1, 2, 0);
        put(&mut b, &db, "cable", -1, 3, 0);
        let _ = core;
        let ore = db.res("ore").unwrap();
        b.block_mut(store).unwrap().add(ore, 20.0);
        b.refresh(&db);
        let dt = 1.0 / 30.0;
        for _ in 0..120 {
            step_power(&db, &mut b, dt);
            step_machines(&db, &mut b, dt);
            step_items(&db, &mut b, dt);
        }
        let ts = db.res("titanstone").unwrap();
        let produced = b.block(crusher).unwrap().amount(ts);
        assert!(produced >= 2.0, "crusher produced {produced}");
        assert!(b.block(store).unwrap().amount(ore) < 20.0);
    }

    #[test]
    fn fluid_network_feeds_thruster_and_records_delivery() {
        let db = db();
        let mut b = Body::new(1, "t", BodyKind::Ship, Some(0), Vec2::ZERO, 0.0);
        let tank = put(&mut b, &db, "fuel_container", 0, 0, 0);
        put(&mut b, &db, "pipe", 0, -1, 0);
        let thr = put(&mut b, &db, "large_thruster", 0, -2, 0);
        let fuel = db.res("fuel").unwrap();
        b.block_mut(tank).unwrap().add(fuel, 100.0);
        b.refresh(&db);
        step_fluids(&db, &mut b);
        let t = b.block(thr).unwrap();
        assert!(t.amount(fuel) > 0.0);
        assert!(t.feed.delivered > 0.0);
        let total = b.block(tank).unwrap().amount(fuel) + t.amount(fuel);
        assert!((total - 100.0).abs() < 1e-3, "fluid conserved: {total}");
    }

    #[test]
    fn power_brownout_prioritises_weapons() {
        let db = db();
        let mut b = Body::new(1, "t", BodyKind::Ship, Some(0), Vec2::ZERO, 0.0);
        put(&mut b, &db, "ship_core", 0, 0, 0);
        let rl = put(&mut b, &db, "rocket_launcher", 1, 0, 0);
        let m = put(&mut b, &db, "arc_furnace", -1, 0, 0);
        b.refresh(&db);
        let ice = db.res("ice").unwrap();
        let rid = db.recipe("electrolysis").unwrap();
        {
            let mb = b.block_mut(m).unwrap();
            mb.recipe = Some(rid);
            mb.add(ice, 5.0);
        }
        b.block_mut(rl).unwrap().feed.tier = 5; // 25 kW demand
        step_power(&db, &mut b, 1.0 / 30.0);
        assert!((b.block(rl).unwrap().power_sat - 1.0).abs() < 1e-4);
        assert!(b.block(m).unwrap().power_sat < 0.01, "factory browned out");
    }
}
