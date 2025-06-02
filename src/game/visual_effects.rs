//! Visual effects and feedback for game interactions

use crate::{
    PausableSystems,
    game::{
        grid::{GridConfig, GridPosition},
        mushrooms::{Mushroom, MushroomDirection, MushroomType},
    },
};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_event::<SpawnTriggerEffect>();
    app.add_event::<SpawnDirectionalPulse>();
    app.add_event::<SpawnClickEffect>();

    app.add_systems(
        Update,
        (
            spawn_trigger_effects,
            spawn_directional_pulses,
            spawn_click_effects,
            update_directional_indicators,
            animate_effects,
            cleanup_expired_effects,
        )
            .chain()
            .in_set(PausableSystems),
    );
}

/// Event to spawn a trigger effect at a position
#[derive(Event)]
pub struct SpawnTriggerEffect {
    pub position: GridPosition,
    pub color: Color,
}

/// Event to spawn a directional pulse effect
#[derive(Event)]
pub struct SpawnDirectionalPulse {
    pub from_position: GridPosition,
    pub to_position: GridPosition,
    pub color: Color,
}

/// Event to spawn a click effect
#[derive(Event)]
pub struct SpawnClickEffect {
    pub position: GridPosition,
}

/// Component for visual effects with a lifetime
#[derive(Component)]
struct VisualEffect {
    lifetime: Timer,
}

/// Component for animated effects
#[derive(Component)]
struct AnimatedEffect {
    start_scale: Vec3,
    end_scale: Vec3,
    start_alpha: f32,
    end_alpha: f32,
}

/// Component for directional indicator
#[derive(Component)]
struct DirectionalIndicator;

/// Spawn trigger effects when mushrooms are activated
fn spawn_trigger_effects(
    mut commands: Commands,
    mut events: EventReader<SpawnTriggerEffect>,
    grid_config: Res<GridConfig>,
) {
    for event in events.read() {
        // Spawn expanding ring effect
        commands.spawn((
            Name::new("Trigger Effect"),
            Sprite {
                color: event.color.with_alpha(0.8),
                custom_size: Some(Vec2::splat(40.0)),
                ..default()
            },
            Transform::from_translation(
                event.position.to_world(&grid_config) + Vec3::new(0.0, 0.0, 1.0),
            )
            .with_scale(Vec3::splat(0.5)),
            VisualEffect {
                lifetime: Timer::from_seconds(0.5, TimerMode::Once),
            },
            AnimatedEffect {
                start_scale: Vec3::splat(0.5),
                end_scale: Vec3::splat(1.5),
                start_alpha: 0.8,
                end_alpha: 0.2,
            },
        ));

        // Spawn inner pulse
        commands.spawn((
            Name::new("Trigger Pulse"),
            Sprite {
                color: Color::WHITE.with_alpha(0.5),
                custom_size: Some(Vec2::splat(30.0)),
                ..default()
            },
            Transform::from_translation(
                event.position.to_world(&grid_config) + Vec3::new(0.0, 0.0, 1.1),
            )
            .with_scale(Vec3::splat(0.3)),
            VisualEffect {
                lifetime: Timer::from_seconds(0.3, TimerMode::Once),
            },
            AnimatedEffect {
                start_scale: Vec3::splat(0.3),
                end_scale: Vec3::splat(1.0),
                start_alpha: 1.0,
                end_alpha: 1.0,
            },
        ));
    }
}

/// Spawn directional pulse effects
fn spawn_directional_pulses(
    mut commands: Commands,
    mut events: EventReader<SpawnDirectionalPulse>,
    grid_config: Res<GridConfig>,
) {
    for event in events.read() {
        let from_world = event.from_position.to_world(&grid_config);
        let to_world = event.to_position.to_world(&grid_config);
        let direction = (to_world - from_world).normalize();

        // Spawn traveling pulse
        for i in 0..3 {
            let delay = i as f32 * 0.1;
            let start_pos = from_world + direction * 40.0;

            commands.spawn((
                Name::new("Directional Pulse"),
                Sprite {
                    color: event.color.with_alpha(0.6),
                    custom_size: Some(Vec2::new(20.0, 10.0)),
                    ..default()
                },
                Transform::from_translation(start_pos + Vec3::new(0.0, 0.0, 1.2))
                    .with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))),
                VisualEffect {
                    lifetime: Timer::from_seconds(0.5 + delay, TimerMode::Once),
                },
                TravelingPulse {
                    start_pos,
                    end_pos: to_world,
                    progress: -delay * 2.0,
                },
            ));
        }
    }
}

