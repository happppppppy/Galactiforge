//! Sprite rendering of bodies, projectiles and effects using the original
//! 32x32 tileset. Bodies map to a root entity with one child sprite per block;
//! children are only rebuilt when the body's block layout changes
//! (`Body::revision`), while colours, animation frames and turret angles are
//! refreshed every frame.

use crate::{FrameEvents, Sim, UiState, TILE};
use bevy::prelude::*;
use gf_content::{Block, BlockId, BlockInfo, Category, ContentDb, WeaponKind};
use gf_sim::body::BlockInst;
use gf_sim::math::{facing_vec, footprint, Cell};
use gf_sim::{Body, BodyId, BodyKind, Event as SimEvent};
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

pub const TILESET: &str = "images/galactiforge_tilesets/factory_devices.png";
/// Tileset frames used for weapon tiers 1..=5: bigger tiers use the later,
/// more detailed frames of the 7-frame weapon strip.
const TIER_FRAMES: [usize; 5] = [0, 2, 3, 5, 6];
const EXPLOSION_TILE: usize = 90;
const EXPLOSION_FRAMES: usize = 7;
const BULLET_TILE: usize = 100;
const TILE_COUNT: usize = 200;

#[derive(Resource)]
pub struct Art {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
pub struct Ghost;

#[derive(Component)]
pub struct Fx {
    age: f32,
    life: f32,
    first: usize,
    frames: usize,
}

type SlotSig = (BlockId, Cell, u8, (u8, u8));

struct SlotView {
    sig: SlotSig,
    main: Entity,
    under: Vec<Entity>,
}

pub struct BodyView {
    root: Entity,
    revision: u64,
    slots: Vec<Option<SlotView>>,
}

#[derive(Resource, Default)]
pub struct Views {
    pub bodies: HashMap<BodyId, BodyView>,
    projectiles: Vec<Entity>,
    beams: Vec<(Vec2, Vec2, u8, f32)>,
    last_cam: Vec2,
}

impl Views {
    /// Drop every rendered body (used after loading a save).
    pub fn clear(&mut self, commands: &mut Commands) {
        for (_, v) in self.bodies.drain() {
            commands.entity(v.root).despawn();
        }
        self.beams.clear();
    }
}

/// Simulation units -> pixels.
pub fn px(p: gf_sim::Vec2) -> Vec2 {
    Vec2::new(p.x, p.y) * TILE
}

pub fn atlas_sprite(art: &Art, index: usize, size: Vec2, color: Color) -> Sprite {
    let mut s = Sprite::from_atlas_image(
        art.image.clone(),
        TextureAtlas { layout: art.layout.clone(), index: index.min(TILE_COUNT - 1) },
    );
    s.custom_size = Some(size);
    s.color = color;
    s
}

pub fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let image = assets.load(TILESET);
    let layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(32), 10, 20, None, None));
    commands.spawn(Camera2d);
    let art = Art { image, layout };
    let mut ghost = atlas_sprite(&art, 0, Vec2::splat(TILE), Color::NONE);
    ghost.color = Color::NONE;
    commands.spawn((ghost, Transform::from_xyz(0.0, 0.0, 8.0), Visibility::Hidden, Ghost));
    commands.insert_resource(art);
}

fn base_tint(info: &BlockInfo) -> Vec3 {
    info.def.sprite.tint.map_or(Vec3::ONE, |(r, g, b)| Vec3::new(r, g, b))
}

fn heat_capacity(info: &BlockInfo) -> f32 {
    match &info.kind {
        Block::Weapon(w) => w.def.heat_capacity,
        Block::Thruster(t) => t.def.heat_capacity,
        _ => 100.0,
    }
}

