//! Fixed timestep configuration

use bevy::prelude::*;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<FixedTimestepConfig>();
    app.init_resource::<GameTime>();

    app.insert_resource(Time::<Fixed>::from_hz(30.0));

    // Add system to update fixed timestep from config changes
    app.add_systems(
        Update,
        update_fixed_timestep.run_if(resource_changed::<FixedTimestepConfig>),
    );

    // Update game time on fixed timestep
    app.add_systems(FixedFirst, update_game_time);
}

/// Configuration for the fixed timestep
///
/// The game runs its core logic at a fixed rate to ensure consistent behavior.
/// Players can adjust this rate through the settings menu.
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct FixedTimestepConfig {
    /// Target updates per second (Hz)
    pub target_hz: f64,
    /// Minimum allowed Hz
    pub min_hz: f64,
    /// Maximum allowed Hz
    pub max_hz: f64,
    /// Base Hz for game speed calculations
    pub base_hz: f64,
}

impl Default for FixedTimestepConfig {
    fn default() -> Self {
        Self {
            target_hz: 30.0,
            min_hz: 10.0,
            max_hz: 120.0,
            base_hz: 30.0,
        }
    }
}

impl FixedTimestepConfig {
    /// Set the target Hz, clamping to valid range
    pub fn _set_hz(&mut self, hz: f64) {
        self.target_hz = hz.clamp(self.min_hz, self.max_hz);
    }

    /// Get the game speed multiplier relative to base Hz
    pub fn speed_multiplier(&self) -> f64 {
        self.target_hz / self.base_hz
    }
}

/// Game time that scales with timestep frequency
#[derive(Resource, Default, Debug)]
pub struct GameTime {
    /// Scaled delta time for this frame
    pub delta_seconds: f32,
}

#[allow(dead_code)]
impl GameTime {
    /// Tick a timer using game time
    pub fn tick_timer(&self, timer: &mut Timer) {
        timer.tick(Duration::from_secs_f32(self.delta_seconds));
    }
}

/// Update game time based on fixed timestep and speed multiplier
#[tracing::instrument(name = "Update game time", skip_all)]
fn update_game_time(
    time: Res<Time<Fixed>>,
    config: Res<FixedTimestepConfig>,
    mut game_time: ResMut<GameTime>,
) {
    // Game time is real time scaled by speed multiplier
    // At 30 Hz (base), game_time.delta = real delta (1/30 sec)
    // At 60 Hz, game_time.delta = real delta * 2 (game runs 2x faster)
    // At 15 Hz, game_time.delta = real delta * 0.5 (game runs 0.5x slower)
    game_time.delta_seconds = time.delta_secs() * config.speed_multiplier() as f32;
}

/// Update the fixed timestep when configuration changes
#[tracing::instrument(name = "Update fixed timestep", skip_all)]
fn update_fixed_timestep(config: Res<FixedTimestepConfig>, mut fixed_time: ResMut<Time<Fixed>>) {
    fixed_time.set_timestep_hz(config.target_hz);
    info!(
        "Fixed timestep updated to {} Hz ({}x speed)",
        config.target_hz,
        config.speed_multiplier()
    );
}
