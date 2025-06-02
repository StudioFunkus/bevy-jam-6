//! Core game resources and state

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GameState>();
    app.init_resource::<UnlockedMushrooms>();
    app.add_systems(Update, check_unlocks);
}

/// Core game state tracking spores and progression
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct GameState {
    /// Current number of spores
    pub spores: f64,
    /// Total spores earned all time
    pub total_spores_earned: f64,
    /// Number of triggers this session
    pub total_triggers: u64,
    /// Number of chain triggers (mushroom triggered by another mushroom)
    pub chain_triggers: u64,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            spores: 25.0,
            total_spores_earned: 25.0,
            total_triggers: 0,
            chain_triggers: 0,
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

    pub fn record_trigger(&mut self, is_chain: bool) {
        self.total_triggers += 1;
        if is_chain {
            self.chain_triggers += 1;
        }
    }
}

/// Mushroom unlock status
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct UnlockedMushrooms {
    pub button: bool,
    pub pulse: bool,
}

/// Check and update mushroom unlocks
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
