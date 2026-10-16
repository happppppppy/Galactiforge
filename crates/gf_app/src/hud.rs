//! Text HUD: ship status, stock, research, missions, hovered block and help.

use crate::render::{buildable, hovered};
use crate::{Sim, UiState};
use bevy::prelude::*;
use gf_content::{Block, ContentDb, Objective};
use gf_sim::{Body, BodyKind};
use std::collections::BTreeMap;
use std::fmt::Write;

#[derive(Component)]
pub struct StatusText;
#[derive(Component)]
pub struct InfoText;
#[derive(Component)]
pub struct HelpText;

const HELP: &str = "WASD thrust/strafe  Q/E turn  Mouse aim  LMB/Space fire\n\
B build mode: [ ] or Z/X select, R rotate, LMB place, RMB deconstruct\n\
C cycle recipe  M weapon attack/suppress  F auto fire-suppression  T research\n\
1 power  2 feed  3 heat overlay  0 off  Scroll zoom  P pause\n\
F5 save  F9 load  F10 respawn  H hide help";

fn text(size: f32) -> (TextFont, TextColor) {
    (TextFont { font_size: FontSize::Px(size), ..default() }, TextColor(Color::srgb(0.85, 0.9, 1.0)))
}

pub fn setup(mut commands: Commands) {
    let (f, c) = text(14.0);
    commands.spawn((
        Text::new(""),
        f.clone(),
        c,
        Node { position_type: PositionType::Absolute, top: px(6), left: px(8), ..default() },
        StatusText,
    ));
    commands.spawn((
        Text::new(""),
        f.clone(),
        c,
        Node { position_type: PositionType::Absolute, top: px(6), right: px(8), ..default() },
        InfoText,
    ));
    commands.spawn((
        Text::new(HELP),
        f,
        TextColor(Color::srgba(0.8, 0.85, 1.0, 0.8)),
        Node { position_type: PositionType::Absolute, bottom: px(6), left: px(8), ..default() },
        HelpText,
    ));
}

fn tier_summary(content: &ContentDb, body: &Body, weapons: bool) -> String {
    let mut by_tier = [0u32; 5];
    for b in body.blocks.iter().flatten() {
        let hit = match content.blocks[b.def as usize].kind {
            Block::Weapon(_) => weapons,
            Block::Thruster(_) => !weapons,
            _ => false,
        };
        if hit {
            by_tier[(b.feed.tier.clamp(1, 5) - 1) as usize] += 1;
        }
    }
    let parts: Vec<String> =
        by_tier.iter().enumerate().filter(|(_, n)| **n > 0).map(|(t, n)| format!("{n}xT{}", t + 1)).collect();
    if parts.is_empty() { "none".into() } else { parts.join(" ") }
}

fn objective_target(o: &Objective) -> f32 {
    match o {
        Objective::DestroyShips { count } | Objective::SalvageBlocks { count } => *count as f32,
        Objective::MineResource { amount, .. } => *amount,
        Objective::Survive { seconds } => *seconds,
    }
}

