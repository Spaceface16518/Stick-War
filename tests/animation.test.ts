import { describe, expect, it } from "vitest";
import * as THREE from "three";
import {
  UnitAnimator,
  attackClipProgress,
  deathClip,
} from "../src/presentation/animation";
import type { UnitState } from "../src/simulation/types";
const unit: UnitState = {
  id: 1,
  team: "blue",
  kind: "archer",
  x: 0,
  z: 0.02,
  previous: { x: 0, z: 0 },
  health: 70,
  maxHealth: 70,
  yaw: 0,
  cooldown: 0.8,
  attack: null,
  attackMotion: {
    clip: "bow_draw",
    contact: 0.35,
    startedAt: 0,
    duration: 1,
    windup: 0.2,
  },
  phase: "attack",
  carried: 0,
  miningRemaining: 0,
  minerState: "outbound",
  target: 2,
  velocity: { x: 0, z: 1.7 },
  hitRemaining: 0,
  hitMotion: null,
  distanceTravelled: 0.02,
};
function animatedRig() {
  const root = new THREE.Group();
  const bones = Object.fromEntries(
    ["hand_l", "thigh_l", "spine"].map((name) => {
      const bone = new THREE.Bone();
      bone.name = name;
      root.add(bone);
      return [name, bone];
    }),
  );
  const track = (name: string, finish: number) =>
    new THREE.VectorKeyframeTrack(
      name + ".position",
      [0, 1],
      [0, 0, 0, finish, 0, 0],
    );
  const clips = [
    new THREE.AnimationClip("idle", 1, [
      track("hand_l", 0),
      track("thigh_l", 0),
      track("spine", 0),
    ]),
    new THREE.AnimationClip("walk", 1, [
      track("hand_l", 0),
      track("thigh_l", 1),
      track("spine", 0),
    ]),
    new THREE.AnimationClip("bow_draw", 1, [
      track("hand_l", 2),
      track("thigh_l", 0),
      track("spine", 0),
    ]),
    new THREE.AnimationClip("hit_front", 0.42, [
      new THREE.VectorKeyframeTrack(
        "spine.position",
        [0, 0.1, 0.42],
        [0, 0, 0, 0.12, 0, 0, 0, 0, 0],
      ),
    ]),
  ];
  return { animator: new UnitAnimator(root, clips), bones };
}
describe("animation follows simulation", () => {
  it("puts contact at the captured windup and preserves the full recovery", () => {
    expect(attackClipProgress(unit, 0.2)).toBeCloseTo(0.35);
    expect(attackClipProgress(unit, 0.6)).toBeCloseTo(0.675);
    expect(attackClipProgress(unit, 1)).toBeNull();
    expect(attackClipProgress({ ...unit, attackMotion: null }, 0.1)).toBeNull();
    expect(
      attackClipProgress(
        {
          ...unit,
          attackMotion: {
            ...unit.attackMotion!,
            contact: 0.54,
            windup: 0.6,
            duration: 1.4,
          },
        },
        0.6,
      ),
    ).toBeCloseTo(0.54);
  });
  it("keeps distance-driven footsteps underneath the bow action and freezes on pause", () => {
    const { animator, bones } = animatedRig();
    animator.update(unit, 0.2, 0.1, false);
    const phase = animator.locomotionPhase;
    expect(bones.hand_l.position.x).toBeCloseTo(0.7);
    expect(bones.thigh_l.position.x).toBeCloseTo(phase);
    animator.update(unit, 0.2, 0.5, true);
    expect(bones.hand_l.position.x).toBeCloseTo(0.7);
    expect(bones.thigh_l.position.x).toBeCloseTo(phase);
    animator.update(
      { ...unit, z: 0.32, previous: { x: 0, z: 0.3 }, distanceTravelled: 0.32 },
      0.3,
      0.1,
      false,
    );
    expect(animator.locomotionPhase).toBeCloseTo((phase + 0.3 / 1.2) % 1);
    animator.dispose();
  });
  it("stops advancing the gait when a collision blocks movement despite lingering smoothed velocity", () => {
    const { animator } = animatedRig();
    animator.update(unit, 0.2, 0.1, false);
    const phase = animator.locomotionPhase;
    animator.update({ ...unit, previous: { x: 0, z: 0.02 } }, 0.3, 0.1, false);
    expect(animator.locomotionPhase).toBe(phase);
    animator.dispose();
  });
  it("layers directional recoil over a moving attack without altering its contact pose", () => {
    const { animator, bones } = animatedRig();
    const hit = {
      ...unit,
      hitMotion: {
        startedAt: 0.1,
        duration: 0.42,
        direction: { x: 0, z: -1 },
        strength: 0.5,
      },
    };
    animator.update(hit, 0.2, 0.1, false);
    expect(bones.hand_l.position.x).toBeCloseTo(0.7);
    expect(bones.spine.position.x).toBeCloseTo(0.06);
    expect(bones.thigh_l.position.x).toBeCloseTo(animator.locomotionPhase);
    animator.update(hit, 0.53, 0.33, false);
    expect(bones.spine.position.x).toBeCloseTo(0);
    animator.dispose();
  });
  it("chooses falls from impact direction relative to the unit, for either team", () => {
    const hit = (yaw: number, x: number, z: number) =>
      deathClip({
        ...unit,
        yaw,
        hitMotion: {
          startedAt: 0,
          duration: 0.42,
          direction: { x, z },
          strength: 1,
        },
      });
    expect(hit(0, 0, -1)).toBe("death_back");
    expect(hit(Math.PI, 0, -1)).toBe("death_front");
    expect(hit(Math.PI / 2, 0, -1)).toBe("death_left");
    expect(hit(-Math.PI / 2, 0, -1)).toBe("death_right");
  });
});
