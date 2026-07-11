use crate::model::*;
use bevy::{app::AppExit, prelude::*};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), setup_menu)
            .add_systems(OnExit(AppState::MainMenu), cleanup::<MainMenuEntity>)
            .add_systems(Update, menu_buttons.run_if(in_state(AppState::MainMenu)))
            .add_systems(OnEnter(AppState::Battle), setup_hud)
            .add_systems(
                Update,
                (battle_buttons, update_hud).run_if(in_state(AppState::Battle)),
            )
            .add_systems(OnEnter(AppState::Results), setup_results)
            .add_systems(OnExit(AppState::Results), cleanup::<ResultsEntity>)
            .add_systems(Update, results_buttons.run_if(in_state(AppState::Results)));
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
        padding: UiRect::axes(px(22), px(10)),
        margin: UiRect::all(px(5)),
        justify_content: JustifyContent::Center,
        min_width: px(170),
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
                MenuButton::Start,
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
            MenuButton::Start => next.set(AppState::Battle),
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
        button_node(),
        BackgroundColor(Color::srgba(0.08, 0.1, 0.16, 0.92)),
        children![(Text::new(label), text_style(18.0), TextColor(Color::WHITE))],
    ));
}

fn setup_hud(mut commands: Commands) {
    commands.spawn((BattleEntity,Node{position_type:PositionType::Absolute,top:px(0),left:px(0),width:percent(100),height:percent(100),flex_direction:FlexDirection::Column,justify_content:JustifyContent::SpaceBetween,..default()},children![
        (Node{width:percent(100),padding:UiRect::all(px(12)),justify_content:JustifyContent::SpaceBetween,..default()},BackgroundColor(Color::srgba(0.03,0.04,0.07,0.9)),children![
            (GoldText,Text::new("Gold"),text_style(20.0),TextColor(Color::srgb(1.0,0.82,0.2))),
            (PopulationText,Text::new("Population"),text_style(20.0),TextColor(Color::WHITE)),
            (StatueText,Text::new("Statues"),text_style(20.0),TextColor(Color::WHITE)),
            (OrderText,Text::new("Order"),text_style(20.0),TextColor(Color::srgb(0.4,0.8,1.0)))
        ]),
        (Node{width:percent(100),padding:UiRect::all(px(8)),flex_direction:FlexDirection::Column,align_items:AlignItems::Center,..default()},children![
            (Node{justify_content:JustifyContent::Center,flex_wrap:FlexWrap::Wrap,..default()},children![]),
            (Text::new("M/S/R train  |  1 Attack  2 Defend  3 Retreat  |  Tab control  A/D move  Space attack  Esc release"),text_style(15.0),TextColor(Color::WHITE),Node{margin:UiRect::top(px(5)),..default()})
        ])
    ])).with_children(|root| {
        let mut row=root.spawn((Node{position_type:PositionType::Absolute,bottom:px(42),left:percent(0),width:percent(100),justify_content:JustifyContent::Center,flex_wrap:FlexWrap::Wrap,..default()},));
        row.with_children(|p|{hud_button(p,"MINER 50 [M]",BattleButton::TrainMiner);hud_button(p,"SWORD 100 [S]",BattleButton::TrainSwordsman);hud_button(p,"ARCHER 125 [R]",BattleButton::TrainArcher);hud_button(p,"ATTACK [1]",BattleButton::Attack);hud_button(p,"DEFEND [2]",BattleButton::Defend);hud_button(p,"RETREAT [3]",BattleButton::Retreat);});
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
