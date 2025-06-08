//! # Hand
//!
//! The hand contains the cards that have been drawn and are currently playable by the player.
//! These are drawn from the deck.

use bevy::{color::palettes::tailwind, prelude::*, sprite::Anchor, text::TextBounds};
use std::collections::VecDeque;

use crate::{
    game::{
        carddeck::{
            card::{Card, CardBundle, spawn_card},
            constants::{CARD_LAYER, CARD_SIZE, CARD_SPACING},
            deck::Deck,
            events::{DrawEvent, HandChangeEvent},
            markers::Dragged,
        },
        level::assets::LevelAssets,
        mushrooms::{MushroomDefinitions, MushroomType},
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
        Name::from("Hand"),
        HandEntity,
        Transform::from_xyz(0.0, -(0.9 * (window.height() / 2.0)), 0.0),
        CARD_LAYER,
        Visibility::Visible,
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

    /// Despawn a card with the given entity
    pub fn despawn_card(&mut self, mut commands: Commands, card_entity: Entity) -> Result {
        for (index, (_, entity)) in self.cards.iter().enumerate() {
            if *entity == Some(card_entity) {
                commands.entity(card_entity).despawn();
                self.cards.remove(index);
                commands.trigger(HandChangeEvent);
                return Ok(());
            }
        }

        Ok(())
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
    mushroom_definitions: Res<MushroomDefinitions>,
    level_assets: Res<LevelAssets>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut local_atlas_layout: Local<Option<Handle<TextureAtlasLayout>>>,
) -> Result {
    // Create the atlas layout if none
    let atlas_layout_handle = local_atlas_layout.clone().unwrap_or_else(|| {
        let new_handle = atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(16, 16),
            2,
            24,
            Some(UVec2::new(2, 2)),
            None,
        ));
        *local_atlas_layout = Some(new_handle.clone());

        new_handle
    });

    let hand_entity = hand_entity.single()?;

    let mut cards_to_draw = trigger.0;

    // Check we can fit the cards, otherwise draw less
    if hand.cards.len() as u32 + cards_to_draw > hand.max_cards as u32 {
        cards_to_draw = (hand.max_cards - hand.cards.len()) as u32;
        info!("Cannot fit cards, will draw {}", cards_to_draw);
    }

    // Check the deck has enough cards, otherwise draw less
    if cards_to_draw > deck.get_card_count() as u32 {
        cards_to_draw = deck.get_card_count() as u32;
        info!("Not enough cards in deck, will draw {}", cards_to_draw);
    }

    for _ in 0..cards_to_draw {
        let Some(drawn_card) = deck.draw() else {
            break;
        };

        let card_entity = spawn_card(
            commands.reborrow(),
            drawn_card.clone(),
            hand_entity,
            &mushroom_definitions,
            &level_assets,
            &atlas_layout_handle,
        )?;

        hand.cards
            .push_back((drawn_card, Some(card_entity.clone())));
    }

    commands.trigger(HandChangeEvent);

    Ok(())
}

// /// Spawn a card, adding the [`Entity`] and [`Card`] component to the [`Hand`] resource.
// ///
// /// This ensures that the [`Hand`] resource is kept up-to-date with changes.
// #[tracing::instrument(skip_all)]
// fn spawn_card(mut commands: Commands, card: Card, hand_entity: Entity) -> Entity {
//     let card_color = card.mushroom_type.color().clone();
//     let card_entity = commands
//         .spawn(CardBundle {
//             name: card.name.clone().into(),
//             card: card,
//             transform: Transform::default().with_scale(Vec3::new(15.0, 20.0, 1.0)),
//             sprite: Sprite::from_color(card_color, Vec2::new(3.0, 5.0)),
//             ..default()
//         })
//         .id();
//     commands.entity(hand_entity).add_child(card_entity);

//     card_entity
// }

