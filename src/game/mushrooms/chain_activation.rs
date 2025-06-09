//! Chain activation system

use bevy::prelude::*;
use bevy_hanabi::{EffectAsset, ParticleEffect};
use std::collections::VecDeque;
use std::time::Duration;

use crate::game::{
    DespawnTimer,
    fixed_timestep::GameTime,
    game_flow::{CurrentLevel, TurnData},
    mushrooms::events::SporeScoreEvent,
    particles::assets::activate_effect,
    play_field::GridPosition,
    resources::GameState,
    visual_effects::ActivationAnimation,
};

use super::{
    Mushroom, MushroomDirection,
    definitions::{ActivationBehavior, GridOffset, MushroomDefinition, MushroomDefinitions},
};

/// A chain represents a single activation sequence
#[derive(Debug, Clone)]
pub struct Chain {
    /// Unique ID for this chain
    pub id: u32,
    /// The mushroom that started this chain
    #[allow(dead_code)]
    pub starter: Entity,
    /// All activations in this chain
    pub activations: Vec<ChainActivation>,
    /// Total spores produced by this chain
    pub total_spores: f64,
    /// Whether this chain is still active
    pub active: bool,
}

/// A single activation within a chain
/// This is for storing the information about each activation, it is not functional
/// may be used in future for replay system or debugging purposes
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ChainActivation {
    pub entity: Entity,
    pub position: GridPosition,
    pub energy: f32,
    pub depth: u32,
    pub parent: Option<Entity>,
}

/// Energy packet traveling through the network
#[derive(Clone, Debug)]
pub struct EnergyPacket {
    pub energy: f32,
    pub source_entity: Entity,
    /// Full path of entities visited in order
    pub path: Vec<Entity>,
}

/// Resource for managing active chains
#[derive(Resource, Default)]
pub struct ChainManager {
    /// All chains that have been started in this run
    pub chains: Vec<Chain>,
    /// Counter for chain IDs
    next_chain_id: u32,
    /// Queue of pending activations
    pub activation_queue: VecDeque<PendingActivation>,
    /// Currently processing chain
    pub current_chain: Option<u32>,
    /// Has a chain been started this turn?
    pub chain_started_this_turn: bool,
}

/// A pending activation waiting to be processed
#[derive(Debug, Clone)]
pub struct PendingActivation {
    pub entity: Entity,
    pub energy_packet: EnergyPacket,
    pub delay: Timer,
    pub chain_id: u32,
}

/// Component tracking per-mushroom activation state
#[derive(Component, Debug, Clone, Default)]
pub struct MushroomActivationState {
    pub activations_this_turn: u32,
    pub last_activation_energy: f32,
    pub cooldown_timer: Option<Timer>,
}

impl ChainManager {
    /// Start a new chain from a mushroom
    pub fn start_chain(&mut self, starter: Entity, position: GridPosition) -> Option<u32> {
        if self.chain_started_this_turn {
            info!("Chain already started this turn!");
            return None;
        }

        let chain_id = self.next_chain_id;
        self.next_chain_id += 1;

        let chain = Chain {
            id: chain_id,
            starter,
            activations: vec![],
            total_spores: 0.0,
            active: true,
        };

        self.chains.push(chain);
        self.current_chain = Some(chain_id);
        self.chain_started_this_turn = true;

        // Queue the initial activation
        let energy_packet = EnergyPacket {
            energy: 1.0,
            source_entity: starter,
            path: vec![],
        };

        self.queue_activation(starter, energy_packet, 0.0, chain_id);

        info!("Started chain {} from mushroom at {:?}", chain_id, position);
        Some(chain_id)
    }

    /// Queue an activation for processing
    pub fn queue_activation(
        &mut self,
        entity: Entity,
        energy_packet: EnergyPacket,
        delay: f32,
        chain_id: u32,
    ) {
        self.activation_queue.push_back(PendingActivation {
            entity,
            energy_packet,
            delay: Timer::from_seconds(delay, TimerMode::Once),
            chain_id,
        });
    }

