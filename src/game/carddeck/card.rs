//! # Card
//!
//! A card is an instance that can be used to perform actions by the player to affect the
//! game world.
//! These are stored in the player's deck, of which duplicates may exist.

use std::time::Duration;

use bevy::{prelude::*, render::view::RenderLayers};
use bevy_tweening::{Animator, Tween, TweenCompleted, lens::TransformPositionLens};

use crate::{CARD_LAYER, game::mushrooms::MushroomType};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Card>();

    app.init_resource::<CardTemplates>();

    app.add_observer(on_card_drag)
        .add_observer(on_card_darg_start)
        .add_observer(on_card_drag_end)
        .add_observer(on_card_move_done);

    app.add_systems(Update, move_cards_back_to_origin);
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

/// A marker component to indicate that a card can be dragged
#[derive(Component)]
pub struct Draggable;

/// A marker component to indicate that a card is being dragged
#[derive(Component, Debug, Default, PartialEq, Eq)]
pub enum Dragged {
    #[default]
    Active,
    Released,
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

impl From<Card> for CardBundle {
    fn from(value: Card) -> Self {
        Self {
            name: value.mushroom_type.name().into(),
            card: value,
            transform: Transform::default().with_scale(Vec3::new(15.0, 20.0, 0.0)),
            ..default()
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn on_card_drag(
    mut trigger: Trigger<Pointer<Drag>>,
    mut card_transform: Query<&mut Transform, (With<Draggable>, With<Card>)>,
) -> Result {
    trigger.propagate(false);
    if let Ok(mut card_transform) = card_transform.get_mut(trigger.target) {
        card_transform.translation.x += trigger.delta.x;
        card_transform.translation.y -= trigger.delta.y;
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn on_card_darg_start(
    mut trigger: Trigger<Pointer<DragStart>>,
    mut commands: Commands,
    cards_being_dragged: Query<&Dragged, With<Card>>,
) -> Result {
    trigger.propagate(false);

    info!("{}", cards_being_dragged.iter().len());

    // Abort if another card is being dragged
    for dragged_component in cards_being_dragged {
        match dragged_component {
            Dragged::Active => return Ok(()),
            _ => continue,
        }
    }

    commands.entity(trigger.target).insert(Dragged::Active);

    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn on_card_drag_end(
    trigger: Trigger<Pointer<DragEnd>>,
    mut cards_being_dragged: Query<&mut Dragged, With<Card>>,
) -> Result {
    if let Ok(mut dragged_component) = cards_being_dragged.get_mut(trigger.target) {
        *dragged_component = Dragged::Released;
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn move_cards_back_to_origin(
    mut commands: Commands,
    mut cards_being_dragged: Query<(Entity, &Transform, &Card, &Dragged), Changed<Dragged>>,
) -> Result {
    for (card_entity, card_transform, card_component, dragged_component) in
        cards_being_dragged.iter_mut()
    {
        match dragged_component {
            Dragged::Active => {
                continue;
            }
            Dragged::Released => {
                create_tween_for_card(
                    commands.reborrow(),
                    card_entity,
                    card_component,
                    card_transform,
                )?;
            }
        }
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
fn create_tween_for_card(
    mut commands: Commands,
    card_entity: Entity,
    card_component: &Card,
    card_transform: &Transform,
) -> Result {
    let move_tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs(1),
        TransformPositionLens {
            start: card_transform.translation,
            end: card_component.origin.translation,
        },
    )
    .with_completed_event(1);

    commands
        .entity(card_entity)
        .insert(Animator::new(move_tween));

    Ok(())
}

#[tracing::instrument(skip_all)]
fn on_card_move_done(trigger: Trigger<TweenCompleted>, mut commands: Commands) -> Result {
    commands.entity(trigger.target()).remove::<Dragged>();
    commands
        .entity(trigger.target())
        .remove::<Animator<Transform>>();

    Ok(())
}
