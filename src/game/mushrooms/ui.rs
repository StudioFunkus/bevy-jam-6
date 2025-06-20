//! Visual information display for mushrooms
//! Add this as a new file: src/game/mushrooms/visual_info.rs

use bevy::prelude::*;
use bevy_rich_text3d::{Text3d, TextAtlas};

use crate::game::{
    game_flow::LevelState,
    mushrooms::{Mushroom, MushroomActivationState, MushroomDefinitions, events::SporeScoreEvent},
    resources::GameState,
    visual_effects::FaceCamera,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (animate_spore_popups, update_uses_display).run_if(in_state(LevelState::Playing)),
    );

    app.add_observer(spawn_spore_popup);
    app.add_observer(spawn_uses_display);
}

/// Component for the uses remaining display
#[derive(Component)]
pub struct UsesDisplay;

/// Component for spore generation popups
#[derive(Component)]
pub struct SporePopup {
    pub timer: Timer,
    pub start_y: f32,
}

impl SporePopup {
    #[allow(dead_code)]
    pub fn new(duration: f32, start_y: f32, _text_span_entity: Entity) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            start_y,
        }
    }
}

/// Update uses display when mushroom activation state changes
fn update_uses_display(
    mushrooms: Query<
        (Entity, &Mushroom, &MushroomActivationState),
        Changed<MushroomActivationState>,
    >,
    definitions: Res<MushroomDefinitions>,
    mut uses_displays: Query<(Entity, &ChildOf), With<UsesDisplay>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mushroom, state) in mushrooms.iter() {
        // Get the mushroom definition
        let Some(definition) = definitions.get(mushroom.0) else {
            warn!("No definition found for mushroom type {:?}", mushroom.0);
            continue;
        };

        // Calculate remaining uses
        let remaining_uses = definition
            .max_uses_per_turn
            .saturating_sub(state.activations_this_turn);

        // Find uses displays that are children of this mushroom
        for (display_entity, child_of) in uses_displays.iter_mut() {
            if child_of.parent() == entity {
                // Replace the Text3d component with updated text
                commands
                    .entity(display_entity)
                    .insert(Text3d::new(format!("{remaining_uses}")));

                // Update color based on remaining uses
                let color = if remaining_uses == 0 {
                    Color::srgb(1.0, 0.2, 0.2) // Red when exhausted
                } else {
                    Color::WHITE // White when uses remain
                };

                // Create new material with the appropriate color
                let mat = materials.add(StandardMaterial {
                    base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    base_color: color,
                    ..Default::default()
                });

                commands.entity(display_entity).insert(MeshMaterial3d(mat));
            }
        }
    }
}

/// Spawn uses display for mushrooms
fn spawn_uses_display(
    trigger: Trigger<OnAdd, Mushroom>,
    mut commands: Commands,
    definitions: Res<MushroomDefinitions>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mushrooms: Query<(&MushroomActivationState, &Mushroom)>,
) {
    let entity = trigger.target();

    let Ok((state, mushroom)) = mushrooms.get(entity) else {
        return;
    };

    let definition = definitions.get(mushroom.0).unwrap();

    let remaining_uses = definition
        .max_uses_per_turn
        .saturating_sub(state.activations_this_turn);

    let mat = materials.add(StandardMaterial {
        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..Default::default()
    });

    // Spawn billboard text as child of mushroom
    commands.spawn((
        Name::new("Uses Display Billboard"),
        Text3d::new(format!("{remaining_uses}")),
        Mesh3d::default(),
        Transform::from_xyz(-0.25, 0.5, 0.0).with_scale(Vec3::splat(0.019)),
        MeshMaterial3d(mat.clone()),
        StateScoped(LevelState::Playing),
        UsesDisplay,
        FaceCamera,
        ChildOf(entity),
    ));
}

/// Spawn a popup showing spore generation
pub fn spawn_spore_popup(
    trigger: Trigger<SporeScoreEvent>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_state: Res<GameState>,
) {
    let mat = materials.add(StandardMaterial {
        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..Default::default()
    });

    let world_pos = trigger.event().position.to_world_in(&game_state.play_field);

    commands.spawn((
        Name::new("Spore Popup"),
        Text3d::new(format!("+{:.0}", trigger.event().production)),
        Mesh3d::default(),
        MeshMaterial3d(mat.clone()),
        Transform::from_xyz(world_pos.x, 1.0, -world_pos.z).with_scale(Vec3::splat(0.022)),
        FaceCamera,
        StateScoped(LevelState::Playing),
        SporePopup {
            timer: Timer::from_seconds(2.0, TimerMode::Once),
            start_y: 1.0,
        },
    ));
}

/// Animate spore popups - float up
fn animate_spore_popups(
    mut commands: Commands,
    time: Res<Time>,
    mut popups: Query<(Entity, &mut Transform, &mut SporePopup)>,
) {
    for (entity, mut transform, mut popup) in popups.iter_mut() {
        popup.timer.tick(time.delta());

        let progress = popup.timer.fraction();

        // Float upward
        let float_distance = 1.5;
        transform.translation.y = popup.start_y + (progress * float_distance);

        // Remove when animation is done
        if popup.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
