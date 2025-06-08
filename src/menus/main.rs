//! The main menu (seen on the title screen).

use bevy::{ecs::spawn::SpawnWith, prelude::*};

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
#[derive(Component)]
struct Spore;

// spawn all menu widgets

fn spawn_main_menu(mut commands: Commands, _screen_assets: Res<ScreenAssets>) {
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
        #[cfg(target_family = "wasm")]
        children![
            widget::button("Play", enter_loading_or_gameplay_screen),
            widget::button("Settings", open_settings_menu),
            widget::button("Credits", open_credits_menu),
        ],
    ));
}

// spawn all main menu art assets

fn spawn_main_menu_art_assets(mut commands: Commands, screen_assets: Res<ScreenAssets>) {
    commands.spawn((
        Name::new("Main Menu - Art"),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Default,
            justify_content: JustifyContent::Default,
            flex_direction: FlexDirection::Row,
            ..default()
        },
        Pickable::IGNORE,
        GlobalZIndex(0),
        StateScoped(Menu::Main),
        children![
            // Splash Art
            (
                Name::new("Container - Splash Art"),
                Node {
                    width: Val::Percent(60.0),
                    align_items: AlignItems::Center,
                    justify_items: JustifyItems::Center,
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![(
                    Name::new("Splash Art"),
                    Pickable::IGNORE,
                    ImageNode::new(screen_assets.titlescreen.clone()),
                    Node {
                        left: Val::Percent(22.5),
                        top: Val::Percent(0.0),
                        height: Val::Percent(90.0),
                        ..default()
                    }
                )],
            ),
            // Game Title
            (
                Name::new("Container - Game Title"),
                Node {
                    width: Val::Percent(60.0),
                    align_items: AlignItems::Center,
                    justify_items: JustifyItems::Center,
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![(
                    Name::new("Game Title"),
                    Pickable::IGNORE,
                    ImageNode::new(screen_assets.gametitle.clone()),
                    Node {
                        left: Val::Percent(-70.0),
                        top: Val::Percent(25.0),
                        height: Val::Percent(60.0),
                        ..default()
                    }
                )],
            ),
            // Spore 1
            (
                Name::new("Spore 1"),
                Node {
                    height: Val::Px(100.0),
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Center,
                    ..default()
                },
                ImageNode::new(screen_assets.spore1.clone()),
                Pickable::IGNORE,
                Spore,
            ),
            // // lil spore 1
            // widget::image(
            //     screen_assets.spore1.clone(),
            //     None,
            //     None,
            //     None,
            //     None,
            //     Val::Px(20.),
            //     Val::Px(20.),
            //     PositionType::Absolute,
            // ),
            // // lil spore 2
            // widget::image(
            //     screen_assets.spore2.clone(),
            //     None,
            //     None,
            //     None,
            //     None,
            //     Val::Px(20.),
            //     Val::Px(20.),
            //     PositionType::Absolute,
            // ),
            // //lil spore 3
            // widget::image(
            //     screen_assets.spore3.clone(),
            //     None,
            //     None,
            //     None,
            //     None,
            //     Val::Px(20.),
            //     Val::Px(20.),
            //     PositionType::Absolute,
            // )
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
