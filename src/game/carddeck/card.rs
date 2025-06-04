//! # Card
//!
//! A card is an instance that can be used to perform actions by the player to affect the
//! game world.
//! These are stored in the player's deck, of which duplicates may exist.

use bevy::prelude::*;

use crate::game::mushrooms::MushroomType;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Card>();

    app.init_resource::<CardTemplates>();

    app.add_observer(on_card_drag);
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

#[derive(Component)]
pub struct Draggable;

#[derive(Bundle)]
pub struct CardBundle {
    pub name: Name,
    pub card: Card,
    pub transform: Transform,
    pub sprite: Sprite,
    pub draggable: Draggable,
}

impl Default for CardBundle {
    fn default() -> Self {
        Self {
            name: "Card".into(),
            card: Card::default(),
            transform: Transform::default(),
            sprite: Sprite::default(),
            draggable: Draggable,
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn on_card_drag(
    mut trigger: Trigger<Pointer<Drag>>,
    mut card_transform: Query<&mut Transform, (With<Draggable>, With<Card>)>,
) {
    trigger.propagate(false);
    if let Ok(mut card_transform) = card_transform.get_mut(trigger.target) {
        card_transform.translation.x += trigger.delta.x;
        card_transform.translation.y -= trigger.delta.y;
    }
}
