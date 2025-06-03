//! Game UI for displaying state and controls

use bevy::{ecs::spawn::SpawnWith, picking::prelude::*, prelude::*};

use crate::{
    game::{
        resources::{GameState, UnlockedMushrooms},
        turn_manager::{CurrentLevel, LevelState, TurnData, TurnPhase},
    },
    screens::Screen,
    theme::{interaction::InteractionPalette, palette as ui_palette},
};

use super::mushrooms::{MushroomType, resources::SelectedMushroomType};

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

fn spawn_game_ui(mut commands: Commands) {
    // Top bar for game stats
    commands.spawn((
        Name::new("Game UI - Top Bar"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            right: Val::Px(10.0),
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        StateScoped(Screen::Gameplay),
        children![
            (
                Name::new("Spore Count"),
                Text::new("Spores: 0"),
                TextFont::from_font_size(32.0),
                TextColor(ui_palette::HEADER_TEXT),
                SporeDisplay,
            ),
            (
                Name::new("Stats"),
                Text::new("Activations: 0 | Chains: 0"),
                TextFont::from_font_size(20.0),
                TextColor(ui_palette::LABEL_TEXT),
                StatsDisplay,
            ),
            // Add turn phase display
            (
                Name::new("Turn Phase"),
                Text::new("Phase: Loading..."),
                TextFont::from_font_size(24.0),
                TextColor(Color::srgb(0.8, 0.8, 0.2)),
                TurnPhaseDisplay,
            ),
            // Add level progress display
            (
                Name::new("Level Progress"),
                Text::new("Level 1 - Turn 1/5 - Goal: 0/100"),
                TextFont::from_font_size(18.0),
                TextColor(ui_palette::LABEL_TEXT),
                LevelProgressDisplay,
            ),
        ],
    ));

    // Add phase control button
    commands
        .spawn((
            Name::new("Phase Control"),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Percent(50.0),
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
            InteractionPalette {
                none: Color::srgb(0.2, 0.5, 0.2),
                hovered: Color::srgb(0.3, 0.6, 0.3),
                pressed: Color::srgb(0.4, 0.7, 0.4),
            },
            children![(
                Name::new("Button Text"),
                Text::new("Next Phase (Space)"),
                TextFont::from_font_size(20.0),
                TextColor(Color::WHITE),
                Pickable::IGNORE,
            )],
        ))
        .observe(advance_phase_on_click);

    // Side panel for mushroom selection
    commands.spawn((
        Name::new("Game UI - Side Panel"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(100.0),
            right: Val::Px(10.0),
            width: Val::Px(300.0),
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        StateScoped(Screen::Gameplay),
        children![
            (
                Name::new("Mushroom Selection Header"),
                Text::new("Mushrooms"),
                TextFont::from_font_size(28.0),
                TextColor(ui_palette::HEADER_TEXT),
            ),
            (
                Name::new("Instructions"),
                Text::new("Click mushrooms to activate them!\nShift+Click to rotate directional mushrooms\nRight-click to delete (50% refund)"),
                TextFont::from_font_size(16.0),
                TextColor(ui_palette::LABEL_TEXT),
            ),
            spawn_mushroom_button(MushroomType::Basic),
            spawn_mushroom_button(MushroomType::Pulse),
        ],
    ));
}

fn spawn_mushroom_button(mushroom_type: MushroomType) -> impl Bundle {
    (
        Name::new(format!("{} Button", mushroom_type.name())),
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
                    BorderColor(mushroom_type.color()),
                    BorderRadius::all(Val::Px(5.0)),
                    MushroomButton { mushroom_type },
                    InteractionPalette {
                        none: Color::srgba(0.3, 0.3, 0.3, 0.8),
                        hovered: Color::srgba(0.4, 0.4, 0.4, 0.9),
                        pressed: Color::srgba(0.5, 0.5, 0.5, 1.0),
                    },
                    children![
                        (
                            Name::new("Mushroom Name"),
                            Text::new(mushroom_type.name()),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::WHITE),
                            Pickable::IGNORE,
                        ),
                        (
                            Name::new("Mushroom Cost"),
                            Text::new(format!("Cost: {:.0} spores", mushroom_type.cost())),
                            TextFont::from_font_size(16.0),
                            TextColor(ui_palette::LABEL_TEXT),
                            Pickable::IGNORE,
                        ),
                        (
                            Name::new("Mushroom Effect"),
                            Text::new(mushroom_type.description()),
                            TextFont::from_font_size(14.0),
                            TextColor(Color::srgba(0.8, 0.8, 0.8, 0.8)),
                            Pickable::IGNORE,
                        ),
                    ],
                ))
                .observe(
                    move |_: Trigger<Pointer<Click>>,
                          mut selected: ResMut<SelectedMushroomType>| {
                        println!("Selected mushroom: {}", mushroom_type.name());
                        selected.mushroom_type = mushroom_type;
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
    if let Ok((mut text, mut text_color)) = phase_display.get_single_mut() {
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
            LevelState::NotPlaying => "Not in game",
        };

        text.0 = format!("Phase: {}", phase_text);

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
    if let Ok((children, mut visibility)) = button.get_single_mut() {
        // Update button visibility
        *visibility = if *current_level_state.get() == LevelState::Playing {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        // Update button text based on phase - find the text child
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

// Add level progress update function:
fn update_level_progress_display(
    current_level: Res<CurrentLevel>,
    turn_data: Res<TurnData>,
    mut progress_display: Query<&mut Text, With<LevelProgressDisplay>>,
) {
    if let Ok(mut text) = progress_display.single_mut() {
        text.0 = format!(
            "Level {} - Turn {}/{} - Goal: {:.0}/{:.0} spores",
            current_level.level_index,
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
    unlocked: Res<UnlockedMushrooms>,
    selected: Res<SelectedMushroomType>,
    mut buttons: Query<(
        &MushroomButton,
        &mut BorderColor,
        &mut Visibility,
        &Children,
    )>,
    mut text_colors: Query<&mut TextColor>,
) {
    for (button, mut border_color, mut visibility, children) in &mut buttons {
        // Check if unlocked
        let is_unlocked = match button.mushroom_type {
            MushroomType::Basic => unlocked.button,
            MushroomType::Pulse => unlocked.pulse,
        };

        // Update visibility
        *visibility = if is_unlocked {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        // Update border to show selection
        if selected.mushroom_type == button.mushroom_type {
            border_color.0 = Color::WHITE;
        } else {
            border_color.0 = button.mushroom_type.color();
        }

        // Update cost text color based on affordability
        if let Some(&cost_entity) = children.get(1) {
            if let Ok(mut text_color) = text_colors.get_mut(cost_entity) {
                let can_afford = game_state.spores >= button.mushroom_type.cost();
                text_color.0 = if can_afford {
                    Color::srgb(0.2, 0.8, 0.2)
                } else {
                    Color::srgb(0.8, 0.2, 0.2)
                };
            }
        }
    }
}
