//! Field rendering with extended material for tiles and mycelium
//!
//! COORDINATE SYSTEM NOTES:
//! - Grid coordinates: Y=0 at TOP, Y increases DOWNWARD
//! - Texture coordinates: Y=0 at BOTTOM, Y increases UPWARD  

use super::tile_atlas::TileSprite;
use super::{GridPosition, PlayField, TileType};
use crate::game::game_flow::LevelState;
use crate::game::level::assets::LevelAssets;
use crate::game::play_field::placement_preview::PreviewConnections;
use crate::game::resources::GameState;
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_asset::RenderAssetUsages,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    render::storage::ShaderStorageBuffer,
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
        .add_systems(Update, update_shader_highlights)
        .add_systems(Update, update_material_time);
}

/// Connection data for storage buffer
#[derive(Debug, Clone, Copy, ShaderType)]
pub struct ConnectionBufferData {
    pub start_pos: Vec2,
    pub end_pos: Vec2,
    pub strength: f32,
    pub distance: f32,
    pub _padding: Vec2,
}

/// Preview highlight data for storage buffer
#[derive(Debug, Clone, Copy, ShaderType)]
pub struct PreviewBufferData {
    pub position: Vec2,
    pub highlight_type: f32,
    pub _padding: f32,
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

    /// Number of active connections
    #[uniform(106)]
    pub connection_count: u32,

    /// Connection data storage buffer
    #[storage(107, read_only)]
    pub connections: Handle<ShaderStorageBuffer>,

    /// Number of preview highlights
    #[uniform(108)]
    pub preview_count: u32,

    /// Preview highlights storage buffer
    #[storage(109, read_only)]
    pub preview_highlights: Handle<ShaderStorageBuffer>,

    /// Mycelium colors
    #[uniform(110)]
    pub mycelium_color_low: LinearRgba,

    #[uniform(111)]
    pub mycelium_color_high: LinearRgba,

    /// Animation parameters
    #[uniform(112)]
    pub pulse_speed: f32,

    #[uniform(113)]
    pub glow_intensity: f32,

    /// Line rendering parameters
    #[uniform(114)]
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

/// Spawn the field ground mesh for a specific level
pub fn spawn_field_ground(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial, FieldGroundExtension>>>,
    images: &mut ResMut<Assets<Image>>,
    buffers: &mut ResMut<Assets<ShaderStorageBuffer>>,
    level_assets: &Res<LevelAssets>,
    play_field: &PlayField,
) -> Entity {
    let (world_width, world_height) = play_field.world_size();

    // Create ground plane mesh - exact size, no padding
    let mesh = meshes.add(Rectangle::new(world_width, world_height));

    // Create tile indices texture
    let tile_indices_texture = create_tile_indices_texture(play_field);
    let tile_indices_handle = images.add(tile_indices_texture);

    // Use tile texture from level assets
    let tile_texture_handle = level_assets.tile_texture.clone();

    // Create empty storage buffers with initial capacity
    let empty_connections = vec![ConnectionBufferData {
        start_pos: Vec2::ZERO,
        end_pos: Vec2::ZERO,
        strength: 0.0,
        distance: 0.0,
        _padding: Vec2::ZERO,
    }];
    let connections_buffer = buffers.add(ShaderStorageBuffer::from(empty_connections));

    let empty_previews = vec![PreviewBufferData {
        position: Vec2::ZERO,
        highlight_type: 0.0,
        _padding: 0.0,
    }];
    let preview_buffer = buffers.add(ShaderStorageBuffer::from(empty_previews));

    // Create material
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
            connections: connections_buffer,
            preview_count: 0,
            preview_highlights: preview_buffer,
            mycelium_color_low: LinearRgba::new(0.2, 0.4, 0.2, 1.0),
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
            FieldGround { material_handle },
            StateScoped(LevelState::Playing),
        ))
        .id()
}

