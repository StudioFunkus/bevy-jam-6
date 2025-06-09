use bevy::prelude::*;
use crate::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<ThemeAssets>();
    app.load_resource::<ThemeAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ThemeAssets {
    #[dependency]
    pub slice_1: Handle<Image>,
    #[dependency]
    pub slice_2: Handle<Image>,
}

impl FromWorld for ThemeAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        
        Self {
            slice_1: asset_server.load("images/ui_slices/slice1.png"),
            slice_2: asset_server.load("images/ui_slices/slice2.png"),
        }
    }
}