fn block_color(content: &ContentDb, body: &Body, b: &BlockInst, overlay: u8, t: f32) -> Color {
    let info = &content.blocks[b.def as usize];
    let mut c = base_tint(info);
    let hpf = (b.hp / b.max_hp(content).max(1e-3)).clamp(0.0, 1.0);
    c *= 0.55 + 0.45 * hpf;
    if body.kind == BodyKind::Derelict {
        let grey = c.dot(Vec3::splat(1.0 / 3.0));
        c = c.lerp(Vec3::splat(grey), 0.6) * 0.65;
    }
    match overlay {
        1 => {
            c = if info.conducts_power() {
                Vec3::new(1.0, 0.9, 0.3)
            } else if b.power_req > 0.0 {
                Vec3::new(1.0, 0.2, 0.2).lerp(Vec3::new(0.3, 1.0, 0.3), b.power_sat.clamp(0.0, 1.0))
            } else {
                c * 0.3
            }
        }
        2 => {
            c = match &info.kind {
                Block::Weapon(_) | Block::Thruster(_) => {
                    let k = (b.feed.tier.saturating_sub(1)) as f32 / 4.0;
                    Vec3::new(0.3, 0.5, 1.0).lerp(Vec3::new(1.0, 0.3, 1.0), k)
                }
                Block::Pipe | Block::Tank { .. } => Vec3::new(0.3, 0.7, 1.0),
                Block::Conveyor { .. } | Block::Splitter { .. } | Block::Storage { .. } => Vec3::new(1.0, 0.7, 0.3),
                _ => c * 0.3,
            }
        }
        3 => {
            let h = (b.heat / heat_capacity(info).max(1e-3)).clamp(0.0, 1.0);
            c = Vec3::splat(0.25).lerp(Vec3::new(1.0, 0.2, 0.0), h);
        }
        _ => {}
    }
    if b.fire > 0.0 {
        let flicker = 0.75 + 0.25 * (t * 17.0 + b.anchor.x as f32 * 3.1 + b.anchor.y as f32 * 1.7).sin();
        c = c.lerp(Vec3::new(1.0, 0.45, 0.1), (0.35 + 0.4 * b.fire.min(1.0)) * flicker);
    }
    if b.chill > 0.0 {
        c = c.lerp(Vec3::new(0.6, 0.85, 1.0), 0.5 * b.chill.min(1.0));
    }
    if b.disabled > 0.0 {
        c = c.lerp(Vec3::new(0.5, 0.6, 1.0), 0.5);
    }
    Color::srgb(c.x, c.y, c.z)
}

fn block_frame(info: &BlockInfo, b: &BlockInst, t: f32) -> usize {
    let s = &info.def.sprite;
    let frames = s.frames.max(1) as usize;
    let tile = s.tile as usize;
    match &info.kind {
        Block::Weapon(_) => {
            let tier = (b.feed.tier.clamp(1, 5) - 1) as usize;
            if frames >= 7 { tile + TIER_FRAMES[tier] } else { tile + tier.min(frames - 1) }
        }
        _ if frames > 1 => {
            let active = b.throttle > 0.05 || (b.progress > 0.0 && b.power_sat > 0.1) || b.work > 0.0;
            if active { tile + ((t * s.fps.max(1.0)) as usize) % frames } else { tile }
        }
        _ => tile,
    }
}

/// Size and local rotation of the main sprite of a block.
fn block_geometry(info: &BlockInfo, b: &BlockInst) -> (Vec2, f32) {
    let (w, h) = (b.size.0 as f32, b.size.1 as f32);
    match &info.kind {
        Block::Weapon(_) => (Vec2::splat(w.max(h) * TILE), b.aim - FRAC_PI_2),
        _ => (Vec2::new(w, h) * TILE, b.rot as f32 * FRAC_PI_2),
    }
}

