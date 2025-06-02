use bevy::prelude::*;
use events::{SpawnMushroomEvent};
use resources::SelectedMushroomType;

use crate::{
    game::{
        mushrooms::events::ActivateMushroomEvent, resources::{GameState, UnlockedMushrooms}, visual_effects::SpawnClickEffect
    }, PausableSystems
};

use super::grid::{Grid, GridClickEvent, GridConfig, GridPosition, find_mushroom_at};

mod events;
pub(crate) mod resources;
mod activation;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((events::plugin, activation::plugin));

    app.add_systems(
        Update,
        (
            handle_grid_clicks,
            spawn_mushrooms,
            update_mushroom_cooldowns,
        )
            .chain()
            .in_set(PausableSystems),
    );

    app.init_resource::<SelectedMushroomType>();
}

/// Different types of mushrooms
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
#[reflect(Component)]
pub enum MushroomType {
    #[default]
    Basic,
    Pulse,
}

/// Activation source for mushroom effects
#[derive(Clone, Copy, Debug, Default)]
pub enum ActivationSource {
    #[default]
    PlayerClick,
    Mushroom
}

impl MushroomType {
    pub fn cost(&self) -> f64 {
        match self {
            MushroomType::Basic => 10.0,
            MushroomType::Pulse => 5.0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            MushroomType::Basic => "Button Mushroom",
            MushroomType::Pulse => "Pulse Mushroom",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            MushroomType::Basic => Color::srgb(0.5, 0.3, 0.1),
            MushroomType::Pulse => Color::srgb(0.2, 0.8, 0.2),
        }
    }

    pub fn cooldown_time(&self) -> f32 {
        match self {
            MushroomType::Basic => 0.5,
            MushroomType::Pulse => 2.0,
        }
    }

    pub fn base_production(&self) -> f64 {
        match self {
            MushroomType::Basic => 10.0,
            MushroomType::Pulse => 2.0,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            MushroomType::Basic => "Produces spores when clicked.",
            MushroomType::Pulse => "Triggers an adjacent mushroom.",
        }
    }

    pub fn is_unlocked(&self, unlocked: &UnlockedMushrooms) -> bool {
        match self {
            MushroomType::Basic => unlocked.button,
            MushroomType::Pulse => unlocked.pulse,
        }
    }
}

/// Marker component for mushrooms
#[derive(Component)]
pub struct Mushroom(MushroomType);

/// Facing direction for mushrooms
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MushroomDirection {
    Up,
    Right,
    Down,
    Left,
}

impl MushroomDirection {
    pub fn to_offset(&self) -> (i32, i32) {
        match self {
            MushroomDirection::Up => (0, 1),
            MushroomDirection::Right => (1, 0),
            MushroomDirection::Down => (0, -1),
            MushroomDirection::Left => (-1, 0),
        }
    }

    pub fn rotate_clockwise(&self) -> MushroomDirection {
        match self {
            MushroomDirection::Up => MushroomDirection::Right,
            MushroomDirection::Right => MushroomDirection::Down,
            MushroomDirection::Down => MushroomDirection::Left,
            MushroomDirection::Left => MushroomDirection::Up,
        }
    }
}

/// Cooldown component for mushrooms
#[derive(Component)]
pub struct MushroomCooldown {
    pub timer: Timer,
}

