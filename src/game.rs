use crate::{battle::BattlePlugin, model::*, ui::UiPlugin};
use bevy::prelude::*;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_resource::<GameConfig>()
            .add_message::<TrainUnitRequest>()
            .add_message::<DamageMessage>()
            .add_plugins((BattlePlugin, UiPlugin))
            .add_systems(Startup, setup_camera);
    }
}
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 2.75,
            ..OrthographicProjection::default_2d()
        }),
    ));
}
