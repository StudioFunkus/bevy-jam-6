//! Core game resources and state

use bevy::{platform::collections::HashMap, prelude::*};
use crate::game::play_field::{GridPosition, PlayField};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GameState>();
    app.init_resource::<UnlockedMushrooms>();
    app.add_systems(Update, check_unlocks);
}

/// Core game state tracking spores and progression
#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct GameState {
    /// Current number of spores
    pub spores: f64,
    /// Total spores earned all time
    pub total_spores_earned: f64,
    /// Number of activations this session
    pub total_activations: u64,
    /// Number of chain activations (mushroom activated by another mushroom)
    pub chain_activations: u64,
    /// The play field containing the spatial index and bounds
    #[reflect(ignore)]
    pub play_field: PlayField,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            spores: 25.0,
            total_spores_earned: 25.0,
            total_activations: 0,
            chain_activations: 0,
            play_field: PlayField::new(6, 6),
        }
    }
}

impl GameState {
    pub fn add_spores(&mut self, amount: f64) {
        self.spores += amount;
        self.total_spores_earned += amount;
    }

    pub fn spend_spores(&mut self, amount: f64) -> bool {
        if self.spores >= amount {
            self.spores -= amount;
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub fn record_activation(&mut self, is_chain: bool) {
        self.total_activations += 1;
        if is_chain {
            self.chain_activations += 1;
        }
    }
}

/// Mushroom unlock status
#[derive(Resource, Default, Reflect, Debug)]
#[reflect(Resource)]
pub struct UnlockedMushrooms {
    pub button: bool,
    pub pulse: bool,
}

/// Check and update mushroom unlocks
#[tracing::instrument(name = "Check unlocks", skip_all)]
fn check_unlocks(game_state: Res<GameState>, mut unlocked: ResMut<UnlockedMushrooms>) {
    if !unlocked.button {
        unlocked.button = true;
        info!("Unlocked Button Mushroom!");
    }

    // Unlock based on total spores earned
    if !unlocked.pulse && game_state.total_spores_earned >= 25.0 {
        unlocked.pulse = true;
        info!("Unlocked Pulse Mushroom!");
    }
}
