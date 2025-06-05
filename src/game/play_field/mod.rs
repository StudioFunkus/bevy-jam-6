//! Play field management with spatial indexing and lifecycle observers

use bevy::prelude::*;

mod field;
mod position;
pub mod events;
pub mod observers;

pub use field::{PlayField, CELL_SIZE, CELL_SPACING};
pub use position::GridPosition;
pub use events::{GridClickEvent, GridCell};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        position::plugin,
        events::plugin,
        observers::plugin,
    ));
}