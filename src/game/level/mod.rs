//! Game level management

use bevy::prelude::*;

pub(crate) mod assets;
pub(crate) mod definitions;
pub(crate) mod spawning;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((assets::plugin, spawning::plugin, definitions::plugin));
}
