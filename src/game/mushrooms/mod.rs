
use bevy::prelude::*;
use resources::SelectedMushroomType;
use trigger::TriggerQueue;

mod events;
mod trigger;
pub(crate) mod resources;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        events::plugin,
    ));

    app.init_resource::<SelectedMushroomType>();
    app.init_resource::<TriggerQueue>();
}

/// Different types of mushrooms
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
#[reflect(Component)]
pub enum MushroomType {
    #[default]
    Basic,
    Pulse,
}

impl MushroomType {
    pub fn cost(&self) -> f64 {
        match self {
            MushroomType::Basic => 10.0,
            MushroomType::Pulse => 5.0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            MushroomType::Basic => "Button Mushroom",
            MushroomType::Pulse => "Pulse Mushroom",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            MushroomType::Basic => Color::srgb(0.5, 0.3, 0.1),
            MushroomType::Pulse => Color::srgb(0.2, 0.8, 0.2),
        }
    }
    
    pub fn cooldown_time(&self) -> f32 {
        match self {
            MushroomType::Basic => 0.5,
            MushroomType::Pulse => 2.0,
        }
    }
    
    pub fn base_production(&self) -> f64 {
        match self {
            MushroomType::Basic => 10.0,
            MushroomType::Pulse => 2.0,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            MushroomType::Basic => "Produces spores when clicked.",
            MushroomType::Pulse => "Triggers an adjacent mushroom.",
        }
    }
}

/// Marker component for mushrooms
#[derive(Component)]
pub struct Mushroom;

/// Facing direction for mushrooms
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