/// Create texture containing tile type indices
fn create_tile_indices_texture(play_field: &PlayField) -> Image {
    let width = play_field.width as u32;
    let height = play_field.height as u32;
    let mut data = vec![0u8; (width * height * 4) as usize];

    info!("Creating tile indices texture: {}x{}", width, height);
    let mut tile_counts = std::collections::HashMap::new();

    for y in 0..height {
        for x in 0..width {
            let pos = GridPosition::new(x as i32, y as i32);
            let tile_type = play_field.get_tile(pos).unwrap_or_default();
            // We need to flip Y coordinate
            let pixel_index = grid_pos_to_texture_index(x, y, width, height);

            // Count tile types
            *tile_counts.entry(tile_type).or_insert(0) += 1;

            // Debug: log special tiles
            if !matches!(tile_type, TileType::Empty) {
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

/// Update connection data in storage buffers
fn update_connection_data(
    field_grounds: Query<&FieldGround>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, FieldGroundExtension>>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    game_state: Res<GameState>,
) {
    if !game_state.is_changed() {
        return;
    }

    for field_ground in field_grounds.iter() {
        if let Some(material) = materials.get_mut(&field_ground.material_handle) {
            // Get all connections from PlayField
            let connections = game_state.play_field.get_all_connections();

            // Create buffer data
            let mut connection_data = Vec::with_capacity(connections.len().max(1));

            for connection in connections {
                // Convert grid positions to normalized UV coordinates
                let grid_size = material.extension.grid_size;
                let start_uv = Vec2::new(
                    (connection.from_pos.x as f32 + 0.5) / grid_size.x,
                    1.0 - ((connection.from_pos.y as f32 + 0.5) / grid_size.y), // Flip Y coordinate
                );
                let end_uv = Vec2::new(
                    (connection.to_pos.x as f32 + 0.5) / grid_size.x,
                    1.0 - ((connection.to_pos.y as f32 + 0.5) / grid_size.y), // Flip Y coordinate
                );

                let distance = start_uv.distance(end_uv);

                connection_data.push(ConnectionBufferData {
                    start_pos: start_uv,
                    end_pos: end_uv,
                    strength: connection.strength,
                    distance,
                    _padding: Vec2::ZERO,
                });
            }

            // Ensure we have at least one element to avoid zero-sized buffer
            if connection_data.is_empty() {
                connection_data.push(ConnectionBufferData {
                    start_pos: Vec2::ZERO,
                    end_pos: Vec2::ZERO,
                    strength: 0.0,
                    distance: 0.0,
                    _padding: Vec2::ZERO,
                });
            }

            // Update storage buffer
            if let Some(buffer) = buffers.get_mut(&material.extension.connections) {
                buffer.set_data(connection_data.as_slice());
            }

            material.extension.connection_count = connections.len() as u32;

            info!(
                "Updated {} mycelium connections in shader",
                connections.len()
            );
        }
    }
}

/// Update preview highlights in storage buffer
fn update_shader_highlights(
    preview_connections: Res<crate::game::play_field::placement_preview::PreviewConnections>,
    field_grounds: Query<&FieldGround>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, FieldGroundExtension>>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    if !preview_connections.is_changed() {
        return;
    }

    for field_ground in field_grounds.iter() {
        if let Some(material) = materials.get_mut(&field_ground.material_handle) {
            let mut preview_data = Vec::new();
            let grid_size = material.extension.grid_size;

            // Check if we have any preview data at all
            let has_preview_data = preview_connections.preview_position.is_some()
                || !preview_connections.connected_positions.is_empty()
                || !preview_connections.empty_connection_points.is_empty()
                || !preview_connections.existing_connection_targets.is_empty();

            if has_preview_data {
                // Add preview position
                if let Some(preview_pos) = preview_connections.preview_position {
                    let preview_uv = Vec2::new(
                        (preview_pos.x as f32 + 0.5) / grid_size.x,
                        1.0 - ((preview_pos.y as f32 + 0.5) / grid_size.y),
                    );
                    preview_data.push(PreviewBufferData {
                        position: preview_uv,
                        highlight_type: -1.0,
                        _padding: 0.0,
                    });
                }

                // Add all connected positions
                for connected_pos in &preview_connections.connected_positions {
                    let connected_uv = Vec2::new(
                        (connected_pos.x as f32 + 0.5) / grid_size.x,
                        1.0 - ((connected_pos.y as f32 + 0.5) / grid_size.y),
                    );
                    preview_data.push(PreviewBufferData {
                        position: connected_uv,
                        highlight_type: -2.0,
                        _padding: 0.0,
                    });
                }

                // Add all empty connection points
                for empty_pos in &preview_connections.empty_connection_points {
                    let empty_uv = Vec2::new(
                        (empty_pos.x as f32 + 0.5) / grid_size.x,
                        1.0 - ((empty_pos.y as f32 + 0.5) / grid_size.y),
                    );
                    preview_data.push(PreviewBufferData {
                        position: empty_uv,
                        highlight_type: -3.0,
                        _padding: 0.0,
                    });
                }

                // Add all existing mushroom connection targets
                for existing_target in &preview_connections.existing_connection_targets {
                    let target_uv = Vec2::new(
                        (existing_target.x as f32 + 0.5) / grid_size.x,
                        1.0 - ((existing_target.y as f32 + 0.5) / grid_size.y),
                    );
                    preview_data.push(PreviewBufferData {
                        position: target_uv,
                        highlight_type: -4.0,
                        _padding: 0.0,
                    });
                }
            }

            // Always ensure we have at least one element (dummy with highlight_type 0.0)
            if preview_data.is_empty() {
                preview_data.push(PreviewBufferData {
                    position: Vec2::new(-1000.0, -1000.0), // Far off-screen
                    highlight_type: 0.0,
                    _padding: 0.0,
                });
            }

            // Update storage buffer
            if let Some(buffer) = buffers.get_mut(&material.extension.preview_highlights) {
                buffer.set_data(preview_data.as_slice());
            }

            // Set count to 0 if we only have the dummy element
            material.extension.preview_count = if has_preview_data {
                preview_data.len() as u32
            } else {
                0
            };
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
