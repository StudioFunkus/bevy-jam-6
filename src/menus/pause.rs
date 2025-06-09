//! The pause menu.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    menus::Menu,
    screens::Screen,
    theme::{
        assets::ThemeAssets,
        widget::{self, slice_2_slicer},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), spawn_pause_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Pause).and(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_pause_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme_assets: Res<ThemeAssets>,
) {
    let font_handle = asset_server.load("fonts/PixelOperatorMonoHB.ttf");
    commands.spawn((
        widget::ui_root("Pause Menu", Some(font_handle.clone())),
        GlobalZIndex(2),
        StateScoped(Menu::Pause),
        children![
            widget::header("Game paused", Some(font_handle.clone())),
            widget::button_sliced(
                "Continue",
                close_menu,
                theme_assets.slice_2.clone(),
                slice_2_slicer(),
                font_handle.clone()
            ),
            widget::button_sliced(
                "Settings",
                open_settings_menu,
                theme_assets.slice_2.clone(),
                slice_2_slicer(),
                font_handle.clone()
            ),
            widget::button_sliced(
                "Quit",
                quit_to_title,
                theme_assets.slice_2.clone(),
                slice_2_slicer(),
                font_handle.clone()
            ),
        ],
    ));
}

fn open_settings_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn close_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

fn quit_to_title(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
