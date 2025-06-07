//! # Card
//!
//! A card is an instance that can be used to perform actions by the player to affect the
//! game world.
//! These are stored in the player's deck, of which duplicates may exist.

use bevy::{prelude::*, render::view::RenderLayers};

use crate::game::{
    carddeck::{
        constants::CARD_LAYER,
        markers::{Draggable, Dragged},
    },
    mushrooms::MushroomType,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Card>();

    app.init_resource::<CardTemplates>();
}

#[derive(Resource, Default, Debug, Clone)]
pub struct CardTemplates(pub Vec<Card>);

#[derive(Component, Clone, Debug, Reflect)]
#[require(Pickable {is_hoverable: true, should_block_lower: true})]
pub struct Card {
    pub name: String,
    pub mushroom_type: MushroomType,
    pub origin: Transform,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            name: "Card".into(),
            mushroom_type: MushroomType::Basic,
            origin: Transform::default(),
        }
    }
}

#[derive(Bundle)]
pub struct CardBundle {
    pub name: Name,
    pub card: Card,
    pub transform: Transform,
    pub sprite: Sprite,
    pub draggable: Draggable,
    pub render_layer: RenderLayers,
    pub dragged: Dragged,
}

impl Default for CardBundle {
    fn default() -> Self {
        Self {
            name: "Card".into(),
            card: Card::default(),
            transform: Transform::default(),
            sprite: Sprite::default(),
            draggable: Draggable,
            render_layer: CARD_LAYER,
            dragged: Dragged::Released,
        }
    }
}

#[tracing::instrument(name = "Create card definitions", skip_all)]
pub fn create_card_definitions(mut card_templates: ResMut<CardTemplates>) -> Result {
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

    Ok(())
}
