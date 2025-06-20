//! # Card
//!
//! A card is an instance that can be used to perform actions by the player to affect the
//! game world.
//! These are stored in the player's deck, of which duplicates may exist.

use bevy::{
    color::palettes::tailwind,
    prelude::*,
    render::view::RenderLayers,
    sprite::Anchor,
    text::{FontSmoothing, TextBounds},
};
use rand::{distr::weighted::WeightedIndex, prelude::*};

use crate::{
    game::{
        carddeck::{
            constants::CARD_LAYER,
            markers::{Draggable, Dragged},
        },
        level::assets::LevelAssets,
        mushrooms::{MushroomDefinitions, MushroomType},
    },
    screens::Screen,
};

use super::constants::CARD_SIZE;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Card>();

    app.init_resource::<CardTemplates>();

    app.add_systems(OnEnter(Screen::Gameplay), create_card_definitions);
}

#[derive(Resource, Default, Debug)]
pub struct CardTemplates {
    pub cards: Vec<CardTemplate>,
}

impl CardTemplates {
    #[tracing::instrument(skip_all)]
    pub fn draw_random_card(&self) -> &CardTemplate {
        let rarity_choices = [Rarity::Common, Rarity::Uncommon, Rarity::Rare];
        info!("Rarity choices: {:?}", rarity_choices);
        let rarity_weights = [40, 40, 20];
        info!("Rarity weights: {:?}", rarity_weights);
        let dist = WeightedIndex::new(rarity_weights).unwrap();
        info!("Distribution: {:?}", dist);
        let mut rng = rand::rng();

        let card_rarity = rarity_choices[dist.sample(&mut rng)];
        info!("Chosen rarity: {:?}", card_rarity);
        let card_choices: Vec<&CardTemplate> = self
            .cards
            .iter()
            .filter(|c| c.rarity == card_rarity)
            .collect();
        info!("Available cards: {:?}", card_choices);

        card_choices[rng.random_range(0..card_choices.len())]
    }
}

#[tracing::instrument(name = "Create card definitions", skip_all)]
pub fn create_card_definitions(mut card_templates: ResMut<CardTemplates>) -> Result {
    info!("Creating card templates");
    card_templates.cards = vec![
        CardTemplate {
            name: "Basic".into(),
            mushroom_type: MushroomType::Basic,
            rarity: Rarity::Uncommon,
        },
        CardTemplate {
            name: "Pulse".into(),
            mushroom_type: MushroomType::Pulse,
            rarity: Rarity::Common,
        },
        CardTemplate {
            name: "Sideways".into(),
            mushroom_type: MushroomType::Sideways,
            rarity: Rarity::Common,
        },
        CardTemplate {
            name: "Fork".into(),
            mushroom_type: MushroomType::Fork,
            rarity: Rarity::Common,
        },
        CardTemplate {
            name: "Threeway".into(),
            mushroom_type: MushroomType::Threeway,
            rarity: Rarity::Rare,
        },
        CardTemplate {
            name: "Diagonal".into(),
            mushroom_type: MushroomType::Diagonal,
            rarity: Rarity::Common,
        },
        CardTemplate {
            name: "Surround".into(),
            mushroom_type: MushroomType::Surround,
            rarity: Rarity::Rare,
        },
        CardTemplate {
            name: "Skipper".into(),
            mushroom_type: MushroomType::Skipper,
            rarity: Rarity::Uncommon,
        },
        CardTemplate {
            name: "Deleter".into(),
            mushroom_type: MushroomType::Deleter,
            rarity: Rarity::Uncommon,
        },
        CardTemplate {
            name: "Bomb".into(),
            mushroom_type: MushroomType::Bomb,
            rarity: Rarity::Rare,
        },
        CardTemplate {
            name: "Amplifier".into(),
            mushroom_type: MushroomType::Amplifier,
            rarity: Rarity::Uncommon,
        },
        CardTemplate {
            name: "Twoway_amplifier".into(),
            mushroom_type: MushroomType::TwoWayAmplifier,
            rarity: Rarity::Rare,
        },
        CardTemplate {
            name: "Threeway_Amplifier".into(),
            mushroom_type: MushroomType::ThreeWayAmplifier,
            rarity: Rarity::Rare,
        },
        CardTemplate {
            name: "Chain".into(),
            mushroom_type: MushroomType::Chain,
            rarity: Rarity::Uncommon,
        },
        CardTemplate {
            name: "Burst".into(),
            mushroom_type: MushroomType::Burst,
            rarity: Rarity::Rare,
        },
        CardTemplate {
            name: "Converter".into(),
            mushroom_type: MushroomType::Converter,
            rarity: Rarity::Uncommon,
        },
        CardTemplate {
            name: "Knight".into(),
            mushroom_type: MushroomType::Knight,
            rarity: Rarity::Rare,
        },
        CardTemplate {
            name: "Unblocker".into(),
            mushroom_type: MushroomType::Unblocker,
            rarity: Rarity::Rare,
        },
    ];
    info!("Done creating templates");

    Ok(())
}

