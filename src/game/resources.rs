//! Core game resources and state

use crate::game::play_field::PlayField;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GameState>();
}

/// Core game state tracking the game field, spores and progression
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
    /// The play field
    #[reflect(ignore)]
    pub play_field: PlayField,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            spores: 0.0,
            total_spores_earned: 0.0,
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

    #[allow(dead_code)]
    pub fn record_activation(&mut self, is_chain: bool) {
        self.total_activations += 1;
        if is_chain {
            self.chain_activations += 1;
        }
    }
}
