//! Core game logic for our game

use bevy::prelude::*;

pub(crate) mod level;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((level::plugin,));
}
