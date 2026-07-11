use crate::{battle::BattlePlugin, model::*, ui::UiPlugin};
use bevy::prelude::*;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .insert_resource(load_game_config())
            .add_message::<TrainUnitRequest>()
            .add_message::<DamageMessage>()
            .add_plugins((BattlePlugin, UiPlugin))
            .add_systems(Startup, setup_camera);
    }
}
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        BattleCamera,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: 720.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
