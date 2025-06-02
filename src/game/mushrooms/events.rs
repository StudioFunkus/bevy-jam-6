use bevy::prelude::*;

use crate::game::{grid::GridPosition, mushrooms::ActivationSource};

use super::{MushroomDirection, MushroomType};

pub(super) fn plugin(app: &mut App) {
    app.add_event::<SpawnMushroomEvent>();
    app.add_event::<RemoveMushroomEvent>();
    app.add_event::<ActivateMushroomEvent>();
}

/// Event to spawn a mushroom
#[derive(Event)]
pub struct SpawnMushroomEvent {
    pub position: GridPosition,
    pub mushroom_type: MushroomType,
}

/// Event to trigger a mushroom's effect
#[derive(Event, Clone, Default)]
pub struct ActivateMushroomEvent {
    pub position: GridPosition,
    pub source: ActivationSource,
    pub energy: f64,
}

/// Event to remove a mushroom
#[derive(Event)]
pub struct RemoveMushroomEvent {
    pub position: GridPosition,
}
