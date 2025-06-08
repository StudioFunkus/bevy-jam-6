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
    #[dependency]
    pub wizard: Handle<Image>,
    #[dependency]
    pub witch: Handle<Image>,
    #[dependency]
    pub titlescreen: Handle<Image>,
     #[dependency]
    pub gametitle: Handle<Image>,
     #[dependency]
    pub spore1: Handle<Image>,
     #[dependency]
    pub spore2: Handle<Image>,
     #[dependency]
    pub spore3: Handle<Image>,
}

impl FromWorld for ScreenAssets {
    fn from_world(world: &mut World) -> Self {
        let music_handle = world
            .resource::<AssetServer>()
            .load("audio/music/title.ogg");
        let image_handle = world
            .resource::<AssetServer>()
            .load("images/Wizard1b.png");
        let image_handle_witch = world
            .resource::<AssetServer>()
            .load("images/Witch2.png");
        let image_handle_titlescreen = world
            .resource::<AssetServer>()
            .load("images/Psychocybin.png");
        let image_handle_gametitle = world
            .resource::<AssetServer>()
            .load("images/title.png");
        let image_handle_spore1 = world
            .resource::<AssetServer>()
            .load("images/spore1.png");
        let image_handle_spore2 = world
            .resource::<AssetServer>()
            .load("images/spore2.png");
        let image_handle_spore3 = world
            .resource::<AssetServer>()
            .load("images/spore3.png");
        Self {
            music: music_handle,
            wizard: image_handle,
            witch: image_handle_witch,
            titlescreen: image_handle_titlescreen,
            gametitle: image_handle_gametitle,
            spore1: image_handle_spore1,
            spore2: image_handle_spore2,
            spore3: image_handle_spore3,
        }
    }
}
