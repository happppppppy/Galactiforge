//! Progression: research (fed by labs and mission rewards, gated by salvaged
//! samples) and missions.

use crate::body::{BodyKind, PLAYER_FACTION};
use crate::world::{Event, World};
use gf_content::{BlockId, ContentDb, Objective, ResId, ResearchId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissionState {
    pub progress: f32,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub unlocked: BTreeSet<BlockId>,
    pub completed: BTreeSet<ResearchId>,
    pub active: Option<ResearchId>,
    /// Points accumulated toward the active research (or banked if none).
    pub points: f32,
    pub samples: BTreeMap<ResId, u32>,
    pub missions: Vec<MissionState>,
    pub ships_destroyed: u32,
    pub blocks_salvaged: u32,
    pub mined: BTreeMap<ResId, f32>,
    pub survived: f32,
}

impl Progress {
    pub fn new(content: &ContentDb) -> Self {
        Self {
            unlocked: (0..content.blocks.len() as BlockId)
                .filter(|b| content.blocks[*b as usize].def.starts_unlocked)
                .collect(),
            completed: BTreeSet::new(),
            active: None,
            points: 0.0,
            samples: BTreeMap::new(),
            missions: content.missions.iter().map(|_| MissionState { progress: 0.0, done: false }).collect(),
            ships_destroyed: 0,
            blocks_salvaged: 0,
            mined: BTreeMap::new(),
            survived: 0.0,
        }
    }

    /// Whether research `r` can be started (prerequisites done, samples held).
    pub fn can_research(&self, content: &ContentDb, r: ResearchId) -> bool {
        (r as usize) < content.research.len()
            && !self.completed.contains(&r)
            && content.research_requires[r as usize].iter().all(|p| self.completed.contains(p))
            && content.research_samples[r as usize].iter().all(|s| self.samples.get(s).copied().unwrap_or(0) > 0)
    }

    pub fn start_research(&mut self, content: &ContentDb, r: ResearchId) -> bool {
        if !self.can_research(content, r) {
            return false;
        }
        self.active = Some(r);
        true
    }

    pub fn add_points(&mut self, p: f32) {
        self.points += p;
    }

    pub fn add_sample(&mut self, r: ResId) {
        *self.samples.entry(r).or_insert(0) += 1;
    }

    pub fn note_mined(&mut self, r: ResId, q: f32) {
        *self.mined.entry(r).or_insert(0.0) += q;
    }
}

pub fn step(world: &mut World, dt: f32) {
    let content = world.content.clone();
    let alive = world.player_body().is_some_and(|b| b.kind == BodyKind::Ship && b.faction == Some(PLAYER_FACTION));
    let p = &mut world.progress;
    if alive {
        p.survived += dt;
    }
    if let Some(r) = p.active {
        let cost = content.research[r as usize].cost;
        if p.points >= cost {
            p.points -= cost;
            p.completed.insert(r);
            p.active = None;
            for b in &content.research_unlocks[r as usize] {
                p.unlocked.insert(*b);
            }
            world.events.push(Event::ResearchComplete { research: r });
        }
    }
    for (i, m) in content.missions.iter().enumerate() {
        if p.missions[i].done {
            continue;
        }
        let (value, goal) = match &m.objective {
            Objective::DestroyShips { count } => (p.ships_destroyed as f32, *count as f32),
            Objective::SalvageBlocks { count } => (p.blocks_salvaged as f32, *count as f32),
            Objective::MineResource { resource, amount } => {
                let v = content.res(resource).and_then(|r| p.mined.get(&r).copied()).unwrap_or(0.0);
                (v, *amount)
            }
            Objective::Survive { seconds } => (p.survived, *seconds),
        };
        p.missions[i].progress = (value / goal.max(1e-6)).min(1.0);
        if value >= goal {
            p.missions[i].done = true;
            p.points += m.reward.research_points;
            for s in &m.reward.samples {
                if let Some(r) = content.res(s) {
                    p.add_sample(r);
                }
            }
            for u in &m.reward.unlocks {
                if let Some(b) = content.block(u) {
                    p.unlocked.insert(b);
                }
            }
            world.events.push(Event::MissionComplete { mission: i });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn research_requires_samples_and_unlocks_blocks() {
        let c = Arc::new(ContentDb::load_dir(gf_content::default_content_dir()).unwrap());
        let mut w = World::new(c.clone(), 1);
        let rocketry = c.research_id("rocketry").unwrap();
        let adv = c.research_id("advanced_fabrication").unwrap();
        assert!(!w.progress.can_research(&c, rocketry), "needs prerequisite + sample");
        assert!(w.progress.start_research(&c, adv));
        w.progress.add_points(10_000.0);
        w.step();
        assert!(w.progress.completed.contains(&adv));
        assert!(!w.progress.can_research(&c, rocketry), "still needs the guidance sample");
        w.progress.add_sample(c.res("sample_guidance").unwrap());
        assert!(w.progress.start_research(&c, rocketry));
        w.step();
        let rl = c.block("rocket_launcher").unwrap();
        assert!(w.progress.unlocked.contains(&rl));
    }

    #[test]
    fn missions_complete_and_pay_out() {
        let c = Arc::new(ContentDb::load_dir(gf_content::default_content_dir()).unwrap());
        let mut w = World::new(c.clone(), 1);
        w.progress.ships_destroyed = 1;
        w.step();
        let i = c.missions.iter().position(|m| m.id == "first_blood").unwrap();
        assert!(w.progress.missions[i].done);
        assert!(w.progress.points >= 50.0);
    }
}
