use bevy::prelude::*;
use card::{Card, CardTemplates};
use deck::Deck;
use hand::{Hand, draw_one};
use rand::Rng;

use crate::screens::Screen;

use super::mushrooms::MushroomType;

mod card;
mod deck;
pub(crate) mod events;
mod hand;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Deck>();
    app.init_resource::<Hand>();
    app.init_resource::<CardTemplates>();

    app.add_systems(
        OnEnter(Screen::Gameplay),
        (create_card_definitions, create_test_deck).chain(),
    );

    app.add_observer(draw_one);
}

#[tracing::instrument(name = "Create card definitions", skip_all)]
fn create_card_definitions(mut card_templates: ResMut<CardTemplates>) -> Result {
    card_templates.0.extend(vec![
        Card {
            name: "Button".to_string(),
            mushroom_type: MushroomType::Basic,
        },
        Card {
            name: "Pulcini".to_string(),
            mushroom_type: MushroomType::Pulse,
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
