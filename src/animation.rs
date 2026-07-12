use crate::{
    model::*,
    rendering::{LightingLayer, TimeOfDayTint},
};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use std::collections::HashMap;
use std::time::Duration;

pub const CHARACTER_FRAME_SIZE: UVec2 = UVec2::splat(384);
pub const CHARACTER_WORLD_SIZE: Vec2 = Vec2::splat(184.0);

/// Animation runtime for high-resolution character sprite sheets.
///
/// Each action is authored as its own horizontal strip. Keeping clips separate
/// makes it possible to replace or regenerate one action without repacking every
/// character atlas, while the runtime still presents one consistent state
/// machine to gameplay systems.
pub struct CharacterAnimationPlugin;

impl Plugin for CharacterAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CharacterAnimationEvent>()
            .add_systems(Startup, load_sprite_sheet_library);
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CharacterAnimation {
    Idle,
    Walk,
    WalkLoaded,
    Mine,
    Backpedal,
    Attack,
    Hit,
    Death,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlaybackMode {
    Loop,
    Once,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CharacterAnimationEventKind {
    MiningContact,
    MeleeContact,
    ProjectileRelease,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct CharacterAnimationEvent {
    pub entity: Entity,
    pub kind: CharacterAnimationEventKind,
}

#[derive(Debug, Clone, Copy)]
pub struct AnimationClip {
    pub frames: usize,
    pub frames_per_second: f32,
    pub playback: PlaybackMode,
    pub event: Option<(usize, CharacterAnimationEventKind)>,
}

impl AnimationClip {
    fn frame_duration(self) -> Duration {
        Duration::from_secs_f32(1.0 / self.frames_per_second.max(1.0))
    }
}

/// The production frame plan used by both Bevy and the asset-generation tools.
/// Frames are large, antialiased raster cells intended to preserve the curves
/// and ink variation of vector-like source art.
pub fn clip_for(kind: UnitKind, animation: CharacterAnimation) -> AnimationClip {
    use CharacterAnimation as A;
    use CharacterAnimationEventKind as E;
    use PlaybackMode as P;

    match (kind, animation) {
        (_, A::Idle) => AnimationClip {
            frames: 8,
            frames_per_second: 12.0,
            playback: P::Loop,
            event: None,
        },
        (_, A::Walk) => AnimationClip {
            frames: 10,
            frames_per_second: 16.0,
            playback: P::Loop,
            event: None,
        },
        (UnitKind::Miner, A::WalkLoaded) => AnimationClip {
            frames: 12,
            frames_per_second: 16.0,
            playback: P::Loop,
            event: None,
        },
        (UnitKind::Miner, A::Mine) => AnimationClip {
            frames: 12,
            frames_per_second: 16.0,
            playback: P::Loop,
            event: Some((7, E::MiningContact)),
        },
        (_, A::Backpedal) => AnimationClip {
            frames: 10,
            frames_per_second: 14.0,
            playback: P::Loop,
            event: None,
        },
        (UnitKind::Swordsman, A::Attack) => AnimationClip {
            frames: 12,
            frames_per_second: 20.0,
            playback: P::Once,
            event: Some((7, E::MeleeContact)),
        },
        (UnitKind::Archer, A::Attack) => AnimationClip {
            frames: 14,
            frames_per_second: 20.0,
            playback: P::Once,
            event: Some((9, E::ProjectileRelease)),
        },
        (_, A::Hit) => AnimationClip {
            frames: 6,
            frames_per_second: 20.0,
            playback: P::Once,
            event: None,
        },
        (_, A::Death) => AnimationClip {
            frames: 12,
            frames_per_second: 16.0,
            playback: P::Once,
            event: None,
        },
        // Unsupported combinations should never be selected, but returning a
        // stable idle clip keeps tooling and debug builds resilient.
        _ => clip_for(kind, A::Idle),
    }
}

#[derive(Component, Debug)]
pub struct CharacterAnimator {
    pub animation: CharacterAnimation,
    pub frame: usize,
    pub finished: bool,
    frame_timer: Timer,
    event_emitted: bool,
}

impl CharacterAnimator {
    pub fn new(kind: UnitKind) -> Self {
        let clip = clip_for(kind, CharacterAnimation::Idle);
        Self {
            animation: CharacterAnimation::Idle,
            frame: 0,
            finished: false,
            frame_timer: Timer::new(clip.frame_duration(), TimerMode::Once),
            event_emitted: false,
        }
    }

    pub fn play(&mut self, kind: UnitKind, animation: CharacterAnimation) {
        if self.animation == animation && !self.finished {
            return;
        }
        let clip = clip_for(kind, animation);
        self.animation = animation;
        self.frame = 0;
        self.finished = false;
        self.event_emitted = false;
        self.frame_timer = Timer::new(clip.frame_duration(), TimerMode::Once);
    }

    pub fn is_locked(&self) -> bool {
        !self.finished
            && matches!(
                self.animation,
                CharacterAnimation::Attack | CharacterAnimation::Hit | CharacterAnimation::Death
            )
    }
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub enum CharacterFacing {
    Left,
    Right,
}

impl CharacterFacing {
    pub fn for_team(team: Team) -> Self {
        if team.direction() > 0.0 {
            Self::Right
        } else {
            Self::Left
        }
    }
}

#[derive(Component)]
pub struct CharacterSprite {
    applied_animation: CharacterAnimation,
}

impl CharacterSprite {
    pub fn new(animation: CharacterAnimation) -> Self {
        Self {
            applied_animation: animation,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct SpriteSheetKey {
    team: Team,
    kind: UnitKind,
    animation: CharacterAnimation,
}

#[derive(Clone)]
pub struct SpriteSheetClip {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource, Default)]
pub struct SpriteSheetLibrary {
    clips: HashMap<SpriteSheetKey, SpriteSheetClip>,
}

impl SpriteSheetLibrary {
    pub fn clip(
        &self,
        team: Team,
        kind: UnitKind,
        animation: CharacterAnimation,
    ) -> Option<&SpriteSheetClip> {
        self.clips.get(&SpriteSheetKey {
            team,
            kind,
            animation,
        })
    }
}

fn team_name(team: Team) -> &'static str {
    match team {
        Team::Player => "blue",
        Team::Enemy => "red",
    }
}

fn kind_name(kind: UnitKind) -> &'static str {
    match kind {
        UnitKind::Miner => "miner",
        UnitKind::Swordsman => "swordsman",
        UnitKind::Archer => "archer",
    }
}

fn animation_name(animation: CharacterAnimation) -> &'static str {
    match animation {
        CharacterAnimation::Idle => "idle",
        CharacterAnimation::Walk => "walk",
        CharacterAnimation::WalkLoaded => "walk-loaded",
        CharacterAnimation::Mine => "mine",
        CharacterAnimation::Backpedal => "backpedal",
        CharacterAnimation::Attack => "attack",
        CharacterAnimation::Hit => "hit",
        CharacterAnimation::Death => "death",
    }
}

fn supported_animations(kind: UnitKind) -> &'static [CharacterAnimation] {
    use CharacterAnimation as A;
    match kind {
        UnitKind::Miner => &[A::Idle, A::Walk, A::WalkLoaded, A::Mine, A::Hit, A::Death],
        UnitKind::Swordsman | UnitKind::Archer => {
            &[A::Idle, A::Walk, A::Backpedal, A::Attack, A::Hit, A::Death]
        }
    }
}

fn sprite_sheet_path(team: Team, kind: UnitKind, animation: CharacterAnimation) -> String {
    format!(
        "characters/{}/{}/{}.png",
        team_name(team),
        kind_name(kind),
        animation_name(animation)
    )
}

fn load_sprite_sheet_library(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut library = SpriteSheetLibrary::default();
    for team in [Team::Player, Team::Enemy] {
        for kind in [UnitKind::Miner, UnitKind::Swordsman, UnitKind::Archer] {
            for animation in supported_animations(kind) {
                let spec = clip_for(kind, *animation);
                let image = asset_server.load(sprite_sheet_path(team, kind, *animation));
                let layout = layouts.add(TextureAtlasLayout::from_grid(
                    CHARACTER_FRAME_SIZE,
                    spec.frames as u32,
                    1,
                    None,
                    None,
                ));
                library.clips.insert(
                    SpriteSheetKey {
                        team,
                        kind,
                        animation: *animation,
                    },
                    SpriteSheetClip { image, layout },
                );
            }
        }
    }
    commands.insert_resource(library);
}

pub(crate) fn select_character_animations(
    mut units: Query<(
        &UnitKind,
        Option<&MinerState>,
        Option<&CombatUnitState>,
        Option<&CarriedGold>,
        Option<&Dying>,
        Option<&PendingAttack>,
        &mut CharacterAnimator,
    )>,
) {
    for (kind, miner, combat, carried, dying, pending_attack, mut animator) in &mut units {
        let desired = if dying.is_some() {
            CharacterAnimation::Death
        } else if pending_attack.is_some() {
            CharacterAnimation::Attack
        } else if animator.is_locked() {
            continue;
        } else if let Some(state) = miner {
            match state {
                MinerState::Mining => CharacterAnimation::Mine,
                MinerState::Returning if carried.is_some_and(|gold| gold.0 > 0) => {
                    CharacterAnimation::WalkLoaded
                }
                MinerState::GoingToMine | MinerState::Returning => CharacterAnimation::Walk,
            }
        } else {
            match combat.copied().unwrap_or(CombatUnitState::Idle) {
                CombatUnitState::Idle => CharacterAnimation::Idle,
                CombatUnitState::Moving => CharacterAnimation::Walk,
                CombatUnitState::Attacking => CharacterAnimation::Idle,
                CombatUnitState::Retreating => CharacterAnimation::Backpedal,
            }
        };

        animator.play(*kind, desired);
    }
}

pub(crate) fn attach_character_sprites(
    mut commands: Commands,
    library: Res<SpriteSheetLibrary>,
    units: Query<(Entity, &Team, &UnitKind), Added<CharacterAnimator>>,
) {
    for (entity, team, kind) in &units {
        let animation = CharacterAnimation::Idle;
        let Some(clip) = library.clip(*team, *kind, animation) else {
            continue;
        };
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                CharacterSprite::new(animation),
                TimeOfDayTint::new(Color::WHITE, LightingLayer::Foreground),
                Sprite {
                    image: clip.image.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: clip.layout.clone(),
                        index: 0,
                    }),
                    custom_size: Some(CHARACTER_WORLD_SIZE),
                    ..default()
                },
                Anchor::BOTTOM_CENTER,
                Transform::from_xyz(0.0, -10.0, 2.0),
            ));
        });
    }
}

