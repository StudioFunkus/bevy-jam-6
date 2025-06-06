use bevy::{pbr::NotShadowReceiver, prelude::*};
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams};

use crate::game::{
    game_flow::{LevelState, TurnPhase},
    level::assets::LevelAssets,
    play_field::{observers::find_entity_at, placement_preview::PreviewState, GridClickEvent, GridPosition, TileGrid},
    resources::GameState,
    visual_effects::FaceCamera,
};

pub use chain_activation::{ChainManager, MushroomActivationState};
pub use definitions::{MushroomDefinitions, MushroomType};
pub use events::SpawnMushroomEvent;
pub use resources::SelectedMushroomType;

pub mod chain_activation;
pub mod definitions;
pub mod events;
pub mod resources;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        definitions::plugin,
        chain_activation::plugin,
        events::plugin,
    ));

    // Initialize resources
    app.init_resource::<SelectedMushroomType>();

    // Add main mushroom systems
    app.add_observer(handle_grid_clicks)
        .add_observer(spawn_mushroom);

    // Add turn phase transitions
    app.add_systems(OnEnter(TurnPhase::Chain), reset_chain_phase);
}

/// Marker component for mushroom entities
#[derive(Component)]
#[require(MushroomActivationState, MushroomDirection, GridPosition)]
pub struct Mushroom(pub MushroomType);

/// Direction component for mushrooms
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MushroomDirection {
    #[default]
    Up,
    Right,
    Down,
    Left,
}

impl MushroomDirection {
    pub fn rotate_clockwise(&self) -> MushroomDirection {
        match self {
            MushroomDirection::Up => MushroomDirection::Right,
            MushroomDirection::Right => MushroomDirection::Down,
            MushroomDirection::Down => MushroomDirection::Left,
            MushroomDirection::Left => MushroomDirection::Up,
        }
    }
}

/// Handle grid clicks based on game phase
#[tracing::instrument(name = "Handle grid clicks", skip_all)]
fn handle_grid_clicks(
    trigger: Trigger<GridClickEvent>,
    commands: Commands,
    selected_type: Res<SelectedMushroomType>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mushrooms: Query<(&Mushroom, Option<&MushroomDirection>)>,
    chain_manager: ResMut<ChainManager>,
    definitions: Res<MushroomDefinitions>,
    game_state: ResMut<GameState>,
    current_phase: Option<Res<State<TurnPhase>>>,
    current_level: Res<crate::game::game_flow::CurrentLevel>,
    preview_state: Res<PreviewState>,
    hovered_cell: Res<crate::game::play_field::placement_preview::HoveredCell>,
    tile_grid: Res<TileGrid>,
) {
    info!("Grid click at {:?}", trigger.position);

    if !game_state.play_field.contains(trigger.position) {
        return;
    }

    let Some(phase_state) = current_phase else {
        info!("Not in a game turn phase");
        return;
    };

    match phase_state.get() {
        TurnPhase::Planting => handle_planting_click(
            trigger.event(),
            commands,
            selected_type,
            keyboard,
            mushrooms,
            definitions,
            game_state,
            current_level.level_index,
            preview_state,
            hovered_cell,
            tile_grid,
        ),
        TurnPhase::Chain => {
            handle_chain_click(trigger.event(), commands, chain_manager, game_state)
        }
        _ => {
            info!(
                "Cannot interact with mushrooms during {:?} phase",
                phase_state.get()
            );
        }
    }
}

/// Handle clicks during planting phase
#[tracing::instrument(name = "Handle planting click", skip_all)]
fn handle_planting_click(
    event: &GridClickEvent,
    mut commands: Commands,
    selected_type: Res<SelectedMushroomType>,
    _keyboard: Res<ButtonInput<KeyCode>>,
    _mushrooms: Query<(&Mushroom, Option<&MushroomDirection>)>,
    definitions: Res<MushroomDefinitions>,
    mut game_state: ResMut<GameState>,
    current_level: usize,
    preview_state: Res<PreviewState>,
    hovered_cell: Res<crate::game::play_field::placement_preview::HoveredCell>,
    tile_grid: Res<TileGrid>,
) {
    // Right-click to delete
    if event.button == bevy::picking::pointer::PointerButton::Secondary {
        if let Some(entity) = find_entity_at(event.position, &game_state) {
            info!("Deleting mushroom at {:?}", event.position);
            commands.entity(entity).despawn();
            game_state.play_field.remove(event.position);
        }
        return;
    }

    // Only handle left clicks from here
    if event.button != bevy::picking::pointer::PointerButton::Primary {
        return;
    }

    // Check for existing mushroom
    if find_entity_at(event.position, &game_state).is_some() {
        // No rotation on existing mushrooms anymore
        // Rotation is handled during preview phase
        return;
    }

    // Only place if we have a valid preview at this position
    if hovered_cell.position != Some(event.position) {
        info!("No preview at clicked position");
        return;
    }

    // Validate placement position
    if !game_state.play_field.contains(event.position) {
        return;
    }

    // Check tile type
    if let Some(tile_type) = tile_grid.get(event.position) {
        if !tile_type.allows_mushroom() {
            info!("Cannot place mushroom on {:?} tile", tile_type);
            return;
        }
    }

    // Check if mushroom type is unlocked
    if !definitions.is_unlocked(selected_type.mushroom_type, &game_state, current_level) {
        info!(
            "Mushroom type {:?} is not unlocked",
            selected_type.mushroom_type
        );
        return;
    }

    // Spawn new mushroom with the preview's rotation
    info!(
        "Spawning {:?} at {:?} with rotation {:?}",
        selected_type.mushroom_type, event.position, preview_state.direction
    );

    commands.trigger(SpawnMushroomEvent {
        position: event.position,
        mushroom_type: selected_type.mushroom_type,
        direction: Some(preview_state.direction),
    });
}

