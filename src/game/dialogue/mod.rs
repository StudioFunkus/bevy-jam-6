//! Loading and managing dialogue assets

use bevy::prelude::*;
use funkus_dialogue_core::{
    AdvanceDialogue, DialogueAsset, DialogueEnded, DialogueNode, DialogueRunner, DialogueState,
    SelectDialogueChoice, StartDialogue as StartDialogueEvent,
};
use rand::prelude::*;
use rand::rng;

use crate::game::{
    dialogue::assets::DialogueAssets,
    game_flow::{CurrentLevel, LevelState},
};

pub mod assets;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(assets::plugin);

    // Systems for dialogue states
    app.add_systems(OnEnter(LevelState::StartDialogue), enter_start_dialogue);
    app.add_systems(OnEnter(LevelState::EndDialogue), enter_end_dialogue);

    // Update systems during dialogue
    app.add_systems(
        Update,
        (
            handle_dialogue_input,
            handle_start_dialogue_end,
            update_dialogue_portrait,
            handle_end_dialogue_end,
        )
            .run_if(in_state(LevelState::StartDialogue).or(in_state(LevelState::EndDialogue))),
    );
}

/// Handle keyboard input during dialogue
fn handle_dialogue_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    dialogue_query: Query<(Entity, &DialogueRunner)>,
    mut advance_events: EventWriter<AdvanceDialogue>,
    mut select_events: EventWriter<SelectDialogueChoice>,
) {
    for (entity, runner) in dialogue_query.iter() {
        // Skip if dialogue is not active
        if runner.state == DialogueState::Inactive {
            continue;
        }

        // Space to advance dialogue
        if keyboard.just_pressed(KeyCode::Space) {
            match runner.state {
                DialogueState::ShowingText => {
                    advance_events.write(AdvanceDialogue { entity });
                }
                DialogueState::ChoiceSelected(_) => {
                    advance_events.write(AdvanceDialogue { entity });
                }
                _ => {}
            }
        }

        // Number keys for choices
        if runner.state == DialogueState::WaitingForChoice
            || matches!(runner.state, DialogueState::ChoiceSelected(_))
        {
            for i in 0..9 {
                let key = match i {
                    0 => KeyCode::Digit1,
                    1 => KeyCode::Digit2,
                    2 => KeyCode::Digit3,
                    3 => KeyCode::Digit4,
                    4 => KeyCode::Digit5,
                    5 => KeyCode::Digit6,
                    6 => KeyCode::Digit7,
                    7 => KeyCode::Digit8,
                    8 => KeyCode::Digit9,
                    _ => unreachable!(),
                };

                if keyboard.just_pressed(key) {
                    select_events.write(SelectDialogueChoice {
                        entity,
                        choice_index: i,
                    });
                    break;
                }
            }
        }
    }
}

/// Start the level intro dialogue
fn enter_start_dialogue(
    mut commands: Commands,
    dialogue_assets: Res<DialogueAssets>,
    current_level: Res<CurrentLevel>,
    level_definitions: Res<crate::game::level::definitions::LevelDefinitions>,
    mut start_dialogue_events: EventWriter<StartDialogueEvent>,
    mut level_state: ResMut<NextState<LevelState>>,
) {
    // Determine if this level has an intro dialogue
    let total_levels = level_definitions.levels.len();
    let dialogue_handle = match current_level.level_index {
        0 => Some(dialogue_assets.level_1_intro.clone()), // Level 1
        1 => Some(dialogue_assets.level_2_intro.clone()), // Level 2
        n if n == total_levels - 1 => Some(dialogue_assets.final_level_intro.clone()), // Last level
        _ => None,                                        // Other levels have no intro
    };

    if let Some(handle) = dialogue_handle {
        info!(
            "Starting intro dialogue for level {}",
            current_level.level_index + 1
        );

        // Create dialogue entity
        let dialogue_entity = commands
            .spawn((
                Name::new("Level Start Dialogue"),
                DialogueRunner::default(),
                StateScoped(LevelState::StartDialogue),
            ))
            .id();

        // Spawn UI
        spawn_dialogue_ui(&mut commands, LevelState::StartDialogue);

        // Start dialogue
        start_dialogue_events.write(StartDialogueEvent {
            entity: dialogue_entity,
            dialogue_handle: handle,
        });
    } else {
        // No intro dialogue for this level, skip straight to playing
        info!(
            "No intro dialogue for level {}, starting gameplay",
            current_level.level_index + 1
        );
        level_state.set(LevelState::Playing);
    }
}

