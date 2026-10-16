//! Chunked sparse grid mapping cells to block slots.

use crate::math::Cell;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const CHUNK: i32 = 16;
const CHUNK_CELLS: usize = (CHUNK * CHUNK) as usize;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Chunk {
    /// 0 = empty, otherwise block slot + 1.
    cells: Vec<u32>,
    count: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Grid {
    chunks: BTreeMap<(i32, i32), Chunk>,
    len: usize,
}

fn split(c: Cell) -> ((i32, i32), usize) {
    let cx = c.x.div_euclid(CHUNK);
    let cy = c.y.div_euclid(CHUNK);
    let lx = c.x.rem_euclid(CHUNK);
    let ly = c.y.rem_euclid(CHUNK);
    ((cx, cy), (ly * CHUNK + lx) as usize)
}

impl Grid {
    pub fn get(&self, c: Cell) -> Option<u32> {
        let (k, i) = split(c);
        let v = self.chunks.get(&k)?.cells[i];
        if v == 0 { None } else { Some(v - 1) }
    }
    pub fn is_empty_at(&self, c: Cell) -> bool {
        self.get(c).is_none()
    }
    pub fn set(&mut self, c: Cell, slot: Option<u32>) {
        let (k, i) = split(c);
        let new = slot.map_or(0, |s| s + 1);
        let chunk = self.chunks.entry(k).or_insert_with(|| Chunk {
            cells: vec![0; CHUNK_CELLS],
            count: 0,
        });
        let old = chunk.cells[i];
        chunk.cells[i] = new;
        match (old == 0, new == 0) {
            (true, false) => {
                chunk.count += 1;
                self.len += 1;
            }
            (false, true) => {
                chunk.count -= 1;
                self.len -= 1;
                if chunk.count == 0 {
                    self.chunks.remove(&k);
                }
            }
            _ => {}
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Occupied cells in deterministic (chunk, row-major) order.
    pub fn iter(&self) -> impl Iterator<Item = (Cell, u32)> + '_ {
        self.chunks.iter().flat_map(|(&(cx, cy), ch)| {
            ch.cells.iter().enumerate().filter(|(_, v)| **v != 0).map(move |(i, v)| {
                let i = i as i32;
                (Cell::new(cx * CHUNK + i % CHUNK, cy * CHUNK + i / CHUNK), *v - 1)
            })
        })
    }
    pub fn chunk_keys(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        self.chunks.keys().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn set_get_negative_coords() {
        let mut g = Grid::default();
        g.set(Cell::new(-1, -17), Some(4));
        g.set(Cell::new(3, 2), Some(0));
        assert_eq!(g.get(Cell::new(-1, -17)), Some(4));
        assert_eq!(g.get(Cell::new(3, 2)), Some(0));
        assert_eq!(g.len(), 2);
        g.set(Cell::new(3, 2), None);
        assert_eq!(g.len(), 1);
        assert_eq!(g.iter().count(), 1);
    }
}
