//! Core game logic for our game

use bevy::prelude::*;
use bevy_hanabi::EffectAsset;

pub(crate) mod assets;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spore_activation_effect);
}

#[allow(dead_code)]
fn spore_activation_effect(_commands: Commands, _effects: ResMut<Assets<EffectAsset>>) {}
