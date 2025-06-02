use bevy::prelude::*;

use super::MushroomType;

/// Currently selected mushroom type for placement
#[derive(Resource, Default)]
pub struct SelectedMushroomType {
    pub mushroom_type: MushroomType,
}
