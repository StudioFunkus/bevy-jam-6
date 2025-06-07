//! Play field management with spatial indexing and lifecycle observers

use bevy::prelude::*;

pub mod events;
mod field;
pub mod field_renderer;
pub mod mycelium;
pub mod observers;
pub mod placement_preview;
mod position;
pub mod tile_atlas;
pub mod tiles;

pub use events::GridClickEvent;
pub use field::{CELL_SIZE, CELL_SPACING, PlayField};
pub use position::GridPosition;
pub use tiles::TileType;

use crate::game::resources::GameState;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        position::plugin,
        events::plugin,
        observers::plugin,
        tiles::plugin,
        mycelium::plugin,
        field_renderer::plugin,
        placement_preview::plugin,
    ));

    app.add_systems(
        OnEnter(crate::game::game_flow::LevelState::Playing),
        clear_play_field_connections,
    );
}

/// Clear all connections when entering a new level
fn clear_play_field_connections(mut game_state: ResMut<GameState>) {
    game_state.play_field.clear_connections();
}
