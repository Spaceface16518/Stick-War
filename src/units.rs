use crate::{
    animation::{CharacterAnimator, CharacterFacing},
    model::*,
    rendering::{LightingLayer, TimeOfDayTint, add_health_bar},
};
use bevy::prelude::*;

fn circle(
    parent: &mut ChildSpawnerCommands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    color: Color,
    radius: f32,
    position: Vec3,
    scale: Vec2,
) {
    parent.spawn((
        Mesh2d(meshes.add(Circle::new(radius))),
        MeshMaterial2d(materials.add(color)),
        Transform::from_translation(position).with_scale(Vec3::new(scale.x, scale.y, 1.0)),
    ));
}

fn stick_figure(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    root: Entity,
) {
    commands.entity(root).with_children(|p| {
        // Characters themselves are now high-resolution atlas sprites attached
        // by the animation runtime. These code-rendered helpers intentionally
        // remain separate so selection and grounding stay crisp at any scale.
        circle(
            p,
            meshes,
            materials,
            Color::srgba(0.02, 0.03, 0.05, 0.3),
            23.0,
            Vec3::new(0.0, -37.0, 0.0),
            Vec2::new(1.6, 0.3),
        );
        p.spawn((
            SelectionMarker,
            Visibility::Hidden,
            Mesh2d(meshes.add(Annulus::new(27.0, 30.0))),
            MeshMaterial2d(materials.add(Color::srgb(0.96, 0.78, 0.2))),
            Transform::from_xyz(0.0, -32.0, 3.0).with_scale(Vec3::new(1.0, 0.3, 1.0)),
        ));
    });
    add_health_bar(commands, root, 62.0, 96.0);
}

pub fn spawn_statue(
    commands: &mut Commands,
    asset_server: &AssetServer,
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
    commands.entity(root).with_children(|p| {
        let image = match team {
            Team::Player => asset_server.load("props/statue-blue.png"),
            Team::Enemy => asset_server.load("props/statue-red.png"),
        };
        p.spawn((
            TimeOfDayTint::new(Color::WHITE, LightingLayer::Foreground),
            Sprite {
                image,
                custom_size: Some(Vec2::new(210.0, 315.0)),
                ..default()
            },
            bevy::sprite::Anchor::BOTTOM_CENTER,
            Transform::from_xyz(0.0, -8.0, 2.0),
        ));
    });
    add_health_bar(commands, root, 160.0, 320.0);
    root
}

pub fn spawn_gold_deposit(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    asset_server: &AssetServer,
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
        circle(
            p,
            meshes,
            materials,
            Color::srgba(0.03, 0.03, 0.02, 0.3),
            44.0,
            Vec3::new(0.0, -2.0, 0.0),
            Vec2::new(1.3, 0.3),
        );
        p.spawn((
            TimeOfDayTint::new(Color::WHITE, LightingLayer::Ground),
            Sprite {
                image: asset_server.load("props/gold-deposit.png"),
                custom_size: Some(Vec2::new(250.0, 125.0)),
                ..default()
            },
            bevy::sprite::Anchor::BOTTOM_CENTER,
            Transform::from_xyz(0.0, -3.0, 2.0),
        ));
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
                current: c.units.miner.health,
                maximum: c.units.miner.health,
            },
            MoveSpeed(c.units.miner.speed),
            MinerState::GoingToMine,
            MiningTimer(Timer::from_seconds(
                c.units.miner.mining_seconds,
                TimerMode::Once,
            )),
            CarriedGold(0),
            MotionEstimate {
                previous_position: position,
                velocity: Vec2::ZERO,
            },
            CharacterAnimator::new(UnitKind::Miner),
            CharacterFacing::for_team(team),
            Transform::from_xyz(position.x, position.y, 5.0),
            Visibility::default(),
        ))
        .id();
    stick_figure(commands, _meshes, _materials, root);
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
                current: c.units.swordsman.health,
                maximum: c.units.swordsman.health,
            },
            MoveSpeed(c.units.swordsman.speed),
            Attack {
                damage: c.units.swordsman.damage,
                range: c.units.swordsman.weapon_range,
                cooldown: Timer::from_seconds(
                    c.units.swordsman.attack_cooldown_seconds,
                    TimerMode::Once,
                ),
            },
            AttackMode::Melee,
            CombatUnitState::Idle,
            MotionEstimate {
                previous_position: position,
                velocity: Vec2::ZERO,
            },
            CharacterAnimator::new(UnitKind::Swordsman),
            CharacterFacing::for_team(team),
            Transform::from_xyz(position.x, position.y, 6.0),
            Visibility::default(),
        ))
        .id();
    stick_figure(commands, _meshes, _materials, root);
    root
}

pub fn spawn_archer(
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
            UnitKind::Archer,
            team,
            Health {
                current: c.units.archer.health,
                maximum: c.units.archer.health,
            },
            MoveSpeed(c.units.archer.speed),
            Attack {
                damage: c.units.archer.damage,
                range: c.units.archer.weapon_range,
                cooldown: Timer::from_seconds(
                    c.units.archer.attack_cooldown_seconds,
                    TimerMode::Once,
                ),
            },
            AttackMode::Projectile,
            CombatUnitState::Idle,
            MotionEstimate {
                previous_position: position,
                velocity: Vec2::ZERO,
            },
            CharacterAnimator::new(UnitKind::Archer),
            CharacterFacing::for_team(team),
            Transform::from_xyz(position.x, position.y, 6.0),
            Visibility::default(),
        ))
        .id();
    stick_figure(commands, _meshes, _materials, root);
    root
}
