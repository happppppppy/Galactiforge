//! Galactiforge simulation.
//!
//! Headless and deterministic: fixed 30 Hz step, ordered containers, `libm`
//! trigonometry and a world-owned PRNG. Every input enters through
//! [`world::Command`], so the same seed + command stream reproduces the same
//! state hash. That is the foundation for lock-step / rollback multiplayer.
//! The Bevy client (`gf_app`) only reads state and emits commands.

pub mod ai;
pub mod body;
pub mod combat;
pub mod feed;
pub mod grid;
pub mod logistics;
pub mod math;
pub mod mining;
pub mod progress;
pub mod rng;
pub mod sector;
pub mod thrust;
pub mod world;

pub use body::{BlockInst, Body, BodyId, BodyKind, Control, Faction, PIRATE_FACTION, PLAYER_FACTION, WeaponMode};
pub use math::{Cell, Vec2};
pub use world::{Command, DT, Event, TICK_RATE, World};
