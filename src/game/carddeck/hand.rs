//! # Hand
//!
//! The hand contains the cards that have been drawn and are currently playable by the player.
//! These are drawn from the deck.

use bevy::prelude::*;
use std::collections::VecDeque;

use crate::{
    game::carddeck::{
        card::{Card, CardBundle},
        constants::{CARD_LAYER, CARD_SPACING},
        deck::Deck,
        events::{DrawEvent, HandChangeEvent},
    },
    screens::Screen,
};

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
///
/// Will check that cards will fit and cards remaining in deck, and
/// adjust amount to draw as needed.
///
/// When the card is drawn, [`spawn_card`] is triggered to also create
/// the entity that will represent the card visually.
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

    commands.trigger(HandChangeEvent);

    Ok(())
}

/// Spawn a card, adding the [`Entity`] and [`Card`] component to the [`Hand`] resource.
///
/// This ensures that the [`Hand`] resource is kept up-to-date with changes.
#[tracing::instrument(skip_all)]
fn spawn_card(mut commands: Commands, card: Card, hand_entity: Entity) -> Entity {
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

/// Despawn a card, whilst also removing the [`Entity`] and [`Card`] component from the [`Hand`] resource.
#[tracing::instrument(skip_all)]
fn despawn_card(mut commands: Commands, card_entity: Entity, mut hand: ResMut<Hand>) -> Result {
    for (index, (_, entity)) in hand.cards.iter().enumerate() {
        if *entity == Some(card_entity) {
            commands.entity(card_entity).despawn();
            hand.cards.remove(index);
            return Ok(());
        }
    }

    error!("Error trying to despawn card, no matching entity found in hand!");
    Ok(())
}

/// Update the value of the origin property on a [`Card`] component.
///
/// Triggered via [`HandChangeEvent`], which is fired whenever a [`Card`] component is added
/// to or removed from the [`Hand`].
#[tracing::instrument(skip_all)]
fn update_card_origins(
    _: Trigger<HandChangeEvent>,
    hand: Res<Hand>,
    mut cards_query: Query<&mut Card>,
) -> Result {
    let number_of_cards: f32 = hand.get_card_count() as f32;

    let first_card_offset: f32;

    if number_of_cards % 2.0 == 1.0 {
        first_card_offset = ((number_of_cards - 1.0) / 2.0) * CARD_SPACING;
    } else {
        first_card_offset = (number_of_cards / 2.0) * CARD_SPACING;
    }

    for (index, card_tuple) in hand.cards.iter().enumerate() {
        if let (_, Some(entity)) = card_tuple {
            let mut card_component = cards_query.get_mut(*entity)?;
            let new_origin = card_component
                .origin
                .translation
                .with_x(-first_card_offset + ((index + 1) as f32 * CARD_SPACING))
                .with_z((index + 1) as f32);

            card_component.origin.translation = new_origin;
        }
    }

    Ok(())
}
