//! Keyboard / mouse input. Everything that changes the game becomes a
//! `gf_sim::Command`, so the same input path works for replays and future
//! lock-step multiplayer.

use crate::render::{Views, buildable, hovered};
use crate::{SAVE_PATH, Sim, TILE, UiState};
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use gf_sim::{Command, Control, WeaponMode, World};

pub fn cursor(
    mut ui: ResMut<UiState>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    let (camera, gt) = *camera;
    ui.cursor = window
        .cursor_position()
        .and_then(|p| camera.viewport_to_world_2d(gt, p).ok())
        .map(|p| gf_sim::Vec2::new(p.x / TILE, p.y / TILE));
}

#[allow(clippy::too_many_arguments)]
pub fn controls(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    scroll: Res<AccumulatedMouseScroll>,
    mut sim: ResMut<Sim>,
    mut ui: ResMut<UiState>,
    mut views: ResMut<Views>,
    mut last: Local<Option<Control>>,
    mut auto_suppress: Local<Option<bool>>,
) {
    let auto = auto_suppress.get_or_insert(true);
    // Camera zoom.
    if scroll.delta.y != 0.0 {
        ui.zoom = (ui.zoom * (1.0 - 0.1 * scroll.delta.y.signum())).clamp(0.25, 8.0);
    }
    if keys.just_pressed(KeyCode::KeyH) {
        ui.show_help = !ui.show_help;
    }
    if keys.just_pressed(KeyCode::KeyP) {
        sim.paused = !sim.paused;
    }
    for (k, o) in [
        (KeyCode::Digit0, 0),
        (KeyCode::Digit1, 1),
        (KeyCode::Digit2, 2),
        (KeyCode::Digit3, 3),
    ] {
        if keys.just_pressed(k) {
            ui.overlay = if ui.overlay == o { 0 } else { o };
        }
    }
    if keys.just_pressed(KeyCode::F5) {
        save(&sim.world, &mut ui);
    }
    if keys.just_pressed(KeyCode::F9) {
        match std::fs::read_to_string(SAVE_PATH)
            .map_err(|e| e.to_string())
            .and_then(|t| World::load_ron(&t, sim.world.content.clone()).map_err(|e| e.to_string()))
        {
            Ok(w) => {
                sim.world = w;
                views.clear(&mut commands);
                *last = None;
                ui.say("Game loaded");
            }
            Err(e) => ui.say(format!("Load failed: {e}")),
        }
    }
    if keys.just_pressed(KeyCode::F10)
        && !sim.world.player_alive()
        && sim.world.respawn_player().is_some()
    {
        *last = None;
        ui.say("New Forge Runner launched");
    }
    let Some(player) = sim.world.player.filter(|_| sim.world.player_alive()) else {
        return;
    };
    let content = sim.world.content.clone();

    if keys.just_pressed(KeyCode::KeyF) {
        *auto = !*auto;
        ui.say(if *auto {
            "Auto fire-suppression ON"
        } else {
            "Auto fire-suppression OFF"
        });
    }
    if keys.just_pressed(KeyCode::KeyT) {
        let p = &sim.world.progress;
        match (0..content.research.len() as u16).find(|&r| p.can_research(&content, r)) {
            Some(r) if p.active.is_none() => {
                sim.world.queue(Command::StartResearch { research: r });
                ui.say(format!(
                    "Researching: {}",
                    content.research[r as usize].name
                ));
            }
            Some(_) => ui.say("A research project is already running"),
            None => ui.say("No research available - find samples by salvaging"),
        }
    }

    // Build mode.
    if keys.just_pressed(KeyCode::KeyB) {
        ui.build = !ui.build;
    }
    let palette = buildable(&content);
    if ui.build && !palette.is_empty() {
        let n = palette.len();
        if keys.just_pressed(KeyCode::BracketRight) || keys.just_pressed(KeyCode::KeyX) {
            ui.build_idx = (ui.build_idx + 1) % n;
        }
        if keys.just_pressed(KeyCode::BracketLeft) || keys.just_pressed(KeyCode::KeyZ) {
            ui.build_idx = (ui.build_idx + n - 1) % n;
        }
        if keys.just_pressed(KeyCode::KeyR) {
            ui.rot = (ui.rot + 1) % 4;
        }
    }
    let body = &sim.world.bodies[&player];
    let cell = ui.cursor.map(|c| body.local_cell_at(c));
    let on_player = ui
        .cursor
        .and_then(|c| hovered(&sim.world, c))
        .filter(|(b, _)| b.id == player);
    let mut queue = Vec::new();
    if let Some(cell) = cell {
        if ui.build && mouse.just_pressed(MouseButton::Left) && !palette.is_empty() {
            let block = palette[ui.build_idx % palette.len()];
            queue.push(Command::Build {
                body: player,
                block,
                cell,
                rot: ui.rot,
            });
        }
        if ui.build && mouse.just_pressed(MouseButton::Right) {
            queue.push(Command::Deconstruct { body: player, cell });
        }
    }
    if let Some((b, slot)) = on_player {
        let anchor = b
            .block(slot)
            .map(|i| i.anchor)
            .expect("hovered slot exists");
        if keys.just_pressed(KeyCode::KeyC) {
            queue.push(Command::CycleRecipe {
                body: player,
                cell: anchor,
            });
        }
        if keys.just_pressed(KeyCode::KeyM) {
            let mode = match b.block(slot).map(|i| i.mode) {
                Some(WeaponMode::Attack) => WeaponMode::Suppress,
                _ => WeaponMode::Attack,
            };
            queue.push(Command::SetWeaponMode {
                body: player,
                cell: anchor,
                mode,
            });
        }
    }

    // Flight controls (local frame: +y forward, +x right).
    let axis =
        |a: KeyCode, b: KeyCode| keys.pressed(a) as i32 as f32 - keys.pressed(b) as i32 as f32;
    let control = Control {
        thrust: gf_sim::Vec2::new(
            axis(KeyCode::KeyD, KeyCode::KeyA),
            axis(KeyCode::KeyW, KeyCode::KeyS),
        ),
        turn: axis(KeyCode::KeyQ, KeyCode::KeyE),
        fire: keys.pressed(KeyCode::Space) || (!ui.build && mouse.pressed(MouseButton::Left)),
        aim: ui.cursor,
        auto_suppress: *auto,
    };
    if last.as_ref() != Some(&control) {
        *last = Some(control.clone());
        queue.push(Command::Control {
            body: player,
            control,
        });
    }
    for c in queue {
        sim.world.queue(c);
    }
}

fn save(world: &World, ui: &mut UiState) {
    let res = world
        .save_ron()
        .map_err(|e| e.to_string())
        .and_then(|text| {
            if let Some(dir) = std::path::Path::new(SAVE_PATH).parent() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            std::fs::write(SAVE_PATH, text).map_err(|e| e.to_string())
        });
    match res {
        Ok(()) => ui.say(format!("Saved to {SAVE_PATH}")),
        Err(e) => ui.say(format!("Save failed: {e}")),
    }
}