/// Handle clicks during chain phase
#[tracing::instrument(name = "Handle chain click", skip_all)]
fn handle_chain_click(
    event: &GridClickEvent,
    _commands: Commands,
    mut chain_manager: ResMut<ChainManager>,
    game_state: ResMut<GameState>,
) {
    if event.button != bevy::picking::pointer::PointerButton::Primary {
        return;
    }

    if let Some(entity) = find_entity_at(event.position, &game_state) {
        // Try to start a chain from this mushroom
        match chain_manager.start_chain(entity, event.position) {
            Some(chain_id) => {
                info!(
                    "Started chain {} from mushroom at {:?}",
                    chain_id, event.position
                );
                // Could trigger visual effects here
            }
            None => {
                info!("Cannot start another chain this turn");
            }
        }
    } else {
        info!("No mushroom at {:?} to start chain", event.position);
    }
}

/// Spawn a mushroom entity
#[tracing::instrument(name = "Spawn mushroom", skip_all)]
fn spawn_mushroom(
    trigger: Trigger<SpawnMushroomEvent>,
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut sprite_params: Sprite3dParams,
    level_assets: Res<LevelAssets>,
    definitions: Res<MushroomDefinitions>,
    preview_state: Res<PreviewState>,
) {
    let Some(definition) = definitions.get(trigger.mushroom_type) else {
        warn!(
            "No definition for mushroom type {:?}",
            trigger.mushroom_type
        );
        return;
    };

    let world_pos = trigger.position.to_world_in(&game_state.play_field);

    // Create texture atlas for sprites
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        2, // columns (front/side views)
        8, // rows (mushroom types)
        Some(UVec2::new(2, 2)),
        None,
    );
    let layout_handle = sprite_params.atlas_layouts.add(layout);

    let atlas = TextureAtlas {
        layout: layout_handle.clone(),
        index: definition.sprite_row * 2,
    };

    // Use direction from event or preview state
    let direction = trigger.event().direction.unwrap_or(preview_state.direction);

    // Spawn mushroom entity
    let entity_commands = commands.spawn((
        Name::new(format!(
            "{} at ({}, {})",
            definition.name, trigger.position.x, trigger.position.y
        )),
        Mushroom(trigger.mushroom_type),
        trigger.position,
        direction, // Use the direction from preview
        Sprite3dBuilder {
            image: level_assets.mushroom_texture.clone(),
            pixels_per_metre: 16.0,
            double_sided: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }
        .bundle_with_atlas(&mut sprite_params, atlas),
        Transform::from_xyz(world_pos.x, 0.5, -world_pos.y),
        FaceCamera,
        NotShadowReceiver,
        StateScoped(LevelState::Playing),
    ));

    let entity = entity_commands.id();

    // Update play field
    game_state
        .play_field
        .entities
        .insert(trigger.position, entity);

    info!(
        "Spawned {} entity {:?} with direction {:?}",
        definition.name, entity, direction
    );
}

/// Reset systems when entering chain phase
fn reset_chain_phase(
    mut chain_manager: ResMut<ChainManager>,
    mut mushroom_states: Query<&mut MushroomActivationState>,
) {
    info!("=== CHAIN PHASE STARTED ===");

    // Reset chain manager for new chain phase
    chain_manager.reset_turn();

    // Reset mushroom activation counts
    for mut state in mushroom_states.iter_mut() {
        state.activations_this_turn = 0;
        state.last_activation_energy = 0.0;
    }

    info!("Click a mushroom to begin a chain reaction!");
}
