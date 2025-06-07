use bevy::prelude::*;
use bevy_tweening::{Animator, Tween, TweenCompleted, lens::TransformPositionLens};
use std::time::Duration;

use crate::game::carddeck::{
    card::Card,
    constants::RETURN_TO_HAND_DURATION,
    markers::{Draggable, Dragged},
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_card_drag)
        .add_observer(on_card_darg_start)
        .add_observer(on_card_drag_end)
        .add_observer(on_card_move_done);

    app.add_systems(Update, move_cards_back_to_origin);
}

#[tracing::instrument(skip_all)]
pub fn on_card_drag(
    mut trigger: Trigger<Pointer<Drag>>,
    mut card_transform: Query<&mut Transform, (With<Draggable>, With<Card>)>,
) -> Result {
    trigger.propagate(false);
    if let Ok(mut card_transform) = card_transform.get_mut(trigger.target) {
        card_transform.translation.x += trigger.delta.x;
        card_transform.translation.y -= trigger.delta.y;
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn on_card_darg_start(
    mut trigger: Trigger<Pointer<DragStart>>,
    mut commands: Commands,
    cards_being_dragged: Query<&Dragged, With<Card>>,
) -> Result {
    trigger.propagate(false);

    // Abort if another card is being dragged
    for dragged_component in cards_being_dragged {
        match dragged_component {
            Dragged::Active => return Ok(()),
            _ => continue,
        }
    }

    commands.entity(trigger.target).insert(Dragged::Active);

    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn on_card_drag_end(
    trigger: Trigger<Pointer<DragEnd>>,
    mut cards_being_dragged: Query<&mut Dragged, With<Card>>,
) -> Result {
    if let Ok(mut dragged_component) = cards_being_dragged.get_mut(trigger.target) {
        *dragged_component = Dragged::Released;
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn move_cards_back_to_origin(
    mut commands: Commands,
    mut cards_being_dragged: Query<(Entity, &Transform, &Card, &Dragged), Changed<Dragged>>,
) -> Result {
    for (card_entity, card_transform, card_component, dragged_component) in
        cards_being_dragged.iter_mut()
    {
        match dragged_component {
            Dragged::Active => {
                continue;
            }
            Dragged::Released => {
                create_tween_for_card(
                    commands.reborrow(),
                    card_entity,
                    card_component,
                    card_transform,
                )?;
            }
        }
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
fn create_tween_for_card(
    mut commands: Commands,
    card_entity: Entity,
    card_component: &Card,
    card_transform: &Transform,
) -> Result {
    let move_tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs_f32(RETURN_TO_HAND_DURATION),
        TransformPositionLens {
            start: card_transform.translation,
            end: card_component.origin.translation,
        },
    )
    .with_completed_event(1);

    commands
        .entity(card_entity)
        .insert(Animator::new(move_tween));

    Ok(())
}

#[tracing::instrument(skip_all)]
fn on_card_move_done(trigger: Trigger<TweenCompleted>, mut commands: Commands) -> Result {
    commands.entity(trigger.target()).remove::<Dragged>();
    commands
        .entity(trigger.target())
        .remove::<Animator<Transform>>();

    Ok(())
}
