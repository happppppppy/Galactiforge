//! Galactiforge client: renders the deterministic simulation with the original
//! 32x32 tileset and turns keyboard / mouse input into simulation commands.
//! All game rules live in `gf_sim`; this crate only reads state and queues
//! [`gf_sim::Command`]s.

mod hud;
mod input;
mod render;

use bevy::prelude::*;
use gf_content::ContentDb;
use gf_sim::{Event as SimEvent, World};
use std::collections::VecDeque;
use std::sync::Arc;

/// Pixels per grid cell (the tileset is 32x32).
pub const TILE: f32 = 32.0;
pub const SAVE_PATH: &str = "saves/quicksave.ron";

#[derive(Resource)]
pub struct Sim {
    pub world: World,
    pub acc: f32,
    pub paused: bool,
}

/// Simulation events produced since the last frame (presentation only).
#[derive(Resource, Default)]
pub struct FrameEvents(pub Vec<SimEvent>);

#[derive(Resource)]
pub struct UiState {
    pub build: bool,
    pub build_idx: usize,
    pub rot: u8,
    /// 0 = normal, 1 = power, 2 = fuel/ammo feed, 3 = heat & fire.
    pub overlay: u8,
    pub zoom: f32,
    pub messages: VecDeque<(String, f32)>,
    /// Cursor position in simulation units.
    pub cursor: Option<gf_sim::Vec2>,
    pub show_help: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            build: false,
            build_idx: 0,
            rot: 0,
            overlay: 0,
            zoom: 1.0,
            messages: VecDeque::new(),
            cursor: None,
            show_help: true,
        }
    }
}

impl UiState {
    pub fn say(&mut self, msg: impl Into<String>) {
        self.messages.push_back((msg.into(), 6.0));
        while self.messages.len() > 6 {
            self.messages.pop_front();
        }
    }
}

fn asset_root() -> String {
    std::env::var("GF_ASSETS")
        .unwrap_or_else(|_| format!("{}/../../assets", env!("CARGO_MANIFEST_DIR")))
}

fn main() {
    let content_dir = std::path::PathBuf::from(asset_root()).join("content");
    let content = match ContentDb::load_dir(&content_dir) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            eprintln!(
                "failed to load content from {}: {e:?}",
                content_dir.display()
            );
            std::process::exit(1);
        }
    };
    let seed = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1337);
    let world = World::new_game(content, seed);

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    file_path: asset_root(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Galactiforge".into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.05)))
        .insert_resource(Sim {
            world,
            acc: 0.0,
            paused: false,
        })
        .init_resource::<FrameEvents>()
        .init_resource::<UiState>()
        .init_resource::<render::Views>()
        .add_systems(Startup, (render::setup, hud::setup))
        .add_systems(
            Update,
            (
                input::cursor,
                input::controls,
                step_sim,
                render::sync_bodies,
                render::sync_projectiles,
                render::spawn_effects,
                render::animate_effects,
                render::draw_gizmos,
                render::build_ghost,
                render::follow_camera,
                hud::update,
            )
                .chain(),
        )
        .run();
}

/// Fixed-step accumulator: the simulation always advances in 1/30 s ticks.
fn step_sim(
    time: Res<Time>,
    mut sim: ResMut<Sim>,
    mut events: ResMut<FrameEvents>,
    mut ui: ResMut<UiState>,
) {
    events.0.clear();
    if sim.paused {
        return;
    }
    sim.acc = (sim.acc + time.delta_secs()).min(gf_sim::DT * 5.0);
    while sim.acc >= gf_sim::DT {
        sim.acc -= gf_sim::DT;
        sim.world.step();
        let content = sim.world.content.clone();
        let player = sim.world.player;
        for e in &sim.world.events {
            match e {
                SimEvent::ResearchComplete { research } => ui.say(format!(
                    "Research complete: {}",
                    content.research[*research as usize].name
                )),
                SimEvent::MissionComplete { mission } => ui.say(format!(
                    "Mission complete: {}",
                    content.missions[*mission].name
                )),
                SimEvent::ShipDestroyed { body, .. } if Some(*body) == player => {
                    ui.say("Your core was destroyed! Press F10 to respawn.")
                }
                SimEvent::ShipDestroyed { .. } => ui.say("Ship disabled - salvage the wreck!"),
                SimEvent::Message(m) => ui.say(m.clone()),
                _ => {}
            }
        }
        events.0.extend(sim.world.events.iter().cloned());
    }
}
