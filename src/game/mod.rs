//! Core game logic for our game

use bevy::prelude::*;

pub(crate) mod level;
mod grid;
mod mushrooms;
mod ui;
mod resources;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        resources::plugin,
        grid::plugin,
        mushrooms::plugin,
        level::plugin,
        ui::plugin,
    ));
}