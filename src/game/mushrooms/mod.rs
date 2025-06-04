use bevy::{pbr::NotShadowReceiver, prelude::*};
use bevy_sprite3d::{Sprite3d, Sprite3dBuilder, Sprite3dParams};
use events::SpawnMushroomEvent;
use resources::SelectedMushroomType;

use crate::{
    PausableSystems,
    game::{
        fixed_timestep::GameTime,
        game_flow::{LevelState, TurnData, TurnPhase},
        level::assets::LevelAssets,
        mushrooms::events::ActivateMushroomEvent,
        resources::{GameState, UnlockedMushrooms},
        visual_effects::FaceCamera,
    },
};

use super::grid::{Grid, GridClickEvent, GridConfig, GridPosition, find_mushroom_at};

mod activation;
pub(crate) mod events;
pub(crate) mod resources;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((events::plugin, activation::plugin));

    app.add_systems(
        FixedUpdate,
        update_mushroom_cooldowns.in_set(PausableSystems),
    );

    app.add_observer(handle_grid_clicks)
        .add_observer(spawn_mushroom);

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
    Mushroom,
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
pub struct Mushroom(pub MushroomType);

/// Facing direction for mushrooms
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MushroomDirection {
    Up,
    Right,
    Down,
    Left,
}

impl MushroomDirection {
    pub fn to_offset(self) -> (i32, i32) {
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

/// Routes grid click events to appropriate handlers based on game phase.
#[tracing::instrument(name = "Handle grid clicks", skip_all)]
fn handle_grid_clicks(
    trigger: Trigger<GridClickEvent>,
    commands: Commands,
    selected_type: Res<SelectedMushroomType>,
    grid: ResMut<Grid>,
    grid_config: Res<GridConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mushrooms: Query<&Mushroom>,
    cooldowns: Query<&MushroomCooldown>,
    directions: Query<&mut MushroomDirection>,
    game_state: ResMut<GameState>,
    unlocked: Res<UnlockedMushrooms>,
    current_phase: Option<Res<State<TurnPhase>>>,
    turn_data: ResMut<TurnData>,
) -> Result {
    info!("System triggered: handle_grid_clicks");

    if !trigger.position.in_bounds(&grid_config) {
        return Ok(());
    }

    // Check if we're in a phase that allows interaction
    let Some(phase_state) = current_phase else {
        info!("Not in a game turn phase");
        return Ok(());
    };

    // TODO: Add click effect for visual feedback

    // Route to appropriate handler based on phase
    match phase_state.get() {
        TurnPhase::Planting => handle_planting_phase_click(
            trigger.event(),
            commands,
            selected_type,
            grid,
            keyboard,
            mushrooms,
            directions,
            game_state,
            unlocked,
        ),
        TurnPhase::Chain => {
            handle_chain_phase_click(trigger.event(), commands, grid, cooldowns, turn_data)
        }
        _ => {
            info!(
                "Cannot interact with mushrooms during {:?} phase",
                phase_state.get()
            );
            Ok(())
        }
    }
}

/// Handles clicks during the planting phase (placing, rotating, deleting mushrooms).
#[tracing::instrument(name = "Handle planting phase click", skip_all)]
fn handle_planting_phase_click(
    event: &GridClickEvent,
    mut commands: Commands,
    selected_type: Res<SelectedMushroomType>,
    mut grid: ResMut<Grid>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mushrooms: Query<&Mushroom>,
    mut directions: Query<&mut MushroomDirection>,
    mut game_state: ResMut<GameState>,
    unlocked: Res<UnlockedMushrooms>,
) -> Result {
    // Handle right-click for deletion
    if event.button == PointerButton::Secondary {
        return delete_mushroom_at(
            event.position,
            &mut commands,
            &mut grid,
            &mushrooms,
            &mut game_state,
        );
    }

    // Only handle left-clicks from here
    if event.button != PointerButton::Primary {
        return Ok(());
    }

    // Check if there's already a mushroom at this position
    if let Some(entity) = find_mushroom_at(event.position, &grid) {
        // Handle rotation if shift is held
        if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            return rotate_mushroom(entity, &mushrooms, &mut directions);
        }
        // Don't place over existing mushroom
        return Ok(());
    }

    // Try to place a new mushroom
    place_mushroom(
        commands,
        &mut game_state,
        &selected_type,
        &unlocked,
        event.position,
    )
}

/// Handles clicks during the chain phase (activating mushrooms).
#[tracing::instrument(name = "Handle chain phase click", skip_all)]
fn handle_chain_phase_click(
    event: &GridClickEvent,
    mut commands: Commands,
    grid: ResMut<Grid>,
    cooldowns: Query<&MushroomCooldown>,
    mut turn_data: ResMut<TurnData>,
) -> Result {
    // Only handle left-clicks
    if event.button != PointerButton::Primary {
        return Ok(());
    }

    // Try to activate mushroom at clicked position
    if let Some(entity) = find_mushroom_at(event.position, &grid) {
        // Check cooldown
        if cooldowns.get(entity).is_ok() {
            info!("Mushroom on cooldown");
            return Ok(());
        }

        info!("Triggering event: ActivateMushroomEvent");
        commands.trigger(ActivateMushroomEvent {
            position: event.position,
            source: ActivationSource::PlayerClick,
            energy: 1.0,
        });

        // Track activation
        turn_data.activations_this_chain += 1;
    } else {
        info!("No mushroom at this position to activate");
    }

    Ok(())
}

/// Deletes a mushroom at the given position and refunds half its cost.
#[tracing::instrument(name = "Delete mushroom", skip_all)]
fn delete_mushroom_at(
    position: GridPosition,
    commands: &mut Commands,
    grid: &mut ResMut<Grid>,
    mushrooms: &Query<&Mushroom>,
    game_state: &mut ResMut<GameState>,
) -> Result {
    if let Some(entity) = find_mushroom_at(position, grid) {
        let mushroom = mushrooms.get(entity)?;

        // Refund half the cost
        let refund = mushroom.0.cost() * 0.5;
        game_state.add_spores(refund);

        info!("Deleted {} - refunded {} spores", mushroom.0.name(), refund);

        // Update grid
        grid.0.remove(&position);

        // Despawn the mushroom entity
        commands.entity(entity).despawn();
    }
    Ok(())
}

/// Rotates a directional mushroom clockwise.
#[tracing::instrument(name = "Rotate mushroom", skip_all)]
fn rotate_mushroom(
    entity: Entity,
    mushrooms: &Query<&Mushroom>,
    directions: &mut Query<&mut MushroomDirection>,
) -> Result {
    let mushroom = mushrooms.get(entity)?;

    // Only pulse mushrooms can be rotated
    if matches!(mushroom.0, MushroomType::Pulse) {
        if let Ok(mut direction) = directions.get_mut(entity) {
            *direction = direction.rotate_clockwise();
            info!("Rotated mushroom to {:?}", *direction);
        }
    }

    Ok(())
}

// If possible, write an event to spawn a mushroom
#[tracing::instrument(name = "Place mushroom", skip_all)]
fn place_mushroom(
    mut commands: Commands,
    game_state: &mut ResMut<GameState>,
    selected_type: &Res<SelectedMushroomType>,
    unlocked: &Res<UnlockedMushrooms>,
    position: GridPosition,
) -> Result {
    info!("System triggered: place_mushroom");

    // Check if unlocked
    if !selected_type.mushroom_type.is_unlocked(unlocked) {
        info!("Mushroom type not unlocked!");
        return Ok(());
    }

    // Check cost
    let cost = selected_type.mushroom_type.cost();
    if !game_state.spend_spores(cost) {
        info!("Not enough spores to place mushroom!");
        return Ok(());
    }

    info!("Triggering event: SpawnMushroomEvent");
    commands.trigger(SpawnMushroomEvent {
        position,
        mushroom_type: selected_type.mushroom_type,
    });

    Ok(())
}

/// Actually spawn mushroom entities
#[tracing::instrument(name = "Spawn placed mushroom", skip_all)]
fn spawn_mushroom(
    trigger: Trigger<SpawnMushroomEvent>,
    mut commands: Commands,
    grid_config: Res<GridConfig>,
    mut grid: ResMut<Grid>,
    mut sprite_params: Sprite3dParams,
    level_assets: Res<LevelAssets>,
) {
    info!("System triggered: spawn_mushrooms");

    let base_scale = 1.0;
    let world_pos = trigger.position.to_world(&grid_config);

    // Create texture atlas layout for mushroom sprites
    // Each sprite is 16x16 with 2px padding (18x18 total per cell)
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),     // sprite size
        2,                      // columns (2 directions per mushroom)
        2,                      // rows (we have 2 mushroom types)
        Some(UVec2::new(2, 2)), // padding
        None,                   // offset
    );
    let layout_handle = sprite_params.atlas_layouts.add(layout);

    // Determine which row to use based on mushroom type
    let row = match trigger.mushroom_type {
        MushroomType::Basic => 0,
        MushroomType::Pulse => 1,
    };

    // Start with index 0 (facing camera)
    let atlas_index = row * 2; // 2 columns per row

    let atlas = TextureAtlas {
        layout: layout_handle.clone(),
        index: atlas_index,
    };

    // Insert core components
    let mushroom = commands
        .spawn((
            Name::new(format!(
                "{} at ({}, {})",
                trigger.mushroom_type.name(),
                trigger.position.x,
                trigger.position.y
            )),
            Mushroom(trigger.mushroom_type),
            trigger.position,
            Sprite3dBuilder {
                image: level_assets.mushroom_texture.clone(),
                pixels_per_metre: 16.0,
                double_sided: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            }
            .bundle_with_atlas(&mut sprite_params, atlas),
            Transform::from_xyz(world_pos.x, 0.5, -world_pos.y).with_scale(Vec3::splat(base_scale)),
            FaceCamera,
            NotShadowReceiver,
            StateScoped(LevelState::Playing),
        ))
        .id();

    // Add the mushroom to the grid
    grid.0.insert(trigger.position, mushroom);

    // Add type-specific components
    #[allow(clippy::single_match)]
    match trigger.mushroom_type {
        MushroomType::Pulse => {
            commands.entity(mushroom).insert(MushroomDirection::Up);
        }
        _ => {}
    }
}

/// Update mushroom cooldowns
#[tracing::instrument(name = "Update mushroom cooldowns", skip_all)]
fn update_mushroom_cooldowns(
    game_time: Res<GameTime>,
    mut commands: Commands,
    mut cooldowns: Query<(Entity, &mut MushroomCooldown, &mut Sprite3d), With<Mushroom>>,
) {
    for (entity, mut cooldown, mut _sprite3d) in &mut cooldowns {
        game_time.tick_timer(&mut cooldown.timer);

        // Remove finished cooldowns
        if cooldown.timer.finished() {
            commands.entity(entity).remove::<MushroomCooldown>();
        }
    }
}
