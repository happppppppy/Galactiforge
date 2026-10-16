//! The simulation world: owns every body, projectile and progression state
//! and advances them in a fixed, deterministic order.

use crate::ai;
use crate::body::{Body, BodyId, BodyKind, Control, Faction, PLAYER_FACTION, WeaponMode};
use crate::combat::{self, Projectile};
use crate::logistics;
use crate::math::{Cell, Vec2};
use crate::mining;
use crate::progress::Progress;
use crate::rng::Rng;
use crate::sector::Sectors;
use crate::thrust;
use gf_content::{Block, BlockId, ContentDb, RecipeId, ResearchId, ResId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Simulation rate. Everything uses this fixed step.
pub const TICK_RATE: u32 = 30;
pub const DT: f32 = 1.0 / TICK_RATE as f32;

/// Presentation-only notifications produced by a tick. They never feed back
/// into the simulation, so clients may drop them freely.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Shot { body: BodyId, pos: Vec2, dir: Vec2, def: BlockId, tier: u8 },
    Hit { pos: Vec2, damage: f32 },
    Laser { from: Vec2, to: Vec2, tier: u8 },
    BlockDestroyed { body: BodyId, pos: Vec2, def: BlockId },
    Explosion { pos: Vec2, radius: f32 },
    Extinguish { pos: Vec2, radius: f32 },
    TierChanged { body: BodyId, slot: u32, tier: u8 },
    ShipDestroyed { body: BodyId, by: Option<Faction> },
    Salvaged { body: BodyId, pos: Vec2 },
    ResearchComplete { research: ResearchId },
    MissionComplete { mission: usize },
    OriginShift { delta: Vec2 },
    Message(String),
}

/// Every player / AI input enters the simulation as a command, applied at the
/// start of the next tick. This is the lock-step / rollback boundary for
/// future multiplayer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    Control { body: BodyId, control: Control },
    Build { body: BodyId, block: BlockId, cell: Cell, rot: u8 },
    Deconstruct { body: BodyId, cell: Cell },
    SetRecipe { body: BodyId, cell: Cell, recipe: RecipeId },
    CycleRecipe { body: BodyId, cell: Cell },
    SetWeaponMode { body: BodyId, cell: Cell, mode: WeaponMode },
    StartResearch { research: ResearchId },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    NoBody,
    Locked,
    Unaffordable,
    Blocked,
    NothingThere,
    Invalid,
}

