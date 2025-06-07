use bevy::prelude::*;

use crate::game::carddeck::{card::Card, markers::Hovered};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_card_hover);
}

#[tracing::instrument(skip_all)]
fn on_card_hover(
    mut trigger: Trigger<Pointer<Over>>,
    mut commands: Commands,
    cards_query: Query<&Card>,
) -> Result {
    trigger.propagate(false);

    // Only add hovered if the target is a card
    if let Ok(_) = cards_query.get(trigger.target) {
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

    if let Ok(_) = cards_query.get(trigger.target) {
        commands.entity(trigger.target).remove::<Hovered>();
    }

    Ok(())
}
