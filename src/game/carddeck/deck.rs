//! # Deck
//!
//! A deck is an ordered collection of cards that can be drawn from, added to, shuffled, etc.
//! stored as a resource in the world when active.

use bevy::prelude::*;
use rand::{seq::SliceRandom, thread_rng};
use std::collections::VecDeque;

use crate::game::carddeck::card::Card;

#[derive(Resource, Default, Debug, Reflect)]
pub struct Deck {
    cards: VecDeque<Card>,
}

impl Deck {
    /// Shuffle the deck.
    ///
    /// Since a [`VecDeque`] cannot be shuffled, the `make_contiguous` method
    /// must first be used, with the return shuffled.
    ///
    /// See [`VecDeque::make_contiguous`] for details.
    #[allow(dead_code)]
    pub fn shuffle(&mut self) -> Result {
        let mut rng = thread_rng();
        self.cards.make_contiguous().shuffle(&mut rng);

        Ok(())
    }

    /// Draw the top card from the deck
    ///
    /// This method operates using [`VecDeque::pop_front`], which will
    /// return an [`Option<T>`] if there are no cards in the deck.
    /// It is left to the caller of this method to handle this situation
    /// as they see fit.
    #[allow(dead_code)]
    #[tracing::instrument(name = "from deck", skip_all)]
    pub fn draw(&mut self) -> Option<Card> {
        info!("Card being drawn from deck");
        let card = self.cards.pop_front();

        info!(
            "Cards remaining in deck: {}",
            self.cards
                .iter()
                .map(|card| card.name.clone())
                .collect::<Vec<String>>()
                .join(", ")
        );

        card
    }

    /// Add a card to the bottom of the deck
    #[allow(dead_code)]
    #[tracing::instrument(name = "Adding card to deck", skip_all)]
    pub fn add_to_bottom(&mut self, card: Card) -> Result {
        info!("Adding card '{}' to bottom of deck", card.name);
        self.cards.push_back(card);

        Ok(())
    }

    /// Get count of remaining cards
    #[allow(dead_code)]
    pub fn get_card_count(&self) -> usize {
        self.cards.len()
    }
}
