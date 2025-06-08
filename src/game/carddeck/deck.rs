//! # Deck
//!
//! A deck is an ordered collection of cards that can be drawn from, added to, shuffled, etc.
//! stored as a resource in the world when active.

use bevy::prelude::*;
use rand::{rng, seq::SliceRandom};
use std::collections::VecDeque;

use crate::game::carddeck::card::Card;

pub(super) fn plugin(app: &mut App) {
    // app.register_type::<Deck>();

    // app.init_resource::<Deck>();
}

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
        let mut rng = rng();
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
        self.cards.pop_front()
    }

    #[allow(dead_code)]
    #[tracing::instrument(name = "from deck", skip_all)]
    pub fn draw_n(&mut self, n: u32) -> VecDeque<Card> {
        let mut drawn_cards = VecDeque::new();

        for _ in 0..n {
            if let Some(card) = self.cards.pop_front() {
                drawn_cards.push_back(card);
            }
        }

        drawn_cards
    }

    /// Add a card to the bottom of the deck
    #[allow(dead_code)]
    #[tracing::instrument(name = "Adding card to deck", skip_all)]
    pub fn add_to_bottom(&mut self, card: Card) -> Result {
        self.cards.push_back(card);

        Ok(())
    }

    /// Get count of remaining cards
    #[allow(dead_code)]
    pub fn get_card_count(&self) -> usize {
        self.cards.len()
    }

    /// Empty this deck
    #[tracing::instrument(skip_all)]
    pub fn empty_deck(&mut self) -> Result {
        self.cards.drain(..);

        Ok(())
    }

    // /// Refresh this deck from the original deck
    // #[tracing::instrument(skip_all)]
    // pub fn refresh_deck(&mut self, template_deck: Res<Deck>) -> Result {
    //     self.cards = template_deck.cards.clone();
    //     self.shuffle()?;

    //     Ok(())
    // }
}

// /// Empty the deck
// #[tracing::instrument(skip_all)]
// pub fn empty_active_deck(mut deck: ResMut<Deck>) -> Result {
//     deck.empty_deck()?;

//     Ok(())
// }
