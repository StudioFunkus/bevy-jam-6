use bevy::prelude::*;

use crate::game::grid::GridPosition;

use super::{MushroomDirection, MushroomType, trigger::TriggerSource};

pub(super) fn plugin(app: &mut App) {
    app.add_event::<SpawnMushroomEvent>();
    app.add_event::<RemoveMushroomEvent>();
    app.add_event::<TriggerMushroomEvent>();
}

/// Event to spawn a mushroom
#[derive(Event)]
pub struct SpawnMushroomEvent {
    pub position: GridPosition,
    pub mushroom_type: MushroomType,
    pub entity: Entity,
}

/// Event to trigger a mushroom's effect
#[derive(Event, Clone)]
pub struct TriggerMushroomEvent {
    pub position: GridPosition,
    pub source: TriggerSource,
    pub energy: f64,
    pub direction: Option<MushroomDirection>,
}

/// Event to remove a mushroom
#[derive(Event)]
pub struct RemoveMushroomEvent {
    pub position: GridPosition,
}