fn empty_content() -> Arc<ContentDb> {
    Arc::new(ContentDb::resolve(Default::default()).expect("empty content resolves"))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct World {
    #[serde(skip, default = "empty_content")]
    pub content: Arc<ContentDb>,
    pub seed: u64,
    pub tick: u64,
    pub rng: Rng,
    pub next_id: BodyId,
    pub bodies: BTreeMap<BodyId, Body>,
    pub projectiles: Vec<Projectile>,
    pub player: Option<BodyId>,
    pub progress: Progress,
    pub sectors: Sectors,
    #[serde(skip)]
    pub events: Vec<Event>,
    #[serde(skip)]
    pending: Vec<Command>,
    #[serde(skip)]
    pub(crate) dead: Vec<(BodyId, u32)>,
}

impl World {
    /// An empty world (no sectors generated).
    pub fn new(content: Arc<ContentDb>, seed: u64) -> Self {
        let progress = Progress::new(&content);
        Self {
            content,
            seed,
            tick: 0,
            rng: Rng::new(seed),
            next_id: 1,
            bodies: BTreeMap::new(),
            projectiles: Vec::new(),
            player: None,
            progress,
            sectors: Sectors::default(),
            events: Vec::new(),
            pending: Vec::new(),
            dead: Vec::new(),
        }
    }

    /// A new open-world game: the player's starter ship in the safe home sector.
    pub fn new_game(content: Arc<ContentDb>, seed: u64) -> Self {
        let mut w = Self::new(content, seed);
        let p = w
            .spawn_blueprint("forge_runner", Vec2::ZERO, 0.0, Some(PLAYER_FACTION))
            .expect("starter blueprint");
        if let Some(b) = w.bodies.get_mut(&p) {
            b.control.auto_suppress = true;
        }
        w.player = Some(p);
        w.sectors.enabled = true;
        crate::sector::stream(&mut w);
        w
    }

    pub fn alloc_id(&mut self) -> BodyId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_body(&mut self, mut body: Body) -> BodyId {
        let id = self.alloc_id();
        body.id = id;
        body.refresh(&self.content);
        self.bodies.insert(id, body);
        id
    }

    /// Build a body from a blueprint. Starting stock is distributed into
    /// matching storage and tanks.
    pub fn spawn_blueprint(&mut self, id: &str, pos: Vec2, angle: f32, faction: Option<Faction>) -> Option<BodyId> {
        let content = self.content.clone();
        let bp = content.blueprint(id)?;
        let kind = if faction.is_some() { BodyKind::Ship } else { BodyKind::Derelict };
        let mut body = Body::new(0, bp.name.clone(), kind, faction, pos, angle);
        for bb in &bp.blocks {
            let def = content.block(&bb.block)?;
            if let Ok(slot) = body.insert(&content, def, Cell::new(bb.x, bb.y), bb.rot) {
                if let Some(r) = bb.recipe.as_deref().and_then(|r| content.recipe(r)) {
                    body.block_mut(slot).unwrap().recipe = Some(r);
                }
            }
        }
        body.refresh(&content);
        // Keep the blueprint origin at `pos` rather than the centre of mass.
        body.pos = pos + body.com.rotate(angle);
        for (rid, qty) in &bp.stock {
            let Some(r) = content.res(rid) else { continue };
            fill_storage(&content, &mut body, r, *qty);
        }
        if faction.is_some_and(|f| f != PLAYER_FACTION) {
            body.ai = Some(ai::AiState::new(pos, body.block_count()));
            body.control.auto_suppress = true;
        }
        Some(self.add_body(body))
    }

    pub fn queue(&mut self, cmd: Command) {
        self.pending.push(cmd);
    }

    pub fn player_body(&self) -> Option<&Body> {
        self.player.and_then(|p| self.bodies.get(&p))
    }

    /// Advance the simulation by one fixed tick.
    pub fn step(&mut self) {
        self.events.clear();
        let content = self.content.clone();
        let dt = DT;

        for cmd in std::mem::take(&mut self.pending) {
            if let Err(e) = self.apply(cmd) {
                self.events.push(Event::Message(format!("command rejected: {e:?}")));
            }
        }
        if self.tick % TICK_RATE as u64 == 0 {
            crate::sector::stream(self);
        }
        ai::think(self, dt);

        let ids: Vec<BodyId> = self.bodies.keys().copied().collect();
        // Logistics, crafting and growth.
        for &id in &ids {
            let body = self.bodies.get_mut(&id).unwrap();
            body.refresh(&content);
            if body.kind == BodyKind::Asteroid {
                continue;
            }
            logistics::step_power(&content, body, dt);
            logistics::step_fluids(&content, body);
            logistics::step_machines(&content, body, dt);
            logistics::step_items(&content, body, dt);
            mining::step_labs(&content, body, &mut self.progress, dt);
            combat::update_tiers(&content, body, dt, &mut self.events);
            body.refresh(&content);
        }

        thrust::apply_thrusters(self, dt);
        combat::step_weapons(self, dt);
        mining::step_tools(self, dt);
        combat::step_projectiles(self, dt);
        combat::step_fire(self, dt);
        combat::process_destruction(self);
        thrust::integrate(self, dt);
        thrust::collide(self, dt);
        combat::process_destruction(self);
        crate::progress::step(self, dt);
        crate::sector::floating_origin(self);
        self.tick += 1;
    }

    fn apply(&mut self, cmd: Command) -> Result<(), CommandError> {
        let content = self.content.clone();
        match cmd {
            Command::Control { body, control } => {
                self.bodies.get_mut(&body).ok_or(CommandError::NoBody)?.control = control;
            }
            Command::Build { body, block, cell, rot } => {
                let info = content.blocks.get(block as usize).ok_or(CommandError::Invalid)?;
                if !info.def.buildable {
                    return Err(CommandError::Invalid);
                }
                let b = self.bodies.get(&body).ok_or(CommandError::NoBody)?;
                if b.faction == Some(PLAYER_FACTION) && !self.progress.unlocked.contains(&block) {
                    return Err(CommandError::Locked);
                }
                if b.kind != BodyKind::Ship {
                    return Err(CommandError::Invalid);
                }
                let cost: Vec<(ResId, f32)> = info.cost.iter().map(|(r, q)| (*r, *q as f32)).collect();
                if cost.iter().any(|(r, q)| b.stored(&content, *r) + 1e-3 < *q) {
                    return Err(CommandError::Unaffordable);
                }
                let b = self.bodies.get_mut(&body).unwrap();
                b.place(&content, block, cell, rot).map_err(|_| CommandError::Blocked)?;
                for (r, q) in cost {
                    take_from_storage(&content, b, r, q);
                }
                b.refresh(&content);
            }
            Command::Deconstruct { body, cell } => {
                let b = self.bodies.get_mut(&body).ok_or(CommandError::NoBody)?;
                let slot = b.block_at(cell).ok_or(CommandError::NothingThere)?;
                if matches!(content.blocks[b.block(slot).unwrap().def as usize].kind, Block::Core) {
                    return Err(CommandError::Invalid);
                }
                let inst = b.remove_block(slot).unwrap();
                b.lost_blocks = false;
                b.refresh(&content);
                // Refund the build cost and any stock the block was holding.
                for (r, q) in &content.blocks[inst.def as usize].cost {
                    fill_storage(&content, b, *r, *q as f32);
                }
                for (r, q) in &inst.stock {
                    fill_storage(&content, b, *r, *q);
                }
                split_if_needed(self, body);
            }
            Command::SetRecipe { body, cell, recipe } => {
                let b = self.bodies.get_mut(&body).ok_or(CommandError::NoBody)?;
                let slot = b.block_at(cell).ok_or(CommandError::NothingThere)?;
                let inst = b.block_mut(slot).unwrap();
                match &content.blocks[inst.def as usize].kind {
                    Block::Machine { recipes, .. } if recipes.contains(&recipe) => {
                        if inst.recipe != Some(recipe) {
                            inst.recipe = Some(recipe);
                            inst.progress = 0.0;
                        }
                    }
                    _ => return Err(CommandError::Invalid),
                }
            }
            Command::CycleRecipe { body, cell } => {
                let b = self.bodies.get_mut(&body).ok_or(CommandError::NoBody)?;
                let slot = b.block_at(cell).ok_or(CommandError::NothingThere)?;
                let inst = b.block_mut(slot).unwrap();
                match &content.blocks[inst.def as usize].kind {
                    Block::Machine { recipes, .. } if !recipes.is_empty() => {
                        let i = recipes.iter().position(|r| Some(*r) == inst.recipe).map_or(0, |i| (i + 1) % recipes.len());
                        inst.recipe = Some(recipes[i]);
                        inst.progress = 0.0;
                    }
                    _ => return Err(CommandError::Invalid),
                }
            }
            Command::SetWeaponMode { body, cell, mode } => {
                let b = self.bodies.get_mut(&body).ok_or(CommandError::NoBody)?;
                let slot = b.block_at(cell).ok_or(CommandError::NothingThere)?;
                let inst = b.block_mut(slot).unwrap();
                match &content.blocks[inst.def as usize].kind {
                    Block::Weapon(_) => inst.mode = mode,
                    _ => return Err(CommandError::Invalid),
                }
            }
            Command::StartResearch { research } => {
                if !self.progress.start_research(&content, research) {
                    return Err(CommandError::Locked);
                }
            }
        }
        Ok(())
    }

    /// FNV-1a hash over the authoritative simulation state. Two worlds that
    /// were fed the same seed and commands must produce identical hashes.
    pub fn state_hash(&self) -> u64 {
        let mut h = Fnv::default();
        h.u64(self.tick);
        h.u64(self.rng.state());
        h.u64(self.next_id as u64);
        for (id, b) in &self.bodies {
            h.u64(*id as u64);
            h.f32(b.pos.x);
            h.f32(b.pos.y);
            h.f32(b.angle);
            h.f32(b.vel.x);
            h.f32(b.vel.y);
            h.f32(b.angvel);
            for (slot, blk) in b.blocks.iter().enumerate() {
                let Some(blk) = blk else { continue };
                h.u64(slot as u64);
                h.u64(blk.def as u64);
                h.u64(((blk.anchor.x as u32 as u64) << 32) | blk.anchor.y as u32 as u64);
                h.u64(blk.size.0 as u64 | (blk.size.1 as u64) << 8 | (blk.feed.tier as u64) << 16);
                h.f32(blk.hp);
                h.f32(blk.fire);
                for (r, q) in &blk.stock {
                    h.u64(*r as u64);
                    h.f32(*q);
                }
            }
        }
        for p in &self.projectiles {
            h.f32(p.pos.x);
            h.f32(p.pos.y);
        }
        h.f32(self.progress.points);
        h.0
    }

    pub fn save_ron(&self) -> Result<String, ron::Error> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
    }

    pub fn load_ron(text: &str, content: Arc<ContentDb>) -> Result<Self, ron::error::SpannedError> {
        let mut w: World = ron::from_str(text)?;
        w.content = content;
        let c = w.content.clone();
        for b in w.bodies.values_mut() {
            b.refresh(&c);
        }
        Ok(w)
    }
}

