//! Spawn the game level

use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, audio::music, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
        }
    }
}

/// Spawn the main game level
pub fn spawn_level(mut commands: Commands, level_assets: Res<LevelAssets>) {
    // Spawn background music
    commands.spawn((
        Name::new("Gameplay Music"),
        StateScoped(Screen::Gameplay),
        music(level_assets.music.clone()),
    ));
}
