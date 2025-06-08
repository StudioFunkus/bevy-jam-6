//! Game UI for displaying state and controls

use bevy::{ecs::spawn::SpawnWith, prelude::*};

use crate::{
    game::{
        game_flow::{CurrentLevel, LevelState, TurnData, TurnPhase},
        mushrooms::{ChainManager, MushroomDefinitions, MushroomType, SelectedMushroomType},
        resources::GameState,
    },
    screens::Screen,
    theme::{interaction::InteractionPalette, palette as ui_palette},
};

/// Marker for UI that should be hidden during dialogue
#[derive(Component)]
struct GameplayUI;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_game_ui);
    app.add_systems(
        Update,
        (
            update_spore_display,
            update_mushroom_buttons,
            update_turn_phase_display,
            update_level_progress_display,
            update_phase_button,
            update_chain_info,
            control_ui_visibility,
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// Marker for the spore count display
#[derive(Component)]
struct SporeDisplay;

/// Marker for the stats display
#[derive(Component)]
struct StatsDisplay;

/// Marker for the turn phase display
#[derive(Component)]
struct TurnPhaseDisplay;

/// Marker for the level progress display
#[derive(Component)]
struct LevelProgressDisplay;

/// Component for phase advance button
#[derive(Component)]
struct PhaseAdvanceButton;

/// Component for mushroom purchase buttons
#[derive(Component)]
struct MushroomButton {
    mushroom_type: MushroomType,
}

/// Marker for chain information display
#[derive(Component)]
struct ChainInfoDisplay;

fn spawn_game_ui(mut commands: Commands, _definitions: Res<MushroomDefinitions>) {
    // Top bar for game stats
    commands
        .spawn((
            Name::new("Game UI - Top Bar"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                right: Val::Px(10.0),
                padding: UiRect::all(Val::Px(20.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                width: Val::Percent(30.0),
                ..default()
            },
            GameplayUI,
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            StateScoped(Screen::Gameplay),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Spore Count"),
                Text::new("Spores: 0"),
                TextFont::from_font_size(32.0),
                TextColor(ui_palette::HEADER_TEXT),
                SporeDisplay,
            ));

            parent.spawn((
                Name::new("Stats"),
                Text::new("Activations: 0 | Chains: 0"),
                TextFont::from_font_size(20.0),
                TextColor(ui_palette::LABEL_TEXT),
                StatsDisplay,
            ));

            // Add turn phase display
            parent.spawn((
                Name::new("Turn Phase"),
                Text::new("Phase: Loading..."),
                TextFont::from_font_size(24.0),
                TextColor(Color::srgb(0.8, 0.8, 0.2)),
                TurnPhaseDisplay,
            ));

            // Add level progress display
            parent.spawn((
                Name::new("Level Progress"),
                Text::new("Level 1 - Turn 1/5 - Goal: 0/100"),
                TextFont::from_font_size(18.0),
                TextColor(ui_palette::LABEL_TEXT),
                LevelProgressDisplay,
            ));

            // Add chain info display
            parent.spawn((
                Name::new("Chain Info"),
                Text::new(""),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.4, 0.8, 1.0)),
                ChainInfoDisplay,
            ));
        });

    // Add phase control button
    commands
        .spawn((
            Name::new("Phase Control"),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(100.0),
                left: Val::Percent(70.0),
                width: Val::Px(200.0),
                height: Val::Px(60.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Button,
            BackgroundColor(Color::srgb(0.2, 0.5, 0.2)),
            BorderColor(Color::WHITE),
            BorderRadius::all(Val::Px(10.0)),
            StateScoped(Screen::Gameplay),
            PhaseAdvanceButton,
            GameplayUI,
            InteractionPalette {
                none: Color::srgb(0.2, 0.5, 0.2),
                hovered: Color::srgb(0.3, 0.6, 0.3),
                pressed: Color::srgb(0.4, 0.7, 0.4),
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Button Text"),
                Text::new("Next Phase"),
                TextFont::from_font_size(20.0),
                TextColor(Color::WHITE),
                Pickable::IGNORE,
            ));
        })
        .observe(advance_phase_on_click);

    // Side panel for mushroom selection
    //     commands
    //         .spawn((
    //             Name::new("Game UI - Side Panel"),
    //             Node {
    //                 position_type: PositionType::Absolute,
    //                 top: Val::Px(100.0),
    //                 right: Val::Px(10.0),
    //                 width: Val::Px(300.0),
    //                 padding: UiRect::all(Val::Px(20.0)),
    //                 flex_direction: FlexDirection::Column,
    //                 row_gap: Val::Px(15.0),
    //                 ..default()
    //             },
    //             BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
    //             StateScoped(Screen::Gameplay),
    //             GameplayUI,
    //             children![
    //                 (
    //                     Name::new("Mushroom Selection Header"),
    //                     Text::new("Mushrooms"),
    //                     TextFont::from_font_size(28.0),
    //                     TextColor(ui_palette::HEADER_TEXT),
    //                 ),
    //                 (
    //                     Name::new("Instructions"),
    //                     Text::new("Hover to preview placement\nPress R to rotate preview\nClick to place mushroom\nRight-click to delete"),
    //                     TextFont::from_font_size(16.0),
    //                     TextColor(ui_palette::LABEL_TEXT),
    //                 ),
    //                 spawn_mushroom_button(MushroomType::Basic, &definitions),
    //                 spawn_mushroom_button(MushroomType::Pulse, &definitions),
    //                 spawn_mushroom_button(MushroomType::Fork, &definitions),
    //                 spawn_mushroom_button(MushroomType::Sideways, &definitions),
    //                 spawn_mushroom_button(MushroomType::Threeway, &definitions),
    //                 spawn_mushroom_button(MushroomType::Diagonal, &definitions),
    //                 spawn_mushroom_button(MushroomType::Surround, &definitions),
    //                 spawn_mushroom_button(MushroomType::Skipper, &definitions),
    // //                spawn_mushroom_button(MushroomType::Deleter, &definitions),
    // //                spawn_mushroom_button(MushroomType::Bomb, &definitions),
    // //                spawn_mushroom_button(MushroomType::Burst, &definitions),
    //                 spawn_mushroom_button(MushroomType::Amplifier, &definitions),
    //                 spawn_mushroom_button(MushroomType::Fourway_amplifier, &definitions),
    // //                spawn_mushroom_button(MushroomType::Splitter, &definitions),
    // //                spawn_mushroom_button(MushroomType::Chain, &definitions),
    // //                spawn_mushroom_button(MushroomType::Converter, &definitions),
    // //                spawn_mushroom_button(MushroomType::Knight, &definitions),
    //             ],
    //         ));
}

