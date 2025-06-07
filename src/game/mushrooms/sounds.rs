use bevy::prelude::*;

use crate::{
    audio::sound_effect,
    game::{level::assets::LevelAssets, mushrooms::events::SporeScoreEvent},
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(chain_activate_sfx);
}

pub fn chain_activate_sfx(
    _trigger: Trigger<SporeScoreEvent>,
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
) {
    commands.spawn(sound_effect(level_assets.sfx_activate.clone()));
}