#[derive(Debug)]
pub struct CardTemplate {
    pub name: String,
    pub mushroom_type: MushroomType,
    pub rarity: Rarity,
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
#[require(Pickable {is_hoverable: true, should_block_lower: true})]
pub struct Card {
    pub name: String,
    pub mushroom_type: MushroomType,
    pub rarity: Rarity,
    pub origin: Transform,
}

impl From<&CardTemplate> for Card {
    fn from(value: &CardTemplate) -> Self {
        Self {
            name: value.name.clone(),
            mushroom_type: value.mushroom_type,
            rarity: value.rarity,
            origin: Transform::from_translation(Vec3::ZERO),
        }
    }
}

impl Default for Card {
    fn default() -> Self {
        Self {
            name: "Card".into(),
            mushroom_type: MushroomType::Basic,
            rarity: Rarity::Common,
            origin: Transform::from_translation(Vec3::ZERO),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Reflect)]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
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
    pub state_scoped: StateScoped<Screen>,
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
            state_scoped: StateScoped(Screen::Gameplay),
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn spawn_card(
    mut commands: Commands,
    card_component: Card,
    hand_entity: Entity,
    mushroom_definitions: &Res<MushroomDefinitions>,
    level_assets: &Res<LevelAssets>,
    atlas_layout_handle: &Handle<TextureAtlasLayout>,
    asset_server: &Res<AssetServer>,
) -> Result<Entity, BevyError> {
    let mushroom_definition = mushroom_definitions
        .get(card_component.mushroom_type)
        .unwrap();

    let atlas = TextureAtlas {
        layout: atlas_layout_handle.clone(),
        index: mushroom_definition.sprite_row * 2,
    };

    // Mushroom Sprite
    let mut mushroom_sprite =
        Sprite::from_atlas_image(level_assets.mushroom_texture.clone(), atlas);
    mushroom_sprite.anchor = Anchor::BottomCenter;

    // Card Back
    let card_back_image = match card_component.rarity {
        Rarity::Common => level_assets.card_common.clone(),
        Rarity::Uncommon => level_assets.card_uncommon.clone(),
        Rarity::Rare => level_assets.card_rare.clone(),
    };
    let card_back_slice = SpriteImageMode::Sliced(TextureSlicer {
        border: BorderRect::all(4.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 4.0,
    });
    let card_back_sprite = Sprite {
        image: card_back_image,
        custom_size: Some(CARD_SIZE),
        image_mode: card_back_slice,
        ..default()
    };

    // Spawn the Card
    let card_entity = commands
        .spawn(CardBundle {
            name: card_component.name.clone().into(),
            card: card_component.clone(),
            sprite: card_back_sprite,
            ..default()
        })
        .with_children(|commands| {
            commands.spawn((
                CARD_LAYER,
                Anchor::TopCenter,
                Text2d::new(mushroom_definition.name.clone()),
                TextColor(tailwind::STONE_200.into()),
                TextLayout::new(JustifyText::Center, LineBreak::NoWrap),
                TextFont {
                    font: asset_server.load("fonts/PixelOperatorMonoHB.ttf"),
                    font_size: 14.0,
                    font_smoothing: FontSmoothing::AntiAliased,
                    ..default()
                },
                Transform::from_xyz(0.0, (CARD_SIZE.y / 2.0) - 10.0, 1.0),
            ));

            // Mushroom Sprite
            commands.spawn((
                CARD_LAYER,
                mushroom_sprite,
                Transform::from_xyz(0.0, 0.0, 1.0).with_scale(Vec3::splat(3.0)),
                StateScoped(Screen::Gameplay),
            ));

            // Activation Limit
            let card_text = format!(
                "Triggers: {}\n{}",
                mushroom_definition.max_uses_per_turn, mushroom_definition.description,
            );
            commands.spawn((
                CARD_LAYER,
                Anchor::TopCenter,
                Text2d::new(card_text),
                TextColor(tailwind::STONE_200.into()),
                TextBounds::from(Vec2::new(CARD_SIZE.x * 0.8, CARD_SIZE.y / 2.0)),
                TextLayout::new(JustifyText::Left, LineBreak::WordBoundary),
                TextFont {
                    font: asset_server.load("fonts/PixelOperatorMonoHB.ttf"),
                    font_size: 14.0,
                    font_smoothing: FontSmoothing::AntiAliased,
                    ..default()
                },
                Transform::from_xyz(0.0, -10.0, 1.0),
            ));
        })
        .id();

    commands.entity(hand_entity).add_child(card_entity);

    Ok(card_entity)
}
