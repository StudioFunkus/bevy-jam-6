use bevy::{prelude::*};
use bevy_spatial::{AutomaticUpdate, SpatialStructure};

use super::mushrooms::Mushroom;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GridConfig>();
    app.add_event::<GridClickEvent>();
    
    // Add spatial acceleration structure for mushrooms
    app.add_plugins(
        AutomaticUpdate::<Mushroom>::new()
            .with_spatial_ds(SpatialStructure::KDTree2)
    );
    
    app.add_systems(Update, handle_grid_clicks);
}

/// Grid configuration
#[derive(Resource)]
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
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Get all adjacent positions (orthogonal only)
    pub fn adjacent(&self) -> [GridPosition; 4] {
        [
            GridPosition::new(self.x - 1, self.y),
            GridPosition::new(self.x + 1, self.y),
            GridPosition::new(self.x, self.y - 1),
            GridPosition::new(self.x, self.y + 1),
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
#[derive(Event)]
pub struct GridClickEvent {
    pub position: GridPosition,
    pub button: bevy::picking::pointer::PointerButton,
}

fn handle_grid_clicks(
    mut click_events: EventReader<Pointer<Click>>,
    mut grid_click_events: EventWriter<GridClickEvent>,
    grid_cells: Query<&GridCell>,
) {
    for event in click_events.read() {
        if let Ok(cell) = grid_cells.get(event.target) {
            grid_click_events.write(GridClickEvent {
                position: cell.position,
                button: event.button,
            });
        }
    }
}

