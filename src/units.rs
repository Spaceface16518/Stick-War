use crate::{
    model::*,
    rendering::{add_health_bar, team_color},
};
use bevy::prelude::*;

fn piece(
    parent: &mut ChildSpawnerCommands,
    color: Color,
    size: Vec2,
    position: Vec3,
    rotation: f32,
) {
    parent.spawn((
        Sprite::from_color(color, size),
        Transform::from_translation(position).with_rotation(Quat::from_rotation_z(rotation)),
    ));
}

fn limb(
    parent: &mut ChildSpawnerCommands,
    color: Color,
    position: Vec3,
    angle: f32,
    kind: LimbKind,
) {
    parent.spawn((
        Limb {
            kind,
            rest_angle: angle,
        },
        Sprite::from_color(color, Vec2::new(7.0, 40.0)),
        Transform::from_translation(position).with_rotation(Quat::from_rotation_z(angle)),
    ));
}

fn stick_figure(commands: &mut Commands, root: Entity, team: Team, sword: bool) {
    let color = team_color(team);
    commands.entity(root).with_children(|p| {
        piece(
            p,
            color,
            Vec2::new(22.0, 22.0),
            Vec3::new(0.0, 58.0, 1.0),
            0.0,
        );
        piece(
            p,
            color,
            Vec2::new(8.0, 48.0),
            Vec3::new(0.0, 25.0, 1.0),
            0.0,
        );
        limb(
            p,
            color,
            Vec3::new(-11.0, 26.0, 1.0),
            0.45,
            LimbKind::LeftArm,
        );
        limb(
            p,
            color,
            Vec3::new(11.0, 26.0, 1.0),
            -0.45,
            LimbKind::RightArm,
        );
        limb(
            p,
            color,
            Vec3::new(-10.0, -14.0, 1.0),
            -0.35,
            LimbKind::LeftLeg,
        );
        limb(
            p,
            color,
            Vec3::new(10.0, -14.0, 1.0),
            0.35,
            LimbKind::RightLeg,
        );
        if sword {
            piece(
                p,
                Color::srgb(0.85, 0.85, 0.9),
                Vec2::new(6.0, 55.0),
                Vec3::new(27.0, 35.0, 2.0),
                -0.35,
            );
        }
        p.spawn((
            SelectionMarker,
            Visibility::Hidden,
            Sprite::from_color(Color::srgb(1.0, 0.9, 0.15), Vec2::new(25.0, 12.0)),
            Transform::from_xyz(0.0, 88.0, 2.0)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 4.0)),
        ));
    });
    add_health_bar(commands, root, 55.0, 80.0);
}

pub fn spawn_statue(
    commands: &mut Commands,
    _meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<ColorMaterial>,
    team: Team,
    position: Vec2,
    max_health: f32,
) -> Entity {
    let root = commands
        .spawn((
            BattleEntity,
            Statue,
            team,
            Health {
                current: max_health,
                maximum: max_health,
            },
            Transform::from_xyz(position.x, position.y, 2.0),
            Visibility::default(),
        ))
        .id();
    let color = team_color(team);
    commands.entity(root).with_children(|p| {
        piece(
            p,
            Color::srgb(0.22, 0.22, 0.24),
            Vec2::new(130.0, 25.0),
            Vec3::new(0.0, -5.0, 1.0),
            0.0,
        );
        piece(
            p,
            color,
            Vec2::new(65.0, 140.0),
            Vec3::new(0.0, 75.0, 1.0),
            0.0,
        );
        piece(
            p,
            Color::srgb(0.85, 0.75, 0.38),
            Vec2::new(50.0, 50.0),
            Vec3::new(0.0, 165.0, 1.0),
            0.0,
        );
    });
    add_health_bar(commands, root, 130.0, 210.0);
    root
}

pub fn spawn_gold_deposit(
    commands: &mut Commands,
    _meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<ColorMaterial>,
    team: Team,
    position: Vec2,
) -> Entity {
    let root = commands
        .spawn((
            BattleEntity,
            GoldDeposit,
            team,
            Transform::from_xyz(position.x, position.y, 1.0),
            Visibility::default(),
        ))
        .id();
    commands.entity(root).with_children(|p| {
        piece(
            p,
            Color::srgb(0.95, 0.7, 0.05),
            Vec2::new(85.0, 55.0),
            Vec3::new(0.0, 20.0, 0.0),
            0.2,
        );
        piece(
            p,
            Color::srgb(1.0, 0.86, 0.15),
            Vec2::new(60.0, 45.0),
            Vec3::new(25.0, 43.0, 1.0),
            -0.25,
        );
    });
    root
}

pub fn spawn_miner(
    commands: &mut Commands,
    _meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<ColorMaterial>,
    c: &GameConfig,
    team: Team,
    position: Vec2,
) -> Entity {
    let root = commands
        .spawn((
            BattleEntity,
            Unit,
            UnitKind::Miner,
            team,
            Health {
                current: c.miner_health,
                maximum: c.miner_health,
            },
            MoveSpeed(c.miner_speed),
            MinerState::GoingToMine,
            MiningTimer(Timer::from_seconds(c.mining_duration, TimerMode::Once)),
            CarriedGold(0),
            Transform::from_xyz(position.x, position.y, 5.0),
            Visibility::default(),
        ))
        .id();
    stick_figure(commands, root, team, false);
    root
}

pub fn spawn_swordsman(
    commands: &mut Commands,
    _meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<ColorMaterial>,
    c: &GameConfig,
    team: Team,
    position: Vec2,
) -> Entity {
    let root = commands
        .spawn((
            BattleEntity,
            Unit,
            UnitKind::Swordsman,
            team,
            Health {
                current: c.swordsman_health,
                maximum: c.swordsman_health,
            },
            MoveSpeed(c.swordsman_speed),
            Attack {
                damage: c.swordsman_damage,
                range: c.swordsman_range,
                cooldown: Timer::from_seconds(c.swordsman_attack_seconds, TimerMode::Once),
            },
            SwordsmanState::Idle,
            Transform::from_xyz(position.x, position.y, 6.0),
            Visibility::default(),
        ))
        .id();
    stick_figure(commands, root, team, true);
    root
}
