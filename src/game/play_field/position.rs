//! Grid position component and utilities

use super::{CELL_SIZE, CELL_SPACING, PlayField};
use bevy::prelude::*;

pub(super) fn plugin(_app: &mut App) {}

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
    pub fn to_world(self, field_width: i32, field_height: i32) -> Vec3 {
        let total_cell_size = CELL_SIZE + CELL_SPACING;
        let grid_width = field_width as f32 * total_cell_size;
        let grid_height = field_height as f32 * total_cell_size;
        let offset_x = -grid_width / 2.0 + CELL_SIZE / 2.0;
        let offset_y = -grid_height / 2.0 + CELL_SIZE / 2.0;

        Vec3::new(
            offset_x + self.x as f32 * total_cell_size,
            0.0,
            offset_y + self.y as f32 * total_cell_size,
        )
    }

    /// Convert grid position to world coordinates using PlayField reference
    pub fn to_world_in(&self, field: &PlayField) -> Vec3 {
        self.to_world(field.width, field.height)
    }
}
