use bevy::prelude::*;

use crate::{
    game::{
        event_queue::{process_scheduled_events, EventQueue, ScheduledEvent},
        grid::{find_mushroom_at, Grid, GridConfig, GridPosition},
        mushrooms::{
            events::ActivateMushroomEvent, ActivationSource, Mushroom, MushroomCooldown, MushroomDirection, MushroomType
        },
        resources::GameState,
        visual_effects::{SpawnActionEffect, SpawnDirectionalPulse},
    }, PausableSystems
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            process_mushroom_activations ,
            process_scheduled_events::<ActivateMushroomEvent>,
        )
            .chain()
            .in_set(PausableSystems),
    );

    app.init_resource::<EventQueue<ActivateMushroomEvent>>();
}

/// Process ActivateMushroomEvents from the action queue
fn process_mushroom_activations (
    mut commands: Commands,
    mut action_queue: ResMut<EventQueue<ActivateMushroomEvent>>,
    mut activate_mushroom_events: EventReader<ActivateMushroomEvent>,
    mut game_state: ResMut<GameState>,
    mut visual_effects: EventWriter<SpawnActionEffect>,
    mut directional_effects: EventWriter<SpawnDirectionalPulse>,
    grid: Res<Grid>,
    grid_config: Res<GridConfig>,
    mushrooms: Query<&Mushroom>,
    cooldowns: Query<&MushroomCooldown>,
    directions: Query<&MushroomDirection>,
) -> Result {
    // Add new actions to immediate queue
    for event in activate_mushroom_events.read() {
        action_queue.immediate.push_back(event.clone());
    }

    // Process immediate actions
    while let Some(event) = action_queue.immediate.pop_front() {
        let Some(entity) = find_mushroom_at(event.position, &grid) else {
            continue;
        };
        let mushroom = mushrooms.get(entity)?;

        // Check if mushroom is on cooldown
        if cooldowns.get(entity).is_ok() {
            continue;
        }

        // Spawn visual effect for action
        visual_effects.write(SpawnActionEffect {
            position: event.position,
            color: mushroom.0.color(),
        });

        // Set cooldown
        commands.entity(entity).insert(MushroomCooldown {
            timer: Timer::from_seconds(mushroom.0.cooldown_time(), TimerMode::Once),
        });

        // Calculate production with multiplier
        let production = mushroom.0.base_production() * event.energy;

        game_state.add_spores(production);

        // Update chain stats
        if matches!(event.source, ActivationSource::Mushroom) {
            game_state.chain_activations += 1;
        }

        game_state.total_activations += 1;

        // Process action pattern
        let triggers = process_mushroom_activation(
            mushroom.0,
            event.position,
            &event,
            entity,
            &grid_config,
            &directions,
        );

        // Add delayed actions and spawn directional pulses
        for (i, (pos, energy)) in triggers.into_iter().enumerate() {
            let delay = 0.1 + (i as f32 * 0.05); // Stagger actions

            // Spawn directional pulse effect
            directional_effects.write(SpawnDirectionalPulse {
                from_position: event.position,
                to_position: pos,
                color: mushroom.0.color(),
            });

            action_queue.scheduled.push(ScheduledEvent {
                event: ActivateMushroomEvent {
                    position: pos,
                    source: ActivationSource::Mushroom,
                    energy,
                },
                delay: Timer::from_seconds(delay, TimerMode::Once),
            });
        }
    }

    Ok(())
}

fn process_mushroom_activation(
    mushroom_type: MushroomType,
    pos: GridPosition,
    event: &ActivateMushroomEvent,
    entity: Entity,
    config: &GridConfig,
    directions: &Query<&MushroomDirection>,
) -> Vec<(GridPosition, f64)> {
    match mushroom_type {
        MushroomType::Basic => {
            // Basic mushrooms don't activate others
            vec![]
        }
        MushroomType::Pulse => {
            // Pulse activates other Mushrooms in the direction it's facing
            let direction = directions
                .get(entity)
                .map(|d| *d)
                .unwrap_or(MushroomDirection::Up);
            let (dx, dy) = direction.to_offset();
            let target = GridPosition::new(pos.x + dx, pos.y + dy);

            if target.in_bounds(config) {
                vec![(target, event.energy * 0.9)]
            } else {
                vec![]
            }
        }
    }
}
