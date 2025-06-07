use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;

mod dragging_manager;
mod hover_manager;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        TweeningPlugin,
        dragging_manager::plugin,
        hover_manager::plugin,
    ));
}
