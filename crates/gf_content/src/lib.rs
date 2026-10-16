//! Galactiforge content: data-driven definitions for resources, recipes,
//! blocks, research, missions and ship blueprints.
//!
//! Content is authored as RON files (see `assets/content`). All files in the
//! content directory are merged, then resolved into a [`ContentDb`] with dense
//! numeric ids that the simulation uses. Validation fails loudly with a list of
//! human-readable errors so broken content never reaches the simulation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub mod legacy;

pub type ResId = u16;
pub type BlockId = u16;
pub type RecipeId = u16;
pub type ResearchId = u16;

// ---------------------------------------------------------------------------
// Raw (authored) schema
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    /// Discrete items moved by conveyors.
    Item,
    /// Fluids moved by pipe networks.
    Fluid,
}

/// Ammunition modifiers a resource applies when fired from a weapon.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AmmoDef {
    #[serde(default = "one")]
    pub damage_mult: f32,
    #[serde(default)]
    pub splash_radius: f32,
    #[serde(default)]
    pub pierce: u8,
    #[serde(default)]
    pub incendiary: f32,
    /// Frost ammo cools and extinguishes; enables cannon suppression mode.
    #[serde(default)]
    pub frost: f32,
    #[serde(default)]
    pub emp: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDef {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub kind: ResourceKind,
    /// 0..1, how readily stored quantities burn / explode.
    #[serde(default)]
    pub flammability: f32,
    #[serde(default)]
    pub ammo: Option<AmmoDef>,
    /// Research points granted per unit when consumed by a lab.
    #[serde(default)]
    pub research_value: f32,
    /// Tileset index used for icons (optional).
    #[serde(default)]
    pub icon: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDef {
    pub id: String,
    #[serde(default)]
    pub inputs: Vec<(String, f32)>,
    #[serde(default)]
    pub outputs: Vec<(String, f32)>,
    /// Seconds per craft at full power.
    pub time: f32,
    /// Power draw (kW) while crafting.
    #[serde(default)]
    pub power: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Hull,
    Core,
    Logistics,
    Factory,
    Power,
    Weapon,
    Thruster,
    Utility,
    Research,
    Mineral,
}

/// Sprite reference into the owner's 32x32 tileset (`factory_devices.png`).
/// Indices are 0-based, row-major over a 10-column sheet.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpriteDef {
    pub tile: u16,
    #[serde(default)]
    pub frames: u16,
    #[serde(default)]
    pub fps: f32,
    /// Optional base tile drawn beneath (e.g. turret base plate).
    #[serde(default)]
    pub base: Option<u16>,
    /// RGB tint applied (palette swap for placeholder variants).
    #[serde(default)]
    pub tint: Option<(f32, f32, f32)>,
}

