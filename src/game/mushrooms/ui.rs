//! Visual information display for mushrooms
//! Add this as a new file: src/game/mushrooms/visual_info.rs

use bevy::prelude::*;
use bevy_rich_text3d::{Text3d, Text3dBounds, Text3dStyling, TextAlign, TextAnchor, TextAtlas};

use crate::game::{
    game_flow::LevelState,
    mushrooms::{events::SporeScoreEvent, Mushroom, MushroomActivationState, MushroomDefinitions},
    resources::GameState, visual_effects::FaceCamera,
};

pub(super) fn plugin(app: &mut App) {
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

    let mat = materials.add(
        StandardMaterial {
            base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..Default::default()
        }
    );
    // Spawn billboard text as child of mushroom
    commands
        .spawn((
            Name::new("Uses Display Billboard"),
            Text3d::new(format!("{remaining_uses}")),
            Mesh3d::default(),
            Text3dStyling {
                size: 32.,
                color: Srgba::new(0., 1., 1., 1.),
                align: TextAlign::Center,
                anchor: TextAnchor::CENTER_LEFT,
                ..Default::default()
            },
            Text3dBounds {
                width: 400.,
            },
            MeshMaterial3d(mat.clone()),
            StateScoped(LevelState::Playing),
            UsesDisplay,
            FaceCamera
        ));
}

/// Spawn a popup showing spore generation
pub fn spawn_spore_popup(
    trigger: Trigger<SporeScoreEvent>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_state: Res<GameState>,
) {

    let mat = materials.add(
        StandardMaterial {
            base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..Default::default()
        }
    );

    commands.spawn((
        Name::new("Spore Popup"),
        Text3d::new(format!("+{:.0}", trigger.event().production)),
        Mesh3d::default(),
        MeshMaterial3d(mat.clone()),
        StateScoped(LevelState::Playing),
        Transform::from_translation(
            trigger
                .event()
                .position
                .to_world(game_state.play_field.width, game_state.play_field.height)
        )
        .with_scale(Vec3::splat(0.012)),
    ));
}

/// Animate spore popups - float up
fn _animate_spore_popups(
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
