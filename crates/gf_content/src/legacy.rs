//! Import ship layouts from the legacy Lua/LÖVE prototype
//! (`code/src/configs/ships.json` and `ship_save.json`).
//!
//! Legacy coordinates are screen-space (y down, negative y = ship front) and
//! directions are clockwise degrees. The new simulation uses a y-up local
//! frame (+y = ship forward) with counter-clockwise quarter-turn rotations.

use crate::{Blueprint, BlueprintBlock};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
struct LegacyCell {
    #[serde(rename = "type")]
    ty: String,
    x: i32,
    y: i32,
    #[serde(default)]
    direction: i32,
    #[serde(default)]
    active_resource: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LegacyShip {
    starter_grid: Vec<LegacyCell>,
}

/// Convert legacy clockwise degrees to CCW quarter turns.
pub fn legacy_rot(direction_deg: i32) -> u8 {
    let q = (direction_deg.rem_euclid(360) / 90) as u8;
    (4 - q) % 4
}

fn convert(id: &str, cells: Vec<LegacyCell>) -> Blueprint {
    Blueprint {
        id: id.to_string(),
        name: id.replace('_', " "),
        blocks: cells
            .into_iter()
            .map(|c| BlueprintBlock {
                block: c.ty,
                x: c.x,
                y: -c.y,
                rot: legacy_rot(c.direction),
                recipe: c.active_resource.map(|r| format!("make_{r}")),
            })
            .collect(),
        stock: Vec::new(),
    }
}

/// Parse legacy `ships.json` (`{ name: { starter_grid: [...] } }`).
pub fn import_ships_json(text: &str) -> Result<Vec<Blueprint>, String> {
    let ships: BTreeMap<String, LegacyShip> =
        serde_json::from_str(text).map_err(|e| e.to_string())?;
    Ok(ships
        .into_iter()
        .map(|(k, v)| convert(&k, v.starter_grid))
        .collect())
}

/// Parse legacy `ship_save.json` (`{ name: [ ...cells ] }`).
pub fn import_ship_save(text: &str) -> Result<Vec<Blueprint>, String> {
    let ships: BTreeMap<String, Vec<LegacyCell>> =
        serde_json::from_str(text).map_err(|e| e.to_string())?;
    Ok(ships.into_iter().map(|(k, v)| convert(&k, v)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_mapping() {
        assert_eq!(legacy_rot(0), 0);
        assert_eq!(legacy_rot(90), 3);
        assert_eq!(legacy_rot(180), 2);
        assert_eq!(legacy_rot(270), 1);
    }

    #[test]
    fn imports_legacy_ships() {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../code/src/configs/ships.json"
        ))
        .unwrap();
        let bps = import_ships_json(&text).unwrap();
        let mb = bps.iter().find(|b| b.id == "micro_bandit").unwrap();
        // Rear large thruster at legacy y=2 faces forward -> new y=-2, rot 0.
        assert!(
            mb.blocks
                .iter()
                .any(|b| b.block == "large_thruster" && b.y == -2 && b.rot == 0)
        );
    }

    #[test]
    fn imports_legacy_save() {
        let text =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../ship_save.json"))
                .unwrap();
        let bps = import_ship_save(&text).unwrap();
        assert!(!bps.is_empty());
        assert!(bps[0].blocks.len() > 10);
    }
}