struct Fnv(u64);
impl Default for Fnv {
    fn default() -> Self {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
}
impl Fnv {
    fn u64(&mut self, v: u64) {
        for byte in v.to_le_bytes() {
            self.0 ^= byte as u64;
            self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
        }
    }
    fn f32(&mut self, v: f32) {
        self.u64(v.to_bits() as u64);
    }
}

/// Put `qty` of `r` into storage / tanks that accept it. Returns the amount
/// that did not fit.
pub fn fill_storage(content: &ContentDb, body: &mut Body, r: ResId, qty: f32) -> f32 {
    let mut left = qty;
    let slots = body.topo.live.clone();
    for s in slots {
        if left <= 0.0 {
            break;
        }
        let b = body.block_mut(s).unwrap();
        let (cap, filter) = match &content.blocks[b.def as usize].kind {
            Block::Storage { capacity, filter } if content.res_kind(r) == gf_content::ResourceKind::Item => (*capacity as f32, filter),
            Block::Tank { capacity, filter } if content.res_kind(r) == gf_content::ResourceKind::Fluid => (*capacity, filter),
            _ => continue,
        };
        if !filter.is_empty() && !filter.contains(&r) {
            continue;
        }
        let put = (cap - b.total_stock()).max(0.0).min(left);
        b.add(r, put);
        left -= put;
    }
    left
}

pub fn take_from_storage(content: &ContentDb, body: &mut Body, r: ResId, qty: f32) -> f32 {
    let mut left = qty;
    for s in body.topo.live.clone() {
        if left <= 0.0 {
            break;
        }
        let b = body.block_mut(s).unwrap();
        if matches!(content.blocks[b.def as usize].kind, Block::Storage { .. } | Block::Tank { .. }) {
            left -= b.take(r, left);
        }
    }
    qty - left
}

/// Split disconnected pieces of a body into separate bodies. The piece with
/// the core (or the largest piece) keeps the original identity.
pub fn split_if_needed(world: &mut World, id: BodyId) {
    let content = world.content.clone();
    let Some(body) = world.bodies.get_mut(&id) else { return };
    body.refresh(&content);
    if body.topo.live.is_empty() {
        world.bodies.remove(&id);
        return;
    }
    let comps = body.components();
    if comps.len() <= 1 {
        return;
    }
    let is_core = |s: u32| matches!(content.blocks[body.block(s).unwrap().def as usize].kind, Block::Core);
    let keep = comps
        .iter()
        .enumerate()
        .max_by_key(|(i, c)| (c.iter().any(|s| is_core(*s)), c.len(), usize::MAX - i))
        .map(|(i, _)| i)
        .unwrap();
    let mut new_bodies = Vec::new();
    for (i, comp) in comps.into_iter().enumerate() {
        if i == keep {
            continue;
        }
        let kind = if body.kind == BodyKind::Asteroid { BodyKind::Asteroid } else { BodyKind::Derelict };
        let mut nb = Body::new(0, format!("{} debris", body.name), kind, None, body.pos, body.angle);
        nb.com = body.com;
        for s in comp {
            let inst = body.remove_block(s).unwrap();
            nb.adopt(inst);
        }
        nb.refresh(&content);
        nb.vel = body.velocity_at(nb.pos);
        nb.angvel = body.angvel;
        new_bodies.push(nb);
    }
    body.lost_blocks = false;
    body.refresh(&content);
    for nb in new_bodies {
        world.add_body(nb);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn content() -> Arc<ContentDb> {
        Arc::new(ContentDb::load_dir(gf_content::default_content_dir()).unwrap())
    }

    #[test]
    fn starter_ship_spawns_with_stock() {
        let w = World::new_game(content(), 7);
        let p = w.player_body().unwrap();
        assert!(p.topo.has_core);
        assert!(p.block_count() > 20);
        let shells = w.content.res("shells").unwrap();
        assert!(p.stored(&w.content, shells) > 0.0);
        assert!(w.bodies.len() > 1, "home sector generated asteroids");
    }

    #[test]
    fn determinism_same_seed_same_hash() {
        let run = || {
            let mut w = World::new_game(content(), 42);
            let p = w.player.unwrap();
            for t in 0..400 {
                if t % 50 == 0 {
                    let control = Control {
                        thrust: Vec2::new(0.0, 1.0),
                        turn: if t % 100 == 0 { 1.0 } else { -0.5 },
                        fire: true,
                        aim: Some(Vec2::new(20.0, 30.0)),
                        auto_suppress: true,
                    };
                    w.queue(Command::Control { body: p, control });
                }
                w.step();
            }
            w.state_hash()
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn save_and_load_round_trip() {
        let c = content();
        let mut w = World::new_game(c.clone(), 3);
        for _ in 0..60 {
            w.step();
        }
        let text = w.save_ron().unwrap();
        let mut w2 = World::load_ron(&text, c).unwrap();
        assert_eq!(w.bodies.len(), w2.bodies.len());
        w.step();
        w2.step();
        assert_eq!(w.state_hash(), w2.state_hash());
    }

    #[test]
    fn build_spends_stock_and_deconstruct_refunds() {
        let c = content();
        let mut w = World::new_game(c.clone(), 1);
        let p = w.player.unwrap();
        let plates = c.res("plates").unwrap();
        let before = w.bodies[&p].stored(&c, plates);
        // Find a free cell adjacent to the structure.
        let body = &w.bodies[&p];
        let cell = body
            .grid
            .iter()
            .flat_map(|(cell, _)| Cell::NEIGHBORS.map(|d| cell.offset(d)))
            .find(|c| body.grid.is_empty_at(*c))
            .unwrap();
        let armor = c.block("armor_full").unwrap();
        w.queue(Command::Build { body: p, block: armor, cell, rot: 0 });
        w.step();
        let mid = w.bodies[&p].stored(&c, plates);
        assert!(mid < before, "build cost paid: {before} -> {mid}");
        assert!(w.bodies[&p].block_at(cell).is_some());
        w.queue(Command::Deconstruct { body: p, cell });
        w.step();
        assert!(w.bodies[&p].block_at(cell).is_none());
        assert!(w.bodies[&p].stored(&c, plates) > mid);
    }
}
