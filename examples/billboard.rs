use core::f32;

use bevy::{color::palettes::tailwind, input::common_conditions::input_just_released, prelude::*};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_sprite3d::{Sprite3d, Sprite3dBuilder, Sprite3dParams, Sprite3dPlugin};

fn main() {
    let mut app = App::new();

    let default_plugins = DefaultPlugins
        .set(WindowPlugin {
            primary_window: Window {
                title: "Billboard Demo".to_string(),
                canvas: Some("#bevy".to_string()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }
            .into(),
            ..default()
        })
        .set(ImagePlugin::default_nearest())
        .build();
    app.add_plugins(default_plugins);

    // State
    app.init_state::<GameState>();

    // Plugins
    app.add_plugins(Sprite3dPlugin)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(WorldInspectorPlugin::default())
        .add_plugins(PanOrbitCameraPlugin);

    // Resources
    app.insert_resource(BillboardAsset::default());

    // Systems
    app.add_systems(Startup, load_assets)
        .add_systems(Update, check_asset_loading)
        .add_systems(OnEnter(GameState::Ready), setup)
        .add_systems(Update, face_camera.run_if(in_state(GameState::Ready)))
        .add_systems(Update, update_fred.run_if(in_state(GameState::Ready)));

    app.run();
}

#[derive(States, Hash, Clone, PartialEq, Eq, Debug, Default)]
enum GameState {
    #[default]
    Loading,
    Ready,
}

#[derive(Resource, Default)]
struct BillboardAsset {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
struct FaceCamera;

fn load_assets(
    mut billboard_asset: ResMut<BillboardAsset>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) -> Result {
    // Load Billboard Assets
    billboard_asset.image = asset_server.load("fred.png");
    billboard_asset.layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 32),
        6,
        4,
        None,
        None,
    ));

    Ok(())
}

fn check_asset_loading(
    asset_server: Res<AssetServer>,
    billboard_asset: Res<BillboardAsset>,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result {
    if asset_server
        .get_load_state(billboard_asset.image.id())
        .is_some_and(|s| s.is_loaded())
    {
        next_state.set(GameState::Ready);
    }

    Ok(())
}

fn setup(
    mut commands: Commands,
    billboard_asset: Res<BillboardAsset>,
    mut sprite_params: Sprite3dParams,
) -> Result {
    // Point Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10000000.0,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));

    // Camera
    commands.spawn((
        Name::new("Camera"),
        PanOrbitCamera::default(),
        IsDefaultUiCamera,
        Transform::from_xyz(0.0, 7.0, 14.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));

    // Ground Plane
    commands.spawn((
        Mesh3d(
            sprite_params
                .meshes
                .add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10)),
        ),
        MeshMaterial3d(
            sprite_params
                .materials
                .add(Color::from(tailwind::SLATE_400)),
        ),
    ));

    // Spawn Billboard Sprite
    let atlas = TextureAtlas {
        layout: billboard_asset.layout.clone(),
        index: 1 as usize,
    };

    commands.spawn((
        Name::new("Fred"),
        Sprite3dBuilder {
            image: billboard_asset.image.clone(),
            pixels_per_metre: 16.0,
            double_sided: false,
            ..default()
        }
        .bundle_with_atlas(&mut sprite_params, atlas),
        Transform::from_xyz(0.0, 1.0, 0.0),
        FaceCamera,
    ));

    Ok(())
}

fn face_camera(
    cam_transform: Single<&Transform, With<Camera>>,
    mut query: Query<&mut Transform, (With<FaceCamera>, Without<Camera>)>,
) {
    for mut transform in query.iter_mut() {
        let mut target = cam_transform.translation;
        target.y = transform.translation.y;
        transform.look_at(-target, Vec3::Y);
    }
}

/// Note that some accomodations need be made for Fred's sprite sheet.
fn update_fred(mut fred: Query<(&Transform, &mut Sprite3d), With<FaceCamera>>) -> Result {
    let (transform, mut sprite) = fred.single_mut()?;
    let atlas = sprite.texture_atlas.as_mut().unwrap();

    let (axis, angle) = transform.rotation.to_axis_angle();
    let mut angle = (angle * axis.y) + (f32::consts::PI / 4.0); // Offset angle by 45 degrees

    // Rotation runs from -2*PI to 2*PI, which is 2 full rotations
    // If negative, offset by a single rotation to get the positive
    if angle < 0.0 {
        angle += 2.0 * f32::consts::PI;
    }
    // Since we're offseting the rotation by 45 degrees above, we
    // now need to clamp adjust so that rotations above 2*PI roll
    // over to zero.
    if angle > (2.0 * f32::consts::PI) {
        angle -= 2.0 * f32::consts::PI;
    }

    let mut index = (angle / (std::f32::consts::PI / 2.0)) as usize;

    // Required to accomodate fred's sprite sheet
    // Ideally, sprites would be ordered, front > left > back > right,
    // but Fred's is front > back > left > right, so we need to swap 1 and 2.
    if index == 1 {
        index = 2;
    } else if index == 2 {
        index = 1;
    }

    // Then multiply by 6, which is the number of columns in the atlas
    index *= 6;

    atlas.index = index;

    Ok(())
}
