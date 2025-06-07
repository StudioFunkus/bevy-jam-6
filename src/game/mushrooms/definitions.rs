//! Mmushroom definitions

use bevy::prelude::*;
use std::collections::HashMap;

/// Relative position offset for connections
#[derive(Debug, Clone, Copy)]
pub struct GridOffset {
    pub x: i32,
    pub y: i32,
}

impl GridOffset {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Common connection patterns
pub mod connection_patterns {
    use super::GridOffset;

    /// Cardinal directions (N, E, S, W)
    pub const CARDINAL: &[GridOffset] = &[
        GridOffset::new(0, 1),  // North
        GridOffset::new(1, 0),  // East
        GridOffset::new(0, -1), // South
        GridOffset::new(-1, 0), // West
    ];

    /// Diagonal directions
    pub const DIAGONAL: &[GridOffset] = &[
        GridOffset::new(1, 1),   // NE
        GridOffset::new(1, -1),  // SE
        GridOffset::new(-1, -1), // SW
        GridOffset::new(-1, 1),  // NW
    ];

    /// All 8 directions
    pub const ALL_DIRECTIONS: &[GridOffset] = &[
        GridOffset::new(0, 1),   // N
        GridOffset::new(1, 1),   // NE
        GridOffset::new(1, 0),   // E
        GridOffset::new(1, -1),  // SE
        GridOffset::new(0, -1),  // S
        GridOffset::new(-1, -1), // SW
        GridOffset::new(-1, 0),  // W
        GridOffset::new(-1, 1),  // NW
    ];

    /// Single direction
    pub const FORWARD: &[GridOffset] = &[
        GridOffset::new(0, 1), // Default facing up
    ];

    /// Single knight move - L-shaped like chess knight
    pub const KNIGHT_FORWARD: &[GridOffset] = &[
        GridOffset::new(2, 1), // 2 right, 1 up (at default orientation)
    ];
}

/// Complete definition of a mushroom type with all its properties
#[derive(Debug, Clone)]
pub struct MushroomDefinition {
    /// Display name
    pub name: String,
    /// Description for UI
    pub description: String,
    /// Base spore production when activated
    pub base_production: f64,
    /// Cooldown time in seconds
    pub cooldown_time: f32,
    /// Maximum activations per turn
    pub max_uses_per_turn: u32,
    /// Row in the sprite sheet for this mushroom
    pub sprite_row: usize,
    /// Activation behavior
    pub activation_behavior: ActivationBehavior,
    /// Unlock requirements
    pub unlock_requirement: UnlockRequirement,
    /// Connection points relative to this mushroom
    pub connection_points: Vec<GridOffset>,
}

/// Defines how a mushroom behaves when activated
#[derive(Debug, Clone, Reflect)]
pub enum ActivationBehavior {
    /// Just produces spores, no propagation
    Basic,
    /// Sends energy in a single direction
    Directional {
        /// How many tiles the pulse can travel
        range: u32,
    },
    /// Boosts incoming energy before forwarding
    Amplifier {
        /// Multiplication factor for energy
        boost_factor: f32,
    },
    /// Splits energy to all adjacent tiles
    Splitter {
        /// Whether to include diagonal tiles
        include_diagonals: bool,
    },
    /// Optimized for long chains with minimal decay
    Chain {
        /// Bonus energy preservation per hop
        chain_bonus: f32,
    },
    /// High threshold but massive production
    Burst {
        /// Production multiplier when activated
        burst_multiplier: f32,
    },
    /// Modifies terrain and forwards energy
    Converter {
        /// What tile type to convert adjacent tiles to
        convert_to: super::super::play_field::TileType,
    },
}

/// Requirements to unlock a mushroom type
#[derive(Debug, Clone)]
pub enum UnlockRequirement {
    /// Always unlocked
    None,
    /// Requires total spores earned
    TotalSpores(f64),
    /// Requires reaching a specific level
    ReachLevel(usize),
    /// Requires a certain number of chain activations
    ChainActivations(u64),
    /// Multiple requirements (all must be met)
    All(Vec<UnlockRequirement>),
    /// Multiple requirements (any can be met)
    Any(Vec<UnlockRequirement>),
}

impl UnlockRequirement {
    /// Check if this requirement is met
    pub fn is_met(
        &self,
        game_state: &crate::game::resources::GameState,
        current_level: usize,
    ) -> bool {
        match self {
            UnlockRequirement::None => true,
            UnlockRequirement::TotalSpores(required) => game_state.total_spores_earned >= *required,
            UnlockRequirement::ReachLevel(level) => current_level >= *level,
            UnlockRequirement::ChainActivations(required) => {
                game_state.chain_activations >= *required
            }
            UnlockRequirement::All(reqs) => {
                reqs.iter().all(|req| req.is_met(game_state, current_level))
            }
            UnlockRequirement::Any(reqs) => {
                reqs.iter().any(|req| req.is_met(game_state, current_level))
            }
        }
    }
}

/// Resource containing all mushroom definitions
#[derive(Resource, Default)]
pub struct MushroomDefinitions {
    definitions: HashMap<MushroomType, MushroomDefinition>,
}

impl MushroomDefinitions {
    /// Get a mushroom definition
    pub fn get(&self, mushroom_type: MushroomType) -> Option<&MushroomDefinition> {
        self.definitions.get(&mushroom_type)
    }

    /// Get all mushroom types
    pub fn all_types(&self) -> Vec<MushroomType> {
        self.definitions.keys().copied().collect()
    }

