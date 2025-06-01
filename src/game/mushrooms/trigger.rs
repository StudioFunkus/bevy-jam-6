use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{
    PausableSystems,
    game::{
        grid::{GridConfig, GridPosition, MushroomSpatial, find_mushroom_at},
        mushrooms::{
            Mushroom, MushroomCooldown, MushroomDirection, MushroomType,
            resources::SelectedMushroomType,
        },
        resources::GameState,
        visual_effects::{SpawnTriggerEffect, SpawnDirectionalPulse},
    },
};

use super::events::TriggerMushroomEvent;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_trigger_queue, process_delayed_triggers)
            .chain()
            .in_set(PausableSystems),
    );

    app.init_resource::<SelectedMushroomType>();
    app.init_resource::<TriggerQueue>();
}

/// Trigger source for mushroom effects
#[derive(Clone, Copy, Debug)]
pub enum TriggerSource {
    PlayerClick,
    MushroomTrigger(GridPosition),
}

/// Queue for processing mushroom triggers
#[derive(Resource, Default)]
pub struct TriggerQueue {
    /// Immediate triggers to process
    immediate: VecDeque<TriggerMushroomEvent>,
    /// Delayed triggers waiting to fire
    delayed: Vec<DelayedTrigger>,
    /// Current chain multiplier
    current_multiplier: f64,
    /// Current chain length
    current_chain_length: u32,
}

#[derive(Clone)]
struct DelayedTrigger {
    event: TriggerMushroomEvent,
    delay: Timer,
}

/// Process trigger events
fn update_trigger_queue(
    mut commands: Commands,
    mut trigger_queue: ResMut<TriggerQueue>,
    mut trigger_events: EventReader<TriggerMushroomEvent>,
    mut game_state: ResMut<GameState>,
    mut visual_effects: EventWriter<SpawnTriggerEffect>,
    mut directional_effects: EventWriter<SpawnDirectionalPulse>,
    game_grid_config: Res<GridConfig>,
    mushrooms: Query<(Entity, &GridPosition, &MushroomType), With<Mushroom>>,
    cooldowns: Query<&MushroomCooldown>,
    directions: Query<&MushroomDirection>,
) {
    // Add new triggers to immediate queue
    for event in trigger_events.read() {
        trigger_queue.immediate.push_back(event.clone());

        // Reset multiplier on player click
        if matches!(event.source, TriggerSource::PlayerClick) {
            trigger_queue.current_multiplier = 1.0;
            trigger_queue.current_chain_length = 0;
        }
    }

    // Process immediate triggers
    while let Some(event) = trigger_queue.immediate.pop_front() {
        let Some((entity, mushroom_type)) = find_mushroom_at(event.position, &mushrooms) else {
            continue;
        };

        // Check if mushroom is on cooldown
        if cooldowns.get(entity).is_ok() {
            continue;
        }

        // Spawn visual effect for trigger
        visual_effects.send(SpawnTriggerEffect {
            position: event.position,
            color: mushroom_type.color(),
        });

        // Set cooldown
        commands.entity(entity).insert(MushroomCooldown {
            timer: Timer::from_seconds(mushroom_type.cooldown_time(), TimerMode::Once),
        });

        // Calculate production with multiplier
        let production =
            mushroom_type.base_production() * event.energy * trigger_queue.current_multiplier;

        game_state.add_spores(production);

        // Update chain stats
        if matches!(event.source, TriggerSource::MushroomTrigger(_)) {
            trigger_queue.current_chain_length += 1;
            trigger_queue.current_multiplier *= 1.1; // 10% multiplier increase per chain
            game_state.chain_triggers += 1;
        }

        game_state.total_triggers += 1;

        // Process trigger pattern
        let triggers = process_mushroom_trigger(
            mushroom_type,
            event.position,
            &event,
            entity,
            &game_grid_config,
            &directions,
        );

        // Add delayed triggers and spawn directional pulses
        for (i, (pos, energy, direction)) in triggers.into_iter().enumerate() {
            let delay = 0.1 + (i as f32 * 0.05); // Stagger triggers

            // Spawn directional pulse effect
            directional_effects.send(SpawnDirectionalPulse {
                from_position: event.position,
                to_position: pos,
                color: mushroom_type.color(),
            });

            trigger_queue.delayed.push(DelayedTrigger {
                event: TriggerMushroomEvent {
                    position: pos,
                    source: TriggerSource::MushroomTrigger(event.position),
                    energy,
                    direction,
                },
                delay: Timer::from_seconds(delay, TimerMode::Once),
            });
        }
    }
}

fn process_mushroom_trigger(
    mushroom_type: MushroomType,
    pos: GridPosition,
    event: &TriggerMushroomEvent,
    entity: Entity,
    config: &GridConfig,
    directions: &Query<&MushroomDirection>,
) -> Vec<(GridPosition, f64, Option<MushroomDirection>)> {
    match mushroom_type {
        MushroomType::Basic => {
            // Basic mushrooms don't trigger others
            vec![]
        }
        MushroomType::Pulse => {
            // Pulse triggers in the direction it's facing
            let direction = directions
                .get(entity)
                .map(|d| *d)
                .unwrap_or(MushroomDirection::Up);
            let (dx, dy) = direction.to_offset();
            let target = GridPosition::new(pos.x + dx, pos.y + dy);

            if target.in_bounds(config) {
                vec![(target, event.energy * 0.9, Some(direction))]
            } else {
                vec![]
            }
        }
    }
}

/// Process delayed triggers
fn process_delayed_triggers(
    mut commands: Commands,
    time: Res<Time>,
    mut trigger_queue: ResMut<TriggerQueue>,
    grid_config: Res<GridConfig>,
) {
    // Update timers and collect ready triggers
    let mut ready_triggers = vec![];

    trigger_queue.delayed.retain_mut(|delayed| {
        delayed.delay.tick(time.delta());

        if delayed.delay.just_finished() {
            ready_triggers.push(delayed.event.clone());

            false
        } else {
            true
        }
    });

    // Add ready triggers to immediate queue
    for trigger in ready_triggers {
        trigger_queue.immediate.push_back(trigger);
    }
}