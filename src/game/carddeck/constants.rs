use bevy::{
    math::{Vec2, Vec3},
    render::view::RenderLayers,
};

pub const CARD_SPACING: f32 = 5.0;
pub const TRANSLATION_TWEEN_DURATION: f32 = 0.1;
pub const SCALE_TWEEN_DURATION: f32 = 0.25;
pub const CARD_LAYER: RenderLayers = RenderLayers::layer(2);
pub const CARD_SIZE: Vec2 = Vec2::new(140.0, 190.0);
pub const CARD_IN_PLAY_POSITION: Vec3 = Vec3::new(400.0, 360.0, 0.0);
pub const HAND_SIZE_LIMIT: usize = 7;
