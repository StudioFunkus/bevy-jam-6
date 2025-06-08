use bevy::prelude::*;
use card::create_card_definitions;
use deck::create_deck;
use hand::draw_n;

use crate::screens::Screen;

pub(crate) mod card;
pub(crate) mod constants;
mod deck;
pub(crate) mod events;
pub(crate) mod hand;
mod managers;
pub(crate) mod markers;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((hand::plugin, card::plugin, deck::plugin, managers::plugin));

    app.add_systems(
        OnEnter(Screen::Gameplay),
        (create_card_definitions, create_deck).chain(),
    );

    app.add_observer(draw_n);
}
