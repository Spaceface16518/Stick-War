mod battle;
mod characters;
mod config;
mod game;
mod model;
mod rendering;
mod ui;
mod units;

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use game::GamePlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: "config".into(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Stick War".into(),
                        resolution: (1280, 720).into(),
                        resizable: true,
                        canvas: Some("#bevy".into()),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(GamePlugin)
        .run();
}
