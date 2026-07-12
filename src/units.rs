use crate::{
    animation::{CharacterAnimator, CharacterFacing},
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

fn polygon(
    parent: &mut ChildSpawnerCommands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    color: Color,
    radius: f32,
    sides: u32,
    position: Vec3,
    rotation: f32,
) {
    parent.spawn((
        Mesh2d(meshes.add(RegularPolygon::new(radius, sides))),
        MeshMaterial2d(materials.add(color)),
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

fn stick_figure(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    root: Entity,
    team: Team,
    kind: UnitKind,
) {
    let color = team_color(team);
    commands.entity(root).with_children(|p| {
        circle(
            p,
            meshes,
            materials,
            Color::srgba(0.02, 0.03, 0.05, 0.3),
            23.0,
            Vec3::new(0.0, -37.0, 0.0),
            Vec2::new(1.15, 0.25),
        );
        circle(
            p,
            meshes,
            materials,
            Color::srgb(0.94, 0.78, 0.59),
            13.0,
            Vec3::new(0.0, 58.0, 2.0),
            Vec2::ONE,
        );
        piece(
            p,
            Color::srgb(0.12, 0.13, 0.16),
            Vec2::new(10.0, 48.0),
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
        piece(
            p,
            color,
            Vec2::new(24.0, 8.0),
            Vec3::new(0.0, 42.0, 2.0),
            0.0,
        );
        match kind {
            UnitKind::Miner => {
                piece(
                    p,
                    Color::srgb(0.92, 0.66, 0.1),
                    Vec2::new(31.0, 9.0),
                    Vec3::new(0.0, 68.0, 3.0),
                    0.0,
                );
                piece(
                    p,
                    Color::srgb(0.36, 0.22, 0.12),
                    Vec2::new(5.0, 42.0),
                    Vec3::new(25.0, 34.0, 3.0),
                    -0.65,
                );
                polygon(
                    p,
                    meshes,
                    materials,
                    Color::srgb(0.7, 0.73, 0.76),
                    14.0,
                    3,
                    Vec3::new(12.0, 53.0, 4.0),
                    -0.65,
                );
                p.spawn((
                    GoldSackVisual,
                    Visibility::Hidden,
                    Mesh2d(meshes.add(Circle::new(13.0))),
                    MeshMaterial2d(materials.add(Color::srgb(0.86, 0.61, 0.08))),
                    Transform::from_xyz(-22.0, 8.0, 3.0).with_scale(Vec3::new(1.0, 1.2, 1.0)),
                ));
            }
            UnitKind::Swordsman => {
                polygon(
                    p,
                    meshes,
                    materials,
                    Color::srgb(0.38, 0.42, 0.5),
                    17.0,
                    5,
                    Vec3::new(0.0, 66.0, 3.0),
                    0.0,
                );
                circle(
                    p,
                    meshes,
                    materials,
                    color,
                    18.0,
                    Vec3::new(-20.0, 28.0, 3.0),
                    Vec2::new(0.7, 1.0),
                );
                p.spawn((
                    WeaponVisual { kind },
                    Sprite::from_color(Color::srgb(0.86, 0.88, 0.94), Vec2::new(6.0, 57.0)),
                    Transform::from_xyz(27.0, 35.0, 4.0)
                        .with_rotation(Quat::from_rotation_z(-0.35)),
                ));
            }
            UnitKind::Archer => {
                piece(
                    p,
                    Color::srgb(0.27, 0.18, 0.1),
                    Vec2::new(10.0, 50.0),
                    Vec3::new(-19.0, 31.0, 2.0),
                    -0.15,
                );
                for y in [15.0, 27.0, 39.0] {
                    piece(
                        p,
                        Color::srgb(0.74, 0.58, 0.3),
                        Vec2::new(3.0, 39.0),
                        Vec3::new(-22.0, y, 3.0),
                        -0.18,
                    );
                }
                p.spawn((
                    WeaponVisual { kind },
                    Mesh2d(meshes.add(Annulus::new(17.0, 20.0))),
                    MeshMaterial2d(materials.add(color)),
                    Transform::from_xyz(24.0, 34.0, 4.0).with_scale(Vec3::new(0.55, 1.25, 1.0)),
                ));
                piece(
                    p,
                    Color::srgb(0.82, 0.73, 0.52),
                    Vec2::new(2.0, 49.0),
                    Vec3::new(24.0, 34.0, 5.0),
                    0.0,
                );
            }
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
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
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
        polygon(
            p,
            meshes,
            materials,
            Color::srgb(0.34, 0.35, 0.4),
            62.0,
            6,
            Vec3::new(0.0, 72.0, 2.0),
            0.0,
        );
        piece(
            p,
            color,
            Vec2::new(65.0, 140.0),
            Vec3::new(0.0, 75.0, 1.0),
            0.0,
        );
        circle(
            p,
            meshes,
            materials,
            Color::srgb(0.92, 0.8, 0.45),
            28.0,
            Vec3::new(0.0, 165.0, 3.0),
            Vec2::ONE,
        );
        polygon(
            p,
            meshes,
            materials,
            color,
            40.0,
            3,
            Vec3::new(0.0, 205.0, 2.0),
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
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
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
        piece(
            p,
            Color::srgb(0.95, 0.7, 0.05),
            Vec2::new(85.0, 55.0),
            Vec3::new(0.0, 20.0, 0.0),
            0.2,
        );
        for (x, y, radius) in [(-25.0, 27.0, 23.0), (10.0, 33.0, 28.0), (34.0, 20.0, 19.0)] {
            polygon(
                p,
                meshes,
                materials,
                Color::srgb(0.96, 0.7, 0.08),
                radius,
                5,
                Vec3::new(x, y, 2.0),
                x * 0.01,
            );
        }
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
    stick_figure(commands, _meshes, _materials, root, team, UnitKind::Miner);
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
    stick_figure(
        commands,
        _meshes,
        _materials,
        root,
        team,
        UnitKind::Swordsman,
    );
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
    stick_figure(commands, _meshes, _materials, root, team, UnitKind::Archer);
    root
}
