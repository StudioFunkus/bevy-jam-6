//! Tile atlas configuration and mapping

/// Available tile sprites in the atlas
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum TileSprite {
    // Rocky terrain variants
    Rock1 = 0,
    Rock2 = 1,
    Rock3 = 2,

    // Standard soil variants
    Soil1 = 3,
    Soil2 = 4,
    Soil3 = 5,

    // Border tiles
    Border1 = 6,
    Border2 = 7,
    Border3 = 8,

    // Corner tiles
    OutsideCorner1 = 9,
    OutsideCorner2 = 10,
    InsideCorner1 = 11,
    InsideCorner2 = 12,

    // Rich/fertile soil variants
    RichSoil1 = 13,
    RichSoil2 = 14,

    // Blocker variants
    BlockerWater1 = 15,
    BlockerWater2 = 16,
    BlockerRock1 = 17,
    BlockerRock2 = 18,
    BlockerMoss1 = 19,
    BlockerMoss2 = 20,
}

impl TileSprite {
    /// Get the atlas index for this sprite
    pub fn index(&self) -> u32 {
        *self as u32
    }
}

/// Configuration for the tile atlas texture
#[allow(dead_code)]
pub struct TileAtlasConfig {
    pub tile_size: u32,
    pub tiles_per_row: u32,
    pub total_tiles: u32,
}

impl Default for TileAtlasConfig {
    fn default() -> Self {
        Self {
            tile_size: 16,
            tiles_per_row: 1, // Vertical strip
            total_tiles: 21,
        }
    }
}
