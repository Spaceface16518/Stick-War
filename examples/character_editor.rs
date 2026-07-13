//! Hot-reloaded, RON-driven 2D character animation editor.
//!
//! Run with `cargo run --example character_editor --locked`, then edit files in
//! `assets/character_editor/`. The asset server reloads character keyframes and
//! independent weapon definitions without restarting the editor.

use std::collections::HashMap;

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    reflect::TypePath,
};
use serde::Deserialize;

const CHARACTER_PATHS: [&str; 2] = [
    "character_editor/swordsman.ron",
    "character_editor/archer.ron",
];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Stick War Character Editor".into(),
                resolution: (1280, 760).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .init_asset::<StudioAsset>()
        .init_asset_loader::<StudioAssetLoader>()
        .init_resource::<Editor>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                receive_hot_reload,
                sync_weapon,
                editor_input,
                update_cursor,
                advance_animation,
                draw_editor,
                update_toolbar,
            )
                .chain(),
        )
        .run();
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
struct StudioAsset {
    kind: AssetKind,
    character: Option<CharacterDefinition>,
    weapon: Option<WeaponDefinition>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
enum AssetKind {
    Character,
    Weapon,
}

#[derive(Debug, Clone, Deserialize)]
struct CharacterDefinition {
    name: String,
    scale: f32,
    skin_color: Rgba,
    team_color: Rgba,
    #[serde(default)]
    visuals: CharacterVisuals,
    weapon: String,
    joints: Vec<JointDefinition>,
    animations: Vec<AnimationDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
struct CharacterVisuals {
    limb_width_scale: f32,
    head_size: (f32, f32),
    hand_size: (f32, f32),
    foot_size: (f32, f32),
    rig_line_width: f32,
    rig_joint_radius: f32,
}

impl Default for CharacterVisuals {
    fn default() -> Self {
        Self {
            limb_width_scale: 1.0,
            head_size: (25.0, 30.0),
            hand_size: (11.0, 14.0),
            foot_size: (19.0, 10.0),
            rig_line_width: 2.0,
            rig_joint_radius: 3.5,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct JointDefinition {
    name: String,
    parent: Option<String>,
    position: (f32, f32),
    angle_degrees: f32,
    length: f32,
    thickness: f32,
    color: JointColor,
}

#[derive(Debug, Clone, Copy, Deserialize)]
enum JointColor {
    Skin,
    Team,
    Dark,
}

#[derive(Debug, Clone, Deserialize)]
struct AnimationDefinition {
    name: String,
    seconds_per_frame: f32,
    frames: Vec<Keyframe>,
}

#[derive(Debug, Clone, Deserialize)]
struct Keyframe {
    joints: Vec<JointKey>,
}

#[derive(Debug, Clone, Deserialize)]
struct JointKey {
    joint: String,
    position: Option<(f32, f32)>,
    angle_degrees: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
struct WeaponDefinition {
    name: String,
    attach_joint: String,
    position: (f32, f32),
    angle_degrees: f32,
    state: WeaponKind,
    length: f32,
    width: f32,
    color: Rgba,
    states: Vec<WeaponState>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
enum WeaponKind {
    Static,
    Bow,
}

#[derive(Debug, Clone, Deserialize)]
struct WeaponState {
    name: String,
    /// Normalized procedural draw amount. Ignored by static weapons.
    draw: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct Rgba(f32, f32, f32, f32);

impl Rgba {
    fn color(self) -> Color {
        Color::srgba(self.0, self.1, self.2, self.3)
    }
}

#[derive(Default, TypePath)]
struct StudioAssetLoader;

impl AssetLoader for StudioAssetLoader {
    type Asset = StudioAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        ron::de::from_bytes(&bytes)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Resource)]
struct Editor {
    characters: Vec<Handle<StudioAsset>>,
    weapon: Option<Handle<StudioAsset>>,
    character_index: usize,
    animation_index: usize,
    weapon_state_index: usize,
    elapsed: f32,
    playing: bool,
    looping: bool,
    show_grid: bool,
    render_mode: RenderMode,
    cursor_world: Vec2,
    hovered_joint: Option<String>,
    reloads: u32,
    status: String,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            characters: Vec::new(),
            weapon: None,
            character_index: 0,
            animation_index: 0,
            weapon_state_index: 0,
            elapsed: 0.0,
            playing: true,
            looping: true,
            show_grid: true,
            render_mode: RenderMode::Drawn,
            cursor_world: Vec2::ZERO,
            hovered_joint: None,
            reloads: 0,
            status: "Loading RON assets...".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderMode {
    Drawn,
    Rig,
}

impl RenderMode {
    fn label(self) -> &'static str {
        match self {
            Self::Drawn => "drawn",
            Self::Rig => "rig",
        }
    }
}

#[derive(Component)]
struct EditorCamera;

#[derive(Component)]
struct Toolbar;

#[derive(Component)]
struct DrawnPreview;

#[derive(Resource)]
struct PreviewAssets {
    circle: Handle<Mesh>,
    rectangle: Handle<Mesh>,
    skin: Handle<ColorMaterial>,
    team: Handle<ColorMaterial>,
    dark: Handle<ColorMaterial>,
    weapon: Handle<ColorMaterial>,
    outline: Handle<ColorMaterial>,
    gold: Handle<ColorMaterial>,
    wood: Handle<ColorMaterial>,
    string: Handle<ColorMaterial>,
    arrow: Handle<ColorMaterial>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut editor: ResMut<Editor>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    editor.characters = CHARACTER_PATHS
        .iter()
        .map(|path| asset_server.load(*path))
        .collect();

    commands.insert_resource(PreviewAssets {
        circle: meshes.add(Circle::new(0.5)),
        rectangle: meshes.add(Rectangle::new(1.0, 1.0)),
        skin: materials.add(Color::WHITE),
        team: materials.add(Color::WHITE),
        dark: materials.add(Color::srgb(0.08, 0.09, 0.12)),
        weapon: materials.add(Color::WHITE),
        outline: materials.add(Color::srgb(0.06, 0.07, 0.09)),
        gold: materials.add(Color::srgb(0.78, 0.57, 0.18)),
        wood: materials.add(Color::srgb(0.35, 0.19, 0.09)),
        string: materials.add(Color::srgb(0.88, 0.82, 0.68)),
        arrow: materials.add(Color::srgb(0.62, 0.42, 0.2)),
    });

    commands.spawn((
        Camera2d,
        EditorCamera,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: 520.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    commands.spawn((
        Text::new("Loading character editor..."),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.95, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            right: Val::Px(12.0),
            bottom: Val::Px(10.0),
            padding: UiRect::all(Val::Px(9.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.035, 0.045, 0.065, 0.94)),
        Toolbar,
    ));
}

fn current_character<'a>(
    editor: &Editor,
    assets: &'a Assets<StudioAsset>,
) -> Option<&'a CharacterDefinition> {
    let asset = assets.get(editor.characters.get(editor.character_index)?)?;
    (asset.kind == AssetKind::Character)
        .then_some(asset.character.as_ref())
        .flatten()
}

fn current_weapon<'a>(
    editor: &Editor,
    assets: &'a Assets<StudioAsset>,
) -> Option<&'a WeaponDefinition> {
    let asset = assets.get(editor.weapon.as_ref()?)?;
    (asset.kind == AssetKind::Weapon)
        .then_some(asset.weapon.as_ref())
        .flatten()
}

fn receive_hot_reload(
    mut editor: ResMut<Editor>,
    mut events: MessageReader<AssetEvent<StudioAsset>>,
) {
    let changed = events.read().any(|event| {
        matches!(
            event,
            AssetEvent::Added { .. }
                | AssetEvent::Modified { .. }
                | AssetEvent::LoadedWithDependencies { .. }
        )
    });
    if !changed {
        return;
    }

    editor.reloads += 1;
    editor.status = "Asset server applied RON changes".into();
}

fn sync_weapon(
    mut editor: ResMut<Editor>,
    assets: Res<Assets<StudioAsset>>,
    asset_server: Res<AssetServer>,
) {
    let Some(path) = current_character(&editor, &assets).map(|c| c.weapon.clone()) else {
        return;
    };
    let needs_new_weapon = editor
        .weapon
        .as_ref()
        .and_then(|handle| handle.path())
        .is_none_or(|loaded| loaded.path().to_string_lossy() != path);
    if needs_new_weapon {
        editor.weapon = Some(asset_server.load(path));
        editor.weapon_state_index = 0;
    }
}

fn editor_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<Editor>,
    assets: Res<Assets<StudioAsset>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        if editor.playing {
            editor.playing = false;
        } else {
            let duration = current_character(&editor, &assets)
                .and_then(|character| character.animations.get(editor.animation_index))
                .map(animation_duration);
            if let Some(duration) = duration {
                editor.elapsed = resume_elapsed(editor.elapsed, duration, editor.looping);
            }
            editor.playing = true;
        }
    }
    if keys.just_pressed(KeyCode::KeyL) {
        editor.looping = !editor.looping;
        editor.elapsed = 0.0;
        editor.playing = true;
    }
    if keys.just_pressed(KeyCode::KeyG) {
        editor.show_grid = !editor.show_grid;
    }
    if keys.just_pressed(KeyCode::KeyV) {
        editor.render_mode = match editor.render_mode {
            RenderMode::Drawn => RenderMode::Rig,
            RenderMode::Rig => RenderMode::Drawn,
        };
    }
    if keys.just_pressed(KeyCode::KeyR) {
        editor.elapsed = 0.0;
        editor.playing = true;
    }
    if keys.just_pressed(KeyCode::KeyC) && !editor.characters.is_empty() {
        editor.character_index = (editor.character_index + 1) % editor.characters.len();
        editor.animation_index = 0;
        editor.weapon_state_index = 0;
        editor.elapsed = 0.0;
        editor.weapon = None;
    }

    let animation_count =
        current_character(&editor, &assets).map_or(0, |character| character.animations.len());
    if animation_count > 0 && keys.just_pressed(KeyCode::KeyA) {
        editor.animation_index = (editor.animation_index + 1) % animation_count;
        editor.elapsed = 0.0;
        editor.playing = true;
    }

    let state_count = current_weapon(&editor, &assets).map_or(0, |weapon| weapon.states.len());
    if state_count > 0 && keys.just_pressed(KeyCode::KeyW) {
        editor.weapon_state_index = (editor.weapon_state_index + 1) % state_count;
    }
}

fn update_cursor(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    mut editor: ResMut<Editor>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    if let Ok(world) = camera.viewport_to_world_2d(camera_transform, cursor) {
        editor.cursor_world = world;
    }
}

fn advance_animation(
    time: Res<Time>,
    assets: Res<Assets<StudioAsset>>,
    mut editor: ResMut<Editor>,
) {
    if !editor.playing {
        return;
    }
    let Some(animation) =
        current_character(&editor, &assets).and_then(|c| c.animations.get(editor.animation_index))
    else {
        return;
    };
    let duration = animation_duration(animation);
    editor.elapsed += time.delta_secs();
    if editor.elapsed >= duration {
        if editor.looping {
            editor.elapsed %= duration;
        } else {
            editor.elapsed = duration;
            editor.playing = false;
        }
    }
}

fn animation_duration(animation: &AnimationDefinition) -> f32 {
    animation.seconds_per_frame.max(0.001) * animation.frames.len().max(1) as f32
}

fn resume_elapsed(elapsed: f32, duration: f32, looping: bool) -> f32 {
    if !looping && elapsed >= duration {
        0.0
    } else {
        elapsed
    }
}

#[derive(Clone, Copy)]
struct JointPose {
    start: Vec2,
    angle: f32,
    end: Vec2,
}

fn draw_editor(
    mut commands: Commands,
    mut gizmos: Gizmos,
    mut editor: ResMut<Editor>,
    assets: Res<Assets<StudioAsset>>,
    preview_assets: Res<PreviewAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    old_preview: Query<Entity, With<DrawnPreview>>,
) {
    editor.hovered_joint = None;
    for entity in &old_preview {
        commands.entity(entity).despawn();
    }
    if editor.show_grid {
        draw_grid(&mut gizmos);
    }
    let Some(character) = current_character(&editor, &assets) else {
        return;
    };
    let Some(animation) = character.animations.get(editor.animation_index) else {
        return;
    };
    if let Some(mut material) = materials.get_mut(&preview_assets.skin) {
        material.color = character.skin_color.color();
    }
    if let Some(mut material) = materials.get_mut(&preview_assets.team) {
        material.color = character.team_color.color();
    }

    let frame_count = animation.frames.len().max(1);
    let frame_position = editor.elapsed / animation.seconds_per_frame.max(0.001);
    let frame_a = (frame_position.floor() as usize).min(frame_count - 1);
    let frame_b = if editor.looping {
        (frame_a + 1) % frame_count
    } else {
        (frame_a + 1).min(frame_count - 1)
    };
    let blend = frame_position.fract();
    let mut poses = HashMap::new();

    for joint in &character.joints {
        let (offset_a, angle_a) = keyed_pose(joint, animation.frames.get(frame_a));
        let (offset_b, angle_b) = keyed_pose(joint, animation.frames.get(frame_b));
        let local_offset = offset_a.lerp(offset_b, blend) * character.scale;
        let local_angle = angle_a.to_radians()
            + shortest_angle(angle_a.to_radians(), angle_b.to_radians()) * blend;
        let (parent_end, parent_angle) = joint
            .parent
            .as_ref()
            .and_then(|name| poses.get(name))
            .map_or((Vec2::new(0.0, -45.0), 0.0), |pose: &JointPose| {
                (pose.end, pose.angle)
            });
        let start = parent_end + rotate(local_offset, parent_angle);
        let angle = parent_angle + local_angle;
        let end = start + rotate(Vec2::Y * joint.length * character.scale, angle);
        poses.insert(joint.name.clone(), JointPose { start, angle, end });
    }

    for joint in &character.joints {
        let Some(pose) = poses.get(&joint.name) else {
            continue;
        };
        let color = joint_color(character, joint.color);
        match editor.render_mode {
            RenderMode::Rig => draw_rig_joint(&mut gizmos, character, pose, color),
            RenderMode::Drawn => {
                spawn_character_joint(&mut commands, &preview_assets, character, joint, pose)
            }
        }
    }

    if editor.render_mode == RenderMode::Rig {
        let hover_radius = character.visuals.rig_joint_radius.max(3.5) * character.scale * 6.0;
        let hovered = character
            .joints
            .iter()
            .filter_map(|joint| {
                let pose = poses.get(&joint.name)?;
                let marker_distance = pose.start.distance(editor.cursor_world);
                let bone_distance =
                    point_segment_distance(editor.cursor_world, pose.start, pose.end);
                let distance = marker_distance.min(bone_distance);
                (distance <= hover_radius).then_some((joint.name.as_str(), distance))
            })
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(name, _)| name.to_owned());
        if let Some(name) = hovered.as_deref()
            && let Some(pose) = poses.get(name)
        {
            draw_filled_ellipse(
                &mut gizmos,
                pose.start,
                Vec2::splat(character.visuals.rig_joint_radius * character.scale * 2.2),
                0.0,
                Color::srgb(1.0, 0.88, 0.12),
            );
        }
        editor.hovered_joint = hovered;
    }

