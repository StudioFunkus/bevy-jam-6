//! # Hand
//!
//! The hand contains the cards that have been drawn and are currently playable by the player.
//! These are drawn from the deck.

use bevy::prelude::*;
use std::collections::VecDeque;

use crate::{
    game::carddeck::{
        card::{Card, CardBundle, Draggable},
        deck::Deck,
        events::DrawEvent,
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Hand>();

    app.add_systems(OnEnter(Screen::Gameplay), create_hand);

    app.add_observer(on_card_add);
}

/// The hand [`Resource`], which contains a [`VecDeque`] of the cards within it.
///
/// You'll notice this is very similar to how the deck is defined. The two could possible
/// be merged into a single defition at a later date.
#[derive(Component, Default, Debug, Reflect)]
pub struct Hand {
    cards: VecDeque<Entity>,
    pub origin: Transform,
    pub max_cards: usize,
}

/// Draw N cards from deck into hand
#[tracing::instrument(name = "Draw N cards", skip_all)]
pub fn draw_n(
    trigger: Trigger<DrawEvent>,
    mut commands: Commands,
    mut hand: Query<&mut Hand>,
    mut deck: ResMut<Deck>,
) -> Result {
    let mut hand = hand.single_mut()?;

    let mut cards_to_draw = trigger.0;
    info!(
        "Trying to draw {} cards to hand with {} cards and a max of {}",
        cards_to_draw,
        hand.cards.len(),
        hand.max_cards
    );

    // Try into to convert usize into u32
    if hand.cards.len() as u32 + cards_to_draw > hand.max_cards as u32 {
        cards_to_draw = (hand.max_cards - hand.cards.len()) as u32;
        info!("Cannot fit cards, will draw {}", cards_to_draw);
    }

    let drawn_cards = deck.draw_n(cards_to_draw);

    for card in drawn_cards {
        let card_entity = commands
            .spawn(CardBundle {
                name: card.name.clone().into(),
                card: card.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                sprite: Sprite {
                    color: card.mushroom_type.color(),
                    custom_size: Some(Vec2::splat(60.0)),
                    ..default()
                },
                draggable: Draggable,
            })
            .id();

        hand.cards.push_back(card_entity);
    }

    Ok(())
}

#[tracing::instrument(name = "Create hand", skip_all)]
fn create_hand(mut commands: Commands) -> Result {
    commands.spawn((
        Hand {
            origin: Transform::from_xyz(0.0, -250.0, 10.0),
            max_cards: 9,
            ..default()
        },
        Name::new("Hand"),
        Transform::from_xyz(0.0, -250.0, 10.0),
    ));

    Ok(())
}

#[tracing::instrument(name = "On card add", skip_all)]
fn on_card_add(
    trigger: Trigger<OnAdd, Card>,
    mut commands: Commands,
    hand: Query<Entity, With<Hand>>,
    mut cards: Query<Entity, With<Card>>,
) -> Result {
    info!("Triggered");
    let hand = hand.single()?;
    let card = cards.get_mut(trigger.target())?;

    commands.entity(hand).add_child(card);
    info!("Made card a child of hand");

    Ok(())
}

#[tracing::instrument(skip_all)]
fn move_card_to_hand(hand: Query<&Hand, Changed<Hand>>, mut cards: Query<&mut Card>) -> Result {
    let hand = hand.single()?;
    let card_count = cards.iter().len();
    let card_gap = 100.0;
    let first_card_offset;

    // The offset of the first card in the x-axis
    if card_count % 2 == 1 {
        first_card_offset = -(((card_count - 1) / 2) as f32 * card_gap);
    } else {
        first_card_offset = -((card_count / 2) as f32 * card_gap);
    }

    for (index, entity) in hand.cards.iter().enumerate() {
        if let Ok(mut card) = cards.get_mut(*entity) {
            let new_translation = card
                .origin
                .translation
                .with_x(first_card_offset + (index as f32 * card_gap));
            card.origin.translation = new_translation;
        }
    }

    Ok(())
}
