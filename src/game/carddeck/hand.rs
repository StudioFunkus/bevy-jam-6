//! # Hand
//!
//! The hand contains the cards that have been drawn and are currently playable by the player.
//! These are drawn from the deck.

use bevy::prelude::*;
use std::collections::VecDeque;

use crate::game::carddeck::{card::Card, deck::Deck};

use super::events::DrawEvent;

/// The hand [`Resource`], which contains a [`VecDeque`] of the cards within it.
///
/// You'll notice this is very similar to how the deck is defined. The two could possible
/// be merged into a single defition at a later date.
#[derive(Resource, Default, Debug, Reflect)]
pub struct Hand {
    cards: VecDeque<Card>,
}

impl Hand {
    /// Draw a card from the deck and add it to the hand
    #[allow(dead_code)]
    pub fn draw(&mut self, mut deck: ResMut<Deck>) -> Result {
        if let Some(card) = deck.draw() {
            self.cards.push_back(card);
            info!(
                "Cards in hand: {}",
                self.cards
                    .iter()
                    .map(|c| c.name.clone())
                    .collect::<Vec<String>>()
                    .join(", ")
            );
        } else {
            warn!("Attempted to draw from empty deck but not implemented");
        }

        Ok(())
    }
}

#[tracing::instrument(name = "Draw to hand", skip_all)]
pub fn draw_one(
    _trigger: Trigger<DrawEvent>,
    mut hand: ResMut<Hand>,
    deck: ResMut<Deck>,
) -> Result {
    info!("System triggered - draw_one");

    hand.draw(deck)?;

    Ok(())
}
