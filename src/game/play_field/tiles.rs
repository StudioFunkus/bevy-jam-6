//! Tile types and terrain system for the play field

use super::GridPosition;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<TileGrid>().register_type::<TileType>();
}

/// Different types of tiles that affect gameplay
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
#[reflect(Component)]
pub enum TileType {
    #[default]
    Empty,
    Fertile,
    BlockedRock,
    BlockedWater,
    BlockedMoss,
}

impl TileType {
    /// Get production multiplier for mushrooms on this tile
    pub fn production_multiplier(&self) -> f32 {
        match self {
            TileType::Empty => 1.0,
            TileType::Fertile => 1.5,
            TileType::BlockedRock | TileType::BlockedWater | TileType::BlockedMoss => 0.0,
        }
    }

    /// Can a mushroom be placed on this tile?
    pub fn allows_mushroom(&self) -> bool {
        !matches!(
            self,
            TileType::BlockedRock | TileType::BlockedWater | TileType::BlockedMoss
        )
    }

    /// Can mycelium grow through this tile?
    pub fn allows_mycelium(&self) -> bool {
        match self {
            TileType::Empty | TileType::Fertile | TileType::BlockedMoss => true,
            TileType::BlockedRock | TileType::BlockedWater => false,
        }
    }

    /// Modifier for mycelium strength on this tile
    pub fn mycelium_strength_modifier(&self) -> f32 {
        match self {
            TileType::Empty => 1.0,
            TileType::Fertile => 1.3,
            TileType::BlockedRock | TileType::BlockedWater | TileType::BlockedMoss => 0.0,
        }
    }
}

/// Grid of tile types for the entire field
#[derive(Resource, Default, Debug)]
pub struct TileGrid {
    tiles: Vec<TileType>,
    width: usize,
    height: usize,
}

impl TileGrid {
    /// Create a new tile grid with the given dimensions
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            tiles: vec![TileType::Empty; width * height],
            width,
            height,
        }
    }

    /// Get the tile type at a position
    pub fn get(&self, pos: GridPosition) -> Option<TileType> {
        if self.contains(pos) {
            let index = self.pos_to_index(pos);
            self.tiles.get(index).copied()
        } else {
            None
        }
    }

    /// Set the tile type at a position
    pub fn set(&mut self, pos: GridPosition, tile_type: TileType) {
        if self.contains(pos) {
            let index = self.pos_to_index(pos);
            self.tiles[index] = tile_type;
        }
    }

    /// Check if a position is within bounds
    pub fn contains(&self, pos: GridPosition) -> bool {
        pos.x >= 0 && pos.x < self.width as i32 && pos.y >= 0 && pos.y < self.height as i32
    }

    /// Convert grid position to array index
    fn pos_to_index(&self, pos: GridPosition) -> usize {
        pos.y as usize * self.width + pos.x as usize
    }

    /// Resize the grid, preserving existing tiles where possible
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        let mut new_tiles = vec![TileType::Empty; new_width * new_height];

        // Copy existing tiles that fit in new dimensions
        for y in 0..self.height.min(new_height) {
            for x in 0..self.width.min(new_width) {
                let old_index = y * self.width + x;
                let new_index = y * new_width + x;
                new_tiles[new_index] = self.tiles[old_index];
            }
        }

        self.tiles = new_tiles;
        self.width = new_width;
        self.height = new_height;
    }

    /// Set tiles from a level definition
    pub fn set_from_level(&mut self, level_tiles: &[(GridPosition, TileType)]) {
        // Clear all tiles to Empty first
        self.clear();

        // Then set the specific tiles for this level
        for (pos, tile_type) in level_tiles {
            self.set(*pos, *tile_type);
        }
    }

    /// Clear all tiles to Empty
    pub fn clear(&mut self) {
        self.tiles.fill(TileType::Empty);
    }

    /// Get width
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get height  
    pub fn height(&self) -> usize {
        self.height
    }
}