/// Handle grid clicks (respond to GridClickEvent)
fn handle_grid_clicks(
    mut commands: Commands,
    mut grid_events: EventReader<GridClickEvent>,
    mut spawn_events: EventWriter<SpawnMushroomEvent>,
    mut activation_events: EventWriter<ActivateMushroomEvent>,
    mut click_effects: EventWriter<SpawnClickEffect>,
    selected_type: Res<SelectedMushroomType>,
    mut grid: ResMut<Grid>,
    grid_config: Res<GridConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mushrooms: Query<&Mushroom>,
    cooldowns: Query<&MushroomCooldown>,
    mut directions: Query<&mut MushroomDirection>,
    mut game_state: ResMut<GameState>,
    unlocked: Res<UnlockedMushrooms>,
) -> Result {
    for event in grid_events.read() {
        if !event.position.in_bounds(&grid_config) {
            continue;
        }

        // Spawn click effect for visual feedback
        click_effects.write(SpawnClickEffect {
            position: event.position,
        });

        // Handle right-click for deletion
        if event.button == PointerButton::Secondary {
            if let Some(entity) = find_mushroom_at(event.position, &grid) {
                let mushroom = mushrooms.get(entity)?;

                // Refund half the cost
                let refund = mushroom.0.cost() * 0.5;
                game_state.add_spores(refund);

                info!("Deleted {} - refunded {} spores", mushroom.0.name(), refund);

                // Update grid
                grid.0.remove(&event.position);

                // Despawn the mushroom entity
                commands.entity(entity).despawn();
            }
            continue;
        }

        // Handle left-click
        if event.button != PointerButton::Primary {
            continue;
        }

        // If cell has a mushroom, try to trigger it
        if let Some(entity) = find_mushroom_at(event.position, &grid) {
            let mushroom = mushrooms.get(entity)?;

            // Check if shift is held for rotation
            if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
                // Rotate directional mushroom
                if matches!(mushroom.0, MushroomType::Pulse) {
                    if let Ok(mut direction) = directions.get_mut(entity) {
                        *direction = direction.rotate_clockwise();
                        info!("Rotated mushroom to {:?}", *direction);
                    }
                }
                continue; // Don't trigger when rotating
            }

            // Check cooldown for triggering
            if cooldowns.get(entity).is_ok() {
                info!("Mushroom on cooldown");
                continue;
            }
            
            activation_events.write(ActivateMushroomEvent {
                position: event.position,
                source: ActivationSource::PlayerClick,
                energy: 1.0,
            });
            continue;
        }

        // Try to place a new mushroom
        place_mushroom(
            &mut spawn_events,
            &mut game_state,
            &selected_type,
            &unlocked,
            event.position,
        );
    }

    Ok(())
}

// If possible, write an event to spawn a mushroom
fn place_mushroom(
    spawn_events: &mut EventWriter<SpawnMushroomEvent>,
    game_state: &mut ResMut<GameState>,
    selected_type: &Res<SelectedMushroomType>,
    unlocked: &Res<UnlockedMushrooms>,
    position: GridPosition,
) {
    // Check if unlocked
    if !selected_type.mushroom_type.is_unlocked(unlocked) {
        return;
    }

    // Check cost
    let cost = selected_type.mushroom_type.cost();
    if !game_state.spend_spores(cost) {
        info!("Not enough spores to place mushroom");
        return;
    }

    spawn_events.write(SpawnMushroomEvent {
        position,
        mushroom_type: selected_type.mushroom_type,
    });
}

/// Actually spawn mushroom entities
fn spawn_mushrooms(
    mut commands: Commands,
    mut spawn_events: EventReader<SpawnMushroomEvent>,
    grid_config: Res<GridConfig>,
    mut grid: ResMut<Grid>,
) {
    for event in spawn_events.read() {
        let base_scale = 1.0;

        // Insert core components
        let mushroom = commands
            .spawn((
                Name::new(format!(
                    "{} at ({}, {})",
                    event.mushroom_type.name(),
                    event.position.x,
                    event.position.y
                )),
                Mushroom(event.mushroom_type),
                event.position,
                Sprite {
                    color: event.mushroom_type.color(),
                    custom_size: Some(Vec2::splat(60.0)),
                    ..default()
                },
                Transform::from_translation(event.position.to_world(&grid_config))
                    .with_scale(Vec3::splat(base_scale)),
            ))
            .id();

        // Add the mushroom to the grid
        grid.0.insert(event.position, mushroom);

        // Add type-specific components
        match event.mushroom_type {
            MushroomType::Pulse => {
                commands.entity(mushroom).insert(MushroomDirection::Up);
            }
            _ => {}
        }
    }
}

/// Update mushroom cooldowns
fn update_mushroom_cooldowns(
    time: Res<Time>,
    mut commands: Commands,
    mut cooldowns: Query<(Entity, &mut MushroomCooldown, &mut Sprite), With<Mushroom>>,
) {
    for (entity, mut cooldown, mut sprite) in &mut cooldowns {
        cooldown.timer.tick(time.delta());

        // Visual feedback for cooldown
        let cooldown_progress = cooldown.timer.fraction_remaining();
        let base_color = sprite.color;
        sprite.color = base_color.with_alpha(0.5 + 0.5 * (1.0 - cooldown_progress));

        // Remove finished cooldowns
        if cooldown.timer.finished() {
            sprite.color = base_color.with_alpha(1.0);
            commands.entity(entity).remove::<MushroomCooldown>();
        }
    }
}
