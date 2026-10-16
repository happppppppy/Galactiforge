//! Procedural test scenarios (benchmarks, stress tests).

use crate::body::{Body, BodyKind, Faction};
use crate::math::{Cell, Vec2};
use crate::world::{World, fill_storage};

/// A large rectangular warship of roughly `(2*half+1)^2` blocks: a core in
/// the middle, fuel tanks on pipe rows feeding a rear thruster bank, cannons
/// on a lattice each beside a shell container, and an armour rim.
pub fn battleship(world: &World, half: i32, faction: Option<Faction>, pos: Vec2, angle: f32) -> Body {
    let c = world.content.clone();
    let id = |n: &str| c.block(n).unwrap();
    let mut b = Body::new(0, "Battleship", BodyKind::Ship, faction, pos, angle);
    for y in -half..=half {
        for x in -half..=half {
            let edge = x.abs() == half || y.abs() == half;
            let def = if x == 0 && y == 0 {
                id("ship_core")
            } else if y == -half {
                id("large_thruster")
            } else if edge {
                id("armor_full")
            } else if x.rem_euclid(6) == 0 && y.rem_euclid(6) == 3 {
                id("cannon")
            } else if x.rem_euclid(6) == 1 && y.rem_euclid(6) == 3 {
                id("shell_container")
            } else if y.rem_euclid(3) == 0 {
                id("pipe")
            } else if x.rem_euclid(4) == 0 {
                id("fuel_container")
            } else if x.rem_euclid(4) == 2 {
                id("cable")
            } else {
                id("armor_full")
            };
            let _ = b.insert(&c, def, Cell::new(x, y), 0);
        }
    }
    b.refresh(&c);
    b.pos = pos;
    let n = b.block_count() as f32;
    fill_storage(&c, &mut b, c.res("fuel").unwrap(), n * 3.0);
    fill_storage(&c, &mut b, c.res("shells").unwrap(), n * 2.0);
    b
}
