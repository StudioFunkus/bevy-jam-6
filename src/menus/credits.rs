//! The credits menu.

use bevy::{
    ecs::spawn::SpawnIter, input::common_conditions::input_just_pressed, prelude::*, ui::Val::*,
};

use crate::{menus::Menu, theme::{assets::ThemeAssets, prelude::*, widget::slice_2_slicer}};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Credits), spawn_credits_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Credits).and(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_credits_menu(mut commands: Commands, asset_server: Res<AssetServer>, theme_assets: Res<ThemeAssets>) {
    let font_handle = asset_server.load("fonts/PixelOperatorMonoHB.ttf");
    commands.spawn((
        widget::ui_root("Credits Menu", Some(font_handle.clone())),
        GlobalZIndex(2),
        StateScoped(Menu::Credits),
        children![
            widget::header("Created by", Some(font_handle.clone())),
            created_by(font_handle.clone()),
            widget::header("Assets", Some(font_handle.clone())),
            assets(font_handle.clone()),
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

fn created_by(font: Handle<Font>) -> impl Bundle {
    grid(vec![["Studio Funkus", "Made the game"]], font.clone())
}

fn assets(font: Handle<Font>) -> impl Bundle {
    grid(vec![
        ["Button SFX", "CC0 by Jaszunio15"],
        ["Font", "Pixel Operator CC0 by Jayvee Enaguas"],
        ["Art", "by darwinscoat, narlantweed"],
        ["Music", "by sazzles"],
        ["Writing", "by Lolly"],
        ["Programming", "whompratt, sazzles, rolypoly, drif, narlantweed"],
        [
            "Bevy logo",
            "All rights reserved by the Bevy Foundation, permission granted for splash screen use when unmodified",
        ],
    ], font.clone())
}

fn grid(content: Vec<[&'static str; 2]>, font: Handle<Font>) -> impl Bundle {
    (
        Name::new("Grid"),
        Node {
            display: Display::Grid,
            row_gap: Px(10.0),
            column_gap: Px(30.0),
            grid_template_columns: RepeatedGridTrack::px(2, 400.0),
            ..default()
        },
        Children::spawn(SpawnIter(content.into_iter().flatten().enumerate().map(
            move |(i, text)| {
                (
                    widget::label(text, Some(font.clone())),
                    Node {
                        justify_self: if i % 2 == 0 {
                            JustifySelf::End
                        } else {
                            JustifySelf::Start
                        },
                        ..default()
                    },
                )
            },
        ))),
    )
}

fn go_back_on_click(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}
