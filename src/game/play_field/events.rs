//! Play field events

use super::GridPosition;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_event::<GridClickEvent>();
}

/// Component for grid cells that can be clicked
#[derive(Component)]
pub struct GridCell {
    pub position: GridPosition,
}

/// Event fired when a grid cell is clicked
#[derive(Event, Debug)]
pub struct GridClickEvent {
    pub position: GridPosition,
    pub button: bevy::picking::pointer::PointerButton,
}

/// Converts low-level pointer clicks on grid cells into high-level GridClickEvents.
///
/// This observer bridges Bevy's picking system with the grid system, allowing
/// game-specific logic to respond to grid interactions without coupling to UI details.
pub fn on_grid_cell_click(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    grid_cells: Query<&GridCell>,
) {
    if let Ok(cell) = grid_cells.get(trigger.target()) {
        info!("Grid cell clicked at position: {:?}", cell.position);
        info!("Triggering observers of GridClickEvent");
        commands.trigger(GridClickEvent {
            position: cell.position,
            button: trigger.event().button,
        });
    }
}
