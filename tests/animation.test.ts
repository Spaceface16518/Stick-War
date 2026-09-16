import { describe, expect, it } from "vitest";
import * as THREE from "three";
import {
  UnitAnimator,
  attackClipProgress,
} from "../src/presentation/animation";
import type { UnitState } from "../src/simulation/types";
const unit: UnitState = {
  id: 1,
  team: "blue",
  kind: "archer",
  x: 0,
  z: 0,
  previous: { x: 0, z: 0 },
  health: 70,
  maxHealth: 70,
  yaw: 0,
  cooldown: 0.8,
  attack: null,
  attackMotion: { startedAt: 0, duration: 1, windup: 0.2 },
  phase: "attack",
  carried: 0,
  miningRemaining: 0,
  minerState: "outbound",
  target: 2,
  velocity: { x: 0, z: 1.7 },
  hitRemaining: 0,
};
describe("animation follows simulation", () => {
  it("puts contact at the captured windup and preserves the full recovery", () => {
    expect(attackClipProgress(unit, 0.2)).toBeCloseTo(0.35);
    expect(attackClipProgress(unit, 0.6)).toBeCloseTo(0.675);
    expect(attackClipProgress(unit, 1)).toBeNull();
    expect(attackClipProgress({ ...unit, attackMotion: null }, 0.1)).toBeNull();
  });
  it("keeps stepping underneath the bow action and freezes both layers on pause", () => {
    const root = new THREE.Group();
    const hand = new THREE.Bone(),
      leg = new THREE.Bone();
    hand.name = "hand_l";
    leg.name = "thigh_l";
    root.add(hand, leg);
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
      ]),
      new THREE.AnimationClip("walk", 1, [
        track("hand_l", 0),
        track("thigh_l", 1),
      ]),
      new THREE.AnimationClip("bow_attack", 1, [
        track("hand_l", 2),
        track("thigh_l", 0),
      ]),
    ];
    const animator = new UnitAnimator(root, clips);
    animator.update(unit, 0.2, 0.1, false);
    expect(hand.position.x).toBeCloseTo(0.7);
    expect(leg.position.x).toBeCloseTo(0.1);
    animator.update(unit, 0.2, 0.5, true);
    expect(hand.position.x).toBeCloseTo(0.7);
    expect(leg.position.x).toBeCloseTo(0.1);
    animator.dispose();
  });
});
