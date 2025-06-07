//! Mycelium connection system (PlayField connections)

use super::GridPosition;
use crate::game::mushrooms::{Mushroom, MushroomDefinitions, MushroomDirection, MushroomType};
use crate::game::play_field::PlayField;
use crate::game::resources::GameState;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, build_playfield_connections);
}

/// Connection building state to avoid rebuilding every frame
#[derive(Resource, Default)]
pub struct ConnectionBuilder {
    pub dirty: bool,
}

/// Build mycelium connections in PlayField based on mushroom positions and types
pub fn build_playfield_connections(
    mut commands: Commands,
    new_mushrooms: Query<
        (Entity, &GridPosition, &Mushroom, Option<&MushroomDirection>),
        Added<Mushroom>,
    >,
    changed_mushrooms: Query<
        (Entity, &GridPosition, &Mushroom, Option<&MushroomDirection>),
        Changed<GridPosition>,
    >,
    mut deleted_mushrooms: RemovedComponents<Mushroom>,
    all_mushrooms: Query<(Entity, &GridPosition, &Mushroom, Option<&MushroomDirection>)>,
    mut game_state: ResMut<GameState>,
    definitions: Res<MushroomDefinitions>,
    mut builder: Local<ConnectionBuilder>,
) {
    // Check if we need to rebuild connections
    let needs_rebuild = !new_mushrooms.is_empty()
        || !changed_mushrooms.is_empty()
        || deleted_mushrooms.read().next().is_some()
        || builder.dirty;

    if !needs_rebuild {
        return;
    }

    info!(
        "Building PlayField connections - {} new, {} changed mushrooms",
        new_mushrooms.iter().count(),
        changed_mushrooms.iter().count()
    );

    // Clear existing connections
    game_state.play_field.clear_connections();

    // Get all mushrooms for connection building
    let mushroom_list: Vec<_> = all_mushrooms.iter().collect();

    let mut connection_count = 0;

    // Build connections for each mushroom based on its connection points
    for (entity, pos, mushroom, direction) in mushroom_list.iter() {
        // Get the mushroom definition
        let Some(definition) = definitions.get(mushroom.0) else {
            warn!("No definition found for mushroom type {:?}", mushroom.0);
            continue;
        };

        // Skip if this mushroom has no connections
        if definition.connection_points.is_empty() {
            continue;
        }

        info!(
            "Building connections for {:?} at {:?} with {} connection points",
            mushroom.0,
            pos,
            definition.connection_points.len()
        );

        // Process each connection point
        for connection_point in &definition.connection_points {
            // Calculate the actual target position
            // If the mushroom has a direction component, rotate the connection point
            let target_pos = if let Some(dir) = direction {
                let rotated_offset = rotate_connection_point(connection_point, dir);
                GridPosition::new(pos.x + rotated_offset.x, pos.y + rotated_offset.y)
            } else {
                // No direction component, use the connection point as-is
                GridPosition::new(pos.x + connection_point.x, pos.y + connection_point.y)
            };

            // Check if there's a mushroom at the target position
            if let Some(target_entity) = game_state.play_field.get(target_pos) {
                // Verify it's actually a mushroom
                if mushroom_list.iter().any(|(e, _, _, _)| *e == target_entity) {
                    // Create connection with pathfinding
                    if let Some((path, strength)) =
                        find_mycelium_path(**pos, target_pos, &game_state.play_field)
                    {
                        game_state.play_field.add_connection(
                            **pos,
                            target_pos,
                            *entity,
                            target_entity,
                            strength,
                            path,
                        );
                        connection_count += 1;
                        info!("Created connection from {:?} to {:?}", pos, target_pos);
                    }
                }
            }
        }
    }

    info!(
        "PlayField connection build complete: {} connections created",
        connection_count
    );
    builder.dirty = false;
}

/// Rotate a connection point based on mushroom direction
fn rotate_connection_point(
    point: &crate::game::mushrooms::definitions::GridOffset,
    direction: &MushroomDirection,
) -> crate::game::mushrooms::definitions::GridOffset {
    use crate::game::mushrooms::definitions::GridOffset;

    match direction {
        MushroomDirection::Up => {
            // No rotation needed, this is the default
            GridOffset::new(point.x, point.y)
        }
        MushroomDirection::Right => {
            // Rotate 90 degrees clockwise: (x, y) -> (y, -x)
            GridOffset::new(point.y, -point.x)
        }
        MushroomDirection::Down => {
            // Rotate 180 degrees: (x, y) -> (-x, -y)
            GridOffset::new(-point.x, -point.y)
        }
        MushroomDirection::Left => {
            // Rotate 270 degrees clockwise: (x, y) -> (-y, x)
            GridOffset::new(-point.y, point.x)
        }
    }
}

/// Find a valid path for mycelium between two positions
fn find_mycelium_path(
    from: GridPosition,
    to: GridPosition,
    play_field: &PlayField,
) -> Option<(Vec<GridPosition>, f32)> {
    let path = bresenham_line(from, to);

    let mut total_strength = 1.0;
    let mut can_connect = true;

    // Check each tile along the path
    for pos in &path {
        if let Some(tile) = play_field.get_tile(*pos) {
            if !tile.allows_mycelium() {
                can_connect = false;
                break;
            }
            total_strength *= tile.mycelium_strength_modifier();
        }
    }

    if can_connect && total_strength > 0.0 {
        Some((path, total_strength.min(1.0)))
    } else {
        None
    }
}

/// Bresenham's line algorithm for grid positions
/// https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm
fn bresenham_line(from: GridPosition, to: GridPosition) -> Vec<GridPosition> {
    let mut points = Vec::new();

    let mut x0 = from.x;
    let mut y0 = from.y;
    let x1 = to.x;
    let y1 = to.y;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        points.push(GridPosition::new(x0, y0));

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x0 += sx;
        }
        if e2 < dx {
            err += dx;
            y0 += sy;
        }
    }

    points
}