pub(crate) fn update_character_facing(
    targets: Query<&Transform, Or<(With<Unit>, With<Statue>)>>,
    mut units: Query<
        (
            &Team,
            &Transform,
            &MotionEstimate,
            Option<&CurrentTarget>,
            Has<Controlled>,
            &mut CharacterFacing,
        ),
        With<Unit>,
    >,
) {
    for (team, transform, motion, target, controlled, mut facing) in &mut units {
        let direction = target
            .and_then(|target| targets.get(target.0).ok())
            .map(|target| target.translation.x - transform.translation.x)
            .filter(|delta| delta.abs() > 0.5)
            .or_else(|| {
                if controlled && motion.velocity.x.abs() > 1.0 {
                    Some(motion.velocity.x)
                } else {
                    // Retreat and autonomous backpedal keep the unit facing the
                    // enemy rather than snapping toward its travel direction.
                    Some(team.direction())
                }
            })
            .unwrap_or_else(|| team.direction());
        *facing = if direction >= 0.0 {
            CharacterFacing::Right
        } else {
            CharacterFacing::Left
        };
    }
}

pub(crate) fn advance_character_animations(
    time: Res<Time>,
    mut events: MessageWriter<CharacterAnimationEvent>,
    mut animators: Query<(Entity, &UnitKind, &mut CharacterAnimator)>,
) {
    for (entity, kind, mut animator) in &mut animators {
        if animator.finished {
            continue;
        }
        let clip = clip_for(*kind, animator.animation);
        animator.frame_timer.tick(time.delta());
        if !animator.frame_timer.just_finished() {
            continue;
        }

        if animator.frame + 1 >= clip.frames {
            match clip.playback {
                PlaybackMode::Loop => animator.frame = 0,
                PlaybackMode::Once => animator.finished = true,
            }
        } else {
            animator.frame += 1;
        }

        if let Some((event_frame, kind)) = clip.event
            && animator.frame == event_frame
            && !animator.event_emitted
        {
            events.write(CharacterAnimationEvent { entity, kind });
            animator.event_emitted = true;
        }
        if animator.frame == 0 {
            animator.event_emitted = false;
        }
        animator.frame_timer = Timer::new(clip.frame_duration(), TimerMode::Once);
    }
}