    /// Check if a mushroom type is unlocked
    pub fn is_unlocked(
        &self,
        mushroom_type: MushroomType,
        game_state: &crate::game::resources::GameState,
        current_level: usize,
    ) -> bool {
        self.definitions
            .get(&mushroom_type)
            .map(|def| def.unlock_requirement.is_met(game_state, current_level))
            .unwrap_or(false)
    }
}

/// All mushroom types in the game
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
#[reflect(Component)]
pub enum MushroomType {
    #[default]
    Basic,
    Pulse,
    Amplifier,
    Splitter,
    Chain,
    Burst,
    Converter,
    Knight,
    Test,
}

/// Plugin to initialize mushroom definitions
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<MushroomDefinitions>()
        .add_systems(Startup, initialize_definitions);
}

/// Initialize all mushroom definitions at startup
fn initialize_definitions(mut definitions: ResMut<MushroomDefinitions>) {
    let mut defs = HashMap::new();

    // Basic Mushroom - no connections
    defs.insert(
        MushroomType::Basic,
        MushroomDefinition {
            name: "Button Mushroom".to_string(),
            description: "Produces spores when activated.".to_string(),
            base_production: 10.0,
            cooldown_time: 0.1,
            max_uses_per_turn: 5,
            sprite_row: 0,
            activation_behavior: ActivationBehavior::Basic,
            unlock_requirement: UnlockRequirement::None,
            connection_points: vec![],
        },
    );

    // Pulse Mushroom - single forward connection
    defs.insert(
        MushroomType::Pulse,
        MushroomDefinition {
            name: "Pulse Mushroom".to_string(),
            description: "Sends energy pulses to trigger adjacent mushrooms in one direction."
                .to_string(),
            base_production: 2.0,
            cooldown_time: 0.1,
            max_uses_per_turn: 2,
            sprite_row: 1,
            activation_behavior: ActivationBehavior::Directional { range: 1 },
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::FORWARD.to_vec(),
        },
    );

    // Amplifier Mushroom - connects to all cardinal directions
    defs.insert(
        MushroomType::Amplifier,
        MushroomDefinition {
            name: "Amplifier Mushroom".to_string(),
            description: "Boosts energy by 50% and forwards to all adjacent mushrooms.".to_string(),
            base_production: 1.0,
            cooldown_time: 1.5,
            max_uses_per_turn: 1,
            sprite_row: 2,
            activation_behavior: ActivationBehavior::Amplifier { boost_factor: 1.5 },
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::CARDINAL.to_vec(),
        },
    );

    // Splitter Mushroom - connects to cardinal directions (can be configured for diagonals)
    defs.insert(
        MushroomType::Splitter,
        MushroomDefinition {
            name: "Splitter Mushroom".to_string(),
            description:
                "Splits energy equally to all adjacent mushrooms, creating branching chains."
                    .to_string(),
            base_production: 3.0,
            cooldown_time: 3.0,
            max_uses_per_turn: 2,
            sprite_row: 3,
            activation_behavior: ActivationBehavior::Splitter {
                include_diagonals: false,
            },
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::CARDINAL.to_vec(),
        },
    );

    // Chain Mushroom - single forward connection
    defs.insert(
        MushroomType::Chain,
        MushroomDefinition {
            name: "Chain Mushroom".to_string(),
            description: "Low activation threshold, optimized for creating long reaction chains."
                .to_string(),
            base_production: 5.0,
            cooldown_time: 0.8,
            max_uses_per_turn: 5,
            sprite_row: 4,
            activation_behavior: ActivationBehavior::Chain { chain_bonus: 0.02 },
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::FORWARD.to_vec(),
        },
    );

    // Burst Mushroom - no connections
    defs.insert(
        MushroomType::Burst,
        MushroomDefinition {
            name: "Burst Mushroom".to_string(),
            description: "High energy threshold, but produces massive spores when triggered."
                .to_string(),
            base_production: 50.0,
            cooldown_time: 5.0,
            max_uses_per_turn: 1,
            sprite_row: 5,
            activation_behavior: ActivationBehavior::Burst {
                burst_multiplier: 2.0,
            },
            unlock_requirement: UnlockRequirement::None,
            connection_points: vec![], // No connections
        },
    );

    // Converter Mushroom
    defs.insert(
        MushroomType::Converter,
        MushroomDefinition {
            name: "Converter Mushroom".to_string(),
            description: "Converts adjacent terrain to fertile soil while forwarding energy."
                .to_string(),
            base_production: 8.0,
            cooldown_time: 2.5,
            max_uses_per_turn: 3,
            sprite_row: 6,
            activation_behavior: ActivationBehavior::Converter {
                convert_to: crate::game::play_field::TileType::Fertile,
            },
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::FORWARD.to_vec(),
        },
    );

    // Knight Mushroom - L-shaped connection like chess knight
    defs.insert(
        MushroomType::Knight,
        MushroomDefinition {
            name: "Knight Mushroom".to_string(),
            description: "Jumps energy in an L-shape like a chess knight.".to_string(),
            base_production: 6.0,
            cooldown_time: 1.8,
            max_uses_per_turn: 3,
            sprite_row: 7,
            activation_behavior: ActivationBehavior::Directional { range: 3 }, // L-shaped is ~2.2 distance
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::KNIGHT_FORWARD.to_vec(),
        },
    );

    // Test Mushroom - Giga overpowered for testing
    defs.insert(
        MushroomType::Test,
        MushroomDefinition {
            name: "Giga Mushroom".to_string(),
            description: "Activates everything.".to_string(),
            base_production: 1000.0,
            cooldown_time: 0.1,
            max_uses_per_turn: 50,
            sprite_row: 7,
            activation_behavior: ActivationBehavior::Amplifier { boost_factor: 2.0 },
            unlock_requirement: UnlockRequirement::None,
            connection_points: vec![GridOffset::new(0, 1)],
        },
    );

    definitions.definitions = defs;
    info!(
        "Initialized {} mushroom definitions",
        definitions.definitions.len()
    );
}
