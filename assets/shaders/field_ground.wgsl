// Field ground extended material shader
// Inherits from standard material and adds custom tile rendering logic

#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

// Storage buffer for connections
struct ConnectionData {
    start_pos: vec2<f32>,
    end_pos: vec2<f32>,
    strength: f32,
    distance: f32,
    _padding: vec2<f32>,
};

// Storage buffer for preview highlights
struct PreviewData {
    position: vec2<f32>,
    highlight_type: f32,
    _padding: f32,
};

// Tile texture atlas
@group(2) @binding(100) var tile_texture: texture_2d<f32>;
@group(2) @binding(101) var tile_sampler: sampler;

// Tile indices texture
@group(2) @binding(102) var tile_indices: texture_2d<f32>;
@group(2) @binding(103) var tile_indices_sampler: sampler;

// Time for animations
@group(2) @binding(104) var<uniform> time: f32;

// Grid dimensions
@group(2) @binding(105) var<uniform> grid_size: vec2<f32>;

// Connection data
@group(2) @binding(106) var<uniform> connection_count: u32;
@group(2) @binding(107) var<storage, read> connections: array<ConnectionData>;

// Preview highlights
@group(2) @binding(108) var<uniform> preview_count: u32;
@group(2) @binding(109) var<storage, read> preview_highlights: array<PreviewData>;

// Mycelium colors
@group(2) @binding(110) var<uniform> mycelium_color_low: vec4<f32>;
@group(2) @binding(111) var<uniform> mycelium_color_high: vec4<f32>;

// Animation parameters
@group(2) @binding(112) var<uniform> pulse_speed: f32;
@group(2) @binding(113) var<uniform> glow_intensity: f32;

// Line rendering parameters
@group(2) @binding(114) var<uniform> line_width: f32;

// SDF for line segment (not technically SDF, but used for line rendering)
fn sdf_line_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, width: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    let d = length(pa - ba * h);
    return smoothstep(width, width * 0.5, d);
}

