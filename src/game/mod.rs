//! Core game logic for our game

use bevy::prelude::*;

pub(crate) mod fixed_timestep;
pub(crate) mod game_flow;
pub(crate) mod level;
mod mushrooms;
pub(crate) mod play_field;
mod resources;
mod ui;
mod visual_effects;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        fixed_timestep::plugin,
        resources::plugin,
        play_field::plugin,
        mushrooms::plugin,
        visual_effects::plugin,
        level::plugin,
        game_flow::plugin,
        ui::plugin,
    ));
}
