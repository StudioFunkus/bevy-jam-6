//! Core game logic for our game

use bevy::prelude::*;

pub(crate) mod level;
mod grid;
mod mushrooms;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        grid::plugin,
        level::plugin,
        mushrooms::plugin,
    ));
}