// Get tile sprite UV coordinates
fn get_tile_uv(tile_index: f32, local_uv: vec2<f32>) -> vec2<f32> {
    // Convert normalized index back to sprite index (0-20)
    let sprite_idx = round(tile_index * 21.0);
    
    // Our texture atlas is a vertical strip (1 column, 21 rows)
    // Each tile is 16x16 pixels, with 2 pixel padding
    let tile_size = 16.0;
    let padding = 2.0;
    let atlas_width = tile_size;
    let atlas_height = (tile_size + padding) * 21.0 - padding;
    
    // Calculate the Y offset for this sprite
    let y_offset = sprite_idx * (tile_size + padding);
    
    // Map local UV (0-1) to the tile's region in the atlas
    let tile_uv = vec2<f32>(
        local_uv.x * tile_size / atlas_width,
        (y_offset + local_uv.y * tile_size) / atlas_height
    );
    
    return tile_uv;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // Get UV coordinates (0-1 range)
    var uv = in.uv;
    
    // Calculate which grid cell we're in
    let grid_pos = floor(uv * grid_size);
    let cell_uv = fract(uv * grid_size);
    
    // Sample tile index from indices texture
    let tile_data = textureSample(tile_indices, tile_indices_sampler, uv);
    let tile_index = tile_data.r;
    
    // Get tile texture
    let tile_atlas_uv = get_tile_uv(tile_index, cell_uv);
    var base_color = textureSample(tile_texture, tile_sampler, tile_atlas_uv);
    
    // Initialize highlight factors
    var preview_highlight = 0.0;
    var connected_highlight = 0.0;
    var empty_connection_highlight = 0.0;
    var existing_target_highlight = 0.0;
    
    // Process preview highlights
    for (var i = 0u; i < preview_count; i = i + 1u) {
        let preview = preview_highlights[i];
        let highlight_pos = preview.position;
        let cell_dist = length((uv - highlight_pos) * grid_size * 0.7);
        
        if (cell_dist < 0.5) {
            if (preview.highlight_type == -1.0) {
                // Preview position highlight (cyan)
                preview_highlight = max(preview_highlight, 0.6 + 0.2 * sin(time * 3.0));
            } else if (preview.highlight_type == -2.0) {
                // Connected position highlight (green)
                connected_highlight = max(connected_highlight, 0.4 + 0.1 * sin(time * 4.0 + 1.57));
            } else if (preview.highlight_type == -3.0) {
                // Empty connection point highlight (red)
                empty_connection_highlight = max(empty_connection_highlight, 0.5 + 0.15 * sin(time * 5.0));
            } else if (preview.highlight_type == -4.0) {
                // Existing target highlight (blue)
                existing_target_highlight = max(existing_target_highlight, 0.5 + 0.1 * sin(time * 2.0));
            }
        }
    }
    
    // Process mycelium connections
    for (var i = 0u; i < connection_count; i = i + 1u) {
        let connection = connections[i];
        
        // Skip invalid connections
        if (connection.distance <= 0.0) {
            continue;
        }
        
        // Calculate mycelium line
        let line_alpha = sdf_line_segment(uv, connection.start_pos, connection.end_pos, line_width);
        
        if (line_alpha > 0.0) {
            // Energy pulse animation
            let flow_offset = length(uv - connection.start_pos) / connection.distance;
            let pulse = sin((flow_offset - time * pulse_speed) * 6.28318) * 0.5 + 0.5;
            
            // Mycelium color with pulse
            let mycelium_color = mix(mycelium_color_low, mycelium_color_high, pulse * connection.strength);
            let glow = 1.0 + pulse * glow_intensity;
            
            // Blend mycelium over base
            base_color = mix(base_color, mycelium_color * glow, line_alpha * connection.strength);
        }
    }
    
    // Apply preview highlights
    if (preview_highlight > 0.0) {
        let preview_color = vec4<f32>(0.2, 0.8, 1.0, 1.0);
        base_color = mix(base_color, preview_color, preview_highlight * 0.5);
        
        // Edge highlight
        let edge_dist = min(min(cell_uv.x, 1.0 - cell_uv.x), min(cell_uv.y, 1.0 - cell_uv.y));
        if (edge_dist < 0.05) {
            base_color = mix(base_color, preview_color, 0.8);
        }
    }
    
    if (connected_highlight > 0.0) {
        let connected_color = vec4<f32>(0.2, 1.0, 0.4, 1.0);
        base_color = mix(base_color, connected_color, connected_highlight * 0.4);
        
        // Edge highlight
        let edge_dist = min(min(cell_uv.x, 1.0 - cell_uv.x), min(cell_uv.y, 1.0 - cell_uv.y));
        if (edge_dist < 0.03) {
            base_color = mix(base_color, connected_color, 0.6);
        }
    }
    
    if (empty_connection_highlight > 0.0) {
        let empty_color = vec4<f32>(1.0, 0.3, 0.2, 1.0);
        base_color = mix(base_color, empty_color, empty_connection_highlight * 0.4);
        
        // Dashed edge
        let edge_dist = min(min(cell_uv.x, 1.0 - cell_uv.x), min(cell_uv.y, 1.0 - cell_uv.y));
        let dash_pattern = sin((cell_uv.x + cell_uv.y) * 20.0 + time * 8.0) * 0.5 + 0.5;
        if (edge_dist < 0.04 && dash_pattern > 0.3) {
            base_color = mix(base_color, empty_color, 0.7);
        }
    }
    
    if (existing_target_highlight > 0.0) {
        let target_color = vec4<f32>(0.3, 0.6, 1.0, 1.0);
        
        // Thin edge outline
        let edge_dist = min(min(cell_uv.x, 1.0 - cell_uv.x), min(cell_uv.y, 1.0 - cell_uv.y));
        if (edge_dist < 0.02) {
            base_color = mix(base_color, target_color, existing_target_highlight * 0.6);
        }
    }
    
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    
    // Override the base color with our calculated color
    pbr_input.material.base_color = base_color;
    pbr_input.material.perceptual_roughness = 0.7;
    pbr_input.material.metallic = 0.0;
    
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    // In deferred mode, output to G-buffer
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // Apply lighting
    out.color = apply_pbr_lighting(pbr_input);
    
    // Apply post-processing
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}