//! Spawn the game level

use bevy::prelude::*;

use super::grid::{GridCell, GridConfig, GridPosition};
use crate::{asset_tracking::LoadResource, audio::music, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
        }
    }
}

/// Spawn the main game level
#[tracing::instrument(name = "Spawn level", skip_all)]
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    grid_config: Res<GridConfig>,
) {
    // Spawn the game grid
    spawn_game_grid(&mut commands, &grid_config);

    // Spawn background music
    commands.spawn((
        Name::new("Gameplay Music"),
        StateScoped(Screen::Gameplay),
        music(level_assets.music.clone()),
    ));
}

/// Spawn the game grid
#[tracing::instrument(name = "Spawn game grid", skip_all)]
fn spawn_game_grid(commands: &mut Commands, config: &GridConfig) {
    let grid_entity = commands
        .spawn((
            Name::new("Game Grid"),
            Transform::default(),
            Visibility::default(),
            StateScoped(Screen::Gameplay),
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
