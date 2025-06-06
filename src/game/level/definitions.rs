//! Level definitions and configuration

use bevy::prelude::*;

use crate::game::{
    mushrooms::MushroomType,
    play_field::{GridPosition, TileType},
    resources::GameState,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<LevelDefinitions>();
}

/// A single level's configuration
#[derive(Debug, Clone)]
pub struct LevelDefinition {
    pub name: String,
    pub grid_width: i32,
    pub grid_height: i32,
    pub target_score: f64,
    pub max_turns: u32,
    pub starting_mushrooms: Vec<StartingMushroom>,
    pub tile_configuration: Vec<(GridPosition, TileType)>,
}

/// Mushrooms that are pre-placed on the level
#[derive(Debug, Clone)]
pub struct StartingMushroom {
    pub x: i32,
    pub y: i32,
    pub mushroom_type: MushroomType,
}

impl Default for LevelDefinition {
    fn default() -> Self {
        Self {
            name: "Level".to_string(),
            grid_width: 8,
            grid_height: 8,
            target_score: 500.0,
            max_turns: 3,
            starting_mushrooms: vec![],
            tile_configuration: vec![],
        }
    }
}

/// Collection of all level definitions
#[derive(Resource)]
pub struct LevelDefinitions {
    pub levels: Vec<LevelDefinition>,
}

impl LevelDefinitions {
    /// Get a level by index, returns None if out of bounds
    pub fn get_level(&self, index: usize) -> Option<&LevelDefinition> {
        self.levels.get(index)
    }
}

impl Default for LevelDefinitions {
    fn default() -> Self {
        Self {
            levels: vec![
                // Level 1 - Larger demo grid with variety of tile types
                LevelDefinition {
                    name: "Level 1".to_string(),
                    grid_width: 12,
                    grid_height: 12,
                    target_score: 100.0,
                    max_turns: 5,
                    starting_mushrooms: vec![StartingMushroom {
                        x: 2,
                        y: 2,
                        mushroom_type: MushroomType::Pulse,
                    }],
                    tile_configuration: vec![
                        // Fertile patches
                        (GridPosition::new(1, 1), TileType::Fertile),
                        (GridPosition::new(2, 1), TileType::Fertile),
                        (GridPosition::new(1, 2), TileType::Fertile),
                        (GridPosition::new(9, 7), TileType::Fertile),
                        (GridPosition::new(10, 7), TileType::Fertile),
                        (GridPosition::new(10, 8), TileType::Fertile),
                        // Rock formation (demonstrates corners and edges)
                        (GridPosition::new(7, 1), TileType::BlockedRock),
                        (GridPosition::new(8, 1), TileType::BlockedRock),
                        (GridPosition::new(9, 1), TileType::BlockedRock),
                        (GridPosition::new(7, 2), TileType::BlockedRock),
                        (GridPosition::new(8, 2), TileType::BlockedRock),
                        (GridPosition::new(9, 2), TileType::BlockedRock),
                        (GridPosition::new(8, 3), TileType::BlockedRock),
                        // Water obstacles (single tiles)
                        (GridPosition::new(3, 6), TileType::BlockedWater),
                        (GridPosition::new(4, 7), TileType::BlockedWater),
                        (GridPosition::new(3, 8), TileType::BlockedWater),
                        // Moss obstacles (single tiles)
                        (GridPosition::new(0, 5), TileType::BlockedMoss),
                        (GridPosition::new(11, 4), TileType::BlockedMoss),
                        // Single rock blockers
                        (GridPosition::new(5, 8), TileType::BlockedRock),
                        (GridPosition::new(2, 5), TileType::BlockedRock),
                    ],
                },
                // Level 2 - Chain intro?
                LevelDefinition {
                    name: "Level 2".to_string(),
                    grid_width: 8,
                    grid_height: 8,
                    target_score: 100.0,
                    max_turns: 4,
                    starting_mushrooms: vec![],
                    tile_configuration: vec![
                        // Create interesting terrain
                        (GridPosition::new(0, 0), TileType::Fertile),
                        (GridPosition::new(7, 7), TileType::Fertile),
                        // Rocky cross pattern
                        (GridPosition::new(4, 3), TileType::BlockedRock),
                        (GridPosition::new(4, 4), TileType::BlockedRock),
                        (GridPosition::new(3, 4), TileType::BlockedRock),
                        (GridPosition::new(5, 4), TileType::BlockedRock),
                    ],
                },
                // Level 3 - Larger grid
                LevelDefinition {
                    name: "Level 3".to_string(),
                    grid_width: 10,
                    grid_height: 10,
                    target_score: 200.0,
                    max_turns: 6,
                    starting_mushrooms: vec![],
                    tile_configuration: vec![],
                },
                // Level 4 - Vertical?
                LevelDefinition {
                    name: "Level 4".to_string(),
                    grid_width: 6,
                    grid_height: 12,
                    target_score: 1000.0,
                    max_turns: 5,
                    starting_mushrooms: vec![],
                    tile_configuration: vec![
                        // Vertical barriers
                        (GridPosition::new(2, 0), TileType::BlockedRock),
                        (GridPosition::new(2, 1), TileType::BlockedRock),
                        (GridPosition::new(2, 10), TileType::BlockedRock),
                        (GridPosition::new(2, 11), TileType::BlockedRock),
                        (GridPosition::new(3, 0), TileType::BlockedRock),
                        (GridPosition::new(3, 1), TileType::BlockedRock),
                        (GridPosition::new(3, 10), TileType::BlockedRock),
                        (GridPosition::new(3, 11), TileType::BlockedRock),
                    ],
                },
                // Level 5 - Horizontal?
                LevelDefinition {
                    name: "Level 5".to_string(),
                    grid_width: 12,
                    grid_height: 6,
                    target_score: 5000.0,
                    max_turns: 5,
                    starting_mushrooms: vec![],
                    tile_configuration: vec![
                        // Horizontal barriers
                        (GridPosition::new(0, 2), TileType::BlockedRock),
                        (GridPosition::new(1, 2), TileType::BlockedRock),
                        (GridPosition::new(10, 2), TileType::BlockedRock),
                        (GridPosition::new(11, 2), TileType::BlockedRock),
                        (GridPosition::new(0, 3), TileType::BlockedRock),
                        (GridPosition::new(1, 3), TileType::BlockedRock),
                        (GridPosition::new(10, 3), TileType::BlockedRock),
                        (GridPosition::new(11, 3), TileType::BlockedRock),
                    ],
                },
            ],
        }
    }
}

/// Load a specific level's configuration
pub fn load_level_config(
    level_index: usize,
    definitions: &LevelDefinitions,
    game_state: &mut GameState,
) -> Option<LevelDefinition> {
    if let Some(level_def) = definitions.get_level(level_index) {
        // Resize the play field
        game_state
            .play_field
            .resize(level_def.grid_width, level_def.grid_height);

        Some(level_def.clone())
    } else {
        None
    }
}
