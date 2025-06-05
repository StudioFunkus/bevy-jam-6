//! The structure that represents our play field
//! It contains a spatial index mapping grid positions to entities

use bevy::{platform::collections::HashMap, prelude::*};
use super::GridPosition;

/// The size of each cell in the grid
pub const CELL_SIZE: f32 = 1.0;
pub const CELL_SPACING: f32 = 0.0;

/// The play field containing the spatial index and bounds
#[derive(Debug, Default)]
pub struct PlayField {
    /// Spatial index mapping positions to entities
    pub entities: HashMap<GridPosition, Entity>,
    /// Width of the field
    pub width: i32,
    /// Height of the field
    pub height: i32,
}

impl PlayField {
    /// Create a new play field with the given dimensions
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            entities: HashMap::default(),
            width,
            height,
        }
    }

    /// Check if a position is within bounds
    pub fn contains(&self, position: GridPosition) -> bool {
        position.x >= 0 && position.x < self.width && position.y >= 0 && position.y < self.height
    }

    /// Get the entity at a position
    pub fn get(&self, position: GridPosition) -> Option<Entity> {
        self.entities.get(&position).copied()
    }

    /// Insert an entity at a position
    pub fn insert(&mut self, position: GridPosition, entity: Entity) -> Option<Entity> {
        self.entities.insert(position, entity)
    }

    /// Remove an entity from a position
    pub fn remove(&mut self, position: GridPosition) -> Option<Entity> {
        self.entities.remove(&position)
    }

    /// Resize the field (preserves entities within new bounds)
    pub fn resize(&mut self, new_width: i32, new_height: i32) {
        self.width = new_width;
        self.height = new_height;
        
        // Remove entities outside new bounds
        self.entities.retain(|pos, _| {
            pos.x >= 0 && pos.x < new_width && pos.y >= 0 && pos.y < new_height
        });
    }

    /// Get total cell size including spacing
    pub fn total_cell_size() -> f32 {
        CELL_SIZE + CELL_SPACING
    }

    /// Get the world dimensions of the grid
    pub fn world_size(&self) -> (f32, f32) {
        let total_size = Self::total_cell_size();
        (
            self.width as f32 * total_size,
            self.height as f32 * total_size,
        )
    }
}