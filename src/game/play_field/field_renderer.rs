//! Field rendering with extended material for tiles and mycelium
//!
//! COORDINATE SYSTEM NOTES:
//! - Grid coordinates: Y=0 at TOP, Y increases DOWNWARD
//! - Texture coordinates: Y=0 at BOTTOM, Y increases UPWARD  

use super::tile_atlas::TileSprite;
use super::{GridPosition, PlayField, TileGrid, TileType};
use crate::game::game_flow::LevelState;
use crate::game::level::assets::LevelAssets;
use crate::game::resources::GameState;
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_asset::RenderAssetUsages,
    render::render_resource::{AsBindGroup, ShaderRef},
};

/// Convert grid Y coordinate to texture Y coordinate
/// Grid Y=0 is at TOP, texture Y=0 is at BOTTOM
fn grid_y_to_texture_y(grid_y: u32, texture_height: u32) -> u32 {
    texture_height - 1 - grid_y
}

/// Convert grid position to texture pixel index for RGBA textures
fn grid_pos_to_texture_index(
    grid_x: u32,
    grid_y: u32,
    texture_width: u32,
    texture_height: u32,
) -> usize {
    let texture_y = grid_y_to_texture_y(grid_y, texture_height);
    ((texture_y * texture_width + grid_x) * 4) as usize
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(MaterialPlugin::<
        ExtendedMaterial<StandardMaterial, FieldGroundExtension>,
    >::default())
        .add_systems(Update, update_connection_data)
        .add_systems(Update, update_material_time);
}

/// Extension data for field ground rendering
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct FieldGroundExtension {
    /// Tile texture atlas
    #[texture(100)]
    #[sampler(101)]
    pub tile_texture: Handle<Image>,

    /// Tile indices texture
    #[texture(102)]
    #[sampler(103)]
    pub tile_indices: Handle<Image>,

    /// Time for animations
    #[uniform(104)]
    pub time: f32,

    /// Grid dimensions
    #[uniform(105)]
    pub grid_size: Vec2,

    /// Number of active connections (max 64)
    #[uniform(106)]
    pub connection_count: u32,

    /// Connection start positions (64 max)
    #[uniform(107)]
    pub connection_starts: [Vec4; 64],

    /// Connection end positions (64 max)  
    #[uniform(108)]
    pub connection_ends: [Vec4; 64],

    /// Mycelium colors
    #[uniform(109)]
    pub mycelium_color_low: LinearRgba,

    #[uniform(110)]
    pub mycelium_color_high: LinearRgba,

    /// Animation parameters
    #[uniform(111)]
    pub pulse_speed: f32,

    #[uniform(112)]
    pub glow_intensity: f32,

    /// Line rendering parameters
    #[uniform(113)]
    pub line_width: f32,
}

impl MaterialExtension for FieldGroundExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/field_ground.wgsl".into()
    }
}

/// Component to track the field ground entity
#[derive(Component)]
pub struct FieldGround {
    pub material_handle: Handle<ExtendedMaterial<StandardMaterial, FieldGroundExtension>>,
}

/// Connection data for shader rendering
#[derive(Debug, Clone, Copy)]
pub struct ConnectionData {
    pub start_pos: Vec2,
    pub end_pos: Vec2,
    pub strength: f32,
    pub distance: f32,
}

impl Default for ConnectionData {
    fn default() -> Self {
        Self {
            start_pos: Vec2::ZERO,
            end_pos: Vec2::ZERO,
            strength: 0.0,
            distance: 0.0,
        }
    }
}

/// Spawn the field ground mesh for a specific level
pub fn spawn_field_ground(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial, FieldGroundExtension>>>,
    images: &mut ResMut<Assets<Image>>,
    level_assets: &Res<LevelAssets>,
    play_field: &PlayField,
    tile_grid: &TileGrid,
) -> Entity {
    let (world_width, world_height) = play_field.world_size();

    // Create ground plane mesh - exact size, no padding
    let mesh = meshes.add(Rectangle::new(world_width, world_height));

    // Create tile indices texture
    let tile_indices_texture = create_tile_indices_texture(tile_grid);
    let tile_indices_handle = images.add(tile_indices_texture);

    // Use tile texture from level assets
    let tile_texture_handle = level_assets.tile_texture.clone();

    // Create material with empty connection arrays
    let material_handle = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            alpha_mode: AlphaMode::Opaque,
            ..default()
        },
        extension: FieldGroundExtension {
            tile_texture: tile_texture_handle.clone(),
            tile_indices: tile_indices_handle,
            time: 0.0,
            grid_size: Vec2::new(play_field.width as f32, play_field.height as f32),
            connection_count: 0,
            connection_starts: [Vec4::ZERO; 64],
            connection_ends: [Vec4::ZERO; 64],
            mycelium_color_low: LinearRgba::new(0.2, 0.8, 0.4, 1.0),
            mycelium_color_high: LinearRgba::new(0.4, 1.0, 0.6, 1.0),
            pulse_speed: 2.0,
            glow_intensity: 0.8,
            line_width: 0.005,
        },
    });

    // Spawn ground entity
    commands
        .spawn((
            Name::new("Field Ground"),
            Mesh3d(mesh),
            MeshMaterial3d(material_handle.clone()),
            Transform::from_xyz(0.0, -0.05, 0.0)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            FieldGround {
                material_handle,
            },
            StateScoped(LevelState::Playing),
        ))
        .id()
}

