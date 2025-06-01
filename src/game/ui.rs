//! Game UI for displaying state and controls

use bevy::{ecs::spawn::SpawnWith, picking::prelude::*, prelude::*};

use crate::{
    game::resources::{GameState, UnlockedMushrooms}, screens::Screen, theme::{interaction::InteractionPalette, palette as ui_palette}
};

use super::mushrooms::{MushroomType, resources::SelectedMushroomType};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_game_ui);
    app.add_systems(
        Update,
        (update_spore_display, update_mushroom_buttons).run_if(in_state(Screen::Gameplay)),
    );
}

/// Marker for the spore count display
#[derive(Component)]
struct SporeDisplay;

/// Marker for the stats display
#[derive(Component)]
struct StatsDisplay;

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
                Text::new("Triggers: 0 | Chains: 0"),
                TextFont::from_font_size(20.0),
                TextColor(ui_palette::LABEL_TEXT),
                StatsDisplay,
            ),
        ],
    ));

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
                Text::new("Click mushrooms to trigger them!\nThere are other ways to trigger mushrooms too!"),
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
            "Triggers: {} | Chains: {}", 
            game_state.total_triggers,
            game_state.chain_triggers,
        );
    }
}

fn update_mushroom_buttons(
    game_state: Res<GameState>,
    unlocked: Res<UnlockedMushrooms>,
    selected: Res<SelectedMushroomType>,
    mut buttons: Query<(&MushroomButton, &mut BorderColor, &mut Visibility, &Children)>,
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