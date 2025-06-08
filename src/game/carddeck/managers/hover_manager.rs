use bevy::prelude::*;
use bevy_tweening::lens::TransformScaleLens;

use crate::game::carddeck::{card::Card, markers::Hovered};

use super::create_card_scale_tween;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_card_hover)
        .add_observer(on_hover_finish)
        .add_observer(on_hover_added)
        .add_observer(on_hover_removed);
}

#[tracing::instrument(skip_all)]
fn on_card_hover(
    mut trigger: Trigger<Pointer<Over>>,
    mut commands: Commands,
    cards_query: Query<&Card>,
) -> Result {
    trigger.propagate(false);

    // Only add hovered if the target is a card
    if cards_query.get(trigger.target).is_ok() {
        commands.entity(trigger.target).insert(Hovered);
    };

    Ok(())
}

#[tracing::instrument(skip_all)]
fn on_hover_finish(
    mut trigger: Trigger<Pointer<Out>>,
    mut commands: Commands,
    cards_query: Query<&Card>,
) -> Result {
    trigger.propagate(false);

    if cards_query.get(trigger.target).is_ok() {
        commands.entity(trigger.target).remove::<Hovered>();
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
fn on_hover_added(
    trigger: Trigger<OnAdd, Hovered>,
    commands: Commands,
    cards_query: Query<(Entity, &Card), (With<Hovered>, With<Card>)>,
) -> Result {
    let (card_entity, card_component) = cards_query.get(trigger.target())?;

    let scale_lens = TransformScaleLens {
        start: card_component.origin.scale,
        end: card_component.origin.scale * 1.1,
    };

    create_card_scale_tween(commands, card_entity, scale_lens)?;

    Ok(())
}

#[tracing::instrument(skip_all)]
fn on_hover_removed(
    trigger: Trigger<OnRemove, Hovered>,
    commands: Commands,
    cards_query: Query<(Entity, &Card), (With<Hovered>, With<Card>)>,
) -> Result {
    let (card_entity, card_component) = cards_query.get(trigger.target())?;

    let scale_lens = TransformScaleLens {
        start: card_component.origin.scale * 1.1,
        end: card_component.origin.scale,
    };

    create_card_scale_tween(commands, card_entity, scale_lens)?;

    Ok(())
}
