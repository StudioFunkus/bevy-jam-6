//! Level spawning systems

use super::super::play_field::events::on_grid_cell_click;
use bevy::{pbr::ExtendedMaterial, prelude::*, render::storage::ShaderStorageBuffer};

use crate::{
    audio::{Music, music},
    game::{
        game_flow::{CurrentLevel, LevelLifecycle, LevelState},
        level::{CurrentGameplayMusic, definitions::LevelDefinitions},
        mushrooms::{MushroomDefinitions, events::SpawnMushroomEvent},
        play_field::{
            CELL_SIZE, GridPosition,
            events::GridCell,
            field_renderer::{FieldGroundExtension, spawn_field_ground},
        },
        resources::GameState,
    },
    screens::Screen,
};

use super::assets::LevelAssets;

pub(super) fn plugin(app: &mut App) {
    // Spawn grid when entering Playing state
    app.add_systems(OnEnter(LevelState::StartDialogue), spawn_level);
}

/// Spawn the main game level
#[tracing::instrument(name = "Spawn level", skip_all)]
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    game_state: Res<GameState>,
    current_level: Res<CurrentLevel>,
    level_definitions: Res<LevelDefinitions>,
    mushroom_definitions: Res<MushroomDefinitions>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut field_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, FieldGroundExtension>>>,
    mut images: ResMut<Assets<Image>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    music_query: Query<&AudioPlayer, With<Music>>,
    mut gameplay_music: ResMut<CurrentGameplayMusic>,
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
    spawn_game_grid(
        &mut commands,
        &game_state,
        &mut meshes,
        &mut materials,
        &mut field_materials,
        &mut images,
        &mut buffers,
        &level_assets,
    );

    // Spawn starting mushrooms if any are defined
    if let Some(level_def) = level_def {
        commands.spawn((
            Name::new("Level Background"),
            SceneRoot(level_assets.background_model_1.clone()),
            Transform::from_xyz(8.0, -6.1, 4.5), // Model isn't centered
            StateScoped(LevelLifecycle::Active),
        ));

        for starting_mushroom in &level_def.starting_mushrooms {
            let position = GridPosition::new(starting_mushroom.x, starting_mushroom.y);

            // Validate position is within bounds
            if game_state.play_field.contains(position) {
                info!(
                    "Spawning starting {} at ({}, {})",
                    mushroom_definitions
                        .get(starting_mushroom.mushroom_type)
                        .map_or("Unknown", |d| d.name.as_str()),
                    starting_mushroom.x,
                    starting_mushroom.y
                );

                commands.trigger(SpawnMushroomEvent {
                    position,
                    mushroom_type: starting_mushroom.mushroom_type,
                    direction: None,
                });
            } else {
                warn!(
                    "Starting mushroom position ({}, {}) is out of bounds for {}x{} grid",
                    starting_mushroom.x,
                    starting_mushroom.y,
                    game_state.play_field.width,
                    game_state.play_field.height
                );
            }
        }
    }

    // Check if we need to change music
    let mut current_music = music_query
        .iter()
        .find(|player| player.0 == level_assets.music)
        .is_some();

    // Only spawn new music if it's different from what's playing
    if gameplay_music.current_track.as_ref() != Some(&level_assets.music) {
        // Despawn old music if it exists
        if let Some(entity) = gameplay_music.entity {
            commands.entity(entity).despawn();
        }

        // Spawn new music
        let music_entity = commands
            .spawn((
                Name::new("Gameplay Music"),
                StateScoped(Screen::Gameplay),
                music(level_assets.music.clone()),
            ))
            .id();

        // Update tracking resource
        gameplay_music.current_track = Some(level_assets.music.clone());
        gameplay_music.entity = Some(music_entity);

        info!("Changed gameplay music track");
    } else {
        info!("Keeping current music track");
    }
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
    field_materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial, FieldGroundExtension>>>,
    images: &mut ResMut<Assets<Image>>,
    buffers: &mut ResMut<Assets<ShaderStorageBuffer>>,
    level_assets: &Res<LevelAssets>,
) {
    // Spawn the custom field ground that handles tile and mycelium rendering
    spawn_field_ground(
        commands,
        meshes,
        field_materials,
        images,
        buffers,
        level_assets,
        &game_state.play_field,
    );

    let grid_entity = commands
        .spawn((
            Name::new("Game Grid"),
            GameGrid,
            Transform::default(),
            Visibility::default(),
            StateScoped(LevelState::Playing), // Grid only exists during gameplay
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
                    // Invisible collider for click detection only
                    Mesh3d(meshes.add(Rectangle::new(CELL_SIZE, CELL_SIZE))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::NONE, // Fully transparent
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..default()
                    })),
                    Transform::from_xyz(world_pos.x, 0.1, -world_pos.z) // Slightly above ground
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                ))
                .observe(on_grid_cell_click)
                .id();
            cell_entities.push(cell);
        }
    }

    // Add all children to grid
    commands.entity(grid_entity).add_children(&cell_entities);
}
