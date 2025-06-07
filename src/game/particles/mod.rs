//! Core game logic for our game

use bevy::prelude::*;
use bevy_hanabi::{
    AccelModifier, Attribute, ColorOverLifetimeModifier, EffectAsset, Gradient, Module,
    ParticleEffect, SetAttributeModifier, SetPositionSphereModifier, SetVelocitySphereModifier,
    ShapeDimension, SpawnerSettings,
};

use crate::game::particles::assets::activate_effect;

pub(crate) mod assets;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spore_activation_effect);
}

fn spore_activation_effect(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {

}
