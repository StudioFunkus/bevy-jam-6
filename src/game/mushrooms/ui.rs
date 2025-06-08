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
        (animate_spore_popups,).run_if(in_state(LevelState::Playing)),
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
    pub fn new(duration: f32, start_y: f32, text_span_entity: Entity) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            start_y,
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
        Transform::from_xyz(-0.25, 0.5, 0.0).with_scale(Vec3::splat(0.012)),
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
        Transform::from_xyz(world_pos.x, 1.0, -world_pos.z).with_scale(Vec3::splat(0.012)),
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
