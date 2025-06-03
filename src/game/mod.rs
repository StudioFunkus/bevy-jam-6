//! Core game logic for our game

use bevy::prelude::*;

mod event_queue;
pub(crate) mod fixed_timestep;
pub(crate) mod grid;
pub(crate) mod level;
mod mushrooms;
mod resources;
pub(crate) mod turn_manager;
mod ui;
mod visual_effects;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        fixed_timestep::plugin,
        resources::plugin,
        grid::plugin,
        mushrooms::plugin,
        visual_effects::plugin,
        level::plugin,
        turn_manager::plugin,
        ui::plugin,
    ));
}