/// Transition to playing when start dialogue ends
fn handle_start_dialogue_end(
    mut dialogue_ended_events: EventReader<DialogueEnded>,
    mut level_state: ResMut<NextState<LevelState>>,
    current_state: Res<State<LevelState>>,
) {
    if *current_state.get() != LevelState::StartDialogue {
        return;
    }

    for _ in dialogue_ended_events.read() {
        info!("Start dialogue ended, transitioning to gameplay");
        level_state.set(LevelState::Playing);
    }
}

/// Start the level outro dialogue
fn enter_end_dialogue(
    mut commands: Commands,
    dialogue_assets: Res<DialogueAssets>,
    current_level: Res<CurrentLevel>,
    mut start_dialogue_events: EventWriter<StartDialogueEvent>,
) {
    info!("Starting level outro dialogue");

    // Select random dialogue based on success/failure
    let dialogue_pool = match current_level.level_completed_successfully {
        Some(true) => &dialogue_assets.success_dialogues,
        Some(false) => &dialogue_assets.failure_dialogues,
        None => {
            warn!("No completion status set, defaulting to failure dialogue");
            &dialogue_assets.failure_dialogues
        }
    };

    // Pick a random dialogue from the pool
    let dialogue_handle = if dialogue_pool.is_empty() {
        error!("No dialogues in pool!");
        return;
    } else {
        let index = rng().random_range(0..dialogue_pool.len());
        dialogue_pool[index].clone()
    };

    // Create dialogue entity
    let dialogue_entity = commands
        .spawn((
            Name::new("Level End Dialogue"),
            DialogueRunner::default(),
            StateScoped(LevelState::EndDialogue),
        ))
        .id();

    // Spawn UI
    spawn_dialogue_ui(&mut commands, LevelState::EndDialogue);

    // Start dialogue
    start_dialogue_events.write(StartDialogueEvent {
        entity: dialogue_entity,
        dialogue_handle,
    });
}

/// Transition to success/failed when end dialogue ends
fn handle_end_dialogue_end(
    mut dialogue_ended_events: EventReader<DialogueEnded>,
    mut level_state: ResMut<NextState<LevelState>>,
    current_level: Res<CurrentLevel>,
    current_state: Res<State<LevelState>>,
) {
    if *current_state.get() != LevelState::EndDialogue {
        return;
    }

    for _ in dialogue_ended_events.read() {
        info!("End dialogue finished");
        match current_level.level_completed_successfully {
            Some(true) => level_state.set(LevelState::Success),
            Some(false) => level_state.set(LevelState::Failed),
            None => {
                warn!("EndDialogue reached without completion status");
                level_state.set(LevelState::Failed);
            }
        }
    }
}

/// Spawn dialogue UI
fn spawn_dialogue_ui(commands: &mut Commands, state: LevelState) {
    // First spawn the default UI
    let ui_entity = funkus_dialogue_ui::spawn_dialogue_ui(commands);

    // Add state scoping to the parent
    commands.entity(ui_entity).insert(StateScoped(state));

    // Add portrait display as a child
    commands.entity(ui_entity).with_children(|parent| {
        parent.spawn((
            Name::new("Dialogue Portrait"),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Percent(100.0),
                height: Val::Px(250.0),
                ..default()
            },
            ImageNode::default(),
            Visibility::Hidden,
            DialoguePortrait,
        ));
    });
}

/// Component to mark the portrait UI element
#[derive(Component)]
struct DialoguePortrait;

/// Update portrait display based on current dialogue node
fn update_dialogue_portrait(
    dialogue_runners: Query<&DialogueRunner>,
    dialogue_assets: Res<Assets<DialogueAsset>>,
    dialogue_res: Res<DialogueAssets>,
    mut portrait_query: Query<(&mut ImageNode, &mut Visibility), With<DialoguePortrait>>,
) {
    // Find active dialogue runner
    for runner in dialogue_runners.iter() {
        if runner.state == DialogueState::Inactive {
            continue;
        }

        // Get the dialogue asset
        let Some(dialogue) = dialogue_assets.get(&runner.dialogue_handle) else {
            continue;
        };

        // Get current node
        let Some(current_node) = runner.current_node(dialogue) else {
            continue;
        };

        // Extract portrait from node
        let portrait_id = match current_node {
            DialogueNode::Text { portrait, .. } => portrait.as_deref(),
            DialogueNode::Choice { portrait, .. } => portrait.as_deref(),
        };

        // Update portrait display
        for (mut image, mut visibility) in portrait_query.iter_mut() {
            match portrait_id {
                Some("wizard") => {
                    image.image = dialogue_res.portraits.wizard.clone();
                    *visibility = Visibility::Inherited;
                }
                Some("witch") => {
                    image.image = dialogue_res.portraits.witch.clone();
                    *visibility = Visibility::Inherited;
                }
                _ => {
                    // Hide portrait if none specified
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}
