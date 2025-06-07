//! Placement preview system for mushroom placement
//!
//! This module handles showing a preview of mushrooms before placement,
//! including highlighting potential connections in the shader.

use bevy::{pbr::NotShadowReceiver, prelude::*};
use bevy_sprite3d::{Sprite3d, Sprite3dBuilder, Sprite3dParams};

use crate::game::{
    game_flow::{LevelState, TurnPhase},
    level::assets::LevelAssets,
    mushrooms::{Mushroom, MushroomDefinitions, MushroomDirection, SelectedMushroomType},
    play_field::{
        GridPosition, PlayField,
        events::GridCell,
    },
    resources::GameState,
    visual_effects::FaceCamera,
};

pub(super) fn plugin(app: &mut App) {
    // Resources
    app.init_resource::<HoveredCell>()
        .init_resource::<PreviewConnections>()
        .init_resource::<PreviewState>();

    // Events
    app.add_event::<CellHoverChanged>();

    // Systems
    app.add_systems(
        Update,
        (
            detect_hover_changes,
            update_placement_preview,
            handle_preview_rotation,
            update_preview_connections,
            update_existing_mushroom_connections,
            apply_preview_transparency,
        )
            .chain()
            .run_if(in_state(TurnPhase::Planting)),
    );

    // Cleanup
    app.add_systems(OnExit(TurnPhase::Planting), cleanup_preview);
    app.add_systems(OnExit(LevelState::Playing), clear_preview_connections);
    app.add_systems(OnEnter(LevelState::Playing), clear_preview_connections);
}

/// Resource tracking the currently hovered grid cell
#[derive(Resource, Default, Debug)]
pub struct HoveredCell {
    pub position: Option<GridPosition>,
}

/// Resource storing which tiles should be highlighted for connection preview
#[derive(Resource, Default, Debug)]
pub struct PreviewConnections {
    /// Grid positions that would be connected (green highlight)
    pub connected_positions: Vec<GridPosition>,
    /// Grid positions where connections would be attempted but no mushroom exists (red highlight)
    pub empty_connection_points: Vec<GridPosition>,
    /// Grid positions where existing mushrooms can connect to (blue outline)
    pub existing_connection_targets: Vec<GridPosition>,
    /// The preview mushroom's position
    pub preview_position: Option<GridPosition>,
}

/// State of the placement preview
#[derive(Resource, Default, Debug)]
pub struct PreviewState {
    /// Current direction of the preview
    pub direction: MushroomDirection,
    /// Entity of the preview mushroom
    pub preview_entity: Option<Entity>,
}

/// Event fired when the hovered cell changes
#[derive(Event, Debug)]
pub struct CellHoverChanged {
    pub old_position: Option<GridPosition>,
    pub new_position: Option<GridPosition>,
}

/// Component marking the preview entity
#[derive(Component)]
pub struct PlacementPreview;

/// Marker for transparency handling
#[derive(Component)]
struct PreviewMarker;

/// Clear preview connections when changing levels
fn clear_preview_connections(
    mut preview_connections: ResMut<PreviewConnections>,
    mut hovered_cell: ResMut<HoveredCell>,
) {
    preview_connections.connected_positions.clear();
    preview_connections.empty_connection_points.clear();
    preview_connections.existing_connection_targets.clear();
    preview_connections.preview_position = None;
    hovered_cell.position = None;
    
    info!("Cleared preview connections for level transition");
}

/// Detect when the cursor hovers over different grid cells
fn detect_hover_changes(
    mut events: EventReader<Pointer<Over>>,
    mut hover_out_events: EventReader<Pointer<Out>>,
    mut hovered_cell: ResMut<HoveredCell>,
    mut hover_changed: EventWriter<CellHoverChanged>,
    grid_cells: Query<&GridCell>,
) {
    // Handle hover out events
    for event in hover_out_events.read() {
        if grid_cells.contains(event.target) {
            let old_position = hovered_cell.position;
            hovered_cell.position = None;
            hover_changed.write(CellHoverChanged {
                old_position,
                new_position: None,
            });
        }
    }

    // Handle hover over events
    for event in events.read() {
        if let Ok(cell) = grid_cells.get(event.target) {
            if hovered_cell.position != Some(cell.position) {
                let old_position = hovered_cell.position;
                hovered_cell.position = Some(cell.position);
                hover_changed.write(CellHoverChanged {
                    old_position,
                    new_position: Some(cell.position),
                });
            }
        }
    }
}

