//! Mmushroom definitions

use bevy::prelude::*;
use std::collections::HashMap;

use crate::game::play_field::TileType;

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

    /// Sideways (East / West)
    pub const SIDEWAYS: &[GridOffset] = &[
        GridOffset::new(1, 0),  // East
        GridOffset::new(-1, 0), // West
    ];

    /// Fork (NE/NW)
    pub const FORK: &[GridOffset] = &[
        GridOffset::new(1, 1),  // NE
        GridOffset::new(-1, 1), // NW
    ];

    /// Threeway in a T shape (N/E/W)
    pub const THREEWAY: &[GridOffset] = &[
        GridOffset::new(0, 1),  // N
        GridOffset::new(1, 0),  // E
        GridOffset::new(-1, 0), // W
    ];

    /// Diagonal line (NE/SW)
    pub const DIAGONALLINE: &[GridOffset] = &[
        GridOffset::new(1, 1),   // NE
        GridOffset::new(-1, -1), // SW
    ];

    /// Single direction
    pub const FORWARD: &[GridOffset] = &[
        GridOffset::new(0, 1), // Default facing up
    ];

    /// Single direction, skip 1 tile
    pub const SKIP_FORWARD: &[GridOffset] = &[
        GridOffset::new(0, 2), // Default facing up
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
    /// Activation behavior - systems perform action based on this
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
    /// Boosts incoming energy before forwarding
    Amplifier {
        /// Multiplication factor for energy
        boost_factor: f32,
    },
    /// Modifies terrain at end of turn
    Converter {
        /// What tile type to convert to
        convert_to: TileType,
        /// Number of tiles to convert per turn
        convert_count: u32,
        /// What tiles can be converted (None = any non-target tile)
        can_convert_from: Option<Vec<TileType>>,
        /// Search radius (how far to look for convertible tiles)
        search_radius: i32,
    },
    /// Deletes a mushroom in the connected square
    Deleter,
}

