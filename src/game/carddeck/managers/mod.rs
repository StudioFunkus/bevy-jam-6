use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{
    Animator, Tween, TweenCompleted, TweeningPlugin,
    lens::{TransformPositionLens, TransformScaleLens},
};

use crate::{
    game::carddeck::{
        card::Card,
        constants::{SCALE_TWEEN_DURATION, TRANSLATION_TWEEN_DURATION},
        markers::Dragged,
    },
    screens::Screen,
};

use super::constants::CARD_IN_PLAY_POSITION;

mod dragging_manager;
mod hover_manager;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        TweeningPlugin,
        dragging_manager::plugin,
        hover_manager::plugin,
    ));

    app.add_systems(
        Update,
        cleanup_finished_animators.run_if(in_state(Screen::Gameplay)),
    );

    app.add_observer(on_finish_transform_tween);
}

#[tracing::instrument(skip_all)]
pub fn create_card_scale_tween(
    mut commands: Commands,
    card_entity: Entity,
    scale_lens: TransformScaleLens,
) -> Result {
    let scale_tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs_f32(SCALE_TWEEN_DURATION),
        scale_lens,
    )
    .with_completed_event(2);

    commands
        .entity(card_entity)
        .with_child(Animator::new(scale_tween).with_target(card_entity));

    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn create_tween_return_to_origin(
    mut commands: Commands,
    card_entity: Entity,
    card_component: &Card,
    card_transform: &Transform,
) -> Result {
    let move_tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs_f32(TRANSLATION_TWEEN_DURATION),
        TransformPositionLens {
            start: card_transform.translation,
            end: card_component.origin.translation,
        },
    )
    .with_completed_event(1);

    commands
        .entity(card_entity)
        .with_child(Animator::new(move_tween).with_target(card_entity));

    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn create_tween_move_to_play(
    mut commands: Commands,
    card_entity: Entity,
    card_transform: &Transform,
) -> Result {
    let move_tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs_f32(TRANSLATION_TWEEN_DURATION),
        TransformPositionLens {
            start: card_transform.translation,
            end: CARD_IN_PLAY_POSITION,
        },
    )
    .with_completed_event(1);

    commands
        .entity(card_entity)
        .with_child(Animator::new(move_tween).with_target(card_entity));

    Ok(())
}

#[tracing::instrument(skip_all)]
fn on_finish_transform_tween(
    trigger: Trigger<TweenCompleted>,
    mut commands: Commands,
    dragged_query: Query<&Dragged, With<Card>>,
) -> Result {
    if trigger.user_data == 1 {
        let mut entity_commands = commands.entity(trigger.entity);
        if let Ok(dragged_component) = dragged_query.get(trigger.entity) {
            if *dragged_component == Dragged::Released {
                entity_commands.remove::<Dragged>();
            }
        }
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
fn cleanup_finished_animators(
    mut commands: Commands,
    animator_query: Query<(Entity, &Animator<Transform>)>,
) -> Result {
    for (entity, animator) in animator_query {
        if animator.tweenable().progress() == 1.0 {
            commands.entity(entity).despawn();
        }
    }

    Ok(())
}
