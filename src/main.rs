// Bevy ECS systems express their filters and component tuples in function
// signatures. Keeping those types visible is more useful than hiding them
// behind aliases solely to satisfy these two generic Clippy thresholds.
#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod animation;
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
                canvas: Some("#bevy".into()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .run();
}
