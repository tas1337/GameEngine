// Spatial partitioning for efficient queries
// Only send entities that are nearby (spatial awareness)

use dashmap::DashMap;
use uuid::Uuid;
use crate::protocol::Vec3;
use crate::player::Player;
use crate::world::World;

/// Spatial hash grid (divide world into cells)
pub struct SpatialHash {
    cell_size: f32,
    cells: DashMap<(i32, i32, i32), Vec<Uuid>>,
}

impl SpatialHash {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: DashMap::new(),
        }
    }
    
    /// Convert world position to grid cell
    fn to_cell(&self, pos: &Vec3) -> (i32, i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
            (pos.z / self.cell_size).floor() as i32,
        )
    }
    
    /// Update spatial hash (called every tick)
    pub fn update(&self, players: &DashMap<Uuid, Player>, _world: &World) {
        // Clear old data
        self.cells.clear();
        
        // Insert all players into grid
        for entry in players.iter() {
            let cell = self.to_cell(&entry.value().position);
            self.cells.entry(cell)
                .or_insert_with(Vec::new)
                .push(*entry.key());
        }
    }
    
    /// Get nearby players (for spatial awareness)
    pub fn get_nearby_players(&self, position: &Vec3, radius: f32) -> Vec<Uuid> {
        let mut nearby = Vec::new();
        
        let cell = self.to_cell(position);
        let range = (radius / self.cell_size).ceil() as i32;
        
        // Check surrounding cells
        for dx in -range..=range {
            for dy in -range..=range {
                for dz in -range..=range {
                    let check_cell = (cell.0 + dx, cell.1 + dy, cell.2 + dz);
                    if let Some(players) = self.cells.get(&check_cell) {
                        nearby.extend(players.iter());
                    }
                }
            }
        }
        
        nearby
    }
}

