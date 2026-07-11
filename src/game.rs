use crate::{battle::BattlePlugin, model::*, ui::UiPlugin};
use bevy::{prelude::*, winit::WinitSettings};

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            // UI-only states do not need a continuously running game loop. Start
            // reactively so the title screen is low-power from the first frame.
            .insert_resource(WinitSettings::desktop_app())
            .insert_resource(load_game_config())
            .add_message::<TrainUnitRequest>()
            .add_message::<DamageMessage>()
            .add_plugins((BattlePlugin, UiPlugin))
            .add_systems(Startup, setup_camera)
            .add_systems(OnEnter(AppState::MainMenu), use_reactive_update_mode)
            .add_systems(OnEnter(AppState::Battle), use_continuous_update_mode)
            .add_systems(OnEnter(AppState::Results), use_reactive_update_mode);
    }
}

/// Let window/input events drive static UI screens instead of polling and
/// redrawing them continuously. This is Bevy's standard desktop-app mode.
fn use_reactive_update_mode(mut settings: ResMut<WinitSettings>) {
    *settings = WinitSettings::desktop_app();
}

/// Battle simulation, animation, and timers require an update every frame.
fn use_continuous_update_mode(mut settings: ResMut<WinitSettings>) {
    *settings = WinitSettings::game();
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