/// Update the placement preview entity
fn update_placement_preview(
    mut commands: Commands,
    mut hover_changed: EventReader<CellHoverChanged>,
    mut preview_state: ResMut<PreviewState>,
    selected_type: Res<SelectedMushroomType>,
    definitions: Res<MushroomDefinitions>,
    play_field: Res<GameState>,
    level_assets: Res<LevelAssets>,
    mut sprite_params: Sprite3dParams,
    mut preview_query: Query<&mut Transform, With<PlacementPreview>>,
) {
    for event in hover_changed.read() {
        match event.new_position {
            Some(position) => {
                // Check if position is valid for placement
                if !is_valid_placement_position(&position, &play_field.play_field) {
                    // Hide preview if position is invalid
                    if let Some(entity) = preview_state.preview_entity {
                        commands.entity(entity).despawn();
                        preview_state.preview_entity = None;
                    }
                    continue;
                }

                // Update existing preview or create new one
                if let Some(entity) = preview_state.preview_entity {
                    // Update position
                    if let Ok(mut transform) = preview_query.get_mut(entity) {
                        let world_pos = position.to_world_in(&play_field.play_field);
                        transform.translation = Vec3::new(world_pos.x, 0.5, -world_pos.z);
                    }
                } else {
                    // Create new preview entity
                    let Some(definition) = definitions.get(selected_type.mushroom_type) else {
                        continue;
                    };

                    let world_pos = position.to_world_in(&play_field.play_field);

                    // Create texture atlas for sprites
                    let layout = TextureAtlasLayout::from_grid(
                        UVec2::new(16, 16),
                        2,
                        24,
                        Some(UVec2::new(2, 2)),
                        None,
                    );
                    let layout_handle = sprite_params.atlas_layouts.add(layout);

                    let atlas = TextureAtlas {
                        layout: layout_handle.clone(),
                        index: definition.sprite_row * 2,
                    };

                    // Create the sprite builder with transparency settings
                    let mut sprite_builder = Sprite3dBuilder {
                        image: level_assets.mushroom_texture.clone(),
                        pixels_per_metre: 16.0,
                        double_sided: true,
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..default()
                    };

                    // Build the sprite bundle
                    let sprite_bundle = sprite_builder.bundle_with_atlas(&mut sprite_params, atlas);

                    // Spawn preview entity
                    let entity = commands
                        .spawn((
                            Name::new("Placement Preview"),
                            PlacementPreview,
                            preview_state.direction,
                            sprite_bundle,
                            Transform::from_xyz(world_pos.x, 0.5, -world_pos.z),
                            FaceCamera,
                            NotShadowReceiver,
                            StateScoped(LevelState::Playing),
                            // Add a custom component to track this is a preview
                            PreviewMarker,
                            Pickable::IGNORE,
                        ))
                        .id();

                    preview_state.preview_entity = Some(entity);
                }
            }
            None => {
                // Remove preview when not hovering
                if let Some(entity) = preview_state.preview_entity.take() {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

/// Handle rotation input for the preview
fn handle_preview_rotation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut preview_state: ResMut<PreviewState>,
    mut preview_query: Query<&mut MushroomDirection, With<PlacementPreview>>,
    hovered_cell: Res<HoveredCell>,
) {
    // Only rotate if we have a hovered cell and preview
    if hovered_cell.position.is_none() || preview_state.preview_entity.is_none() {
        return;
    }

    // Check for rotation input (R key)
    if keyboard.just_pressed(KeyCode::KeyR) {
        preview_state.direction = preview_state.direction.rotate_clockwise();

        // Update the preview entity
        if let Some(entity) = preview_state.preview_entity {
            if let Ok(mut direction) = preview_query.get_mut(entity) {
                *direction = preview_state.direction;
            }
        }
    }
}

/// Calculate which tiles would be connected by placing the mushroom
fn update_preview_connections(
    mut preview_connections: ResMut<PreviewConnections>,
    hovered_cell: Res<HoveredCell>,
    preview_state: Res<PreviewState>,
    selected_type: Res<SelectedMushroomType>,
    definitions: Res<MushroomDefinitions>,
    play_field: Res<GameState>,
) {
    // Clear previous connections (but keep existing mushroom connections)
    preview_connections.connected_positions.clear();
    preview_connections.empty_connection_points.clear();
    preview_connections.preview_position = None;

    // Only calculate if we have a valid hover position
    let Some(position) = hovered_cell.position else {
        return;
    };

    // Check if position is valid
    if !is_valid_placement_position(&position, &play_field.play_field) {
        return;
    }

    preview_connections.preview_position = Some(position);

    // Get mushroom definition
    let Some(definition) = definitions.get(selected_type.mushroom_type) else {
        return;
    };

    // Calculate connection points and categorize them
    for connection_point in &definition.connection_points {
        // Rotate connection point based on preview rotation
        let rotated_offset = rotate_connection_point(connection_point, &preview_state.direction);
        let target_pos =
            GridPosition::new(position.x + rotated_offset.x, position.y + rotated_offset.y);

        // Check if target position is within bounds
        if !play_field.play_field.contains(target_pos) {
            continue;
        }

        // Check if target tile allows mushroom placement (for visual feedback)
        let target_allows_mushroom = play_field
            .play_field
            .get_tile(target_pos)
            .map(|tile| tile.allows_mushroom())
            .unwrap_or(false);

        if !target_allows_mushroom {
            continue; // Don't highlight blocked tiles
        }

        // Check if there's a mushroom at the target position
        if play_field.play_field.get(target_pos).is_some() {
            // Will connect (green highlight)
            preview_connections.connected_positions.push(target_pos);
        } else {
            // Connection point but no mushroom (red highlight)
            preview_connections.empty_connection_points.push(target_pos);
        }
    }
}

/// Show connection targets for all existing mushrooms during placement
fn update_existing_mushroom_connections(
    mut preview_connections: ResMut<PreviewConnections>,
    all_mushrooms: Query<(Entity, &GridPosition, &Mushroom, Option<&MushroomDirection>)>,
    definitions: Res<MushroomDefinitions>,
    play_field: Res<GameState>,
) {
    // Clear existing connection targets
    preview_connections.existing_connection_targets.clear();

    // Calculate connection targets for all placed mushrooms
    for (entity, pos, mushroom, direction) in all_mushrooms.iter() {
        let Some(definition) = definitions.get(mushroom.0) else {
            continue;
        };

        // Skip mushrooms with no connection points
        if definition.connection_points.is_empty() {
            continue;
        }

        // Calculate where this mushroom can connect to
        for connection_point in &definition.connection_points {
            // Rotate connection point based on mushroom direction
            let rotated_offset = if let Some(dir) = direction {
                rotate_connection_point(connection_point, dir)
            } else {
                rotate_connection_point(connection_point, &MushroomDirection::Up)
            };

            let target_pos = GridPosition::new(pos.x + rotated_offset.x, pos.y + rotated_offset.y);

            // Check if target position is within bounds and allows mushrooms
            if !play_field.play_field.contains(target_pos) {
                continue;
            }

            let target_allows_mushroom = play_field
                .play_field
                .get_tile(target_pos)
                .map(|tile| tile.allows_mushroom())
                .unwrap_or(false);

            if !target_allows_mushroom {
                continue;
            }

            // Only highlight empty positions (where you could place a mushroom)
            if play_field.play_field.get(target_pos).is_none() {
                preview_connections
                    .existing_connection_targets
                    .push(target_pos);
            }
        }
    }
}

/// Check if a position is valid for mushroom placement
fn is_valid_placement_position(position: &GridPosition, play_field: &PlayField) -> bool {
    // Check bounds
    if !play_field.contains(*position) {
        return false;
    }

    // Check if already occupied
    if play_field.get(*position).is_some() {
        return false;
    }

    // Check tile type
    if let Some(tile_type) = play_field.get_tile(*position) {
        return tile_type.allows_mushroom();
    }

    true
}

/// Rotate a connection point based on direction
fn rotate_connection_point(
    point: &crate::game::mushrooms::definitions::GridOffset,
    direction: &MushroomDirection,
) -> crate::game::mushrooms::definitions::GridOffset {
    use crate::game::mushrooms::definitions::GridOffset;

    match direction {
        MushroomDirection::Up => GridOffset::new(point.x, point.y),
        MushroomDirection::Right => GridOffset::new(point.y, -point.x),
        MushroomDirection::Down => GridOffset::new(-point.x, -point.y),
        MushroomDirection::Left => GridOffset::new(-point.y, point.x),
    }
}

/// Cleanup preview when exiting planting phase
fn cleanup_preview(
    mut commands: Commands,
    mut preview_state: ResMut<PreviewState>,
    mut preview_connections: ResMut<PreviewConnections>,
    mut hovered_cell: ResMut<HoveredCell>,
) {
    // Despawn preview entity
    if let Some(entity) = preview_state.preview_entity.take() {
        commands.entity(entity).despawn();
    }

    // Clear resources
    *preview_state = PreviewState::default();
    preview_connections.connected_positions.clear();
    preview_connections.empty_connection_points.clear();
    preview_connections.existing_connection_targets.clear();
    preview_connections.preview_position = None;
    hovered_cell.position = None;
}

/// Apply transparency to preview materials
fn apply_preview_transparency(
    mut materials: ResMut<Assets<StandardMaterial>>,
    preview_query: Query<
        &MeshMaterial3d<StandardMaterial>,
        (With<PreviewMarker>, Added<MeshMaterial3d<StandardMaterial>>),
    >,
) {
    for material_handle in preview_query.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color = Color::srgba(1.0, 1.0, 1.0, 0.5);
            material.alpha_mode = AlphaMode::Blend;
        }
    }
}
