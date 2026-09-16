import * as THREE from "three";
import { clamp, type UnitState } from "../simulation/types";

// Blender attack clips put contact/release at 35%. Retiming that marker to
// the simulation's captured windup keeps every balance preset visually honest.
export function attackClipProgress(
  unit: UnitState,
  time: number,
): number | null {
  const motion = unit.attackMotion;
  if (!motion || time >= motion.startedAt + motion.duration) return null;
  const elapsed = Math.max(0, time - motion.startedAt);
  return elapsed <= motion.windup && motion.windup > 0
    ? (0.35 * elapsed) / motion.windup
    : 0.35 +
        0.65 *
          clamp(
            (elapsed - motion.windup) /
              Math.max(0.001, motion.duration - motion.windup),
            0,
            1,
          );
}

interface AnimationLayer {
  actions: Map<string, THREE.AnimationAction>;
  active: string;
  lowerBody: boolean;
}

/** Playback only: no animation event mutates the battle. */
export class UnitAnimator {
  readonly mixer: THREE.AnimationMixer;
  private layers: AnimationLayer[] = [];
  constructor(
    private root: THREE.Object3D,
    clips: THREE.AnimationClip[],
  ) {
    this.mixer = new THREE.AnimationMixer(root);
    const split = !!root.getObjectByName("thigh_l");
    const isLeg = (track: THREE.KeyframeTrack) =>
      /^(root|pelvis|thigh_[lr]|shin_[lr]|foot_[lr])\./.test(track.name);
    for (const lowerBody of split ? [true, false] : [false]) {
      const layer: AnimationLayer = {
        actions: new Map(),
        active: "",
        lowerBody,
      };
      for (const source of clips) {
        const clip = split
          ? new THREE.AnimationClip(
              source.name + (lowerBody ? ":legs" : ":upper"),
              source.duration,
              source.tracks.filter((track) => isLeg(track) === lowerBody),
            )
          : source;
        const action = this.mixer.clipAction(clip);
        if (source.name.endsWith("_attack") || source.name === "hit") {
          action.setLoop(THREE.LoopOnce, 1);
          action.clampWhenFinished = true;
        }
        layer.actions.set(source.name, action);
      }
      this.layers.push(layer);
    }
  }
  update(unit: UnitState, time: number, dt: number, paused: boolean): void {
    const attack = attackClipProgress(unit, time);
    const speed = Math.hypot(unit.velocity.x, unit.velocity.z);
    const locomotion =
      speed > 0.08 ? (unit.carried > 0 ? "carry" : "walk") : null;
    const desired =
      attack !== null
        ? unit.kind === "archer"
          ? "bow_attack"
          : "melee_attack"
        : unit.phase === "mine"
          ? "mine"
          : (locomotion ?? (unit.hitRemaining > 0 ? "hit" : "idle"));
    const backwards =
      unit.velocity.x * Math.sin(unit.yaw) +
        unit.velocity.z * Math.cos(unit.yaw) <
      -0.03;
    let legTime: number | undefined;
    for (const layer of this.layers) {
      const requested = layer.lowerBody && locomotion ? locomotion : desired;
      const name = layer.actions.has(requested) ? requested : "idle";
      const action = layer.actions.get(name);
      if (!action) continue;
      const striking = attack !== null && name.endsWith("_attack");
      if (name !== layer.active) {
        const previous = layer.actions.get(layer.active);
        previous?.fadeOut(0.1);
        action.reset().setEffectiveWeight(1).play();
        if (previous && !paused) action.fadeIn(striking ? 0.06 : 0.1);
        else action.stopFading();
        layer.active = name;
      }
      action.paused = striking;
      if (striking) action.time = attack * action.getClip().duration;
      else if (name === "walk" || name === "carry") {
        if (!layer.lowerBody && legTime !== undefined) action.time = legTime;
        action.setEffectiveTimeScale(
          clamp(speed / 1.7, 0.35, 1.8) * (backwards ? -1 : 1),
        );
        if (layer.lowerBody) legTime = action.time;
      } else action.setEffectiveTimeScale(1);
    }
    this.mixer.update(paused ? 0 : dt);
  }
  dispose(): void {
    this.mixer.stopAllAction();
    this.mixer.uncacheRoot(this.root);
  }
}
