//! Menu asset loading

use bevy::prelude::*;

use crate::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<ScreenAssets>();
    app.load_resource::<ScreenAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ScreenAssets {
    #[dependency]
    pub music: Handle<AudioSource>,
}

impl FromWorld for ScreenAssets {
    fn from_world(world: &mut World) -> Self {
        let music_handle = world
            .resource::<AssetServer>()
            .load("audio/music/title.ogg");

        Self {
            music: music_handle,
        }
    }
}