    /// Get the currently active chain
    #[allow(dead_code)]
    pub fn get_chain(&self, chain_id: u32) -> Option<&Chain> {
        self.chains.iter().find(|c| c.id == chain_id)
    }

    /// Get a mutable reference to a chain
    pub fn get_chain_mut(&mut self, chain_id: u32) -> Option<&mut Chain> {
        self.chains.iter_mut().find(|c| c.id == chain_id)
    }

    /// Check if any chains are still processing
    pub fn has_active_chains(&self) -> bool {
        !self.activation_queue.is_empty() || self.chains.iter().any(|c| c.active)
    }

    /// Reset for new turn
    pub fn reset_turn(&mut self) {
        self.chains.clear();
        self.activation_queue.clear();
        self.current_chain = None;
        self.chain_started_this_turn = false;
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ChainManager>().add_systems(
        FixedUpdate,
        (process_activation_queue, update_mushroom_cooldowns).chain(),
    );
}

/// Process pending activations in the queue
fn process_activation_queue(
    mut commands: Commands,
    mut chain_manager: ResMut<ChainManager>,
    mut game_state: ResMut<GameState>,
    mut turn_data: ResMut<TurnData>,
    mut current_level: ResMut<CurrentLevel>,
    time: Res<GameTime>,
    definitions: Res<MushroomDefinitions>,
    mut mushrooms: Query<(
        &Mushroom,
        &mut MushroomActivationState,
        &GridPosition,
        Option<&MushroomDirection>,
        &Transform,
    )>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    // Update timers and collect ready activations
    let mut ready_activations = Vec::new();

    for activation in &mut chain_manager.activation_queue {
        activation
            .delay
            .tick(Duration::from_secs_f32(time.delta_seconds));
        if activation.delay.finished() {
            ready_activations.push(activation.clone());
        }
    }

    // Remove processed activations
    chain_manager
        .activation_queue
        .retain(|a| !a.delay.finished());

    // Process ready activations
    for activation in ready_activations {
        process_single_activation(
            &mut commands,
            &mut chain_manager,
            &mut game_state,
            &mut turn_data,
            &mut current_level,
            &definitions,
            &mut effects,
            &mut mushrooms,
            activation,
        );
    }

    // Check if chains are complete
    let completed_chains: Vec<(u32, f64)> = chain_manager
        .chains
        .iter()
        .filter(|chain| {
            chain.active
                && !chain_manager
                    .activation_queue
                    .iter()
                    .any(|a| a.chain_id == chain.id)
        })
        .map(|chain| (chain.id, chain.total_spores))
        .collect();

    // Mark completed chains as inactive
    for (chain_id, total_spores) in completed_chains {
        if let Some(chain) = chain_manager.chains.iter_mut().find(|c| c.id == chain_id) {
            chain.active = false;
            info!(
                "Chain {} complete! Total spores: {}",
                chain_id, total_spores
            );
        }
    }
}

/// Process a single mushroom activation
fn process_single_activation(
    commands: &mut Commands,
    chain_manager: &mut ChainManager,
    game_state: &mut GameState,
    turn_data: &mut TurnData,
    _current_level: &mut CurrentLevel,
    definitions: &MushroomDefinitions,
    effects: &mut ResMut<Assets<EffectAsset>>,
    mushrooms: &mut Query<(
        &Mushroom,
        &mut MushroomActivationState,
        &GridPosition,
        Option<&MushroomDirection>,
        &Transform,
    )>,
    activation: PendingActivation,
) {
    let Ok((mushroom, mut state, position, direction, transform)) =
        mushrooms.get_mut(activation.entity)
    else {
        warn!("Mushroom entity {:?} not found", activation.entity);
        return;
    };

    let Some(definition) = definitions.get(mushroom.0) else {
        warn!("No definition for mushroom type {:?}", mushroom.0);
        return;
    };

    // Check if mushroom can activate
    if state.activations_this_turn >= definition.max_uses_per_turn {
        info!("Mushroom at {:?} reached max activations", position);
        return;
    }

    if state.cooldown_timer.is_some() {
        info!("Mushroom at {:?} is on cooldown", position);
        return;
    }

    // Activate the mushroom
    state.activations_this_turn += 1;
    state.last_activation_energy = activation.energy_packet.energy;
    state.cooldown_timer = Some(Timer::from_seconds(
        definition.cooldown_time,
        TimerMode::Once,
    ));

    // Calculate spore production
    let tile_modifier = game_state
        .play_field
        .get_tile(*position)
        .map(|t| t.production_multiplier())
        .unwrap_or(1.0);

    let mut production =
        definition.base_production * activation.energy_packet.energy as f64 * tile_modifier as f64;

    // Apply behavior-specific modifications
    let should_propagate = match &definition.activation_behavior {
        ActivationBehavior::Deleter => {
            // First, collect all mushrooms to delete
            let mut targets_to_delete = Vec::new();

            // Calculate target positions based on connection points
            for connection_point in &definition.connection_points {
                let target_pos = if let Some(dir) = direction {
                    let rotated = rotate_connection_point(connection_point, dir);
                    GridPosition::new(position.x + rotated.x, position.y + rotated.y)
                } else {
                    GridPosition::new(
                        position.x + connection_point.x,
                        position.y + connection_point.y,
                    )
                };

                // Check if there's a mushroom at the target position
                if let Some(target_entity) = game_state.play_field.get(target_pos) {
                    targets_to_delete.push((target_entity, target_pos));
                }
            }

            // Process deletions
            let mut deleted_count = 0;
            for (target_entity, target_pos) in targets_to_delete {
                // Spawn delete effect at target position before deletion
                let delete_effect = effects.add(crate::game::particles::assets::delete_effect());
                let target_world_pos = target_pos.to_world_in(&game_state.play_field);

                commands.spawn((
                    Name::new("Delete Effect"),
                    ParticleEffect::new(delete_effect),
                    Transform::from_translation(Vec3::new(
                        target_world_pos.x,
                        0.7,
                        -target_world_pos.z,
                    )),
                    DespawnTimer::new(1.0),
                ));

                // Delete the mushroom
                commands.entity(target_entity).despawn();
                game_state.play_field.remove(target_pos);
                deleted_count += 1;
                info!(
                    "Deleter mushroom at {:?} destroyed mushroom at {:?}",
                    position, target_pos
                );
            }

            // Award bonus spores: base production * number of mushrooms destroyed
            if deleted_count > 0 {
                production *= deleted_count as f64;
                info!(
                    "Deleter mushroom destroyed {} mushrooms, production multiplied to: {}",
                    deleted_count, production
                );
            } else {
                production = 0.0; // No targets, no production
                info!(
                    "Deleter mushroom found no targets to destroy, producing base amount: {}",
                    production
                );
            }

            // Deleter does not propagate
            false
        }
        _ => true, // Other behaviors propagate normally
    };

    // Add spores
    game_state.add_spores(production);
    turn_data.spores_this_chain += production;
    game_state.total_activations += 1;

    // Spawn spore popup
    commands.trigger(SporeScoreEvent {
        position: *position,
        production,
    });

    //Spawn particle effect
    let activate_effect = effects.add(activate_effect());

    let world_pos = position.to_world_in(&game_state.play_field);

    commands.spawn((
        Name::new("Spore Effect"),
        ParticleEffect::new(activate_effect),
        Transform::from_translation(Vec3::new(world_pos.x, 0.7, -world_pos.z)),
        DespawnTimer::new(1.0),
    ));

    // Update chain
    if let Some(chain) = chain_manager.get_chain_mut(activation.chain_id) {
        chain.total_spores += production;
        chain.activations.push(ChainActivation {
            entity: activation.entity,
            position: *position,
            energy: activation.energy_packet.energy,
            depth: activation.energy_packet.path.len() as u32,
            parent: Some(activation.energy_packet.source_entity),
        });
    }

    info!("Mushroom at {:?} produced {} spores", position, production);

    // Add activation animation
    let original_scale = transform.scale;
    commands
        .entity(activation.entity)
        .insert(ActivationAnimation::new(
            0.4, // Animation duration in seconds
            1.3, // Scale multiplier at peak
            original_scale,
        ));

    // Process propagation based on behavior
    if should_propagate {
        // Apply tile modifier to outgoing energy
        let mut modified_energy_packet = activation.energy_packet.clone();
        modified_energy_packet.energy *= tile_modifier;

        process_propagation(
            chain_manager,
            &definition.activation_behavior,
            definition,
            activation.entity,
            *position,
            direction.copied(),
            modified_energy_packet,
            activation.chain_id,
            game_state,
        );
    }
}

/// Process energy propagation based on mushroom behavior
fn process_propagation(
    chain_manager: &mut ChainManager,
    behavior: &ActivationBehavior,
    definition: &MushroomDefinition,
    source_entity: Entity,
    source_pos: GridPosition,
    direction: Option<MushroomDirection>,
    mut energy_packet: EnergyPacket,
    chain_id: u32,
    game_state: &GameState,
) {
    // Add this mushroom to the path
    energy_packet.path.push(source_entity);

    // Get connection points from definition
    let connection_points = &definition.connection_points;

    // Process based on behavior type (for special modifications)
    match behavior {
        ActivationBehavior::Basic => {
            // No behaviour modification, just propagate if there are connection points
        }

        ActivationBehavior::Amplifier { boost_factor } => {
            // Boost energy before propagating
            energy_packet.energy *= boost_factor;
        }
        _ => {}
    }

    // Perform propagation to connection points
    propagate_to_connection_points(
        chain_manager,
        source_pos,
        connection_points,
        direction,
        energy_packet,
        chain_id,
        game_state,
    );
}

/// Propagate energy to connection points
fn propagate_to_connection_points(
    chain_manager: &mut ChainManager,
    source_pos: GridPosition,
    connection_points: &[GridOffset],
    direction: Option<MushroomDirection>,
    energy_packet: EnergyPacket,
    chain_id: u32,
    game_state: &GameState,
) {
    if connection_points.is_empty() {
        return;
    }

    // Calculate actual target positions
    let mut targets = Vec::new();

    for point in connection_points {
        let target_pos = if let Some(dir) = direction {
            // Rotate connection point based on facing
            let rotated = rotate_connection_point(point, &dir);
            GridPosition::new(source_pos.x + rotated.x, source_pos.y + rotated.y)
        } else {
            // No direction, use as-is
            GridPosition::new(source_pos.x + point.x, source_pos.y + point.y)
        };

        if let Some(entity) = game_state.play_field.get(target_pos) {
            targets.push((entity, target_pos));
        }
    }

    if targets.is_empty() {
        return;
    }

    // Split energy among targets if there are multiple
    let split_energy = if targets.len() > 1 {
        energy_packet.energy / targets.len() as f32
    } else {
        energy_packet.energy
    };

    for (i, (target_entity, _)) in targets.into_iter().enumerate() {
        let mut new_packet = energy_packet.clone();
        new_packet.energy = split_energy;
        new_packet.source_entity = target_entity;

        let delay = 0.2 + (i as f32 * 0.05);
        chain_manager.queue_activation(target_entity, new_packet, delay, chain_id);
    }
}

/// Rotate a connection point based on mushroom direction
fn rotate_connection_point(point: &GridOffset, direction: &MushroomDirection) -> GridOffset {
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

/// Update mushroom cooldowns
fn update_mushroom_cooldowns(
    time: Res<GameTime>,
    mut mushrooms: Query<&mut MushroomActivationState>,
) {
    for mut state in mushrooms.iter_mut() {
        if let Some(ref mut timer) = state.cooldown_timer {
            timer.tick(Duration::from_secs_f32(time.delta_seconds));
            if timer.finished() {
                state.cooldown_timer = None;
            }
        }
    }
}

/// Reset mushroom states (at the start of a new turn)
pub fn reset_mushroom_states(
    mut mushrooms: Query<&mut MushroomActivationState>,
    mut chain_manager: ResMut<ChainManager>,
) {
    for mut state in mushrooms.iter_mut() {
        state.activations_this_turn = 0;
        state.last_activation_energy = 0.0;
        state.cooldown_timer = None; // Reset cooldowns
    }

    chain_manager.reset_turn();
}