fn spawn_slot(commands: &mut Commands, art: &Art, content: &ContentDb, root: Entity, b: &BlockInst) -> SlotView {
    let info = &content.blocks[b.def as usize];
    let mut under = Vec::new();
    if let Some(base) = info.def.sprite.base {
        for c in b.cells() {
            under.push(
                commands
                    .spawn((
                        atlas_sprite(art, base as usize, Vec2::splat(TILE), Color::WHITE),
                        Transform::from_translation(px(c.center()).extend(0.0)),
                        ChildOf(root),
                    ))
                    .id(),
            );
        }
    }
    let (size, rot) = block_geometry(info, b);
    let z = if matches!(info.def.category, Category::Hull) { 0.05 } else { 0.1 };
    let main = commands
        .spawn((
            atlas_sprite(art, info.def.sprite.tile as usize, size, Color::WHITE),
            Transform::from_translation(px(b.center()).extend(z)).with_rotation(Quat::from_rotation_z(rot)),
            ChildOf(root),
        ))
        .id();
    SlotView { sig: (b.def, b.anchor, b.rot, b.size), main, under }
}

fn body_transform(b: &Body) -> Transform {
    let origin = b.pos - b.com.rotate(b.angle);
    Transform::from_translation(px(origin).extend(0.0)).with_rotation(Quat::from_rotation_z(b.angle))
}

pub fn sync_bodies(
    mut commands: Commands,
    sim: Res<Sim>,
    art: Res<Art>,
    ui: Res<UiState>,
    time: Res<Time>,
    mut views: ResMut<Views>,
    mut transforms: Query<&mut Transform>,
    mut sprites: Query<&mut Sprite>,
) {
    let world = &sim.world;
    let content = world.content.clone();
    let t = time.elapsed_secs();
    views.bodies.retain(|id, v| {
        let keep = world.bodies.contains_key(id);
        if !keep {
            commands.entity(v.root).despawn();
        }
        keep
    });
    for (id, body) in &world.bodies {
        let view = views.bodies.entry(*id).or_insert_with(|| BodyView {
            root: commands.spawn((body_transform(body), Visibility::default())).id(),
            revision: u64::MAX,
            slots: Vec::new(),
        });
        if let Ok(mut tf) = transforms.get_mut(view.root) {
            *tf = body_transform(body);
        }
        if view.revision != body.revision {
            view.revision = body.revision;
            view.slots.resize_with(body.blocks.len().max(view.slots.len()), || None);
            for (i, slot) in view.slots.iter_mut().enumerate() {
                let want = body.blocks.get(i).and_then(|b| b.as_ref());
                let same = match (&*slot, want) {
                    (Some(s), Some(b)) => s.sig == (b.def, b.anchor, b.rot, b.size),
                    (None, None) => true,
                    _ => false,
                };
                if same {
                    continue;
                }
                if let Some(old) = slot.take() {
                    commands.entity(old.main).despawn();
                    for e in old.under {
                        commands.entity(e).despawn();
                    }
                }
                if let Some(b) = want {
                    *slot = Some(spawn_slot(&mut commands, &art, &content, view.root, b));
                }
            }
            view.slots.truncate(body.blocks.len());
        }
        for (i, slot) in view.slots.iter().enumerate() {
            let (Some(sv), Some(Some(b))) = (slot, body.blocks.get(i)) else { continue };
            let info = &content.blocks[b.def as usize];
            let color = block_color(&content, body, b, ui.overlay, t);
            if let Ok(mut s) = sprites.get_mut(sv.main) {
                s.color = color;
                if let Some(a) = s.texture_atlas.as_mut() {
                    a.index = block_frame(info, b, t).min(TILE_COUNT - 1);
                }
            }
            if matches!(info.kind, Block::Weapon(_)) {
                if let Ok(mut tf) = transforms.get_mut(sv.main) {
                    tf.rotation = Quat::from_rotation_z(b.aim - FRAC_PI_2);
                }
            }
            for e in &sv.under {
                if let Ok(mut s) = sprites.get_mut(*e) {
                    s.color = color;
                }
            }
        }
    }
}

