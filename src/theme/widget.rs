//! Helper functions for creating common widgets.

use std::borrow::Cow;

use bevy::{
    ecs::{spawn::SpawnWith, system::IntoObserverSystem},
    prelude::*,
    ui::Val::*,
};

use crate::theme::{assets::ThemeAssets, interaction::InteractionPalette, palette::*};

/// A root UI node that fills the window and centers its content.
pub fn ui_root(name: impl Into<Cow<'static, str>>, font: Option<Handle<Font>>) -> impl Bundle {
    if let Some(font) = font {
        (
            Name::new(name),
            Node {
                position_type: PositionType::Absolute,
                width: Percent(100.0),
                height: Percent(100.0),
                padding: UiRect {
                    left: Px(10.),
                    right: Px(50.),
                    top: Px(10.),
                    bottom: Px(10.),
                },
                align_items: AlignItems::End,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Px(20.0),
                ..default()
            },
            TextFont {
                font: font.clone(),
                font_size: 40.0,
                ..default()
            },
            // Don't block picking events for other UI roots.
            Pickable::IGNORE,
        )
    } else {
        (
            Name::new(name),
            Node {
                position_type: PositionType::Absolute,
                width: Percent(100.0),
                height: Percent(100.0),
                padding: UiRect {
                    left: Px(10.),
                    right: Px(50.),
                    top: Px(10.),
                    bottom: Px(10.),
                },
                align_items: AlignItems::End,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Px(20.0),
                ..default()
            },
            TextFont::from_font_size(40.0),
            // Don't block picking events for other UI roots.
            Pickable::IGNORE,
        )
    }
}

/// A simple header label. Bigger than [`label`].
pub fn header(text: impl Into<String>, font: Option<Handle<Font>>) -> impl Bundle {
    if let Some(font) = font {
        (
            Name::new("Header"),
            Text(text.into()),
            TextFont {
                font: font.clone(),
                font_size: 40.0,
                ..default()
            },
            TextColor(HEADER_TEXT),
        )
    } else {
        (
            Name::new("Header"),
            Text(text.into()),
            TextFont::from_font_size(40.0),
            TextColor(HEADER_TEXT),
        )
    }
}

/// A simple text label.
pub fn label(text: impl Into<String>, font: Option<Handle<Font>>) -> impl Bundle {
    if let Some(font) = font {
        (
            Name::new("Label"),
            Text(text.into()),
            TextFont {
                font: font.clone(),
                font_size: 24.0,
                ..default()
            },
            TextColor(LABEL_TEXT),
        )
    } else {
        (
            Name::new("Label"),
            Text(text.into()),
            TextFont::from_font_size(24.0),
            TextColor(LABEL_TEXT),
        )
    }
}

/// A large rounded button with text and an action defined as an [`Observer`].
pub fn button<E, B, M, I>(text: impl Into<String>, action: I) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        action,
        (
            Node {
                width: Px(250.0),
                height: Px(80.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderRadius::new(Val::Px(10.), Val::Px(10.), Val::Px(10.), Val::Px(10.)),
        ),
    )
}

/// A small square button with text and an action defined as an [`Observer`].
pub fn button_small<E, B, M, I>(text: impl Into<String>, action: I) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        action,
        Node {
            width: Px(30.0),
            height: Px(30.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )
}

/// A simple button with text and an action defined as an [`Observer`]. The button's layout is provided by `button_bundle`.
fn button_base<E, B, M, I>(
    text: impl Into<String>,
    action: I,
    button_bundle: impl Bundle,
) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into();
    let action = IntoObserverSystem::into_system(action);
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    BackgroundColor(BUTTON_BACKGROUND),
                    InteractionPalette {
                        none: BUTTON_BACKGROUND,
                        hovered: BUTTON_HOVERED_BACKGROUND,
                        pressed: BUTTON_PRESSED_BACKGROUND,
                    },
                    children![(
                        Name::new("Button Text"),
                        Text(text),
                        TextFont::from_font_size(40.0),
                        TextColor(Color::WHITE),
                        // Don't bubble picking events from the text up to the button.
                        Pickable::IGNORE,
                    )],
                ))
                .insert(button_bundle)
                .observe(action);
        })),
    )
}

/// A simple button with text and an action defined as an [`Observer`]. The button's layout is provided by `button_bundle`.
pub fn image(
    handle: Handle<Image>,
    left: Option<Val>,
    right: Option<Val>,
    top: Option<Val>,
    bottom: Option<Val>,
    width: Val,
    height: Val,
    position_type: PositionType,
) -> impl Bundle {
    (
        Name::new("UI_image"),
        Node {
            left: left.unwrap_or_default(),
            right: right.unwrap_or_default(),
            top: top.unwrap_or_default(),
            bottom: bottom.unwrap_or_default(),
            width: width,
            height: height,
            position_type: position_type,
            ..default()
        },
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent.spawn((
                Name::new("Image Inner"),
                Pickable::IGNORE,
                ImageNode::new(handle),
            ));
        })),
    )
}

pub fn slice_1_slicer() -> TextureSlicer {
    TextureSlicer {
        // Adjust these values based on your button texture's border size
        border: BorderRect::all(16.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    }
}

pub fn slice_2_slicer() -> TextureSlicer {
    TextureSlicer {
        // Adjust these values based on your button texture's border size
        border: BorderRect::all(16.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    }
}

/// A large rounded button with nine-slice texture
pub fn button_sliced<E, B, M, I>(
    text: impl Into<String>,
    action: I,
    slice_handle: Handle<Image>,
    slicer: TextureSlicer,
    font: Handle<Font>,
) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base_sliced(
        text,
        action,
        slice_handle,
        Node {
            width: Px(250.0),
            height: Px(80.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        slicer,
        font,
    )
}

/// Base function for sliced buttons
fn button_base_sliced<E, B, M, I>(
    text: impl Into<String>,
    action: I,
    slice_handle: Handle<Image>,
    node_bundle: Node,
    slicer: TextureSlicer,
    font: Handle<Font>,
) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into();
    let action = IntoObserverSystem::into_system(action);

    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    ImageNode {
                        image: slice_handle.clone(),
                        image_mode: NodeImageMode::Sliced(slicer.clone()),
                        color: Color::WHITE,
                        ..default()
                    },
                    node_bundle,
                    children![(
                        Name::new("Button Text"),
                        Text(text),
                        TextFont {
                            font: font.clone(),
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Pickable::IGNORE,
                    )],
                ))
                .observe(action);
        })),
    )
}
