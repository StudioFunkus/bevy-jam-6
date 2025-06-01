use std::collections::VecDeque;

use bevy::prelude::*;

use crate::game::grid::GridPosition;

use super::events::TriggerMushroomEvent;

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
