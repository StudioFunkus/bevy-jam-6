use bevy::{platform::collections::HashMap, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GridConfig>();
    app.init_resource::<Grid>();
    app.add_event::<GridClickEvent>();

    app.add_systems(Update, handle_grid_clicks);
}

// Type alias for the spatial data structure
#[derive(Resource, Default)]
pub struct Grid(pub HashMap<GridPosition, Entity>); // Should this be &Entity?

/// Grid configuration
#[derive(Resource, Debug)]
pub struct GridConfig {
    pub width: i32,
    pub height: i32,
    pub cell_size: f32,
    pub cell_spacing: f32,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            width: 128,
            height: 128,
            cell_size: 64.0,
            cell_spacing: 4.0,
        }
    }
}

/// Position on the grid
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash, Reflect, Default)]
#[reflect(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Get all adjacent positions
    #[allow(dead_code)]
    pub fn adjacent(&self) -> [GridPosition; 8] {
        [
            GridPosition::new(self.x - 1, self.y),
            GridPosition::new(self.x + 1, self.y),
            GridPosition::new(self.x, self.y - 1),
            GridPosition::new(self.x, self.y + 1),
            GridPosition::new(self.x - 1, self.y - 1),
            GridPosition::new(self.x + 1, self.y - 1),
            GridPosition::new(self.x + 1, self.y + 1),
            GridPosition::new(self.x - 1, self.y + 1),
        ]
    }

    /// Get all adjacent cardinal positions
    #[allow(dead_code)]
    pub fn adjacent_cardinal(&self) -> [GridPosition; 4] {
        [
            GridPosition::new(self.x - 1, self.y),
            GridPosition::new(self.x + 1, self.y),
            GridPosition::new(self.x, self.y - 1),
            GridPosition::new(self.x, self.y + 1),
        ]
    }

    /// Get all adjacent diagonal positions
    #[allow(dead_code)]
    pub fn adjacent_diagonal(&self) -> [GridPosition; 4] {
        [
            GridPosition::new(self.x - 1, self.y - 1),
            GridPosition::new(self.x + 1, self.y - 1),
            GridPosition::new(self.x + 1, self.y + 1),
            GridPosition::new(self.x - 1, self.y + 1),
        ]
    }

    /// Convert grid position to world coordinates
    pub fn to_world(&self, config: &GridConfig) -> Vec3 {
        let total_cell_size = config.cell_size + config.cell_spacing;
        let grid_width = config.width as f32 * total_cell_size - config.cell_spacing;
        let grid_height = config.height as f32 * total_cell_size - config.cell_spacing;
        let offset_x = -grid_width / 2.0 + config.cell_size / 2.0;
        let offset_y = -grid_height / 2.0 + config.cell_size / 2.0;

        Vec3::new(
            offset_x + self.x as f32 * total_cell_size,
            offset_y + self.y as f32 * total_cell_size,
            0.0,
        )
    }

    /// Check if position is within grid bounds
    pub fn in_bounds(&self, config: &GridConfig) -> bool {
        self.x >= 0 && self.x < config.width && self.y >= 0 && self.y < config.height
    }
}

/// Component for grid cells that can be clicked
#[derive(Component)]
pub struct GridCell {
    pub position: GridPosition,
}

/// Event fired when a grid cell is clicked
#[derive(Event, Debug)]
pub struct GridClickEvent {
    pub position: GridPosition,
    pub button: bevy::picking::pointer::PointerButton,
}

#[tracing::instrument(name = "Handle grid clicks", skip_all)]
fn handle_grid_clicks(
    mut click_events: EventReader<Pointer<Click>>,
    mut commands: Commands,
    grid_cells: Query<&GridCell>,
) {
    for event in click_events.read() {
        if let Ok(cell) = grid_cells.get(event.target) {
            info!("Grid cell clicked at position: {:?}", cell.position);
            info!("Triggering event: GridClickEvent");
            commands.trigger(GridClickEvent {
                position: cell.position,
                button: event.button,
            });
        }
    }
}

/// Spatial mushroom lookup
#[tracing::instrument(name = "Find mushrooms at GridPosition", skip_all)]
pub fn find_mushroom_at(position: GridPosition, grid: &Grid) -> Option<Entity> {
    grid.0.get(&position).copied()
}
