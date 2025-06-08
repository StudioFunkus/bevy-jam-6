//! Game level management

use bevy::prelude::*;

pub(crate) mod assets;
pub(crate) mod definitions;
pub(crate) mod spawning;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((assets::plugin, spawning::plugin, definitions::plugin));

    // Add music tracking resource
    app.init_resource::<CurrentGameplayMusic>();
}

/// Tracks the currently playing gameplay music
#[derive(Resource, Default)]
pub struct CurrentGameplayMusic {
    pub current_track: Option<Handle<AudioSource>>,
    pub entity: Option<Entity>,
}
