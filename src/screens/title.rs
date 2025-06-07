//! The title screen that appears after the splash screen.

use bevy::prelude::*;

use crate::{audio::music, menus::Menu, screens::{assets::ScreenAssets, Screen}};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), (open_main_menu, start_title_music));
    app.add_systems(OnExit(Screen::Title), close_menu);
}

fn open_main_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
// Spawn background music
fn start_title_music(mut commands: Commands,screen_assets: Res<ScreenAssets>) {
    commands.spawn((
        Name::new("Gameplay Music"),
        StateScoped(Screen::Title),
        music(screen_assets.music.clone()),
    ));
}