/// Create texture containing tile type indices
fn create_tile_indices_texture(tile_grid: &TileGrid) -> Image {
    let width = tile_grid.width() as u32;
    let height = tile_grid.height() as u32;
    let mut data = vec![0u8; (width * height * 4) as usize];

    info!("Creating tile indices texture: {}x{}", width, height);
    let mut tile_counts = std::collections::HashMap::new();

    for y in 0..height {
        for x in 0..width {
            let pos = GridPosition::new(x as i32, y as i32);
            let tile_type = tile_grid.get(pos).unwrap_or_default();
            // We need to flip Y coordinate 
            let pixel_index = grid_pos_to_texture_index(x, y, width, height);

            // Count tile types
            *tile_counts.entry(tile_type).or_insert(0) += 1;

            // Debug: log special tiles
            if !matches!(tile_type, crate::game::play_field::TileType::Empty) {
                info!("Tile at ({}, {}): {:?}", x, y, tile_type);
            }

            // Select sprite based on tile type
            let sprite = select_tile_sprite(tile_type, pos);
            let sprite_index = sprite.index();

            // Store tile index in R channel
            // Our atlas has 21 sprites, so we'll normalize to 0-255 range
            let tile_value = ((sprite_index as f32 / 21.0) * 255.0) as u8;
            data[pixel_index] = tile_value;
            data[pixel_index + 1] = 0;
            data[pixel_index + 2] = 0;
            data[pixel_index + 3] = 255;
        }
    }

    let image = Image::new(
        bevy::render::render_resource::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );

    image
}

/// Update the material time uniform
fn update_material_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, FieldGroundExtension>>>,
    query: Query<&FieldGround>,
) {
    for field_ground in query.iter() {
        if let Some(material) = materials.get_mut(&field_ground.material_handle) {
            material.extension.time = time.elapsed_secs();
        }
    }
}

/// Update connection data in material uniforms based on active connections from PlayField
fn update_connection_data(
    field_grounds: Query<&FieldGround>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, FieldGroundExtension>>>,
    game_state: Res<GameState>,
) {
    // Only update when game state changes (connections are added/removed)
    if !game_state.is_changed() {
        return;
    }

    for field_ground in field_grounds.iter() {
        if let Some(material) = materials.get_mut(&field_ground.material_handle) {
            // Get all connections from PlayField
            let connections = game_state.play_field.get_all_connections();
            let connection_count = connections.len().min(64); // Cap at 64 connections for now

            // Clear arrays
            material.extension.connection_starts = [Vec4::ZERO; 64];
            material.extension.connection_ends = [Vec4::ZERO; 64];
            material.extension.connection_count = connection_count as u32;

            // Convert grid coordinates to normalized UV coordinates (0.0-1.0)
            let grid_size = material.extension.grid_size;

            for (i, connection) in connections.iter().take(64).enumerate() {
                // Convert grid positions to normalized coordinates with Y-flip
                // Grid positions are cell indices, but we want center of cells in UV space
                // Add 0.5 to get cell center, then normalize
                // Grid Y=0 is at TOP, UV Y=0 is at BOTTOM, so we need to flip Y
                let start_uv = Vec2::new(
                    (connection.from_pos.x as f32 + 0.5) / grid_size.x,
                    1.0 - ((connection.from_pos.y as f32 + 0.5) / grid_size.y), // Flip Y coordinate
                );
                let end_uv = Vec2::new(
                    (connection.to_pos.x as f32 + 0.5) / grid_size.x,
                    1.0 - ((connection.to_pos.y as f32 + 0.5) / grid_size.y), // Flip Y coordinate
                );

                // Calculate distance for energy flow timing
                let distance = start_uv.distance(end_uv);

                // Store in uniform arrays (using Vec4 for GPU alignment)
                material.extension.connection_starts[i] =
                    Vec4::new(start_uv.x, start_uv.y, connection.strength, distance);
                material.extension.connection_ends[i] = Vec4::new(end_uv.x, end_uv.y, 0.0, 0.0);
            }

            info!(
                "Updated {} mycelium connections in shader",
                connection_count
            );
        }
    }
}

/// Select tile sprite
fn select_tile_sprite(tile_type: TileType, position: GridPosition) -> TileSprite {
    match tile_type {
        TileType::Empty => {
            let variant = (position.x * 11 + position.y * 17) % 3;
            match variant {
                0 => TileSprite::Soil1,
                1 => TileSprite::Soil2,
                _ => TileSprite::Soil3,
            }
        }
        TileType::Fertile => {
            let variant = (position.x * 13 + position.y * 19) % 2;
            match variant {
                0 => TileSprite::RichSoil1,
                _ => TileSprite::RichSoil2,
            }
        }
        TileType::BlockedRock => {
            let variant = (position.x * 23 + position.y * 29) % 2;
            match variant {
                0 => TileSprite::BlockerRock1,
                _ => TileSprite::BlockerRock2,
            }
        }
        TileType::BlockedWater => {
            let variant = (position.x * 31 + position.y * 37) % 2;
            match variant {
                0 => TileSprite::BlockerWater1,
                _ => TileSprite::BlockerWater2,
            }
        }
        TileType::BlockedMoss => {
            let variant = (position.x * 17 + position.y * 41) % 2;
            match variant {
                0 => TileSprite::BlockerMoss1,
                _ => TileSprite::BlockerMoss2,
            }
        }
    }
}
