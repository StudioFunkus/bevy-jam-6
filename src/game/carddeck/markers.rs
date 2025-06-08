use bevy::prelude::*;

/// A marker component to indicate that a card can be dragged
#[derive(Component)]
pub struct Draggable;

/// A marker component to indicate that a card is being dragged
#[derive(Component, Debug, Default, PartialEq, Eq)]
pub enum Dragged {
    #[default]
    Active,
    Released,
    Played,
}

/// A marker component to indicate that a card is being hovered
#[derive(Component)]
pub struct Hovered;
