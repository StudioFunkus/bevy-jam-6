//! # Card
//!
//! A card is an instance that can be used to perform actions by the player to affect the
//! game world.
//! These are stored in the player's deck, of which duplicates may exist.

use bevy::prelude::*;

use crate::game::mushrooms::MushroomType;

#[derive(Resource, Default, Debug, Clone)]
pub struct CardTemplates(pub Vec<Card>);

#[derive(Clone, Debug, Reflect)]
pub struct Card {
    pub name: String,
    pub mushroom_type: MushroomType,
}
