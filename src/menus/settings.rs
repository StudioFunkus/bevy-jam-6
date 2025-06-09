//! The settings menu.
//!
//! Additional settings and accessibility options should go here.

use bevy::{audio::Volume, input::common_conditions::input_just_pressed, prelude::*, ui::Val::*};

use crate::{
    game::fixed_timestep::FixedTimestepConfig,
    menus::Menu,
    screens::Screen,
    theme::{assets::ThemeAssets, prelude::*, widget::slice_2_slicer},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Settings), spawn_settings_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Settings).and(input_just_pressed(KeyCode::Escape))),
    );

    app.register_type::<GlobalVolumeLabel>();
    app.register_type::<TimestepLabel>();
    app.add_systems(
        Update,
        (update_global_volume_label, update_timestep_label).run_if(in_state(Menu::Settings)),
    );
}

fn spawn_settings_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme_assets: Res<ThemeAssets>,
) {
    let font_handle = asset_server.load("fonts/PixelOperatorMonoHB.ttf");
    commands.spawn((
        widget::ui_root("Settings Menu", Some(font_handle.clone())),
        GlobalZIndex(2),
        StateScoped(Menu::Settings),
        children![
            widget::header("Settings", Some(font_handle.clone())),
            settings_grid(font_handle.clone()),
            widget::button_sliced(
                "Back",
                go_back_on_click,
                theme_assets.slice_2.clone(),
                slice_2_slicer(),
                font_handle.clone()
            ),
        ],
    ));
}

fn settings_grid(font: Handle<Font>) -> impl Bundle {
    (
        Name::new("Settings Grid"),
        Node {
            display: Display::Grid,
            row_gap: Px(10.0),
            column_gap: Px(30.0),
            grid_template_columns: RepeatedGridTrack::px(2, 400.0),
            ..default()
        },
        children![
            (
                widget::label("Master Volume", Some(font.clone())),
                Node {
                    justify_self: JustifySelf::End,
                    ..default()
                }
            ),
            global_volume_widget(font.clone()),
            (
                widget::label("Game Speed (Hz)", Some(font.clone())),
                Node {
                    justify_self: JustifySelf::End,
                    ..default()
                }
            ),
            timestep_widget(font.clone()),
        ],
    )
}

fn global_volume_widget(font: Handle<Font>) -> impl Bundle {
    (
        Name::new("Global Volume Widget"),
        Node {
            justify_self: JustifySelf::Start,
            ..default()
        },
        children![
            widget::button_small("-", lower_global_volume),
            (
                Name::new("Current Volume"),
                Node {
                    padding: UiRect::horizontal(Px(10.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![(widget::label("", Some(font.clone())), GlobalVolumeLabel)],
            ),
            widget::button_small("+", raise_global_volume),
        ],
    )
}

const MIN_VOLUME: f32 = 0.0;
const MAX_VOLUME: f32 = 3.0;

fn lower_global_volume(_: Trigger<Pointer<Click>>, mut global_volume: ResMut<GlobalVolume>) {
    let linear = (global_volume.volume.to_linear() - 0.1).max(MIN_VOLUME);
    global_volume.volume = Volume::Linear(linear);
}

fn raise_global_volume(_: Trigger<Pointer<Click>>, mut global_volume: ResMut<GlobalVolume>) {
    let linear = (global_volume.volume.to_linear() + 0.1).min(MAX_VOLUME);
    global_volume.volume = Volume::Linear(linear);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct GlobalVolumeLabel;

fn update_global_volume_label(
    global_volume: Res<GlobalVolume>,
    mut label: Single<&mut Text, With<GlobalVolumeLabel>>,
) {
    let percent = 100.0 * global_volume.volume.to_linear();
    label.0 = format!("{percent:3.0}%");
}

fn go_back_on_click(
    _: Trigger<Pointer<Click>>,
    screen: Res<State<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    next_menu.set(if screen.get() == &Screen::Title {
        Menu::Main
    } else {
        Menu::Pause
    });
}

fn go_back(screen: Res<State<Screen>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(if screen.get() == &Screen::Title {
        Menu::Main
    } else {
        Menu::Pause
    });
}

fn timestep_widget(font: Handle<Font>) -> impl Bundle {
    (
        Name::new("Timestep Widget"),
        Node {
            justify_self: JustifySelf::Start,
            ..default()
        },
        children![
            widget::button_small("-", lower_timestep),
            (
                Name::new("Current Timestep"),
                Node {
                    padding: UiRect::horizontal(Px(10.0)),
                    justify_content: JustifyContent::Center,
                    min_width: Val::Px(60.0),
                    ..default()
                },
                children![(widget::label("", Some(font.clone())), TimestepLabel)],
            ),
            widget::button_small("+", raise_timestep),
        ],
    )
}

fn lower_timestep(_: Trigger<Pointer<Click>>, mut config: ResMut<FixedTimestepConfig>) {
    let new_hz = (config.target_hz - 5.0).max(config.min_hz);
    config.set_hz(new_hz);
}

fn raise_timestep(_: Trigger<Pointer<Click>>, mut config: ResMut<FixedTimestepConfig>) {
    let new_hz = (config.target_hz + 5.0).min(config.max_hz);
    config.set_hz(new_hz);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct TimestepLabel;

fn update_timestep_label(
    config: Res<FixedTimestepConfig>,
    mut label: Single<&mut Text, With<TimestepLabel>>,
) {
    let speed_mult = config.speed_multiplier();
    label.0 = format!("{:.0} Hz ({:.1}x)", config.target_hz, speed_mult);
}
