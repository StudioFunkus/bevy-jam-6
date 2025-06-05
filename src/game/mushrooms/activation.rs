use bevy::prelude::*;

use crate::{
    PausableSystems,
    game::{
        event_queue::{EventQueue, ScheduledEvent, process_scheduled_events},
        game_flow::{CurrentLevel, TurnData},
        play_field::GridPosition,
        play_field::observers::find_entity_at,
        mushrooms::{
            ActivationSource, Mushroom, MushroomCooldown, MushroomDirection, MushroomType,
            events::ActivateMushroomEvent,
        },
        resources::GameState,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            process_mushroom_activations,
            process_scheduled_events::<ActivateMushroomEvent>,
        )
            .chain()
            .in_set(PausableSystems),
    );

    app.add_observer(handle_new_immediate_event);

    app.init_resource::<EventQueue<ActivateMushroomEvent>>();
}

/// Push new immediate events to the queue
#[tracing::instrument(name = "Handle new immediate events", skip_all)]
fn handle_new_immediate_event(
    trigger: Trigger<ActivateMushroomEvent>,
    mut action_queue: ResMut<EventQueue<ActivateMushroomEvent>>,
) -> Result {
    info!("System trigger: handle_new_immediate_event");

    action_queue.immediate.push_back(trigger.clone());

    Ok(())
}

/// Process ActivateMushroomEvents from the action queue
#[tracing::instrument(name = "Process all mushroom activations", skip_all)]
fn process_mushroom_activations(
    mut commands: Commands,
    mut action_queue: ResMut<EventQueue<ActivateMushroomEvent>>,
    mut game_state: ResMut<GameState>,
    mushrooms: Query<&Mushroom>,
    cooldowns: Query<&MushroomCooldown>,
    directions: Query<&MushroomDirection>,
    mut turn_data: ResMut<TurnData>,
    mut current_level: ResMut<CurrentLevel>,
) -> Result {
    // Process immediate actions
    while let Some(event) = action_queue.immediate.pop_front() {
        let Some(entity) = find_entity_at(event.position, &game_state) else {
            continue;
        };
        let mushroom = mushrooms.get(entity)?;

        // Check if mushroom is on cooldown
        if cooldowns.get(entity).is_ok() {
            continue;
        }

        // Set cooldown
        commands.entity(entity).insert(MushroomCooldown {
            timer: Timer::from_seconds(mushroom.0.cooldown_time(), TimerMode::Once),
        });

        // Calculate production with multiplier
        let production = mushroom.0.base_production() * event.energy;

        game_state.add_spores(production);

        // Update turn tracking
        turn_data.spores_this_chain += production;
        current_level.total_spores_earned += production;

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
            &game_state,
            &directions,
        );

        // Add delayed actions
        for (i, (pos, energy)) in triggers.into_iter().enumerate() {
            let delay = 0.1 + (i as f32 * 0.05); // Stagger actions

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

#[tracing::instrument(name = "Process single mushroom activation", skip_all)]
fn process_mushroom_activation(
    mushroom_type: MushroomType,
    pos: GridPosition,
    event: &ActivateMushroomEvent,
    entity: Entity,
    game_state: &GameState,
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
                .copied()
                .unwrap_or(MushroomDirection::Up);
            let (dx, dy) = direction.to_offset();
            let target = GridPosition::new(pos.x + dx, pos.y + dy);

            if game_state.play_field.contains(target) {
                vec![(target, event.energy * 0.9)]
            } else {
                vec![]
            }
        }
    }
}
