use crate::{
    animation::CharacterAnimationPlugin, battle::BattlePlugin, model::*, rendering::TimeOfDay,
    ui::UiPlugin,
};
use bevy::{camera::Hdr, prelude::*, winit::WinitSettings};
use bevy::{core_pipeline::tonemapping::Tonemapping, render::view::ColorGrading};

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let config = load_game_config();
        let time_of_day = TimeOfDay::from_config(&config.graphics);
        app.init_state::<AppState>()
            .add_computed_state::<RuntimeActivity>()
            // UI-only states do not need a continuously running game loop. Start
            // reactively so the title screen is low-power from the first frame.
            .insert_resource(WinitSettings::desktop_app())
            .insert_resource(config)
            .insert_resource(time_of_day)
            .add_message::<TrainUnitRequest>()
            .add_message::<DamageMessage>()
            .add_plugins((CharacterAnimationPlugin, BattlePlugin, UiPlugin))
            .add_systems(Startup, setup_camera)
            .add_systems(
                OnEnter(RuntimeActivity::Interface),
                use_reactive_update_mode,
            )
            .add_systems(
                OnEnter(RuntimeActivity::Simulation),
                use_continuous_update_mode,
            );
    }
}

/// The update-loop behavior required by the current app state.
///
/// New screens are low-power by default. Only states that actively simulate
/// gameplay should be listed as `Simulation` here.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum RuntimeActivity {
    Interface,
    Simulation,
}

impl ComputedStates for RuntimeActivity {
    type SourceStates = AppState;

    fn compute(state: AppState) -> Option<Self> {
        Some(match state {
            AppState::Battle => Self::Simulation,
            _ => Self::Interface,
        })
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
        Hdr,
        Tonemapping::Reinhard,
        ColorGrading::default(),
        BattleCamera,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: 720.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
