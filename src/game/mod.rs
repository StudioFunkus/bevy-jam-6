//! Core game logic for our game

use bevy::prelude::*;

mod grid;
pub(crate) mod level;
mod mushrooms;
mod resources;
mod ui;
mod visual_effects;
mod event_queue;
pub(crate) mod fixed_timestep;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        fixed_timestep::plugin,
        resources::plugin,
        grid::plugin,
        mushrooms::plugin,
        visual_effects::plugin,
        level::plugin,
        ui::plugin,
    ));
}