#[allow(dead_code)]
fn spawn_mushroom_button(
    mushroom_type: MushroomType,
    definitions: &MushroomDefinitions,
) -> impl Bundle {
    (
        Name::new(format!(
            "{} Button",
            definitions
                .get(mushroom_type)
                .map_or("Unknown", |d| d.name.as_str())
        )),
        Node::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        row_gap: Val::Px(5.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.8)),
                    BorderColor(Color::srgba(0.5, 0.5, 0.5, 0.8)),
                    BorderRadius::all(Val::Px(5.0)),
                    MushroomButton { mushroom_type },
                    InteractionPalette {
                        none: Color::srgba(0.3, 0.3, 0.3, 0.8),
                        hovered: Color::srgba(0.4, 0.4, 0.4, 0.9),
                        pressed: Color::srgba(0.5, 0.5, 0.5, 1.0),
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Name::new("Mushroom Name"),
                        Text::new("Loading..."),
                        TextFont::from_font_size(20.0),
                        TextColor(Color::WHITE),
                        Pickable::IGNORE,
                    ));

                    parent.spawn((
                        Name::new("Mushroom Effect"),
                        Text::new(""),
                        TextFont::from_font_size(14.0),
                        TextColor(Color::srgba(0.8, 0.8, 0.8, 0.8)),
                        Pickable::IGNORE,
                    ));

                    parent.spawn((
                        Name::new("Mushroom Stats"),
                        Text::new(""),
                        TextFont::from_font_size(12.0),
                        TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
                        Pickable::IGNORE,
                    ));
                })
                .observe(
                    move |_: Trigger<Pointer<Click>>,
                          mut selected: ResMut<SelectedMushroomType>| {
                        println!("Selected mushroom: {:?}", mushroom_type);
                        selected.mushroom_type = Some(mushroom_type);
                    },
                );
        })),
    )
}

