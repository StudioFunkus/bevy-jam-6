use bevy::prelude::*;
use card::{Card, CardTemplates};
use deck::Deck;
use hand::draw_n;
use rand::Rng;

use crate::screens::Screen;

use crate::game::mushrooms::MushroomType;

mod card;
mod deck;
pub(crate) mod events;
mod hand;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((hand::plugin, card::plugin, deck::plugin));

    app.add_systems(
        OnEnter(Screen::Gameplay),
        (create_card_definitions, create_test_deck).chain(),
    );

    app.add_observer(draw_n);
}

#[tracing::instrument(name = "Create card definitions", skip_all)]
fn create_card_definitions(mut card_templates: ResMut<CardTemplates>) -> Result {
    card_templates.0.extend(vec![
        Card {
            name: "Button".to_string(),
            mushroom_type: MushroomType::Basic,
            ..default()
        },
        Card {
            name: "Pulcini".to_string(),
            mushroom_type: MushroomType::Pulse,
            ..default()
        },
    ]);

    info!("Defined {} card(s)", card_templates.0.len());

    Ok(())
}

#[tracing::instrument(name = "Create test deck", skip_all)]
fn create_test_deck(mut deck: ResMut<Deck>, card_templates: Res<CardTemplates>) -> Result {
    let mut rng = rand::thread_rng();
    let count_of_defined_cards = card_templates.0.len();

    for _ in 0..10 {
        deck.add_to_bottom(
            card_templates
                .0
                .get(rng.gen_range(0..count_of_defined_cards))
                .unwrap()
                .clone(),
        )?;
    }

    info!("Added {} card(s) to deck", deck.get_card_count());

    Ok(())
}