pub fn sync_projectiles(
    mut commands: Commands,
    sim: Res<Sim>,
    art: Res<Art>,
    mut views: ResMut<Views>,
    mut q: Query<(&mut Transform, &mut Sprite, &mut Visibility)>,
) {
    let world = &sim.world;
    while views.projectiles.len() < world.projectiles.len() {
        let e = commands
            .spawn((
                atlas_sprite(&art, BULLET_TILE, Vec2::splat(TILE * 0.5), Color::WHITE),
                Transform::from_xyz(0.0, 0.0, 5.0),
                Visibility::Hidden,
            ))
            .id();
        views.projectiles.push(e);
    }
    for (i, e) in views.projectiles.iter().enumerate() {
        let Ok((mut tf, mut sprite, mut vis)) = q.get_mut(*e) else { continue };
        let Some(p) = world.projectiles.get(i) else {
            *vis = Visibility::Hidden;
            continue;
        };
        *vis = Visibility::Inherited;
        let kind = match &world.content.blocks.get(p.def as usize).map(|b| &b.kind) {
            Some(Block::Weapon(w)) => w.def.kind,
            _ => WeaponKind::Cannon,
        };
        let (index, color) = if p.suppress {
            (BULLET_TILE + 2, Color::srgb(0.6, 0.9, 1.0))
        } else if kind == WeaponKind::Rocket {
            (BULLET_TILE + 1, Color::WHITE)
        } else if p.fx.incendiary > 0.0 {
            (BULLET_TILE, Color::srgb(1.0, 0.6, 0.3))
        } else if p.fx.frost > 0.0 {
            (BULLET_TILE, Color::srgb(0.6, 0.9, 1.0))
        } else {
            (BULLET_TILE, Color::WHITE)
        };
        sprite.color = color;
        sprite.custom_size = Some(Vec2::splat(TILE * (0.45 + 0.15 * p.tier as f32)));
        if let Some(a) = sprite.texture_atlas.as_mut() {
            a.index = index;
        }
        tf.translation = px(p.pos).extend(5.0);
        tf.rotation = Quat::from_rotation_z(p.vel.angle() - FRAC_PI_2);
    }
}

fn spawn_fx(commands: &mut Commands, art: &Art, at: Vec2, size: f32, color: Color, life: f32) {
    commands.spawn((
        atlas_sprite(art, EXPLOSION_TILE, Vec2::splat(size), color),
        Transform::from_translation(at.extend(6.0)),
        Fx { age: 0.0, life, first: EXPLOSION_TILE, frames: EXPLOSION_FRAMES },
    ));
}

pub fn spawn_effects(
    mut commands: Commands,
    art: Res<Art>,
    events: Res<FrameEvents>,
    mut views: ResMut<Views>,
    mut fx: Query<&mut Transform, With<Fx>>,
) {
    for e in &events.0 {
        match e {
            SimEvent::Explosion { pos, radius } => {
                spawn_fx(&mut commands, &art, px(*pos), (radius * 2.0).max(1.0) * TILE, Color::WHITE, 0.45)
            }
            SimEvent::BlockDestroyed { pos, .. } => spawn_fx(&mut commands, &art, px(*pos), TILE * 1.2, Color::WHITE, 0.35),
            SimEvent::Hit { pos, damage } => spawn_fx(
                &mut commands,
                &art,
                px(*pos),
                TILE * (0.4 + (damage / 40.0).min(0.8)),
                Color::srgb(1.0, 0.9, 0.6),
                0.15,
            ),
            SimEvent::Extinguish { pos, radius } => spawn_fx(
                &mut commands,
                &art,
                px(*pos),
                (radius * 2.0).max(1.0) * TILE,
                Color::srgba(0.5, 0.85, 1.0, 0.8),
                0.5,
            ),
            SimEvent::Salvaged { pos, .. } => {
                spawn_fx(&mut commands, &art, px(*pos), TILE, Color::srgba(0.4, 1.0, 0.5, 0.8), 0.3)
            }
            SimEvent::Laser { from, to, tier } => views.beams.push((px(*from), px(*to), *tier, 0.08)),
            SimEvent::OriginShift { delta } => {
                let d = px(*delta);
                for mut tf in &mut fx {
                    tf.translation -= d.extend(0.0);
                }
                for b in &mut views.beams {
                    b.0 -= d;
                    b.1 -= d;
                }
            }
            _ => {}
        }
    }
}

