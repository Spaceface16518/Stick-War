mod battle;
mod config;
mod game;
mod model;
mod rendering;
mod ui;
mod units;

use bevy::prelude::*;
use game::GamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Stick War".into(),
                resolution: (1280, 720).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .run();
}