/// One growth stage of a weapon / thruster / mining laser.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthStage {
    /// Footprint (width, depth) in cells, anchored at the mount cell.
    pub size: (u8, u8),
    /// Sustained supply (units/s of primary feed) needed to hold this tier.
    pub demand: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponKind {
    Cannon,
    Rocket,
    MiningLaser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponDef {
    pub kind: WeaponKind,
    /// Accepted ammo resources (items or fluids).
    #[serde(default)]
    pub ammo: Vec<String>,
    pub growth: Vec<GrowthStage>,
    pub damage: f32,
    /// Seconds between shots at tier 1.
    pub interval: f32,
    pub projectile_speed: f32,
    pub range: f32,
    #[serde(default)]
    pub heat_per_shot: f32,
    #[serde(default = "hundred")]
    pub heat_capacity: f32,
    #[serde(default = "ten")]
    pub cool_rate: f32,
    /// Power draw (kW) at tier 1 (scaled by tier).
    #[serde(default)]
    pub power: f32,
    /// Turret slew rate in radians/s.
    #[serde(default = "three")]
    pub turn_rate: f32,
    /// Seconds of tier-max demand buffered locally.
    #[serde(default = "two")]
    pub buffer_secs: f32,
    #[serde(default)]
    pub homing: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrusterDef {
    pub fuel: String,
    #[serde(default)]
    pub oxidizer: Option<String>,
    pub growth: Vec<GrowthStage>,
    /// Thrust (kN) at tier 1 at full throttle.
    pub thrust: f32,
    #[serde(default = "five")]
    pub heat_per_sec: f32,
    #[serde(default = "hundred")]
    pub heat_capacity: f32,
    #[serde(default = "ten")]
    pub cool_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockKind {
    Hull,
    Core,
    Storage {
        capacity: u32,
        #[serde(default)]
        filter: Vec<String>,
    },
    Tank {
        capacity: f32,
        #[serde(default)]
        filter: Vec<String>,
    },
    Conveyor {
        speed: f32,
    },
    Splitter {
        speed: f32,
    },
    Pipe,
    Cable,
    Machine {
        recipes: Vec<String>,
        #[serde(default = "ten_u32")]
        buffer: u32,
    },
    Generator {
        output: f32,
        #[serde(default)]
        fuel: Option<(String, f32)>,
    },
    Solar {
        output: f32,
    },
    Battery {
        capacity: f32,
        rate: f32,
    },
    Weapon(WeaponDef),
    Thruster(ThrusterDef),
    Drill {
        rate: f32,
        power: f32,
    },
    Salvager {
        rate: f32,
        power: f32,
    },
    Tractor {
        range: f32,
        force: f32,
        power: f32,
    },
    Lab {
        power: f32,
        /// Max research items consumed per second.
        rate: f32,
    },
    Radar {
        range: f32,
    },
    Mineral {
        yields: Vec<(String, f32)>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDef {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub category: Category,
    pub sprite: SpriteDef,
    #[serde(default = "one")]
    pub mass: f32,
    #[serde(default = "ten")]
    pub hp: f32,
    /// Flat damage reduction per hit.
    #[serde(default)]
    pub armor: f32,
    #[serde(default)]
    pub cost: Vec<(String, u32)>,
    #[serde(default = "yes")]
    pub buildable: bool,
    #[serde(default)]
    pub starts_unlocked: bool,
    #[serde(default)]
    pub flammability: f32,
    /// Research sample awarded when this block is salvaged intact.
    #[serde(default)]
    pub sample: Option<String>,
    pub kind: BlockKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchDef {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub cost: f32,
    #[serde(default)]
    pub requires: Vec<String>,
    /// Salvage samples that must have been recovered before research can start.
    #[serde(default)]
    pub samples: Vec<String>,
    pub unlocks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Objective {
    DestroyShips { count: u32 },
    SalvageBlocks { count: u32 },
    MineResource { resource: String, amount: f32 },
    Survive { seconds: f32 },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Reward {
    #[serde(default)]
    pub research_points: f32,
    #[serde(default)]
    pub samples: Vec<String>,
    #[serde(default)]
    pub unlocks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionDef {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub objective: Objective,
    #[serde(default)]
    pub reward: Reward,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlueprintBlock {
    pub block: String,
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub rot: u8,
    #[serde(default)]
    pub recipe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Blueprint {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub blocks: Vec<BlueprintBlock>,
    /// Starting stock placed into storage/tanks.
    #[serde(default)]
    pub stock: Vec<(String, f32)>,
}

/// One authored content file. Every section is optional; files are merged.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContentFile {
    #[serde(default)]
    pub resources: Vec<ResourceDef>,
    #[serde(default)]
    pub recipes: Vec<RecipeDef>,
    #[serde(default)]
    pub blocks: Vec<BlockDef>,
    #[serde(default)]
    pub research: Vec<ResearchDef>,
    #[serde(default)]
    pub missions: Vec<MissionDef>,
    #[serde(default)]
    pub blueprints: Vec<Blueprint>,
}

fn one() -> f32 {
    1.0
}
fn two() -> f32 {
    2.0
}
fn three() -> f32 {
    3.0
}
fn five() -> f32 {
    5.0
}
fn ten() -> f32 {
    10.0
}
fn hundred() -> f32 {
    100.0
}
fn ten_u32() -> u32 {
    10
}
fn yes() -> bool {
    true
}

// ---------------------------------------------------------------------------
// Resolved database
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: String,
    pub inputs: Vec<(ResId, f32)>,
    pub outputs: Vec<(ResId, f32)>,
    pub time: f32,
    pub power: f32,
}

#[derive(Debug, Clone)]
pub enum Block {
    Hull,
    Core,
    Storage {
        capacity: u32,
        filter: Vec<ResId>,
    },
    Tank {
        capacity: f32,
        filter: Vec<ResId>,
    },
    Conveyor {
        speed: f32,
    },
    Splitter {
        speed: f32,
    },
    Pipe,
    Cable,
    Machine {
        recipes: Vec<RecipeId>,
        buffer: u32,
    },
    Generator {
        output: f32,
        fuel: Option<(ResId, f32)>,
    },
    Solar {
        output: f32,
    },
    Battery {
        capacity: f32,
        rate: f32,
    },
    Weapon(Weapon),
    Thruster(Thruster),
    Drill {
        rate: f32,
        power: f32,
    },
    Salvager {
        rate: f32,
        power: f32,
    },
    Tractor {
        range: f32,
        force: f32,
        power: f32,
    },
    Lab {
        power: f32,
        rate: f32,
    },
    Radar {
        range: f32,
    },
    Mineral {
        yields: Vec<(ResId, f32)>,
    },
}

#[derive(Debug, Clone)]
pub struct Weapon {
    pub def: WeaponDef,
    pub ammo: Vec<ResId>,
}

#[derive(Debug, Clone)]
pub struct Thruster {
    pub def: ThrusterDef,
    pub fuel: ResId,
    pub oxidizer: Option<ResId>,
}

#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub def: BlockDef,
    pub kind: Block,
    pub cost: Vec<(ResId, u32)>,
    pub sample: Option<ResId>,
}

impl BlockInfo {
    pub fn growth(&self) -> Option<&[GrowthStage]> {
        match &self.kind {
            Block::Weapon(w) => Some(&w.def.growth),
            Block::Thruster(t) => Some(&t.def.growth),
            _ => None,
        }
    }
    /// The full footprint reserved by this block (largest growth stage).
    pub fn reserved_size(&self) -> (u8, u8) {
        match self.growth() {
            Some(g) => g
                .iter()
                .fold((1, 1), |a, s| (a.0.max(s.size.0), a.1.max(s.size.1))),
            None => (1, 1),
        }
    }
    pub fn conducts_items(&self) -> bool {
        matches!(
            self.kind,
            Block::Conveyor { .. } | Block::Splitter { .. } | Block::Storage { .. }
        )
    }
    /// Takes part in fluid networks.
    pub fn conducts_fluid(&self) -> bool {
        matches!(self.kind, Block::Pipe | Block::Tank { .. })
    }
    /// Takes part in power networks.
    pub fn conducts_power(&self) -> bool {
        matches!(
            self.kind,
            Block::Cable
                | Block::Core
                | Block::Generator { .. }
                | Block::Solar { .. }
                | Block::Battery { .. }
        )
    }
}

#[derive(Debug, Clone)]
pub struct ContentDb {
    pub resources: Vec<ResourceDef>,
    pub recipes: Vec<Recipe>,
    pub blocks: Vec<BlockInfo>,
    pub research: Vec<ResearchDef>,
    pub research_requires: Vec<Vec<ResearchId>>,
    pub research_samples: Vec<Vec<ResId>>,
    pub research_unlocks: Vec<Vec<BlockId>>,
    pub missions: Vec<MissionDef>,
    pub blueprints: Vec<Blueprint>,
    res_ids: HashMap<String, ResId>,
    recipe_ids: HashMap<String, RecipeId>,
    block_ids: HashMap<String, BlockId>,
    research_ids: HashMap<String, ResearchId>,
}

#[derive(Debug)]
pub struct ContentError(pub Vec<String>);

impl std::fmt::Display for ContentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "content validation failed:")?;
        for e in &self.0 {
            writeln!(f, "  - {e}")?;
        }
        Ok(())
    }
}
impl std::error::Error for ContentError {}

impl ContentDb {
    pub fn res(&self, id: &str) -> Option<ResId> {
        self.res_ids.get(id).copied()
    }
    pub fn recipe(&self, id: &str) -> Option<RecipeId> {
        self.recipe_ids.get(id).copied()
    }
    pub fn block(&self, id: &str) -> Option<BlockId> {
        self.block_ids.get(id).copied()
    }
    pub fn research_id(&self, id: &str) -> Option<ResearchId> {
        self.research_ids.get(id).copied()
    }
    pub fn blueprint(&self, id: &str) -> Option<&Blueprint> {
        self.blueprints.iter().find(|b| b.id == id)
    }
    pub fn res_kind(&self, r: ResId) -> ResourceKind {
        self.resources[r as usize].kind
    }

    /// Load and merge every `.ron` file under `dir` (recursively, sorted for
    /// determinism), then resolve and validate.
    pub fn load_dir(dir: impl AsRef<Path>) -> Result<Self, ContentError> {
        let mut files = Vec::new();
        collect_ron(dir.as_ref(), &mut files).map_err(|e| ContentError(vec![e]))?;
        files.sort();
        let mut merged = ContentFile::default();
        let mut errors = Vec::new();
        for f in files {
            match std::fs::read_to_string(&f) {
                Ok(text) => match ron::from_str::<ContentFile>(&text) {
                    Ok(cf) => merge(&mut merged, cf),
                    Err(e) => errors.push(format!("{}: {e}", f.display())),
                },
                Err(e) => errors.push(format!("{}: {e}", f.display())),
            }
        }
        if !errors.is_empty() {
            return Err(ContentError(errors));
        }
        Self::resolve(merged)
    }

    /// Parse a single RON string (used by tests and tools).
    pub fn from_ron_str(text: &str) -> Result<Self, ContentError> {
        let cf: ContentFile = ron::from_str(text).map_err(|e| ContentError(vec![e.to_string()]))?;
        Self::resolve(cf)
    }

    pub fn resolve(cf: ContentFile) -> Result<Self, ContentError> {
        let mut errors = Vec::new();
        let mut res_ids = HashMap::new();
        for (i, r) in cf.resources.iter().enumerate() {
            if res_ids.insert(r.id.clone(), i as ResId).is_some() {
                errors.push(format!("duplicate resource `{}`", r.id));
            }
        }
        let lookup_res = |id: &str, ctx: &str, errors: &mut Vec<String>| -> ResId {
            match res_ids.get(id) {
                Some(r) => *r,
                None => {
                    errors.push(format!("{ctx}: unknown resource `{id}`"));
                    0
                }
            }
        };

        let mut recipe_ids = HashMap::new();
        let mut recipes = Vec::new();
        for (i, r) in cf.recipes.iter().enumerate() {
            if recipe_ids.insert(r.id.clone(), i as RecipeId).is_some() {
                errors.push(format!("duplicate recipe `{}`", r.id));
            }
            let ctx = format!("recipe `{}`", r.id);
            if r.time <= 0.0 {
                errors.push(format!("{ctx}: time must be > 0"));
            }
            if r.outputs.is_empty() {
                errors.push(format!("{ctx}: has no outputs"));
            }
            recipes.push(Recipe {
                id: r.id.clone(),
                inputs: r
                    .inputs
                    .iter()
                    .map(|(n, q)| (lookup_res(n, &ctx, &mut errors), *q))
                    .collect(),
                outputs: r
                    .outputs
                    .iter()
                    .map(|(n, q)| (lookup_res(n, &ctx, &mut errors), *q))
                    .collect(),
                time: r.time,
                power: r.power,
            });
        }

        let mut block_ids = HashMap::new();
        for (i, b) in cf.blocks.iter().enumerate() {
            if block_ids.insert(b.id.clone(), i as BlockId).is_some() {
                errors.push(format!("duplicate block `{}`", b.id));
            }
        }
        let mut blocks = Vec::new();
        for b in &cf.blocks {
            let ctx = format!("block `{}`", b.id);
            let rl = |v: &Vec<String>, errors: &mut Vec<String>| -> Vec<ResId> {
                v.iter().map(|n| lookup_res(n, &ctx, errors)).collect()
            };
            let kind = match &b.kind {
                BlockKind::Hull => Block::Hull,
                BlockKind::Core => Block::Core,
                BlockKind::Storage { capacity, filter } => Block::Storage {
                    capacity: *capacity,
                    filter: rl(filter, &mut errors),
                },
                BlockKind::Tank { capacity, filter } => Block::Tank {
                    capacity: *capacity,
                    filter: rl(filter, &mut errors),
                },
                BlockKind::Conveyor { speed } => Block::Conveyor { speed: *speed },
                BlockKind::Splitter { speed } => Block::Splitter { speed: *speed },
                BlockKind::Pipe => Block::Pipe,
                BlockKind::Cable => Block::Cable,
                BlockKind::Machine {
                    recipes: rs,
                    buffer,
                } => {
                    let mut ids = Vec::new();
                    for r in rs {
                        match recipe_ids.get(r) {
                            Some(id) => ids.push(*id),
                            None => errors.push(format!("{ctx}: unknown recipe `{r}`")),
                        }
                    }
                    if ids.is_empty() {
                        errors.push(format!("{ctx}: machine has no recipes"));
                    }
                    Block::Machine {
                        recipes: ids,
                        buffer: *buffer,
                    }
                }
                BlockKind::Generator { output, fuel } => Block::Generator {
                    output: *output,
                    fuel: fuel
                        .as_ref()
                        .map(|(n, r)| (lookup_res(n, &ctx, &mut errors), *r)),
                },
                BlockKind::Solar { output } => Block::Solar { output: *output },
                BlockKind::Battery { capacity, rate } => Block::Battery {
                    capacity: *capacity,
                    rate: *rate,
                },
                BlockKind::Weapon(w) => {
                    check_growth(&w.growth, &ctx, &mut errors);
                    if w.kind != WeaponKind::MiningLaser && w.ammo.is_empty() {
                        errors.push(format!("{ctx}: weapon has no ammo"));
                    }
                    for a in &w.ammo {
                        if let Some(id) = res_ids.get(a)
                            && cf.resources[*id as usize].ammo.is_none()
                        {
                            errors.push(format!("{ctx}: ammo `{a}` has no ammo definition"));
                        }
                    }
                    Block::Weapon(Weapon {
                        def: w.clone(),
                        ammo: rl(&w.ammo, &mut errors),
                    })
                }
                BlockKind::Thruster(t) => {
                    check_growth(&t.growth, &ctx, &mut errors);
                    let fuel = lookup_res(&t.fuel, &ctx, &mut errors);
                    let oxidizer = t
                        .oxidizer
                        .as_ref()
                        .map(|o| lookup_res(o, &ctx, &mut errors));
                    Block::Thruster(Thruster {
                        def: t.clone(),
                        fuel,
                        oxidizer,
                    })
                }
                BlockKind::Drill { rate, power } => Block::Drill {
                    rate: *rate,
                    power: *power,
                },
                BlockKind::Salvager { rate, power } => Block::Salvager {
                    rate: *rate,
                    power: *power,
                },
                BlockKind::Tractor {
                    range,
                    force,
                    power,
                } => Block::Tractor {
                    range: *range,
                    force: *force,
                    power: *power,
                },
                BlockKind::Lab { power, rate } => Block::Lab {
                    power: *power,
                    rate: *rate,
                },
                BlockKind::Radar { range } => Block::Radar { range: *range },
                BlockKind::Mineral { yields } => Block::Mineral {
                    yields: yields
                        .iter()
                        .map(|(n, q)| (lookup_res(n, &ctx, &mut errors), *q))
                        .collect(),
                },
            };
            if b.hp <= 0.0 {
                errors.push(format!("{ctx}: hp must be > 0"));
            }
            if b.mass <= 0.0 {
                errors.push(format!("{ctx}: mass must be > 0"));
            }
            let cost = b
                .cost
                .iter()
                .map(|(n, q)| (lookup_res(n, &ctx, &mut errors), *q))
                .collect();
            let sample = b.sample.as_ref().map(|s| lookup_res(s, &ctx, &mut errors));
            blocks.push(BlockInfo {
                def: b.clone(),
                kind,
                cost,
                sample,
            });
        }

        let mut research_ids = HashMap::new();
        for (i, r) in cf.research.iter().enumerate() {
            if research_ids.insert(r.id.clone(), i as ResearchId).is_some() {
                errors.push(format!("duplicate research `{}`", r.id));
            }
        }
        let mut research_requires = Vec::new();
        let mut research_samples = Vec::new();
        let mut research_unlocks = Vec::new();
        for r in &cf.research {
            let ctx = format!("research `{}`", r.id);
            research_requires.push(
                r.requires
                    .iter()
                    .filter_map(|q| {
                        let id = research_ids.get(q).copied();
                        if id.is_none() {
                            errors.push(format!("{ctx}: unknown prerequisite `{q}`"));
                        }
                        id
                    })
                    .collect(),
            );
            research_samples.push(
                r.samples
                    .iter()
                    .map(|s| lookup_res(s, &ctx, &mut errors))
                    .collect(),
            );
            research_unlocks.push(
                r.unlocks
                    .iter()
                    .filter_map(|b| {
                        let id = block_ids.get(b).copied();
                        if id.is_none() {
                            errors.push(format!("{ctx}: unknown block `{b}`"));
                        }
                        id
                    })
                    .collect(),
            );
        }

        for m in &cf.missions {
            let ctx = format!("mission `{}`", m.id);
            if let Objective::MineResource { resource, .. } = &m.objective {
                lookup_res(resource, &ctx, &mut errors);
            }
            for s in &m.reward.samples {
                lookup_res(s, &ctx, &mut errors);
            }
            for b in &m.reward.unlocks {
                if !block_ids.contains_key(b) {
                    errors.push(format!("{ctx}: unknown block `{b}`"));
                }
            }
        }

        for bp in &cf.blueprints {
            let ctx = format!("blueprint `{}`", bp.id);
            for bb in &bp.blocks {
                match block_ids.get(&bb.block) {
                    None => errors.push(format!("{ctx}: unknown block `{}`", bb.block)),
                    Some(_) => {
                        if bb.rot > 3 {
                            errors.push(format!("{ctx}: rot must be 0..=3"));
                        }
                    }
                }
                if let Some(r) = &bb.recipe
                    && !recipe_ids.contains_key(r)
                {
                    errors.push(format!("{ctx}: unknown recipe `{r}`"));
                }
            }
            for (s, _) in &bp.stock {
                lookup_res(s, &ctx, &mut errors);
            }
        }

        if !errors.is_empty() {
            return Err(ContentError(errors));
        }
        Ok(ContentDb {
            resources: cf.resources,
            recipes,
            blocks,
            research: cf.research,
            research_requires,
            research_samples,
            research_unlocks,
            missions: cf.missions,
            blueprints: cf.blueprints,
            res_ids,
            recipe_ids,
            block_ids,
            research_ids,
        })
    }
}

fn check_growth(g: &[GrowthStage], ctx: &str, errors: &mut Vec<String>) {
    if g.is_empty() {
        errors.push(format!("{ctx}: growth must have at least one stage"));
    }
    for w in g.windows(2) {
        if w[1].size.0 < w[0].size.0 || w[1].size.1 < w[0].size.1 {
            errors.push(format!("{ctx}: growth stages must not shrink"));
        }
        if w[1].demand < w[0].demand {
            errors.push(format!("{ctx}: growth demand must be non-decreasing"));
        }
    }
    if let Some(s) = g.first()
        && s.size != (1, 1)
    {
        errors.push(format!("{ctx}: first growth stage must be 1x1"));
    }
}

fn merge(into: &mut ContentFile, cf: ContentFile) {
    into.resources.extend(cf.resources);
    into.recipes.extend(cf.recipes);
    into.blocks.extend(cf.blocks);
    into.research.extend(cf.research);
    into.missions.extend(cf.missions);
    into.blueprints.extend(cf.blueprints);
}

fn collect_ron(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    let rd = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in rd {
        let p = entry.map_err(|e| e.to_string())?.path();
        if p.is_dir() {
            collect_ron(&p, out)?;
        } else if p.extension().is_some_and(|e| e == "ron") {
            out.push(p);
        }
    }
    Ok(())
}

/// Default content directory relative to the workspace root.
pub fn default_content_dir() -> std::path::PathBuf {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("../../assets/content")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_content_is_valid() {
        let db = ContentDb::load_dir(default_content_dir()).unwrap_or_else(|e| panic!("{e}"));
        assert!(db.block("ship_core").is_some());
        assert!(db.block("cannon").is_some());
        assert!(db.block("rocket_launcher").is_some());
        assert!(db.blueprint("micro_bandit").is_some());
    }

    #[test]
    fn unknown_references_are_reported() {
        let err = ContentDb::from_ron_str(
            r#"(recipes: [(id: "x", inputs: [("nope", 1.0)], outputs: [("nada", 1.0)], time: 1.0)])"#,
        )
        .unwrap_err();
        assert_eq!(err.0.len(), 2);
    }

    #[test]
    fn growth_must_start_at_1x1() {
        let err = ContentDb::from_ron_str(
            r#"(resources: [(id: "fuel", kind: Fluid)],
                blocks: [(id: "t", category: Thruster, sprite: (tile: 0),
                  kind: Thruster((fuel: "fuel", growth: [(size: (2,2), demand: 1.0)], thrust: 1.0)))])"#,
        )
        .unwrap_err();
        assert!(err.0.iter().any(|e| e.contains("1x1")));
    }
}
