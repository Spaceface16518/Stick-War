use std::collections::HashMap;

use bevy::{
    asset::{AssetLoader, LoadContext, RenderAssetUsages, io::Reader},
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    reflect::TypePath,
};
use serde::Deserialize;

use crate::{model::*, rendering::team_color};

const SWORDSMAN_PATH: &str = "characters/swordsman.ron";
const ARCHER_PATH: &str = "characters/archer.ron";
const SWORD_PATH: &str = "characters/sword.ron";
const BOW_PATH: &str = "characters/bow.ron";

#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct StudioAsset {
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
    #[serde(rename = "name")]
    _name: String,
    scale: f32,
    skin_color: Rgba,
    team_color: Rgba,
    #[serde(default)]
    visuals: CharacterVisuals,
    #[serde(rename = "weapon")]
    _weapon: String,
    joints: Vec<JointDefinition>,
    animations: Vec<AnimationDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
struct CharacterVisuals {
    limb_width_scale: f32,
    head_size: (f32, f32),
    hand_size: (f32, f32),
    foot_size: (f32, f32),
    #[serde(rename = "rig_line_width")]
    _rig_line_width: f32,
    #[serde(rename = "rig_joint_radius")]
    _rig_joint_radius: f32,
}

impl Default for CharacterVisuals {
    fn default() -> Self {
        Self {
            limb_width_scale: 1.0,
            head_size: (25.0, 30.0),
            hand_size: (11.0, 14.0),
            foot_size: (19.0, 10.0),
            _rig_line_width: 2.0,
            _rig_joint_radius: 3.5,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct JointDefinition {
    name: String,
    parent: Option<String>,
    position: (f32, f32),
    angle_degrees: f32,
    #[serde(default)]
    length: Option<f32>,
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
    #[serde(rename = "name")]
    _name: String,
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
    #[serde(rename = "name")]
    _name: String,
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
pub struct StudioAssetLoader;

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
pub struct CharacterAssetHandles {
    swordsman: Handle<StudioAsset>,
    archer: Handle<StudioAsset>,
    sword: Handle<StudioAsset>,
    bow: Handle<StudioAsset>,
}

pub fn load_character_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CharacterAssetHandles {
        swordsman: asset_server.load(SWORDSMAN_PATH),
        archer: asset_server.load(ARCHER_PATH),
        sword: asset_server.load(SWORD_PATH),
        bow: asset_server.load(BOW_PATH),
    });
}

#[derive(Component)]
pub struct CharacterVisual {
    owner: Entity,
    character: Handle<StudioAsset>,
    weapon: Handle<StudioAsset>,
    animation: String,
    elapsed: f32,
}

type CharacterRootQuery<'world, 'state> = Query<
    'world,
    'state,
    (
        &'static Team,
        &'static UnitKind,
        Option<&'static CombatUnitState>,
        Option<&'static MotionEstimate>,
        Option<&'static Attack>,
    ),
>;

pub fn spawn_character_visuals(
    mut commands: Commands,
    handles: Res<CharacterAssetHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    units: Query<(Entity, &UnitKind), Added<Unit>>,
) {
    for (owner, kind) in &units {
        let (character, weapon) = match kind {
            UnitKind::Swordsman => (&handles.swordsman, &handles.sword),
            UnitKind::Archer => (&handles.archer, &handles.bow),
            UnitKind::Miner => continue,
        };
        let visual = commands
            .spawn((
                CharacterVisual {
                    owner,
                    character: character.clone(),
                    weapon: weapon.clone(),
                    animation: "idle".into(),
                    elapsed: 0.0,
                },
                Mesh2d(meshes.add(Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                ))),
                MeshMaterial2d(materials.add(Color::WHITE)),
                Transform::from_xyz(0.0, 0.0, 1.0),
            ))
            .id();
        commands.entity(owner).add_child(visual);
    }
}

pub fn animate_characters(
    time: Res<Time>,
    assets: Res<Assets<StudioAsset>>,
    roots: CharacterRootQuery,
    mut meshes: ResMut<Assets<Mesh>>,
    mut visuals: Query<(&mut CharacterVisual, &Mesh2d)>,
) {
    for (mut visual, mesh_handle) in &mut visuals {
        let Ok((team, kind, state, motion, attack)) = roots.get(visual.owner) else {
            continue;
        };
        let Some(character_asset) = assets.get(&visual.character) else {
            continue;
        };
        let Some(weapon_asset) = assets.get(&visual.weapon) else {
            continue;
        };
        if character_asset.kind != AssetKind::Character || weapon_asset.kind != AssetKind::Weapon {
            continue;
        }
        let (Some(character), Some(weapon)) = (
            character_asset.character.as_ref(),
            weapon_asset.weapon.as_ref(),
        ) else {
            continue;
        };

        let attacking = state.is_some_and(|state| *state == CombatUnitState::Attacking);
        let moving = state.is_some_and(|state| {
            matches!(state, CombatUnitState::Moving | CombatUnitState::Retreating)
        }) || motion.is_some_and(|motion| motion.velocity.x.abs() > 1.0);
        let requested = if attacking {
            match kind {
                UnitKind::Swordsman => "attack",
                UnitKind::Archer => "draw",
                UnitKind::Miner => "idle",
            }
        } else if moving {
            "walk"
        } else {
            "idle"
        };
        let animation = character
            .animations
            .iter()
            .find(|animation| animation.name == requested)
            .or_else(|| {
                character
                    .animations
                    .iter()
                    .find(|animation| animation.name == "idle")
            });
        let Some(animation) = animation else {
            continue;
        };
        if visual.animation != animation.name {
            visual.animation.clone_from(&animation.name);
            visual.elapsed = 0.0;
        }
        let attack_progress = attack.map_or(0.0, |attack| attack.cooldown.fraction());
        if attacking {
            visual.elapsed = animation_duration(animation) * attack_progress;
        } else {
            visual.elapsed += time.delta_secs();
        }
        let poses = evaluate_pose(character, animation, visual.elapsed, !attacking);
        let bow_draw = if attacking && matches!(kind, UnitKind::Archer) {
            attack_progress
        } else {
            0.0
        };
        if let Some(mut mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = build_character_mesh(character, weapon, &poses, *team, bow_draw);
        }
    }
}

fn animation_duration(animation: &AnimationDefinition) -> f32 {
    animation.seconds_per_frame.max(0.001) * animation.frames.len().max(1) as f32
}

#[derive(Clone, Copy)]
struct JointPose {
    start: Vec2,
    angle: f32,
    end: Vec2,
}

fn evaluate_pose(
    character: &CharacterDefinition,
    animation: &AnimationDefinition,
    elapsed: f32,
    looping: bool,
) -> HashMap<String, JointPose> {
    let frame_count = animation.frames.len().max(1);
    let frame_position = elapsed / animation.seconds_per_frame.max(0.001);
    let frame_a_index = if looping {
        frame_position.floor() as usize % frame_count
    } else {
        (frame_position.floor() as usize).min(frame_count - 1)
    };
    let frame_b_index = if looping {
        (frame_a_index + 1) % frame_count
    } else {
        (frame_a_index + 1).min(frame_count - 1)
    };
    let frame_a = animation.frames.get(frame_a_index);
    let frame_b = animation.frames.get(frame_b_index);
    let blend = frame_position.fract();
    let mut poses = HashMap::new();

    for joint in &character.joints {
        let (offset_a, angle_a) = keyed_pose(joint, frame_a);
        let (offset_b, angle_b) = keyed_pose(joint, frame_b);
        let local_offset = offset_a.lerp(offset_b, blend) * character.scale;
        let angle_a = angle_a.to_radians();
        let angle_b = angle_b.to_radians();
        let local_angle = angle_a + shortest_angle(angle_a, angle_b) * blend;
        let (parent_position, parent_angle) = joint
            .parent
            .as_ref()
            .and_then(|name| poses.get(name))
            .map_or((Vec2::new(0.0, -45.0), 0.0), |pose: &JointPose| {
                (pose.start, pose.angle)
            });
        let start = parent_position + rotate(local_offset, parent_angle);
        poses.insert(
            joint.name.clone(),
            JointPose {
                start,
                angle: parent_angle + local_angle,
                end: start,
            },
        );
    }

    for joint in &character.joints {
        let Some(pose) = poses.get(&joint.name).copied() else {
            continue;
        };
        let end = if let Some(length) = joint.length {
            pose.start + rotate(Vec2::Y * length * character.scale, pose.angle)
        } else {
            character
                .joints
                .iter()
                .find(|candidate| candidate.parent.as_deref() == Some(joint.name.as_str()))
                .and_then(|child| poses.get(&child.name))
                .map_or(pose.start, |child_pose| child_pose.start)
        };
        if let Some(pose) = poses.get_mut(&joint.name) {
            pose.end = end;
        }
    }
    poses
}

fn keyed_pose(joint: &JointDefinition, frame: Option<&Keyframe>) -> (Vec2, f32) {
    let key = frame.and_then(|frame| frame.joints.iter().find(|key| key.joint == joint.name));
    let offset = key.and_then(|key| key.position).unwrap_or(joint.position);
    let angle = joint.angle_degrees + key.and_then(|key| key.angle_degrees).unwrap_or(0.0);
    (Vec2::new(offset.0, offset.1), angle)
}

fn build_character_mesh(
    character: &CharacterDefinition,
    weapon: &WeaponDefinition,
    poses: &HashMap<String, JointPose>,
    team: Team,
    bow_draw: f32,
) -> Mesh {
    let mut builder = MeshBuilder::default();
    let direction = team.direction();
    for joint in &character.joints {
        let Some(pose) = poses.get(&joint.name) else {
            continue;
        };
        let color = match joint.color {
            JointColor::Skin => character.skin_color.color(),
            JointColor::Team => {
                if team == Team::Player {
                    character.team_color.color()
                } else {
                    team_color(team)
                }
            }
            JointColor::Dark => Color::srgb(0.08, 0.09, 0.12),
        };
        builder.capsule(
            mirror(pose.start, direction),
            mirror(pose.end, direction),
            joint.thickness * character.visuals.limb_width_scale * character.scale,
            color,
            0.0,
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
            builder.ellipse(
                mirror(pose.end, direction),
                Vec2::new(width, height) * character.scale * 0.5,
                pose.angle * direction,
                color,
                0.1,
            );
        }
    }

    if let Some(hand) = poses.get(&weapon.attach_joint) {
        add_weapon_mesh(
            &mut builder,
            weapon,
            hand,
            character.scale,
            direction,
            bow_draw,
        );
    }
    builder.finish()
}

fn add_weapon_mesh(
    builder: &mut MeshBuilder,
    weapon: &WeaponDefinition,
    hand: &JointPose,
    scale: f32,
    direction: f32,
    bow_draw: f32,
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
    let point = |point| mirror(point, direction);

    match weapon.state {
        WeaponKind::Static => {
            builder.capsule(
                point(position - axis * half),
                point(position + axis * half),
                (weapon.width + 3.0) * scale,
                Color::srgb(0.10, 0.12, 0.16),
                1.0,
            );
            builder.capsule(
                point(position - axis * half),
                point(position + axis * half),
                weapon.width * scale,
                color,
                1.1,
            );
            builder.capsule(
                point(position - side * 10.0 * scale),
                point(position + side * 10.0 * scale),
                5.0 * scale,
                Color::srgb(0.78, 0.57, 0.18),
                1.2,
            );
            builder.capsule(
                point(position - axis * half),
                point(position - axis * (half + 14.0 * scale)),
                6.0 * scale,
                Color::srgb(0.35, 0.19, 0.09),
                1.1,
            );
            builder.ellipse(
                point(position - axis * (half + 16.0 * scale)),
                Vec2::splat(5.0 * scale),
                angle * direction,
                Color::srgb(0.78, 0.57, 0.18),
                1.2,
            );
        }
        WeaponKind::Bow => {
            let top = position + axis * half;
            let bottom = position - axis * half;
            let belly = position + side * 12.0 * scale;
            for (width, stave_color, z) in [
                (weapon.width + 3.0, Color::srgb(0.12, 0.07, 0.035), 1.0),
                (weapon.width, color, 1.1),
            ] {
                builder.capsule(point(bottom), point(belly), width * scale, stave_color, z);
                builder.capsule(point(belly), point(top), width * scale, stave_color, z);
            }
            let configured_max = weapon
                .states
                .iter()
                .map(|state| state.draw)
                .fold(0.0_f32, f32::max)
                .max(1.0);
            let draw = bow_draw.clamp(0.0, 1.0) * configured_max;
            let nock = position - side * draw * 27.0 * scale;
            let string_color = Color::srgb(0.88, 0.82, 0.68);
            builder.capsule(point(top), point(nock), 1.25 * scale, string_color, 1.2);
            builder.capsule(point(nock), point(bottom), 1.25 * scale, string_color, 1.2);
            if draw > 0.05 {
                let arrow_tip = nock + side * (62.0 + draw * 18.0) * scale;
                builder.capsule(
                    point(nock),
                    point(arrow_tip),
                    2.0 * scale,
                    Color::srgb(0.62, 0.42, 0.2),
                    1.3,
                );
            }
        }
    }
}

#[derive(Default)]
struct MeshBuilder {
    positions: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    fn capsule(&mut self, start: Vec2, end: Vec2, width: f32, color: Color, z: f32) {
        let delta = end - start;
        if delta.length_squared() <= f32::EPSILON {
            self.ellipse(start, Vec2::splat(width * 0.5), 0.0, color, z);
            return;
        }
        let side = Vec2::new(-delta.y, delta.x).normalize() * width * 0.5;
        self.quad(
            [start - side, start + side, end + side, end - side],
            color,
            z,
        );
        self.ellipse(start, Vec2::splat(width * 0.5), 0.0, color, z + 0.01);
        self.ellipse(end, Vec2::splat(width * 0.5), 0.0, color, z + 0.01);
    }