pub fn animate_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut views: ResMut<Views>,
    mut q: Query<(Entity, &mut Fx, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (e, mut fx, mut sprite) in &mut q {
        fx.age += dt;
        if fx.age >= fx.life {
            commands.entity(e).despawn();
            continue;
        }
        let f = ((fx.age / fx.life) * fx.frames as f32) as usize;
        if let Some(a) = sprite.texture_atlas.as_mut() {
            a.index = fx.first + f.min(fx.frames - 1);
        }
    }
    views.beams.retain_mut(|b| {
        b.3 -= dt;
        b.3 > 0.0
    });
}

fn star_hash(i: u32) -> (f32, f32, f32) {
    let mut x = i.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    let mut next = || {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        (x & 0xFFFF) as f32 / 65535.0
    };
    (next(), next(), next())
}

pub fn draw_gizmos(mut gizmos: Gizmos, sim: Res<Sim>, ui: Res<UiState>, views: Res<Views>, window: Single<&Window>) {
    let world = &sim.world;
    let content = &world.content;
    // Parallax starfield, wrapped around the camera.
    let cam = views.last_cam;
    let span = Vec2::new(window.width(), window.height()) * ui.zoom * 1.2 + Vec2::splat(64.0);
    for i in 0..260u32 {
        let (x, y, depth) = star_hash(i);
        let par = 0.05 + 0.25 * depth;
        let p = Vec2::new(x, y) * span - cam * par;
        let p = Vec2::new(p.x.rem_euclid(span.x), p.y.rem_euclid(span.y)) - span * 0.5 + cam;
        let b = 0.25 + 0.5 * depth;
        gizmos.circle_2d(p, (0.6 + depth) * ui.zoom.max(1.0), Color::srgb(b, b, b * 1.1));
    }
    // Mining / laser beams.
    for (from, to, tier, _) in &views.beams {
        let w = 0.4 + 0.15 * *tier as f32;
        gizmos.line_2d(*from, *to, Color::srgb(0.5, 1.0, 1.0));
        let n = (*to - *from).perp().normalize_or_zero() * w * 3.0;
        gizmos.line_2d(*from + n, *to + n, Color::srgba(0.3, 0.8, 1.0, 0.5));
        gizmos.line_2d(*from - n, *to - n, Color::srgba(0.3, 0.8, 1.0, 0.5));
    }
    // Thruster flames: length and colour grow with the feed tier.
    for body in world.bodies.values() {
        if body.kind != BodyKind::Ship {
            continue;
        }
        for b in body.blocks.iter().flatten() {
            if b.throttle <= 0.02 || !matches!(content.blocks[b.def as usize].kind, Block::Thruster(_)) {
                continue;
            }
            let dir = facing_vec(b.rot);
            let back = gf_sim::Vec2::new(-dir.x, -dir.y);
            let depth = b.size.1 as f32;
            let nozzle = body.to_world(b.center() + back * (depth * 0.5));
            let wdir = body.dir_to_world(back);
            let len = (0.6 + 0.5 * b.feed.tier as f32) * b.throttle * (1.0 + 0.15 * (b.rr as f32 * 0.7).sin());
            let half = 0.25 * b.size.0 as f32;
            let side = body.dir_to_world(dir.perp()) * half;
            let tip = px(nozzle + wdir * len);
            let hot = Color::srgb(1.0, 0.9, 0.5);
            let cool = if b.feed.tier >= 3 { Color::srgb(0.5, 0.7, 1.0) } else { Color::srgb(1.0, 0.45, 0.1) };
            gizmos.line_2d(px(nozzle + side), tip, cool);
            gizmos.line_2d(px(nozzle - side), tip, cool);
            gizmos.line_2d(px(nozzle), px(nozzle + wdir * (len * 0.6)), hot);
        }
    }
    // Hovered block outline.
    if let Some(c) = ui.cursor {
        if let Some((body, slot)) = hovered(world, c) {
            let b = body.block(slot).expect("hovered slot exists");
            outline_cells(&mut gizmos, body, b.cells(), Color::srgba(1.0, 1.0, 1.0, 0.6));
        }
    }
    // Build mode: reserved growth footprint of the selected block.
    if ui.build {
        if let (Some(c), Some(body)) = (ui.cursor, world.player_body()) {
            let ids = buildable(content);
            if let Some(&id) = ids.get(ui.build_idx % ids.len().max(1)) {
                let cell = body.local_cell_at(c);
                let size = content.blocks[id as usize].reserved_size();
                let col = Color::srgba(0.4, 1.0, 0.4, 0.5);
                outline_cells(&mut gizmos, body, footprint(cell, ui.rot, size), col);
            }
        }
    }
}

