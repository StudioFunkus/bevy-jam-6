//! Core game logic for our game

use bevy::prelude::*;

pub(crate) mod fixed_timestep;
pub(crate) mod game_flow;
pub(crate) mod level;
mod mushrooms;
pub(crate) mod play_field;
mod resources;
mod ui;
mod visual_effects;

pub(super) fn plugin(app: &mut App) {

    app.add_systems(Update, despawn_timer_system);

    app.add_plugins((
        fixed_timestep::plugin,
        resources::plugin,
        play_field::plugin,
        mushrooms::plugin,
        visual_effects::plugin,
        level::plugin,
        game_flow::plugin,
        ui::plugin,
    ));
}

/// Component that despawns an entity after a specified duration
#[derive(Component)]
pub struct DespawnTimer {
    /// Time remaining until despawn
    pub timer: Timer,
}

impl DespawnTimer {
    /// Creates a new DespawnTimer that will despawn the entity after the specified seconds
    pub fn new(seconds: f32) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, TimerMode::Once),
        }
    }
}

/// System that updates despawn timers and removes entities when their time is up
pub fn despawn_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DespawnTimer)>,
) {
    for (entity, mut despawn_timer) in &mut query {
        // Tick the timer with the time that has passed since the last frame
        despawn_timer.timer.tick(time.delta());
        
        // If the timer has finished, despawn the entity
        if despawn_timer.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
