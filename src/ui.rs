use crate::model::*;
use bevy::{app::AppExit, prelude::*};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

const INK: Color = Color::srgb(0.09, 0.065, 0.045);
const PARCHMENT: Color = Color::srgb(0.88, 0.78, 0.58);
const GOLD: Color = Color::srgb(0.83, 0.62, 0.19);
const GOLD_BRIGHT: Color = Color::srgb(1.0, 0.82, 0.34);
const WOOD: Color = Color::srgba(0.105, 0.065, 0.04, 0.94);
const WOOD_HOVER: Color = Color::srgba(0.19, 0.11, 0.06, 0.98);
const BLUE: Color = Color::srgb(0.12, 0.28, 0.63);
const RED: Color = Color::srgb(0.62, 0.12, 0.1);

#[derive(Resource)]
struct UiAssets {
    font: Handle<Font>,
    battlefield: Handle<Image>,
}

impl FromWorld for UiAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            font: asset_server.load("fonts/Cinzel.ttf"),
            battlefield: asset_server.load("environment/battlefield.png"),
        }
    }
}

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiAssets>()
            .add_systems(OnEnter(AppState::MainMenu), setup_menu)
            .add_systems(OnExit(AppState::MainMenu), cleanup::<MainMenuEntity>)
            .add_systems(
                Update,
                (menu_buttons, update_button_visuals).run_if(in_state(AppState::MainMenu)),
            )
            .add_systems(OnEnter(AppState::Battle), setup_hud)
            .add_systems(
                Update,
                (battle_buttons, update_button_visuals, update_hud)
                    .run_if(in_state(AppState::Battle)),
            )
            .add_systems(OnEnter(AppState::Results), setup_results)
            .add_systems(OnExit(AppState::Results), cleanup::<ResultsEntity>)
            .add_systems(
                Update,
                (results_buttons, update_button_visuals).run_if(in_state(AppState::Results)),
            );
        #[cfg(target_arch = "wasm32")]
        app.add_systems(
            Update,
            signal_web_menu_ready.run_if(in_state(AppState::MainMenu)),
        );
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = stickWarReady)]
    fn stick_war_ready();
}

#[cfg(target_arch = "wasm32")]
fn signal_web_menu_ready(
    asset_server: Res<AssetServer>,
    assets: Res<UiAssets>,
    mut signaled: Local<bool>,
) {
    if !*signaled
        && asset_server.is_loaded_with_dependencies(&assets.font)
        && asset_server.is_loaded_with_dependencies(&assets.battlefield)
    {
        stick_war_ready();
        *signaled = true;
    }
}

#[derive(Component)]
enum MenuButton {
    Start,
    Quit,
}
#[derive(Component)]
enum BattleButton {
    TrainMiner,
    TrainSwordsman,
    TrainArcher,
    Attack,
    Defend,
    Retreat,
}
#[derive(Component)]
enum ResultsButton {
    Restart,
    MainMenu,
}
#[derive(Component)]
struct GoldText;
#[derive(Component)]
struct PopulationText;
#[derive(Component)]
struct StatueText;
#[derive(Component)]
struct OrderText;

#[derive(Component, Clone, Copy)]
enum ButtonTheme {
    Blue,
    Red,
    Wood,
}

fn root_node() -> Node {
    Node {
        width: percent(100),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        row_gap: px(18),
        ..default()
    }
}
fn text_style(assets: &UiAssets, size: f32, weight: FontWeight) -> TextFont {
    TextFont::from(assets.font.clone())
        .with_font_size(size)
        .with_font_weight(weight)
}
fn button_node(min_width: f32) -> Node {
    Node {
        padding: UiRect::axes(px(15), px(9)),
        margin: UiRect::all(px(3)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        min_width: px(min_width),
        border: UiRect::all(px(2)),
        border_radius: BorderRadius::all(px(3)),
        ..default()
    }
}

fn panel_node() -> Node {
    Node {
        padding: UiRect::axes(px(20), px(14)),
        border: UiRect::all(px(2)),
        border_radius: BorderRadius::all(px(4)),
        ..default()
    }
}

fn themed_button(theme: ButtonTheme, min_width: f32, label: &str, font: TextFont) -> impl Bundle {
    let color = match theme {
        ButtonTheme::Blue => BLUE,
        ButtonTheme::Red => RED,
        ButtonTheme::Wood => WOOD,
    };
    (
        Button,
        theme,
        button_node(min_width),
        BackgroundColor(color),
        BorderColor::all(GOLD),
        children![(Text::new(label), font, TextColor(PARCHMENT))],
    )
}

fn setup_menu(mut commands: Commands, assets: Res<UiAssets>) {
    let title = text_style(&assets, 76.0, FontWeight::BLACK);
    let subtitle = text_style(&assets, 19.0, FontWeight::MEDIUM);
    let button = text_style(&assets, 22.0, FontWeight::BOLD);
    commands
        .spawn((MainMenuEntity, root_node(), BackgroundColor(INK)))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                ImageNode {
                    image: assets.battlefield.clone(),
                    image_mode: NodeImageMode::Stretch,
                    color: Color::srgb(0.42, 0.35, 0.28),
                    ..default()
                },
            ));
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.035, 0.022, 0.016, 0.48)),
            ));
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(15),
                    padding: UiRect::axes(px(46), px(30)),
                    border: UiRect::all(px(3)),
                    border_radius: BorderRadius::all(px(5)),
                    max_width: px(660),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.075, 0.043, 0.026, 0.92)),
                BorderColor::all(GOLD),
            ))
            .with_children(|panel| {
                panel.spawn((Text::new("STICK WAR"), title, TextColor(GOLD_BRIGHT)));
                panel.spawn((
                    Text::new("A hand-drawn war of weight and steel"),
                    subtitle,
                    TextColor(PARCHMENT),
                    TextLayout::justify(Justify::Center),
                ));
                panel.spawn((
                    MenuButton::Start,
                    themed_button(ButtonTheme::Blue, 250.0, "BEGIN BATTLE", button.clone()),
                ));
                panel.spawn((
                    MenuButton::Quit,
                    themed_button(ButtonTheme::Wood, 250.0, "LEAVE THE FIELD", button),
                ));
            });
        });
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
            MenuButton::Start => next.set(AppState::Battle),
            MenuButton::Quit => {
                exit.write(AppExit::Success);
            }
        }
    }
}

