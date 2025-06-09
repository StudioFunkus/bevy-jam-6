//! Tile types and terrain system for the play field

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<TileType>();
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
            TileType::Fertile => 1.25,
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
            TileType::Fertile => 1.0,
            TileType::BlockedRock | TileType::BlockedWater | TileType::BlockedMoss => 0.0,
        }
    }
}
