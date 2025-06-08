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

    // Add the dialogue delay timer resource
    app.init_resource::<DialogueAdvanceDelay>();

    // Systems for dialogue states
    app.add_systems(OnEnter(LevelState::StartDialogue), enter_start_dialogue);
    app.add_systems(OnEnter(LevelState::EndDialogue), enter_end_dialogue);

    // Update systems during dialogue
    app.add_systems(
        Update,
        (
            update_dialogue_advance_timer,
            handle_dialogue_input,
            handle_dialogue_click,
            handle_start_dialogue_end,
            update_dialogue_portrait,
            handle_end_dialogue_end,
        )
            .chain()
            .run_if(in_state(LevelState::StartDialogue).or(in_state(LevelState::EndDialogue))),
    );
}

/// Resource to track dialogue advance delay
#[derive(Resource, Debug)]
pub struct DialogueAdvanceDelay {
    /// Timer that must elapse before dialogue can be advanced
    timer: Timer,
    /// Whether the current dialogue node can be advanced
    can_advance: bool,
}

impl Default for DialogueAdvanceDelay {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
            can_advance: false,
        }
    }
}

impl DialogueAdvanceDelay {
    /// Reset the timer for a new dialogue node
    fn reset(&mut self, delay_seconds: f32) {
        self.timer = Timer::from_seconds(delay_seconds, TimerMode::Once);
        self.can_advance = false;
    }

    /// Update the timer
    fn tick(&mut self, delta: std::time::Duration) {
        self.timer.tick(delta);
        if self.timer.finished() {
            self.can_advance = true;
        }
    }
}

/// Update the dialogue advance timer
fn update_dialogue_advance_timer(
    time: Res<Time>,
    mut delay: ResMut<DialogueAdvanceDelay>,
    dialogue_query: Query<&DialogueRunner>,
    mut last_state: Local<Option<DialogueState>>,
) {
    // Check if dialogue state changed
    for runner in dialogue_query.iter() {
        if runner.state == DialogueState::Inactive {
            continue;
        }

        // If state changed, reset the timer
        if last_state.as_ref() != Some(&runner.state) {
            match runner.state {
                DialogueState::ShowingText => {
                    // Reset timer when new text is shown
                    delay.reset(0.5);
                    *last_state = Some(runner.state.clone());
                }
                DialogueState::WaitingForChoice => {
                    // No delay for choices - can select immediately
                    delay.can_advance = true;
                    *last_state = Some(runner.state.clone());
                }
                _ => {
                    *last_state = Some(runner.state.clone());
                }
            }
        }
    }

    // Update the timer
    delay.tick(time.delta());
}

/// Handle keyboard input during dialogue
fn handle_dialogue_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    dialogue_query: Query<(Entity, &DialogueRunner)>,
    mut advance_events: EventWriter<AdvanceDialogue>,
    mut select_events: EventWriter<SelectDialogueChoice>,
    delay: Res<DialogueAdvanceDelay>,
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
                    if delay.can_advance {
                        advance_events.write(AdvanceDialogue { entity });
                    }
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

/// Handle mouse click to advance dialogue
fn handle_dialogue_click(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    dialogue_query: Query<(Entity, &DialogueRunner)>,
    mut advance_events: EventWriter<AdvanceDialogue>,
    delay: Res<DialogueAdvanceDelay>,
) {
    // Check for left mouse button click
    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    for (entity, runner) in dialogue_query.iter() {
        // Skip if dialogue is not active
        if runner.state == DialogueState::Inactive {
            continue;
        }

        match runner.state {
            DialogueState::ShowingText => {
                if delay.can_advance {
                    advance_events.write(AdvanceDialogue { entity });
                }
            }
            DialogueState::ChoiceSelected(_) => {
                advance_events.write(AdvanceDialogue { entity });
            }
            _ => {}
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
    mut delay: ResMut<DialogueAdvanceDelay>,
) {
    // Reset dialogue delay
    delay.reset(0.5);

    // Determine if this level has an intro dialogue
    let total_levels = level_definitions.levels.len();
    let dialogue_handle = match current_level.level_index {
        0 => Some(dialogue_assets.level_1_intro.clone()), // Level 1
        1 => Some(dialogue_assets.level_2_intro.clone()), // Level 2
        2 => Some(dialogue_assets.level_3_intro.clone()), // Level 3
        3 => Some(dialogue_assets.level_4_intro.clone()), // Level 4
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
    mut delay: ResMut<DialogueAdvanceDelay>,
) {
    info!("Starting level outro dialogue");

    // Reset dialogue delay
    delay.reset(0.5);

    // Select random dialogue based on success/failure
    let dialogue_pool = match current_level.level_completed_successfully {
        Some(true) => &dialogue_assets.success_dialogues_1,
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

    // Add portrait display and click hint as children
    commands.entity(ui_entity).with_children(|parent| {
        // Portrait
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

        // Click hint
        parent.spawn((
            Name::new("Dialogue Click Hint"),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                right: Val::Px(10.0),
                ..default()
            },
            Text::new("Click or press Space to continue..."),
            TextFont::from_font_size(14.0),
            TextColor(Color::srgba(0.8, 0.8, 0.8, 0.6)),
            DialogueClickHint,
        ));
    });
}

/// Component to mark the portrait UI element
#[derive(Component)]
struct DialoguePortrait;

/// Component to mark the click hint UI element
#[derive(Component)]
struct DialogueClickHint;

/// Update portrait display based on current dialogue node
fn update_dialogue_portrait(
    dialogue_runners: Query<&DialogueRunner>,
    dialogue_assets: Res<Assets<DialogueAsset>>,
    dialogue_res: Res<DialogueAssets>,
    mut portrait_query: Query<(&mut ImageNode, &mut Visibility), With<DialoguePortrait>>,
    mut click_hint_query: Query<
        &mut Visibility,
        (With<DialogueClickHint>, Without<DialoguePortrait>),
    >,
    delay: Res<DialogueAdvanceDelay>,
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

        // Update click hint visibility based on dialogue state and timer
        for mut hint_visibility in click_hint_query.iter_mut() {
            *hint_visibility = match runner.state {
                DialogueState::ShowingText if delay.can_advance => Visibility::Inherited,
                DialogueState::ChoiceSelected(_) => Visibility::Inherited,
                _ => Visibility::Hidden,
            };
        }
    }
}
