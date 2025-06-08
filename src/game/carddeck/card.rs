//! # Card
//!
//! A card is an instance that can be used to perform actions by the player to affect the
//! game world.
//! These are stored in the player's deck, of which duplicates may exist.

use bevy::{
    color::palettes::tailwind, prelude::*, render::view::RenderLayers, sprite::Anchor,
    text::TextBounds,
};

use crate::game::{
    carddeck::{
        constants::CARD_LAYER,
        markers::{Draggable, Dragged},
    },
    level::assets::LevelAssets,
    mushrooms::{MushroomDefinitions, MushroomType},
};

use super::constants::CARD_SIZE;

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

#[tracing::instrument(skip_all)]
pub fn spawn_card(
    mut commands: Commands,
    card_component: Card,
    hand_entity: Entity,
    mushroom_definitions: &Res<MushroomDefinitions>,
    level_assets: &Res<LevelAssets>,
    atlas_layout_handle: &Handle<TextureAtlasLayout>,
) -> Result<Entity, BevyError> {
    let mushroom_definition = mushroom_definitions
        .get(card_component.mushroom_type)
        .unwrap();

    let atlas = TextureAtlas {
        layout: atlas_layout_handle.clone(),
        index: mushroom_definition.sprite_row * 2,
    };

    let mushroom_sprite = Sprite::from_atlas_image(level_assets.mushroom_texture.clone(), atlas);

    let card_entity = commands
        .spawn(CardBundle {
            name: card_component.name.clone().into(),
            card: card_component.clone(),
            sprite: Sprite {
                color: tailwind::STONE_800.into(),
                custom_size: Some(CARD_SIZE),
                ..default()
            },
            ..default()
        })
        .with_children(|commands| {
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
        })
        .id();

    commands.entity(hand_entity).add_child(card_entity);

    Ok(card_entity)
}
