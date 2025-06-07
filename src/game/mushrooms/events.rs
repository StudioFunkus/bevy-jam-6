//! Mushroom-related events

use super::{MushroomDirection, MushroomType};
use crate::game::play_field::GridPosition;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_event::<SpawnMushroomEvent>();
    app.add_event::<RemoveMushroomEvent>();
}

/// Event to spawn a mushroom
#[derive(Event)]
pub struct SpawnMushroomEvent {
    pub position: GridPosition,
    pub mushroom_type: MushroomType,
    pub direction: Option<MushroomDirection>,
}

/// Event to remove a mushroom
#[derive(Event)]
#[allow(dead_code)]
pub struct RemoveMushroomEvent {
    pub position: GridPosition,
}

// Spawn score event
#[derive(Event)]
pub struct SporeScoreEvent {
    pub position: GridPosition,
    pub production: f64,
}
