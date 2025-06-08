//! The main menu (seen on the title screen).

use bevy::prelude::*;

use crate::{
    asset_tracking::ResourceHandles,
    menus::Menu,
    screens::{Screen, assets::ScreenAssets},
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Menu::Main),
        (spawn_main_menu, spawn_main_menu_art_assets),
    );
}

//component to store floating parameters for spores

// spawn all menu widgets

fn spawn_main_menu(mut commands: Commands, screen_assets: Res<ScreenAssets>) {
    commands.spawn((
        widget::ui_root("Main Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::Main),
        #[cfg(not(target_family = "wasm"))]
        children![
            widget::button("Play", enter_loading_or_gameplay_screen),
            widget::button("Settings", open_settings_menu),
            widget::button("Credits", open_credits_menu),
            widget::button("Exit", exit_app),
        ],
    ));
}

// spawn all main menu art assets

fn spawn_main_menu_art_assets(mut commands: Commands, screen_assets: Res<ScreenAssets>) {
    commands.spawn((
        widget::ui_root("Main Menu"),
        GlobalZIndex(0),
        StateScoped(Menu::Main),
        #[cfg(not(target_family = "wasm"))]
        children![
            widget::image(
                screen_assets.titlescreen.clone(),
                Val::Percent(100.),
                Val::Percent(100.),
                PositionType::Absolute,
            ),
            widget::image(
                screen_assets.gametitle.clone(),
                Val::Px(1000.),
                Val::Px(200.),
                PositionType::Absolute,
            ),
            widget::image(
                screen_assets.spore1.clone(),
                Val::Px(20.),
                Val::Px(20.),
                PositionType::Relative,
            ),
            widget::image(
                screen_assets.spore2.clone(),
                Val::Px(20.),
                Val::Px(20.),
                PositionType::Relative,
            ),
            widget::image(
                screen_assets.spore3.clone(),
                Val::Px(20.),
                Val::Px(20.),
                PositionType::Relative,
            )
        ],
    ));
}

// navigation tools for the main menu widgets

fn enter_loading_or_gameplay_screen(
    _: Trigger<Pointer<Click>>,
    resource_handles: Res<ResourceHandles>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if resource_handles.is_all_done() {
        next_screen.set(Screen::Gameplay);
    } else {
        next_screen.set(Screen::Loading);
    }
}

fn open_settings_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn open_credits_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Credits);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: Trigger<Pointer<Click>>, mut app_exit: EventWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}

//system for making spores move in the main menu
