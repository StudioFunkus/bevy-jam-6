//! # Hand
//!
//! The hand contains the cards that have been drawn and are currently playable by the player.
//! These are drawn from the deck.

use bevy::prelude::*;
use std::collections::VecDeque;

use crate::{
    game::{
        carddeck::{
            card::{Card, CardTemplates, spawn_card},
            constants::{CARD_LAYER, CARD_SPACING},
            events::{DrawEvent, HandChangeEvent},
            markers::Dragged,
        },
        level::assets::LevelAssets,
        mushrooms::MushroomDefinitions,
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Hand>();

    app.init_resource::<Hand>();

    app.add_systems(OnEnter(Screen::Gameplay), spawn_hand_entity);

    app.add_observer(update_card_origins).add_observer(draw_n);
}

fn spawn_hand_entity(mut commands: Commands, window: Query<&Window>) -> Result {
    let window = window.single()?;

    commands.spawn((
        Name::from("Hand"),
        HandEntity,
        Transform::from_xyz(0.0, -(0.9 * (window.height() / 2.0)), 0.0),
        CARD_LAYER,
        Visibility::Visible,
        StateScoped(Screen::Gameplay),
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
    card_templates: Res<CardTemplates>,
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

    for _ in 0..cards_to_draw {
        let drawn_card = card_templates.draw_random_card();

        let card_component = Card::from(drawn_card);
        let card_entity = spawn_card(
            commands.reborrow(),
            card_component.clone(),
            hand_entity,
            &mushroom_definitions,
            &level_assets,
            &atlas_layout_handle,
        )?;

        hand.cards
            .push_back((card_component, Some(card_entity.clone())));
    }

    commands.trigger(HandChangeEvent);

    Ok(())
}

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
