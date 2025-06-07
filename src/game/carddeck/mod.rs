use bevy::math::VectorSpace;
use bevy::prelude::*;
use card::{Card, create_card_definitions};
use constants::CARD_LAYER;
use deck::create_deck;
use hand::{HandEntity, draw_n};

use crate::screens::Screen;

use crate::game::mushrooms::{MushroomDefinitions, MushroomType};

mod card;
pub mod constants;
mod deck;
pub(crate) mod events;
mod hand;
mod managers;
mod markers;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((hand::plugin, card::plugin, deck::plugin, managers::plugin));

    app.add_systems(
        OnEnter(Screen::Gameplay),
        (create_card_definitions, create_deck).chain(),
    );

    app.add_observer(draw_n);
}
