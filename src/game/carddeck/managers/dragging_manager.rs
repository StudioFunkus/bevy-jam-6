use bevy::prelude::*;

use crate::game::carddeck::{
    card::Card,
    managers::create_card_translation_tween,
    markers::{Draggable, Dragged},
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_card_drag)
        .add_observer(on_card_darg_start)
        .add_observer(on_card_drag_end);

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
                create_card_translation_tween(
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
