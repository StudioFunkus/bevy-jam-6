use bevy::prelude::*;

pub(crate) mod card;
pub(crate) mod constants;
// mod deck;
pub(crate) mod events;
pub(crate) mod hand;
mod managers;
pub(crate) mod markers;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((hand::plugin, card::plugin, managers::plugin));
}