fn update_turn_phase_display(
    current_phase: Option<Res<State<TurnPhase>>>,
    current_level_state: Res<State<LevelState>>,
    mut phase_display: Query<
        (&mut Text, &mut TextColor),
        (With<TurnPhaseDisplay>, Without<LevelProgressDisplay>),
    >,
) {
    if let Ok((mut text, mut text_color)) = phase_display.single_mut() {
        let phase_text = match current_level_state.get() {
            LevelState::Playing => {
                if let Some(ref phase) = current_phase {
                    match phase.get() {
                        TurnPhase::Draw => "DRAW PHASE - Draw mushrooms from your bag",
                        TurnPhase::Planting => "PLANTING PHASE - Place mushrooms on the grid",
                        TurnPhase::Chain => "CHAIN PHASE - Click a mushroom to start the reaction",
                        TurnPhase::Score => "SCORE PHASE - Checking results...",
                    }
                } else {
                    "Waiting..."
                }
            }
            LevelState::Failed => "LEVEL COMPLETE (FAILURE)!",
            LevelState::Success => "LEVEL COMPLETE (SUCCESS)!",
            LevelState::StartDialogue => "Start Dialogue",
            LevelState::EndDialogue => "End Dialogue",
            LevelState::NotPlaying => "Not in game",
        };

        text.0 = format!("Phase: {phase_text}");

        // Change color based on phase
        if let Some(ref phase) = current_phase {
            text_color.0 = match phase.get() {
                TurnPhase::Draw => Color::srgb(0.2, 0.8, 0.8),
                TurnPhase::Planting => Color::srgb(0.2, 0.8, 0.2),
                TurnPhase::Chain => Color::srgb(0.8, 0.8, 0.2),
                TurnPhase::Score => Color::srgb(0.8, 0.2, 0.8),
            };
        }
    }
}

