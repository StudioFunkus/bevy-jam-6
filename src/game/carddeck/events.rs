//! # Events
//!
//! Collection of events relating to the deck system

use bevy::prelude::*;

/// Event to fire when a card should be drawn from the deck and into the hand
#[derive(Event, Debug)]
pub struct DrawEvent;