/// Component for traveling pulse effects
#[derive(Component)]
struct TravelingPulse {
    start_pos: Vec3,
    end_pos: Vec3,
    progress: f32,
}

/// Spawn click effects
fn spawn_click_effects(
    mut commands: Commands,
    mut events: EventReader<SpawnClickEffect>,
    grid_config: Res<GridConfig>,
) {
    for event in events.read() {
        // Spawn click ripple
        commands.spawn((
            Name::new("Click Effect"),
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 0.3),
                custom_size: Some(Vec2::splat(20.0)),
                ..default()
            },
            Transform::from_translation(
                event.position.to_world(&grid_config) + Vec3::new(0.0, 0.0, 1.3),
            ),
            VisualEffect {
                lifetime: Timer::from_seconds(0.2, TimerMode::Once),
            },
            AnimatedEffect {
                start_scale: Vec3::splat(0.2),
                end_scale: Vec3::splat(0.8),
                start_alpha: 0.3,
                end_alpha: 0.3,
            },
        ));
    }
}

/// Update directional indicators for mushrooms
fn update_directional_indicators(
    mut commands: Commands,
    mushrooms: Query<
        (Entity, &GridPosition, &MushroomDirection, &MushroomType),
        (With<Mushroom>, Changed<MushroomDirection>),
    >,
    existing_indicators: Query<(Entity, &ChildOf), With<DirectionalIndicator>>,
    grid_config: Res<GridConfig>,
) {
    for (mushroom_entity, position, direction, mushroom_type) in &mushrooms {
        // Remove existing indicator if any
        for (indicator_entity, child_of) in &existing_indicators {
            if child_of.parent() == mushroom_entity {
                commands.entity(indicator_entity).despawn();
            }
        }

        // Only show indicators for directional mushrooms
        if matches!(mushroom_type, MushroomType::Pulse) {
            let arrow_offset = match direction {
                MushroomDirection::Up => Vec3::new(0.0, 20.0, 0.0),
                MushroomDirection::Right => Vec3::new(20.0, 0.0, 0.0),
                MushroomDirection::Down => Vec3::new(0.0, -20.0, 0.0),
                MushroomDirection::Left => Vec3::new(-20.0, 0.0, 0.0),
            };

            let rotation = match direction {
                MushroomDirection::Up => 0.0,
                MushroomDirection::Right => -std::f32::consts::FRAC_PI_2,
                MushroomDirection::Down => std::f32::consts::PI,
                MushroomDirection::Left => std::f32::consts::FRAC_PI_2,
            };

            // Spawn arrow indicator as child of mushroom
            let indicator = commands
                .spawn((
                    Name::new("Direction Indicator"),
                    DirectionalIndicator,
                    Sprite {
                        color: Color::srgba(1.0, 1.0, 1.0, 0.7),
                        custom_size: Some(Vec2::new(10.0, 15.0)),
                        ..default()
                    },
                    Transform::from_translation(arrow_offset)
                        .with_rotation(Quat::from_rotation_z(rotation)),
                ))
                .id();

            commands.entity(mushroom_entity).add_child(indicator);
        }
    }
}

/// Animate visual effects
fn animate_effects(
    time: Res<Time>,
    mut effects: Query<(&mut Transform, &mut Sprite, &VisualEffect, &AnimatedEffect)>,
    mut pulses: Query<(&mut Transform, &TravelingPulse, &VisualEffect), Without<AnimatedEffect>>,
) {
    // Animate scaling/fading effects
    for (mut transform, mut sprite, effect, animated) in &mut effects {
        let progress = effect.lifetime.fraction();

        // Interpolate scale
        let scale = animated.start_scale.lerp(animated.end_scale, progress);
        transform.scale = scale;

        // Interpolate alpha
        // let alpha = animated.start_alpha + (animated.end_alpha - animated.start_alpha) * progress;
        // sprite.color.set_alpha(alpha);
    }

    // Animate traveling pulses
    for (mut transform, pulse, effect) in &mut pulses {
        let mut progress = pulse.progress + time.delta_secs() * 3.0; // Speed of travel
        progress = progress.clamp(0.0, 1.0);

        let pos = pulse.start_pos.lerp(pulse.end_pos, progress);
        transform.translation = pos + Vec3::new(0.0, 0.0, 1.2);
    }
}

/// Update effect lifetimes and remove expired ones
fn cleanup_expired_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut VisualEffect)>,
) {
    for (entity, mut effect) in &mut effects {
        effect.lifetime.tick(time.delta());

        if effect.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
