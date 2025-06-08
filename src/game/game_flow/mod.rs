//! Turn-based gameplay state management

use bevy::prelude::*;

use crate::{
    game::{
        carddeck::events::DrawEvent,
        level::{definitions::{load_level_config, LevelDefinitions}, CurrentGameplayMusic},
        mushrooms::{chain_activation::reset_mushroom_states, ChainManager},
        play_field::placement_preview::PreviewConnections,
        resources::GameState,
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Initialize states
    app.init_state::<LevelState>();
    app.init_state::<LevelLifecycle>();
    app.add_sub_state::<TurnPhase>();

    // Add events
    app.add_event::<LevelCompleteAction>();

    // Add transition systems
    app.add_systems(OnEnter(Screen::Gameplay), enter_first_level);
    app.add_systems(OnExit(Screen::Gameplay), cleanup_gameplay_state);
    app.add_systems(
        Update,
        check_phase_completion.run_if(in_state(Screen::Gameplay)),
    );

    // Handle level complete actions
    app.add_observer(handle_level_complete_action);

    // State transition systems
    app.add_systems(OnEnter(TurnPhase::Draw), enter_draw_phase);
    app.add_systems(OnEnter(TurnPhase::Planting), enter_planting_phase);
    app.add_systems(
        OnEnter(TurnPhase::Chain),
        (enter_chain_phase, reset_mushroom_states).chain(),
    );
    app.add_systems(OnEnter(TurnPhase::Score), enter_score_phase);
    app.add_systems(OnEnter(LevelState::Success), spawn_level_success_ui);
    app.add_systems(OnEnter(LevelState::Failed), spawn_level_failed_ui);

    // Level lifecycle management
    app.add_systems(OnEnter(LevelState::StartDialogue), activate_level_lifecycle);
    app.add_systems(
        OnEnter(LevelState::Success), 
        deactivate_level_lifecycle.after(spawn_level_success_ui)
    );
    app.add_systems(
        OnEnter(LevelState::Failed), 
        deactivate_level_lifecycle.after(spawn_level_failed_ui)
    );
    app.add_systems(OnEnter(LevelState::NotPlaying), deactivate_level_lifecycle);

    // Initialize resources
    app.init_resource::<TurnData>();
    app.init_resource::<CurrentLevel>();
}

/// High-level state of the current level
#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
#[states(scoped_entities)]
pub enum LevelState {
    #[default]
    NotPlaying,
    StartDialogue,
    Playing,
    EndDialogue,
    Success,
    Failed,
}

/// State tracking if a level is currently active (from start dialogue through completion)
#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
#[states(scoped_entities)]
pub enum LevelLifecycle {
    #[default]
    Inactive,
    Active,
}

/// Turn phases - these substates only exist when in LevelState::Playing
#[derive(SubStates, Default, Clone, Eq, PartialEq, Hash, Debug)]
#[source(LevelState = LevelState::Playing)]
#[states(scoped_entities)]
pub enum TurnPhase {
    #[default]
    Draw,
    Planting,
    Chain,
    Score,
}

/// Data about the current turn
#[derive(Resource, Default, Debug)]
pub struct TurnData {
    pub current_turn: u32,
    pub mushrooms_drawn_this_turn: u32,
    pub activations_this_chain: u32,
    pub spores_this_chain: f64,
}

/// Current level configuration
#[derive(Resource, Default, Debug)]
pub struct CurrentLevel {
    pub level_index: usize,
    pub target_score: f64,
    pub max_turns: u32,
    pub total_spores_earned: f64,
    pub level_completed_successfully: Option<bool>, // None = still playing, Some(true) = won, Some(false) = lost
}

/// Actions available when a level is complete
#[derive(Event, Debug)]
pub enum LevelCompleteAction {
    NextLevel,
    RetryLevel,
    MainMenu,
}

/// Activate level lifecycle when starting a level
fn activate_level_lifecycle(mut next_state: ResMut<NextState<LevelLifecycle>>) {
    info!("Activating level lifecycle");
    next_state.set(LevelLifecycle::Active);
}

/// Deactivate level lifecycle when level ends
fn deactivate_level_lifecycle(mut next_state: ResMut<NextState<LevelLifecycle>>) {
    info!("Deactivating level lifecycle");
    next_state.set(LevelLifecycle::Inactive);
}

/// Load a specific level by index
fn load_level(
    level_index: usize,
    level_definitions: &LevelDefinitions,
    current_level: &mut CurrentLevel,
    turn_data: &mut TurnData,
    game_state: &mut GameState,
) -> Result<String, String> {
    if let Some(level_def) = load_level_config(level_index, level_definitions, game_state) {
        let level_name = level_def.name.clone();

        *current_level = CurrentLevel {
            level_index,
            target_score: level_def.target_score,
            max_turns: level_def.max_turns,
            total_spores_earned: 0.0,
            level_completed_successfully: None,
        };

        *turn_data = TurnData {
            current_turn: 1,
            ..default()
        };

        game_state.spores = 0.0;

        info!(
            "Loaded level {}: {} ({}x{} grid, {} turns, {} spore target)",
            level_index + 1,
            level_name,
            level_def.grid_width,
            level_def.grid_height,
            level_def.max_turns,
            level_def.target_score
        );

        Ok(level_name)
    } else {
        Err(format!("Failed to load level {level_index}"))
    }
}

/// Enter the first level when gameplay starts
fn enter_first_level(
    mut level_state: ResMut<NextState<LevelState>>,
    mut current_level: ResMut<CurrentLevel>,
    mut turn_data: ResMut<TurnData>,
    level_definitions: Res<LevelDefinitions>,
    mut game_state: ResMut<GameState>,
) {
    info!("Starting first level");

    match load_level(
        0,
        &level_definitions,
        &mut current_level,
        &mut turn_data,
        &mut game_state,
    ) {
        Ok(_) => {
            level_state.set(LevelState::StartDialogue);
        }
        Err(e) => {
            error!("{}", e);
        }
    }
}

/// Draw phase - player draws mushrooms from bag
fn enter_draw_phase(
    mut commands: Commands,
    mut turn_data: ResMut<TurnData>,
    current_level: Res<CurrentLevel>,
    mut phase_state: ResMut<NextState<TurnPhase>>,
) {
    info!("=== DRAW PHASE ===");
    info!(
        "Turn {}/{}",
        turn_data.current_turn, current_level.max_turns
    );

    let draw_amount = if turn_data.current_turn == 1 { 6 } else { 4 };
    turn_data.mushrooms_drawn_this_turn = draw_amount;

    info!("Drawing {} mushrooms from bag", draw_amount);
    commands.trigger(DrawEvent(draw_amount));

    phase_state.set(TurnPhase::Planting);
}

/// Planting phase - player places mushrooms on grid
fn enter_planting_phase() {
    info!("=== PLANTING PHASE ===");
}

/// Chain phase - watch mushrooms activate in sequence
fn enter_chain_phase(mut turn_data: ResMut<TurnData>) {
    info!("=== CHAIN PHASE ===");

    // Reset chain counters
    turn_data.activations_this_chain = 0;
    turn_data.spores_this_chain = 0.0;
}

/// Spawn success UI
fn spawn_level_success_ui(commands: Commands) {
    spawn_level_complete_ui(commands, true);
}

/// Spawn failure UI
fn spawn_level_failed_ui(commands: Commands) {
    spawn_level_complete_ui(commands, false);
}

/// Score phase - check win/loss conditions
fn enter_score_phase(
    chain_manager: Res<ChainManager>,
    mut current_level: ResMut<CurrentLevel>,
    mut level_state: ResMut<NextState<LevelState>>,
    mut _game_state: ResMut<GameState>,
    turn_data: Res<TurnData>,
) {
    info!("=== SCORE PHASE ===");

    // Calculate total spores from all chains this turn
    let chain_score: f64 = chain_manager
        .chains
        .iter()
        .map(|chain| chain.total_spores)
        .sum();

    current_level.total_spores_earned += chain_score;

    info!(
        "Generated {} spores from {} chains this turn",
        chain_score,
        chain_manager.chains.len()
    );
    info!(
        "Total: {}/{} spores",
        current_level.total_spores_earned, current_level.target_score
    );

    // Check win condition
    if current_level.total_spores_earned >= current_level.target_score {
        info!("Level complete - SUCCESS!");
        current_level.level_completed_successfully = Some(true);
        level_state.set(LevelState::EndDialogue);
        return;
    }

    // Check loss condition (out of turns)
    if turn_data.current_turn >= current_level.max_turns {
        info!("Level complete - FAILED (out of turns)");
        current_level.level_completed_successfully = Some(false);
        level_state.set(LevelState::EndDialogue);
        return;
    }

    // Continue to next turn
    info!("Continuing to next turn...");
}

fn handle_level_complete_action(
    trigger: Trigger<LevelCompleteAction>,
    mut level_state: ResMut<NextState<LevelState>>,
    mut current_level: ResMut<CurrentLevel>,
    mut turn_data: ResMut<TurnData>,
    level_definitions: Res<LevelDefinitions>,
    mut game_state: ResMut<GameState>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    match trigger.event() {
        LevelCompleteAction::RetryLevel => {
            info!("Retrying level {}", current_level.level_index + 1);

            if load_level(
                current_level.level_index,
                &level_definitions,
                &mut current_level,
                &mut turn_data,
                &mut game_state,
            )
            .is_ok()
            {
                // Transition from Success/Failed -> StartDialogue
                // The lifecycle will be handled by state transitions
                level_state.set(LevelState::StartDialogue);
            }
        }

        LevelCompleteAction::NextLevel => {
            let next_index = current_level.level_index + 1;
            info!("Progressing to level {}", next_index + 1);

            match load_level(
                next_index,
                &level_definitions,
                &mut current_level,
                &mut turn_data,
                &mut game_state,
            ) {
                Ok(_) => {
                    level_state.set(LevelState::StartDialogue);
                }
                Err(_) => {
                    info!("No more levels! Game complete!");
                    next_screen.set(Screen::Title);
                }
            }
        }

        LevelCompleteAction::MainMenu => {
            info!("Returning to main menu");
            next_screen.set(Screen::Title);
        }
    }
}

/// Clean up game state when exiting gameplay
fn cleanup_gameplay_state(
    mut level_state: ResMut<NextState<LevelState>>,
    mut level_lifecycle: ResMut<NextState<LevelLifecycle>>,
    mut turn_data: ResMut<TurnData>,
    mut current_level: ResMut<CurrentLevel>,
    mut game_state: ResMut<GameState>,
    mut chain_manager: ResMut<ChainManager>,
    mut preview_connections: ResMut<PreviewConnections>,
    mut gameplay_music: ResMut<CurrentGameplayMusic>,
) {
    info!("Cleaning up gameplay state");

    // Reset states - this will trigger StateScoped cleanup
    level_state.set(LevelState::NotPlaying);
    level_lifecycle.set(LevelLifecycle::Inactive);

    // Clear resources
    *turn_data = TurnData::default();
    *current_level = CurrentLevel::default();

    // Reset game state
    game_state.spores = 0.0;
    game_state.total_activations = 0;
    game_state.chain_activations = 0;

    // Clear chain manager
    chain_manager.chains.clear();
    chain_manager.activation_queue.clear();
    chain_manager.current_chain = None;
    chain_manager.chain_started_this_turn = false;

    // Clear preview connections
    preview_connections.connected_positions.clear();
    preview_connections.empty_connection_points.clear();
    preview_connections.existing_connection_targets.clear();
    preview_connections.preview_position = None;

    // Clear music tracking
    gameplay_music.current_track = None;
    gameplay_music.entity = None;
}

/// Automatically advance phases based on completion conditions
fn check_phase_completion(
    current_phase: Option<Res<State<TurnPhase>>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    chain_manager: Res<ChainManager>,
) {
    let Some(phase_state) = current_phase else {
        return;
    };

    match phase_state.get() {
        TurnPhase::Draw => {
            // Auto-advance handled by draw system
        }
        TurnPhase::Planting => {
            // Handled by player action
        }
        TurnPhase::Chain => {
            if !chain_manager.has_active_chains() && chain_manager.chain_started_this_turn {
                info!("All chains complete, advancing to score phase");
                next_phase.set(TurnPhase::Score);
            }
        }
        TurnPhase::Score => {
            // Handled by score phase logic
        }
    }
}

fn spawn_level_complete_ui(mut commands: Commands, success: bool) {
    use bevy::ui::Val::*;

    commands
        .spawn((
            Name::new("Level Complete UI"),
            Node {
                position_type: PositionType::Absolute,
                width: Percent(100.0),
                height: Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            StateScoped(if success {
                LevelState::Success
            } else {
                LevelState::Failed
            }),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new(if success {
                    "LEVEL COMPLETE!"
                } else {
                    "LEVEL FAILED!"
                }),
                TextFont::from_font_size(48.0),
                TextColor(if success {
                    Color::srgb(0.2, 0.8, 0.2)
                } else {
                    Color::srgb(0.8, 0.2, 0.2)
                }),
            ));

            // Buttons container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Px(20.0),
                    ..default()
                })
                .with_children(|buttons| {
                    // Retry button
                    buttons
                        .spawn((
                            Button,
                            Node {
                                padding: UiRect::all(Px(20.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        ))
                        .with_child((Text::new("Retry Level"), TextFont::from_font_size(24.0)))
                        .observe(|_: Trigger<Pointer<Click>>, mut commands: Commands| {
                            commands.trigger(LevelCompleteAction::RetryLevel);
                        });

                    // Next level button (only if success)
                    if success {
                        buttons
                            .spawn((
                                Button,
                                Node {
                                    padding: UiRect::all(Px(20.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.2, 0.5, 0.2)),
                            ))
                            .with_child((Text::new("Next Level"), TextFont::from_font_size(24.0)))
                            .observe(|_: Trigger<Pointer<Click>>, mut commands: Commands| {
                                commands.trigger(LevelCompleteAction::NextLevel);
                            });
                    }

                    // Main menu button
                    buttons
                        .spawn((
                            Button,
                            Node {
                                padding: UiRect::all(Px(20.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        ))
                        .with_child((Text::new("Main Menu"), TextFont::from_font_size(24.0)))
                        .observe(|_: Trigger<Pointer<Click>>, mut commands: Commands| {
                            commands.trigger(LevelCompleteAction::MainMenu);
                        });
                });
        });
}