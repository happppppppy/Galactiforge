//! Bodies (ships, asteroids, derelicts) built from a chunked block grid.

use crate::feed::Feed;
use crate::grid::Grid;
use crate::math::{Cell, Vec2, footprint};
use gf_content::{Block, BlockId, ContentDb, RecipeId, ResId};
use serde::{Deserialize, Serialize};

pub type BodyId = u32;
pub type Faction = u8;
pub const PLAYER_FACTION: Faction = 0;
pub const PIRATE_FACTION: Faction = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyKind {
    Ship,
    Asteroid,
    Derelict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WeaponMode {
    #[default]
    Attack,
    /// Fire frost ammo at fires on our own ship.
    Suppress,
}

/// One placed block instance. A single struct is used for every block kind;
/// unused fields stay at their defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInst {
    pub def: BlockId,
    pub anchor: Cell,
    pub rot: u8,
    /// Current footprint (grows with tier for weapons / thrusters).
    pub size: (u8, u8),
    pub hp: f32,
    pub heat: f32,
    /// Fire intensity 0..1.
    pub fire: f32,
    /// Remaining seconds of frost slow-down.
    pub chill: f32,
    /// Remaining seconds disabled by EMP.
    pub disabled: f32,
    /// Local resource buffer (items and fluids).
    pub stock: Vec<(ResId, f32)>,
    pub recipe: Option<RecipeId>,
    pub progress: f32,
    pub feed: Feed,
    /// Recent share of each ammo type delivered (for ammo modifiers).
    pub ammo_mix: Vec<(ResId, f32)>,
    pub power_sat: f32,
    pub power_req: f32,
    /// Stored energy (batteries).
    pub energy: f32,
    pub cooldown: f32,
    /// Turret / laser aim angle in the body's local frame (radians).
    pub aim: f32,
    pub throttle: f32,
    pub mode: WeaponMode,
    /// Round-robin cursor for outputs.
    pub rr: u32,
    pub move_acc: f32,
    /// Generic per-block work timer (drills, salvagers, labs).
    pub work: f32,
}

impl BlockInst {
    pub fn new(content: &ContentDb, def: BlockId, anchor: Cell, rot: u8) -> Self {
        let info = &content.blocks[def as usize];
        let recipe = match &info.kind {
            Block::Machine { recipes, .. } => recipes.first().copied(),
            _ => None,
        };
        Self {
            def,
            anchor,
            rot: rot & 3,
            size: (1, 1),
            hp: info.def.hp,
            heat: 0.0,
            fire: 0.0,
            chill: 0.0,
            disabled: 0.0,
            stock: Vec::new(),
            recipe,
            progress: 0.0,
            feed: Feed::default(),
            ammo_mix: Vec::new(),
            power_sat: 0.0,
            power_req: 0.0,
            energy: 0.0,
            cooldown: 0.0,
            aim: crate::math::PI * 0.5,
            throttle: 0.0,
            mode: WeaponMode::Attack,
            rr: 0,
            move_acc: 0.0,
            work: 0.0,
        }
    }
    pub fn cells(&self) -> impl Iterator<Item = Cell> {
        footprint(self.anchor, self.rot, self.size)
    }
    pub fn cell_count(&self) -> u32 {
        self.size.0 as u32 * self.size.1 as u32
    }
    /// Centre of the current footprint in local coordinates.
    pub fn center(&self) -> Vec2 {
        let mut s = Vec2::ZERO;
        let mut n = 0.0;
        for c in self.cells() {
            s += c.center();
            n += 1.0;
        }
        s * (1.0 / n)
    }
    pub fn max_hp(&self, content: &ContentDb) -> f32 {
        content.blocks[self.def as usize].def.hp * (1.0 + 0.6 * (self.cell_count() as f32 - 1.0))
    }
    pub fn amount(&self, r: ResId) -> f32 {
        self.stock
            .iter()
            .find(|(id, _)| *id == r)
            .map_or(0.0, |(_, q)| *q)
    }
    pub fn total_stock(&self) -> f32 {
        self.stock.iter().map(|(_, q)| *q).sum()
    }
    pub fn add(&mut self, r: ResId, q: f32) {
        if q == 0.0 {
            return;
        }
        match self.stock.iter_mut().find(|(id, _)| *id == r) {
            Some(e) => e.1 += q,
            None => {
                self.stock.push((r, q));
                self.stock.sort_by_key(|e| e.0);
            }
        }
    }
    /// Remove up to `q`; returns the amount removed.
    pub fn take(&mut self, r: ResId, q: f32) -> f32 {
        let mut got = 0.0;
        if let Some(i) = self.stock.iter().position(|(id, _)| *id == r) {
            let e = &mut self.stock[i];
            got = e.1.min(q);
            e.1 -= got;
            if e.1 <= 1e-6 {
                self.stock.remove(i);
            }
        }
        got
    }
    pub fn flammability(&self, content: &ContentDb) -> f32 {
        let base = content.blocks[self.def as usize].def.flammability;
        let stored: f32 = self
            .stock
            .iter()
            .map(|(r, q)| content.resources[*r as usize].flammability * q)
            .sum();
        (base + stored / 200.0).min(1.5)
    }
}