    if let Some(weapon) = current_weapon(&editor, &assets)
        && let Some(hand) = poses.get(&weapon.attach_joint)
    {
        if let Some(mut material) = materials.get_mut(&preview_assets.weapon) {
            material.color = weapon.color.color();
        }
        match editor.render_mode {
            RenderMode::Rig => draw_weapon_rig(
                &mut gizmos,
                weapon,
                hand,
                editor.weapon_state_index,
                character.scale,
            ),
            RenderMode::Drawn => spawn_weapon_sprite(
                &mut commands,
                &preview_assets,
                weapon,
                hand,
                editor.weapon_state_index,
                character.scale,
            ),
        }
    }
}

fn joint_color(character: &CharacterDefinition, color: JointColor) -> Color {
    match color {
        JointColor::Skin => character.skin_color.color(),
        JointColor::Team => character.team_color.color(),
        JointColor::Dark => Color::srgb(0.08, 0.09, 0.12),
    }
}

fn draw_rig_joint(
    gizmos: &mut Gizmos,
    character: &CharacterDefinition,
    pose: &JointPose,
    color: Color,
) {
    draw_rounded_line(
        gizmos,
        pose.start,
        pose.end,
        character.visuals.rig_line_width * character.scale,
        color,
    );
    draw_filled_ellipse(
        gizmos,
        pose.start,
        Vec2::splat(character.visuals.rig_joint_radius * character.scale),
        0.0,
        color,
    );
}

fn spawn_character_joint(
    commands: &mut Commands,
    assets: &PreviewAssets,
    character: &CharacterDefinition,
    joint: &JointDefinition,
    pose: &JointPose,
) {
    let material = match joint.color {
        JointColor::Skin => assets.skin.clone(),
        JointColor::Team => assets.team.clone(),
        JointColor::Dark => assets.dark.clone(),
    };
    spawn_capsule(
        commands,
        assets,
        pose.start,
        pose.end,
        joint.thickness * character.visuals.limb_width_scale * character.scale,
        material.clone(),
        10.0,
    );

    let terminal = !character
        .joints
        .iter()
        .any(|candidate| candidate.parent.as_deref() == Some(joint.name.as_str()));
    let feature = if joint.name == "head" {
        Some(character.visuals.head_size)
    } else if joint.name.contains("hand") || (terminal && joint.name.contains("forearm")) {
        Some(character.visuals.hand_size)
    } else if terminal && (joint.name.contains("shin") || joint.name.contains("leg")) {
        Some(character.visuals.foot_size)
    } else {
        None
    };
    if let Some((width, height)) = feature {
        spawn_ellipse(
            commands,
            assets,
            pose.end,
            Vec2::new(width, height) * character.scale * 0.5,
            pose.angle,
            material,
            12.0,
        );
    }
}

fn keyed_pose(joint: &JointDefinition, frame: Option<&Keyframe>) -> (Vec2, f32) {
    let key = frame.and_then(|frame| frame.joints.iter().find(|key| key.joint == joint.name));
    let offset = key.and_then(|key| key.position).unwrap_or(joint.position);
    let angle = joint.angle_degrees + key.and_then(|key| key.angle_degrees).unwrap_or(0.0);
    (Vec2::new(offset.0, offset.1), angle)
}

fn draw_weapon_rig(
    gizmos: &mut Gizmos,
    weapon: &WeaponDefinition,
    hand: &JointPose,
    state_index: usize,
    scale: f32,
) {
    let position = hand.end
        + rotate(
            Vec2::new(weapon.position.0, weapon.position.1) * scale,
            hand.angle,
        );
    let angle = hand.angle + weapon.angle_degrees.to_radians();
    let axis = rotate(Vec2::Y, angle);
    let side = Vec2::new(-axis.y, axis.x);
    let half = weapon.length * scale * 0.5;
    let color = weapon.color.color();

    match weapon.state {
        WeaponKind::Static => {
            // A small stacked sword sprite: dark outline, steel blade, center
            // highlight, crossguard, grip, and pommel.
            draw_rounded_line(
                gizmos,
                position - axis * half,
                position + axis * half,
                (weapon.width + 3.0) * scale,
                Color::srgb(0.10, 0.12, 0.16),
            );
            draw_rounded_line(
                gizmos,
                position - axis * half,
                position + axis * half,
                weapon.width * scale,
                color,
            );
            gizmos.line_2d(
                position - axis * half,
                position + axis * half,
                Color::srgba(1.0, 1.0, 1.0, 0.65),
            );
            draw_rounded_line(
                gizmos,
                position - side * 9.0 * scale,
                position + side * 9.0 * scale,
                5.0 * scale,
                Color::srgb(0.78, 0.57, 0.18),
            );
            draw_rounded_line(
                gizmos,
                position - axis * half,
                position - axis * (half + 13.0 * scale),
                6.0 * scale,
                Color::srgb(0.35, 0.19, 0.09),
            );
            draw_filled_ellipse(
                gizmos,
                position - axis * (half + 15.0 * scale),
                Vec2::splat(5.0 * scale),
                angle,
                Color::srgb(0.78, 0.57, 0.18),
            );
        }
        WeaponKind::Bow => {
            let top = position + axis * half;
            let bottom = position - axis * half;
            let belly = side * 12.0 * scale;
            // Layered bow sprite with a dark silhouette and warm inner stave.
            draw_rounded_line(
                gizmos,
                bottom,
                position + belly,
                (weapon.width + 3.0) * scale,
                Color::srgb(0.12, 0.07, 0.035),
            );
            draw_rounded_line(
                gizmos,
                position + belly,
                top,
                (weapon.width + 3.0) * scale,
                Color::srgb(0.12, 0.07, 0.035),
            );
            draw_rounded_line(
                gizmos,
                bottom,
                position + belly,
                weapon.width * scale,
                color,
            );
            draw_rounded_line(gizmos, position + belly, top, weapon.width * scale, color);
            let draw = weapon
                .states
                .get(state_index)
                .map_or(0.0, |state| state.draw.clamp(0.0, 1.0));
            let nock = position - side * draw * 27.0 * scale;
            gizmos.line_2d(top, nock, Color::srgb(0.88, 0.82, 0.68));
            gizmos.line_2d(nock, bottom, Color::srgb(0.88, 0.82, 0.68));
            if draw > 0.05 {
                let arrow_tip = nock + side * (62.0 + draw * 18.0) * scale;
                draw_rounded_line(
                    gizmos,
                    nock,
                    arrow_tip,
                    2.0 * scale,
                    Color::srgb(0.62, 0.42, 0.2),
                );
                let tip_side = Vec2::new(-side.y, side.x);
                gizmos.line_2d(
                    arrow_tip,
                    arrow_tip - side * 8.0 * scale + tip_side * 4.0 * scale,
                    Color::srgb(0.82, 0.84, 0.87),
                );
                gizmos.line_2d(
                    arrow_tip,
                    arrow_tip - side * 8.0 * scale - tip_side * 4.0 * scale,
                    Color::srgb(0.82, 0.84, 0.87),
                );
            }
        }
    }
}

fn spawn_weapon_sprite(
    commands: &mut Commands,
    assets: &PreviewAssets,
    weapon: &WeaponDefinition,
    hand: &JointPose,
    state_index: usize,
    scale: f32,
) {
    let position = hand.end
        + rotate(
            Vec2::new(weapon.position.0, weapon.position.1) * scale,
            hand.angle,
        );
    let angle = hand.angle + weapon.angle_degrees.to_radians();
    let axis = rotate(Vec2::Y, angle);
    let side = Vec2::new(-axis.y, axis.x);
    let half = weapon.length * scale * 0.5;
    match weapon.state {
        WeaponKind::Static => {
            spawn_capsule(
                commands,
                assets,
                position - axis * half,
                position + axis * half,
                (weapon.width + 3.0) * scale,
                assets.outline.clone(),
                20.0,
            );
            spawn_capsule(
                commands,
                assets,
                position - axis * half,
                position + axis * half,
                weapon.width * scale,
                assets.weapon.clone(),
                21.0,
            );
            spawn_capsule(
                commands,
                assets,
                position - side * 10.0 * scale,
                position + side * 10.0 * scale,
                5.0 * scale,
                assets.gold.clone(),
                22.0,
            );
            spawn_capsule(
                commands,
                assets,
                position - axis * half,
                position - axis * (half + 14.0 * scale),
                6.0 * scale,
                assets.wood.clone(),
                21.0,
            );
            spawn_ellipse(
                commands,
                assets,
                position - axis * (half + 16.0 * scale),
                Vec2::splat(5.0 * scale),
                angle,
                assets.gold.clone(),
                22.0,
            );
        }
        WeaponKind::Bow => {
            let top = position + axis * half;
            let bottom = position - axis * half;
            let belly = position + side * 12.0 * scale;
            spawn_capsule(
                commands,
                assets,
                bottom,
                belly,
                (weapon.width + 3.0) * scale,
                assets.outline.clone(),
                20.0,
            );
            spawn_capsule(
                commands,
                assets,
                belly,
                top,
                (weapon.width + 3.0) * scale,
                assets.outline.clone(),
                20.0,
            );
            spawn_capsule(
                commands,
                assets,
                bottom,
                belly,
                weapon.width * scale,
                assets.weapon.clone(),
                21.0,
            );
            spawn_capsule(
                commands,
                assets,
                belly,
                top,
                weapon.width * scale,
                assets.weapon.clone(),
                21.0,
            );
            let draw = weapon
                .states
                .get(state_index)
                .map_or(0.0, |state| state.draw.clamp(0.0, 1.0));
            let nock = position - side * draw * 27.0 * scale;
            spawn_capsule(
                commands,
                assets,
                top,
                nock,
                1.25 * scale,
                assets.string.clone(),
                22.0,
            );
            spawn_capsule(
                commands,
                assets,
                nock,
                bottom,
                1.25 * scale,
                assets.string.clone(),
                22.0,
            );
            if draw > 0.05 {
                let arrow_tip = nock + side * (62.0 + draw * 18.0) * scale;
                spawn_capsule(
                    commands,
                    assets,
                    nock,
                    arrow_tip,
                    2.0 * scale,
                    assets.arrow.clone(),
                    23.0,
                );
            }
        }
    }
}

fn spawn_capsule(
    commands: &mut Commands,
    assets: &PreviewAssets,
    start: Vec2,
    end: Vec2,
    width: f32,
    material: Handle<ColorMaterial>,
    z: f32,
) {
    let delta = end - start;
    let length = delta.length();
    let midpoint = (start + end) * 0.5;
    let angle = Vec2::Y.angle_to(delta);
    commands.spawn((
        DrawnPreview,
        Mesh2d(assets.rectangle.clone()),
        MeshMaterial2d(material.clone()),
        Transform::from_xyz(midpoint.x, midpoint.y, z)
            .with_rotation(Quat::from_rotation_z(angle))
            .with_scale(Vec3::new(width, length, 1.0)),
    ));
    for point in [start, end] {
        spawn_ellipse(
            commands,
            assets,
            point,
            Vec2::splat(width * 0.5),
            0.0,
            material.clone(),
            z + 0.01,
        );
    }
}

fn spawn_ellipse(
    commands: &mut Commands,
    assets: &PreviewAssets,
    center: Vec2,
    radii: Vec2,
    angle: f32,
    material: Handle<ColorMaterial>,
    z: f32,
) {
    commands.spawn((
        DrawnPreview,
        Mesh2d(assets.circle.clone()),
        MeshMaterial2d(material),
        Transform::from_xyz(center.x, center.y, z)
            .with_rotation(Quat::from_rotation_z(angle))
            .with_scale(Vec3::new(radii.x * 2.0, radii.y * 2.0, 1.0)),
    ));
}

fn draw_grid(gizmos: &mut Gizmos) {
    for coordinate in (-500..=500).step_by(20) {
        let major = coordinate % 100 == 0;
        let color = if major {
            Color::srgba(0.3, 0.43, 0.62, 0.35)
        } else {
            Color::srgba(0.2, 0.29, 0.42, 0.16)
        };
        let c = coordinate as f32;
        gizmos.line_2d(Vec2::new(c, -300.0), Vec2::new(c, 300.0), color);
        gizmos.line_2d(Vec2::new(-600.0, c), Vec2::new(600.0, c), color);
    }
    gizmos.line_2d(
        Vec2::new(-600.0, 0.0),
        Vec2::new(600.0, 0.0),
        Color::srgba(0.9, 0.3, 0.3, 0.6),
    );
    gizmos.line_2d(
        Vec2::new(0.0, -300.0),
        Vec2::new(0.0, 300.0),
        Color::srgba(0.3, 0.9, 0.45, 0.6),
    );
}

fn draw_rounded_line(gizmos: &mut Gizmos, start: Vec2, end: Vec2, width: f32, color: Color) {
    let normal = (end - start).normalize_or_zero().perp() * width * 0.5;
    for factor in [-1.0, -0.75, -0.5, -0.25, 0.0, 0.25, 0.5, 0.75, 1.0] {
        gizmos.line_2d(start + normal * factor, end + normal * factor, color);
    }
    draw_filled_ellipse(gizmos, start, Vec2::splat(width * 0.5), 0.0, color);
    draw_filled_ellipse(gizmos, end, Vec2::splat(width * 0.5), 0.0, color);
}

fn draw_filled_ellipse(
    gizmos: &mut Gizmos,
    center: Vec2,
    radii: Vec2,
    rotation: f32,
    color: Color,
) {
    let steps = 12;
    for step in -steps..=steps {
        let normalized_y = step as f32 / steps as f32;
        let half_width = radii.x * (1.0 - normalized_y * normalized_y).sqrt();
        let local_y = normalized_y * radii.y;
        let start = center + rotate(Vec2::new(-half_width, local_y), rotation);
        let end = center + rotate(Vec2::new(half_width, local_y), rotation);
        gizmos.line_2d(start, end, color);
    }
}

fn rotate(vector: Vec2, angle: f32) -> Vec2 {
    Vec2::new(
        vector.x * angle.cos() - vector.y * angle.sin(),
        vector.x * angle.sin() + vector.y * angle.cos(),
    )
}

fn point_segment_distance(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let segment = end - start;
    let length_squared = segment.length_squared();
    if length_squared <= f32::EPSILON {
        return point.distance(start);
    }
    let progress = ((point - start).dot(segment) / length_squared).clamp(0.0, 1.0);
    point.distance(start + segment * progress)
}

fn shortest_angle(from: f32, to: f32) -> f32 {
    (to - from + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

fn update_toolbar(
    editor: Res<Editor>,
    assets: Res<Assets<StudioAsset>>,
    mut toolbar: Query<&mut Text, With<Toolbar>>,
) {
    let Ok(mut text) = toolbar.single_mut() else {
        return;
    };
    let character = current_character(&editor, &assets);
    let animation = character.and_then(|c| c.animations.get(editor.animation_index));
    let weapon = current_weapon(&editor, &assets);
    let weapon_state = weapon.and_then(|w| w.states.get(editor.weapon_state_index));
    let frame = animation.map_or(0, |animation| {
        ((editor.elapsed / animation.seconds_per_frame.max(0.001)).floor() as usize)
            .min(animation.frames.len().saturating_sub(1))
    });

    **text = format!(
        "CHARACTER  {}   |   ANIMATION  {}   |   FRAME  {}/{}   |   TIME  {:.2}s   |   {:.3}s/frame\n\
         VIEW  {}   |   JOINT  {}   |   WEAPON  {}   |   STATE  {}   |   PLAYBACK  {} / {}   |   GRID  {}   |   CURSOR  ({:.1}, {:.1})\n\
         [V] drawn/rig   [C] character   [A] animation   [W] weapon state   [Space] play/pause   [L] loop/once   [R] restart   [G] grid   |   {} (reload #{})",
        character.map_or("--", |c| c.name.as_str()),
        animation.map_or("--", |a| a.name.as_str()),
        frame + 1,
        animation.map_or(0, |a| a.frames.len()),
        editor.elapsed,
        animation.map_or(0.0, |a| a.seconds_per_frame),
        editor.render_mode.label(),
        editor.hovered_joint.as_deref().unwrap_or("--"),
        weapon.map_or("--", |w| w.name.as_str()),
        weapon_state.map_or("--", |state| state.name.as_str()),
        if editor.playing { "playing" } else { "paused" },
        if editor.looping { "loop" } else { "once" },
        if editor.show_grid { "on" } else { "off" },
        editor.cursor_world.x,
        editor.cursor_world.y,
        editor.status,
        editor.reloads,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> StudioAsset {
        ron::from_str(source).expect("example RON should deserialize")
    }

    #[test]
    fn bundled_character_and_weapon_assets_are_valid() {
        let assets = [
            parse(include_str!("../assets/character_editor/swordsman.ron")),
            parse(include_str!("../assets/character_editor/archer.ron")),
            parse(include_str!("../assets/character_editor/sword.ron")),
            parse(include_str!("../assets/character_editor/bow.ron")),
        ];

        for asset in assets {
            match asset.kind {
                AssetKind::Character => {
                    let character = asset.character.expect("character payload");
                    assert!(!character.animations.is_empty());
                    assert!(
                        character
                            .animations
                            .iter()
                            .all(|animation| animation.seconds_per_frame > 0.0
                                && !animation.frames.is_empty())
                    );
                    for joint in &character.joints {
                        if let Some(parent) = &joint.parent {
                            assert!(
                                character
                                    .joints
                                    .iter()
                                    .any(|candidate| &candidate.name == parent)
                            );
                        }
                    }
                }
                AssetKind::Weapon => {
                    let weapon = asset.weapon.expect("weapon payload");
                    assert!(!weapon.attach_joint.is_empty());
                    assert!(!weapon.states.is_empty());
                }
            }
        }
    }

    #[test]
    fn once_playback_rewinds_only_after_reaching_the_end() {
        assert_eq!(resume_elapsed(0.4, 1.0, false), 0.4);
        assert_eq!(resume_elapsed(1.0, 1.0, false), 0.0);
        assert_eq!(resume_elapsed(1.0, 1.0, true), 1.0);
    }

    #[test]
    fn joint_hit_test_accepts_markers_and_bone_segments() {
        let start = Vec2::new(10.0, 20.0);
        let end = Vec2::new(10.0, 60.0);
        assert_eq!(
            point_segment_distance(Vec2::new(10.0, 20.0), start, end),
            0.0
        );
        assert_eq!(
            point_segment_distance(Vec2::new(13.0, 40.0), start, end),
            3.0
        );
        assert_eq!(
            point_segment_distance(Vec2::new(10.0, 70.0), start, end),
            10.0
        );
    }
}
