// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
mod audio;
#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod menus;
mod screens;
mod theme;

use bevy::{
    asset::AssetMetaCheck, pbr::light_consts, picking::mesh_picking::MeshPickingPlugin, prelude::*,
};
use bevy_hanabi::HanabiPlugin;
use bevy_panorbit_camera::{FocusBoundsShape, PanOrbitCameraPlugin};
use bevy_rich_text3d::{LoadFonts, Text3dPlugin};
use bevy_sprite3d::Sprite3dPlugin;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

// Embed the font at compile time
const DEFAULT_FONT: &[u8] = include_bytes!("../assets/fonts/PixelOperatorMonoHB.ttf");

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Bevy Jam 6".to_string(),
                        fit_canvas_to_parent: true,
                        resolution: (1280., 720.).into(),
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            Sprite3dPlugin,
            PanOrbitCameraPlugin,
            MeshPickingPlugin,
            HanabiPlugin,
            DialoguePlugin,
            DialogueUIPlugin,
            // DialogueDebugPlugin,
        ));

        app.insert_resource(LoadFonts {
            font_embedded: vec![DEFAULT_FONT],
            ..Default::default()
        });

        app.add_plugins(Text3dPlugin {
            default_atlas_dimension: (1024, 1024),
            ..default()
        });

        // Add other plugins.
        app.add_plugins((
            asset_tracking::plugin,
            audio::plugin,
            game::plugin,
            // #[cfg(feature = "dev")]
            dev_tools::plugin,
            menus::plugin,
            screens::plugin,
            theme::plugin,
        ));

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        app.insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)));

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));
        app.configure_sets(FixedUpdate, PausableSystems.run_if(in_state(Pause(false))));

        // Spawn the main camera and lighting.
        app.add_systems(Startup, (spawn_camera, setup_lighting));
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
struct Pause(pub bool);

#[derive(Component)]
/// A marker component for the main camera entity.
pub struct MainCamera;

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;

use bevy_panorbit_camera::PanOrbitCamera;
use funkus_dialogue_core::DialoguePlugin;
use funkus_dialogue_ui::DialogueUIPlugin;
use game::carddeck::constants::CARD_LAYER;

fn spawn_camera(mut commands: Commands) {
    // Hand Camera
    commands.spawn((
        CARD_LAYER,
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
    ));

    // Main Camera
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        DistanceFog {
            color: Color::srgb(0.25, 0.25, 0.25),
            falloff: FogFalloff::Linear {
                start: 20.0,
                end: 150.0,
            },
            ..default()
        },
        MainCamera,
        Camera::default(),
        PanOrbitCamera {
            button_orbit: MouseButton::Middle,
            pitch_upper_limit: Some(1.0),
            pitch_lower_limit: Some(0.25),
            zoom_upper_limit: Some(20.0),
            zoom_lower_limit: 5.0,
            focus_bounds_origin: Vec3::ZERO,
            focus_bounds_shape: Some(FocusBoundsShape::Cuboid(Cuboid::new(12.0, 0.0, 12.0))),
            ..default()
        },
        Transform::from_xyz(0.0, 7.0, 14.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));
}

fn setup_lighting(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.9, 0.85, 0.7),
        brightness: 200.0,
        affects_lightmapped_meshes: false,
    });

    commands.spawn((
        Name::new("Sun Light"),
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            color: Color::srgb(1.0, 0.98, 0.82),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::PI / 3.0,
            std::f32::consts::PI / 4.0,
            0.0,
        )),
    ));

    commands.spawn((
        Name::new("Fill Light"),
        DirectionalLight {
            illuminance: 200.0,
            color: Color::srgb(0.7, 0.8, 1.0),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::PI / 6.0,
            -std::f32::consts::PI * 3.0 / 4.0,
            0.0,
        )),
    ));
}