/// A connected logistics network (fluid pipes/tanks or power cables).
#[derive(Debug, Clone, Default)]
pub struct Net {
    pub conductors: Vec<u32>,
    pub attached: Vec<u32>,
}

/// Derived per-body data rebuilt whenever the block layout changes.
#[derive(Debug, Clone, Default)]
pub struct Topology {
    pub live: Vec<u32>,
    pub neighbors: Vec<Vec<u32>>,
    pub fluid_nets: Vec<Net>,
    pub power_nets: Vec<Net>,
    /// Power network per slot (conductor or attached consumer).
    pub power_net_of: Vec<Option<u32>>,
    pub has_core: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Control {
    /// Desired translation in the local frame: x = strafe right, y = forward.
    pub thrust: Vec2,
    /// Desired rotation, +1 = counter-clockwise.
    pub turn: f32,
    pub fire: bool,
    /// World-space aim point for turrets.
    pub aim: Option<Vec2>,
    /// Automatically switch frost-fed cannons to suppression when on fire.
    pub auto_suppress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    pub id: BodyId,
    pub name: String,
    pub kind: BodyKind,
    pub faction: Option<Faction>,
    /// World position of the centre of mass.
    pub pos: Vec2,
    pub angle: f32,
    pub vel: Vec2,
    pub angvel: f32,
    pub mass: f32,
    pub inertia: f32,
    /// Centre of mass in local grid coordinates.
    pub com: Vec2,
    /// Bounding radius around the centre of mass.
    pub radius: f32,
    pub grid: Grid,
    pub blocks: Vec<Option<BlockInst>>,
    free: Vec<u32>,
    pub control: Control,
    pub ai: Option<crate::ai::AiState>,
    pub last_hit_by: Option<Faction>,
    #[serde(skip)]
    pub force: Vec2,
    #[serde(skip)]
    pub torque: f32,
    #[serde(skip, default = "yes")]
    pub dirty: bool,
    #[serde(skip)]
    pub lost_blocks: bool,
    #[serde(skip)]
    pub topo: Topology,
    /// Bumped whenever the block layout changes (clients use it to resync).
    #[serde(skip)]
    pub revision: u64,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlaceError {
    Occupied,
    NotAdjacent,
}

impl Body {
    pub fn new(
        id: BodyId,
        name: impl Into<String>,
        kind: BodyKind,
        faction: Option<Faction>,
        pos: Vec2,
        angle: f32,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            faction,
            pos,
            angle,
            vel: Vec2::ZERO,
            angvel: 0.0,
            mass: 1.0,
            inertia: 1.0,
            com: Vec2::ZERO,
            radius: 1.0,
            grid: Grid::default(),
            blocks: Vec::new(),
            free: Vec::new(),
            control: Control::default(),
            ai: None,
            last_hit_by: None,
            force: Vec2::ZERO,
            torque: 0.0,
            dirty: true,
            lost_blocks: false,
            topo: Topology::default(),
            revision: 0,
        }
    }

    pub fn block(&self, slot: u32) -> Option<&BlockInst> {
        self.blocks.get(slot as usize).and_then(|b| b.as_ref())
    }
    pub fn block_mut(&mut self, slot: u32) -> Option<&mut BlockInst> {
        self.blocks.get_mut(slot as usize).and_then(|b| b.as_mut())
    }
    pub fn block_at(&self, c: Cell) -> Option<u32> {
        self.grid.get(c)
    }
    pub fn block_count(&self) -> usize {
        self.blocks.iter().filter(|b| b.is_some()).count()
    }

    pub fn to_world(&self, local: Vec2) -> Vec2 {
        self.pos + (local - self.com).rotate(self.angle)
    }
    pub fn to_local(&self, world: Vec2) -> Vec2 {
        (world - self.pos).rotate(-self.angle) + self.com
    }
    pub fn dir_to_world(&self, local_dir: Vec2) -> Vec2 {
        local_dir.rotate(self.angle)
    }
    pub fn velocity_at(&self, world: Vec2) -> Vec2 {
        self.vel + (world - self.pos).perp() * self.angvel
    }
    pub fn apply_force(&mut self, f: Vec2, at_world: Vec2) {
        self.force += f;
        self.torque += (at_world - self.pos).cross(f);
    }
    pub fn local_cell_at(&self, world: Vec2) -> Cell {
        let l = self.to_local(world);
        Cell::new(
            libm::floorf(l.x + 0.5) as i32,
            libm::floorf(l.y + 0.5) as i32,
        )
    }

    /// Insert a block without connectivity checks (used for blueprints/worldgen).
    pub fn insert(
        &mut self,
        content: &ContentDb,
        def: BlockId,
        anchor: Cell,
        rot: u8,
    ) -> Result<u32, PlaceError> {
        if !self.grid.is_empty_at(anchor) {
            return Err(PlaceError::Occupied);
        }
        let inst = BlockInst::new(content, def, anchor, rot);
        let slot = match self.free.pop() {
            Some(s) => {
                self.blocks[s as usize] = Some(inst);
                s
            }
            None => {
                self.blocks.push(Some(inst));
                self.blocks.len() as u32 - 1
            }
        };
        self.grid.set(anchor, Some(slot));
        self.dirty = true;
        Ok(slot)
    }

    /// Move an existing block instance (with its current footprint) into this
    /// body. Used when a body splits apart. Caller guarantees cells are free.
    pub fn adopt(&mut self, inst: BlockInst) -> u32 {
        let cells: Vec<Cell> = inst.cells().collect();
        let slot = match self.free.pop() {
            Some(s) => {
                self.blocks[s as usize] = Some(inst);
                s
            }
            None => {
                self.blocks.push(Some(inst));
                self.blocks.len() as u32 - 1
            }
        };
        for c in cells {
            self.grid.set(c, Some(slot));
        }
        self.dirty = true;
        slot
    }

    /// Player placement: the cell must be free and touch the existing structure.
    pub fn place(
        &mut self,
        content: &ContentDb,
        def: BlockId,
        anchor: Cell,
        rot: u8,
    ) -> Result<u32, PlaceError> {
        if !self.grid.is_empty_at(anchor) {
            return Err(PlaceError::Occupied);
        }
        if !self.grid.is_empty()
            && !Cell::NEIGHBORS
                .iter()
                .any(|d| !self.grid.is_empty_at(anchor.offset(*d)))
        {
            return Err(PlaceError::NotAdjacent);
        }
        self.insert(content, def, anchor, rot)
    }

    pub fn remove_block(&mut self, slot: u32) -> Option<BlockInst> {
        let inst = self.blocks.get_mut(slot as usize)?.take()?;
        for c in inst.cells() {
            if self.grid.get(c) == Some(slot) {
                self.grid.set(c, None);
            }
        }
        self.free.push(slot);
        self.free.sort_unstable_by(|a, b| b.cmp(a));
        self.dirty = true;
        self.lost_blocks = true;
        Some(inst)
    }

    /// Change the footprint of a growable block. Caller guarantees cells are free.
    pub fn resize_block(&mut self, slot: u32, size: (u8, u8)) {
        let Some(b) = self.block(slot) else { return };
        let old: Vec<Cell> = b.cells().collect();
        for c in old {
            if self.grid.get(c) == Some(slot) {
                self.grid.set(c, None);
            }
        }
        let b = self.block_mut(slot).unwrap();
        b.size = size;
        let new: Vec<Cell> = b.cells().collect();
        for c in new {
            self.grid.set(c, Some(slot));
        }
        self.dirty = true;
    }

    /// Largest growth stage whose footprint is free (or already ours).
    pub fn growth_cap(&self, content: &ContentDb, slot: u32) -> u8 {
        let Some(b) = self.block(slot) else { return 1 };
        let Some(stages) = content.blocks[b.def as usize].growth() else {
            return 1;
        };
        let mut cap = 1;
        for (i, s) in stages.iter().enumerate() {
            let free = footprint(b.anchor, b.rot, s.size).all(|c| match self.grid.get(c) {
                None => true,
                Some(o) => o == slot,
            });
            if free {
                cap = i as u8 + 1;
            } else {
                break;
            }
        }
        cap
    }

    /// Recompute mass properties and topology if the layout changed.
    pub fn refresh(&mut self, content: &ContentDb) {
        if !self.dirty {
            return;
        }
        self.dirty = false;
        self.revision = self.revision.wrapping_add(1);
        self.recompute_mass(content);
        self.rebuild_topology(content);
        let live = self.topo.live.clone();
        for slot in live {
            let cap = self.growth_cap(content, slot);
            if let Some(b) = self.block_mut(slot) {
                b.feed.cap = cap;
            }
        }
    }

    fn recompute_mass(&mut self, content: &ContentDb) {
        let mut m = 0.0;
        let mut mc = Vec2::ZERO;
        for b in self.blocks.iter().flatten() {
            let bm = content.blocks[b.def as usize].def.mass;
            let n = b.cell_count() as f32;
            let per = bm * (1.0 + 0.5 * (n - 1.0)) / n;
            for c in b.cells() {
                m += per;
                mc += c.center() * per;
            }
        }
        if m <= 0.0 {
            return;
        }
        let new_com = mc * (1.0 / m);
        let mut inertia = 0.0;
        let mut r2max: f32 = 0.0;
        for b in self.blocks.iter().flatten() {
            let bm = content.blocks[b.def as usize].def.mass;
            let n = b.cell_count() as f32;
            let per = bm * (1.0 + 0.5 * (n - 1.0)) / n;
            for c in b.cells() {
                let r2 = (c.center() - new_com).length_sq();
                inertia += per * (r2 + 1.0 / 6.0);
                r2max = r2max.max(r2);
            }
        }
        // Keep cells fixed in world space while the centre of mass moves.
        self.pos += (new_com - self.com).rotate(self.angle);
        self.com = new_com;
        self.mass = m;
        self.inertia = inertia.max(0.1);
        self.radius = libm::sqrtf(r2max) + 1.0;
    }

    fn rebuild_topology(&mut self, content: &ContentDb) {
        let n = self.blocks.len();
        let mut topo = Topology {
            live: (0..n as u32)
                .filter(|s| self.blocks[*s as usize].is_some())
                .collect(),
            neighbors: vec![Vec::new(); n],
            power_net_of: vec![None; n],
            ..Default::default()
        };
        for &s in &topo.live {
            let b = self.blocks[s as usize].as_ref().unwrap();
            let mut nb = Vec::new();
            for c in b.cells() {
                for d in Cell::NEIGHBORS {
                    if let Some(o) = self.grid.get(c.offset(d))
                        && o != s
                    {
                        nb.push(o);
                    }
                }
            }
            nb.sort_unstable();
            nb.dedup();
            topo.neighbors[s as usize] = nb;
            if matches!(content.blocks[b.def as usize].kind, Block::Core) {
                topo.has_core = true;
            }
        }
        let conducts = |slot: u32, fluid: bool| -> bool {
            let info = &content.blocks[self.blocks[slot as usize].as_ref().unwrap().def as usize];
            if fluid {
                info.conducts_fluid()
            } else {
                info.conducts_power()
            }
        };
        for fluid in [true, false] {
            let mut visited = vec![false; n];
            let mut nets = Vec::new();
            for &s in &topo.live {
                if visited[s as usize] || !conducts(s, fluid) {
                    continue;
                }
                let mut net = Net::default();
                let mut stack = vec![s];
                visited[s as usize] = true;
                while let Some(cur) = stack.pop() {
                    net.conductors.push(cur);
                    for &o in &topo.neighbors[cur as usize] {
                        if conducts(o, fluid) {
                            if !visited[o as usize] {
                                visited[o as usize] = true;
                                stack.push(o);
                            }
                        } else {
                            net.attached.push(o);
                        }
                    }
                }
                net.conductors.sort_unstable();
                net.attached.sort_unstable();
                net.attached.dedup();
                nets.push(net);
            }
            if fluid {
                topo.fluid_nets = nets;
            } else {
                for (i, net) in nets.iter().enumerate() {
                    for &c in net.conductors.iter().chain(net.attached.iter()) {
                        if topo.power_net_of[c as usize].is_none() {
                            topo.power_net_of[c as usize] = Some(i as u32);
                        }
                    }
                }
                topo.power_nets = nets;
            }
        }
        self.topo = topo;
    }

    /// Connected components of the block graph (requires fresh topology).
    pub fn components(&self) -> Vec<Vec<u32>> {
        let n = self.blocks.len();
        let mut seen = vec![false; n];
        let mut comps = Vec::new();
        for &s in &self.topo.live {
            if seen[s as usize] {
                continue;
            }
            let mut comp = Vec::new();
            let mut stack = vec![s];
            seen[s as usize] = true;
            while let Some(c) = stack.pop() {
                comp.push(c);
                for &o in &self.topo.neighbors[c as usize] {
                    if !seen[o as usize] {
                        seen[o as usize] = true;
                        stack.push(o);
                    }
                }
            }
            comp.sort_unstable();
            comps.push(comp);
        }
        comps
    }

    /// Slots of blocks matching a predicate on their content kind.
    pub fn slots_where(&self, content: &ContentDb, f: impl Fn(&Block) -> bool) -> Vec<u32> {
        self.topo
            .live
            .iter()
            .copied()
            .filter(|s| {
                f(&content.blocks[self.blocks[*s as usize].as_ref().unwrap().def as usize].kind)
            })
            .collect()
    }

    pub fn is_hostile_to(&self, other: &Body) -> bool {
        match (self.faction, other.faction) {
            (Some(a), Some(b)) => a != b,
            _ => false,
        }
    }

    /// Total stored amount of a resource across storage blocks and tanks.
    pub fn stored(&self, content: &ContentDb, r: ResId) -> f32 {
        self.blocks
            .iter()
            .flatten()
            .filter(|b| {
                matches!(
                    content.blocks[b.def as usize].kind,
                    Block::Storage { .. } | Block::Tank { .. }
                )
            })
            .map(|b| b.amount(r))
            .sum()
    }
}
