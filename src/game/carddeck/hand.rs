//! # Hand
//!
//! The hand contains the cards that have been drawn and are currently playable by the player.
//! These are drawn from the deck.

use bevy::prelude::*;
use std::collections::VecDeque;

use crate::{
    CARD_LAYER,
    game::carddeck::{
        card::{Card, CardBundle},
        deck::Deck,
        events::{CardAddedEvent, DrawEvent},
    },
    screens::Screen,
};

const CARD_SPACING: f32 = 75.0;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Hand>();

    app.init_resource::<Hand>();

    app.add_systems(OnEnter(Screen::Gameplay), spawn_hand_entity);

    app.add_observer(update_card_origins);
}

fn spawn_hand_entity(mut commands: Commands, window: Query<&Window>) -> Result {
    let window = window.single()?;

    commands.spawn((
        HandEntity,
        Transform::from_xyz(0.0, -(0.9 * (window.height() / 2.0)), 0.0),
        CARD_LAYER,
    ));

    Ok(())
}

/// The hand [`Resource`], which contains a [`VecDeque`] of the cards within it.
///
/// You'll notice this is very similar to how the deck is defined. The two could possible
/// be merged into a single defition at a later date.
#[derive(Resource, Debug, Reflect)]
pub struct Hand {
    cards: VecDeque<(Card, Option<Entity>)>,
    pub max_cards: usize,
}

impl Hand {
    /// Get count of cards in hand
    #[allow(dead_code)]
    pub fn get_card_count(&self) -> usize {
        self.cards.len()
    }
}

impl Default for Hand {
    fn default() -> Self {
        Self {
            cards: VecDeque::new(),
            max_cards: 9,
        }
    }
}

#[derive(Component)]
pub struct HandEntity;

/// Draw N cards from deck into hand
#[tracing::instrument(skip_all)]
pub fn draw_n(
    trigger: Trigger<DrawEvent>,
    mut commands: Commands,
    mut hand: ResMut<Hand>,
    hand_entity: Query<Entity, With<HandEntity>>,
    mut deck: ResMut<Deck>,
) -> Result {
    let hand_entity = hand_entity.single()?;

    let mut cards_to_draw = trigger.0;
    info!(
        "Trying to draw {} cards to hand with {} cards and a max of {}",
        cards_to_draw,
        hand.cards.len(),
        hand.max_cards
    );

    if hand.cards.len() as u32 + cards_to_draw > hand.max_cards as u32 {
        cards_to_draw = (hand.max_cards - hand.cards.len()) as u32;
        info!("Cannot fit cards, will draw {}", cards_to_draw);
    }

    if cards_to_draw > deck.get_card_count() as u32 {
        cards_to_draw = deck.get_card_count() as u32;
        info!("Not enough cards in deck, will draw {}", cards_to_draw);
    }

    for _ in 0..cards_to_draw {
        let Some(drawn_card) = deck.draw() else {
            break;
        };

        let card_entity = spawn_card(commands.reborrow(), drawn_card.clone(), hand_entity);

        hand.cards
            .push_back((drawn_card, Some(card_entity.clone())));
    }

    commands.trigger(CardAddedEvent);

    Ok(())
}

#[tracing::instrument(skip_all)]
fn spawn_card(mut commands: Commands, card: Card, hand_entity: Entity) -> Entity {
    info!("Spawning card entity");
    let card_color = card.mushroom_type.color().clone();
    let card_entity = commands
        .spawn(CardBundle {
            name: card.name.clone().into(),
            card: card,
            transform: Transform::default().with_scale(Vec3::new(15.0, 20.0, 5.0)),
            sprite: Sprite::from_color(card_color, Vec2::new(3.0, 5.0)),
            ..default()
        })
        .id();
    commands.entity(hand_entity).add_child(card_entity);

    card_entity
}

#[tracing::instrument(skip_all)]
fn update_card_origins(
    _: Trigger<CardAddedEvent>,
    hand: Res<Hand>,
    mut cards_query: Query<(&mut Card, &Transform)>,
) -> Result {
    let number_of_cards: f32 = hand.get_card_count() as f32;

    let first_card_offset: f32;

    if number_of_cards % 2.0 == 1.0 {
        first_card_offset = ((number_of_cards - 1.0) / 2.0) * CARD_SPACING;
    } else {
        first_card_offset = (number_of_cards / 2.0) * CARD_SPACING;
    }

    for (index, card_tuple) in hand.cards.iter().enumerate() {
        if let (card, Some(entity)) = card_tuple {
            info!("Searching for card: {}", entity);
            let (mut card_component, card_transform) = cards_query.get_mut(*entity)?;
            let new_origin = card_component
                .origin
                .translation
                .with_x(-first_card_offset + ((index + 1) as f32 * CARD_SPACING));

            card_component.origin.translation = new_origin;
            info!(
                "Updated card origin to {}",
                card_component.origin.translation
            );
        }
    }

    Ok(())
}
