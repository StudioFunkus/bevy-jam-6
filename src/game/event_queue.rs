use std::collections::VecDeque;

use bevy::prelude::*;

/// Queue for processing events
#[derive(Resource, Default)]
pub struct EventQueue<T: Event> {
    /// Immediate events to process
    pub immediate: VecDeque<T>,
    /// Scheduled events waiting to fire
    pub scheduled: Vec<ScheduledEvent<T>>,
}

#[derive(Clone)]
pub struct ScheduledEvent<T: Event> {
    pub event: T,
    pub delay: Timer,
}

/// Process scheduled events
pub fn process_scheduled_events<T: Event + Clone>(
    time: Res<Time>,
    mut event_queue: ResMut<EventQueue<T>>,
) {
    // Update timers and collect ready events
    let mut ready_events = vec![];

    event_queue.scheduled.retain_mut(|scheduled| {
        scheduled.delay.tick(time.delta());

        if scheduled.delay.just_finished() {
            ready_events.push(scheduled.event.clone());
            false
        } else {
            true
        }
    });

    // Add ready events to immediate queue
    for event in ready_events {
        event_queue.immediate.push_back(event);
    }
}

