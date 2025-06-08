use bevy::prelude::*;

use super::MushroomType;

/// Currently selected mushroom type for placement
#[derive(Resource, Default, Debug)]
pub struct SelectedMushroomType {
    pub mushroom_type: Option<MushroomType>,
}
