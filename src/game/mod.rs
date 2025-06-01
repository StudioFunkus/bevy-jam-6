//! Core game logic for our game

use bevy::prelude::*;

pub(crate) mod level;
mod grid;
mod mushrooms;
mod ui;
mod resources;
mod visual_effects;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        resources::plugin,
        grid::plugin,
        mushrooms::plugin,
        visual_effects::plugin,
        level::plugin,
        ui::plugin,
    ));
}