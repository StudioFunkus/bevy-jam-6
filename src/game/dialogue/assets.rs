//! Dialogue asset loading

use bevy::prelude::*;
use funkus_dialogue_core::DialogueAsset;

use crate::asset_tracking::LoadResource;

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<DialogueAssets>();
    app.load_resource::<DialogueAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct DialogueAssets {
    // Intro dialogues for specific levels
    #[dependency]
    pub level_1_intro: Handle<DialogueAsset>,
    #[dependency]
    pub level_2_intro: Handle<DialogueAsset>,
    #[dependency]
    pub final_level_intro: Handle<DialogueAsset>,

    // Pool of success dialogues
    #[dependency]
    pub success_dialogues: Vec<Handle<DialogueAsset>>,

    // Pool of failure dialogues
    #[dependency]
    pub failure_dialogues: Vec<Handle<DialogueAsset>>,

    // Portraits for characters
    #[dependency]
    pub portraits: PortraitAssets,
}

#[derive(Asset, Clone, Reflect)]
pub struct PortraitAssets {
    #[dependency]
    pub wizard: Handle<Image>,
    #[dependency]
    pub witch: Handle<Image>,
}

impl FromWorld for DialogueAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        Self {
            level_1_intro: asset_server.load("dialogues/intros/level_1.dialogue.json"),
            level_2_intro: asset_server.load("dialogues/intros/level_2.dialogue.json"),
            final_level_intro: asset_server.load("dialogues/intros/final_level.dialogue.json"),

            success_dialogues: vec![asset_server.load("dialogues/success/success_1.dialogue.json")],

            failure_dialogues: vec![asset_server.load("dialogues/failure/failure_1.dialogue.json")],

            portraits: PortraitAssets {
                wizard: asset_server.load("images/portraits/wizard.png"),
                witch: asset_server.load("images/portraits/witch.png"),
            },
        }
    }
}
