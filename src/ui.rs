use crate::model::*;
use bevy::{app::AppExit, ecs::system::SystemParam, prelude::*};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), setup_menu)
            .add_systems(OnExit(AppState::MainMenu), cleanup::<MainMenuEntity>)
            .add_systems(Update, menu_buttons.run_if(in_state(AppState::MainMenu)))
            .add_systems(OnEnter(AppState::Battle), setup_hud)
            .add_systems(OnEnter(AppState::Sandbox), setup_hud)
            .add_systems(Update, (battle_buttons, update_hud).run_if(in_gameplay))
            .add_systems(OnEnter(AppState::Results), setup_results)
            .add_systems(OnExit(AppState::Results), cleanup::<ResultsEntity>)
            .add_systems(Update, results_buttons.run_if(in_state(AppState::Results)));
    }
}

#[derive(Component)]
enum MenuButton {
    Battle,
    Sandbox,
    Quit,
}
#[derive(Component, Clone, Copy)]
enum BattleButton {
    Train(Team, UnitKind),
    Order(Team, ArmyOrder),
    TogglePause,
    ToggleCosts,
    ToggleTrainingTime,
    PopulationDelta(i32),
    MainMenu,
}
#[derive(Component)]
enum ResultsButton {
    Restart,
    MainMenu,
}
#[derive(Component)]
struct PlayerStatusText;
#[derive(Component)]
struct EnemyStatusText;
#[derive(Component)]
struct ClockText;
#[derive(Component)]
struct SandboxStatusText;

#[derive(SystemParam)]
struct HudData<'w> {
    economy: Res<'w, Economy>,
    orders: Res<'w, ArmyOrders>,
    settings: Res<'w, SandboxSettings>,
    clock: Res<'w, BattleClock>,
    training: Res<'w, TrainingQueue>,
}

fn in_gameplay(state: Res<State<AppState>>) -> bool {
    matches!(state.get(), AppState::Battle | AppState::Sandbox)
}

fn root_node() -> Node {
    Node {
        width: percent(100),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        row_gap: px(22),
        ..default()
    }
}
fn text_style(size: f32) -> TextFont {
    TextFont {
        font_size: FontSize::Px(size),
        ..default()
    }
}
fn button_node() -> Node {
    Node {
        padding: UiRect::axes(px(14), px(8)),
        margin: UiRect::all(px(3)),
        justify_content: JustifyContent::Center,
        min_width: px(132),
        ..default()
    }
}

fn hud_button_node() -> Node {
    Node {
        padding: UiRect::axes(px(7), px(4)),
        margin: UiRect::all(px(2)),
        justify_content: JustifyContent::Center,
        min_width: px(72),
        ..default()
    }
}

fn setup_menu(mut commands: Commands) {
    commands.spawn((
        MainMenuEntity,
        root_node(),
        BackgroundColor(Color::srgb(0.05, 0.07, 0.12)),
        children![
            (
                Text::new("STICK WAR"),
                text_style(72.0),
                TextColor(Color::srgb(0.95, 0.75, 0.15))
            ),
            (
                Text::new("A minimal side-view strategy battle"),
                text_style(24.0),
                TextColor(Color::WHITE)
            ),
            (
                Button,
                MenuButton::Battle,
                button_node(),
                BackgroundColor(Color::srgb(0.12, 0.35, 0.75)),
                children![(
                    Text::new("START BATTLE"),
                    text_style(24.0),
                    TextColor(Color::WHITE)
                )]
            ),
            (
                Button,
                MenuButton::Sandbox,
                button_node(),
                BackgroundColor(Color::srgb(0.16, 0.48, 0.38)),
                children![(
                    Text::new("SANDBOX"),
                    text_style(22.0),
                    TextColor(Color::WHITE)
                )]
            ),
            (
                Button,
                MenuButton::Quit,
                button_node(),
                BackgroundColor(Color::srgb(0.3, 0.15, 0.15)),
                children![(Text::new("QUIT"), text_style(20.0), TextColor(Color::WHITE))]
            )
        ],
    ));
}

fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn menu_buttons(
    interactions: Query<(&Interaction, &MenuButton), (Changed<Interaction>, With<Button>)>,
    mut next: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, button) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match button {
            MenuButton::Battle => next.set(AppState::Battle),
            MenuButton::Sandbox => next.set(AppState::Sandbox),
            MenuButton::Quit => {
                exit.write(AppExit::Success);
            }
        }
    }
}

fn hud_button(parent: &mut ChildSpawnerCommands, label: &str, kind: BattleButton) {
    parent.spawn((
        Button,
        kind,
        hud_button_node(),
        BackgroundColor(Color::srgba(0.08, 0.1, 0.16, 0.92)),
        children![(Text::new(label), text_style(13.0), TextColor(Color::WHITE))],
    ));
}

fn setup_hud(mut commands: Commands, state: Res<State<AppState>>, c: Res<GameConfig>) {
    let sandbox = *state.get() == AppState::Sandbox;
    commands
        .spawn((
            BattleEntity,
            Node {
                position_type: PositionType::Absolute,
                top: px(0),
                left: px(0),
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: percent(100),
                    padding: UiRect::axes(px(8), px(5)),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    column_gap: px(10),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.03, 0.04, 0.07, 0.92)),
            ))
            .with_children(|top| {
                top.spawn((
                    PlayerStatusText,
                    Text::new("PLAYER"),
                    text_style(14.0),
                    TextColor(Color::srgb(0.45, 0.8, 1.0)),
                ));
                top.spawn((
                    ClockText,
                    Text::new("00:00"),
                    text_style(16.0),
                    TextColor(Color::WHITE),
                ));
                top.spawn((
                    EnemyStatusText,
                    Text::new("ENEMY"),
                    text_style(14.0),
                    TextColor(Color::srgb(1.0, 0.48, 0.42)),
                ));
            });

            root.spawn((
                Node {
                    width: percent(100),
                    padding: UiRect::axes(px(5), px(4)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.03, 0.04, 0.07, 0.9)),
            ))
            .with_children(|panel| {
                if sandbox {
                    panel
                        .spawn((
                            Node {
                                justify_content: JustifyContent::Center,
                                flex_wrap: FlexWrap::Wrap,
                                ..default()
                            },
                        ))
                        .with_children(|settings| {
                            hud_button(settings, "PAUSE", BattleButton::TogglePause);
                            hud_button(settings, "COSTS", BattleButton::ToggleCosts);
                            hud_button(
                                settings,
                                "TRAIN TIME",
                                BattleButton::ToggleTrainingTime,
                            );
                            hud_button(settings, "POP -5", BattleButton::PopulationDelta(-5));
                            hud_button(settings, "POP -1", BattleButton::PopulationDelta(-1));
                            hud_button(settings, "POP +1", BattleButton::PopulationDelta(1));
                            hud_button(settings, "POP +5", BattleButton::PopulationDelta(5));
                            hud_button(settings, "MENU", BattleButton::MainMenu);
                        });
                    panel.spawn((
                        SandboxStatusText,
                        Text::new("SANDBOX"),
                        text_style(12.0),
                        TextColor(Color::srgb(0.55, 1.0, 0.72)),
                    ));
                } else {
                    panel.spawn((SandboxStatusText, Text::new(""), text_style(1.0)));
                }

                spawn_team_controls(panel, Team::Player, &c, true);
                if sandbox {
                    spawn_team_controls(panel, Team::Enemy, &c, false);
                }
                panel.spawn((
                    Text::new(
                        "M/S/R train | 1/2/3 orders | Tab control | A/D or arrows move | Space attack | Esc release",
                    ),
                    text_style(11.0),
                    TextColor(Color::srgb(0.78, 0.8, 0.86)),
                    TextLayout::justify(Justify::Center),
                    Node {
                        margin: UiRect::top(px(2)),
                        max_width: percent(100),
                        ..default()
                    },
                ));
            });
        });
}

