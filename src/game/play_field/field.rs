//! The structure that represents our play field
//! It contains a spatial index mapping grid positions to entities

use crate::game::mushrooms::{MushroomDirection, MushroomType};
use bevy::{platform::collections::HashMap, prelude::*};

use super::GridPosition;

/// The size of each cell in the grid
pub const CELL_SIZE: f32 = 1.0;
pub const CELL_SPACING: f32 = 0.0;

/// Mycelium connection data for rendering and gameplay
#[derive(Debug, Clone)]
pub struct Connection {
    pub from_pos: GridPosition,
    pub to_pos: GridPosition,
    pub from_entity: Entity,
    pub to_entity: Entity,
    pub strength: f32,
    pub active: bool, // Currently pulsing with energy
    pub path: Vec<GridPosition>, // Path the mycelium takes
}

/// The play field containing the spatial index and bounds
#[derive(Debug, Default)]
pub struct PlayField {
    /// Spatial index mapping positions to entities (Mushrooms, maybe others later?)
    pub entities: HashMap<GridPosition, Entity>,
    /// Direct connections between positions
    pub connections: Vec<Connection>,
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
            connections: Vec::new(),
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

    /// Resize the field dimensions
    /// Note: This only changes the bounds. Entities outside the new bounds
    /// should be despawned separately, which will trigger observers to update the spatial index
    pub fn resize(&mut self, new_width: i32, new_height: i32) {
        self.width = new_width;
        self.height = new_height;
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

    /// Clear all connections
    pub fn clear_connections(&mut self) {
        info!("Clearing all connections");
        self.connections.clear();
    }

    /// Get all connections for rendering
    pub fn get_all_connections(&self) -> &[Connection] {
        &self.connections
    }
    
    /// Add a mycelium connection between two mushrooms
    pub fn add_connection(
        &mut self,
        from_pos: GridPosition,
        to_pos: GridPosition,
        from_entity: Entity,
        to_entity: Entity,
        strength: f32,
        path: Vec<GridPosition>,
    ) {
        self.connections.push(Connection {
            from_pos,
            to_pos,
            from_entity,
            to_entity,
            strength,
            active: false,
            path,
        });
        info!(
            "Added mycelium connection from {:?} to {:?} with strength {}",
            from_pos, to_pos, strength
        );
    }
}