fn update_phase_button(
    current_phase: Option<Res<State<TurnPhase>>>,
    current_level_state: Res<State<LevelState>>,
    mut button: Query<(&Children, &mut Visibility), With<PhaseAdvanceButton>>,
    mut texts: Query<&mut Text>,
) {
    if let Ok((children, mut visibility)) = button.single_mut() {
        // Update button visibility
        *visibility = if *current_level_state.get() == LevelState::Playing {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        // Update button text based on phase
        if let Some(&text_entity) = children.first() {
            if let Ok(mut text) = texts.get_mut(text_entity) {
                if let Some(ref phase) = current_phase {
                    text.0 = match phase.get() {
                        TurnPhase::Draw => "Start Planting",
                        TurnPhase::Planting => "Start Chain Phase",
                        TurnPhase::Chain => "End Turn",
                        TurnPhase::Score => "Next Turn",
                    }
                    .to_string();
                }
            }
        }
    }
}

fn advance_phase_on_click(
    _: Trigger<Pointer<Click>>,
    current_phase: Option<Res<State<TurnPhase>>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut turn_data: ResMut<TurnData>,
) {
    if let Some(ref phase) = current_phase {
        let next = match phase.get() {
            TurnPhase::Draw => TurnPhase::Planting,
            TurnPhase::Planting => TurnPhase::Chain,
            TurnPhase::Chain => TurnPhase::Score,
            TurnPhase::Score => {
                turn_data.current_turn += 1;
                TurnPhase::Draw
            }
        };
        next_phase.set(next);
    }
}

fn update_level_progress_display(
    current_level: Res<CurrentLevel>,
    turn_data: Res<TurnData>,
    mut progress_display: Query<&mut Text, With<LevelProgressDisplay>>,
) {
    if let Ok(mut text) = progress_display.single_mut() {
        text.0 = format!(
            "Level {} - Turn {}/{} - Goal: {:.0}/{:.0} spores",
            current_level.level_index + 1,
            turn_data.current_turn,
            current_level.max_turns,
            current_level.total_spores_earned,
            current_level.target_score,
        );
    }
}

fn update_spore_display(
    game_state: Res<GameState>,
    mut spore_display: Query<&mut Text, (With<SporeDisplay>, Without<StatsDisplay>)>,
    mut stats_display: Query<&mut Text, (With<StatsDisplay>, Without<SporeDisplay>)>,
) {
    // Update spore count
    if let Ok(mut text) = spore_display.single_mut() {
        text.0 = format!("Spores: {:.0}", game_state.spores);
    }

    // Update stats
    if let Ok(mut text) = stats_display.single_mut() {
        text.0 = format!(
            "Activations: {} | Chains: {}",
            game_state.total_activations, game_state.chain_activations,
        );
    }
}

fn update_mushroom_buttons(
    game_state: Res<GameState>,
    definitions: Res<MushroomDefinitions>,
    current_level: Res<CurrentLevel>,
    selected: Res<SelectedMushroomType>,
    mut buttons: Query<(
        &MushroomButton,
        &mut BorderColor,
        &mut Visibility,
        &Children,
    )>,
    mut texts: Query<&mut Text>,
) {
    for (button, mut border_color, mut visibility, children) in &mut buttons {
        // Get mushroom definition
        let definition = definitions.get(button.mushroom_type);

        // Check if unlocked
        let is_unlocked =
            definitions.is_unlocked(button.mushroom_type, &game_state, current_level.level_index);

        // Update visibility
        *visibility = if is_unlocked {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        // Update border to show selection
        if selected.mushroom_type == Some(button.mushroom_type) {
            border_color.0 = Color::WHITE;
        } else {
            border_color.0 = Color::srgba(0.5, 0.5, 0.5, 0.8);
        }

        // Update text content if we have a definition
        if let Some(def) = definition {
            // Update name (first child)
            if let Some(name_entity) = children.iter().next() {
                if let Ok(mut text) = texts.get_mut(name_entity) {
                    text.0 = def.name.clone();
                }
            }

            // Update description (second child)
            if let Some(desc_entity) = children.iter().nth(1) {
                if let Ok(mut text) = texts.get_mut(desc_entity) {
                    text.0 = def.description.clone();
                }
            }

            // Update stats (third child)
            if let Some(stats_entity) = children.iter().nth(2) {
                if let Ok(mut text) = texts.get_mut(stats_entity) {
                    text.0 = format!(
                        "Production: {} | Uses: {}/turn",
                        def.base_production, def.max_uses_per_turn
                    );
                }
            }
        }
    }
}

fn update_chain_info(
    chain_manager: Res<ChainManager>,
    current_phase: Option<Res<State<TurnPhase>>>,
    mut chain_display: Query<&mut Text, With<ChainInfoDisplay>>,
) {
    if let Ok(mut text) = chain_display.single_mut() {
        if let Some(phase) = current_phase {
            if *phase.get() == TurnPhase::Chain {
                if chain_manager.chain_started_this_turn {
                    let _active_chains = chain_manager.chains.iter().filter(|c| c.active).count();
                    let total_spores: f64 =
                        chain_manager.chains.iter().map(|c| c.total_spores).sum();

                    text.0 = format!(
                        "Chain Active! {} activations queued | {:.0} spores generated",
                        chain_manager.activation_queue.len(),
                        total_spores
                    );
                } else {
                    text.0 = "Click a mushroom to start a chain reaction!".to_string();
                }
            } else {
                text.0 = String::new();
            }
        }
    }
}

fn control_ui_visibility(
    level_state: Res<State<LevelState>>,
    mut ui_query: Query<&mut Visibility, With<GameplayUI>>,
) {
    let should_hide = matches!(
        level_state.get(),
        LevelState::StartDialogue | LevelState::EndDialogue
    );

    for mut visibility in ui_query.iter_mut() {
        *visibility = if should_hide {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}
