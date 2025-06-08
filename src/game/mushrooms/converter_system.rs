//! End-of-turn converter system for mushrooms

use bevy::prelude::*;
use bevy_hanabi::{EffectAsset, ParticleEffect};
use rand::{prelude::*, rng};

use crate::game::{
    game_flow::{LevelState, TurnPhase},
    mushrooms::{Mushroom, MushroomActivationState, MushroomDefinitions},
    play_field::{
        field_renderer::{FieldGround, TilesDirty}, GridPosition, TileType
    },
    resources::GameState, DespawnTimer,
};

use super::definitions::ActivationBehavior;

pub(super) fn plugin(app: &mut App) {
    // Process conversions when entering the Score phase (end of turn)
    app.add_systems(
        OnEnter(TurnPhase::Score),
        process_end_of_turn_conversions.run_if(in_state(LevelState::Playing)),
    );
}

/// Process all converter mushrooms at the end of turn
fn process_end_of_turn_conversions(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mushrooms: Query<(&Mushroom, &GridPosition, &MushroomActivationState)>,
    definitions: Res<MushroomDefinitions>,
    field_ground_query: Query<Entity, With<FieldGround>>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let mut conversions_to_apply = Vec::new();

    // First, collect all conversions to apply
    for (mushroom, position, state) in mushrooms.iter() {
        // Skip if mushroom didn't activate this turn
        if state.activations_this_turn == 0 {
            continue;
        }

        let Some(definition) = definitions.get(mushroom.0) else {
            continue;
        };

        // Check if this is a converter
        if let ActivationBehavior::Converter {
            convert_to,
            convert_count,
            can_convert_from,
            search_radius,
        } = &definition.activation_behavior
        {
            // Find tiles to convert
            let convertible_tiles = find_convertible_tiles(
                *position,
                *convert_to,
                can_convert_from.as_ref(),
                *search_radius,
                &game_state.play_field,
            );

            // Select random tiles up to convert_count
            let mut rng = rng();
            let tiles_to_convert: Vec<_> = convertible_tiles
                .choose_multiple(&mut rng, *convert_count as usize)
                .cloned()
                .collect();

            for tile_pos in tiles_to_convert {
                conversions_to_apply.push((tile_pos, *convert_to));
                info!(
                    "{} at {:?} will convert tile at {:?} to {:?}",
                    definition.name, position, tile_pos, convert_to
                );
            }
        }
    }

    // Apply all conversions
    if !conversions_to_apply.is_empty() {
        for (pos, new_type) in conversions_to_apply {
            game_state.play_field.set_tile(pos, new_type);
            info!("Converted tile at {:?} to {:?}", pos, new_type);

            // Spawn conversion effect at the tile position
            let world_pos = pos.to_world_in(&game_state.play_field);
            let conversion_effect = effects.add(
                crate::game::particles::assets::tile_conversion_effect()
            );
            
            commands.spawn((
                Name::new("Tile Conversion Effect"),
                ParticleEffect::new(conversion_effect),
                Transform::from_translation(Vec3::new(
                    world_pos.x, 
                    0.1,  // Just above ground level
                    -world_pos.z
                )),
                DespawnTimer::new(1.0),
            ));
        }

        // Mark the field ground as needing texture update
        for entity in field_ground_query.iter() {
            commands.entity(entity).insert(TilesDirty);
        }
    }
}

/// Find all tiles that can be converted
fn find_convertible_tiles(
    origin: GridPosition,
    target_type: TileType,
    allowed_sources: Option<&Vec<TileType>>,
    radius: i32,
    play_field: &crate::game::play_field::PlayField,
) -> Vec<GridPosition> {
    let mut convertible = Vec::new();

    // Search in a square radius around the origin
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            // Skip the origin tile
            if dx == 0 && dy == 0 {
                continue;
            }

            let check_pos = GridPosition::new(origin.x + dx, origin.y + dy);

            // Check if position is valid
            if !play_field.contains(check_pos) {
                continue;
            }

            // Get current tile type
            let Some(current_type) = play_field.get_tile(check_pos) else {
                continue;
            };

            // Skip if already the target type
            if current_type == target_type {
                continue;
            }

            // Check if this tile type can be converted
            let can_convert = if let Some(allowed) = allowed_sources {
                allowed.contains(&current_type)
            } else {
                // If no restrictions, can convert any non-target tile
                true
            };

            if can_convert {
                convertible.push(check_pos);
            }
        }
    }

    convertible
}
