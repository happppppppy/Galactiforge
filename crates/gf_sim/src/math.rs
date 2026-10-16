//! Deterministic 2D math. Transcendental functions go through `libm` so results
//! are bit-identical across platforms (required for lockstep multiplayer).

use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

pub const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn dot(self, o: Vec2) -> f32 {
        self.x * o.x + self.y * o.y
    }
    /// Scalar 2D cross product (z of the 3D cross).
    pub fn cross(self, o: Vec2) -> f32 {
        self.x * o.y - self.y * o.x
    }
    pub fn length_sq(self) -> f32 {
        self.dot(self)
    }
    pub fn length(self) -> f32 {
        libm::sqrtf(self.length_sq())
    }
    pub fn normalize_or_zero(self) -> Vec2 {
        let l = self.length();
        if l > 1e-6 { self * (1.0 / l) } else { Vec2::ZERO }
    }
    pub fn rotate(self, angle: f32) -> Vec2 {
        let (s, c) = (libm::sinf(angle), libm::cosf(angle));
        self.rotate_sc(s, c)
    }
    pub fn rotate_sc(self, s: f32, c: f32) -> Vec2 {
        Vec2::new(self.x * c - self.y * s, self.x * s + self.y * c)
    }
    /// Perpendicular (rotated +90 degrees).
    pub fn perp(self) -> Vec2 {
        Vec2::new(-self.y, self.x)
    }
    pub fn angle(self) -> f32 {
        libm::atan2f(self.y, self.x)
    }
    pub fn from_angle(a: f32) -> Vec2 {
        Vec2::new(libm::cosf(a), libm::sinf(a))
    }
    pub fn distance(self, o: Vec2) -> f32 {
        (self - o).length()
    }
    pub fn clamp_length(self, max: f32) -> Vec2 {
        let l = self.length();
        if l > max { self * (max / l) } else { self }
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }
}
impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }
}
impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, s: f32) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }
}
impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}
impl AddAssign for Vec2 {
    fn add_assign(&mut self, o: Vec2) {
        self.x += o.x;
        self.y += o.y;
    }
}
impl SubAssign for Vec2 {
    fn sub_assign(&mut self, o: Vec2) {
        self.x -= o.x;
        self.y -= o.y;
    }
}

/// Integer grid cell coordinate in a body's local frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
}

impl Cell {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    pub fn offset(self, d: (i32, i32)) -> Cell {
        Cell::new(self.x + d.0, self.y + d.1)
    }
    pub fn center(self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }
    pub const NEIGHBORS: [(i32, i32); 4] = [(0, 1), (-1, 0), (0, -1), (1, 0)];
}

/// Unit facing for a rotation (CCW quarter turns, rot 0 = +y / ship forward).
pub fn facing(rot: u8) -> (i32, i32) {
    match rot & 3 {
        0 => (0, 1),
        1 => (-1, 0),
        2 => (0, -1),
        _ => (1, 0),
    }
}

pub fn facing_vec(rot: u8) -> Vec2 {
    let (x, y) = facing(rot);
    Vec2::new(x as f32, y as f32)
}

/// Rotate an integer offset CCW by `rot` quarter turns.
pub fn rotate_offset(d: (i32, i32), rot: u8) -> (i32, i32) {
    match rot & 3 {
        0 => d,
        1 => (-d.1, d.0),
        2 => (-d.0, -d.1),
        _ => (d.1, -d.0),
    }
}

/// Cells covered by a footprint of `size` (width, depth) anchored at `anchor`.
/// In the local (unrotated) frame the footprint grows towards +x and -y
/// (i.e. sideways and backwards from the block's facing).
pub fn footprint(anchor: Cell, rot: u8, size: (u8, u8)) -> impl Iterator<Item = Cell> {
    let (w, h) = (size.0 as i32, size.1 as i32);
    (0..h).flat_map(move |j| (0..w).map(move |i| anchor.offset(rotate_offset((i, -j), rot))))
}

/// Wrap an angle to (-PI, PI].
pub fn wrap_angle(a: f32) -> f32 {
    let mut a = a % (2.0 * PI);
    if a > PI {
        a -= 2.0 * PI;
    } else if a <= -PI {
        a += 2.0 * PI;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footprint_rotations() {
        let a = Cell::new(0, 0);
        let r0: Vec<_> = footprint(a, 0, (2, 2)).collect();
        assert!(r0.contains(&Cell::new(1, -1)));
        let r1: Vec<_> = footprint(a, 1, (2, 2)).collect();
        assert!(r1.contains(&Cell::new(1, 1)));
        let r2: Vec<_> = footprint(a, 2, (2, 2)).collect();
        assert!(r2.contains(&Cell::new(-1, 1)));
        let r3: Vec<_> = footprint(a, 3, (2, 2)).collect();
        assert!(r3.contains(&Cell::new(-1, -1)));
    }

    #[test]
    fn wrap() {
        assert!((wrap_angle(3.0 * PI) - PI).abs() < 1e-5);
        assert!((wrap_angle(-1.5 * PI) - 0.5 * PI).abs() < 1e-5);
    }
}
