use crate::{battle::BattlePlugin, model::*, ui::UiPlugin};
use bevy::{prelude::*, winit::WinitSettings};

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_computed_state::<GameplayState>()
            .add_computed_state::<RuntimeActivity>()
            .init_asset::<GameConfig>()
            .init_asset_loader::<GameConfigLoader>()
            // UI-only states do not need a continuously running game loop. Start
            // reactively so the title screen is low-power from the first frame.
            .insert_resource(WinitSettings::desktop_app())
            .insert_resource(load_game_config())
            .add_message::<TrainUnitRequest>()
            .add_message::<DamageMessage>()
            .add_plugins((BattlePlugin, UiPlugin))
            .add_systems(Startup, (setup_camera, load_config_asset))
            .add_systems(Update, apply_loaded_config)
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

#[derive(Resource)]
struct GameConfigHandle(Handle<GameConfig>);

fn load_config_asset(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameConfigHandle(asset_server.load("game_config.ron")));
}

fn apply_loaded_config(
    handle: Res<GameConfigHandle>,
    configs: Res<Assets<GameConfig>>,
    mut events: MessageReader<AssetEvent<GameConfig>>,
    mut config: ResMut<GameConfig>,
) {
    if events
        .read()
        .any(|event| event.is_loaded_with_dependencies(handle.0.id()))
        && let Some(loaded) = configs.get(&handle.0)
    {
        *config = loaded.clone();
        info!("Reloaded game config");
    }
}

/// The update-loop behavior required by the current gameplay lifecycle.
///
/// New screens are low-power by default. Only states that actively simulate
/// gameplay should be listed as `Simulation` here.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum RuntimeActivity {
    Interface,
    Simulation,
}

impl ComputedStates for RuntimeActivity {
    type SourceStates = GameplayState;

    fn compute(state: GameplayState) -> Option<Self> {
        Some(match state {
            GameplayState::Active => Self::Simulation,
            GameplayState::Interface => Self::Interface,
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
        BattleCamera,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: 720.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