fn hud_button(
    parent: &mut ChildSpawnerCommands,
    assets: &UiAssets,
    label: &str,
    kind: BattleButton,
    theme: ButtonTheme,
) {
    parent.spawn((
        kind,
        themed_button(
            theme,
            138.0,
            label,
            text_style(assets, 15.0, FontWeight::BOLD),
        ),
    ));
}

fn setup_hud(mut commands: Commands, assets: Res<UiAssets>) {
    commands
        .spawn((
            BattleEntity,
            Node {
                position_type: PositionType::Absolute,
                top: px(0),
                left: px(0),
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::axes(px(8), px(8)),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: percent(92),
                    max_width: px(1050),
                    justify_content: JustifyContent::SpaceAround,
                    align_items: AlignItems::Center,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(24),
                    row_gap: px(4),
                    ..panel_node()
                },
                BackgroundColor(WOOD),
                BorderColor::all(GOLD),
            ))
            .with_children(|panel| {
                let status = text_style(&assets, 17.0, FontWeight::BOLD);
                panel.spawn((
                    GoldText,
                    Text::new("GOLD"),
                    status.clone(),
                    TextColor(GOLD_BRIGHT),
                ));
                panel.spawn((
                    PopulationText,
                    Text::new("POPULATION"),
                    status.clone(),
                    TextColor(PARCHMENT),
                ));
                panel.spawn((
                    StatueText,
                    Text::new("MONUMENTS"),
                    status.clone(),
                    TextColor(PARCHMENT),
                ));
                panel.spawn((
                    OrderText,
                    Text::new("ORDER"),
                    status,
                    TextColor(Color::srgb(0.46, 0.67, 1.0)),
                ));
            });

            root.spawn((
                Node {
                    width: percent(96),
                    max_width: px(1120),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(3),
                    padding: UiRect::all(px(7)),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.035, 0.022, 0.9)),
                BorderColor::all(GOLD),
            ))
            .with_children(|command_bar| {
                command_bar
                    .spawn((Node {
                        width: percent(100),
                        justify_content: JustifyContent::Center,
                        flex_wrap: FlexWrap::Wrap,
                        ..default()
                    },))
                    .with_children(|row| {
                        hud_button(
                            row,
                            &assets,
                            "MINER  50  [M]",
                            BattleButton::TrainMiner,
                            ButtonTheme::Blue,
                        );
                        hud_button(
                            row,
                            &assets,
                            "SWORD  100  [S]",
                            BattleButton::TrainSwordsman,
                            ButtonTheme::Blue,
                        );
                        hud_button(
                            row,
                            &assets,
                            "ARCHER  125  [R]",
                            BattleButton::TrainArcher,
                            ButtonTheme::Blue,
                        );
                        hud_button(
                            row,
                            &assets,
                            "ATTACK  [1]",
                            BattleButton::Attack,
                            ButtonTheme::Red,
                        );
                        hud_button(
                            row,
                            &assets,
                            "DEFEND  [2]",
                            BattleButton::Defend,
                            ButtonTheme::Wood,
                        );
                        hud_button(
                            row,
                            &assets,
                            "RETREAT  [3]",
                            BattleButton::Retreat,
                            ButtonTheme::Wood,
                        );
                    });
                command_bar.spawn((
                    Text::new("TAB: COMMAND WARRIOR   A/D: MOVE   SPACE: STRIKE   ESC: RELEASE"),
                    text_style(&assets, 12.0, FontWeight::SEMIBOLD),
                    TextColor(Color::srgb(0.72, 0.66, 0.54)),
                    TextLayout::justify(Justify::Center),
                    Node {
                        padding: UiRect::horizontal(px(8)),
                        max_width: percent(100),
                        ..default()
                    },
                ));
            });
        });
}

