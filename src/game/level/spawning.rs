//! Level spawning systems

use bevy::prelude::*;

use crate::{
    audio::music,
    game::{
        grid::{GridCell, GridConfig, GridPosition},
        level::definitions::LevelDefinitions,
        mushrooms::events::SpawnMushroomEvent,
        game_flow::{CurrentLevel, LevelState},
    },
    screens::Screen,
};

use super::assets::LevelAssets;

pub(super) fn plugin(app: &mut App) {
    // Spawn grid when entering Playing state
    app.add_systems(OnEnter(LevelState::Playing), spawn_level);
}

/// Spawn the main game level
#[tracing::instrument(name = "Spawn level", skip_all)]
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    grid_config: Res<GridConfig>,
    current_level: Res<CurrentLevel>,
    level_definitions: Res<LevelDefinitions>,
) {
    // Get level definition
    let level_def = level_definitions
        .get_level(current_level.level_index)
        .cloned();

    let level_name = level_def
        .as_ref()
        .map(|l| l.name.as_str())
        .unwrap_or("Unknown Level");

    info!("Spawning level: {}", level_name);

    // Spawn the game grid with current configuration
    spawn_game_grid(&mut commands, &grid_config);

    // Spawn starting mushrooms if any are defined
    if let Some(level_def) = level_def {
        for starting_mushroom in &level_def.starting_mushrooms {
            let position = GridPosition::new(starting_mushroom.x, starting_mushroom.y);

            // Validate position is within bounds
            if position.in_bounds(&grid_config) {
                info!(
                    "Spawning starting {} at ({}, {})",
                    starting_mushroom.mushroom_type.name(),
                    starting_mushroom.x,
                    starting_mushroom.y
                );

                commands.trigger(SpawnMushroomEvent {
                    position,
                    mushroom_type: starting_mushroom.mushroom_type,
                });
            } else {
                warn!(
                    "Starting mushroom position ({}, {}) is out of bounds for {}x{} grid",
                    starting_mushroom.x, starting_mushroom.y, grid_config.width, grid_config.height
                );
            }
        }
    }

    // Spawn background music
    commands.spawn((
        Name::new("Gameplay Music"),
        StateScoped(Screen::Gameplay),
        music(level_assets.music.clone()),
    ));

    // TODO: Spawn level name display that fades out?
}

/// Marker for the main game grid entity
#[derive(Component)]
pub struct GameGrid;

/// Spawn the game grid
#[tracing::instrument(name = "Spawn game grid", skip_all)]
pub fn spawn_game_grid(commands: &mut Commands, config: &GridConfig) {
    let grid_entity = commands
        .spawn((
            Name::new("Game Grid"),
            GameGrid,
            Transform::default(),
            Visibility::default(),
            StateScoped(LevelState::Playing), // Grid cleaned up when level ends
        ))
        .id();

    // Spawn grid cells
    let mut cell_entities = Vec::new();
    for y in 0..config.height {
        for x in 0..config.width {
            let position = GridPosition::new(x, y);
            let cell = commands
                .spawn((
                    Name::new(format!("Grid Cell ({}, {})", x, y)),
                    GridCell { position },
                    Sprite {
                        color: Color::srgba(0.2, 0.2, 0.2, 0.5),
                        custom_size: Some(Vec2::splat(config.cell_size)),
                        ..default()
                    },
                    Transform::from_translation(
                        position.to_world(config) - Vec3::new(0.0, 0.0, 1.0),
                    ),
                    Pickable::default(),
                ))
                .id();
            cell_entities.push(cell);
        }
    }

    // Spawn grid background
    let grid_width =
        config.width as f32 * (config.cell_size + config.cell_spacing) - config.cell_spacing;
    let grid_height =
        config.height as f32 * (config.cell_size + config.cell_spacing) - config.cell_spacing;
    let background = commands
        .spawn((
            Name::new("Grid Background"),
            Sprite {
                color: Color::srgb(0.1, 0.1, 0.1),
                custom_size: Some(Vec2::new(grid_width + 20.0, grid_height + 20.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, -2.0)),
        ))
        .id();

    // Add all children to grid
    commands.entity(grid_entity).add_children(&cell_entities);
    commands.entity(grid_entity).add_child(background);
}
