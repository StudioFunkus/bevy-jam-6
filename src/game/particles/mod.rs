//! Core game logic for our game

use bevy::prelude::*;
use bevy_hanabi::EffectAsset;


pub(crate) mod assets;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spore_activation_effect);
}

fn spore_activation_effect(commands: Commands, effects: ResMut<Assets<EffectAsset>>) {}
