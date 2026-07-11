use crate::model::*;
use bevy::prelude::*;

pub fn team_color(team: Team) -> Color {
    if team == Team::Player {
        Color::srgb(0.15, 0.3, 0.9)
    } else {
        Color::srgb(0.85, 0.15, 0.12)
    }
}

pub fn spawn_battlefield(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    config: &GameConfig,
) {
    commands.spawn((
        BattleEntity,
        Sprite::from_color(Color::srgb(0.48, 0.72, 0.9), Vec2::new(3400.0, 900.0)),
        Transform::from_xyz(0.0, 50.0, -20.0),
    ));
    commands.spawn((
        BattleEntity,
        Sprite::from_color(Color::srgb(0.08, 0.24, 0.16), Vec2::new(3400.0, 80.0)),
        Transform::from_xyz(0.0, -40.0, -18.0),
    ));
    for (x, scale, color) in [
        (-1100.0, Vec2::new(9.0, 2.2), Color::srgb(0.28, 0.48, 0.48)),
        (0.0, Vec2::new(12.0, 2.8), Color::srgb(0.22, 0.42, 0.4)),
        (1200.0, Vec2::new(10.0, 2.4), Color::srgb(0.26, 0.46, 0.44)),
    ] {
        commands.spawn((
            BattleEntity,
            Mesh2d(meshes.add(RegularPolygon::new(70.0, 3))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(x, config.battlefield.ground_y + 185.0, -17.0)
                .with_scale(Vec3::new(scale.x, scale.y, 1.0)),
        ));
    }
    commands.spawn((
        BattleEntity,
        Mesh2d(meshes.add(Circle::new(58.0))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.82, 0.28))),
        Transform::from_xyz(-1150.0, 185.0, -16.0),
    ));
    commands.spawn((
        BattleEntity,
        Sprite::from_color(Color::srgb(0.2, 0.5, 0.2), Vec2::new(3400.0, 190.0)),
        Transform::from_xyz(0.0, config.battlefield.ground_y - 120.0, -10.0),
    ));
    commands.spawn((
        BattleEntity,
        Sprite::from_color(Color::srgb(0.12, 0.28, 0.1), Vec2::new(3400.0, 8.0)),
        Transform::from_xyz(0.0, config.battlefield.ground_y - 2.0, -9.0),
    ));
    commands.spawn((
        BattleEntity,
        Sprite::from_color(Color::srgb(0.26, 0.16, 0.09), Vec2::new(3400.0, 55.0)),
        Transform::from_xyz(0.0, config.battlefield.ground_y - 210.0, -8.0),
    ));
    commands.spawn((
        BattleEntity,
        Sprite::from_color(Color::srgb(0.12, 0.09, 0.07), Vec2::new(3400.0, 22.0)),
        Transform::from_xyz(0.0, config.battlefield.ground_y - 250.0, -7.0),
    ));
}

pub fn add_health_bar(commands: &mut Commands, owner: Entity, width: f32, y: f32) {
    commands.entity(owner).with_children(|p| {
        p.spawn((
            Sprite::from_color(Color::srgb(0.12, 0.08, 0.08), Vec2::new(width + 4.0, 9.0)),
            Transform::from_xyz(0.0, y, 3.0),
        ));
        p.spawn((
            HealthBarFill { owner },
            Sprite::from_color(Color::srgb(0.2, 0.9, 0.25), Vec2::new(width, 5.0)),
            Transform::from_xyz(0.0, y, 4.0),
        ));
    });
}

pub fn update_health_bars(
    health: Query<&Health>,
    mut fills: Query<(&HealthBarFill, &mut Transform)>,
) {
    for (fill, mut transform) in &mut fills {
        if let Ok(h) = health.get(fill.owner) {
            transform.scale.x = (h.current / h.maximum).clamp(0.0, 1.0);
        }
    }
}

pub fn update_selection_markers(
    controlled: Query<Entity, With<Controlled>>,
    mut markers: Query<(&ChildOf, &mut Visibility), With<SelectionMarker>>,
) {
    for (parent, mut visibility) in &mut markers {
        *visibility = if controlled.get(parent.parent()).is_ok() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn animate_walking(
    time: Res<Time>,
    roots: Query<(Option<&MinerState>, Option<&CombatUnitState>), With<Unit>>,
    mut limbs: Query<(&ChildOf, &Limb, &mut Transform)>,
) {
    let swing = (time.elapsed_secs() * 8.0).sin() * 0.4;
    for (parent, limb, mut transform) in &mut limbs {
        let Ok((miner, swordsman)) = roots.get(parent.parent()) else {
            continue;
        };
        let moving = miner.is_some_and(|state| *state != MinerState::Mining)
            || swordsman.is_some_and(|state| {
                matches!(state, CombatUnitState::Moving | CombatUnitState::Retreating)
            });
        let direction = match limb.kind {
            LimbKind::LeftArm | LimbKind::RightLeg => 1.0,
            LimbKind::RightArm | LimbKind::LeftLeg => -1.0,
        };
        let angle = limb.rest_angle + if moving { swing * direction } else { 0.0 };
        transform.rotation = Quat::from_rotation_z(angle);
    }
}

pub fn animate_weapons(
    roots: Query<(&Attack, &UnitKind)>,
    mut weapons: Query<(&ChildOf, &WeaponVisual, &mut Transform)>,
) {
    for (parent, weapon, mut transform) in &mut weapons {
        let Ok((attack, kind)) = roots.get(parent.parent()) else {
            continue;
        };
        let progress = attack.cooldown.fraction();
        match weapon.kind {
            UnitKind::Swordsman => {
                let swing = if *kind == UnitKind::Swordsman && progress < 0.45 {
                    -1.35 + progress * 4.5
                } else {
                    -0.35
                };
                transform.rotation = Quat::from_rotation_z(swing);
            }
            UnitKind::Archer => {
                transform.scale.x = 0.45 + (1.0 - progress).max(0.0) * 0.18;
            }
            UnitKind::Miner => {}
        }
    }
}

pub fn update_gold_sacks(
    miners: Query<&CarriedGold>,
    mut sacks: Query<(&ChildOf, &mut Visibility), With<GoldSackVisual>>,
) {
    for (parent, mut visibility) in &mut sacks {
        *visibility = miners
            .get(parent.parent())
            .map(|gold| {
                if gold.0 > 0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                }
            })
            .unwrap_or(Visibility::Hidden);
    }
}

pub fn tick_timed_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut TimedEffect, &mut Sprite)>,
) {
    for (entity, mut timer, mut sprite) in &mut effects {
        timer.0.tick(time.delta());
        sprite.color = sprite.color.with_alpha(1.0 - timer.0.fraction());
        if timer.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
