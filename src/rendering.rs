use crate::model::*;
use bevy::prelude::*;
use bevy::render::view::ColorGrading;

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
    mut sprites: Query<(Ref<TimeOfDayTint>, &mut Sprite)>,
) {
    for (tint, mut sprite) in &mut sprites {
        if time.is_changed() || tint.is_added() || tint.is_changed() {
            sprite.color = grade_color(tint.base, &time, tint.layer);
        }
    }
}

pub fn apply_time_of_day_to_materials(
    time: Res<TimeOfDay>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tinted: Query<(Ref<TimeOfDayTint>, &MeshMaterial2d<ColorMaterial>)>,
) {
    for (tint, handle) in &tinted {
        if (time.is_changed() || tint.is_added() || tint.is_changed())
            && let Some(mut material) = materials.get_mut(handle)
        {
            material.color = grade_color(tint.base, &time, tint.layer);
        }
    }
}

pub fn apply_time_of_day_to_camera(
    time: Res<TimeOfDay>,
    mut grading: Single<&mut ColorGrading, With<BattleCamera>>,
) {
    if !time.is_changed() {
        return;
    }
    let daylight = daylight_amount(time.hour);
    grading.global.post_saturation = (0.68 + daylight * 0.32) * time.saturation_multiplier;
    grading.global.temperature = twilight_amount(time.hour) * 0.045 - (1.0 - daylight) * 0.035;
    grading.global.tint = twilight_amount(time.hour) * 0.012;
}

pub fn spawn_battlefield(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        BattleEntity,
        ParallaxLayer(0.04),
        TimeOfDayTint::new(Color::WHITE, LightingLayer::Sky),
        Sprite {
            image: asset_server.load("environment/battlefield.png"),
            custom_size: Some(Vec2::new(3500.0, 1013.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -65.0, -20.0),
    ));
}

pub fn update_parallax_layers(
    camera: Single<&Transform, (With<BattleCamera>, Without<ParallaxLayer>)>,
    mut layers: Query<(&ParallaxLayer, &mut Transform), Without<BattleCamera>>,
) {
    for (layer, mut transform) in &mut layers {
        transform.translation.x = camera.translation.x * layer.0;
    }
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
