// Debugging the turn manager state

use bevy::{dev_tools::states::log_transitions, prelude::*};
use bevy_inspector_egui::{
    bevy_egui::{EguiContextPass, EguiContexts},
    egui,
};

use crate::{
    game::turn_manager::{CurrentLevel, LevelState, TurnData, TurnPhase},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Log turn state transitions
    app.add_systems(Update, log_transitions::<LevelState>);
    app.add_systems(Update, log_transitions::<TurnPhase>);

    // Add turn state debug window
    app.add_systems(
        EguiContextPass,
        turn_state_debug_window.run_if(in_state(Screen::Gameplay)),
    );
}

/// Show turn state information in an egui window
pub(crate) fn turn_state_debug_window(
    mut contexts: EguiContexts,
    current_level_state: Res<State<LevelState>>,
    current_phase: Option<Res<State<TurnPhase>>>,
    turn_data: Res<TurnData>,
    current_level: Res<CurrentLevel>,
) {
    egui::Window::new("Turn State Debug")
        .default_pos([10.0, 200.0])
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Level State");
            ui.label(format!("State: {:?}", current_level_state.get()));

            if let Some(phase) = current_phase {
                ui.label(format!("Phase: {:?}", phase.get()));
            } else {
                ui.label("Phase: None (not in game)");
            }

            ui.separator();

            ui.heading("Level Info");
            ui.label(format!("Level: {}", current_level.level_index));
            ui.label(format!("Target Score: {:.0}", current_level.target_score));
            ui.label(format!("Max Turns: {}", current_level.max_turns));
            ui.label(format!(
                "Total Spores: {:.0}",
                current_level.total_spores_earned
            ));

            ui.separator();

            ui.heading("Turn Info");
            ui.label(format!("Current Turn: {}", turn_data.current_turn));
            ui.label(format!(
                "Mushrooms Drawn: {}",
                turn_data.mushrooms_drawn_this_turn
            ));
            ui.label(format!(
                "Chain Activations: {}",
                turn_data.activations_this_chain
            ));
            ui.label(format!(
                "Spores This Chain: {:.0}",
                turn_data.spores_this_chain
            ));
        });
}