fn outline_cells(gizmos: &mut Gizmos, body: &Body, cells: impl Iterator<Item = Cell>, color: Color) {
    for c in cells {
        let corners = [(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)]
            .map(|(x, y)| px(body.to_world(c.center() + gf_sim::Vec2::new(x, y))));
        for i in 0..4 {
            gizmos.line_2d(corners[i], corners[(i + 1) % 4], color);
        }
    }
}

/// Body and slot under a world-space point (simulation units).
pub fn hovered(world: &gf_sim::World, p: gf_sim::Vec2) -> Option<(&Body, u32)> {
    world.bodies.values().find_map(|b| {
        if b.pos.distance(p) > b.radius + 1.0 {
            return None;
        }
        b.block_at(b.local_cell_at(p)).map(|s| (b, s))
    })
}

/// Blocks offered in the build palette, in content order.
pub fn buildable(content: &ContentDb) -> Vec<BlockId> {
    content.blocks.iter().enumerate().filter(|(_, b)| b.def.buildable).map(|(i, _)| i as BlockId).collect()
}

pub fn build_ghost(
    sim: Res<Sim>,
    ui: Res<UiState>,
    ghost: Single<(&mut Transform, &mut Sprite, &mut Visibility), With<Ghost>>,
) {
    let (mut tf, mut sprite, mut vis) = ghost.into_inner();
    let world = &sim.world;
    let ids = buildable(&world.content);
    let (true, Some(c), Some(body), false) = (ui.build, ui.cursor, world.player_body(), ids.is_empty()) else {
        *vis = Visibility::Hidden;
        return;
    };
    let id = ids[ui.build_idx % ids.len()];
    let info = &world.content.blocks[id as usize];
    let cell = body.local_cell_at(c);
    let free = body.block_at(cell).is_none();
    let unlocked = world.progress.unlocked.contains(&id);
    *vis = Visibility::Inherited;
    tf.translation = px(body.to_world(cell.center())).extend(8.0);
    tf.rotation = Quat::from_rotation_z(body.angle + ui.rot as f32 * FRAC_PI_2);
    sprite.color = if free && unlocked { Color::srgba(0.6, 1.0, 0.6, 0.6) } else { Color::srgba(1.0, 0.3, 0.3, 0.6) };
    if let Some(a) = sprite.texture_atlas.as_mut() {
        a.index = (info.def.sprite.tile as usize).min(TILE_COUNT - 1);
    }
}

pub fn follow_camera(
    sim: Res<Sim>,
    ui: Res<UiState>,
    mut views: ResMut<Views>,
    events: Res<FrameEvents>,
    cam: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let (mut tf, mut proj) = cam.into_inner();
    let shift: Vec2 = events
        .0
        .iter()
        .filter_map(|e| if let SimEvent::OriginShift { delta } = e { Some(px(*delta)) } else { None })
        .sum();
    let target = sim.world.player_body().map(|b| px(b.pos)).unwrap_or(tf.translation.truncate() - shift);
    tf.translation = target.extend(tf.translation.z);
    views.last_cam = target;
    if let Projection::Orthographic(o) = proj.as_mut() {
        o.scale = ui.zoom;
    }
}
