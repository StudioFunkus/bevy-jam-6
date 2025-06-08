use bevy::prelude::*;

use crate::game::{
    carddeck::{
        card::Card,
        managers::{create_tween_move_to_play, create_tween_return_to_origin},
        markers::{Draggable, Dragged},
    },
    game_flow::TurnPhase,
    mushrooms::SelectedMushroomType,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_card_drag)
        .add_observer(on_card_darg_start)
        .add_observer(on_card_drag_end)
        .add_observer(exit_play_card_mode);

    app.add_systems(Update, move_cards_back_to_origin);
}

#[tracing::instrument(skip_all)]
fn card_already_being_dragged(
    card_entity: Entity,
    dragged_cards: Query<(Entity, &Dragged), With<Card>>,
) -> bool {
    for (entity, dragged_component) in dragged_cards {
        if entity == card_entity {
            continue;
        }
        if dragged_component != &Dragged::Released {
            return true;
        }
    }

    false
}

#[tracing::instrument(skip_all)]
pub fn on_card_drag(
    mut trigger: Trigger<Pointer<Drag>>,
    mut card_transform: Query<&mut Transform, (With<Draggable>, With<Card>)>,
    dragged_query: Query<(Entity, &Dragged), With<Card>>,
    turn_phase: Option<Res<State<TurnPhase>>>,
) -> Result {
    trigger.propagate(false);

    // Only allow card drags if in planting phase
    let Some(phase) = turn_phase else {
        return Ok(());
    };
    if phase.get() != &TurnPhase::Planting {
        return Ok(());
    }

    if card_already_being_dragged(trigger.target, dragged_query) {
        return Ok(());
    }

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
    dragged_query: Query<(Entity, &Dragged), With<Card>>,
    turn_phase: Option<Res<State<TurnPhase>>>,
) -> Result {
    trigger.propagate(false);

    // Only allow card drags if in planting phase
    let Some(phase) = turn_phase else {
        return Ok(());
    };
    if phase.get() != &TurnPhase::Planting {
        return Ok(());
    }

    if card_already_being_dragged(trigger.target, dragged_query) {
        return Ok(());
    }

    commands.entity(trigger.target).insert(Dragged::Active);

    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn on_card_drag_end(
    trigger: Trigger<Pointer<DragEnd>>,
    mut cards_being_dragged: Query<(&mut Dragged, &Card)>,
    mut selected_type: ResMut<SelectedMushroomType>,
    window: Query<&Window>,
) -> Result {
    let window = window.single()?;

    if let Ok((mut dragged_component, card)) = cards_being_dragged.get_mut(trigger.target) {
        if trigger.pointer_location.position.y < window.height() * 0.8 {
            *dragged_component = Dragged::Played;
            selected_type.mushroom_type = Some(card.mushroom_type);
        } else {
            *dragged_component = Dragged::Released;
        };
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
            Dragged::Released => {
                create_tween_return_to_origin(
                    commands.reborrow(),
                    card_entity,
                    card_component,
                    card_transform,
                )?;
            }
            Dragged::Played => {
                create_tween_move_to_play(commands.reborrow(), card_entity, card_transform)?;
            }
            _ => continue,
        }
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn exit_play_card_mode(
    trigger: Trigger<Pointer<Click>>,
    dragged_query: Query<&mut Dragged, With<Card>>,
    mut selected_type: ResMut<SelectedMushroomType>,
) -> Result {
    if trigger.button == PointerButton::Secondary {
        info!("Exiting play mode");

        for mut dragged_component in dragged_query {
            if *dragged_component == Dragged::Played {
                *dragged_component = Dragged::Released;
                selected_type.mushroom_type = None;
            }
        }
    }

    Ok(())
}
