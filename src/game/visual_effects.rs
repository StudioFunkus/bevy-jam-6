//! Visual effects and feedback for game interactions

use crate::game::mushrooms::{Mushroom, MushroomDefinitions};
use bevy::prelude::*;
use bevy_sprite3d::Sprite3d;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            face_camera,
            update_mushroom_sprite_direction,
            update_activation_animations,
        ),
    );
}

/// Component for entities that should face the camera
#[derive(Component)]
pub struct FaceCamera;

/// Animation component for mushroom activation effects
#[derive(Component)]
pub struct ActivationAnimation {
    pub timer: Timer,
    pub original_scale: Vec3,
    pub peak_scale: Vec3,
}

impl ActivationAnimation {
    pub fn new(duration: f32, scale_multiplier: f32, original_scale: Vec3) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            original_scale,
            peak_scale: original_scale * scale_multiplier,
        }
    }
}

/// Make entities face the camera (billboard rotation)
fn face_camera(
    cam_transform: Query<&Transform, With<Camera>>,
    mut query: Query<&mut Transform, (With<FaceCamera>, Without<Camera>)>,
) {
    let Ok(camera_transform) = cam_transform.single() else {
        return;
    };

    for mut transform in query.iter_mut() {
        let mut target = camera_transform.translation;
        target.y = transform.translation.y;
        transform.look_at(target, Vec3::Y);
    }
}

/// Update mushroom sprite direction based on camera angle
fn update_mushroom_sprite_direction(
    cam_transform: Query<&Transform, With<Camera>>,
    definitions: Res<MushroomDefinitions>,
    mut mushrooms: Query<
        (&mut Transform, &mut Sprite3d, &Mushroom),
        (With<FaceCamera>, Without<Camera>),
    >,
) {
    let Ok(camera_transform) = cam_transform.single() else {
        return;
    };

    for (mut transform, mut sprite, mushroom) in mushrooms.iter_mut() {
        // Get camera position and adjust Y to match mushroom's height
        // This ensures we're only considering horizontal direction, not vertical
        let mut target = camera_transform.translation;
        target.y = transform.translation.y;

        // Calculate the direction vector from mushroom to camera
        // Example: If mushroom at (0,0,0) and camera at (3,0,4):
        // direction = (3,0,4) - (0,0,0) = (3,0,4)
        // After normalize: (0.6, 0, 0.8) - a unit vector pointing toward camera
        let direction = (target - transform.translation).normalize();

        // Determine which sprite to show based on viewing angle
        // We compare the absolute Z and X components to determine if we're
        // viewing more from front/back (Z axis) or from the sides (X axis)
        //
        // Top-down view of the mushroom and camera positions:
        //
        //                    Back
        //                (z < 0, flip)
        //                     |
        //                     |
        //    Left -------- Mushroom -------- Right
        //  (x < 0, flip)      M           (x > 0, no flip)
        //                     |
        //                     |
        //                   Front
        //               (z > 0, no flip)
        //
        let (sprite_index, should_flip) = if direction.z.abs() > direction.x.abs() {
            // Camera is more in front or behind the mushroom
            // Show front/back sprite (index 0), flip when behind (negative Z)
            (0, direction.z < 0.0)
        } else {
            // Camera is more to the side of the mushroom
            // Show side sprite (index 1), flip when on left (negative X)
            (1, direction.x < 0.0)
        };

        // Calculate texture atlas index based on mushroom type
        // The texture is organized in rows (mushroom types) and columns (view angles)
        // Get the sprite row from the definition
        if let Some(definition) = definitions.get(mushroom.0) {
            if let Some(ref mut atlas) = sprite.texture_atlas {
                atlas.index = definition.sprite_row * 2 + sprite_index;
            }
        }

        // Apply horizontal flip by negating the X scale
        transform.scale.x = transform.scale.x.abs() * if should_flip { -1.0 } else { 1.0 };
    }
}

/// Update activation animations for mushrooms
fn update_activation_animations(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut ActivationAnimation)>,
) {
    for (entity, mut transform, mut animation) in query.iter_mut() {
        animation.timer.tick(time.delta());

        if animation.timer.finished() {
            // Animation complete, restore original scale and remove component
            transform.scale = animation.original_scale;
            commands.entity(entity).remove::<ActivationAnimation>();
        } else {
            // Calculate animation progress (0.0 to 1.0)
            let progress =
                animation.timer.elapsed_secs() / animation.timer.duration().as_secs_f32();

            // Bounce
            let scale_factor = if progress < 0.3 {
                let expansion_progress = progress / 0.3;
                1.0 + (expansion_progress
                    * (animation.peak_scale.x / animation.original_scale.x - 1.0))
            } else {
                let return_progress = (progress - 0.3) / 0.7;
                let peak_scale_factor = animation.peak_scale.x / animation.original_scale.x;
                peak_scale_factor + (return_progress * (1.0 - peak_scale_factor))
            };

            // Apply the scale while preserving the flip state
            let flip_multiplier = if transform.scale.x < 0.0 { -1.0 } else { 1.0 };
            transform.scale = animation.original_scale * scale_factor;
            transform.scale.x *= flip_multiplier;
        }
    }
}