fn spawn_team_controls(
    parent: &mut ChildSpawnerCommands,
    team: Team,
    c: &GameConfig,
    show_shortcuts: bool,
) {
    parent
        .spawn((Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        },))
        .with_children(|row| {
            row.spawn((
                Text::new(if team == Team::Player { "YOU" } else { "ENEMY" }),
                text_style(12.0),
                TextColor(if team == Team::Player {
                    Color::srgb(0.45, 0.8, 1.0)
                } else {
                    Color::srgb(1.0, 0.48, 0.42)
                }),
                Node {
                    width: px(48),
                    ..default()
                },
            ));
            let shortcut = |key| if show_shortcuts { key } else { "" };
            hud_button(
                row,
                &format!("MINER {}{}", c.units.miner.cost, shortcut(" [M]")),
                BattleButton::Train(team, UnitKind::Miner),
            );
            hud_button(
                row,
                &format!("SWORD {}{}", c.units.swordsman.cost, shortcut(" [S]")),
                BattleButton::Train(team, UnitKind::Swordsman),
            );
            hud_button(
                row,
                &format!("ARCHER {}{}", c.units.archer.cost, shortcut(" [R]")),
                BattleButton::Train(team, UnitKind::Archer),
            );
            hud_button(
                row,
                &format!("ATTACK{}", shortcut(" [1]")),
                BattleButton::Order(team, ArmyOrder::Attack),
            );
            hud_button(
                row,
                &format!("DEFEND{}", shortcut(" [2]")),
                BattleButton::Order(team, ArmyOrder::Defend),
            );
            hud_button(
                row,
                &format!("RETREAT{}", shortcut(" [3]")),
                BattleButton::Order(team, ArmyOrder::Retreat),
            );
        });
}

fn battle_buttons(
    interactions: Query<(&Interaction, &BattleButton), (Changed<Interaction>, With<Button>)>,
    mut train: MessageWriter<TrainUnitRequest>,
    mut orders: ResMut<ArmyOrders>,
    mut settings: ResMut<SandboxSettings>,
    mut economy: ResMut<Economy>,
    mut next: ResMut<NextState<AppState>>,
) {
    for (interaction, button) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match button {
            BattleButton::Train(team, kind) => {
                train.write(TrainUnitRequest {
                    team: *team,
                    kind: *kind,
                });
            }
            BattleButton::Order(team, order) => orders.set(*team, *order),
            BattleButton::TogglePause if settings.is_sandbox => {
                settings.paused = !settings.paused;
            }
            BattleButton::ToggleCosts if settings.is_sandbox => {
                settings.charge_costs = !settings.charge_costs;
            }
            BattleButton::ToggleTrainingTime if settings.is_sandbox => {
                settings.training_time_enabled = !settings.training_time_enabled;
            }
            BattleButton::PopulationDelta(delta) if settings.is_sandbox => {
                economy.population_limit = if *delta < 0 {
                    economy
                        .population_limit
                        .saturating_sub(delta.unsigned_abs())
                } else {
                    economy.population_limit.saturating_add(*delta as u32)
                }
                .max(1);
            }
            BattleButton::MainMenu => next.set(AppState::MainMenu),
            _ => {}
        }
    }
}