pub(crate) fn apply_animation_frame_to_sprites(
    library: Res<SpriteSheetLibrary>,
    animators: Query<(&Team, &UnitKind, &CharacterAnimator, &CharacterFacing), With<Unit>>,
    mut sprites: Query<(&ChildOf, &mut CharacterSprite, &mut Sprite)>,
) {
    for (parent, mut character_sprite, mut sprite) in &mut sprites {
        let Ok((team, kind, animator, facing)) = animators.get(parent.parent()) else {
            continue;
        };
        if character_sprite.applied_animation != animator.animation
            && let Some(clip) = library.clip(*team, *kind, animator.animation)
        {
            sprite.image = clip.image.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: clip.layout.clone(),
                index: animator.frame,
            });
            character_sprite.applied_animation = animator.animation;
        }
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = animator.frame;
        }
        sprite.flip_x = *facing == CharacterFacing::Left;
    }
}

pub(crate) fn despawn_finished_characters(
    mut commands: Commands,
    dying: Query<(Entity, &CharacterAnimator), With<Dying>>,
) {
    for (entity, animator) in &dying {
        if animator.animation == CharacterAnimation::Death && animator.finished {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attack_events_land_after_a_visible_windup() {
        let sword = clip_for(UnitKind::Swordsman, CharacterAnimation::Attack);
        let archer = clip_for(UnitKind::Archer, CharacterAnimation::Attack);
        assert!(matches!(
            sword.event,
            Some((7, CharacterAnimationEventKind::MeleeContact))
        ));
        assert!(matches!(
            archer.event,
            Some((9, CharacterAnimationEventKind::ProjectileRelease))
        ));
        assert!(sword.frames > 7);
        assert!(archer.frames > 9);
    }

    #[test]
    fn loaded_miners_have_a_distinct_walk_cycle() {
        let loaded = clip_for(UnitKind::Miner, CharacterAnimation::WalkLoaded);
        let unladen = clip_for(UnitKind::Miner, CharacterAnimation::Walk);
        assert_ne!(loaded.frames, unladen.frames);
    }

    #[test]
    fn sprite_sheet_paths_are_team_and_clip_specific() {
        assert_eq!(
            sprite_sheet_path(
                Team::Player,
                UnitKind::Miner,
                CharacterAnimation::WalkLoaded
            ),
            "characters/blue/miner/walk-loaded.png"
        );
        assert_eq!(
            sprite_sheet_path(Team::Enemy, UnitKind::Archer, CharacterAnimation::Attack),
            "characters/red/archer/attack.png"
        );
    }
}
