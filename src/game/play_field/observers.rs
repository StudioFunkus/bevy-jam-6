//! Play field lifecycle observers

use super::GridPosition;
use crate::game::resources::GameState;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_add_grid_position);
    app.add_observer(on_remove_grid_position);
}

/// Automatically add entities to the play field when GridPosition is added
#[tracing::instrument(name = "Add entity to play field", skip_all)]
fn on_add_grid_position(
    trigger: Trigger<OnAdd, GridPosition>,
    query: Query<&GridPosition>,
    mut game_state: ResMut<GameState>,
) {
    let entity = trigger.target();
    if let Ok(position) = query.get(entity) {
        // Check if position is within bounds
        if !game_state.play_field.contains(*position) {
            warn!(
                "Position {:?} is out of bounds ({}x{} field)",
                position, game_state.play_field.width, game_state.play_field.height
            );
            return;
        }

        // Check if position is already occupied
        if let Some(existing) = game_state.play_field.get(*position) {
            warn!(
                "Position {:?} already occupied by {:?}, replacing with {:?}",
                position, existing, entity
            );
        }
        game_state.play_field.insert(*position, entity);
        info!("Added entity {:?} to play field at {:?}", entity, position);
    }
}

/// Automatically remove entities from the play field when GridPosition is removed or entity is despawned
#[tracing::instrument(name = "Remove entity from play field", skip_all)]
fn on_remove_grid_position(
    trigger: Trigger<OnRemove, GridPosition>,
    query: Query<&GridPosition>,
    mut game_state: ResMut<GameState>,
) {
    let entity = trigger.target();
    if let Ok(position) = query.get(entity) {
        if let Some(removed) = game_state.play_field.remove(*position) {
            if removed != entity {
                warn!(
                    "Removed entity {:?} from position {:?}, but it was occupied by {:?}",
                    entity, position, removed
                );
            } else {
                info!(
                    "Removed entity {:?} from play field at {:?}",
                    entity, position
                );
            }
        }
    }
}

/// Helper function to find entities at a specific position
pub fn find_entity_at(position: GridPosition, game_state: &GameState) -> Option<Entity> {
    game_state.play_field.get(position)
}