/// Requirements to unlock a mushroom type
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    #[allow(dead_code)]
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
    Sideways,
    Fork,
    Threeway,
    Diagonal,
    Surround,
    Skipper,
    Deleter,
    Bomb,
    Amplifier,
    TwoWayAmplifier,
    ThreeWayAmplifier,
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
            name: "Button".to_string(),
            description: "10 Spores.".to_string(),
            base_production: 10.0,
            cooldown_time: 0.1,
            max_uses_per_turn: 5,
            sprite_row: 8,
            activation_behavior: ActivationBehavior::Basic,
            unlock_requirement: UnlockRequirement::None,
            connection_points: vec![],
        },
    );

    // Pulse Mushroom - single forward connection
    defs.insert(
        MushroomType::Pulse,
        MushroomDefinition {
            name: "Pulcini".to_string(),
            description: "5 Spores.\nConnect 1.".to_string(),
            base_production: 5.0,
            cooldown_time: 0.1,
            max_uses_per_turn: 3,
            sprite_row: 6,
            activation_behavior: ActivationBehavior::Basic,
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::FORWARD.to_vec(),
        },
    );

    // Sideways Mushroom - two connections, sideways
    defs.insert(
        MushroomType::Sideways,
        MushroomDefinition {
            name: "Dicholoma".to_string(),
            description: "2 Spores.\nConnect 2.".to_string(),
            base_production: 2.0,
            cooldown_time: 0.5,
            max_uses_per_turn: 3,
            sprite_row: 19,
            activation_behavior: ActivationBehavior::Amplifier { boost_factor: 2.0 },
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::SIDEWAYS.to_vec(),
        },
    );

    // Fork mushroom - two connections, forwards
    defs.insert(
        MushroomType::Fork,
        MushroomDefinition {
            name: "Forchione".to_string(),
            description: "2 Spores.\nConnect 2.".to_string(),
            base_production: 2.0,
            cooldown_time: 1.0,
            max_uses_per_turn: 2,
            sprite_row: 9,
            activation_behavior: ActivationBehavior::Amplifier { boost_factor: 2.0 },
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::FORK.to_vec(),
        },
    );

    // Wizard's Cap - Activates opposite diagonals
    defs.insert(
        MushroomType::Diagonal,
        MushroomDefinition {
            name: "Wizard's Cap".to_string(),
            description: "5 Spores.\nConnect 2.".to_string(),
            base_production: 5.0,
            cooldown_time: 1.0,
            max_uses_per_turn: 2,
            sprite_row: 0,
            activation_behavior: ActivationBehavior::Amplifier { boost_factor: 2.0 },
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::DIAGONALLINE.to_vec(),
        },
    );

    // T Mushroom - three connections in a T shape
    defs.insert(
        MushroomType::Threeway,
        MushroomDefinition {
            name: "Spliitake".to_string(),
            description: "8 Spores.\nConnect 3.".to_string(),
            base_production: 8.0,
            cooldown_time: 2.0,
            max_uses_per_turn: 2,
            sprite_row: 7,
            activation_behavior: ActivationBehavior::Amplifier { boost_factor: 3.0 },
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::THREEWAY.to_vec(),
        },
    );

    // Umberella - all surrounding mushrooms
    defs.insert(
        MushroomType::Surround,
        MushroomDefinition {
            name: "Umberella".to_string(),
            description: "4 Spores.\nConnect 8.".to_string(),
            base_production: 4.0,
            cooldown_time: 1.0,
            max_uses_per_turn: 2,
            sprite_row: 1,
            activation_behavior: ActivationBehavior::Amplifier { boost_factor: 8.0 },
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::ALL_DIRECTIONS.to_vec(),
        },
    );

    // Skipper - skips one tile and activates the next
    defs.insert(
        MushroomType::Skipper,
        MushroomDefinition {
            name: "Portini".to_string(),
            description: "8 Spores.\nConnect 1.".to_string(),
            base_production: 8.0,
            cooldown_time: 1.0,
            max_uses_per_turn: 2,
            sprite_row: 2,
            activation_behavior: ActivationBehavior::Basic,
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::SKIP_FORWARD.to_vec(),
        },
    );

    // Deleter - deletes a mushroom
    defs.insert(
        MushroomType::Deleter,
        MushroomDefinition {
            name: "Delita".to_string(),
            description: "Destroy connected.\n100 Spores for each.".to_string(),
            base_production: 100.0,
            cooldown_time: 10.0,
            max_uses_per_turn: 1,
            sprite_row: 3,
            activation_behavior: ActivationBehavior::Deleter,
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::FORWARD.to_vec(),
        },
    );

    // Bomb - deletes 4 mushrooms
    defs.insert(
        MushroomType::Bomb,
        MushroomDefinition {
            name: "Skullcap".to_string(),
            description: "Destroy connected.\n100 Spores for each.".to_string(),
            base_production: 100.0,
            cooldown_time: 10.0,
            max_uses_per_turn: 1,
            sprite_row: 4,
            activation_behavior: ActivationBehavior::Deleter,
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::DIAGONAL.to_vec(),
        },
    );

    // Burst Mushroom - no connections
    defs.insert(
        MushroomType::Burst,
        MushroomDefinition {
            name: "Puffball".to_string(),
            description: "25 Spores.".to_string(),
            base_production: 25.0,
            cooldown_time: 0.2,
            max_uses_per_turn: 3,
            sprite_row: 5,
            activation_behavior: ActivationBehavior::Basic,
            unlock_requirement: UnlockRequirement::None,
            connection_points: vec![], // No connections
        },
    );

    // Amplifier Mushroom - connects to a single cardinal directions
    defs.insert(
        MushroomType::Amplifier,
        MushroomDefinition {
            name: "Amplicus".to_string(),
            description: "3 Spores.\nConnect 1.\nEnergy Boost 2.".to_string(),
            base_production: 1.0,
            cooldown_time: 1.5,
            max_uses_per_turn: 1,
            sprite_row: 10,
            activation_behavior: ActivationBehavior::Amplifier { boost_factor: 1.5 },
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::FORWARD.to_vec(),
        },
    );

    // Four Way Amplifier Mushroom - connects to all cardinal directions
    defs.insert(
        MushroomType::TwoWayAmplifier,
        MushroomDefinition {
            name: "Enoki".to_string(),
            description: "2 Spores.\nConnect 2.\nEnergy Boost 1.".to_string(),
            base_production: 2.0,
            cooldown_time: 1.5,
            max_uses_per_turn: 1,
            sprite_row: 16,
            activation_behavior: ActivationBehavior::Amplifier { boost_factor: 2.5 },
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::SIDEWAYS.to_vec(),
        },
    );

    // Splitter Mushroom - connects to cardinal directions (can be configured for diagonals)
    defs.insert(
        MushroomType::ThreeWayAmplifier,
        MushroomDefinition {
            name: "Ink Cap".to_string(),
            description: "1 Spore.\nConnect 3.\nEnergy Boost 1.".to_string(),
            base_production: 1.0,
            cooldown_time: 3.0,
            max_uses_per_turn: 1,
            sprite_row: 17,
            activation_behavior: ActivationBehavior::Amplifier { boost_factor: 3.75 },
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::THREEWAY.to_vec(),
        },
    );

    // Chain Mushroom - single forward connection
    defs.insert(
        MushroomType::Chain,
        MushroomDefinition {
            name: "Mumbling Truffle".to_string(),
            description: "5 Spores.\nConnect 1.\nRapid Fire.".to_string(),
            base_production: 5.0,
            cooldown_time: 0.01,
            max_uses_per_turn: 15,
            sprite_row: 14,
            activation_behavior: ActivationBehavior::Basic,
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::FORWARD.to_vec(),
        },
    );

    // Converter Mushroom
    defs.insert(
        MushroomType::Converter,
        MushroomDefinition {
            name: "False Broccoli".to_string(),
            description: "8 Spores.\nConnect 1.\nFertilise 1.".to_string(),
            base_production: 8.0,
            cooldown_time: 0.3,
            max_uses_per_turn: 3,
            sprite_row: 18,
            activation_behavior: ActivationBehavior::Converter {
                convert_to: TileType::Fertile,
                convert_count: 1,
                can_convert_from: Some(vec![TileType::Empty]), // Only converts empty soil
                search_radius: 2,                              // Searches within 2 tiles
            },
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::FORWARD.to_vec(),
        },
    );

    // Knight Mushroom - L-shaped connection like chess knight
    defs.insert(
        MushroomType::Knight,
        MushroomDefinition {
            name: "Unicorn's Mane".to_string(),
            description: "15 Spores.\nConnect 1.".to_string(),
            base_production: 10.0,
            cooldown_time: 0.2,
            max_uses_per_turn: 3,
            sprite_row: 12,
            activation_behavior: ActivationBehavior::Basic,
            unlock_requirement: UnlockRequirement::None,
            connection_points: connection_patterns::KNIGHT_FORWARD.to_vec(),
        },
    );

    definitions.definitions = defs;
    info!(
        "Initialized {} mushroom definitions",
        definitions.definitions.len()
    );
}