fn update_hud(
    data: HudData,
    statues: Query<(&Team, &Health), With<Statue>>,
    mut player_text: Single<
        &mut Text,
        (
            With<PlayerStatusText>,
            Without<EnemyStatusText>,
            Without<ClockText>,
            Without<SandboxStatusText>,
        ),
    >,
    mut enemy_text: Single<
        &mut Text,
        (
            With<EnemyStatusText>,
            Without<PlayerStatusText>,
            Without<ClockText>,
            Without<SandboxStatusText>,
        ),
    >,
    mut clock_text: Single<
        &mut Text,
        (
            With<ClockText>,
            Without<PlayerStatusText>,
            Without<EnemyStatusText>,
            Without<SandboxStatusText>,
        ),
    >,
    mut sandbox_text: Single<
        &mut Text,
        (
            With<SandboxStatusText>,
            Without<PlayerStatusText>,
            Without<EnemyStatusText>,
            Without<ClockText>,
        ),
    >,
) {
    let HudData {
        economy,
        orders,
        settings,
        clock,
        training,
    } = data;
    let mut player = 0.0;
    let mut enemy = 0.0;
    for (team, h) in &statues {
        if *team == Team::Player {
            player = h.current
        } else {
            enemy = h.current
        }
    }
    player_text.0 = format!(
        "YOU  G {} | P {}/{} | HP {:.0} | {:?}{}",
        economy.player_gold,
        economy.player_population,
        economy.population_limit,
        player,
        orders.player,
        training_summary(&training, Team::Player),
    );
    enemy_text.0 = format!(
        "ENEMY  G {} | P {}/{} | HP {:.0} | {:?}{}",
        economy.enemy_gold,
        economy.enemy_population,
        economy.population_limit,
        enemy,
        orders.enemy,
        training_summary(&training, Team::Enemy),
    );
    let total_seconds = clock.elapsed_seconds.floor() as u64;
    clock_text.0 = format!("{:02}:{:02}", total_seconds / 60, total_seconds % 60);
    sandbox_text.0 = if settings.is_sandbox {
        format!(
            "{}  |  COSTS [{}]  |  TRAIN TIME [{}]  |  POP LIMIT {}",
            if settings.paused { "PAUSED" } else { "RUNNING" },
            if settings.charge_costs { "x" } else { " " },
            if settings.training_time_enabled {
                "x"
            } else {
                " "
            },
            economy.population_limit,
        )
    } else {
        String::new()
    };
}

fn training_summary(queue: &TrainingQueue, team: Team) -> String {
    let entries = [
        (UnitKind::Miner, "M"),
        (UnitKind::Swordsman, "S"),
        (UnitKind::Archer, "A"),
    ]
    .into_iter()
    .filter_map(|(kind, label)| {
        queue
            .remaining(team, kind)
            .map(|seconds| format!("{label} {:.1}s", seconds))
    })
    .collect::<Vec<_>>();
    if entries.is_empty() {
        String::new()
    } else {
        format!(" | TRAIN {}", entries.join(" "))
    }
}

fn setup_results(mut commands: Commands, result: Res<BattleResult>) {
    let (title, color) = match *result {
        BattleResult::Victory => ("VICTORY", Color::srgb(0.25, 0.9, 0.35)),
        BattleResult::Defeat => ("DEFEAT", Color::srgb(0.9, 0.2, 0.2)),
    };
    commands.spawn((
        ResultsEntity,
        root_node(),
        BackgroundColor(Color::srgb(0.04, 0.05, 0.09)),
        children![
            (Text::new(title), text_style(76.0), TextColor(color)),
            (
                Button,
                ResultsButton::Restart,
                button_node(),
                BackgroundColor(Color::srgb(0.12, 0.35, 0.75)),
                children![(
                    Text::new("RESTART"),
                    text_style(22.0),
                    TextColor(Color::WHITE)
                )]
            ),
            (
                Button,
                ResultsButton::MainMenu,
                button_node(),
                BackgroundColor(Color::srgb(0.2, 0.22, 0.28)),
                children![(
                    Text::new("MAIN MENU"),
                    text_style(22.0),
                    TextColor(Color::WHITE)
                )]
            )
        ],
    ));
}

fn results_buttons(
    interactions: Query<(&Interaction, &ResultsButton), (Changed<Interaction>, With<Button>)>,
    mut next: ResMut<NextState<AppState>>,
) {
    for (interaction, button) in &interactions {
        if *interaction == Interaction::Pressed {
            next.set(match button {
                ResultsButton::Restart => AppState::Battle,
                ResultsButton::MainMenu => AppState::MainMenu,
            });
        }
    }
}