fn battle_buttons(
    interactions: Query<(&Interaction, &BattleButton), (Changed<Interaction>, With<Button>)>,
    mut train: MessageWriter<TrainUnitRequest>,
    mut order: ResMut<PlayerArmyOrder>,
) {
    for (interaction, button) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match button {
            BattleButton::TrainMiner => {
                train.write(TrainUnitRequest {
                    team: Team::Player,
                    kind: UnitKind::Miner,
                });
            }
            BattleButton::TrainSwordsman => {
                train.write(TrainUnitRequest {
                    team: Team::Player,
                    kind: UnitKind::Swordsman,
                });
            }
            BattleButton::TrainArcher => {
                train.write(TrainUnitRequest {
                    team: Team::Player,
                    kind: UnitKind::Archer,
                });
            }
            BattleButton::Attack => order.0 = ArmyOrder::Attack,
            BattleButton::Defend => order.0 = ArmyOrder::Defend,
            BattleButton::Retreat => order.0 = ArmyOrder::Retreat,
        }
    }
}

fn update_hud(
    economy: Res<Economy>,
    order: Res<PlayerArmyOrder>,
    statues: Query<(&Team, &Health), With<Statue>>,
    mut gold: Single<
        &mut Text,
        (
            With<GoldText>,
            Without<PopulationText>,
            Without<StatueText>,
            Without<OrderText>,
        ),
    >,
    mut pop: Single<
        &mut Text,
        (
            With<PopulationText>,
            Without<GoldText>,
            Without<StatueText>,
            Without<OrderText>,
        ),
    >,
    mut statue_text: Single<
        &mut Text,
        (
            With<StatueText>,
            Without<GoldText>,
            Without<PopulationText>,
            Without<OrderText>,
        ),
    >,
    mut order_text: Single<
        &mut Text,
        (
            With<OrderText>,
            Without<GoldText>,
            Without<PopulationText>,
            Without<StatueText>,
        ),
    >,
) {
    gold.0 = format!("GOLD  {}", economy.player_gold);
    pop.0 = format!(
        "POP  {}/{}",
        economy.player_population, economy.population_limit
    );
    order_text.0 = format!("ORDER  {:?}", order.0);
    let mut player = 0.0;
    let mut enemy = 0.0;
    for (team, h) in &statues {
        if *team == Team::Player {
            player = h.current
        } else {
            enemy = h.current
        }
    }
    statue_text.0 = format!("STATUES  {:.0}  /  {:.0}", player, enemy);
}

fn setup_results(mut commands: Commands, result: Res<BattleResult>, assets: Res<UiAssets>) {
    let (title, color) = match *result {
        BattleResult::Victory => ("VICTORY", GOLD_BRIGHT),
        BattleResult::Defeat => ("DEFEAT", Color::srgb(0.9, 0.24, 0.18)),
    };
    let button = text_style(&assets, 21.0, FontWeight::BOLD);
    commands
        .spawn((ResultsEntity, root_node(), BackgroundColor(INK)))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                ImageNode {
                    image: assets.battlefield.clone(),
                    image_mode: NodeImageMode::Stretch,
                    color: Color::srgb(0.28, 0.22, 0.18),
                    ..default()
                },
            ));
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(16),
                    padding: UiRect::axes(px(50), px(34)),
                    border: UiRect::all(px(3)),
                    border_radius: BorderRadius::all(px(5)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.035, 0.022, 0.94)),
                BorderColor::all(GOLD),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(title),
                    text_style(&assets, 76.0, FontWeight::BLACK),
                    TextColor(color),
                ));
                panel.spawn((
                    ResultsButton::Restart,
                    themed_button(ButtonTheme::Blue, 240.0, "FIGHT AGAIN", button.clone()),
                ));
                panel.spawn((
                    ResultsButton::MainMenu,
                    themed_button(ButtonTheme::Wood, 240.0, "RETURN TO HALL", button),
                ));
            });
        });
}

fn update_button_visuals(
    mut buttons: Query<
        (
            &Interaction,
            &ButtonTheme,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, theme, mut background, mut border) in &mut buttons {
        let base = match theme {
            ButtonTheme::Blue => BLUE,
            ButtonTheme::Red => RED,
            ButtonTheme::Wood => WOOD,
        };
        match interaction {
            Interaction::Pressed => {
                background.0 = INK;
                *border = BorderColor::all(GOLD_BRIGHT);
            }
            Interaction::Hovered => {
                background.0 = match theme {
                    ButtonTheme::Blue => Color::srgb(0.18, 0.4, 0.82),
                    ButtonTheme::Red => Color::srgb(0.8, 0.18, 0.13),
                    ButtonTheme::Wood => WOOD_HOVER,
                };
                *border = BorderColor::all(GOLD_BRIGHT);
            }
            Interaction::None => {
                background.0 = base;
                *border = BorderColor::all(GOLD);
            }
        }
    }
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
