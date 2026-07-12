use crate::model::*;
use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct TimeOfDay {
    pub hour: f32,
    pub saturation_multiplier: f32,
    pub brightness_multiplier: f32,
    day_brightness: f32,
    night_brightness: f32,
}

impl TimeOfDay {
    pub fn from_config(config: &crate::config::GraphicsConfig) -> Self {
        Self {
            hour: config.initial_time_of_day.rem_euclid(24.0),
            saturation_multiplier: config.saturation_multiplier,
            brightness_multiplier: config.brightness_multiplier,
            day_brightness: config.day_brightness,
            night_brightness: config.night_brightness,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LightingLayer {
    Sky,
    Distant,
    Ground,
    Foreground,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct TimeOfDayTint {
    base: Color,
    layer: LightingLayer,
}

impl TimeOfDayTint {
    pub fn new(base: Color, layer: LightingLayer) -> Self {
        Self { base, layer }
    }
}

fn daylight_amount(hour: f32) -> f32 {
    (((hour.rem_euclid(24.0) - 6.0) / 12.0) * std::f32::consts::PI)
        .sin()
        .clamp(0.0, 1.0)
}

fn twilight_amount(hour: f32) -> f32 {
    let distance = [6.0_f32, 18.0]
        .into_iter()
        .map(|center| {
            let direct = (hour.rem_euclid(24.0) - center).abs();
            direct.min(24.0 - direct)
        })
        .fold(f32::INFINITY, f32::min);
    (1.0 - distance / 2.75).clamp(0.0, 1.0)
}

pub fn grade_color(base: Color, time: &TimeOfDay, layer: LightingLayer) -> Color {
    let daylight = daylight_amount(time.hour);
    let twilight = twilight_amount(time.hour);
    let base = base.to_srgba();
    let layer_saturation = match layer {
        LightingLayer::Sky => 0.92,
        LightingLayer::Distant => 0.72,
        LightingLayer::Ground => 0.88,
        LightingLayer::Foreground => 1.0,
    };
    let saturation = (0.64 + daylight * 0.36) * time.saturation_multiplier * layer_saturation;
    let brightness = (time.night_brightness
        + (time.day_brightness - time.night_brightness) * daylight)
        * time.brightness_multiplier;
    let night_tint = Vec3::new(0.48, 0.58, 0.86);
    let day_tint = Vec3::ONE;
    let sunset_tint = Vec3::new(1.08, 0.78, 0.62);
    let tint = night_tint
        .lerp(day_tint, daylight)
        .lerp(sunset_tint, twilight * 0.55);
    let rgb = Vec3::new(base.red, base.green, base.blue);
    let luminance = rgb.dot(Vec3::new(0.2126, 0.7152, 0.0722));
    let graded = Vec3::splat(luminance).lerp(rgb, saturation) * tint * brightness;
    Color::srgba(
        graded.x.clamp(0.0, 1.0),
        graded.y.clamp(0.0, 1.0),
        graded.z.clamp(0.0, 1.0),
        base.alpha,
    )
}

pub fn apply_time_of_day_to_sprites(
    time: Res<TimeOfDay>,
    mut sprites: Query<(&TimeOfDayTint, &mut Sprite)>,
) {
    if !time.is_changed() {
        return;
    }
    for (tint, mut sprite) in &mut sprites {
        sprite.color = grade_color(tint.base, &time, tint.layer);
    }
}

pub fn apply_time_of_day_to_materials(
    time: Res<TimeOfDay>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tinted: Query<(&TimeOfDayTint, &MeshMaterial2d<ColorMaterial>)>,
) {
    if !time.is_changed() {
        return;
    }
    for (tint, handle) in &tinted {
        if let Some(mut material) = materials.get_mut(handle) {
            material.color = grade_color(tint.base, &time, tint.layer);
        }
    }
}

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
    let sky = Color::srgb(0.48, 0.72, 0.9);
    commands.spawn((
        BattleEntity,
        TimeOfDayTint::new(sky, LightingLayer::Sky),
        Sprite::from_color(sky, Vec2::new(3400.0, 900.0)),
        Transform::from_xyz(0.0, 50.0, -20.0),
    ));
    let far_ground = Color::srgb(0.08, 0.24, 0.16);
    commands.spawn((
        BattleEntity,
        TimeOfDayTint::new(far_ground, LightingLayer::Distant),
        Sprite::from_color(far_ground, Vec2::new(3400.0, 80.0)),
        Transform::from_xyz(0.0, -40.0, -18.0),
    ));
    for (x, scale, color) in [
        (-1100.0, Vec2::new(9.0, 2.2), Color::srgb(0.28, 0.48, 0.48)),
        (0.0, Vec2::new(12.0, 2.8), Color::srgb(0.22, 0.42, 0.4)),
        (1200.0, Vec2::new(10.0, 2.4), Color::srgb(0.26, 0.46, 0.44)),
    ] {
        commands.spawn((
            BattleEntity,
            TimeOfDayTint::new(color, LightingLayer::Distant),
            Mesh2d(meshes.add(RegularPolygon::new(70.0, 3))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(x, config.battlefield.ground_y + 185.0, -17.0)
                .with_scale(Vec3::new(scale.x, scale.y, 1.0)),
        ));
    }
    let sun = Color::srgb(1.0, 0.82, 0.28);
    commands.spawn((
        BattleEntity,
        TimeOfDayTint::new(sun, LightingLayer::Sky),
        Mesh2d(meshes.add(Circle::new(58.0))),
        MeshMaterial2d(materials.add(sun)),
        Transform::from_xyz(-1150.0, 185.0, -16.0),
    ));
    let grass = Color::srgb(0.2, 0.5, 0.2);
    commands.spawn((
        BattleEntity,
        TimeOfDayTint::new(grass, LightingLayer::Ground),
        Sprite::from_color(grass, Vec2::new(3400.0, 190.0)),
        Transform::from_xyz(0.0, config.battlefield.ground_y - 120.0, -10.0),
    ));
    let grass_edge = Color::srgb(0.12, 0.28, 0.1);
    commands.spawn((
        BattleEntity,
        TimeOfDayTint::new(grass_edge, LightingLayer::Foreground),
        Sprite::from_color(grass_edge, Vec2::new(3400.0, 8.0)),
        Transform::from_xyz(0.0, config.battlefield.ground_y - 2.0, -9.0),
    ));
    let soil = Color::srgb(0.26, 0.16, 0.09);
    commands.spawn((
        BattleEntity,
        TimeOfDayTint::new(soil, LightingLayer::Ground),
        Sprite::from_color(soil, Vec2::new(3400.0, 55.0)),
        Transform::from_xyz(0.0, config.battlefield.ground_y - 210.0, -8.0),
    ));
    let deep_soil = Color::srgb(0.12, 0.09, 0.07);
    commands.spawn((
        BattleEntity,
        TimeOfDayTint::new(deep_soil, LightingLayer::Ground),
        Sprite::from_color(deep_soil, Vec2::new(3400.0, 22.0)),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn time(hour: f32) -> TimeOfDay {
        TimeOfDay {
            hour,
            saturation_multiplier: 1.0,
            brightness_multiplier: 1.0,
            day_brightness: 1.0,
            night_brightness: 0.46,
        }
    }

    #[test]
    fn night_grading_is_darker_than_day_grading() {
        let base = Color::srgb(0.8, 0.6, 0.4);
        let day = grade_color(base, &time(12.0), LightingLayer::Foreground).to_srgba();
        let night = grade_color(base, &time(0.0), LightingLayer::Foreground).to_srgba();
        assert!(night.red + night.green + night.blue < day.red + day.green + day.blue);
    }

    #[test]
    fn configured_time_wraps_to_a_day() {
        let mut config = crate::config::GameConfig::default().graphics;
        config.initial_time_of_day = 27.0;
        assert_eq!(TimeOfDay::from_config(&config).hour, 3.0);
    }
}
