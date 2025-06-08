//! Level-specific asset loading

use bevy::prelude::*;

use crate::asset_tracking::LoadResource;

pub(crate) fn plugin(app: &mut App) {
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
    #[dependency]
    pub tile_texture: Handle<Image>,
    #[dependency]
    pub background_model_1: Handle<Scene>,
    #[dependency]
    pub sfx_activate: Handle<AudioSource>,
    #[dependency]
    pub card_common: Handle<Image>,
    #[dependency]
    pub card_uncommon: Handle<Image>,
    #[dependency]
    pub card_rare: Handle<Image>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let music_handle = world
            .resource::<AssetServer>()
            .load("audio/music/level1.ogg");
        let sfx_activate_handle = world
            .resource::<AssetServer>()
            .load("audio/sound_effects/activate.ogg");
        let mushroom_texture_handle = world
            .resource::<AssetServer>()
            .load("textures/mushrooms.png");
        let tile_texture_handle = world.resource::<AssetServer>().load("textures/tiles.png");
        let background_model_1_handle = world
            .resource::<AssetServer>()
            .load("models/background1.gltf#Scene0");

        let card_common = world
            .resource::<AssetServer>()
            .load("images/Card_Common.png");
        let card_uncommon = world
            .resource::<AssetServer>()
            .load("images/Card_Uncommon.png");
        let card_rare = world.resource::<AssetServer>().load("images/Card_Rare.png");

        Self {
            music: music_handle,
            sfx_activate: sfx_activate_handle,
            mushroom_texture: mushroom_texture_handle,
            tile_texture: tile_texture_handle,
            background_model_1: background_model_1_handle,
            card_common: card_common,
            card_uncommon: card_uncommon,
            card_rare: card_rare,
        }
    }
}