pub fn update(
    time: Res<Time>,
    sim: Res<Sim>,
    mut ui: ResMut<UiState>,
    mut status: Single<&mut Text, (With<StatusText>, Without<InfoText>, Without<HelpText>)>,
    mut info: Single<&mut Text, (With<InfoText>, Without<StatusText>, Without<HelpText>)>,
    mut help: Single<&mut Visibility, With<HelpText>>,
) {
    let dt = time.delta_secs();
    for m in ui.messages.iter_mut() {
        m.1 -= dt;
    }
    ui.messages.retain(|m| m.1 > 0.0);
    **help = if ui.show_help { Visibility::Inherited } else { Visibility::Hidden };

    let world = &sim.world;
    let content = &world.content;
    let mut s = String::new();
    let _ = writeln!(s, "GALACTIFORGE   tick {}{}", world.tick, if sim.paused { "   [PAUSED]" } else { "" });
    match world.player_body() {
        Some(p) if p.kind == BodyKind::Ship => {
            let _ = writeln!(s, "{}: {} blocks, mass {:.0}, speed {:.1}", p.name, p.block_count(), p.mass, p.vel.length());
            let _ = writeln!(s, "Weapons: {}", tier_summary(content, p, true));
            let _ = writeln!(s, "Thrusters: {}", tier_summary(content, p, false));
            let fires = p.blocks.iter().flatten().filter(|b| b.fire > 0.0).count();
            if fires > 0 {
                let _ = writeln!(s, "!! {fires} blocks ON FIRE - feed frost ammo to a cannon (M / F)");
            }
            let mut stock: BTreeMap<u16, f32> = BTreeMap::new();
            for b in p.blocks.iter().flatten() {
                if matches!(content.blocks[b.def as usize].kind, Block::Storage { .. } | Block::Tank { .. }) {
                    for (r, q) in &b.stock {
                        *stock.entry(*r).or_default() += q;
                    }
                }
            }
            let items: Vec<String> = stock
                .iter()
                .filter(|(_, q)| **q >= 0.5)
                .map(|(r, q)| format!("{} {:.0}", content.resources[*r as usize].name, q))
                .collect();
            let _ = writeln!(s, "Stock: {}", if items.is_empty() { "empty".into() } else { items.join(", ") });
        }
        _ => {
            let _ = writeln!(s, "SHIP LOST - press F10 to launch a new Forge Runner");
        }
    }
    let pr = &world.progress;
    match pr.active {
        Some(r) => {
            let def = &content.research[r as usize];
            let _ = writeln!(s, "Research: {} {:.0}/{:.0}", def.name, pr.points, def.cost);
        }
        None => {
            let _ = writeln!(s, "Research: idle ({:.0} pts) - T to start", pr.points);
        }
    }
    let samples: Vec<String> =
        pr.samples.iter().map(|(r, n)| format!("{} x{n}", content.resources[*r as usize].name)).collect();
    if !samples.is_empty() {
        let _ = writeln!(s, "Samples: {}", samples.join(", "));
    }
    for (m, st) in content.missions.iter().zip(&pr.missions) {
        if !st.done {
            let _ = writeln!(s, "Mission: {} {:.0}/{:.0}", m.name, st.progress, objective_target(&m.objective));
        }
    }
    let _ = writeln!(s, "Destroyed {}  Salvaged {}", pr.ships_destroyed, pr.blocks_salvaged);
    for (m, _) in &ui.messages {
        let _ = writeln!(s, "> {m}");
    }
    status.0 = s;

    let mut i = String::new();
    if ui.build {
        let ids = buildable(content);
        if !ids.is_empty() {
            let id = ids[ui.build_idx % ids.len()];
            let b = &content.blocks[id as usize];
            let cost: Vec<String> =
                b.cost.iter().map(|(r, n)| format!("{} {n}", content.resources[*r as usize].name)).collect();
            let _ = writeln!(i, "BUILD: {} (rot {})", b.def.name, ui.rot);
            let _ = writeln!(i, "Cost: {}", cost.join(", "));
            if !pr.unlocked.contains(&id) {
                let _ = writeln!(i, "LOCKED - research required");
            }
            let (w, h) = b.reserved_size();
            if (w, h) != (1, 1) {
                let _ = writeln!(i, "Keep {w}x{h} free so it can grow");
            }
        }
    }
    if let Some((body, slot)) = ui.cursor.and_then(|c| hovered(world, c)) {
        let b = body.block(slot).expect("hovered slot exists");
        let def = &content.blocks[b.def as usize];
        let _ = writeln!(i, "{} [{}]", def.def.name, body.name);
        let _ = writeln!(i, "HP {:.0}/{:.0}", b.hp, b.max_hp(content));
        if def.growth().is_some() {
            let _ = writeln!(i, "Tier {} (max {}) feed {:.2}/s size {}x{}", b.feed.tier, b.feed.cap, b.feed.rate, b.size.0, b.size.1);
        }
        if let Block::Weapon(_) = def.kind {
            let _ = writeln!(i, "Mode {:?}", b.mode);
        }
        if let Some(r) = b.recipe {
            let _ = writeln!(i, "Recipe {} ({:.0}%)", content.recipes[r as usize].id, b.progress * 100.0);
        }
        if b.power_req > 0.0 {
            let _ = writeln!(i, "Power {:.0}% of {:.1} kW", b.power_sat * 100.0, b.power_req);
        }
        if b.heat > 0.5 {
            let _ = writeln!(i, "Heat {:.0}", b.heat);
        }
        if b.fire > 0.0 {
            let _ = writeln!(i, "ON FIRE");
        }
        for (r, q) in b.stock.iter().filter(|e| e.1 >= 0.05) {
            let _ = writeln!(i, "  {} {:.1}", content.resources[*r as usize].name, q);
        }
    }
    info.0 = i;
}
