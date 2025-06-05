//! Level definitions and configuration

use bevy::prelude::*;

use crate::game::{mushrooms::MushroomType, resources::GameState};

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
    // TODO: Add more level-specific data?
    // pub tile_types: Vec<(GridPosition, TileType)>,
    // pub blockers: Vec<GridPosition>,
    // pub rich_soil_positions: Vec<GridPosition>,
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
                // Level 1 - Small grid with low score requirement, example of starting mushrooms
                LevelDefinition {
                    name: "Level 1".to_string(),
                    grid_width: 6,
                    grid_height: 6,
                    target_score: 50.0,
                    max_turns: 3,
                    starting_mushrooms: vec![
                        StartingMushroom {
                            x: 2,
                            y: 2,
                            mushroom_type: MushroomType::Basic,
                        },
                        StartingMushroom {
                            x: 3,
                            y: 3,
                            mushroom_type: MushroomType::Pulse,
                        },
                    ],
                },
                // Level 2 - Chain intro?
                LevelDefinition {
                    name: "Level 2".to_string(),
                    grid_width: 8,
                    grid_height: 8,
                    target_score: 100.0,
                    max_turns: 4,
                    starting_mushrooms: vec![
                        // TODO: Pre-place some mushrooms to teach chainin?
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
                },
                // Level 4 - Vertical?
                LevelDefinition {
                    name: "Level 4".to_string(),
                    grid_width: 6,
                    grid_height: 12,
                    target_score: 1000.0,
                    max_turns: 5,
                    starting_mushrooms: vec![],
                },
                // Level 5 - Horizontal?
                LevelDefinition {
                    name: "Level 5".to_string(),
                    grid_width: 12,
                    grid_height: 6,
                    target_score: 5000.0,
                    max_turns: 5,
                    starting_mushrooms: vec![],
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
        game_state.play_field.resize(level_def.grid_width, level_def.grid_height);

        Some(level_def.clone())
    } else {
        None
    }
}