/// Update the value of the origin property on a [`Card`] component.
///
/// Triggered via [`HandChangeEvent`], which is fired whenever a [`Card`] component is added
/// to or removed from the [`Hand`].
#[tracing::instrument(skip_all)]
fn update_card_origins(
    _: Trigger<HandChangeEvent>,
    mut commands: Commands,
    hand: Res<Hand>,
    mut cards_query: Query<(&mut Card, &Transform)>,
) -> Result {
    info!("==========");
    info!("Updating card origins in hand");
    info!("==========");
    let number_of_cards: f32 = hand.get_card_count() as f32;
    debug!("Number of cards: {}", number_of_cards);
    debug!("Using spacing: {}", CARD_SPACING);

    let first_card_offset: f32 = ((number_of_cards - 1.0) / 2.0) * CARD_SPACING;

    for (index, card_tuple) in hand.cards.iter().enumerate() {
        if let (_, Some(entity)) = card_tuple {
            let (mut card_component, card_transform) = cards_query.get_mut(*entity)?;
            let new_origin = card_component
                .origin
                .translation
                .with_x(-first_card_offset + (index as f32 * CARD_SPACING))
                .with_z((index + 1) as f32);
            debug!("Card at index {} will have offset {}", index, new_origin.x);

            card_component.origin.translation = new_origin;

            // Also set the origin's scale for the hover system
            card_component.origin.scale = card_transform.scale;

            commands.entity(*entity).insert(Dragged::Released);
        }
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
fn create_test_card(
    mut commands: Commands,
    mushroom_definitions: Res<MushroomDefinitions>,
    hand_query: Query<Entity, With<HandEntity>>,
    level_assets: Res<LevelAssets>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut atlas_layout_handle: Local<Option<Handle<TextureAtlasLayout>>>,
) -> Result {
    let mushroom_type = MushroomType::Chain;
    let mushroom_definition = mushroom_definitions.get(MushroomType::Chain).unwrap();

    let test_card_component = Card {
        mushroom_type: mushroom_type,
        name: mushroom_definition.name.clone(),
        origin: Transform::from_translation(Vec3::ZERO.with_z(10.0)),
    };

    let hand = hand_query.single()?;

    commands.entity(hand).with_children(|commands| {
        commands
            .spawn(CardBundle {
                name: mushroom_definition.name.clone().into(),
                card: test_card_component,
                sprite: Sprite {
                    color: tailwind::STONE_800.into(),
                    custom_size: Some(CARD_SIZE),
                    ..default()
                },
                ..default()
            })
            .with_children(|commands| {
                let atlas_layout_handle = atlas_layout_handle.clone().unwrap_or_else(|| {
                    info!("No layout yet, creating");
                    let new_handle = atlas_layouts.add(TextureAtlasLayout::from_grid(
                        UVec2::new(16, 16),
                        2,
                        24,
                        Some(UVec2::new(2, 2)),
                        None,
                    ));
                    *atlas_layout_handle = Some(new_handle.clone());

                    new_handle
                });

                let atlas = TextureAtlas {
                    layout: atlas_layout_handle,
                    index: mushroom_definition.sprite_row * 2,
                };
                let mushroom_sprite =
                    Sprite::from_atlas_image(level_assets.mushroom_texture.clone(), atlas);

                commands.spawn((
                    CARD_LAYER,
                    mushroom_sprite,
                    Transform::from_xyz(0.0, CARD_SIZE.y / 4.0, 2.0).with_scale(Vec3::splat(3.0)),
                ));

                commands.spawn((
                    CARD_LAYER,
                    Anchor::Center,
                    Text2d::new(mushroom_definition.description.clone()),
                    TextColor(tailwind::STONE_200.into()),
                    TextBounds::from(Vec2::new(CARD_SIZE.x * 0.9, CARD_SIZE.y / 2.0)),
                    TextLayout::new_with_linebreak(LineBreak::WordBoundary),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    Transform::from_xyz(0.0, -(CARD_SIZE.y / 4.0), 1.0),
                ));
            });
    });

    Ok(())
}
