//! Level-specific asset loading

use bevy::prelude::*;

use crate::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    pub music: Handle<AudioSource>,
    #[dependency]
    pub mushroom_texture: Handle<Image>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let music_handle = world
            .resource::<AssetServer>()
            .load("audio/music/Fluffing A Duck.ogg");
        let mushroom_texture_handle = world
            .resource::<AssetServer>()
            .load("textures/mushrooms.png");

        Self {
            music: music_handle,
            mushroom_texture: mushroom_texture_handle,
        }
    }
}
