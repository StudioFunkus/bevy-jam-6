//! Level spawning systems

use super::super::play_field::events::on_grid_cell_click;
use bevy::{pbr::NotShadowReceiver, prelude::*};

use crate::{
    audio::music,
    game::{
        game_flow::{CurrentLevel, LevelState},
        play_field::{GridPosition, CELL_SIZE, CELL_SPACING, events::GridCell},
        level::definitions::LevelDefinitions,
        mushrooms::events::SpawnMushroomEvent,
        resources::GameState,
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
    game_state: Res<GameState>,
    current_level: Res<CurrentLevel>,
    level_definitions: Res<LevelDefinitions>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    spawn_game_grid(&mut commands, &game_state, &mut meshes, &mut materials);

    // Spawn starting mushrooms if any are defined
    if let Some(level_def) = level_def {
        for starting_mushroom in &level_def.starting_mushrooms {
            let position = GridPosition::new(starting_mushroom.x, starting_mushroom.y);

            // Validate position is within bounds
            if game_state.play_field.contains(position) {
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
                    starting_mushroom.x, starting_mushroom.y, game_state.play_field.width, game_state.play_field.height
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
pub fn spawn_game_grid(
    commands: &mut Commands,
    game_state: &GameState,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Ground Plane"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.3, 0.2),
            ..default()
        })),
        Transform::from_xyz(0.0, -0.5, 0.0),
        NotShadowReceiver,
        StateScoped(LevelState::Playing),
    ));
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
    for y in 0..game_state.play_field.height {
        for x in 0..game_state.play_field.width {
            let position = GridPosition::new(x, y);
            let world_pos = position.to_world_in(&game_state.play_field);
            let cell = commands
                .spawn((
                    Name::new(format!("Grid Cell ({x}, {y})")),
                    GridCell { position },
                    Mesh3d(meshes.add(Rectangle::new(CELL_SIZE, CELL_SIZE))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgba(0.2, 0.2, 0.2, 0.5),
                        alpha_mode: AlphaMode::Opaque,
                        unlit: false,
                        ..default()
                    })),
                    Transform::from_xyz(world_pos.x, 0.0, -world_pos.y)
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                ))
                .observe(on_grid_cell_click)
                .id();
            cell_entities.push(cell);
        }
    }

    // Spawn grid background
    let (grid_width, grid_height) = game_state.play_field.world_size();
    let background = commands
        .spawn((
            Name::new("Grid Background"),
            Mesh3d(meshes.add(Rectangle::new(grid_width + 0.2, grid_height + 0.2))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.1, 0.1, 0.1),
                unlit: false,
                ..default()
            })),
            Transform::from_xyz(0.0, -0.01, 0.0)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ))
        .id();

    // Add all children to grid
    commands.entity(grid_entity).add_children(&cell_entities);
    commands.entity(grid_entity).add_child(background);
}
