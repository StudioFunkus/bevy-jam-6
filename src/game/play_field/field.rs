//! The structure that represents our play field
//! It contains a spatial index mapping grid positions to entities

use bevy::{platform::collections::HashMap, prelude::*};

use super::{GridPosition, TileType};

/// The size of each cell in the grid
pub const CELL_SIZE: f32 = 1.0;
pub const CELL_SPACING: f32 = 0.0;

/// Mycelium connection data for rendering and gameplay
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Connection {
    pub from_pos: GridPosition,
    pub to_pos: GridPosition,
    pub from_entity: Entity,
    pub to_entity: Entity,
    pub strength: f32,
    pub active: bool,            // Currently pulsing with energy
    pub path: Vec<GridPosition>, // Path the mycelium takes
}

/// The play field containing the spatial index and bounds
#[derive(Debug)]
pub struct PlayField {
    /// Spatial index mapping positions to entities (Mushrooms, maybe others later?)
    pub entities: HashMap<GridPosition, Entity>,
    /// Direct connections between positions
    pub connections: Vec<Connection>,
    /// Tile types for each position
    pub tiles: Vec<TileType>,
    /// Width of the field
    pub width: i32,
    /// Height of the field
    pub height: i32,
}

impl Default for PlayField {
    fn default() -> Self {
        Self::new(6, 6)
    }
}

impl PlayField {
    /// Create a new play field with the given dimensions
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            entities: HashMap::default(),
            connections: Vec::new(),
            tiles: vec![TileType::Empty; (width * height) as usize],
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

    /// Get the tile type at a position
    pub fn get_tile(&self, pos: GridPosition) -> Option<TileType> {
        if self.contains(pos) {
            let index = self.pos_to_index(pos);
            self.tiles.get(index).copied()
        } else {
            None
        }
    }

    /// Set the tile type at a position
    pub fn set_tile(&mut self, pos: GridPosition, tile_type: TileType) {
        if self.contains(pos) {
            let index = self.pos_to_index(pos);
            self.tiles[index] = tile_type;
        }
    }

    /// Convert grid position to array index
    fn pos_to_index(&self, pos: GridPosition) -> usize {
        pos.y as usize * self.width as usize + pos.x as usize
    }

    /// Set tiles from a level definition
    pub fn set_tiles_from_level(&mut self, level_tiles: &[(GridPosition, TileType)]) {
        // Clear all tiles to Empty first
        self.tiles.fill(TileType::Empty);

        // Then set the specific tiles for this level
        for (pos, tile_type) in level_tiles {
            self.set_tile(*pos, *tile_type);
        }
    }

    /// Resize the field dimensions
    /// Note: This only changes the bounds. Entities outside the new bounds
    /// should be despawned separately, which will trigger observers to update the spatial index
    pub fn resize(&mut self, new_width: i32, new_height: i32) {
        let mut new_tiles = vec![TileType::Empty; (new_width * new_height) as usize];

        // Copy existing tiles that fit in new dimensions
        for y in 0..self.height.min(new_height) {
            for x in 0..self.width.min(new_width) {
                let old_index = (y * self.width + x) as usize;
                let new_index = (y * new_width + x) as usize;
                new_tiles[new_index] = self.tiles[old_index];
            }
        }

        self.tiles = new_tiles;
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