    fn quad(&mut self, points: [Vec2; 4], color: Color, z: f32) {
        let base = self.positions.len() as u32;
        let color = LinearRgba::from(color).to_f32_array();
        self.positions
            .extend(points.map(|point| [point.x, point.y, z]));
        self.colors.extend([color; 4]);
        self.indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    fn ellipse(&mut self, center: Vec2, radii: Vec2, angle: f32, color: Color, z: f32) {
        const SEGMENTS: usize = 12;
        let base = self.positions.len() as u32;
        let color = LinearRgba::from(color).to_f32_array();
        self.positions.push([center.x, center.y, z]);
        self.colors.push(color);
        for index in 0..SEGMENTS {
            let theta = index as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            let point = center
                + rotate(
                    Vec2::new(theta.cos() * radii.x, theta.sin() * radii.y),
                    angle,
                );
            self.positions.push([point.x, point.y, z]);
            self.colors.push(color);
        }
        for index in 0..SEGMENTS {
            self.indices.extend([
                base,
                base + 1 + index as u32,
                base + 1 + ((index + 1) % SEGMENTS) as u32,
            ]);
        }
    }

    fn finish(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

fn mirror(point: Vec2, direction: f32) -> Vec2 {
    Vec2::new(point.x * direction, point.y)
}

fn rotate(vector: Vec2, angle: f32) -> Vec2 {
    Vec2::new(
        vector.x * angle.cos() - vector.y * angle.sin(),
        vector.x * angle.sin() + vector.y * angle.cos(),
    )
}

fn shortest_angle(from: f32, to: f32) -> f32 {
    (to - from + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortest_angle_wraps_across_zero() {
        let delta = shortest_angle(350.0_f32.to_radians(), 10.0_f32.to_radians());
        assert!((delta.to_degrees() - 20.0).abs() < 0.001);
    }

    #[test]
    fn mirrored_points_keep_height() {
        assert_eq!(mirror(Vec2::new(12.0, 7.0), -1.0), Vec2::new(-12.0, 7.0));
    }

    #[test]
    fn runtime_character_assets_match_the_editor_schema() {
        for source in [
            include_str!("../config/characters/swordsman.ron"),
            include_str!("../config/characters/archer.ron"),
            include_str!("../config/characters/sword.ron"),
            include_str!("../config/characters/bow.ron"),
        ] {
            ron::from_str::<StudioAsset>(source)
                .expect("runtime character asset should deserialize");
        }
    }
}
