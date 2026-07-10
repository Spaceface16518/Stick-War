use crate::model::*;
use bevy::prelude::*;

pub fn team_color(team: Team) -> Color {
    if team == Team::Player {
        Color::srgb(0.15, 0.3, 0.9)
    } else {
        Color::srgb(0.85, 0.15, 0.12)
    }
}

pub fn spawn_battlefield(commands: &mut Commands, config: &GameConfig) {
    commands.spawn((
        BattleEntity,
        Sprite::from_color(Color::srgb(0.48, 0.72, 0.9), Vec2::new(3400.0, 900.0)),
        Transform::from_xyz(0.0, 50.0, -20.0),
    ));
    commands.spawn((
        BattleEntity,
        Sprite::from_color(Color::srgb(0.18, 0.45, 0.18), Vec2::new(3400.0, 260.0)),
        Transform::from_xyz(0.0, config.ground_y - 120.0, -10.0),
    ));
    commands.spawn((
        BattleEntity,
        Sprite::from_color(Color::srgb(0.12, 0.28, 0.1), Vec2::new(3400.0, 8.0)),
        Transform::from_xyz(0.0, config.ground_y - 2.0, -9.0),
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
